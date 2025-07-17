use std::sync::Arc;

use vulkano::pipeline::Pipeline;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::graphics::vertex_input::VertexDefinition;
use vulkano::sync::GpuFuture;

use crate::graphics::backend::error::VulkanError;
use crate::graphics::backend::{RenderBackend, RenderContext};

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

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

#[derive(Debug)]
pub struct VulkanBackend {
    instance: Arc<vulkano::instance::Instance>,
    pub descriptor_set_allocator:
        Arc<vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator>,
    pub queue: Arc<vulkano::device::Queue>,
    pub device: Arc<vulkano::device::Device>,
    pub command_buffer_allocator:
        Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
}

pub struct VulkanContext {
    pub window: Arc<winit::window::Window>,
    pub swapchain: Arc<vulkano::swapchain::Swapchain>,
    pub pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    pub viewport: vulkano::pipeline::graphics::viewport::Viewport,
    pub previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
    pub image_views: Vec<Arc<vulkano::image::view::ImageView>>,
    pub need_recreate_swapchain: bool,
    pub depth_buffer: Arc<vulkano::image::view::ImageView>,
    pub gui: egui_winit_vulkano::Gui,
}
pub struct VulkanScene {
    pub vertex_buffer: vulkano::buffer::Subbuffer<[crate::graphics::vertex::Vertex]>,
    pub index_buffer: vulkano::buffer::Subbuffer<[u32]>,
    pub uniform_buffer: vulkano::buffer::Subbuffer<vs::CameraUbo>,
    pub matrix_buffer: vulkano::buffer::Subbuffer<[glam::Mat4]>,
    pub normal_buffer: vulkano::buffer::Subbuffer<[glam::Mat4]>,
    pub indirect_buffer:
        vulkano::buffer::Subbuffer<[vulkano::command_buffer::DrawIndexedIndirectCommand]>,
    pub material_buffer:
        vulkano::buffer::Subbuffer<[crate::graphics::material::MaterialProperties]>,
}

impl VulkanScene {
    pub fn from_scene(
        scene: &crate::graphics::scene::Scene,
        memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
        aspect_ratio: f32,
    ) -> Result<Self, crate::graphics::error::GraphicsError> {
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
            .ok_or_else(|| crate::graphics::error::GraphicsError::NoCameraFound)?
            .view_matrix();

        for mesh in &scene.objects {
            let model_matrix = mesh.world_transform;
            let normal_matrix = (view_matrix * mesh.world_transform).inverse().transpose();

            for submesh in &mesh.submeshes {
                if submesh.vertices.is_empty() || submesh.indices.is_empty() {
                    continue; // Skip empty submeshes
                }

                model_matrices.push(model_matrix);
                normal_matrices.push(normal_matrix);

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
                materials.push(submesh.material.properties.clone());
                instance_id += 1;
            }
        }
        let vertex_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_vertices.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                "Failed to create vertex buffer: {}",
                e
            ))
        })?;

        let index_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_indices.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create index buffer: {}",
                    e
                )),
            )
        })?;

        let indirect_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::INDIRECT_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indirect_commands.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create indirect buffer: {}",
                    e
                )),
            )
        })?;

        let projection_matrix = scene
            .cameras
            .get(scene.main_camera_index)
            .ok_or_else(|| crate::graphics::error::GraphicsError::NoCameraFound)?
            .projection_matrix(aspect_ratio);

        let view_matrix = scene
            .cameras
            .get(scene.main_camera_index)
            .ok_or_else(|| crate::graphics::error::GraphicsError::NoCameraFound)?
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
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            ubo_data,
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create uniform buffer: {}",
                    e
                )),
            )
        })?;

        let matrix_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            model_matrices.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create matrix buffer: {}",
                    e
                )),
            )
        })?;

        let material_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            materials.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create material buffer: {}",
                    e
                )),
            )
        })?;

        let normal_buffer = vulkano::buffer::Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: vulkano::buffer::BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: vulkano::memory::allocator::MemoryTypeFilter::PREFER_DEVICE
                    | vulkano::memory::allocator::MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            normal_matrices.into_iter(),
        )
        .map_err(|e| {
            crate::graphics::error::GraphicsError::from(
                crate::graphics::backend::error::VulkanError::BufferCreationError(format!(
                    "Failed to create normal buffer: {}",
                    e
                )),
            )
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
impl VulkanBackend {
    pub fn recreate_swapchain(
        &self,
        render_context: &mut VulkanContext,
    ) -> Result<(), VulkanError> {
        let new_window_size = render_context.window.inner_size();
        let (new_swapchain, new_images) = render_context
            .swapchain
            .recreate(vulkano::swapchain::SwapchainCreateInfo {
                image_extent: new_window_size.into(),
                ..render_context.swapchain.create_info()
            })
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                    "Failed to recreate swapchain: {}",
                    e
                ))
            })?;

        render_context.swapchain = new_swapchain;

        render_context.image_views = new_images
            .into_iter()
            .filter_map(|image| {
                vulkano::image::view::ImageView::new_default(image)
                    .map_err(|e| {
                        crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                            "Failed to create image view: {}",
                            e
                        ))
                    })
                    .ok()
            })
            .collect::<Vec<_>>();

        render_context.viewport.extent = new_window_size.into();

        render_context.depth_buffer = create_depth_buffer(
            self.memory_allocator.clone(),
            [new_window_size.width, new_window_size.height],
        )?;
        Ok(())
    }
}

