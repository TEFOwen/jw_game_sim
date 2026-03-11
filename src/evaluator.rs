#![allow(dead_code)]

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::{BufReader, Read},
};

use once_cell::sync::Lazy;

use crate::card::{Card, CardSet, Rank};

const HAND_RANKS_FILE_NAME: &str = "handranks.dat";

pub static HAND_RANKS: Lazy<Box<[i32]>> = Lazy::new(|| {
    let mut hand_ranks = vec![0; 32487834];
    let file = File::open(HAND_RANKS_FILE_NAME)
        .expect(&format!("Could not open {}", HAND_RANKS_FILE_NAME));
    let mut reader = BufReader::new(file);
    let mut buffer = [0; std::mem::size_of::<i32>()];
    for i in 0..hand_ranks.len() {
        reader
            .read_exact(&mut buffer)
            .expect(&format!("Could not read from {}", HAND_RANKS_FILE_NAME));
        hand_ranks[i] = i32::from_le_bytes(buffer);
    }
    hand_ranks.into_boxed_slice()
});

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandRank(i32);

impl Into<i32> for HandRank {
    fn into(self) -> i32 {
        self.0
    }
}

impl Display for HandRank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

impl HandRank {
    pub fn hand_type(&self) -> HandType {
        (self.0 >> 12).into()
    }
}

impl Debug for HandRank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} ({})", self.hand_type(), self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandType {
    HighCard = 1,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl From<i32> for HandType {
    fn from(value: i32) -> Self {
        match value {
            1 => HandType::HighCard,
            2 => HandType::OnePair,
            3 => HandType::TwoPair,
            4 => HandType::ThreeOfAKind,
            5 => HandType::Straight,
            6 => HandType::Flush,
            7 => HandType::FullHouse,
            8 => HandType::FourOfAKind,
            9 => HandType::StraightFlush,
            _ => panic!("Invalid hand type"),
        }
    }
}

impl Display for HandType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HandType::HighCard => "High Card",
                HandType::OnePair => "One Pair",
                HandType::TwoPair => "Two Pair",
                HandType::ThreeOfAKind => "Three of a Kind",
                HandType::Straight => "Straight",
                HandType::Flush => "Flush",
                HandType::FullHouse => "Full House",
                HandType::FourOfAKind => "Four of a Kind",
                HandType::StraightFlush => "Straight Flush",
            }
        )
    }
}

// pub fn evaluate(cards: &[i32]) -> HandRank {
//     debug_assert!(
//         [5, 6, 7].contains(&cards.len()),
//         "Invalid number of cards (must be 5, 6, or 7)"
//     );
//     let mut rank = HAND_RANKS[53 + cards[0] as usize];
//     rank = HAND_RANKS[(rank + cards[1]) as usize];
//     rank = HAND_RANKS[(rank + cards[2]) as usize];
//     rank = HAND_RANKS[(rank + cards[3]) as usize];
//     rank = HAND_RANKS[(rank + cards[4]) as usize];
//     if cards.len() >= 5 {
//         rank = HAND_RANKS[(rank + cards[5]) as usize];
//         if cards.len() >= 6 {
//             HandRank(HAND_RANKS[(rank + cards[6]) as usize])
//         } else {
//             HandRank(HAND_RANKS[rank as usize])
//         }
//     } else {
//         HandRank(HAND_RANKS[rank as usize])
//     }
// }

// TODO: Change evaluate function as above?
#[inline(always)]
pub fn evaluate(cards: CardSet) -> HandRank {
    debug_assert!(
        [5, 6, 7].contains(&cards.len()),
        "Invalid number of cards (must be 5, 6, or 7)"
    );
    let iter = (1..=52).filter(|&i| cards.0 & (1 << (i - 1)) > 0);
    evaluate_iter(iter)
}

pub fn try_evaluate(cards: impl IntoIterator<Item = i32>) -> Option<HandRank> {
    let mut iter = cards.into_iter();
    let mut rank = HAND_RANKS[53 + iter.next()? as usize];
    rank = HAND_RANKS[(rank + iter.next()?) as usize];
    rank = HAND_RANKS[(rank + iter.next()?) as usize];
    rank = HAND_RANKS[(rank + iter.next()?) as usize];
    rank = HAND_RANKS[(rank + iter.next()?) as usize];
    if let Some(card) = iter.next() {
        rank = HAND_RANKS[(rank + card) as usize];
        if let Some(card) = iter.next() {
            Some(HandRank(HAND_RANKS[(rank + card) as usize]))
        } else {
            Some(HandRank(HAND_RANKS[rank as usize]))
        }
    } else {
        Some(HandRank(HAND_RANKS[rank as usize]))
    }
}

