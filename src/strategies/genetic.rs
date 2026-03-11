use rand::RngExt;
use rand_distr::Distribution;

use crate::{
    card::{Card, CardSet, Rank},
    game::Strategy,
};

#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct GeneticStrategy {
    flush_bias: f32,
    straight_bias: f32,
    pair_bias: f32,
    high_card_bias: f32,
    dealer_strength_bias: f32,
    score_threshold: f32,
    starting_set: CardSet,
}

#[allow(dead_code)]
impl GeneticStrategy {
    pub const fn new(
        flush_bias: f32,
        straight_bias: f32,
        pair_bias: f32,
        high_card_bias: f32,
        dealer_strength_bias: f32,
        score_threshold: f32,
        starting_set: CardSet,
    ) -> Self {
        Self {
            flush_bias,
            straight_bias,
            pair_bias,
            high_card_bias,
            dealer_strength_bias,
            score_threshold,
            starting_set,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        let dist = rand_distr::Normal::new(0.5, 0.2).unwrap();

        Self {
            flush_bias: dist.sample(&mut rng),
            straight_bias: dist.sample(&mut rng),
            pair_bias: dist.sample(&mut rng),
            high_card_bias: dist.sample(&mut rng),
            dealer_strength_bias: dist.sample(&mut rng),
            score_threshold: dist.sample(&mut rng),
            starting_set: CardSet::from_cards(
                (1..=52)
                    .map(Card::from)
                    .filter(|card| {
                        [Rank::Ace, Rank::King, Rank::Queen, Rank::Ten, Rank::Jack]
                            .contains(&card.rank())
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
        }
    }

    pub fn mutate(self, std_dev: f32) -> Self {
        let mut rng = rand::rng();
        let dist = rand_distr::Normal::new(0.0, std_dev).unwrap();

        let starting_set = if rng.random_range(0.0..1.0) < std_dev {
            self.starting_set.0 ^ (1 << rng.random_range(1..=52))
        } else {
            self.starting_set.0
        };

        Self {
            flush_bias: (self.flush_bias + dist.sample(&mut rng)),
            straight_bias: (self.straight_bias + dist.sample(&mut rng)),
            pair_bias: (self.pair_bias + dist.sample(&mut rng)),
            high_card_bias: (self.high_card_bias + dist.sample(&mut rng)),
            dealer_strength_bias: (self.dealer_strength_bias + dist.sample(&mut rng)),
            score_threshold: (self.score_threshold + dist.sample(&mut rng)).clamp(0.0, 1.0),
            starting_set: CardSet(starting_set),
        }
    }

    fn straight_potential(mut hand: CardSet, card: Card) -> f32 {
        hand.insert(card);
        let set = hand.0;

        let mut best = 0;
        let mut count = 0;
        for i in std::iter::once(12).chain(0..13) {
            if set & (0xF << i) != 0 {
                count += 1;
                best = best.max(count);
            } else {
                count = 0;
            }
        }
        best as f32
    }

    fn basic_score(&self, hand: CardSet, card: Card) -> f32 {
        let mut score = 0.0;

        // Pair potential
        score += (hand.0 & (0xF << card.rank() as i32 * 4)).count_ones() as f32 * self.pair_bias;

        // Flush potential, TODO: non linear?
        score += (hand.0 & (0x1111111111111 << card.suit() as i32)).pow(2) as f32 * self.flush_bias;

        // Straight potential
        score += Self::straight_potential(hand, card).powf(2.0) * self.straight_bias;

        // High card potential
        score += (card.rank() as i32) as f32 * self.high_card_bias / 13.0;
        score
    }

    fn hand_strength(hand: CardSet) -> f32 {
        let set = hand.0;
        let mut groups = [0; 5];
        for i in 0..13 {
            groups[(set & (0b1111 << i)).count_ones() as usize] += 1;
        }
        let mut flush_count = 0;
        for i in 0..4 {
            flush_count = flush_count
                .max((set & (0x1111111111111 << i)).count_ones())
                .max(5);
        }
        return 2.0 * groups[2] as f32
            + 4.0 * groups[3] as f32
            + 8.0 * groups[4] as f32
            + (flush_count as f32).powf(2.0);
    }

    fn evaluate_card(&self, hand: CardSet, mut dealers_hand: CardSet, card: Card) -> f32 {
        let player_score = self.basic_score(hand, card);

        let dealer_strength = Self::hand_strength(dealers_hand);
        dealers_hand.insert(card);
        let new_dealer_strength = Self::hand_strength(dealers_hand);
        player_score - (new_dealer_strength - dealer_strength) * self.dealer_strength_bias
    }
}

impl Strategy for GeneticStrategy {
    fn decide(&mut self, hand: CardSet, dealers_hand: CardSet) -> CardSet {
        match hand.len() {
            0 => self.starting_set,
            _ => CardSet::from_cards(
                (!(hand | dealers_hand))
                    .into_iter()
                    .map(|card| (card, self.evaluate_card(hand, dealers_hand, card)))
                    .filter_map(|(card, score)| {
                        if score > self.score_threshold {
                            Some(card)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
        }
    }
}
