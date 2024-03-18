use crate::game_state::Tetromino;

mod game_state;

fn main() {
    println!("Hello, world!");
    let mut gs = game_state::GameState::new();
    gs.print();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.print();
    gs.lock_active_tetromino();
    gs.new_active_tetromino(Tetromino::new());
    gs.print();
    gs.down();
    gs.print();
}
