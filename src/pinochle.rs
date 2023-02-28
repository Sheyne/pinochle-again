// use super::ai::Bot;
use enum_iterator::{all, cardinality, next_cycle, previous_cycle, Sequence};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Sequence, Serialize, Deserialize, Hash,
)]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace,
}

impl Rank {
    fn points(self) -> i32 {
        match self {
            Rank::Nine => 0,
            Rank::Jack => 0,
            Rank::Queen => 5,
            Rank::King => 5,
            Rank::Ten => 10,
            Rank::Ace => 10,
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Nine => "9",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ten => "T",
                Rank::Ace => "A",
            }
        )
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Sequence, Serialize, Deserialize, Hash,
)]
pub enum Suit {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
                Suit::Hearts => "♥",
                Suit::Spades => "♠",
            }
        )
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Sequence, Serialize, Deserialize, Hash,
)]
pub struct Card(pub Suit, pub Rank);

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.1, self.0)
    }
}

fn shuffled<R>(rng: &mut R) -> Vec<Card>
where
    R: Rng + ?Sized,
{
    let mut deck = all::<Card>().collect::<Vec<_>>();
    deck.extend(all::<Card>());
    deck.shuffle(rng);
    deck
}

fn dealt(mut deck: Vec<Card>) -> [Vec<Card>; 4] {
    let len = deck.len();
    let range = 0..len / 4;
    [
        deck.drain(range.clone()).collect(),
        deck.drain(range.clone()).collect(),
        deck.drain(range.clone()).collect(),
        deck.drain(range).collect(),
    ]
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Game<R: Rng> {
    rng: R,
    hand: RoundState,
    scores: [i32; 2],
    current_player: Player,
}

impl Default for Game<ThreadRng> {
    fn default() -> Self {
        Self::new(thread_rng())
    }
}

impl<R: Rng> Game<R> {
    pub fn new(mut rng: R) -> Self {
        Self {
            hand: RoundState::start(&mut rng, Player::A),
            rng,
            current_player: Player::A,
            scores: [0; 2],
        }
    }

    pub fn act(&mut self, player: Player, action: Action) -> Result<(), Error> {
        if let Phase::ReviewingRevealedCards { .. } = self.hand.phase {
            self.hand.current_player = player;
        }
        if player != self.hand.current_player {
            return Err(Error::NotTheCurrentPlayer);
        }
        let result = self.hand.act(action)?;
        if let Some((a, b)) = result {
            self.scores[0] += a;
            self.scores[1] += b;
            self.current_player = next_cycle(&self.current_player).unwrap();
            self.hand = RoundState::start(&mut self.rng, self.current_player);
        }
        Ok(())
    }

    pub fn player_hand(&self, player: Player) -> Vec<Card> {
        self.hand.hands[player as usize].clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameInfo {
    first_bidder: Player,
    current_player: Player,
    phase: Phase,
    scores: [i32; 2],
}

impl<R: Rng> From<&Game<R>> for GameInfo {
    fn from(value: &Game<R>) -> Self {
        GameInfo {
            first_bidder: value.current_player,
            current_player: value.hand.current_player,
            phase: value.hand.phase.clone(),
            scores: value.scores,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RoundState {
    current_player: Player,
    hands: [Vec<Card>; 4],
    phase: Phase,
    // bots: [Option<Bot>; 3],
}

impl Display for RoundState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for hand in self.hands.iter() {
            let mut hand = hand.clone();
            hand.sort();
            for card in hand {
                write!(f, "{card} ")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl RoundState {
    pub fn start<R>(rng: &mut R, player: Player) -> Self
    where
        R: Rng + ?Sized,
    {
        Self {
            current_player: player,
            hands: dealt(shuffled(rng)),
            phase: Phase::Bidding {
                first_bidder: player,
                bids: vec![],
            },
            // bots: Default::default(),
        }
    }
}

fn is_legal_play(pile: &[Card], hand: &[Card], card: Card, trump: Suit) -> bool {
    // if there isn't a card played, anything is legal
    if let Some(first_card) = pile.get(0) {
        let starting_suit = first_card.0;
        // if the card doesn't match the starting suit
        let suitwise_legal = if card.0 != starting_suit {
            // you'd better not have any of the starting suit
            if hand.iter().any(|c| c.0 == starting_suit) {
                false
            } else {
                // and if you didn't play trump, you'd better have none of those either
                card.0 == trump || !hand.iter().any(|c| c.0 == trump)
            }
        } else {
            true
        };
        if suitwise_legal {
            if card.0 == starting_suit {
                let max_of_lead = *pile
                    .iter()
                    .filter(|x| x.0 == starting_suit)
                    .max_by_key(|x| x.1)
                    .expect("There's at least one card");
                // if the card is of the starting suit, but it doesn't beat the best card so far
                if card.1 <= max_of_lead.1 {
                    // then you better not have any cards that can beat the best card so far
                    !hand
                        .iter()
                        .any(|c| c.0 == starting_suit && c.1 > max_of_lead.1)
                } else {
                    true
                }
            } else if card.0 == trump {
                if let Some(max_of_trump) = pile.iter().filter(|x| x.0 == trump).max_by_key(|x| x.1)
                {
                    // if the card is trump, but it doesn't beat the best trump so far
                    if card.1 <= max_of_trump.1 {
                        // then you better not have any cards that can beat the best trump so far
                        !hand.iter().any(|c| c.0 == trump && c.1 > max_of_trump.1)
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    } else {
        true
    }
}

#[test]
fn test_is_legal_to_play() {
    use Player::*;
    use Rank::*;
    use Suit::*;

    let card = Card(Clubs, King);
    let trick = Trick {
        first_player: A,
        cards: vec![Card(Clubs, Ten)],
    };
    let hand = vec![
        Card(Spades, Nine),
        Card(Spades, Nine),
        Card(Diamonds, Nine),
        Card(Clubs, Queen),
        Card(Spades, King),
        Card(Clubs, Ten),
        Card(Spades, Jack),
        Card(Spades, Ten),
        Card(Clubs, King),
        Card(Clubs, Nine),
        Card(Hearts, Ace),
    ];

    assert!(is_legal_play(&trick.cards, &hand, card, Spades));
    assert!(is_legal_play(&trick.cards, &hand, card, Diamonds));
    assert!(is_legal_play(&trick.cards, &hand, card, Hearts));
    assert!(is_legal_play(&trick.cards, &hand, card, Clubs));
}

struct EachPlayer {
    start: Player,
    end: Player,
    gas: usize,
}
fn each_player(player: Player) -> EachPlayer {
    EachPlayer {
        gas: cardinality::<Player>(),
        start: previous_cycle(&player).unwrap(),
        end: player,
    }
}

impl Iterator for EachPlayer {
    type Item = Player;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gas > 0 {
            self.gas -= 1;
            self.start = next_cycle(&self.start).unwrap();
            Some(self.start)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for EachPlayer {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.gas > 0 {
            self.gas -= 1;
            self.end = previous_cycle(&self.end).unwrap();
            Some(self.end)
        } else {
            None
        }
    }
}

#[test]
fn test_each_player() {
    let mut iter = each_player(Player::A);
    assert_eq!(iter.next_back().unwrap(), Player::D);
    assert_eq!(iter.next().unwrap(), Player::A);
    assert_eq!(iter.next_back().unwrap(), Player::C);
    assert_eq!(iter.next_back().unwrap(), Player::B);
    assert!(iter.next_back().is_none());
}

fn partner(player: Player) -> Player {
    next_cycle(&next_cycle(&player).unwrap()).unwrap()
}

fn take_indices<T>(source: &mut Vec<T>, indices: BTreeSet<usize>) -> Result<Vec<T>, Error> {
    match indices.last() {
        Some(max_index) if *max_index >= source.len() => Err(Error::PlayingNonExtantCard),
        _ => Ok(indices
            .into_iter()
            .rev()
            .map(|index| source.remove(index))
            .collect()),
    }
}

fn compare(a: Card, b: Card, trump: Suit, lead: Suit) -> Ordering {
    if a.0 == b.0 {
        a.1.cmp(&b.1)
    } else {
        if a.0 == trump {
            Ordering::Greater
        } else if b.0 == trump {
            Ordering::Less
        } else if a.0 == lead {
            Ordering::Greater
        } else if b.0 == lead {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

#[test]
fn test_compare() {
    let ah = Card(Suit::Hearts, Rank::Ace);
    let kh = Card(Suit::Hearts, Rank::King);
    let ac = Card(Suit::Clubs, Rank::Ace);

    assert_eq!(
        compare(ah, kh, Suit::Diamonds, Suit::Hearts),
        Ordering::Greater
    );
    assert_eq!(
        compare(kh, ac, Suit::Hearts, Suit::Hearts),
        Ordering::Greater
    );
    assert_eq!(compare(kh, ac, Suit::Clubs, Suit::Hearts), Ordering::Less);
}

impl RoundState {
    fn act(&mut self, action: Action) -> Result<Option<(i32, i32)>, Error> {
        match (&mut self.phase, action) {
            (Phase::Bidding { bids, .. }, Action::Bid(amount)) => {
                bids.push(amount);
                self.current_player = next_cycle(&self.current_player).unwrap();
                if bids.len() == 4 {
                    let (winning_bidder, highest_bid) = bids
                        .iter()
                        .map(|x| *x)
                        .enumerate()
                        .max_by_key(|(_, b)| *b)
                        .unwrap();
                    self.current_player = winning_bidder.try_into().unwrap();
                    self.phase = Phase::DeclareTrump {
                        bid_winner: self.current_player,
                        highest_bid,
                    }
                }
            }
            (
                Phase::DeclareTrump {
                    bid_winner,
                    highest_bid,
                },
                Action::DeclareSuit(suit),
            ) => {
                self.current_player = partner(*bid_winner);
                self.phase = Phase::PassingTo {
                    bid_winner: *bid_winner,
                    highest_bid: *highest_bid,
                    trump: suit,
                }
            }
            (
                Phase::PassingTo {
                    trump,
                    bid_winner,
                    highest_bid,
                },
                Action::Pass(indices),
            ) => {
                Self::pass_cards(&mut self.hands, self.current_player, indices)?;
                self.current_player = *bid_winner;
                self.phase = Phase::PassingBack {
                    trump: *trump,
                    bid_winner: *bid_winner,
                    highest_bid: *highest_bid,
                };
            }
            (
                Phase::PassingBack {
                    trump,
                    bid_winner,
                    highest_bid,
                },
                Action::Pass(indices),
            ) => {
                Self::pass_cards(&mut self.hands, self.current_player, indices)?;
                self.current_player = *bid_winner;
                self.phase = Phase::RevealingCards {
                    extra_points: [0, 0],
                    reveals: Default::default(),
                    trump: *trump,
                    highest_bid: *highest_bid,
                    bid_winner: *bid_winner,
                };
            }
            (
                Phase::RevealingCards {
                    reveals,
                    extra_points,
                    bid_winner,
                    highest_bid,
                    trump,
                },
                Action::ShowPoints(cards),
            ) => {
                let the_cards = cards
                    .into_iter()
                    .filter_map(|x| self.hands[self.current_player as usize].get(x))
                    .map(|x| *x)
                    .collect();
                extra_points[self.current_player as usize % 2] += bonus_points(&the_cards, *trump);
                reveals[self.current_player as usize] = Some(the_cards);
                self.current_player = next_cycle(&self.current_player).unwrap();
                if reveals.iter().filter(|x| x.is_some()).count() == 4 {
                    self.current_player = *bid_winner;
                    self.phase = Phase::ReviewingRevealedCards {
                        reveals: reveals.clone(),
                        trump: *trump,
                        bid_winner: *bid_winner,
                        highest_bid: *highest_bid,
                        extra_points: *extra_points,
                        reviews: Default::default(),
                    }
                }
            }
            (
                Phase::ReviewingRevealedCards {
                    reviews,
                    extra_points,
                    bid_winner,
                    highest_bid,
                    trump,
                    ..
                },
                Action::Continue,
            ) => {
                reviews[self.current_player as usize] = true;
                self.current_player = next_cycle(&self.current_player).unwrap();
                if reviews.iter().all(|x| *x) {
                    self.current_player = *bid_winner;
                    let playing_phase = PlayingPhase {
                        trump: *trump,
                        bid_winner: *bid_winner,
                        highest_bid: *highest_bid,
                        extra_points: *extra_points,
                        piles: [vec![], vec![]],
                        trick: Trick {
                            first_player: *bid_winner,
                            cards: vec![],
                        },
                    };
                    // self.bots = [
                    //     Some(Bot::new(
                    //         Player::B,
                    //         self.hands[1].clone(),
                    //         playing_phase.clone(),
                    //     )),
                    //     Some(Bot::new(
                    //         Player::C,
                    //         self.hands[2].clone(),
                    //         playing_phase.clone(),
                    //     )),
                    //     Some(Bot::new(
                    //         Player::D,
                    //         self.hands[3].clone(),
                    //         playing_phase.clone(),
                    //     )),
                    // ];
                    self.phase = Phase::Play(playing_phase)
                }
            }
            (Phase::Play(playing_phase), Action::Play(index)) => {
                let current_hand = &mut self.hands[self.current_player as usize];
                if index >= current_hand.len() {
                    return Err(Error::PlayingNonExtantCard);
                }
                let card = current_hand[index];

                let (next_player, res) =
                    playing_phase.play(self.current_player, &current_hand, card)?;

                current_hand.remove(index);

                // self.bots[0].as_mut().unwrap().update(
                //     self.current_player,
                //     card,
                //     playing_phase.trump,
                //     &playing_phase.trick.cards,
                // );
                // self.bots[1].as_mut().unwrap().update(
                //     self.current_player,
                //     card,
                //     playing_phase.trump,
                //     &playing_phase.trick.cards,
                // );
                // self.bots[2].as_mut().unwrap().update(
                //     self.current_player,
                //     card,
                //     playing_phase.trump,
                //     &playing_phase.trick.cards,
                // );

                self.current_player = next_player;
                // return if res.is_some() || self.current_player == Player::A {
                return Ok(res);
                // } else {
                //     let bot = self.bots[self.current_player as usize - 1]
                //         .as_ref()
                //         .unwrap();
                //     let card = bot.get_move();
                //     self.act(Action::Play(
                //         self.hands[self.current_player as usize]
                //             .iter()
                //             .position(|x| *x == card)
                //             .unwrap(),
                //     ))
                // };
            }
            _ => return Err(Error::IncorrectAction),
        }
        Ok(None)
    }

    fn pass_cards(
        hands: &mut [Vec<Card>; 4],
        current_player: Player,
        indices: Vec<usize>,
    ) -> Result<(), Error> {
        let indices: BTreeSet<_> = indices.into_iter().collect();
        if indices.len() != 4 {
            return Err(Error::PassingWrongNumberOfCards);
        }
        let taken_cards = take_indices(&mut hands[current_player as usize], indices)?;
        hands[partner(current_player) as usize].extend(taken_cards);
        Ok(())
    }
}

fn bonus_points(cards: &Vec<Card>, trump: Suit) -> i32 {
    fn marriage(cards: &Vec<Card>, suit: Suit) -> i32 {
        or_double(
            cards,
            vec![Card(suit, Rank::King), Card(suit, Rank::Queen)],
            20,
            40,
        )
    }

    fn or_double(reveal: &Vec<Card>, pattern: Vec<Card>, points: i32, double: i32) -> i32 {
        let mut counts = vec![0; pattern.len()];
        for needle in reveal {
            if let Some(index) = pattern.iter().position(|card| needle == card) {
                counts[index] += 1
            }
        }
        if counts.iter().all(|x| *x >= 2) {
            double
        } else if counts.iter().all(|x| *x >= 1) {
            points
        } else {
            0
        }
    }

    fn round(reveal: &Vec<Card>, rank: Rank, points: i32) -> i32 {
        or_double(
            reveal,
            all::<Suit>().map(|suit| Card(suit, rank)).collect(),
            points,
            points * 10,
        )
    }

    // pinochle
    or_double(cards, vec![Card(Suit::Spades, Rank::Queen), Card(Suit::Diamonds, Rank::Jack)], 40, 300)  +
    // run
    or_double(cards, all::<Rank>().skip(1).map(|rank| Card(trump, rank)).collect(), 150 - 40, 1500 - 80)  +
    // rounds
    round(cards, Rank::Ace, 100)  +
    round(cards, Rank::King, 80)  +
    round(cards, Rank::Queen, 60)  +
    round(cards, Rank::Jack, 40)  +
    // trump marriage
    marriage(cards, trump) +
    // other marriages
    all::<Suit>().map(|suit| marriage(cards, suit)).sum::<i32>() +
    // nine of trump
    or_double(cards, vec![Card(trump, Rank::Nine)], 10, 20)
}

#[test]
fn test_bonus_points() {
    use Rank::*;
    use Suit::*;

    fn case(cards: &str, trump: Suit) -> i32 {
        let cards = cards
            .split(' ')
            .map(|x| {
                let rank = match x.chars().next().unwrap() {
                    '9' => Nine,
                    'J' => Jack,
                    'Q' => Queen,
                    'K' => King,
                    'T' => Ten,
                    'A' => Ace,
                    _ => todo!(),
                };
                let suit = match x.chars().skip(1).next().unwrap() {
                    'H' => Hearts,
                    'D' => Diamonds,
                    'C' => Clubs,
                    'S' => Spades,
                    _ => todo!(),
                };
                Card(suit, rank)
            })
            .collect();

        bonus_points(&cards, trump)
    }

    assert_eq!(case("AD AD AH AC", Clubs), 0);
    assert_eq!(case("AS AD AH AC", Clubs), 100);
    assert_eq!(case("AS AD AH AC AS AD AH AC", Clubs), 1000);
    assert_eq!(case("JD QS", Clubs), 40);
    assert_eq!(case("JD QS JD QS", Clubs), 300);
    assert_eq!(case("KD QD", Diamonds), 40);
    assert_eq!(case("KD QD", Clubs), 20);
    assert_eq!(case("KD QD TD AD JD", Diamonds), 150);
    assert_eq!(case("KD QD TD AD JD", Clubs), 20);
    assert_eq!(case("KD QD TD AD JD 9D", Diamonds), 160);
    assert_eq!(case("KD QD TD AD JD 9D KD QD TD AD JD 9D", Diamonds), 1520);
    assert_eq!(case("KD QD KD QD TD AD JD 9D", Diamonds), 200);
    assert_eq!(case("KD QD KS QS KH QH KC QC", Diamonds), 240);
    assert_eq!(case("AC AH AS KD QD TD AD JD 9D", Diamonds), 260);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence, Deserialize, Serialize)]
pub enum Player {
    A,
    B,
    C,
    D,
}

impl TryFrom<usize> for Player {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Player::A),
            1 => Ok(Player::B),
            2 => Ok(Player::C),
            3 => Ok(Player::D),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trick {
    pub first_player: Player,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Phase {
    Bidding {
        first_bidder: Player,
        bids: Vec<i32>,
    },
    DeclareTrump {
        bid_winner: Player,
        highest_bid: i32,
    },
    PassingTo {
        bid_winner: Player,
        highest_bid: i32,
        trump: Suit,
    },
    PassingBack {
        bid_winner: Player,
        highest_bid: i32,
        trump: Suit,
    },
    RevealingCards {
        reveals: [Option<Vec<Card>>; 4],
        extra_points: [i32; 2],
        bid_winner: Player,
        highest_bid: i32,
        trump: Suit,
    },
    ReviewingRevealedCards {
        reveals: [Option<Vec<Card>>; 4],
        reviews: [bool; 4],
        trump: Suit,
        bid_winner: Player,
        highest_bid: i32,
        extra_points: [i32; 2],
    },
    Play(PlayingPhase),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayingPhase {
    pub trump: Suit,
    pub bid_winner: Player,
    pub highest_bid: i32,
    pub extra_points: [i32; 2],
    pub piles: [Vec<Card>; 2],
    pub trick: Trick,
}

impl PlayingPhase {
    pub fn play(
        &mut self,
        current_player: Player,
        current_hand: &[Card],
        card: Card,
    ) -> Result<(Player, Option<(i32, i32)>), Error> {
        if !is_legal_play(&self.trick.cards, &current_hand, card, self.trump) {
            return Err(Error::CardIsNotLegalToPlay);
        }

        self.trick.cards.push(card);
        if self.trick.cards.len() == 4 {
            let player_cards = each_player(self.trick.first_player)
                .rev()
                .zip(self.trick.cards.iter().rev());
            let (winning_player, _) = player_cards
                .max_by(|(_, a), (_, b)| compare(**a, **b, self.trump, self.trick.cards[0].0))
                .unwrap();
            self.piles[winning_player as usize % 2].extend(self.trick.cards.drain(..));
            self.trick.first_player = winning_player;

            if current_hand.len() == 1 {
                fn count(
                    cards: &Vec<Card>,
                    last_trick: bool,
                    extra_points: i32,
                    highest_bid: i32,
                    is_bid_winner: bool,
                ) -> i32 {
                    let score = cards.iter().map(|Card(_, rank)| rank.points()).sum::<i32>()
                        + (if last_trick { 10 } else { 0 })
                        + extra_points;

                    if is_bid_winner {
                        if score >= highest_bid {
                            score
                        } else {
                            -highest_bid
                        }
                    } else {
                        score
                    }
                }

                let a = count(
                    &self.piles[0],
                    winning_player as usize % 2 == 0,
                    self.extra_points[0],
                    self.highest_bid,
                    self.bid_winner as usize % 2 == 0,
                );
                let b = count(
                    &self.piles[1],
                    winning_player as usize % 2 == 1,
                    self.extra_points[1],
                    self.highest_bid,
                    self.bid_winner as usize % 2 == 1,
                );

                Ok((winning_player, Some((a, b))))
            } else {
                Ok((winning_player, None))
            }
        } else {
            Ok((next_cycle(&current_player).unwrap(), None))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    PlayingNonExtantCard,
    PassingWrongNumberOfCards,
    IncorrectAction,
    NotTheCurrentPlayer,
    CardIsNotLegalToPlay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Bid(i32),
    Continue,
    DeclareSuit(Suit),
    ShowPoints(Vec<usize>),
    Pass(Vec<usize>),
    Play(usize),
}
