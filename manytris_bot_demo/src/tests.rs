#![cfg(test)]

use anyhow::Result;
use manytris_bot::bot_cpu::CpuBotContext;
use manytris_bot::{BotContext, BotResults};
use manytris_bot_metal::BotShaderContext;
use manytris_core::game_state::GameState;
use manytris_core::shapes::Shape;

#[test]
fn verify_metal_consistent_moves() -> Result<()> {
    let metal_ctx = BotShaderContext::new()?;
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
    let metal_results = metal_ctx.compute_drop_search(2, &shapes, &source_state)?;
    let cpu_results = cpu_ctx.compute_drop_search(2, &shapes, &source_state)?;

    assert_eq!(cpu_results.configs(), metal_results.configs());
    assert_eq!(cpu_results.fields(), metal_results.fields());
    assert_eq!(cpu_results.scores(), metal_results.scores());

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
