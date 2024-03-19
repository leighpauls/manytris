use crate::game_state::{GameState, Shift, Tetromino};

mod game_state;
mod shapes;

fn main() {
    println!("Hello, world!");
    let mut gs = GameState::new();
    gs.print();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.shift(Shift::Right);
    gs.print();
    gs.drop();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.cw();
    gs.print();
}
