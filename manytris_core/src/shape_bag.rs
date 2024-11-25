use crate::shapes::Shape;
use rand::{thread_rng, Rng};

#[derive(Default)]
pub struct ShapeBag {
    remaining: Vec<Shape>,
}
impl ShapeBag {
    fn take(&mut self) -> Shape {
        if self.remaining.is_empty() {
            self.remaining = enum_iterator::all::<Shape>().collect();
        }
        let idx = thread_rng().gen_range(0..self.remaining.len());
        self.remaining.remove(idx)
    }
}

impl Iterator for ShapeBag {
    type Item = Shape;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.take())
    }
}
