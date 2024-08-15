use genetic_algorithm::strategy::evolve::prelude::*;

use manytris::bot_player;
use manytris::bot_player::ScoringKs;
use manytris::game_state::{GameState, TickMutation};
use manytris::plugins::shape_producer::ShapeProducer;
use std::iter;

use ordered_float::OrderedFloat;

pub fn main() {

    let best_ks =  [-863.55994, 24.596436, -651.4709, -825.0811];
    println!("Game length {}", run_game(&best_ks));

    let genotype = ContinuousGenotype::builder()
        .with_genes_size(4)
        .with_allele_range(-1000.0..1000.0)
        .build()
        .unwrap();

    let mut rng = rand::thread_rng();
    let evolve = Evolve::builder()
        .with_genotype(genotype)
        .with_target_population_size(100)
        .with_target_fitness_score(400)
        .with_fitness(GameFitness)
        .with_fitness_ordering(FitnessOrdering::Maximize)
        .with_multithreading(true)
        .with_crossover(CrossoverUniform::new(true))
        .with_mutate(MutateSingleGeneRandom::new(0.02))
        .with_compete(CompeteElite::new())
        .with_reporter(PrintBestReporter)
        .call(&mut rng)
        .unwrap();

    let bc = evolve.best_chromosome().unwrap();
    println!("Best chromosome: {:?}", bc);

    println!("Best chromosome genes: {:?}", bc.genes as Vec<f32>);

    /*
    let ks: ScoringKs = [-100.0, 10.0, -5.0, -10.0];
    println!("Game length {}", run_game(&ks));
     */
}

#[derive(Clone, Debug)]
pub struct PrintBestReporter;

impl EvolveReporter for PrintBestReporter {
    type Genotype = ContinuousGenotype;

    fn on_new_best_chromosome(&mut self, state: &EvolveState<Self::Genotype>, _config: &EvolveConfig) {
        if let Some(c) = &state.best_chromosome {
            println!("New best chromosome: {:?}", c);
        }
    }

    fn on_new_generation(&mut self, state: &EvolveState<Self::Genotype>, _config: &EvolveConfig) {
        println!("Generation: {}", state.current_generation);
    }
}

#[derive(Clone, Debug)]
pub struct GameFitness;

impl Fitness for GameFitness {
    type Genotype = ContinuousGenotype;

    fn calculate_for_chromosome(
        &mut self,
        chromosome: &Chromosome<Self::Genotype>,
    ) -> Option<FitnessValue> {
        let ks: ScoringKs = chromosome.genes.clone().try_into().unwrap();
        Some(evaluate_ks(&ks) as FitnessValue)
    }
}

fn evaluate_ks(ks: &ScoringKs) -> i32 {
    let num_games = 4;
    let mut score = 0;
    for _ in 0..num_games {
        score += run_game(ks);
    }
    score / num_games
}

fn run_game(ks: &ScoringKs) -> i32 {
    let max_game_length = 500;

    let mut sp = ShapeProducer::new();
    let inital_shapes = iter::repeat_with(|| sp.take()).take(7).collect();
    let mut gs = GameState::new(inital_shapes);

    for i in 0..max_game_length {
        let mr = bot_player::enumerate_moves(&gs)
            .max_by_key(|mr| OrderedFloat(bot_player::weighted_result_score(&mr.score, &ks)))
            .unwrap();
        if mr.score.game_over {
            return i;
        }
        gs = mr.gs;
        gs.tick_mutation(vec![TickMutation::EnqueueTetromino(sp.take())]);
    }
    max_game_length
}