impl RenderBackend for VulkanBackend {
    type Context = VulkanContext;
    type Error = crate::graphics::backend::error::VulkanError;
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, Self::Error> {
        let instance = create_instance(event_loop)?;

        let (virtual_device, mut queues) =
            create_virtual_device(Arc::clone(&instance), event_loop)?;

        let memory_allocator = Arc::new(
            vulkano::memory::allocator::StandardMemoryAllocator::new_default(
                virtual_device.clone(),
            ),
        );

        let command_buffer_allocator = Arc::new(
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator::new(
                Arc::clone(&virtual_device),
                Default::default(),
            ),
        );

        let descriptor_set_allocator = Arc::new(
            vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
                Arc::clone(&virtual_device),
                Default::default(),
            ),
        );

        Ok(Self {
            instance,
            device: virtual_device,
            queue: queues.next().expect("No queues available"),
            descriptor_set_allocator,
            command_buffer_allocator,
            memory_allocator,
        })
    }

    fn create_window_context(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: Arc<winit::window::Window>,
    ) -> Result<Self::Context, crate::graphics::error::GraphicsError> {
        let surface =
            vulkano::swapchain::Surface::from_window(self.instance.clone(), window.clone())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::SurfaceCreationError(format!(
                        "Failed to create surface from window: {}",
                        e
                    ))
                })?;

        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = self
                .device
                .clone()
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                        "Failed to get surface capabilities: {}",
                        e
                    ))
                })?;

            let (image_format, _) = self
                .device
                .clone()
                .physical_device()
                .surface_formats(&surface, Default::default())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                        "Failed to get surface formats: {}",
                        e
                    ))
                })?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    crate::graphics::backend::error::VulkanError::SwapchainError(
                        "No suitable surface format found".to_string(),
                    )
                })?;

            vulkano::swapchain::Swapchain::new(
                self.device.clone(),
                surface,
                vulkano::swapchain::SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: vulkano::image::ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap_or(vulkano::swapchain::CompositeAlpha::Opaque),
                    ..Default::default()
                },
            )
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                    "Failed to create swapchain: {}",
                    e
                ))
            })?
        };

        let image_views = images
            .into_iter()
            .filter_map(|image| {
                vulkano::image::view::ImageView::new_default(image)
                    .map_err(|e| {
                        crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                            "Failed to create image view: {}",
                            e
                        ))
                    })
                    .ok()
            })
            .collect::<Vec<_>>();

        let pipeline = {
            let vs = vs::load(self.device.clone())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::ShaderCompilationError(format!(
                        "Failed to load vertex shader: {}",
                        e
                    ))
                })?
                .entry_point("main")
                .ok_or_else(|| {
                    crate::graphics::backend::error::VulkanError::ShaderCompilationError(
                        "Vertex shader entry point 'main' not found".to_string(),
                    )
                })?;

            let fs = fs::load(self.device.clone())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::ShaderCompilationError(format!(
                        "Failed to load fragment shader: {}",
                        e
                    ))
                })?
                .entry_point("main")
                .ok_or_else(|| {
                    crate::graphics::backend::error::VulkanError::ShaderCompilationError(
                        "Fragment shader entry point 'main' not found".to_string(),
                    )
                })?;

            let vertex_input_state = crate::graphics::vertex::Vertex::per_vertex()
                .definition(&vs)
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::ShaderCompilationError(format!(
                        "Failed to define vertex input: {}",
                        e
                    ))
                })?;

            let stages = [
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(vs),
                vulkano::pipeline::PipelineShaderStageCreateInfo::new(fs),
            ];

            let pipeline_layout_create_info =
                vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo::from_stages(
                    &stages,
                )
                .into_pipeline_layout_create_info(self.device.clone())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::PipelineLayoutError(format!(
                        "Failed to create pipeline layout: {}",
                        e
                    ))
                })?;

            println!(
                "Pipeline layout create info: {:?}",
                pipeline_layout_create_info
            );
            let layout = vulkano::pipeline::layout::PipelineLayout::new(
                self.device.clone(),
                pipeline_layout_create_info,
            )
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::PipelineLayoutError(format!(
                    "Failed to create pipeline layout: {}",
                    e
                ))
            })?;
            let subpass = vulkano::pipeline::graphics::subpass::PipelineRenderingCreateInfo {
                color_attachment_formats: [Some(swapchain.image_format())].to_vec(),
                depth_attachment_format: Some(vulkano::format::Format::D32_SFLOAT),
                ..Default::default()
            };

            vulkano::pipeline::graphics::GraphicsPipeline::new(
                self.device.clone(),
                None,
                vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
                    stages: stages.to_vec().into(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(
                        vulkano::pipeline::graphics::input_assembly::InputAssemblyState::default(),
                    ),
                    viewport_state: Some(
                        vulkano::pipeline::graphics::viewport::ViewportState::default(),
                    ),
                    rasterization_state: Some(
                        vulkano::pipeline::graphics::rasterization::RasterizationState {
                            cull_mode: vulkano::pipeline::graphics::rasterization::CullMode::Back,
                            front_face:
                                vulkano::pipeline::graphics::rasterization::FrontFace::Clockwise,
                            ..Default::default()
                        },
                    ),
                    multisample_state: Some(
                        vulkano::pipeline::graphics::multisample::MultisampleState::default(),
                    ),
                    color_blend_state: Some(
                        vulkano::pipeline::graphics::color_blend::ColorBlendState {
                            attachments: [vulkano::pipeline::graphics::color_blend::ColorBlendAttachmentState::default()].to_vec(),
                            ..Default::default()
                        },
                    ),

                    depth_stencil_state: Some(
                        vulkano::pipeline::graphics::depth_stencil::DepthStencilState {
                            depth: Some(
                                vulkano::pipeline::graphics::depth_stencil::DepthState::simple(),
                            ),
                            ..Default::default()
                        },
                    ),

                    dynamic_state: [vulkano::pipeline::DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.into()),

                    ..vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::PipelineCreationError(format!(
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

        let previous_frame_end = Some(vulkano::sync::GpuFuture::boxed(vulkano::sync::now(
            self.device.clone(),
        )));

        let depth_buffer = create_depth_buffer(
            self.memory_allocator.clone(),
            [window_size.width, window_size.height],
        )?;

        let gui = egui_winit_vulkano::Gui::new(
            event_loop,
            vulkano::swapchain::Surface::from_window(self.instance.clone(), window.clone())
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::SurfaceCreationError(format!(
                        "Failed to create surface from window: {}",
                        e
                    ))
                })?,
            self.queue.clone(),
            swapchain.image_format(),
            egui_winit_vulkano::GuiConfig {
                is_overlay: true,
                ..Default::default()
            },
        );

        Ok(VulkanContext {
            window: window.clone(),
            viewport,
            swapchain,
            depth_buffer,
            pipeline,
            need_recreate_swapchain: false,
            previous_frame_end,
            image_views,
            gui,
        })
    }

    fn draw_frame(
        &mut self,
        context: &mut Self::Context,
        gui_callback: impl FnOnce(&mut egui_winit_vulkano::egui::Context),
        scene: &crate::graphics::scene::Scene,
    ) -> Result<(), crate::graphics::error::GraphicsError> {
        if context.need_recreate_swapchain {
            self.recreate_swapchain(context).map_err(|e| {
                crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                    "Failed to recreate swapchain: {}",
                    e
                ))
            })?;
            context.need_recreate_swapchain = false;
        }
        let (image_index, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(context.swapchain.clone(), None)
                .map_err(vulkano::Validated::unwrap)
            {
                Ok(result) => result,
                Err(vulkano::VulkanError::OutOfDate) => {
                    context.need_recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => {
                    return Err(
                        crate::graphics::backend::error::VulkanError::SwapchainError(format!(
                            "Failed to acquire next image: {}",
                            e
                        ))
                        .into(),
                    );
                }
            };

        if suboptimal {
            context.need_recreate_swapchain = true;
        }

        let aspect_ratio = {
            let extent = context.window.inner_size();
            extent.width as f32 / extent.height as f32
        };

        let renderable_scene =
            VulkanScene::from_scene(scene, self.memory_allocator.clone(), aspect_ratio).map_err(
                |e| {
                    crate::graphics::backend::error::VulkanError::SceneError(format!(
                        "Failed to create renderable scene: {}",
                        e
                    ))
                },
            )?;

        let descriptor_set = vulkano::descriptor_set::DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            context
                .pipeline
                .layout()
                .set_layouts()
                .get(0)
                .ok_or_else(|| {
                    crate::graphics::backend::error::VulkanError::PipelineLayoutError(
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
            crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                "Failed to create descriptor set: {}",
                e
            ))
        })?;
        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| {
            crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
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
                        context.image_views[image_index as usize].clone(),
                    )
                })],
                depth_attachment: Some(vulkano::command_buffer::RenderingAttachmentInfo {
                    load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
                    store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                    clear_value: Some(1.0f32.into()),
                    ..vulkano::command_buffer::RenderingAttachmentInfo::image_view(
                        context.depth_buffer.clone(),
                    )
                }),
                ..Default::default()
            })
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to begin rendering: {}",
                    e
                ))
            })?
            .set_viewport(0, vec![context.viewport.clone()].into_iter().collect())
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to set viewport: {}",
                    e
                ))
            })?
            .bind_pipeline_graphics(context.pipeline.clone())
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to bind graphics pipeline: {}",
                    e
                ))
            })?
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                context.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to bind descriptor sets: {}",
                    e
                ))
            })?
            .bind_vertex_buffers(0, renderable_scene.vertex_buffer)
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to bind vertex buffers: {}",
                    e
                ))
            })?
            .bind_index_buffer(renderable_scene.index_buffer)
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to bind index buffer: {}",
                    e
                ))
            })?;

        unsafe {
            builder
                .draw_indexed_indirect(renderable_scene.indirect_buffer)
                .map_err(|e| {
                    crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                        "Failed to draw indexed indirect: {}",
                        e
                    ))
                })?;
        }

        context.gui.immediate_ui(|gui| {
            gui_callback(&mut gui.context());
        });

        builder.end_rendering().map_err(|e| {
            crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                "Failed to end rendering: {}",
                e
            ))
        })?;

        let command_buffer = builder.build().map_err(|e| {
            crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                "Failed to build command buffer: {}",
                e
            ))
        })?;

        let scene_future = context
            .previous_frame_end
            .take()
            .unwrap_or_else(|| vulkano::sync::now(self.queue.device().clone()).boxed())
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to execute command buffer: {}",
                    e
                ))
            })?;

        let gui_future = context.gui.draw_on_image(
            scene_future,
            context.image_views[image_index as usize].clone(),
        );

        let _ = gui_future
            .then_swapchain_present(
                self.queue.clone(),
                vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
                    context.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush()
            .map_err(|e| {
                crate::graphics::backend::error::VulkanError::CommandBufferError(format!(
                    "Failed to flush command buffer: {}",
                    e
                ))
            })?
            .cleanup_finished();

        Ok(())
    }
}

