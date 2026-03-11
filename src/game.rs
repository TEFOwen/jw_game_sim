use rand::seq::SliceRandom;

use crate::{
    card::{Card, CardSet},
    evaluator::{evaluate, evaluate_many},
};

pub trait Strategy {
    fn decide(&mut self, hand: CardSet, dealers_hand: CardSet) -> CardSet;
}

pub fn play<T: Strategy>(mut strategy: T) -> bool {
    let mut hand = CardSet::default();
    let mut dealers_hand = CardSet::default();
    let mut deck = (1..=52).map(Card::from).collect::<Vec<_>>();

    let mut rng = rand::rng();
    deck.shuffle(&mut rng);
    let mut deck = deck.into_iter();

    'outer: for _ in 0..5 {
        let set = strategy.decide(hand, dealers_hand);

        loop {
            let Some(card) = deck.next() else {
                break 'outer;
            };

            if set.contains(card) {
                hand.insert(card);
                break;
            }

            dealers_hand.insert(card);
        }
    }

    if hand.len() != 5 {
        return false;
    }

    while dealers_hand.len() < 8 {
        dealers_hand.insert(deck.next().unwrap());
    }

    evaluate(hand) > evaluate_many(dealers_hand)
}
