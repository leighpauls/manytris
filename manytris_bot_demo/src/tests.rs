#![cfg(test)]

use anyhow::{Context, Result};
use manytris_bot::bot_cpu::CpuBotContext;
use manytris_bot::compute_types::{ComputedDropConfig, MoveResultScore};
use manytris_bot::{BotContext, BotResults};
use manytris_bot_metal::BotShaderContext;
use manytris_bot_vulkan::VulkanBotContext;
use manytris_core::consts;
use manytris_core::field::{Field, Pos};
use manytris_core::game_state::GameState;
use manytris_core::shapes::Shape;

use pretty_assertions::assert_eq;

macro_rules! assert_lists_eq {
    ($left:expr, $right:expr) => {{
        assert_eq!($left.len(), $right.len(), "Lengths differ");
        for i in 0..$left.len() {
            assert_eq!($left[i], $right[i], "Element {i} differs");
        }
    }};
}

#[test]
fn verify_metal_consistent_moves() -> Result<()> {
    verify_consistent_moves(BotShaderContext::new()?)
}

#[test]
fn verify_vulkan_consistent_moves() -> Result<()> {
    verify_consistent_moves(VulkanBotContext::init()?)
}

fn verify_consistent_moves(compare_ctx: impl BotContext) -> Result<()> {
    let cpu_ctx = CpuBotContext;

    let shapes = [Shape::I; 7];

    let source_state = GameState::new(shapes.into());
    let metal_results = compare_ctx.compute_drop_search(2, &shapes, &source_state)?;
    let cpu_results = cpu_ctx.compute_drop_search(2, &shapes, &source_state)?;

    assert_lists_eq!(cpu_results.configs(), metal_results.configs());

    for cfg in metal_results.configs() {
        let dest_field_idx = cfg.dest_field_idx as usize;
        assert_eq!(
            cpu_results.fields()[dest_field_idx],
            metal_results.fields()[dest_field_idx],
            "Field mismatch, cfg {cfg:?}"
        );
    }

    assert_lists_eq!(cpu_results.fields(), metal_results.fields());
    assert_lists_eq!(cpu_results.scores(), metal_results.scores());

    Ok(())
}

#[test]
fn verify_search_depth() -> Result<()> {
    let ctx = CpuBotContext;

    use Shape::I;
    let upcoming_shapes = [I, I, I, I, I, I, I];
    let source_state = GameState::new(upcoming_shapes.into());
    {
        let result = ctx.compute_drop_search(0, &upcoming_shapes, &source_state)?;

        assert_eq!(result.fields().len(), 1);
        assert_eq!(result.configs().len(), 0);
        assert_eq!(result.scores().len(), 0);
    }

    {
        let result = ctx.compute_drop_search(1, &upcoming_shapes, &source_state)?;

        assert_eq!(result.fields().len(), 41);
        assert_eq!(result.configs().len(), 40);
        assert_eq!(result.scores().len(), 40);
    }

    {
        let result = ctx.compute_drop_search(2, &upcoming_shapes, &source_state)?;

        assert_eq!(result.fields().len(), 1641);
        assert_eq!(result.configs().len(), 1640);
        assert_eq!(result.scores().len(), 1640);
    }

    Ok(())
}

#[test]
fn verify_clear_lines_cpu() -> Result<()> {
    verify_clear_lines(CpuBotContext)
}

#[test]
fn verify_clear_lines_metal() -> Result<()> {
    verify_clear_lines(BotShaderContext::new()?)
}

#[test]
fn verify_clear_lines_vulkan() -> Result<()> {
    verify_clear_lines(VulkanBotContext::init()?)
}

fn verify_clear_lines(ctx: impl BotContext) -> Result<()> {
    let upcoming_shapes = [Shape::I; 7];

    let gs = GameState::with_initial_state(
        upcoming_shapes.into(),
        Field::with_initial_occupied((0..9).map(|x| Pos { x, y: 0 })),
    );

    let result = ctx.compute_drop_search(1, &upcoming_shapes, &gs)?;

    {
        let score = find_score(&result, |cfg| {
            cfg.cw_rotations == 1 && cfg.right_shifts == 4
        })?;
        assert_eq!(score, MoveResultScore::init(false, 1, 3, 0));
    }

    {
        let score = find_score(&result, |cfg| cfg.cw_rotations == 1 && cfg.left_shifts == 1)?;
        assert_eq!(score, MoveResultScore::init(false, 0, 5, 0));
    }

    Ok(())
}

#[test]
fn verify_covered_lines_cpu() -> Result<()> {
    verify_covered_lines(CpuBotContext)
}

#[test]
fn verify_covered_lines_metal() -> Result<()> {
    verify_covered_lines(BotShaderContext::new()?)
}

#[test]
fn verify_covered_lines_vulkan() -> Result<()> {
    verify_covered_lines(VulkanBotContext::init()?)
}

fn verify_covered_lines(ctx: impl BotContext) -> Result<()> {
    let upcoming_shapes = [Shape::I; 7];

    let gs = GameState::with_initial_state(
        upcoming_shapes.into(),
        Field::with_initial_occupied((0..9).map(|x| Pos { x, y: 0 })),
    );

    let result = ctx.compute_drop_search(1, &upcoming_shapes, &gs)?;

    {
        let score = find_score(&result, |cfg| cfg.cw_rotations == 0 && cfg.left_shifts == 4)?;
        assert_eq!(score, MoveResultScore::init(false, 0, 2, 0));
    }

    {
        let score = find_score(&result, |cfg| {
            cfg.cw_rotations == 0 && cfg.right_shifts == 3
        })?;
        assert_eq!(score, MoveResultScore::init(false, 0, 2, 1));
    }
    Ok(())
}

#[test]
fn verify_game_over_cpu() -> Result<()> {
    verify_game_over(CpuBotContext)
}

#[test]
fn verify_game_over_metal() -> Result<()> {
    verify_game_over(BotShaderContext::new()?)
}

#[test]
fn verify_game_over_vulkan() -> Result<()> {
    verify_game_over(VulkanBotContext::init()?)
}

fn verify_game_over(ctx: impl BotContext) -> Result<()> {
    let upcoming_shapes = [Shape::I; 7];

    let gs = GameState::with_initial_state(
        upcoming_shapes.into(),
        Field::with_initial_occupied((0..consts::MAX_H).map(|y| Pos { x: 4, y })),
    );

    let result = ctx.compute_drop_search(1, &upcoming_shapes, &gs)?;

    // all moves should be game over
    for score in result.scores() {
        assert_eq!(score.is_game_over(), true);
    }

    Ok(())
}

fn find_score<P>(result: &impl BotResults, predicate: P) -> Result<MoveResultScore>
where
    P: Fn(&ComputedDropConfig) -> bool,
{
    let (idx, _) = result
        .configs()
        .iter()
        .enumerate()
        .find(|(_, cfg)| predicate(cfg))
        .context("Couldn't find config matching predicate")?;

    Ok(result.scores()[idx])
}
