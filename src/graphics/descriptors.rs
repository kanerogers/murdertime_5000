use hotham::vk;

pub struct Descriptors {
    #[allow(unused)]
    pub pool: vk::DescriptorPool,
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
}

impl Descriptors {
    pub const LINES_BINDING: u32 = 0;

    pub fn new(context: &hotham::contexts::VulkanContext) -> Descriptors {
        let device = &context.device;

        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1000,
        }];

        let pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&pool_sizes)
                    .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND),
                None,
            )
        }
        .unwrap();

        let flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND];
        let mut binding_flags =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);

        let layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default()
                    .bindings(&[
                        // // Textures
                        // vk::DescriptorSetLayoutBinding {
                        //     binding: Self::TEXTURE_BINDING,
                        //     descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        //     stage_flags: vk::ShaderStageFlags::COMPUTE
                        //         | vk::ShaderStageFlags::FRAGMENT,
                        //     descriptor_count: 1000,
                        //     ..Default::default()
                        // },
                        // Lines
                        vk::DescriptorSetLayoutBinding {
                            binding: Self::LINES_BINDING,
                            descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                            stage_flags: vk::ShaderStageFlags::VERTEX,
                            descriptor_count: 1,
                            ..Default::default()
                        },
                    ])
                    .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                    .push_next(&mut binding_flags),
                None,
            )
        }
        .unwrap();

        let set = unsafe {
            device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(std::slice::from_ref(&layout)),
                )
                .unwrap()[0]
        };

        Descriptors { pool, set, layout }
    }

    // #[allow(unused)]
    // pub unsafe fn update_texture_descriptor_set(
    //     &self,
    //     texture_id: u32,
    //     image_view: vk::ImageView,
    //     sampler: vk::Sampler,
    //     context: &hotham::contexts::VulkanContext,
    // ) {
    //     context.device.update_descriptor_sets(
    //         std::slice::from_ref(
    //             &vk::WriteDescriptorSet::default()
    //                 .image_info(std::slice::from_ref(
    //                     &vk::DescriptorImageInfo::default()
    //                         .sampler(sampler)
    //                         .image_view(image_view)
    //                         .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
    //                 ))
    //                 .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
    //                 .dst_array_element(texture_id)
    //                 .dst_binding(Self::TEXTURE_BINDING)
    //                 .dst_set(self.set),
    //         ),
    //         &[],
    //     );
    // }

    pub unsafe fn update_storage_buffer_descriptor_set(
        &self,
        binding: u32,
        handle: vk::Buffer,
        context: &hotham::contexts::VulkanContext,
    ) {
        context.device.update_descriptor_sets(
            std::slice::from_ref(
                &vk::WriteDescriptorSet::default()
                    .buffer_info(&[vk::DescriptorBufferInfo::default()
                        .buffer(handle)
                        .range(vk::WHOLE_SIZE)])
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .dst_binding(binding)
                    .dst_set(self.set),
            ),
            &[],
        );
    }
}
