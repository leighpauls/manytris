use std::cmp::min;
use std::ffi::c_void;
use std::mem::{size_of, size_of_val};
use std::slice;

use crate::bot_player::MovementDescriptor;
use crate::compute_types::{BitmapField, DropConfig, TetrominoPositions};
use crate::shapes::{Rot, Shape};
use crate::tetromino::Tetromino;
use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    Function, Library, MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};

pub fn evaluate_move(
    src_state: &BitmapField,
    md: &MovementDescriptor,
) -> Result<BitmapField, String> {
    let kc = KernalConfig::prepare("drop_tetromino")?;
    let mut tet = Tetromino::new(md.shape);
    for _ in 0..md.cw_rotations {
        tet = tet.rotation_options(Rot::Cw).get(0).unwrap().clone();
    }

    let mut buffers = kc.make_buffers();
    write_to_buffer(&mut buffers.fields, 0, src_state);
    write_to_buffer(&mut buffers.positions, 0, &TetrominoPositions::from(tet));
    write_to_buffer(
        &mut buffers.configs,
        0,
        &DropConfig {
            tetromino_idx: 0,
            initial_field_idx: 0,
            dest_field_idx: 1,
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

    kc.run_cmd(&buffers)?;

    Ok(slice_from_buffer::<BitmapField>(&buffers.fields)[1].clone())
}

pub fn call_drop_tetromino() -> Result<(), String> {
    let kc = KernalConfig::prepare("drop_tetromino")?;

    let mut buffers = kc.make_buffers();

    kc.run_cmd(&buffers)?;

    println!(
        "Positions array: {:?}",
        slice_from_buffer::<BitmapField>(&buffers.fields)
    );

    {
        let config = slice_from_buffer_mut::<DropConfig>(&mut buffers.configs);
        config[0].initial_field_idx = 1;
        config[0].dest_field_idx = 0;
    }

    kc.run_cmd(&buffers)?;

    println!(
        "Positions array: {:?}",
        slice_from_buffer::<BitmapField>(&buffers.fields)
    );

    Ok(())
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
    fn prepare(shader_name: &str) -> Result<Self, String> {
        let library_data = include_bytes!("bot_shader.metallib");
        autoreleasepool(|| -> Result<KernalConfig, String> {
            let device = Device::system_default().expect("No metal device available.");
            let library = device.new_library_with_data(&library_data[..])?;
            let function = library.get_function(shader_name, None)?;
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

    fn make_data_buffer<T>(&self, source_data: &[T]) -> Buffer {
        self.device.new_buffer_with_data(
            source_data.as_ptr() as *const c_void,
            size_of_val(source_data) as NSUInteger,
            MTLResourceOptions::StorageModeShared,
        )
    }

    fn make_buffers(&self) -> Buffers {
        autoreleasepool(|| Buffers {
            positions: self.make_data_buffer(&[TetrominoPositions::from(Tetromino::new(Shape::O))]),
            fields: self.make_data_buffer(&[BitmapField::default(), BitmapField::default()]),
            configs: self.make_data_buffer(&[DropConfig {
                tetromino_idx: 0,
                initial_field_idx: 0,
                dest_field_idx: 1,
                left_shifts: 2,
                right_shifts: 0,
            }]),
        })
    }

    fn run_cmd(&self, buffers: &Buffers) -> Result<(), String> {
        autoreleasepool(|| -> Result<(), String> {
            let (command_buffer, encoder) = self.make_command_buffer();
            encoder.set_buffer(0, Some(&buffers.positions), 0);
            encoder.set_buffer(1, Some(&buffers.fields), 0);
            encoder.set_buffer(2, Some(&buffers.configs), 0);
            encoder.dispatch_threads(MTLSize::new(1, 1, 1), MTLSize::new(1, 1, 1));
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
