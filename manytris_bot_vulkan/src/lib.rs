use anyhow::{Context, Result};
use manytris_bot::bot_start_positions::START_POSITIONS;
use manytris_bot::compute_types::{
    ComputedDropConfig, MoveResultScore, SearchParams, ShapePositionConfig, UpcomingShapes,
};
use manytris_core::bitmap_field::BitmapField;
use std::iter;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::shader::ShaderModule;
use vulkano::sync::GpuFuture;
use vulkano::{sync, VulkanLibrary};

pub struct VulkanBotContext {
    device: Arc<Device>,
    make_configs_pipeline: Arc<ComputePipeline>,
    eval_moves_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    queue: Arc<Queue>,
}

/// Container for producing an ExactSizeIterator in the initial shape of the fields buffer.
struct FieldBufferInitContainer {
    source_field: BitmapField,
    num_outputs: usize,
}

struct FieldBufferInitIterator {
    container: FieldBufferInitContainer,
    idx: usize,
}

impl VulkanBotContext {
    pub fn init() -> Result<VulkanBotContext> {
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
                    storage_buffer8_bit_access: true,
                    uniform_and_storage_buffer16_bit_access: true,
                    storage_buffer16_bit_access: true,
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

        let load_pipeline_fn = |shader: Arc<ShaderModule>| -> Result<Arc<ComputePipeline>> {
            let entry = shader
                .entry_point("main")
                .context("Couldn't find entrypoint")?;
            let stage = PipelineShaderStageCreateInfo::new(entry);
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())?,
            )?;

            Ok(ComputePipeline::new(
                device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout),
            )
            .context("failed to create compute pipeline")?)
        };

        let make_configs = make_configs_shader::load(device.clone())
            .context("Failed to load make_configs shader module")?;

        let make_configs_pipeline = load_pipeline_fn(make_configs)?;

        let eval_moves = eval_moves_shader::load(device.clone())
            .context("Failed to load eval_moves shader module")?;

        let eval_moves_pipeline = load_pipeline_fn(eval_moves)?;

        Ok(VulkanBotContext {
            device,
            make_configs_pipeline,
            eval_moves_pipeline,
            command_buffer_allocator,
            queue,
        })
    }

    pub fn compute_drop_scores(
        self,
        upcoming_shapes: UpcomingShapes,
        source_field: BitmapField,
    ) -> Result<()> {
        let search_params = SearchParams {
            cur_search_depth: 0,
            upcoming_shape_idxs: upcoming_shapes.map(|s| START_POSITIONS.shape_to_idx[s]),
        };
        let num_outputs = manytris_bot::num_outputs(1);
        let num_groups = num_outputs / 64 + (if num_outputs % 64 == 0 { 0 } else { 1 });

        let work_group_counts = [num_groups as u32, 1, 1];

        let memory_allocator: Arc<StandardMemoryAllocator> =
            Arc::new(StandardMemoryAllocator::new_default(self.device.clone()));

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
            iter::repeat_n(ComputedDropConfig::default(), num_outputs),
        )?;

        self.make_configs(
            work_group_counts,
            search_params_buffer.clone(),
            drop_configs_buffer.clone(),
        )?;

        let shape_position_config_buffer = Buffer::from_data(
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
            START_POSITIONS.shape_position_config,
        )?;

        let fields_buffer = Buffer::from_iter(
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
            FieldBufferInitContainer {
                source_field,
                num_outputs,
            },
        )?;

        let scores_buffer = Buffer::from_iter(
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
            iter::repeat_n(MoveResultScore::default(), num_outputs),
        )?;

        self.eval_moves(
            search_params_buffer,
            drop_configs_buffer,
            shape_position_config_buffer,
            fields_buffer,
            scores_buffer,
        )?;

        Ok(())
    }

    fn make_configs(
        &self,
        work_group_counts: [u32; 3],
        search_params_buffer: Subbuffer<SearchParams>,
        drop_configs_buffer: Subbuffer<[ComputedDropConfig]>,
    ) -> Result<()> {
        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(self.device.clone(), Default::default());

        let descriptor_set_index = 0;
        let descriptor_set_layout = self
            .make_configs_pipeline
            .layout()
            .set_layouts()
            .get(descriptor_set_index)
            .context("No descriptor_set_layouts found")?;

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

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        command_buffer_builder
            .bind_pipeline_compute(self.make_configs_pipeline.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.make_configs_pipeline.layout().clone(),
                0,
                descriptor_set,
            )?
            .dispatch(work_group_counts)?;

        let command_buffer = command_buffer_builder.build()?;

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)?
            .then_signal_fence_and_flush()?;

        future.wait(None)?;

        Ok(())
    }

    fn eval_moves(
        &self,
        search_params_buffer: Subbuffer<SearchParams>,
        drop_configs_buffer: Subbuffer<[ComputedDropConfig]>,
        shape_position_config_buffer: Subbuffer<ShapePositionConfig>,
        fields_buffer: Subbuffer<[BitmapField]>,
        scores_buffer: Subbuffer<[MoveResultScore]>,
    ) -> Result<()> {
        Ok(())
    }
}

impl IntoIterator for FieldBufferInitContainer {
    type Item = BitmapField;
    type IntoIter = FieldBufferInitIterator;

    fn into_iter(self) -> Self::IntoIter {
        FieldBufferInitIterator {
            container: self,
            idx: 0,
        }
    }
}

impl Iterator for FieldBufferInitIterator {
    type Item = BitmapField;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.idx == 0 {
            Some(std::mem::take(&mut self.container.source_field))
        } else if self.idx <= self.container.num_outputs {
            Some(self.container.source_field)
        } else {
            None
        };
        self.idx += 1;
        result
    }
}

impl ExactSizeIterator for FieldBufferInitIterator {
    fn len(&self) -> usize {
        self.container.num_outputs + 1
    }
}

mod make_configs_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/make_configs.glsl",
        include: ["."],
    }
}

mod eval_moves_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/eval_moves.glsl",
        include: ["."],
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::bail;
    use manytris_core::{consts, shapes::Shape};

    #[test]
    fn simple_init() -> Result<()> {
        let ctx = VulkanBotContext::init()?;
        ctx.compute_drop_scores(
            [Shape::I; consts::MAX_SEARCH_DEPTH + 1],
            BitmapField::default(),
        )?;

        Ok(())
    }
}
