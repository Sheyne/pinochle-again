use super::{Card, Player, PlayingPhase, Rank, Suit};
use enum_iterator::all;
use itertools::Itertools;
use ordered_float::NotNan;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::iter::repeat;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
struct PlayerState {
    known_cards: Vec<Card>,
    highest_possible: [Option<Rank>; 4],
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            known_cards: Default::default(),
            highest_possible: [Some(Rank::Ace); 4],
        }
    }
}

unsafe fn player_from_usize(i: usize) -> Player {
    unsafe { std::mem::transmute(i as u8) }
}

impl PlayerState {
    fn highest_possible(&self, suit: Suit) -> Option<Rank> {
        self.highest_possible[suit as usize]
    }
    fn set_highest_possible(&mut self, suit: Suit, rank: Option<Rank>) {
        self.highest_possible[suit as usize] = rank
    }

    fn could_have_card(&self, candidate: Card) -> bool {
        self.highest_possible(candidate.0)
            .map(|highest| highest >= candidate.1)
            .unwrap_or(false)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
struct State {
    played_cards: Vec<Card>,
    hand_sizes: [u8; 4],
    players: [PlayerState; 4],
}

impl Default for State {
    fn default() -> Self {
        Self {
            played_cards: Default::default(),
            hand_sizes: [12; 4],
            players: Default::default(),
        }
    }
}

impl State {
    fn player_state(&self, player: Player) -> &PlayerState {
        &self.players[player as usize]
    }
    fn player_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.players[player as usize]
    }

    fn calculate_solution<R>(
        &self,
        mut unknown_cards: Vec<Card>,
        mut res: [Vec<Card>; 4],
        rng: &mut R,
    ) -> [Vec<Card>; 4]
    where
        R: Rng + ?Sized,
    {
        unknown_cards.shuffle(rng);
        let mut candidacy_table: Vec<u8> = vec![0; unknown_cards.len()];

        for (idx, card) in unknown_cards.iter().enumerate() {
            for player in 0..4 {
                let player_state = &self.player_state(unsafe { player_from_usize(player) });
                candidacy_table[idx] |= (player_state.could_have_card(*card) as u8) << player;
            }
        }

        fn clear_constrained(
            candidacy_table: &mut Vec<u8>,
            unknown_cards: &mut Vec<Card>,
            res: &mut [Vec<Card>; 4],
        ) {
            let mut index = 0;
            while index < candidacy_table.len() {
                let row = candidacy_table[index];
                if row.count_ones() == 1 {
                    candidacy_table.remove(index);
                    let card = unknown_cards.remove(index);
                    let hot_col = row.trailing_zeros() as usize;
                    res[hot_col].push(card);
                } else {
                    index += 1;
                }
            }
        }

        clear_constrained(&mut candidacy_table, &mut unknown_cards, &mut res);

        fn search(candidacy_table: &mut Vec<u8>, hand_sizes: [u8; 4]) -> bool {
            let mut freedoms: Vec<_> = (0u8..4)
                .map(|idx| {
                    candidacy_table
                        .iter()
                        .filter(move |x| (*x & (1 << idx)) != 0)
                        .count()
                })
                .enumerate()
                .collect();
            freedoms.sort_by_key(|x| x.1);
            let column_order: Vec<_> = freedoms.into_iter().map(|x| x.0).collect();

            // get unsolved row
            if let Some(my_row) = candidacy_table.iter().position(|x| x.count_ones() != 1) {
                // save old value
                let old_value = candidacy_table[my_row];
                // loop through possible values
                for idx in column_order {
                    let bitset = 1 << idx;
                    if hand_sizes[idx] > 0 && old_value & bitset != 0 {
                        let mut copied_hand_sizes = hand_sizes.clone();
                        copied_hand_sizes[idx] -= 1;
                        // set possible value
                        candidacy_table[my_row] = bitset;
                        // recurse
                        if search(candidacy_table, copied_hand_sizes) {
                            return true;
                        }
                    }
                }
                false
            } else {
                true
            }
        }

        assert!(search(
            &mut candidacy_table,
            [
                self.hand_sizes[0] - res[0].len() as u8,
                self.hand_sizes[1] - res[1].len() as u8,
                self.hand_sizes[2] - res[2].len() as u8,
                self.hand_sizes[3] - res[3].len() as u8,
            ],
        ));

        for (card, location) in unknown_cards.iter().zip(candidacy_table) {
            res[location.trailing_zeros() as usize].push(*card);
        }

        res
    }

    fn random_solution<R>(
        &self,
        mut unknown_cards: Vec<Card>,
        mut res: [Vec<Card>; 4],
        rng: &mut R,
    ) -> Option<[Vec<Card>; 4]>
    where
        R: Rng + ?Sized,
    {
        unknown_cards.shuffle(rng);

        for i in 0..4 {
            let new_hand = &mut res[i];
            let player_state = &self.player_state(unsafe { player_from_usize(i) });

            let mut cant_have = vec![];
            while new_hand.len() < self.hand_sizes[i] as usize {
                loop {
                    let candidate = unknown_cards.pop()?;
                    if player_state.could_have_card(candidate) {
                        new_hand.push(candidate);
                        break;
                    } else {
                        cant_have.push(candidate);
                    }
                }
            }
            if cant_have.len() > 0 {
                unknown_cards.extend(cant_have);
                if i < 2 {
                    // if there's more than 1 player left, and we put
                    // cards back in the deck, we skewed the distribution and need to re-shuffle
                    unknown_cards.shuffle(rng);
                }
            }
        }
        Some(res)
    }

    fn produce_candidate_hands<R>(&self, rng: &mut R) -> [Vec<Card>; 4]
    where
        R: Rng + ?Sized,
    {
        let mut res = [vec![], vec![], vec![], vec![]];
        let mut known_cards = self.played_cards.clone();
        for i in 0..4 {
            let player_cards = &self
                .player_state(unsafe { player_from_usize(i) })
                .known_cards;
            known_cards.extend(player_cards);
            res[i].extend(player_cards)
        }
        known_cards.sort();
        let mut deck = all::<Card>().flat_map(|x| repeat(x).take(2));
        let mut unknown_cards: Vec<Card> = vec![];
        for card in known_cards {
            loop {
                let next = deck.next().expect("theres not more cards than we know of");
                if card == next {
                    break;
                }
                unknown_cards.push(next);
            }
        }
        unknown_cards.extend(deck);

        self.calculate_solution(unknown_cards, res, rng)
    }
    fn update(&mut self, player: Player, played: Card, trump: Suit, stack: &[Card]) {
        self.played_cards.push(played);
        self.hand_sizes[player as usize] -= 1;
        let player_state = self.player_state_mut(player);
        if let Some(position) = player_state.known_cards.iter().position(|x| x == &played) {
            player_state.known_cards.remove(position);
        }
        if let Some(lead_card) = stack.get(0) {
            if played.0 != lead_card.0 {
                player_state.set_highest_possible(lead_card.0, None);
                if played.0 != trump {
                    player_state.set_highest_possible(trump, None);
                } else {
                    if let Some(best_of_trump) = stack.iter().filter(|x| x.0 == lead_card.0).max() {
                        if played.1 <= best_of_trump.1 {
                            player_state
                                .set_highest_possible(best_of_trump.0, Some(best_of_trump.1))
                        }
                    }
                }
            } else {
                let best_of_lead = *stack
                    .iter()
                    .filter(|x| x.0 == lead_card.0)
                    .max()
                    .expect("At least one card of lead suit exists");

                if played.1 <= best_of_lead.1 {
                    player_state.set_highest_possible(best_of_lead.0, Some(best_of_lead.1))
                }
            }
        }
    }
}

#[test]
fn test_candidate_hands_exclusive_constraints() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(1);
    let mut state = State::default();
    state
        .player_state_mut(Player::A)
        .set_highest_possible(Suit::Clubs, None);
    state
        .player_state_mut(Player::B)
        .set_highest_possible(Suit::Clubs, None);
    state
        .player_state_mut(Player::D)
        .set_highest_possible(Suit::Clubs, None);
    let res = state.produce_candidate_hands(&mut rng);
    assert!(res[2].iter().all(|x| x.0 == Suit::Clubs));
    assert!(res.iter().all(|x| x.len() == 12));
}