impl RenderContext for VulkanContext {
    fn resize(&mut self) -> Result<(), crate::graphics::error::GraphicsError> {
        let size = self.window.inner_size();
        self.need_recreate_swapchain = true;
        self.viewport.extent = size.into();

        Ok(())
    }

    fn gui_update(&mut self, winit_event: &winit::event::WindowEvent) -> bool {
        self.gui.update(&winit_event)
    }
    fn window(&self) -> Arc<winit::window::Window> {
        self.window.clone()
    }
}

impl VulkanContext {}

fn create_depth_buffer(
    memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    extent: [u32; 2],
) -> Result<Arc<vulkano::image::view::ImageView>, super::error::VulkanError> {
    let image = vulkano::image::Image::new(
        memory_allocator,
        vulkano::image::ImageCreateInfo {
            image_type: vulkano::image::ImageType::Dim2d,
            format: vulkano::format::Format::D32_SFLOAT,
            extent: [extent[0], extent[1], 1].into(),
            usage: vulkano::image::ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            ..Default::default()
        },
        vulkano::memory::allocator::AllocationCreateInfo::default(),
    )
    .map_err(|e| {
        crate::graphics::backend::error::VulkanError::ImageCreationError(format!(
            "Failed to create depth image: {}",
            e
        ))
    })?;

    vulkano::image::view::ImageView::new_default(image).map_err(|e| {
        crate::graphics::backend::error::VulkanError::ImageViewCreationError(format!(
            "Failed to create depth image view: {}",
            e
        ))
    })
}

