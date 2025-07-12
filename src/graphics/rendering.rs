use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo},
    descriptor_set::layout,
    image::ImageUsage,
    memory::allocator::AllocationCreateInfo,
    pipeline::{
        DynamicState, Pipeline,
        graphics::{
            self, GraphicsPipelineCreateInfo,
            color_blend::ColorBlendAttachmentState,
            vertex_input::{Vertex, VertexDefinition},
        },
        layout::PipelineLayoutCreateInfo,
    },
    swapchain::{Swapchain, SwapchainCreateInfo},
    sync::GpuFuture,
};

use crate::graphics::{
    error::{self, GraphicsError, VulkanError},
    scene,
};

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/shaders/vertex.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/graphics/shaders/fragment.glsl",
    }
}
pub struct RenderContext {
    pub window: Arc<winit::window::Window>,
    pub swapchain: Arc<vulkano::swapchain::Swapchain>,
    pub pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    pub viewport: vulkano::pipeline::graphics::viewport::Viewport,
    pub previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
    pub image_views: Vec<Arc<vulkano::image::view::ImageView>>,
    pub recreate_swapchain: bool,
}

pub struct RenderableScene {
    pub vertex_buffer: vulkano::buffer::Subbuffer<[crate::graphics::vertex::Vertex]>,
    pub index_buffer: vulkano::buffer::Subbuffer<[u32]>,
    pub uniform_buffer: vulkano::buffer::Subbuffer<vs::CameraUbo>,
    pub matrix_buffer: vulkano::buffer::Subbuffer<[glam::Mat4]>,
    pub normal_buffer: vulkano::buffer::Subbuffer<[glam::Mat4]>,
    pub indirect_buffer:
        vulkano::buffer::Subbuffer<[vulkano::command_buffer::DrawIndexedIndirectCommand]>,
    pub material_buffer: vulkano::buffer::Subbuffer<[crate::graphics::material::GpuMaterial]>,
}

