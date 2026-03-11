use std::collections::HashSet;

use once_cell::sync::Lazy;
use rand::{rngs::SmallRng, seq::SliceRandom};

use crate::{
    card::{Card, CardSet},
    evaluator::{HandRank, evaluate, evaluate_many},
    game::Strategy,
};

const MAX_DECKS: usize = 400;

static PRECOMPUTE_BINOM: Lazy<[[f64; 53]; 53]> = Lazy::new(|| {
    let mut c = [[0.0; 53]; 53];

    for n in 0..=52 {
        c[n][0] = 1.0;
        c[n][n] = 1.0;

        for k in 1..n {
            c[n][k] = c[n - 1][k - 1] + c[n - 1][k];
        }
    }

    c
});

type DealerDist = Vec<(usize, f64)>;
#[allow(non_snake_case)]
static PRECOMPUTE_DEALER_DISTRIBUTION: Lazy<Vec<Vec<[DealerDist; 6]>>> = Lazy::new(|| {
    let mut table = vec![vec![std::array::from_fn(|_| Vec::new()); 53]; 53];

    for N in 0..=52 {
        for K in 0..=N {
            for r in 1..=5 {
                if K < r {
                    continue;
                }

                let mut dist = Vec::new();

                let min_n = r;
                let max_n = N - (K - r);

                for n in min_n..=max_n {
                    let p = comb(n - 1, r - 1) * comb(N - n, K - r) / comb(N, K);

                    let dealer_cards = n - r;

                    dist.push((dealer_cards, p));
                }

                table[N][K][r] = dist;
            }
        }
    }

    table
});

#[inline(always)]
fn comb(n: usize, k: usize) -> f64 {
    return PRECOMPUTE_BINOM[n][k];

    // if k > n {
    //     return 0.0;
    // }

    // let mut num = 1.0;
    // let mut den = 1.0;

    // for i in 0..k {
    //     num *= (n - i) as f64;
    //     den *= (i + 1) as f64;
    // }

    // num / den
}

#[inline(always)]
#[allow(non_snake_case)]
fn dealer_distribution(N: usize, K: usize, r: usize) -> &'static Vec<(usize, f64)> {
    &PRECOMPUTE_DEALER_DISTRIBUTION[N][K][r]
    // let min_n = r;
    // let max_n = N - K + r;

    // let mut out = Vec::new();

    // for n in min_n..=max_n {
    //     let p = comb(n - 1, r - 1) * comb(N - n, K - r) / comb(N, K);
    //     let dealer_cards = n - r;
    //     out.push((dealer_cards, p));
    // }

    // out
}

fn sample_hands(
    mut player: CardSet,
    mut dealer: CardSet,
    deck_cards: &[Card],
    subset: CardSet,
    dealer_cards: usize,
) -> (CardSet, CardSet) {
    let dealer_cards = dealer_cards.max(8);

    for &card in deck_cards {
        if subset.contains(card) && player.len() < 5 {
            player.insert(card);
        } else if dealer.len() < dealer_cards {
            dealer.insert(card);
        } else if player.len() == 5 {
            break;
        }
    }

    (player, dealer)
}

