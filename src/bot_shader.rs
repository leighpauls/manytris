use std::cmp::min;
use std::collections::HashMap;
use std::mem::size_of;
use std::slice;

use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    Function, Library, MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};

use crate::bot_player::{MoveResult, MovementDescriptor};
use crate::bot_start_positions::StartPositions;
use crate::compute_types::{
    BitmapField, ComputedDropConfig, DropConfig, MoveResultScore, SearchParams,
    ShapePositionConfig, TetrominoPositions,
};
use crate::consts;
use crate::shapes::Shape;
use crate::upcoming::UpcomingTetrominios;

pub struct BotShaderContext {
    kc: KernalConfig,
    pub sp: StartPositions,
}

pub struct MovementBatchRequest {
    pub src_state: BitmapField,
    pub moves: Vec<MovementDescriptor>,
}

pub struct MovementBatchResult {
    pub result: Vec<(BitmapField, MoveResultScore)>,
}

pub type UpcomingShapes = [Shape; consts::MAX_SEARCH_DEPTH + 1];
pub struct ComputedDropSearchResults {
    pub search_depth: usize,
    pub upcoming_shapes: UpcomingShapes,
    pub drop_configs: Vec<ComputedDropConfig>,
    pub scores: Vec<MoveResultScore>,
}

impl ComputedDropSearchResults {
    pub fn make_move_result(
        &self,
        search_depth: usize,
        idx: usize,
        sp: &StartPositions,
    ) -> MoveResult {
        let (start_idx, _) = Self::idx_range(search_depth);
        let mut next_idx = idx + start_idx;
        let mut moves = vec![];
        loop {
            let cfg = &self.drop_configs[next_idx];
            moves.insert(0, cfg.as_move_descriptor(sp));

            if cfg.src_field_idx == 0 {
                break;
            }
            next_idx = cfg.src_field_idx as usize - 1;
        }
        let score = self.result_slice(search_depth).1[idx].clone();
        MoveResult { moves, score }
    }
    pub fn result_slice(&self, search_depth: usize) -> (&[ComputedDropConfig], &[MoveResultScore]) {
        let (start_idx, end_idx) = Self::idx_range(search_depth);
        (
            &self.drop_configs.as_slice()[start_idx..end_idx],
            &self.scores.as_slice()[start_idx..end_idx],
        )
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

    pub fn make_drop_configs(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_field: &BitmapField,
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
                upcoming_shape_idxs: upcoming_shapes
                    .map(|s| self.sp.shape_to_idx[s].clone()),
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
        Ok(ComputedDropSearchResults {
            search_depth,
            upcoming_shapes: upcoming_shapes.clone(),
            drop_configs: Vec::from(config_slice),
            scores: Vec::from(scores_slice),
        })
    }

    pub fn evaluate_moves(
        &self,
        batches: &Vec<MovementBatchRequest>,
    ) -> Result<Vec<MovementBatchResult>, String> {
        let initial_states = batches.len();
        let num_moves = batches.iter().map(|b| b.moves.len()).sum();

        let mut buffers = self.kc.make_buffers(initial_states, num_moves);

        let mut move_idx = 0;

        batches.iter().enumerate().for_each(|(batch_idx, b)| {
            write_to_buffer(&mut buffers.fields, batch_idx, &b.src_state);

            b.moves.iter().for_each(|md| {
                let cur_position_idx = move_idx * 2;
                write_to_buffer(
                    &mut buffers.positions,
                    cur_position_idx,
                    self.sp.bot_start_tps(md.shape, md.cw_rotations),
                );
                let next_position_idx = cur_position_idx + 1;
                write_to_buffer(
                    &mut buffers.positions,
                    next_position_idx,
                    self.sp.player_start_tps(md.next_shape),
                );

                let output_field_idx = initial_states + move_idx;
                write_to_buffer(
                    &mut buffers.configs,
                    move_idx,
                    &DropConfig {
                        tetromino_idx: cur_position_idx as u32,
                        next_tetromino_idx: next_position_idx as u32,
                        initial_field_idx: batch_idx as u32,
                        dest_field_idx: output_field_idx as u32,
                        left_shifts: if md.shifts_right < 0 {
                            (-md.shifts_right) as u8
                        } else {
                            0
                        },
                        right_shifts: if md.shifts_right > 0 {
                            md.shifts_right as u8
                        } else {
                            0
                        },
                    },
                );
                move_idx += 1;
            });
        });

        self.kc.run_cmd(&buffers, num_moves)?;

        let result_slice = &slice_from_buffer::<BitmapField>(&buffers.fields)[initial_states..];
        let score_slice = slice_from_buffer::<MoveResultScore>(&buffers.scores);
        assert_eq!(result_slice.len(), score_slice.len());

        let mut result_iter = result_slice
            .into_iter()
            .zip(score_slice)
            .map(|(b, s)| (b.clone(), s.clone()));

        Ok(batches
            .iter()
            .map(|b| MovementBatchResult {
                result: result_iter.by_ref().take(b.moves.len()).collect(),
            })
            .collect())
    }
}

struct KernalConfig {
    command_queue: CommandQueue,

