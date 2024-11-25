use genetic_algorithm::strategy::evolve::prelude::*;
use rand::thread_rng;
use manytris_bot::bot_player;
use manytris_bot::bot_player::ScoringKs;
use manytris_bot::bot_shader::BotShaderContext;
use manytris_core::consts;
use manytris_core::game_state::{GameState, TickMutation};
use manytris_core::shape_bag::ShapeBag;

const SEARCH_DEPTH: usize = 3;

pub fn main() {
    println!("Start test games...");
    for _ in 0..4 {
        println!("Game length {}", run_game(&bot_player::BEST_BOT_KS, 600));
    }

    println!("Start evolving...");
    let genotype = ContinuousGenotype::builder()
        .with_genes_size(4)
        .with_allele_range(-10000.0..10000.0)
        .build()
        .unwrap();

    let mut rng = thread_rng();
    let evolve = Evolve::builder()
        .with_genotype(genotype)
        .with_target_population_size(50)
        .with_target_fitness_score(550)
        .with_fitness(GameFitness)
        .with_fitness_ordering(FitnessOrdering::Maximize)
        .with_multithreading(true)
        .with_crossover(CrossoverUniform::new(true))
        .with_mutate(MutateSingleGeneRandom::new(0.1))
        .with_compete(CompeteElite::new())
        .with_reporter(PrintBestReporter)
        .call(&mut rng)
        .unwrap();

    let bc = evolve.best_chromosome().unwrap();
    println!("Best chromosome: {:?}", bc);

    println!("Best chromosome genes: {:?}", bc.genes as Vec<f32>);
}

#[derive(Clone, Debug)]
pub struct PrintBestReporter;

impl EvolveReporter for PrintBestReporter {
    type Genotype = ContinuousGenotype;

    fn on_new_best_chromosome(
        &mut self,
        state: &EvolveState<Self::Genotype>,
        _config: &EvolveConfig,
    ) {
        if let Some(c) = &state.best_chromosome {
            println!("New best chromosome: {:?}", c);
        }
    }

    fn on_new_generation(&mut self, state: &EvolveState<Self::Genotype>, _config: &EvolveConfig) {
        println!(
            "Generation: {}, Score Cardinality: {}, Score Median: {:?}",
            state.current_generation,
            state.population.fitness_score_cardinality(),
            state.population.fitness_score_median(),
        );
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
    let num_games = 10;
    let mut worst_score = 600;
    for _ in 0..num_games {
        let score = run_game(ks, worst_score);
        if score < worst_score {
            worst_score = score;
        }
    }
    worst_score
}

fn run_game(ks: &ScoringKs, max_game_length: i32) -> i32 {
    let mut shape_bag = ShapeBag::default();
    let initial_shapes = shape_bag.by_ref().take(consts::NUM_PREVIEWS * 2).collect();
    let mut gs = GameState::new(initial_shapes);

    let bot_context = BotShaderContext::new().unwrap();

    for i in 0..max_game_length {
        let mr = bot_player::select_next_move(&gs, &bot_context, ks, SEARCH_DEPTH).unwrap();

        if mr.score.game_over {
            return i;
        }
        // Evaluate 1 move on the best result.
        (gs, _) = bot_player::evaluate_moves_cpu(&gs, &mr.moves[0..1], &bot_context.sp);
        gs.tick_mutation(vec![TickMutation::EnqueueTetromino(
            shape_bag.next().unwrap(),
        )]);
    }
    max_game_length
}