fn generate_candidate_subsets(player: CardSet, _dealer: CardSet, deck: CardSet) -> Vec<CardSet> {
    const SUIT_MASK: u64 = 0x1111111111111;
    const HIGH_MASK: u64 = 0xFFFFF00000000;

    // ---------- Flush strategies ----------
    let mut flush_subsets = Vec::new();
    for suit in 0..4 {
        let suit_mask = CardSet(SUIT_MASK << suit);
        if (player & suit_mask).len() == player.len() {
            flush_subsets.push(deck & suit_mask);

            // flush + high cards
            for cutoff in 6..11 {
                let rank_mask = !CardSet((1u64 << (cutoff * 4)) - 1);
                flush_subsets.push(deck & suit_mask & rank_mask);
            }

            // ---------- Straight flush ------------
            for start in 0..9 {
                let mut mask = CardSet::default();

                for r in start..start + 5 {
                    mask |= CardSet(0b1111 << (r * 4));
                }

                if (player & suit_mask & mask & deck).len() == player.len() {
                    flush_subsets.push(deck & suit_mask & mask);
                }
            }

            let wheel_mask = deck & suit_mask & CardSet(0xF00000000FFFF);
            if (player & wheel_mask).len() == player.len() {
                flush_subsets.push(wheel_mask); // Wheel straight flush
            }
        }
    }

    // ---------- Straight windows ----------
    let mut straight_subsets = Vec::new();
    for start in 0..9 {
        let mut mask = CardSet::default();
        for r in start..start + 5 {
            mask |= CardSet(0b1111 << (r * 4));
        }

        if (player & deck & mask).len() == player.len() {
            straight_subsets.push(deck & mask);
        }
    }

    // ---------- Player rank focus ----------
    let mut player_ranks = CardSet::default();
    let mut rank_subsets = Vec::new();
    for rank in 0..13 {
        if player.0 & (0xF << (rank * 4)) != 0 {
            player_ranks |= CardSet(0xF << (rank * 4));
        }
    }

    if player_ranks.len() > 0 {
        rank_subsets.push(deck & player_ranks);
    }

    // ---------- High cards ----------
    rank_subsets.push(deck & CardSet(HIGH_MASK));

    let mut subsets = HashSet::new();

    // Mix flush and straight strategies
    for &s in &straight_subsets {
        subsets.insert(s);
        for &f in &flush_subsets {
            subsets.insert(f);
            subsets.insert(f | s);
        }
    }

    // Mix flush and rank strategies
    for &f in &flush_subsets {
        subsets.insert(f);
        for &r in &rank_subsets {
            subsets.insert(r);
            subsets.insert(f | r);
        }
    }

    // Mix straight and rank strategies
    for &r in &rank_subsets {
        subsets.insert(r);
        for &s in &straight_subsets {
            subsets.insert(s);
            subsets.insert(s | r);
        }
    }

    subsets.into_iter().collect()
}

fn samples_for_bucket(prob: f64) -> usize {
    let scaled = (MAX_DECKS as f64 * prob.sqrt()) as usize;
    scaled.max(10)
}

#[allow(dead_code)]
pub struct HypergeometricStrategy;

impl Strategy for HypergeometricStrategy {
    fn decide(&mut self, player: CardSet, dealer: CardSet) -> CardSet {
        if player.len() == 0 {
            return CardSet(0xFFFFF00000000);
        }

        let mut rng: SmallRng = rand::make_rng();
        let deck = !CardSet::default() & !player & !dealer;
        let deck_cards = deck.to_cards();

        let mut best_subset = CardSet::default();
        let mut best_ev = f64::NEG_INFINITY;
        let r = 5 - player.len();

        let mut shuffled_decks = Vec::with_capacity(MAX_DECKS);
        for _ in 0..MAX_DECKS {
            let mut d = deck_cards.clone();
            d.shuffle(&mut rng);
            shuffled_decks.push(d);
        }

        for mut subset in generate_candidate_subsets(player, dealer, deck) {
            subset &= deck;
            let subset_size = subset.len();
            let deck_size = deck.len();

            if subset_size < r {
                continue;
            }

            let distribution = dealer_distribution(deck_size, subset_size, r);

            let mut ev = 0.0;

            for &(dealer_cards, prob) in distribution {
                if prob < 1e-6 {
                    continue;
                }

                let samples = samples_for_bucket(prob);
                let mut bucket_score = 0.0;

                for shuffled_deck in shuffled_decks.iter().take(samples) {
                    let (p_hand, d_hand) =
                        sample_hands(player, dealer, shuffled_deck, subset, dealer_cards);

                    let p_rank = evaluate(p_hand);
                    // let d_rank = evaluate_many(d_hand);
                    let d_rank = evaluate_many(d_hand);

                    if p_rank > d_rank {
                        bucket_score += 1.0;
                    }
                }

                bucket_score /= samples.min(shuffled_decks.len()) as f64;

                ev += bucket_score * prob;
            }

            if ev > best_ev {
                best_ev = ev;
                best_subset = subset;
            }
        }

        best_subset
    }
}
