use crate::game_state::{GameState, Shift, Tetromino};

mod game_state;

fn main() {
    println!("Hello, world!");
    let mut gs = GameState::new();
    gs.print();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.shift(Shift::Right);
    gs.print();
    gs.lock_active_tetromino();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.print();
}