impl RenderableScene {
    pub fn from_scene(
        scene: &scene::Scene,
        memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
        aspect_ratio: f32,
    ) -> Result<Self, GraphicsError> {
        // create two vec to hold all vertices and indices
        // we need to merge all meshes in the scene into one vertex buffer and one index buffer
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut indirect_commands = Vec::new();
        let mut instance_id = 0;
        let mut model_matrices = Vec::new();
        let mut normal_matrices = Vec::new();
        let mut materials = Vec::new();

        let view_matrix = scene
            .cameras
            .get(scene.main_camera_index)
            .ok_or_else(|| error::GraphicsError::NoCameraFound)?
            .view_matrix();

        for mesh in &scene.objects {
            model_matrices.push(mesh.world_transform);
            normal_matrices.push((view_matrix * mesh.world_transform).inverse().transpose());

            for submesh in &mesh.submeshes {
                if submesh.vertices.is_empty() || submesh.indices.is_empty() {
                    continue; // Skip empty submeshes
                }

                let first_index = all_indices.len() as u32;
                let vertex_offset = all_vertices.len() as u32;
                let index_count = submesh.indices.len() as u32;

                all_vertices.extend_from_slice(&submesh.vertices);
                all_indices.extend_from_slice(&submesh.indices);

                let command = vulkano::command_buffer::DrawIndexedIndirectCommand {
                    index_count,
                    instance_count: 1,
                    first_index,
                    vertex_offset,
                    first_instance: instance_id,
                };
                indirect_commands.push(command);
                materials.push(submesh.material.clone());
            }
            instance_id += 1;
        }
        let vertex_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_vertices.into_iter(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create vertex buffer: {}",
                e
            )))
        })?;

        let index_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_indices.into_iter(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create index buffer: {}",
                e
            )))
        })?;

        let indirect_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::INDIRECT_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indirect_commands.into_iter(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create indirect buffer: {}",
                e
            )))
        })?;

        let projection_matrix = scene
            .cameras
            .get(scene.main_camera_index)
            .ok_or_else(|| error::GraphicsError::NoCameraFound)?
            .projection_matrix(aspect_ratio);

        let view_matrix = scene
            .cameras
            .get(scene.main_camera_index)
            .ok_or_else(|| error::GraphicsError::NoCameraFound)?
            .view_matrix();

        let ubo_data = vs::CameraUbo {
            proj: projection_matrix.to_cols_array_2d(),
            view: view_matrix.to_cols_array_2d(),
        };

        let uniform_buffer = vulkano::buffer::Buffer::from_data(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            ubo_data,
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create uniform buffer: {}",
                e
            )))
        })?;

        let matrix_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            model_matrices.into_iter(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create matrix buffer: {}",
                e
            )))
        })?;

        let material_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            materials.into_iter().map(|m| m.into()).collect::<Vec<_>>(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create material buffer: {}",
                e
            )))
        })?;

        let normal_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            normal_matrices.into_iter(),
        )
        .map_err(|e| {
            error::GraphicsError::from(error::VulkanError::BufferCreationError(format!(
                "Failed to create normal buffer: {}",
                e
            )))
        })?;
        Ok(Self {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            indirect_buffer,
            matrix_buffer,
            material_buffer,
            normal_buffer,
        })
    }
}
pub fn create_render_context(
    window: Arc<winit::window::Window>,
    device: Arc<vulkano::device::Device>,
    instance: Arc<vulkano::instance::Instance>,
) -> Result<RenderContext, crate::graphics::error::VulkanError> {
    let surface =
        vulkano::swapchain::Surface::from_window(instance, window.clone()).map_err(|e| {
            error::VulkanError::SurfaceCreationError(format!(
                "Failed to create surface from window: {}",
                e
            ))
        })?;

    let window_size = window.inner_size();

    let (swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .map_err(|e| {
                error::VulkanError::SwapchainError(format!(
                    "Failed to get surface capabilities: {}",
                    e
                ))
            })?;

        let (image_format, _) = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .map_err(|e| {
                error::VulkanError::SwapchainError(format!("Failed to get surface formats: {}", e))
            })?
            .into_iter()
            .next()
            .ok_or_else(|| {
                error::VulkanError::SwapchainError("No suitable surface format found".to_string())
            })?;

        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: window_size.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap_or(vulkano::swapchain::CompositeAlpha::Opaque),
                ..Default::default()
            },
        )
        .map_err(|e| {
            error::VulkanError::SwapchainError(format!("Failed to create swapchain: {}", e))
        })?
    };

    let image_views = images
        .into_iter()
        .filter_map(|image| {
            vulkano::image::view::ImageView::new_default(image)
                .map_err(|e| {
                    error::VulkanError::SwapchainError(format!(
                        "Failed to create image view: {}",
                        e
                    ))
                })
                .ok()
        })
        .collect::<Vec<_>>();

    let pipeline = {
        let vs = vs::load(device.clone())
            .map_err(|e| {
                error::VulkanError::ShaderCompilationError(format!(
                    "Failed to load vertex shader: {}",
                    e
                ))
            })?
            .entry_point("main")
            .ok_or_else(|| {
                error::VulkanError::ShaderCompilationError(
                    "Vertex shader entry point 'main' not found".to_string(),
                )
            })?;

        let fs = fs::load(device.clone())
            .map_err(|e| {
                error::VulkanError::ShaderCompilationError(format!(
                    "Failed to load fragment shader: {}",
                    e
                ))
            })?
            .entry_point("main")
            .ok_or_else(|| {
                error::VulkanError::ShaderCompilationError(
                    "Fragment shader entry point 'main' not found".to_string(),
                )
            })?;

        let vertex_input_state = crate::graphics::vertex::Vertex::per_vertex()
            .definition(&vs)
            .map_err(|e| {
                error::VulkanError::ShaderCompilationError(format!(
                    "Failed to define vertex input: {}",
                    e
                ))
            })?;

        let stages = [
            vulkano::pipeline::PipelineShaderStageCreateInfo::new(vs),
            vulkano::pipeline::PipelineShaderStageCreateInfo::new(fs),
        ];

        let pipeline_layout_create_info =
            vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| {
                    error::VulkanError::PipelineLayoutError(format!(
                        "Failed to create pipeline layout: {}",
                        e
                    ))
                })?;

        println!(
            "Pipeline layout create info: {:?}",
            pipeline_layout_create_info
        );
        let layout = vulkano::pipeline::layout::PipelineLayout::new(
            device.clone(),
            pipeline_layout_create_info,
        )
        .map_err(|e| {
            error::VulkanError::PipelineLayoutError(format!(
                "Failed to create pipeline layout: {}",
                e
            ))
        })?;
        let subpass = vulkano::pipeline::graphics::subpass::PipelineRenderingCreateInfo {
            color_attachment_formats: [Some(swapchain.image_format())].to_vec(),
            ..Default::default()
        };

        vulkano::pipeline::graphics::GraphicsPipeline::new(
            device.clone(),
            None,
            vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
                stages: stages.to_vec().into(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(graphics::input_assembly::InputAssemblyState::default()),
                viewport_state: Some(graphics::viewport::ViewportState::default()),
                rasterization_state: Some(graphics::rasterization::RasterizationState {
                    cull_mode: graphics::rasterization::CullMode::Back,
                    front_face: graphics::rasterization::FrontFace::Clockwise,
                    ..Default::default()
                }),
                multisample_state: Some(graphics::multisample::MultisampleState::default()),
                color_blend_state: Some(graphics::color_blend::ColorBlendState {
                    attachments: [ColorBlendAttachmentState::default()].to_vec(),
                    ..Default::default()
                }),

                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),

                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|e| {
            crate::graphics::error::VulkanError::PipelineCreationError(format!(
                "Failed to create graphics pipeline: {}",
                e
            ))
        })
    }?;

    let viewport = vulkano::pipeline::graphics::viewport::Viewport {
        offset: [0.0, 0.0],
        extent: window_size.into(),
        ..Default::default()
    };

    let previous_frame_end = Some(vulkano::sync::now(device.clone()).boxed());

    Ok(RenderContext {
        window: window.clone(),
        viewport,
        swapchain,
        pipeline,
        recreate_swapchain: false,
        previous_frame_end,
        image_views,
    })
}

