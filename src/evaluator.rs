#![allow(dead_code)]

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::{BufReader, Read},
};

use once_cell::sync::Lazy;

use crate::card::{Card, Rank};

const HAND_RANKS_FILE_NAME: &str = "handranks.dat";

pub static HAND_RANKS: Lazy<Vec<i32>> = Lazy::new(|| {
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
    hand_ranks
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
        write!(f, "{}", self.0)
    }
}

impl HandRank {
    pub fn hand_type(&self) -> HandType {
        (self.0 >> 12).into()
    }
}

impl Debug for HandRank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} ({})", self.hand_type(), self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandType {
    HighCard,
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

#[derive(Clone, Copy)]
pub enum EvaluationStage {
    Stage1(i32),
    Stage2(i32),
    Stage3(i32),
    Stage4(i32),
    Stage5(i32),
    Stage6(i32),
    Stage7(i32),
}

impl EvaluationStage {
    pub fn is_evaluatable(&self) -> bool {
        matches!(
            self,
            EvaluationStage::Stage7(_) | EvaluationStage::Stage6(_) | EvaluationStage::Stage5(_)
        )
    }

    pub fn is_final(&self) -> bool {
        matches!(self, EvaluationStage::Stage7(_))
    }

    pub fn start(card: Card) -> Self {
        EvaluationStage::Stage1(HAND_RANKS[53 + Into::<i32>::into(card) as usize])
    }

