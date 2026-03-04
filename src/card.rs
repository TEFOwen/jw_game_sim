#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl From<i32> for Suit {
    fn from(value: i32) -> Self {
        match value {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            3 => Suit::Clubs,
            _ => panic!("Invalid suit"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl From<i32> for Rank {
    fn from(value: i32) -> Self {
        match value {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => panic!("Invalid rank"),
        }
    }
}

/// Card represented as 1-52, 2s, 2h, 2d, 2c, ..., As, Ah, Ad, Ac
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card(i32);

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card(rank as i32 * 4 + suit as i32 + 1)
    }

    pub fn suit(&self) -> Suit {
        ((self.0 - 1) % 4).into()
    }

    pub fn rank(&self) -> Rank {
        ((self.0 - 1) / 4).into()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            RANK_DIGITS[self.rank() as usize],
            SUIT_DIGITS[self.suit() as usize]
        )
    }
}

impl From<i32> for Card {
    fn from(value: i32) -> Self {
        Card(value)
    }
}

impl Into<i32> for Card {
    fn into(self) -> i32 {
        self.0
    }
}

const RANK_DIGITS: [char; 13] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
];

const SUIT_DIGITS: [char; 4] = ['s', 'h', 'd', 'c'];

impl<'a> TryFrom<&'a str> for Card {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err("Invalid card length");
        }

        let mut chars = value.chars();
        let rank_char = chars.next().ok_or("Invalid card length")?;

        let rank = RANK_DIGITS
            .iter()
            .enumerate()
            .find_map(|(i, &r)| {
                if r == rank_char {
                    Some(Rank::from(i as i32))
                } else {
                    None
                }
            })
            .ok_or("Invalid rank")?;

        let suit_char = chars.next().ok_or("Invalid card length")?;
        let suit = SUIT_DIGITS
            .iter()
            .enumerate()
            .find_map(|(i, &r)| {
                if r == suit_char {
                    Some(Suit::from(i as i32))
                } else {
                    None
                }
            })
            .ok_or("Invalid suit")?;

        Ok(Card::new(rank, suit))
    }
}

#[derive(Default, Clone, Copy)]
#[repr(transparent)]
pub struct CardSet(u64);

impl CardSet {
    pub fn insert(&mut self, card: Card) {
        self.0 |= 1 << (card.0 - 1);
    }

    pub fn contains(&self, card: &Card) -> bool {
        (self.0 & (1 << (card.0 - 1))) != 0
    }

    pub fn from_cards(cards: &[Card]) -> Self {
        let mut set = CardSet(0);
        for &card in cards {
            set.insert(card);
        }
        set
    }

    pub fn to_cards(&self) -> Vec<Card> {
        let mut cards = Vec::new();
        for i in 0..52 {
            if (self.0 & (1 << i)) != 0 {
                cards.push(Card(i + 1));
            }
        }
        cards
    }

    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn union(&self, other: &CardSet) -> CardSet {
        CardSet(self.0 | other.0)
    }

    pub fn intersection(&self, other: &CardSet) -> CardSet {
        CardSet(self.0 & other.0)
    }
}

impl Not for CardSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        CardSet(!self.0 & ((1 << 52) - 1))
    }
}

impl BitOr for CardSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(&rhs)
    }
}

impl BitAnd for CardSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(&rhs)
    }
}

impl BitOrAssign for CardSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for CardSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
