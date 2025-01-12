use anyhow::{Context, Result, bail};
use manytris_bot::compute_types::{DropConfig, SearchParams};
use std::iter;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Features, QueueCreateInfo, QueueFlags};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{sync, VulkanLibrary};

pub struct VulkanBotContext {}

fn init_vulkan_bot(search_params: SearchParams) -> Result<VulkanBotContext> {
    let library = VulkanLibrary::new().context("no local Vulkan library/DLL")?;
    let instance: Arc<Instance> = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )
    .context("failed to create instance")?;

    let physical_device: Arc<PhysicalDevice> = instance
        .enumerate_physical_devices()
        .context("Could not enumerate devices")?
        .next()
        .context("No devices available")?;

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .position(|family| family.queue_flags.contains(QueueFlags::COMPUTE))
        .context("No Compute queues available")? as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: DeviceExtensions {
                khr_16bit_storage: true,
                khr_8bit_storage: true,
                ..DeviceExtensions::empty()
            },
            enabled_features: Features {
                shader_int8: true,
                shader_int16: true,
                uniform_and_storage_buffer8_bit_access: true,
                ..Features::empty()
            },
            ..Default::default()
        },
    )
    .context("Failed to create device")?;

    let queue = queues.next().context("No Queues available")?;

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    let shader = bot_shader::load(device.clone()).context("Failed to create shader module")?;

    let entry = shader
        .entry_point("main")
        .context("Couldn't find entrypoint")?;
    let stage = PipelineShaderStageCreateInfo::new(entry);
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())?,
    )?;

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .context("failed to create compute pipeline")?;

    let descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());

    let layouts = compute_pipeline
        .layout()
        .set_layouts();
    
    println!("num sets: {}", layouts.len());
    
    let descriptor_set_index = 0;
    let descriptor_set_layout = compute_pipeline
        .layout()
        .set_layouts()
        .get(descriptor_set_index)
        .context("No descriptor_set_layouts found")?;

    let memory_allocator: Arc<StandardMemoryAllocator> =
        Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let search_params_buffer = Buffer::from_data(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        search_params,
    )?;

    let num_outputs = manytris_bot::num_outputs(1);

    let drop_configs_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        iter::repeat_n(DropConfig::default(), num_outputs),
    )?;

    let search_params_binding = 0;
    let drop_configs_binding = 1;

    let descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        descriptor_set_layout.clone(),
        [
            WriteDescriptorSet::buffer(search_params_binding, search_params_buffer.clone()),
            WriteDescriptorSet::buffer(drop_configs_binding, drop_configs_buffer.clone()),
        ],
        [],
    )?;

    let num_groups = num_outputs / 64 + (if num_outputs % 64 == 0 { 0 } else { 1 });

    let work_group_counts = [num_groups as u32, 1, 1];

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )?;

    command_buffer_builder
        .bind_pipeline_compute(compute_pipeline.clone())?
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0,
            descriptor_set,
        )?
        .dispatch(work_group_counts)?;

    let command_buffer = command_buffer_builder.build()?;

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)?
        .then_signal_fence_and_flush()?;

    future.wait(None)?;

    let config_content = drop_configs_buffer.read()?;

    for (i, cfg) in config_content.iter().enumerate() {
        println!("{i}: {cfg:?}");
    }

    Ok(VulkanBotContext {})
}

mod bot_shader {
    use vulkano_shaders;

    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/bot.glsl",
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_init() -> Result<()> {
        init_vulkan_bot(SearchParams {
            cur_search_depth: 0,
            upcoming_shape_idxs: [0, 0, 0, 0, 0, 0, 0],
        })?;
        // Ok(())
        bail!("expected");
    }
}