#[inline(always)]
pub fn evaluate_iter(cards: impl IntoIterator<Item = i32>) -> HandRank {
    let mut iter = cards.into_iter();
    let mut rank = HAND_RANKS[53 + iter.next().unwrap() as usize];
    rank = HAND_RANKS[(rank + iter.next().unwrap()) as usize];
    rank = HAND_RANKS[(rank + iter.next().unwrap()) as usize];
    rank = HAND_RANKS[(rank + iter.next().unwrap()) as usize];
    rank = HAND_RANKS[(rank + iter.next().unwrap()) as usize];
    if let Some(card) = iter.next() {
        rank = HAND_RANKS[(rank + card) as usize];
        if let Some(card) = iter.next() {
            HandRank(HAND_RANKS[(rank + card) as usize])
        } else {
            HandRank(HAND_RANKS[rank as usize])
        }
    } else {
        HandRank(HAND_RANKS[rank as usize])
    }
}

const STRAIGHT_FLUSH_MASK: u64 = 0x11111;
const STRAIGHT_FLUSH_WHEEL_MASK: u64 = 0x1111 + (1 << (Rank::Ace as i32 * 4));
const FLUSH_MASK: u64 = 0x1111111111111;
const QUADS_MASK: u64 = 0xF;

pub fn evaluate_many(cards: CardSet) -> HandRank {
    if cards.len() <= 7 {
        return evaluate(cards);
    }

    let set_bit = cards.0;
    for i in (0..=(13 - 5)).rev() {
        for j in 0..4 {
            let mask = STRAIGHT_FLUSH_MASK << (i * 4 + j);
            if set_bit & mask == mask {
                return HandRank(((HandType::StraightFlush as i32) << 12) + i as i32 + 2);
            }
        }
    }

    for j in 0..4 {
        let mask = STRAIGHT_FLUSH_WHEEL_MASK << j;
        if set_bit & mask == mask {
            return HandRank(((HandType::StraightFlush as i32) << 12) + 1);
        }
    }

    let mut full_house = None;
    let mut trips = None;
    let mut rank_counts = [0; 13];
    for rank in (0..13).rev() {
        match (set_bit >> (rank * 4) & QUADS_MASK).count_ones() {
            4 => {
                let others = set_bit & !(0xF << (rank * 4));
                let kicker = (51 - (others.leading_zeros() as i32 - (64 - 52))) / 4;
                return HandRank(
                    ((HandType::FourOfAKind as i32) << 12)
                        + 1
                        + rank * 12
                        + kicker
                        + if kicker < rank { 0 } else { -1 },
                );
            }
            3 => {
                if let Some(pair_rank) = (0..13)
                    .rev()
                    .find(|&i| i != rank && (set_bit & (QUADS_MASK << (i * 4))).count_ones() >= 2)
                {
                    if full_house.is_none() {
                        full_house = Some(HandRank(
                            ((HandType::FullHouse as i32) << 12)
                                + 1
                                + rank * 12
                                + pair_rank
                                + if pair_rank < rank { 0 } else { -1 },
                        ));
                    }
                }
                let kickers = (0..13)
                    .rev()
                    .filter_map(|i| {
                        if i != rank && (set_bit & (QUADS_MASK << (i * 4))).count_ones() > 0 {
                            Some(i * 4 + 1)
                        } else {
                            None
                        }
                    })
                    .take(2);
                let hand_rank = evaluate_iter(kickers.chain((1..=3).map(|i| rank * 4 + i)));

                if trips.is_none() {
                    trips = Some(hand_rank);
                }
            }
            count => rank_counts[rank as usize] = count,
        }
    }

    if let Some(full_house) = full_house {
        return full_house;
    }

    if let Some(flush_rank) = (0..4)
        .filter_map(|i| {
            let bits = set_bit & (FLUSH_MASK << i);
            if bits.count_ones() < 5 {
                return None;
            }
            try_evaluate(
                (0..13)
                    .rev()
                    .filter_map(|j| {
                        if bits & (0xF << (j * 4)) != 0 {
                            Some(j * 4 + 1)
                        } else {
                            None
                        }
                    })
                    .take(5),
            )
        })
        .max()
    {
        return flush_rank;
    }

    // Wheel check
    let mut ranks = 0u16;

    for rank in 0..13 {
        if set_bit & (0xF << (rank * 4)) > 0 {
            ranks |= 1 << rank;
        }
    }

    for start in (0..=8).rev() {
        if (ranks >> start) & 0b11111 == 0b11111 {
            return HandRank(((HandType::Straight as i32) << 12) + 2 + start);
        }
    }

    if ranks & 0b1000000001111 == 0b1000000001111 {
        return HandRank(((HandType::Straight as i32) << 12) + 1);
    }

    if let Some(trips) = trips {
        return trips;
    }

    // Pair check
    for rank in (0..13).rev() {
        if rank_counts[rank as usize] == 2 {
            if let Some(pair2) = (0..13)
                .rev()
                .find(|&rank2| rank2 != rank && rank_counts[rank2 as usize] == 2)
            {
                let others = set_bit & !(0xF << (rank * 4) | 0xF << (pair2 * 4));
                let kicker = (51 - (others.leading_zeros() as i32 - (64 - 52))) / 4;
                return evaluate_iter([
                    rank * 4 + 1,
                    rank * 4 + 2,
                    pair2 * 4 + 1,
                    pair2 * 4 + 2,
                    kicker * 4 + 1,
                ]);
            }
            let kickers = (0..13)
                .rev()
                .filter_map(|i| {
                    if i != rank && rank_counts[i as usize] > 0 {
                        Some(i * 4 + 1)
                    } else {
                        None
                    }
                })
                .take(3)
                .chain(std::iter::once(rank * 4 + 1))
                .chain(std::iter::once(rank * 4 + 2));
            return evaluate_iter(kickers);
        }
    }

    // High card
    evaluate_iter(
        (0..13)
            .rev()
            .filter(|&i| rank_counts[i as usize] > 0)
            .take(5),
    )
}

