use crate::{
    card::{Card, CardSet, Rank, Suit},
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
                let mut board_ranks = 0;
                for card in hand.iter().chain(dealers_hand.iter()) {
                    board_ranks |= 1 << card.rank() as u64;
                }
                let mut set = CardSet::from_cards(
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
                let mut set = CardSet::from_cards(
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
                if self.force_flush {
                    return CardSet::from_cards(
                        (1..=52)
                            .map(Card::from)
                            .filter(|card| card.suit() == hand[0].suit())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                }

                !CardSet::default()
            }
            _ => unreachable!(),
        }
    }
}