#[test]
fn test_candidate_hands_inclusive_constraints() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(1);
    let mut state = State::default();
    state
        .player_state_mut(Player::C)
        .set_highest_possible(Suit::Hearts, None);
    state
        .player_state_mut(Player::C)
        .set_highest_possible(Suit::Clubs, None);
    state
        .player_state_mut(Player::C)
        .set_highest_possible(Suit::Diamonds, None);
    let res = state.produce_candidate_hands(&mut rng);
    assert!(res[2].iter().all(|x| x.0 == Suit::Spades));
    assert!(res.iter().all(|x| x.len() == 12));
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bot {
    state: State,
    player: Player,
    hand: Vec<Card>,
    playing_phase: PlayingPhase,
}

impl Bot {
    pub fn update(&mut self, player: Player, played: Card, trump: Suit, stack: &[Card]) {
        self.state.update(player, played, trump, stack);
        self.playing_phase.play(player, &[], played).unwrap();
        if player == self.player {
            self.hand
                .remove(self.hand.iter().position(|x| *x == played).unwrap());
        }
    }

    fn try_random_hand(&self) -> Option<(Card, i32)> {
        let mut candidate = self.state.produce_candidate_hands(&mut thread_rng());

        for hand in &mut candidate {
            hand.shuffle(&mut thread_rng());
        }

        fn step(
            phase: &mut PlayingPhase,
            current_player: &mut Player,
            hands: &mut [Vec<Card>; 4],
        ) -> Option<(Card, Option<(i32, i32)>)> {
            let hand = &mut hands[*current_player as usize];
            if let Some((next_player, points, card)) = hand.iter().find_map(|card| {
                phase
                    .play(*current_player, hand, *card)
                    .ok()
                    .map(|(n, p)| (n, p, *card))
            }) {
                hand.remove(hand.iter().position(|c| *c == card).unwrap());
                *current_player = next_player;
                Some((card, points))
            } else {
                dbg!("This SHOULD NOT HAVE HAPPENED");
                None
            }
        }

        fn filter_points(player: Player, (x, y): (i32, i32)) -> i32 {
            if player as u8 % 2 == 0 {
                x
            } else {
                y
            }
        }
        let mut current_player = self.player;
        let mut phase = self.playing_phase.clone();
        let (first_card, points) = step(&mut phase, &mut current_player, &mut candidate)?;
        if let Some(points) = points {
            return Some((first_card, filter_points(self.player, points)));
        }
        loop {
            let (_, points) = step(&mut phase, &mut current_player, &mut candidate)?;
            if let Some(points) = points {
                return Some((first_card, filter_points(self.player, points)));
            }
        }
    }

