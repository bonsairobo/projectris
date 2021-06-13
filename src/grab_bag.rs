use crate::{PieceType, ALL_PIECE_TYPES};

use rand::{prelude::SliceRandom, thread_rng};

pub struct GrabBag {
    repeats_per_bag: usize,
    bag: Vec<PieceType>,
}

impl GrabBag {
    pub fn new(repeats_per_bag: usize) -> Self {
        let mut bag = Self {
            repeats_per_bag,
            bag: Vec::new(),
        };
        bag.refill();

        bag
    }

    fn refill(&mut self) {
        let num_pieces = ALL_PIECE_TYPES.len();
        self.bag = ALL_PIECE_TYPES
            .iter()
            .cloned()
            .cycle()
            .take(num_pieces * self.repeats_per_bag)
            .collect();
        self.bag.shuffle(&mut thread_rng());
    }

    pub fn choose_next_piece_type(&mut self) -> PieceType {
        if let Some(next_type) = self.bag.pop() {
            next_type
        } else {
            self.refill();

            self.bag.pop().unwrap()
        }
    }
}
