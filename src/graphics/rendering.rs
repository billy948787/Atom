use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Deref, RangeInclusive},
    sync::Arc,
};

use vulkano::{
    buffer::BufferCreateInfo,
    image::ImageUsage,
    pipeline::{
        DynamicState, GraphicsPipeline,
        graphics::{
            self, GraphicsPipelineCreateInfo,
            color_blend::ColorBlendAttachmentState,
            vertex_input::{Vertex, VertexDefinition},
        },
    },
    swapchain::{Swapchain, SwapchainCreateInfo},
    sync::GpuFuture,
};

use crate::graphics::{
    error::{self, GraphicsError, VulkanError},
    scene,
};

// fn draw(scene: &scene::Scene, render_context: &RenderContext) -> Result<(), GraphicsError> {
//     let serface = vulkano::swapchain::Surface::from_window(instance, window)
//     Ok(())
// }

pub struct RenderContext {
    pub window: Arc<winit::window::Window>,
    pub swapchain: Arc<vulkano::swapchain::Swapchain>,
    pub pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    pub viewport: vulkano::pipeline::graphics::viewport::Viewport,
    pub previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
    pub image_views: Vec<Arc<vulkano::image::view::ImageView>>,
    pub recreate_swapchain: bool,
}
pub fn create_render_context(
    window: Arc<winit::window::Window>,
    device: Arc<vulkano::device::Device>,
    instance: Arc<vulkano::instance::Instance>,
    memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
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

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/graphics/shaders/triangle.vert",
        }
    }

    mod fs {
        vulkano_shaders::shader! {
                ty: "fragment",
                path: "src/graphics/shaders/triangle.frag",
        }
    }

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

        let layout = vulkano::pipeline::PipelineLayout::new(
            device.clone(),
            vulkano::pipeline::layout::PipelineLayoutCreateInfo {
                set_layouts: vec![],
                push_constant_ranges: vec![],
                ..Default::default()
            },
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
                rasterization_state: Some(graphics::rasterization::RasterizationState::default()),
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
    queue: Arc<vulkano::device::Queue>,
    scene: &scene::Scene,
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
        })?.bind_vertex_buffers(0, scene);

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
    Ok(())
}