    pub fn new(player: Player, hand: Vec<Card>, playing_phase: PlayingPhase) -> Self {
        let mut me = Bot {
            state: Default::default(),
            player: player,
            hand: hand.clone(),
            playing_phase,
        };

        me.state.players[player as usize].known_cards = hand;
        me
    }

    pub fn get_move(&self) -> Card {
        let groups = (0..30000)
            .filter_map(|_| self.try_random_hand())
            .into_group_map();
        *groups
            .iter()
            .map(|(card, scores)| {
                (
                    card,
                    NotNan::new(scores.iter().sum::<i32>() as f32 / scores.len() as f32).unwrap(),
                )
            })
            .max_by_key(|(_, score)| *score)
            .expect("some move exists")
            .0
    }
}

#[test]
fn try_bot() {
    use pinochle::Trick;
    use Rank::*;
    use Suit::*;

    let hand = vec![
        Card(Clubs, Ace),
        Card(Clubs, Ace),
        Card(Clubs, King),
        Card(Clubs, Queen),
        Card(Clubs, Ten),
        Card(Clubs, Jack),
        Card(Clubs, Nine),
        Card(Diamonds, Jack),
        Card(Hearts, Ace),
        Card(Hearts, King),
        Card(Hearts, Jack),
        Card(Hearts, Nine),
    ];

    let bot = Bot::new(
        Player::A,
        hand,
        PlayingPhase {
            trump: Suit::Spades,
            bid_winner: Player::A,
            highest_bid: 0,
            extra_points: Default::default(),
            piles: Default::default(),
            trick: Trick {
                first_player: Player::A,
                cards: Default::default(),
            },
        },
    );

    dbg!(bot.get_move());

    assert!(false);
}
