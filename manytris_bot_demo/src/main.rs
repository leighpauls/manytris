mod tests;

use std::fmt::Debug;
use std::time::Instant;

use anyhow::{Context, Result};
use genetic_algorithm::strategy::evolve::prelude::*;
use manytris_bot::bot_player::ScoringKs;
use manytris_bot::{bot_player, BotContext};
use manytris_bot_metal::BotShaderContext;
use manytris_core::consts;
use manytris_core::game_state::{GameState, TickMutation};
use manytris_core::shape_bag::ShapeBag;
use rand::thread_rng;

const SEARCH_DEPTH: usize = 3;

pub fn main() -> Result<()> {
    {
        println!("Start metal test games...");
        let metal_bot = BotShaderContext::new()?;
        for _ in 0..4 {
            println!(
                "Game results {:?}",
                run_game(&bot_player::BEST_BOT_KS, 600, &metal_bot)
            );
        }
    }

    {
        println!("Start vulkan test games...");
        let metal_bot = BotShaderContext::new()?;
        for _ in 0..4 {
            println!(
                "Game results {:?}",
                run_game(&bot_player::BEST_BOT_KS, 600, &metal_bot)
            );
        }
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
        .with_fitness(GameFitness {
            context_ctor: || BotShaderContext::new().unwrap(),
        })
        .with_fitness_ordering(FitnessOrdering::Maximize)
        .with_multithreading(true)
        .with_crossover(CrossoverUniform::new(true))
        .with_mutate(MutateSingleGeneRandom::new(0.1))
        .with_compete(CompeteElite::new())
        .with_reporter(PrintBestReporter)
        .call(&mut rng)
        .unwrap();

    let bc = evolve
        .best_chromosome()
        .context("Couldn't get best chromosome")?;
    println!("Best chromosome: {:?}", bc);

    println!("Best chromosome genes: {:?}", bc.genes as Vec<f32>);
    Ok(())
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

pub struct GameFitness<C, F>
where
    C: BotContext,
    F: Fn() -> C + Send + Sync + Clone,
{
    context_ctor: F,
}

impl<C, F> Fitness for GameFitness<C, F>
where
    C: BotContext,
    F: Fn() -> C + Send + Sync + Clone,
{
    type Genotype = ContinuousGenotype;

    fn calculate_for_chromosome(
        &mut self,
        chromosome: &Chromosome<Self::Genotype>,
    ) -> Option<FitnessValue> {
        let ks: ScoringKs = chromosome.genes.clone().try_into().unwrap();
        let ctx = (self.context_ctor)();
        Some(evaluate_ks(&ks, &ctx))
    }
}

impl<C, F> Debug for GameFitness<C, F>
where
    C: BotContext,
    F: Fn() -> C + Send + Sync + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GameFitness")
    }
}

impl<C, F> Clone for GameFitness<C, F>
where
    C: BotContext,
    F: Fn() -> C + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self {
            context_ctor: self.context_ctor.clone(),
        }
    }
}

fn evaluate_ks(ks: &ScoringKs, bot_context: &impl BotContext) -> FitnessValue {
    let num_games = 10;
    let mut worst_score = 600;
    for _ in 0..num_games {
        let res = run_game(ks, worst_score, bot_context);
        if res.game_length < worst_score {
            worst_score = res.game_length;
        }
    }
    worst_score as FitnessValue
}

#[derive(Debug)]
struct RunGameResults {
    game_length: usize,
    moves_per_second: f64,
}

fn run_game(
    ks: &ScoringKs,
    max_game_length: usize,
    bot_context: &impl BotContext,
) -> RunGameResults {
    let mut shape_bag = ShapeBag::default();
    let initial_shapes = shape_bag.by_ref().take(consts::NUM_PREVIEWS * 2).collect();
    let mut gs = GameState::new(initial_shapes);

    let start_time = Instant::now();
    let mut game_length = 0;

    while game_length < max_game_length {
        let mr = bot_player::select_next_move(&gs, bot_context, ks, SEARCH_DEPTH).unwrap();

        if mr.score.is_game_over() {
            break;
        }
        game_length += 1;

        // Evaluate 1 move on the best result.
        (gs, _, _) = manytris_bot::evaluate_moves_cpu(&gs, &mr.moves[0..1]);
        gs.tick_mutation(vec![TickMutation::EnqueueTetromino(
            shape_bag.next().unwrap(),
        )]);
    }

    let end_time = Instant::now();
    RunGameResults {
        game_length,
        moves_per_second: game_length as f64 / (end_time - start_time).as_secs_f64(),
    }
}
