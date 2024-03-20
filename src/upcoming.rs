use crate::shapes::Shape;
use rand::prelude::*;

const NUM_PREVIEWS: usize = 6;

pub struct UpcomingTetrominios {
    upcoming_blocks: Vec<Shape>,
    bag_remaining: Vec<Shape>,
}

impl UpcomingTetrominios {
    pub fn new() -> Self {
        let mut ut = UpcomingTetrominios {
            upcoming_blocks: vec![],
            bag_remaining: vec![],
        };

        ut.refill();
        ut
    }

    pub fn preview(&self) -> &[Shape] {
        &self.upcoming_blocks
    }

    pub fn take(&mut self) -> Shape {
        let res = self.upcoming_blocks.remove(0);
        self.refill();
        res
    }

    fn refill(&mut self) {
        while self.upcoming_blocks.len() < NUM_PREVIEWS {
            if self.bag_remaining.is_empty() {
                self.bag_remaining = enum_iterator::all::<Shape>().collect();
            }
            let next_idx = thread_rng().next_u32() as usize % self.bag_remaining.len();
            self.upcoming_blocks
                .push(self.bag_remaining.remove(next_idx));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::upcoming::{UpcomingTetrominios, NUM_PREVIEWS};

    #[test]
    fn test() {
        let mut ut = UpcomingTetrominios::new();
        assert_eq!(ut.upcoming_blocks.len(), NUM_PREVIEWS);

        let s = ut.take();
        assert_eq!(ut.upcoming_blocks.len(), NUM_PREVIEWS);

    }
}
