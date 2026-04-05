use hotham::vk;
use std::marker::PhantomData;

pub struct ComputePipeline<Registers> {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    _registers: PhantomData<Registers>,
}

impl<Registers: bytemuck::Pod> ComputePipeline<Registers> {
    pub fn load(
        context: &hotham::contexts::VulkanContext,
        module: &[u8],
    ) -> ComputePipeline<Registers> {
        let device = &context.device;
        unsafe {
            let layout = device
                .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None) // TODO PC
                .unwrap();

            let handle = device
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::ComputePipelineCreateInfo::default()
                        .layout(layout)
                        .stage(
                            vk::PipelineShaderStageCreateInfo::default()
                                .stage(vk::ShaderStageFlags::COMPUTE)
                                .module(super::pipeline::load_module(module, &context))
                                .name(c"main"),
                        )],
                    None,
                )
                .unwrap()[0];

            ComputePipeline {
                handle,
                layout,
                _registers: PhantomData,
            }
        }
    }

    pub fn update_registers(
        &self,
        registers: &Registers,
        context: &hotham::contexts::VulkanContext,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            context.device.cmd_push_constants(
                command_buffer,
                self.layout,
                vk::ShaderStageFlags::ALL,
                0,
                bytemuck::bytes_of(registers),
            )
        };
    }
}

/// A thin wrapper around descriptor related functionality
#[allow(unused)]
pub struct ComputeDescriptors {
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
}

impl ComputeDescriptors {
    #[allow(unused)]
    pub fn new(context: hotham::contexts::VulkanContext) -> ComputeDescriptors {
        let device = &context.device;

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&[vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::STORAGE_IMAGE,
                        descriptor_count: 1000,
                    }]),
                None,
            )
        }
        .unwrap();

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&[
                    vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
                        stage_flags: vk::ShaderStageFlags::COMPUTE,
                        descriptor_count: 1000,
                        ..Default::default()
                    },
                ]),
                None,
            )
        }
        .unwrap();

        let set = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(std::slice::from_ref(&layout)),
            )
        }
        .unwrap()[0];

        ComputeDescriptors { pool, set, layout }
    }

    #[allow(unused)]
    pub fn write_image_to_set(
        &self,
        texture: &hotham::rendering::texture::Texture,
        context: &hotham::contexts::VulkanContext,
    ) {
        unsafe {
            context.device.update_descriptor_sets(
                std::slice::from_ref(
                    &vk::WriteDescriptorSet::default()
                        .image_info(std::slice::from_ref(
                            &vk::DescriptorImageInfo::default()
                                .image_view(texture.image.view)
                                .image_layout(vk::ImageLayout::GENERAL),
                        ))
                        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                        .dst_array_element(texture.index)
                        .dst_set(self.set),
                ),
                &[],
            );
        }
    }
}
