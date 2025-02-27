#![cfg(test)]

use anyhow::Result;
use manytris_bot::bot_cpu::CpuBotContext;
use manytris_bot::{BotContext, BotResults};
use manytris_bot_metal::BotShaderContext;
use manytris_bot_vulkan::VulkanBotContext;
use manytris_core::game_state::GameState;
use manytris_core::shapes::Shape;

use pretty_assertions::assert_eq;

macro_rules! assert_lists_eq {
    ($left:expr, $right:expr) => ({
        assert_eq!($left.len(), $right.len(), "Lengths differ");
        for i in 0..$left.len() {
            assert_eq!($left[i], $right[i], "Element {i} differs");
        }
    });
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

    let shapes = [
        Shape::I,
        Shape::J,
        Shape::L,
        Shape::I,
        Shape::I,
        Shape::I,
        Shape::I,
    ];

    let source_state = GameState::new(shapes.into());
    let metal_results = compare_ctx.compute_drop_search(1, &shapes, &source_state)?;
    let cpu_results = cpu_ctx.compute_drop_search(1, &shapes, &source_state)?;

    assert_lists_eq!(cpu_results.configs(), metal_results.configs());

    for cfg in metal_results.configs() {
        let dest_field_idx = cfg.dest_field_idx as usize;
        assert_eq!(cpu_results.fields()[dest_field_idx], metal_results.fields()[dest_field_idx], "Field mismatch, cfg {cfg:?}");
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
