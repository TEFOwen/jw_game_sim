use crate::{
    card::{Card, CardSet},
    game::Strategy,
};

#[derive(Default)]
#[allow(dead_code)]
pub struct FlushNaive;

impl Strategy for FlushNaive {
    fn decide(&mut self, hand: CardSet, _dealers_hand: CardSet) -> CardSet {
        if hand.is_empty() {
            return !CardSet::default();
        }

        let suit = hand.to_cards()[0].suit();
        (1..=52)
            .map(Card::from)
            .filter(|card| card.suit() == suit)
            .fold(CardSet::default(), |mut acc, x| {
                acc.insert(x);
                acc
            })
    }
}
