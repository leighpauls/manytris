use metal::objc::rc::autoreleasepool;
use metal::{ComputePipelineDescriptor, Device, MTLResourceOptions, MTLSize};
use std::cmp::min;
use std::slice;

pub fn call_shader() {
    let library_data = include_bytes!("bot_shader.metallib");

    autoreleasepool(|| -> Result<(), String> {
        let device = Device::system_default().expect("No metal device available.");
        let library = device.new_library_with_data(&library_data[..])?;
        let kernel = library.get_function("test_func", None)?;

        let command_queue = device.new_command_queue();
        let command_buffer = command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        // TODO: not sure if/why pipeline_state_descriptor exists instead of paassing kernel directly to pipeline state.
        let pipeline_state_descriptor = ComputePipelineDescriptor::new();
        pipeline_state_descriptor.set_compute_function(Some(&kernel));

        let pipeline_state = device.new_compute_pipeline_state_with_function(
            pipeline_state_descriptor
                .compute_function()
                .ok_or("No compute function found.")?,
        )?;

        encoder.set_compute_pipeline_state(&pipeline_state);

        let items = 100000000;
        let threads = 100;
        let items_per_thread = (items / threads) as u32;
        let num_threads_per_group = min(threads, pipeline_state.thread_execution_width());
        let threads_size = MTLSize {
            width: threads,
            height: 1,
            depth: 1,
        };
        let group_size = MTLSize {
            width: num_threads_per_group,
            height: 1,
            depth: 1,
        };

        let input_buffer = device.new_buffer(
            (std::mem::size_of::<u32>() * items as usize) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let input_ptr = input_buffer.contents() as *mut u32;
        for i in 0..(items as u32) {
            unsafe {
                *input_ptr.offset(i as isize) = i;
            }
        }

        let items_per_thread_buffer = device.new_buffer(
            std::mem::size_of::<u32>() as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let items_per_thread_ptr = items_per_thread_buffer.contents() as *mut u32;
        unsafe {
            *items_per_thread_ptr = items_per_thread;
        }

        let output_buffer = device.new_buffer(
            (std::mem::size_of::<u32>() * threads as usize) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        encoder.set_buffer(0, Some(&input_buffer), 0);
        encoder.set_buffer(1, Some(&items_per_thread_buffer), 0);
        encoder.set_buffer(2, Some(&output_buffer), 0);

        encoder.dispatch_threads(threads_size, group_size);
        encoder.end_encoding();

        println!("Committing...");
        command_buffer.commit();
        command_buffer.wait_until_completed();
        println!("Done!");

        let output_ptr = output_buffer.contents() as *const u32;
        unsafe {
            let output_slice = slice::from_raw_parts(output_ptr, threads as usize);
            println!("Result: {:?}", output_slice);
        }
        Ok(())
    })
    .unwrap();
}
