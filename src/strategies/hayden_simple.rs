use crate::{
    card::{Card, CardSet, Rank, Suit},
    evaluator::{HandType, evaluate_many},
    game::Strategy,
};

#[derive(Default)]
pub struct HaydenSimple {
    force_flush: bool,
}

fn top_up_set(mut set: CardSet, min_size: usize) -> CardSet {
    for i in (0..13).rev() {
        if set.len() >= min_size {
            break;
        }
        set |= CardSet::from_cards(
            (1..=52)
                .map(Card::from)
                .filter(|card| card.rank() as u64 == i)
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }
    set
}

fn is_ahead(hand: &[Card], dealers_hand: &[Card]) -> bool {
    let mut rank_counts = [0; 13];
    for card in hand {
        rank_counts[card.rank() as usize] += 1;
    }

    if dealers_hand.len() < 5 {
        let mut dealer_rank_counts = [0; 13];
        for card in dealers_hand {
            dealer_rank_counts[card.rank() as usize] += 1;
        }

        let freq_counts = rank_counts.into_iter().fold([0; 4], |mut acc, c| {
            if c > 0 {
                acc[c as usize - 1] += 1;
            }
            acc
        });

        let dealer_freq_counts = dealer_rank_counts.into_iter().fold([0; 4], |mut acc, c| {
            if c > 0 {
                acc[c as usize - 1] += 1;
            }
            acc
        });

        for i in (0..4).rev() {
            if freq_counts[i] > dealer_freq_counts[i] {
                return true;
            } else if freq_counts[i] == dealer_freq_counts[i] {
                let rank = rank_counts
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(j, &c)| if c == freq_counts[i] { Some(j) } else { None });
                let dealer_rank =
                    dealer_rank_counts
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(j, &c)| {
                            if c == dealer_freq_counts[i] {
                                Some(j)
                            } else {
                                None
                            }
                        });
                return rank > dealer_rank;
            } else if freq_counts[i] < dealer_freq_counts[i] {
                return false;
            }
        }

        return false;
    }

    let dealer_rank = evaluate_many(dealers_hand);

    if rank_counts.iter().max().unwrap_or(&0) == &4 {
        return dealer_rank.hand_type() < HandType::FourOfAKind;
    }

    if rank_counts.iter().max().unwrap_or(&0) == &3 {
        if rank_counts.iter().filter(|&&c| c == 2).count() > 0 {
            return dealer_rank.hand_type() < HandType::FullHouse;
        } else {
            return dealer_rank.hand_type() < HandType::ThreeOfAKind;
        }
    }

    if rank_counts.iter().filter(|&&c| c == 2).count() >= 2 {
        return dealer_rank.hand_type() < HandType::TwoPair;
    }

    if rank_counts.iter().filter(|&&c| c == 2).count() == 1 {
        return dealer_rank.hand_type() < HandType::OnePair;
    }

    false
}

impl Strategy for HaydenSimple {
    fn decide(
        &mut self,
        hand: &[crate::card::Card],
        dealers_hand: &[crate::card::Card],
    ) -> crate::card::CardSet {
        match hand.len() {
            0 => CardSet::from_cards(
                (1..=52)
                    .map(Card::from)
                    .filter(|card| {
                        [Rank::Ace, Rank::King, Rank::Queen, Rank::Ten, Rank::Five]
                            .contains(&card.rank())
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            1 => {
                if dealers_hand.len() >= 8 && is_ahead(hand, dealers_hand) {
                    return !CardSet::default();
                }

                let mut board_ranks = 0;
                for card in hand.iter().chain(dealers_hand.iter()) {
                    board_ranks |= 1 << card.rank() as u64;
                }
                let set = CardSet::from_cards(
                    (1..=52)
                        .map(Card::from)
                        .filter(|card| {
                            let rank_bit = 1 << card.rank() as u64;
                            (rank_bit & board_ranks != 0)
                                || card.suit() == hand[0].suit()
                                || [Rank::Ten, Rank::Five].contains(&card.rank())
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                top_up_set(set, 19)
            }
            2 => {
                if dealers_hand.len() >= 8 && is_ahead(hand, dealers_hand) {
                    return !CardSet::default();
                }

                if hand[0].suit() == hand[1].suit() && dealers_hand.len() <= 1 {
                    self.force_flush = true;
                    return CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| card.suit() == hand[0].suit())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                }

                let mut board_ranks = 0;
                for card in hand.iter().chain(dealers_hand.iter()) {
                    board_ranks |= 1 << card.rank() as u64;
                }
                let set = CardSet::from_cards(
                    (1..=52)
                        .map(Card::from)
                        .filter(|card| {
                            let rank_bit = 1 << card.rank() as u64;
                            let mut allow = (rank_bit & board_ranks != 0)
                                || [Rank::Ten, Rank::Five].contains(&card.rank());
                            if hand[0].suit() == hand[1].suit() {
                                allow = allow || (card.suit() == hand[0].suit());
                            }
                            allow
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                top_up_set(set, 19)
            }
            3 => {
                if dealers_hand.len() >= 8 && is_ahead(hand, dealers_hand) {
                    return !CardSet::default();
                }

                if self.force_flush
                    || (hand[0].suit() == hand[1].suit()
                        && hand[1].suit() == hand[2].suit()
                        && dealers_hand.len() <= 5)
                {
                    self.force_flush = true;
                    return CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| card.suit() == hand[0].suit())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                }

                let mut board_ranks = 0;
                for card in hand.iter().chain(dealers_hand.iter()) {
                    board_ranks |= 1 << card.rank() as u64;
                }
                if hand[0].suit() == hand[1].suit() && hand[1].suit() == hand[2].suit() {
                    CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| {
                                let rank_bit = 1 << card.rank() as u64;
                                (rank_bit & board_ranks != 0) || card.suit() == hand[0].suit()
                            })
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                } else {
                    top_up_set(
                        CardSet::from_cards(
                            (1..=52)
                                .map(Card::from)
                                .filter(|card| {
                                    let rank_bit = 1 << card.rank() as u64;
                                    (rank_bit & board_ranks != 0)
                                        || [Rank::Ten, Rank::Five].contains(&card.rank())
                                })
                                .collect::<Vec<_>>()
                                .as_slice(),
                        ),
                        19,
                    )
                }
            }
            4 => {
                let ahead = is_ahead(hand, dealers_hand);
                if dealers_hand.len() >= 8 && ahead {
                    return !CardSet::default();
                }

                if self.force_flush {
                    return CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| card.suit() == hand[0].suit())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                }

                let board_ranks = hand
                    .iter()
                    .chain(dealers_hand.iter())
                    .fold(0, |acc, card| acc | (1 << card.rank() as u64));

                if ahead {
                    CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| {
                                let rank_bit = 1 << card.rank() as u64;
                                (rank_bit & board_ranks != 0)
                                    || [Rank::Ten, Rank::Five].contains(&card.rank())
                            })
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                } else {
                    // TODO: Request cards that improve the hand, not just any card
                    !CardSet::default()
                }
            }
            _ => unreachable!(),
        }
    }
}