    pub fn next(self, card: Card) -> Self {
        match self {
            EvaluationStage::Stage1(rank) => {
                EvaluationStage::Stage2(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            EvaluationStage::Stage2(rank) => {
                EvaluationStage::Stage3(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            EvaluationStage::Stage3(rank) => {
                EvaluationStage::Stage4(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            EvaluationStage::Stage4(rank) => {
                EvaluationStage::Stage5(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            EvaluationStage::Stage5(rank) => {
                EvaluationStage::Stage6(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            EvaluationStage::Stage6(rank) => {
                EvaluationStage::Stage7(HAND_RANKS[(rank + Into::<i32>::into(card)) as usize])
            }
            _ => panic!("Cannot add more cards to the stage"),
        }
    }

    pub fn evaluate(self) -> HandRank {
        match self {
            EvaluationStage::Stage5(rank) | EvaluationStage::Stage6(rank) => {
                HandRank(HAND_RANKS[rank as usize])
            }
            EvaluationStage::Stage7(rank) => HandRank(rank),
            _ => panic!("Cannot evaluate stage"),
        }
    }

    pub fn stage_num(&self) -> u32 {
        match self {
            EvaluationStage::Stage1(_) => 1,
            EvaluationStage::Stage2(_) => 2,
            EvaluationStage::Stage3(_) => 3,
            EvaluationStage::Stage4(_) => 4,
            EvaluationStage::Stage5(_) => 5,
            EvaluationStage::Stage6(_) => 6,
            EvaluationStage::Stage7(_) => 7,
        }
    }
}

pub fn evaluate(cards: &[Card]) -> HandRank {
    debug_assert!(
        [5, 6, 7].contains(&cards.len()),
        "Invalid number of cards (must be 5, 6, or 7)"
    );
    let mut stage = EvaluationStage::start(cards[0]);
    for &card in &cards[1..] {
        stage = stage.next(card);
    }
    stage.evaluate()
}

pub fn evaluate_many(cards: &[Card]) -> HandRank {
    if cards.len() <= 7 {
        return evaluate(cards);
    }

    let mut sorted = cards.to_vec();
    sorted.sort();
    let mut ranks = [const { vec![] }; 13];
    for card in &sorted {
        ranks[card.rank() as usize].push(*card);
    }

    let mut suits = [const { vec![] }; 4];
    for card in &sorted {
        suits[card.suit() as usize].push(*card);
    }

    // Straight flush check
    let mut suit_best = None;
    for suit in 0..4 {
        if suits[suit].len() >= 5 {
            for i in 0..=suits[suit].len() - 5 {
                let hand_rank = evaluate(&suits[suit][i..i + 5]);
                if suit_best.is_none() || hand_rank > suit_best.unwrap() {
                    suit_best = Some(hand_rank);
                }
            }

            // Wheel check
            if suits[suit].last().unwrap().rank() == Rank::Ace {
                let hand_rank = evaluate(&[
                    *suits[suit].last().unwrap(),
                    suits[suit][0],
                    suits[suit][1],
                    suits[suit][2],
                    suits[suit][3],
                ]);
                if suit_best.is_none() || hand_rank > suit_best.unwrap() {
                    suit_best = Some(hand_rank);
                }
            }
        }
    }

    if let Some(suit_best) = suit_best {
        if suit_best.hand_type() == HandType::StraightFlush {
            return suit_best;
        }
    }

    // Quads
    for rank in ranks.iter().rev() {
        if rank.len() == 4 {
            let kicker = sorted
                .iter()
                .rev()
                .find(|&&card| card.rank() != rank[0].rank())
                .unwrap();
            return evaluate(
                rank.clone()
                    .into_iter()
                    .chain(std::iter::once(*kicker))
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
        }
    }

    // Full house or trips
    let mut trips = None;
    for trip_rank in ranks.iter().rev() {
        if trip_rank.len() == 3 {
            for pair_rank in ranks.iter().rev() {
                if pair_rank.len() == 2 {
                    return evaluate(&[
                        pair_rank[0],
                        pair_rank[1],
                        trip_rank[0],
                        trip_rank[1],
                        trip_rank[2],
                    ]);
                }
            }
            let kickers = sorted
                .iter()
                .rev()
                .filter(|&&card| card.rank() != trip_rank[0].rank())
                .take(2)
                .copied();
            let hand_rank = evaluate(
                trip_rank
                    .clone()
                    .into_iter()
                    .chain(kickers)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            if trips.is_none() || hand_rank > trips.unwrap() {
                trips = Some(hand_rank);
            }
        }
    }

    if let Some(suit_best) = suit_best {
        return suit_best;
    }

    // Wheel check
    let rank_bools = ranks
        .iter()
        .map(|rank| !rank.is_empty())
        .collect::<Vec<_>>();
    let mut straight_best =
        if rank_bools[12] && rank_bools[0] && rank_bools[1] && rank_bools[2] && rank_bools[3] {
            Some(HandRank(HandType::Straight as i32 + 1))
        } else {
            None
        };

    // Straight check
    for i in 0..=8 {
        if rank_bools[i]
            && rank_bools[i + 1]
            && rank_bools[i + 2]
            && rank_bools[i + 3]
            && rank_bools[i + 4]
        {
            let hand_rank = HandRank((i as i32 + HandType::Straight as i32) + 1);
            if straight_best.is_none() || hand_rank > straight_best.unwrap() {
                straight_best = Some(hand_rank);
            }
        }
    }

    if let Some(straight_best) = straight_best {
        return straight_best;
    }

    if let Some(trips) = trips {
        return trips;
    }

    // Pair check
    let mut pair_best = None;
    for rank in ranks.iter().rev() {
        if rank.len() == 2 {
            let kickers = sorted
                .iter()
                .rev()
                .filter(|&&card| card.rank() != rank[0].rank())
                .take(3)
                .copied();
            let hand_rank = evaluate(
                rank.clone()
                    .into_iter()
                    .chain(kickers)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            if pair_best.is_none() || hand_rank > pair_best.unwrap() {
                pair_best = Some(hand_rank);
            }
        }
    }

    if let Some(pair_best) = pair_best {
        return pair_best;
    }

    // High card
    evaluate(&sorted[sorted.len() - 5..])
}

#[cfg(test)]
mod tests {
    use crate::card::{Rank, Suit};

    use super::*;

    #[test]
    fn test_hand_rank() {
        let hand_rank = evaluate(&[
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Two, Suit::Hearts),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Three, Suit::Spades),
        ]);
        assert!(hand_rank.hand_type() == HandType::FourOfAKind);
    }

    #[test]
    fn test_evaluate_many() {
        let hand_rank = evaluate_many(&[
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
        ]);
        assert!(hand_rank.hand_type() == HandType::StraightFlush);
    }
}