pub fn draw_scene(
    render_context: &mut RenderContext,
    command_buffer_allocator: Arc<
        vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
    >,
    descriptor_set_allocator: Arc<
        vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator,
    >,
    queue: Arc<vulkano::device::Queue>,
    renderable_scene: RenderableScene,
) -> Result<(), GraphicsError> {
    if render_context.recreate_swapchain {
        recreate_swapchain(render_context)?;
        render_context.recreate_swapchain = false;
    }
    let (image_index, suboptimal, acquire_future) =
        match vulkano::swapchain::acquire_next_image(render_context.swapchain.clone(), None)
            .map_err(vulkano::Validated::unwrap)
        {
            Ok(result) => result,
            Err(vulkano::VulkanError::OutOfDate) => {
                render_context.recreate_swapchain = true;
                return Ok(());
            }
            Err(e) => {
                return Err(error::GraphicsError::from(
                    error::VulkanError::SwapchainError(format!(
                        "Failed to acquire next image: {}",
                        e
                    )),
                ));
            }
        };

    if suboptimal {
        render_context.recreate_swapchain = true;
    }

    let descriptor_set = vulkano::descriptor_set::DescriptorSet::new(
        descriptor_set_allocator,
        render_context
            .pipeline
            .layout()
            .set_layouts()
            .get(0)
            .ok_or_else(|| {
                error::VulkanError::PipelineLayoutError(
                    "No descriptor set layout found in pipeline".to_string(),
                )
            })?
            .clone(),
        [
            vulkano::descriptor_set::WriteDescriptorSet::buffer(
                0,
                renderable_scene.uniform_buffer.clone(),
            ),
            vulkano::descriptor_set::WriteDescriptorSet::buffer(
                1,
                renderable_scene.matrix_buffer.clone(),
            ),
            vulkano::descriptor_set::WriteDescriptorSet::buffer(
                2,
                renderable_scene.material_buffer.clone(),
            ),
            vulkano::descriptor_set::WriteDescriptorSet::buffer(
                3,
                renderable_scene.normal_buffer.clone(),
            ),
        ],
        [],
    )
    .map_err(|e| {
        error::VulkanError::CommandBufferError(format!("Failed to create descriptor set: {}", e))
    })?;
    let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| {
        error::VulkanError::CommandBufferError(format!(
            "Failed to create command buffer builder: {}",
            e
        ))
    })?;

    builder
        .begin_rendering(vulkano::command_buffer::RenderingInfo {
            color_attachments: vec![Some(vulkano::command_buffer::RenderingAttachmentInfo {
                load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
                ..vulkano::command_buffer::RenderingAttachmentInfo::image_view(
                    render_context.image_views[image_index as usize].clone(),
                )
            })],
            ..Default::default()
        })
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!("Failed to begin rendering: {}", e))
        })?
        .set_viewport(
            0,
            vec![render_context.viewport.clone()].into_iter().collect(),
        )
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!("Failed to set viewport: {}", e))
        })?
        .bind_pipeline_graphics(render_context.pipeline.clone())
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!(
                "Failed to bind graphics pipeline: {}",
                e
            ))
        })?
        .bind_descriptor_sets(
            vulkano::pipeline::PipelineBindPoint::Graphics,
            render_context.pipeline.layout().clone(),
            0,
            descriptor_set,
        )
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!("Failed to bind descriptor sets: {}", e))
        })?
        .bind_vertex_buffers(0, renderable_scene.vertex_buffer)
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!("Failed to bind vertex buffers: {}", e))
        })?
        .bind_index_buffer(renderable_scene.index_buffer)
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!("Failed to bind index buffer: {}", e))
        })?;

    unsafe {
        builder
            .draw_indexed_indirect(renderable_scene.indirect_buffer)
            .map_err(|e| {
                error::VulkanError::CommandBufferError(format!(
                    "Failed to draw indexed indirect: {}",
                    e
                ))
            })?;
    }

    builder.end_rendering().map_err(|e| {
        error::VulkanError::CommandBufferError(format!("Failed to end rendering: {}", e))
    })?;

    let command_buffer = builder.build().map_err(|e| {
        error::VulkanError::CommandBufferError(format!("Failed to build command buffer: {}", e))
    })?;

    let future = render_context
        .previous_frame_end
        .take()
        .unwrap_or_else(|| vulkano::sync::now(queue.device().clone()).boxed())
        .join(acquire_future)
        .then_execute(queue.clone(), command_buffer)
        .map_err(|e| {
            error::VulkanError::CommandBufferError(format!(
                "Failed to execute command buffer: {}",
                e
            ))
        })?
        .then_swapchain_present(
            queue.clone(),
            vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
                render_context.swapchain.clone(),
                image_index,
            ),
        )
        .then_signal_fence_and_flush();

    match future.map_err(vulkano::Validated::unwrap) {
        Ok(future) => {
            render_context.previous_frame_end = Some(future.boxed());
        }
        Err(vulkano::VulkanError::OutOfDate) => {
            render_context.recreate_swapchain = true;
            render_context.previous_frame_end =
                Some(vulkano::sync::now(queue.device().clone()).boxed());
        }
        Err(e) => {
            return Err(GraphicsError::from(VulkanError::SwapchainError(format!(
                "Failed to flush future: {}",
                e
            ))));
        }
    }

    Ok(())
}

pub fn recreate_swapchain(render_context: &mut RenderContext) -> Result<(), VulkanError> {
    let new_window_size = render_context.window.inner_size();
    let (new_swapchain, new_images) = render_context
        .swapchain
        .recreate(vulkano::swapchain::SwapchainCreateInfo {
            image_extent: new_window_size.into(),
            ..render_context.swapchain.create_info()
        })
        .map_err(|e| {
            error::VulkanError::SwapchainError(format!("Failed to recreate swapchain: {}", e))
        })?;

    render_context.swapchain = new_swapchain;

    render_context.image_views = new_images
        .into_iter()
        .filter_map(|image| {
            vulkano::image::view::ImageView::new_default(image)
                .map_err(|e| {
                    error::VulkanError::SwapchainError(format!(
                        "Failed to create image view: {}",
                        e
                    ))
                })
                .ok()
        })
        .collect::<Vec<_>>();

    render_context.viewport.extent = new_window_size.into();
    Ok(())
}
