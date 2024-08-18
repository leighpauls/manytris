use std::cmp::min;
use std::ffi::c_void;
use std::mem::{size_of, size_of_val};
use std::slice;

use crate::compute_types::{BitmapField, DropConfig, TetrominoPositions};
use crate::shapes::Shape;
use crate::tetromino::Tetromino;
use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineState, Device,
    Function, Library, MTLCommandBufferStatus, MTLResourceOptions, MTLSize, NSUInteger,
};

pub fn call_drop_tetromino() -> Result<(), String> {
    let kc = KernalConfig::prepare("drop_tetromino")?;

    let (positions_buffer, fields_buffer, mut configs_buffer) = autoreleasepool(|| {
        (
            kc.make_data_buffer(&[TetrominoPositions::from(Tetromino::new(Shape::O))]),
            kc.make_data_buffer(&[BitmapField::default(), BitmapField::default()]),
            kc.make_data_buffer(&[DropConfig {
                tetromino_idx: 0,
                initial_field_idx: 0,
                dest_field_idx: 1,
                left_shifts: 2,
                right_shifts: 0,
            }]),
        )
    });

    let run_cmd = |config_buffer: &Buffer| {
        autoreleasepool(|| -> Result<(), String> {
            let (command_buffer, encoder) = kc.make_command_buffer();
            encoder.set_buffer(0, Some(&positions_buffer), 0);
            encoder.set_buffer(1, Some(&fields_buffer), 0);
            encoder.set_buffer(2, Some(config_buffer), 0);
            encoder.dispatch_threads(MTLSize::new(1, 1, 1), MTLSize::new(1, 1, 1));
            encoder.end_encoding();

            println!("Committing...");
            command_buffer.commit();
            command_buffer.wait_until_completed();

            let status = command_buffer.status();
            if status != MTLCommandBufferStatus::Completed {
                assert_eq!(status, MTLCommandBufferStatus::Error);
                return Err("Command buffer returned error.".to_string());
            }

            println!("Done!");

            let fields = slice_from_buffer::<BitmapField>(&fields_buffer);
            println!("Positions array: {:?}", fields);
            Ok(())
        })
    };

    run_cmd(&configs_buffer)?;

    {
        let config = slice_from_buffer_mut::<DropConfig>(&mut configs_buffer);
        config[0].initial_field_idx = 1;
        config[0].dest_field_idx = 0;
    }

    run_cmd(&configs_buffer)?;
    Ok(())
}

struct KernalConfig {
    pipeline_state: ComputePipelineState,
    command_queue: CommandQueue,
    _function: Function,
    _library: Library,
    device: Device,
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
}

fn slice_from_buffer<T>(buffer: &Buffer) -> &[T] {
    let items = buffer.length() as usize / size_of::<T>();
    unsafe { slice::from_raw_parts(buffer.contents() as *const T, items) }
}

fn slice_from_buffer_mut<T>(buffer: &mut Buffer) -> &mut [T] {
    let items = buffer.length() as usize / size_of::<T>();
    unsafe { slice::from_raw_parts_mut(buffer.contents() as *mut T, items) }
}
