use std::sync::Arc;

use crate::graphics::{
    error::{self, GraphicsError},
    scene,
};

// fn draw(scene: &scene::Scene, render_context: &RenderContext) -> Result<(), GraphicsError> {
//     let serface = vulkano::swapchain::Surface::from_window(instance, window)
//     Ok(())
// }

pub struct RenderContext {
    window: Arc<winit::window::Window>,
    scene: scene::Scene,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipeline>,
    render_pass: Arc<vulkano::render_pass::RenderPass>,
    viewport: vulkano::pipeline::graphics::viewport::Viewport,
}

// fn create_render_context(
//     window: Arc<winit::window::Window>,
//     scene: scene::Scene,
// ) -> Result<RenderContext, GraphicsError> {
    

// }
