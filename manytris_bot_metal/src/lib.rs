use anyhow::{ensure, Context, Result};
use derive_more::{Display, Error};
use manytris_bot;
use manytris_bot::bot_start_positions::START_POSITIONS;
use manytris_bot::compute_types::{
    ComputedDropConfig, MoveResultScore, SearchParams, ShapePositionConfig, UpcomingShapes,
};
use manytris_bot::{BotContext, BotResults};
use manytris_core::bitmap_field::BitmapField;
use manytris_core::consts;
use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};
use std::cmp::min;

use std::slice;

pub struct BotShaderContext {
    kc: KernalConfig,
}

pub struct MetalBotResults {
    configs_buffer: Buffer,
    scores_buffer: Buffer,
}

struct KernalConfig {
    command_queue: CommandQueue,

    make_configs_pipeline_state: ComputePipelineState,
    computed_drop_pipeline_state: ComputePipelineState,

    device: Device,
}

#[derive(Debug, Display, Clone, Error)]
struct ObjCError {
    pub message: String,
}

impl BotShaderContext {
    pub fn new() -> Result<Self> {
        Ok(Self {
            kc: KernalConfig::prepare()?,
        })
    }
}

impl KernalConfig {
    fn prepare() -> Result<Self> {
        let library_data = include_bytes!("bot_shader.metallib");
        autoreleasepool(|| -> Result<KernalConfig> {
            let device = Device::system_default().context("No metal device available.")?;
            let library = device
                .new_library_with_data(&library_data[..])
                .map_err(into_objc)?;
            let command_queue = device.new_command_queue();

            let make_configs_function = library
                .get_function("compute_drop_config", None)
                .map_err(into_objc)?;
            let make_configs_pipeline_state = device
                .new_compute_pipeline_state_with_function(&make_configs_function)
                .map_err(into_objc)?;

            let computed_drop_function = library
                .get_function("drop_tetromino_for_config", None)
                .map_err(into_objc)?;
            let computed_drop_pipeline_state = device
                .new_compute_pipeline_state_with_function(&computed_drop_function)
                .map_err(into_objc)?;

            Ok(KernalConfig {
                command_queue,
                make_configs_pipeline_state,
                computed_drop_pipeline_state,
                device,
            })
        })
    }

    fn make_make_config_command_buffer(&self) -> (&CommandBufferRef, &ComputeCommandEncoderRef) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.make_configs_pipeline_state);

        (command_buffer, encoder)
    }

    fn make_computed_drop_command_buffer(&self) -> (&CommandBufferRef, &ComputeCommandEncoderRef) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.computed_drop_pipeline_state);

        (command_buffer, encoder)
    }

    fn make_data_buffer<T>(&self, items: usize) -> Buffer {
        self.device.new_buffer(
            (size_of::<T>() * items) as NSUInteger,
            MTLResourceOptions::StorageModeShared,
        )
    }
}

impl BotResults for MetalBotResults {
    fn configs(&self) -> &[ComputedDropConfig] {
        slice_from_buffer::<ComputedDropConfig>(&self.configs_buffer)
    }
    fn scores(&self) -> &[MoveResultScore] {
        slice_from_buffer::<MoveResultScore>(&self.scores_buffer)
    }
}

