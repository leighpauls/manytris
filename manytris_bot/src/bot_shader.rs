use std::cmp::min;
use std::mem::size_of;
use std::slice;

use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};
use ordered_float::OrderedFloat;

use crate::bot_player::{MoveResult, MovementDescriptor};
use crate::bot_start_positions::StartPositions;
use crate::compute_types::{
    ComputedDropConfig, MoveResultScore, SearchParams, ShapePositionConfig,
};
use manytris_core::consts;
use manytris_core::shapes::Shape;

use manytris_core::bitmap_field::BitmapField;

pub struct BotShaderContext {
    kc: KernalConfig,
    pub sp: StartPositions,
}

pub type UpcomingShapes = [Shape; consts::MAX_SEARCH_DEPTH + 1];
pub struct ComputedDropSearchResults {
    pub search_depth: usize,
    pub upcoming_shapes: UpcomingShapes,
    pub drops: Vec<MovementDescriptor>,
    pub score: MoveResultScore,
}

impl ComputedDropSearchResults {
    pub fn find_results<F: Fn(&MoveResultScore) -> OrderedFloat<f32>>(
        search_depth: usize,
        upcoming_shapes: UpcomingShapes,
        configs: &[ComputedDropConfig],
        scores: &[MoveResultScore],
        scoring_fn: F,
        sp: &StartPositions,
    ) -> Self {
        let (start_idx, end_idx) = Self::idx_range(search_depth);
        assert_eq!(end_idx, scores.len());

        // Find the best score
        let (best_idx, best_score) = scores[start_idx..end_idx]
            .into_iter()
            .enumerate()
            .max_by_key(|(_i, s)| scoring_fn(s))
            .unwrap();

        let mut next_config_idx = start_idx + best_idx;
        let mut moves = vec![];
        loop {
            let cfg = &configs[next_config_idx];
            moves.insert(0, cfg.as_move_descriptor(sp));

            if cfg.src_field_idx == 0 {
                break;
            }
            next_config_idx = cfg.src_field_idx as usize - 1;
        }

        ComputedDropSearchResults {
            search_depth,
            upcoming_shapes: upcoming_shapes.clone(),
            drops: moves,
            score: best_score.clone(),
        }
    }

    pub fn make_move_result(&self) -> MoveResult {
        MoveResult {
            moves: self.drops.clone(),
            score: self.score.clone(),
        }
    }

    fn idx_range(search_depth: usize) -> (usize, usize) {
        let mut start_idx = 0;
        let mut end_idx = 0;
        for i in 0..search_depth + 1 {
            start_idx = end_idx;
            end_idx += consts::OUTPUTS_PER_INPUT_FIELD.pow(i as u32 + 1);
        }
        (start_idx, end_idx)
    }
}

impl BotShaderContext {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            kc: KernalConfig::prepare()?,
            sp: StartPositions::new(),
        })
    }

    pub fn compute_drop_search<F: Fn(&MoveResultScore) -> OrderedFloat<f32>>(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_field: &BitmapField,
        scoring_fn: F,
    ) -> Result<ComputedDropSearchResults, String> {
        let mut total_outputs = 0;
        (0..search_depth + 1)
            .for_each(|i| total_outputs += consts::OUTPUTS_PER_INPUT_FIELD.pow(i as u32 + 1));
        let configs_buffer = self
            .kc
            .make_data_buffer::<ComputedDropConfig>(total_outputs);
        let mut search_param_buffer = self.kc.make_data_buffer::<SearchParams>(1);

        let mut shape_position_config_buffer = self.kc.make_data_buffer::<ShapePositionConfig>(1);
        write_to_buffer(
            &mut shape_position_config_buffer,
            0,
            &self.sp.shape_position_config,
        );

        let mut fields_buffer = self.kc.make_data_buffer::<BitmapField>(total_outputs + 1);
        write_to_buffer(&mut fields_buffer, 0, source_field);

        let scores_buffer = self.kc.make_data_buffer::<MoveResultScore>(total_outputs);

        for cur_search_depth in 0..(search_depth as u8 + 1) {
            let sp = SearchParams {
                cur_search_depth,
                upcoming_shape_idxs: upcoming_shapes.map(|s| self.sp.shape_to_idx[s].clone()),
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

                assert_eq!(cmd_buffer.status(), MTLCommandBufferStatus::Completed);
            });

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

                assert_eq!(cmd_buffer.status(), MTLCommandBufferStatus::Completed);
            });
        }

        let config_slice = slice_from_buffer::<ComputedDropConfig>(&configs_buffer);
        let scores_slice = slice_from_buffer::<MoveResultScore>(&scores_buffer);

        Ok(ComputedDropSearchResults::find_results(
            search_depth,
            upcoming_shapes.clone(),
            config_slice,
            scores_slice,
            scoring_fn,
            &self.sp,
        ))
    }
}

struct KernalConfig {
    command_queue: CommandQueue,

    make_configs_pipeline_state: ComputePipelineState,
    computed_drop_pipeline_state: ComputePipelineState,

    device: Device,
}

impl KernalConfig {
    fn prepare() -> Result<Self, String> {
        let library_data = include_bytes!("bot_shader.metallib");
        autoreleasepool(|| -> Result<KernalConfig, String> {
            let device = Device::system_default().expect("No metal device available.");
            let library = device.new_library_with_data(&library_data[..])?;
            let command_queue = device.new_command_queue();

            let make_configs_function = library.get_function("compute_drop_config", None)?;
            let make_configs_pipeline_state =
                device.new_compute_pipeline_state_with_function(&make_configs_function)?;

            let computed_drop_function = library.get_function("drop_tetromino_for_config", None)?;
            let computed_drop_pipeline_state =
                device.new_compute_pipeline_state_with_function(&computed_drop_function)?;

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

#[cfg(test)]
mod test {
    use std::cmp::max;

    use crate::bot::bot_shader::BotShaderContext;
    use crate::bot::compute_types::ComputedDropConfig;
    use manytris_core::bitmap_field::BitmapField;
    use manytris_core::shapes::Shape;

    #[test]
    fn verify_computed_configs() {
        let ctx = BotShaderContext::new();

        let shapes = [
            Shape::I,
            Shape::J,
            Shape::L,
            Shape::I,
            Shape::I,
            Shape::I,
            Shape::I,
        ];

        let results = ctx
            .unwrap()
            .compute_drop_search(1, &shapes, &BitmapField::default())
            .unwrap();

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

        assert_eq!(results.drop_configs.len(), expected_cfgs.len());
        assert_eq!(results.drop_configs, expected_cfgs);
    }
}