#[cfg(test)]
mod tests {
    use crate::card::{Rank, Suit};

    use super::*;

    #[test]
    fn test_hand_rank() {
        let hand_rank = evaluate(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
        ]));
        assert!(hand_rank.hand_type() == HandType::FourOfAKind);

        let hand_rank = evaluate(CardSet::from_cards(&[
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Four, Suit::Spades),
            Card::new(Rank::Five, Suit::Spades),
        ]));
        assert!(hand_rank.0 == 0x9001);
    }

    #[test]
    fn test_evaluate_many() {
        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::Four, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Queen, Suit::Spades),
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Spades),
        ]));
        assert!(hand_rank.hand_type() == HandType::StraightFlush);

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::King, Suit::Spades),
                    Card::new(Rank::King, Suit::Hearts),
                    Card::new(Rank::King, Suit::Diamonds),
                    Card::new(Rank::King, Suit::Clubs),
                    Card::new(Rank::Jack, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::King, Suit::Diamonds),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Four, Suit::Hearts),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::King, Suit::Spades),
                    Card::new(Rank::King, Suit::Hearts),
                    Card::new(Rank::King, Suit::Diamonds),
                    Card::new(Rank::Ten, Suit::Clubs),
                    Card::new(Rank::Jack, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Five, Suit::Spades),
            Card::new(Rank::Five, Suit::Hearts),
            Card::new(Rank::Five, Suit::Diamonds),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Three, Suit::Hearts),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Five, Suit::Spades),
                    Card::new(Rank::Five, Suit::Hearts),
                    Card::new(Rank::Five, Suit::Diamonds),
                    Card::new(Rank::Ten, Suit::Clubs),
                    Card::new(Rank::Three, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Two, Suit::Spades),
                    Card::new(Rank::Two, Suit::Hearts),
                    Card::new(Rank::Two, Suit::Diamonds),
                    Card::new(Rank::Two, Suit::Clubs),
                    Card::new(Rank::Jack, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Five, Suit::Hearts),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Four, Suit::Hearts),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Two, Suit::Spades),
                    Card::new(Rank::Two, Suit::Hearts),
                    Card::new(Rank::Ace, Suit::Diamonds),
                    Card::new(Rank::King, Suit::Clubs),
                    Card::new(Rank::Ten, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Ace, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Five, Suit::Hearts),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Two, Suit::Spades),
                    Card::new(Rank::Two, Suit::Hearts),
                    Card::new(Rank::Ace, Suit::Diamonds),
                    Card::new(Rank::Five, Suit::Clubs),
                    Card::new(Rank::Five, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Six, Suit::Spades),
            Card::new(Rank::King, Suit::Spades),
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Ace, Suit::Spades),
            // --
            Card::new(Rank::Three, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Diamonds),
            Card::new(Rank::Four, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Diamonds),
            Card::new(Rank::King, Suit::Diamonds),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Seven, Suit::Spades),
                    Card::new(Rank::Six, Suit::Spades),
                    Card::new(Rank::King, Suit::Spades),
                    Card::new(Rank::Ten, Suit::Spades),
                    Card::new(Rank::Ace, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Spades),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Two, Suit::Spades),
                    Card::new(Rank::Two, Suit::Hearts),
                    Card::new(Rank::Jack, Suit::Diamonds),
                    Card::new(Rank::Jack, Suit::Clubs),
                    Card::new(Rank::Jack, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Three, Suit::Spades),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Ace, Suit::Spades),
                    Card::new(Rank::Two, Suit::Hearts),
                    Card::new(Rank::Three, Suit::Diamonds),
                    Card::new(Rank::Four, Suit::Clubs),
                    Card::new(Rank::Five, Suit::Spades),
                ]))
        );

        let hand_rank = evaluate_many(CardSet::from_cards(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Five, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Spades),
            Card::new(Rank::Jack, Suit::Hearts),
            Card::new(Rank::Jack, Suit::Clubs),
            Card::new(Rank::Four, Suit::Hearts),
            Card::new(Rank::Three, Suit::Spades),
            Card::new(Rank::Six, Suit::Diamonds),
            Card::new(Rank::Seven, Suit::Spades),
            Card::new(Rank::Eight, Suit::Clubs),
        ]));
        assert!(
            hand_rank
                == evaluate(CardSet::from_cards(&[
                    Card::new(Rank::Six, Suit::Spades),
                    Card::new(Rank::Eight, Suit::Hearts),
                    Card::new(Rank::Seven, Suit::Diamonds),
                    Card::new(Rank::Four, Suit::Clubs),
                    Card::new(Rank::Five, Suit::Spades),
                ]))
        );
    }
}