fn create_instance(
    event_loop: &impl winit::raw_window_handle::HasDisplayHandle,
) -> Result<Arc<vulkano::instance::Instance>, crate::graphics::backend::error::VulkanError> {
    let library = vulkano::library::VulkanLibrary::new().map_err(|e| {
        crate::graphics::backend::error::VulkanError::InstanceCreationError(format!(
            "Failed to create Vulkan library: {}",
            e
        ))
    })?;
    let mut required_extensions = vulkano::swapchain::Surface::required_extensions(&event_loop)
        .expect("Failed to get required extensions");

    required_extensions.khr_portability_enumeration = true;

    let mut enable_layers = vec![];
    if ENABLE_VALIDATION_LAYERS {
        enable_layers.push("VK_LAYER_KHRONOS_validation");
        // required_extensions.ext_validation_features = true;
        required_extensions.ext_debug_utils = true;
    }

    return vulkano::instance::Instance::new(
        library,
        vulkano::instance::InstanceCreateInfo {
            application_name: Some("Atom Engine".to_string()),
            // max_api_version: Some(vulkano::Version::V1_0),
            enabled_extensions: required_extensions,

            flags: vulkano::instance::InstanceCreateFlags::ENUMERATE_PORTABILITY,

            ..Default::default()
        },
    )
    .map_err(|e| {
        crate::graphics::backend::error::VulkanError::InstanceCreationError(format!(
            "Failed to create Vulkan instance: {}",
            e
        ))
    });
}
fn create_virtual_device(
    instance: Arc<vulkano::instance::Instance>,
    event_loop: &impl winit::raw_window_handle::HasDisplayHandle,
) -> Result<
    (
        Arc<vulkano::device::Device>,
        impl ExactSizeIterator<Item = Arc<vulkano::device::Queue>>,
    ),
    crate::graphics::backend::error::VulkanError,