    drop_pipeline_state: ComputePipelineState,
    _drop_function: Function,

    make_configs_pipeline_state: ComputePipelineState,
    _make_configs_function: Function,

    computed_drop_pipeline_state: ComputePipelineState,

    _library: Library,
    device: Device,
}

struct Buffers {
    positions: Buffer,
    fields: Buffer,
    configs: Buffer,
    scores: Buffer,
}

impl KernalConfig {
    fn prepare() -> Result<Self, String> {
        let library_data = include_bytes!("bot_shader.metallib");
        autoreleasepool(|| -> Result<KernalConfig, String> {
            let device = Device::system_default().expect("No metal device available.");
            let library = device.new_library_with_data(&library_data[..])?;
            let command_queue = device.new_command_queue();

            let drop_function = library.get_function("drop_tetromino", None)?;
            let drop_pipeline_state =
                device.new_compute_pipeline_state_with_function(&drop_function)?;

            let make_configs_function = library.get_function("compute_drop_config", None)?;
            let make_configs_pipeline_state =
                device.new_compute_pipeline_state_with_function(&make_configs_function)?;

            let computed_drop_function = library.get_function("drop_tetromino_for_config", None)?;
            let computed_drop_pipeline_state =
                device.new_compute_pipeline_state_with_function(&computed_drop_function)?;

            Ok(KernalConfig {
                command_queue,
                drop_pipeline_state,
                _drop_function: drop_function,
                make_configs_pipeline_state,
                _make_configs_function: make_configs_function,
                computed_drop_pipeline_state,
                _library: library,
                device,
            })
        })
    }

    fn make_drop_command_buffer(&self) -> (&CommandBufferRef, &ComputeCommandEncoderRef) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.drop_pipeline_state);

        (command_buffer, encoder)
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

    fn make_buffers(&self, initial_states: usize, outputs: usize) -> Buffers {
        autoreleasepool(|| Buffers {
            // TODO: make positions a shard constant
            positions: self.make_data_buffer::<TetrominoPositions>(outputs * 2),
            fields: self.make_data_buffer::<BitmapField>(initial_states + outputs),
            configs: self.make_data_buffer::<DropConfig>(outputs),
            scores: self.make_data_buffer::<MoveResultScore>(outputs),
        })
    }

    fn run_cmd(&self, buffers: &Buffers, moves: usize) -> Result<(), String> {
        autoreleasepool(|| -> Result<(), String> {
            let (command_buffer, encoder) = self.make_drop_command_buffer();
            encoder.set_buffer(0, Some(&buffers.positions), 0);
            encoder.set_buffer(1, Some(&buffers.fields), 0);
            encoder.set_buffer(2, Some(&buffers.configs), 0);
            encoder.set_buffer(3, Some(&buffers.scores), 0);
            let max_threads = self.drop_pipeline_state.max_total_threads_per_threadgroup();
            let threads_per_threadgoupd = min(max_threads, moves as NSUInteger);
            encoder.dispatch_threads(
                MTLSize::new(moves as NSUInteger, 1, 1),
                MTLSize::new(threads_per_threadgoupd, 1, 1),
            );
            encoder.end_encoding();

            command_buffer.commit();
            command_buffer.wait_until_completed();

            let status = command_buffer.status();
            if status != MTLCommandBufferStatus::Completed {
                assert_eq!(status, MTLCommandBufferStatus::Error);
                return Err("Command buffer returned error.".to_string());
            }

            Ok(())
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

#[cfg(test)]
mod test {
    use std::cmp::max;

    use enum_iterator::all;

    use crate::bot_shader::BotShaderContext;
    use crate::compute_types::{BitmapField, ComputedDropConfig};
    use crate::consts;
    use crate::shapes::Shape;

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
            .make_drop_configs(1, &shapes, &BitmapField::default())
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