impl BotContext for BotShaderContext {
    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_field: &BitmapField,
    ) -> Result<impl BotResults> {
        let total_outputs = manytris_bot::num_outputs(search_depth);
        
        let configs_buffer = self
            .kc
            .make_data_buffer::<ComputedDropConfig>(total_outputs);
        let mut search_param_buffer = self.kc.make_data_buffer::<SearchParams>(1);

        let mut shape_position_config_buffer = self.kc.make_data_buffer::<ShapePositionConfig>(1);
        write_to_buffer(
            &mut shape_position_config_buffer,
            0,
            &START_POSITIONS.shape_position_config,
        );

        let mut fields_buffer = self.kc.make_data_buffer::<BitmapField>(total_outputs + 1);
        write_to_buffer(&mut fields_buffer, 0, source_field);

        let scores_buffer = self.kc.make_data_buffer::<MoveResultScore>(total_outputs);

        for cur_search_depth in 0..(search_depth as u8 + 1) {
            let sp = SearchParams {
                cur_search_depth,
                upcoming_shape_idxs: upcoming_shapes.map(|s| START_POSITIONS.shape_to_idx[s]),
            };

            write_to_buffer(&mut search_param_buffer, 0, &sp);

            let mut total_threads = consts::OUTPUTS_PER_INPUT_FIELD;
            (0..cur_search_depth).for_each(|_| total_threads *= consts::OUTPUTS_PER_INPUT_FIELD);
            let max_threads_per_threadgroup = self
                .kc
                .make_configs_pipeline_state
                .max_total_threads_per_threadgroup();

            let threads = MTLSize::new(total_threads as NSUInteger, 1, 1);
            let threads_per_threadgroup = MTLSize::new(
                min(total_threads as NSUInteger, max_threads_per_threadgroup),
                1,
                1,
            );

            autoreleasepool(|| {
                let (cmd_buffer, encoder) = self.kc.make_make_config_command_buffer();

                encoder.set_buffer(0, Some(&search_param_buffer), 0);
                encoder.set_buffer(1, Some(&configs_buffer), 0);

                encoder.dispatch_threads(threads, threads_per_threadgroup);
                encoder.end_encoding();

                cmd_buffer.commit();
                cmd_buffer.wait_until_completed();

                ensure!(
                    cmd_buffer.status() == MTLCommandBufferStatus::Completed,
                    "failed to make config command buffers"
                );
                Ok(())
            })?;

            autoreleasepool(|| {
                let (cmd_buffer, encoder) = self.kc.make_computed_drop_command_buffer();

                encoder.set_buffer(0, Some(&search_param_buffer), 0);
                encoder.set_buffer(1, Some(&shape_position_config_buffer), 0);
                encoder.set_buffer(2, Some(&fields_buffer), 0);
                encoder.set_buffer(3, Some(&configs_buffer), 0);
                encoder.set_buffer(4, Some(&scores_buffer), 0);

                encoder.dispatch_threads(threads, threads_per_threadgroup);
                encoder.end_encoding();

                cmd_buffer.commit();
                cmd_buffer.wait_until_completed();

                ensure!(
                    cmd_buffer.status() == MTLCommandBufferStatus::Completed,
                    "failed to compute scores"
                );
                Ok(())
            })?;
        }

        Ok(MetalBotResults {
            configs_buffer,
            scores_buffer,
        })
    }
}

fn slice_from_buffer<T>(buffer: &Buffer) -> &[T] {
    let items = buffer.length() as usize / size_of::<T>();
    unsafe { slice::from_raw_parts(buffer.contents() as *const T, items) }
}

fn slice_from_buffer_mut<T>(buffer: &mut Buffer) -> &mut [T] {
    let items = buffer.length() as usize / size_of::<T>();
    unsafe { slice::from_raw_parts_mut(buffer.contents() as *mut T, items) }
}

fn write_to_buffer<T: Clone>(buffer: &mut Buffer, index: usize, value: &T) {
    slice_from_buffer_mut(buffer)[index] = value.clone();
}

fn into_objc(message: String) -> ObjCError {
    ObjCError { message }
}

#[cfg(test)]
mod test {
    use std::cmp::max;

    use super::BotShaderContext;
    use manytris_bot::compute_types::ComputedDropConfig;
    use manytris_bot::{BotContext, BotResults};
    use manytris_core::bitmap_field::BitmapField;
    use manytris_core::shapes::Shape;

    #[test]
    fn verify_computed_configs() {
        let ctx = BotShaderContext::new().unwrap();

        let shapes = [
            Shape::I,
            Shape::J,
            Shape::L,
            Shape::I,
            Shape::I,
            Shape::I,
            Shape::I,
        ];

        let source = BitmapField::default();
        let results = ctx.compute_drop_search(1, &shapes, &source).unwrap();

        let mut expected_cfgs = vec![];
        let mut next_idx = 1;

        // First move depth
        for cw_rotations in 0..4 {
            for shifts in 0..10 {
                let left_shifts = max(4 - shifts, 0) as u8;
                let right_shifts = max(shifts - 4, 0) as u8;
                expected_cfgs.push(ComputedDropConfig {
                    shape_idx: 4,
                    cw_rotations,
                    left_shifts,
                    right_shifts,
                    src_field_idx: 0,
                    dest_field_idx: next_idx,
                });
                next_idx += 1;
            }
        }

        // Second move depth
        for src_field_idx in 1..next_idx {
            for cw_rotations in 0..4 {
                for shifts in 0..10 {
                    let left_shifts = max(4 - shifts, 0) as u8;
                    let right_shifts = max(shifts - 4, 0) as u8;
                    expected_cfgs.push(ComputedDropConfig {
                        shape_idx: 3,
                        cw_rotations,
                        left_shifts,
                        right_shifts,
                        src_field_idx,
                        dest_field_idx: next_idx,
                    });
                    next_idx += 1;
                }
            }
        }

        assert_eq!(results.configs().len(), expected_cfgs.len());
        assert_eq!(results.configs(), expected_cfgs);
    }
}
