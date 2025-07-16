use std::{fmt::Debug, sync::Arc};

pub mod error;
pub mod vulkan_backend;
pub trait RenderBackend: Sized + Debug {
    type Context: RenderContext;
    type Error: Debug;

    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, Self::Error>;

    fn create_window_context(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: Arc<winit::window::Window>,
    ) -> Result<Self::Context, crate::graphics::error::GraphicsError>;

    fn draw_frame(
        &mut self,
        context: &mut Self::Context,
        gui_callback: impl FnOnce(&mut egui_winit_vulkano::egui::Context),
        scene: &crate::graphics::scene::Scene,
    ) -> Result<(), crate::graphics::error::GraphicsError>;
}

pub trait RenderContext {
    fn window(&self) -> Arc<winit::window::Window>;
    fn resize(&mut self) -> Result<(), crate::graphics::error::GraphicsError>;
}
