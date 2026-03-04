use crate::{
    card::{Card, CardSet},
    game::Strategy,
};

#[derive(Default)]
pub struct FlushNaive;

impl Strategy for FlushNaive {
    fn decide(&mut self, hand: &[Card], _dealers_hand: &[Card]) -> CardSet {
        if hand.is_empty() {
            return !CardSet::default();
        }

        let suit = hand[0].suit();
        (1..=52)
            .map(Card::from)
            .filter(|card| card.suit() == suit)
            .fold(CardSet::default(), |mut acc, x| {
                acc.insert(x);
                acc
            })
    }
}
