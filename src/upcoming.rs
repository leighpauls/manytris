use crate::consts;
use crate::shapes::Shape;
use rand::prelude::*;

pub struct UpcomingTetrominios {
    upcoming_blocks: Vec<Shape>,
}

impl UpcomingTetrominios {
    pub fn new(initial_state: Vec<Shape>) -> Self {
        UpcomingTetrominios {
            upcoming_blocks: initial_state,
        }
    }

    pub fn preview(&self) -> [Shape; consts::NUM_PREVIEWS] {
        self.upcoming_blocks[0..consts::NUM_PREVIEWS]
            .try_into()
            .unwrap()
    }

    pub fn take(&mut self) -> Shape {
        self.upcoming_blocks.remove(0)
    }

    pub fn enqueue(&mut self, shape: Shape) {
        self.upcoming_blocks.push(shape);
    }
}
