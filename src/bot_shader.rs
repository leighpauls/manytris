use std::cmp::min;
use std::ffi::c_void;
use std::mem::{size_of, size_of_val};
use std::slice;

use metal::objc::rc::autoreleasepool;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputeCommandEncoderRef, ComputePipelineDescriptor,
    ComputePipelineState, Device, Function, Library, MTLResourceOptions, MTLSize, NSUInteger,
};

const W: usize = 10;
const H: usize = 22;
const NUM_BLOCKS: usize = W * H;
const FIELD_BYTES: usize = NUM_BLOCKS / 8 + if (NUM_BLOCKS % 8) == 0 { 0 } else { 1 };

pub fn call_drop_tetromino() -> Result<(), String> {
    let kc = prepare_shader_method("drop_tetromino")?;

    autoreleasepool(|| -> Result<(), String> {
        let (command_buffer, encoder, pipeline_state, device) = kc.prep_command();

        let positions_buffer = create_mtl_buffer(
            device,
            &[TetrominoPositions {
                pos: [[0, 0], [1, 0], [0, 1], [1, 1]],
            }],
        );

        let fields_buffer =
            create_mtl_buffer(&device, &[ShaderField::default(), ShaderField::default()]);

        let configs_buffer = create_mtl_buffer(
            device,
            &[DropConfig {
                tetromino_idx: 0,
                initial_field_idx: 0,
                dest_field_idx: 1,
            }],
        );

        encoder.set_buffer(0, Some(&positions_buffer), 0);
        encoder.set_buffer(1, Some(&fields_buffer), 0);
        encoder.set_buffer(2, Some(&configs_buffer), 0);
        encoder.dispatch_threads(MTLSize::new(1, 1, 1), MTLSize::new(1, 1, 1));
        encoder.end_encoding();

        println!("Committing...");
        command_buffer.commit();
        command_buffer.wait_until_completed();

        println!("Done!");

        let fields = slice_from_buffer::<ShaderField>(&fields_buffer);
        println!("Positions array: {:?}", fields);

        Ok(())
    })
}

struct KernalConfig {
    pipeline_state: ComputePipelineState,
    pipeline_state_descriptor: ComputePipelineDescriptor,
    command_queue: CommandQueue,
    kernel: Function,
    library: Library,
    device: Device,
}

fn prepare_shader_method(shader_name: &str) -> Result<KernalConfig, String> {
    let library_data = include_bytes!("bot_shader.metallib");
    autoreleasepool(|| -> Result<KernalConfig, String> {
        let device = Device::system_default().expect("No metal device available.");
        let library = device.new_library_with_data(&library_data[..])?;
        let kernel = library.get_function(shader_name, None)?;
        let command_queue = device.new_command_queue();

        // TODO: not sure if/why pipeline_state_descriptor exists instead of paassing kernel directly to pipeline state.
        let pipeline_state_descriptor = ComputePipelineDescriptor::new();
        pipeline_state_descriptor.set_compute_function(Some(&kernel));

        let pipeline_state = device.new_compute_pipeline_state_with_function(
            pipeline_state_descriptor
                .compute_function()
                .ok_or("No compute function found.")?,
        )?;

        Ok(KernalConfig {
            pipeline_state,
            pipeline_state_descriptor,
            command_queue,
            kernel,
            library,
            device,
        })
    })
}

impl KernalConfig {
    fn prep_command(
        &self,
    ) -> (
        &CommandBufferRef,
        &ComputeCommandEncoderRef,
        &ComputePipelineState,
        &Device,
    ) {
        let command_buffer = self.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline_state);

        (command_buffer, encoder, &self.pipeline_state, &self.device)
    }
}

fn create_mtl_buffer<T>(device: &Device, source_data: &[T]) -> Buffer {
    println!("Size of buffer: {}", size_of_val(source_data));
    device.new_buffer_with_data(
        source_data.as_ptr() as *const c_void,
        size_of_val(source_data) as NSUInteger,
        MTLResourceOptions::StorageModeShared,
    )
}

fn slice_from_buffer<T>(buffer: &Buffer) -> &[T] {
    let items = buffer.length() as usize / size_of::<T>();
    unsafe { slice::from_raw_parts(buffer.contents() as *const T, items) }
}

#[repr(C)]
#[derive(Debug)]
struct TetrominoPositions {
    pos: [[u8; 2]; 4],
}

#[repr(C)]
#[derive(Debug, Default)]
struct ShaderField {
    bytes: [u8; FIELD_BYTES],
}

#[repr(C)]
struct DropConfig {
    tetromino_idx: u32,
    initial_field_idx: u32,
    dest_field_idx: u32,
}