> {
    let mut device_extensions = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        // khr_dynamic_rendering: true,
        ..vulkano::device::DeviceExtensions::empty()
    };

    let devices = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices");

    let required_features = vulkano::device::DeviceFeatures {
        dynamic_rendering: true,
        multi_draw_indirect: true,
        shader_draw_parameters: true,
        image_view_format_swizzle: true,
        ..vulkano::device::DeviceFeatures::empty()
    };

    let (suitable_device, queue_family_indexs) = devices
        .into_iter()
        .filter(|device| {
            device.api_version() >= vulkano::Version::V1_3
                || device.supported_extensions().khr_dynamic_rendering
        })
        .filter(|device| device.supported_extensions().contains(&device_extensions))
        .map(|device| {
            let indexs = device
                .queue_family_properties()
                .iter()
                .enumerate()
                .filter(|(index, queue_family)| {
                    println!("Queue family {}: {:?}", index, queue_family.queue_flags);

                    queue_family
                        .queue_flags
                        .intersects(vulkano::device::QueueFlags::GRAPHICS)
                        && device
                            .presentation_support(*index as u32, event_loop)
                            .map_or(false, |support| support)
                })
                .map(|(index, _)| index as u32)
                .collect::<Vec<_>>();

            (device, indexs)
        })
        .min_by_key(|(device, _)| match device.properties().device_type {
            vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 0,
            vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 1,
            vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 2,
            vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
            vulkano::device::physical::PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("No suitable device found");

    println!("Using device: {}", suitable_device.properties().device_name);

    if suitable_device.api_version() < vulkano::Version::V1_3 {
        device_extensions.khr_dynamic_rendering = true;
    }
    let enabled_features = suitable_device
        .supported_features()
        .intersection(&required_features);

    println!("Enabled features: {:?}", enabled_features);
    let (device, queues) = vulkano::device::Device::new(
        suitable_device,
        vulkano::device::DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: queue_family_indexs
                .iter()
                .map(|index| vulkano::device::QueueCreateInfo {
                    queue_family_index: *index,
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
            enabled_features,
            ..Default::default()
        },
    )
    .map_err(|e| {
        crate::graphics::backend::error::VulkanError::DeviceCreationError(format!(
            "Failed to create Vulkan device: {}",
            e
        ))
    })?;
    println!("Number of created queues: {}", queues.len());
    Ok((device, queues))
}
