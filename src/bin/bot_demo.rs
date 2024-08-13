use manytris::bot_player;
use manytris::game_state::{GameState, TickMutation};
use manytris::plugins::shape_producer::ShapeProducer;
use std::iter;

pub fn main() {
    let mut sp = ShapeProducer::new();
    let inital_shapes = iter::repeat_with(|| sp.take()).take(7).collect();
    let mut gs = GameState::new(inital_shapes);

    println!("Initial:");
    println!("{}", gs);

    for i in 0..100 {
        if let Some(mr) = bot_player::enumerate_moves(&gs).max_by_key(|mr| mr.score.clone()) {
            if mr.score.game_over {
                println!("{}\n{}\nMoves: {}", mr.gs, mr.score, i);
                return;
            }

            gs = mr.gs;
            gs.tick_mutation(vec![TickMutation::EnqueueTetromino(sp.take())]);
        }
    }
    println!("{}\nDid not lose!", gs);
}
