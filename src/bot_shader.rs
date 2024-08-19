use std::mem::size_of;
use std::slice;

use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    Function, Library, MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};

use crate::bot_player::MovementDescriptor;
use crate::bot_start_positions::bot_start_position;
use crate::compute_types::{BitmapField, DropConfig, TetrominoPositions};

pub fn evaluate_moves(
    src_state: &BitmapField,
    moves: &Vec<MovementDescriptor>,
) -> Result<Vec<BitmapField>, String> {
    let kc = KernalConfig::prepare()?;

    let initial_states = 1;
    let num_moves = moves.len();
    let mut buffers = kc.make_buffers(initial_states, num_moves);
    write_to_buffer(&mut buffers.fields, 0, src_state);

    moves.iter().enumerate().for_each(|(i, md)| {
        let output_field_idx = initial_states + i;
        write_to_buffer(
            &mut buffers.positions,
            i,
            &TetrominoPositions::from(bot_start_position(md.shape, md.cw_rotations)),
        );
        write_to_buffer(
            &mut buffers.configs,
            i,
            &DropConfig {
                tetromino_idx: i as u32,
                initial_field_idx: 0,
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
        )
    });

    kc.run_cmd(&buffers, num_moves)?;

    let result_slice = &slice_from_buffer::<BitmapField>(&buffers.fields)[initial_states..];
    Ok(Vec::from(result_slice))
}

struct KernalConfig {
    pipeline_state: ComputePipelineState,
    command_queue: CommandQueue,
    _function: Function,
    _library: Library,
    device: Device,
}

struct Buffers {
    positions: Buffer,
    fields: Buffer,
    configs: Buffer,
}

impl KernalConfig {
    fn prepare() -> Result<Self, String> {
        let library_data = include_bytes!("bot_shader.metallib");
        autoreleasepool(|| -> Result<KernalConfig, String> {
            let device = Device::system_default().expect("No metal device available.");
            let library = device.new_library_with_data(&library_data[..])?;
            let function = library.get_function("drop_tetromino", None)?;
            let command_queue = device.new_command_queue();

            let pipeline_state = device.new_compute_pipeline_state_with_function(&function)?;

            Ok(KernalConfig {
                pipeline_state,
                command_queue,
                _function: function,
                _library: library,
                device,
            })
        })
    }

    fn make_command_buffer(&self) -> (&CommandBufferRef, &ComputeCommandEncoderRef) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline_state);

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
            positions: self.make_data_buffer::<TetrominoPositions>(outputs),
            fields: self.make_data_buffer::<BitmapField>(initial_states + outputs),
            configs: self.make_data_buffer::<DropConfig>(outputs),
        })
    }

    fn run_cmd(&self, buffers: &Buffers, moves: usize) -> Result<(), String> {
        autoreleasepool(|| -> Result<(), String> {
            let (command_buffer, encoder) = self.make_command_buffer();
            encoder.set_buffer(0, Some(&buffers.positions), 0);
            encoder.set_buffer(1, Some(&buffers.fields), 0);
            encoder.set_buffer(2, Some(&buffers.configs), 0);
            encoder.dispatch_threads(
                MTLSize::new(moves as NSUInteger, 1, 1),
                MTLSize::new(moves as NSUInteger, 1, 1),
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
