use enum_iterator::{all, cardinality, next_cycle, Sequence};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Sequence, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Sequence, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Sequence, Serialize, Deserialize)]
pub struct Card(Suit, Rank);

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.1, self.0)
    }
}

fn shuffled() -> Vec<Card> {
    let mut deck = all::<Card>().collect::<Vec<_>>();
    deck.extend(all::<Card>());
    deck.shuffle(&mut thread_rng());
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
pub struct Game {
    hand: RoundState,
    scores: [i32; 2],
    current_player: Player,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            current_player: Player::A,
            hand: RoundState::start(Player::A),
            scores: [0; 2],
        }
    }
}

impl Game {
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
            self.hand = RoundState::start(self.current_player);
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

impl From<&Game> for GameInfo {
    fn from(value: &Game) -> Self {
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
    pub fn start(player: Player) -> Self {
        Self {
            current_player: player,
            hands: dealt(shuffled()),
            phase: Phase::Bidding {
                first_bidder: player,
                bids: vec![],
            },
        }
    }
}

struct EachPlayer {
    current_player: Player,
    gas: usize,
}
fn each_player(player: Player) -> EachPlayer {
    EachPlayer {
        current_player: player,
        gas: cardinality::<Player>(),
    }
}

impl Iterator for EachPlayer {
    type Item = Player;

    fn next(&mut self) -> Option<Self::Item> {
        if self.gas > 0 {
            let player = self.current_player;
            self.current_player = next_cycle(&self.current_player).unwrap();
            self.gas -= 1;
            Some(player)
        } else {
            None
        }
    }
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
                Action::ShowPoints(cards, points),
            ) => {
                let the_cards = cards
                    .into_iter()
                    .filter_map(|x| self.hands[self.current_player as usize].get(x))
                    .map(|x| *x)
                    .collect();
                reveals[self.current_player as usize] = Some(the_cards);
                extra_points[self.current_player as usize % 2] += points;
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
                    self.phase = Phase::Play {
                        trump: *trump,
                        bid_winner: *bid_winner,
                        highest_bid: *highest_bid,
                        extra_points: *extra_points,
                        piles: [vec![], vec![]],
                        trick: Trick {
                            first_player: *bid_winner,
                            cards: vec![],
                        },
                    }
                }
            }
            (
                Phase::Play {
                    trick,
                    trump,
                    piles,
                    extra_points,
                    highest_bid,
                    bid_winner,
                },
                Action::Play(index),
            ) => {
                let current_hand = &mut self.hands[self.current_player as usize];
                if index >= current_hand.len() {
                    return Err(Error::PlayingNonExtantCard);
                }
                let card = current_hand.remove(index);
                trick.cards.push(card);
                if trick.cards.len() == 4 {
                    let player_cards = each_player(trick.first_player).zip(trick.cards.iter());
                    let (winning_player, _) = player_cards
                        .max_by(|(_, a), (_, b)| compare(**a, **b, *trump, trick.cards[0].0))
                        .unwrap();
                    piles[winning_player as usize % 2].extend(trick.cards.drain(..));
                    self.current_player = winning_player;
                    trick.first_player = winning_player;

                    if current_hand.len() == 0 {
                        fn count(
                            cards: &Vec<Card>,
                            last_trick: bool,
                            extra_points: i32,
                            highest_bid: i32,
                            is_bid_winner: bool,
                        ) -> i32 {
                            let score =
                                cards.iter().map(|Card(_, rank)| rank.points()).sum::<i32>()
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
                            &piles[0],
                            winning_player as usize % 2 == 0,
                            extra_points[0],
                            *highest_bid,
                            *bid_winner as usize % 2 == 0,
                        );
                        let b = count(
                            &piles[1],
                            winning_player as usize % 2 == 1,
                            extra_points[1],
                            *highest_bid,
                            *bid_winner as usize % 2 == 1,
                        );

                        return Ok(Some((a, b)));
                    }
                } else {
                    self.current_player = next_cycle(&self.current_player).unwrap();
                }
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
struct Trick {
    first_player: Player,
    cards: Vec<Card>,
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
    Play {
        trump: Suit,
        bid_winner: Player,
        highest_bid: i32,
        extra_points: [i32; 2],
        piles: [Vec<Card>; 2],
        trick: Trick,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    PlayingNonExtantCard,
    PassingWrongNumberOfCards,
    IncorrectAction,
    NotTheCurrentPlayer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Bid(i32),
    Continue,
    DeclareSuit(Suit),
    ShowPoints(Vec<usize>, i32),
    Pass(Vec<usize>),
    Play(usize),
}
