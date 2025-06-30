use std::sync::Arc;

pub trait RenderBackend {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self;

    fn create_window_context(
        &mut self,
        window: Arc<winit::window::Window>,
    ) -> Result<(), crate::graphics::error::GraphicsError>;
}
