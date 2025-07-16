fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut app =
        atom::app::App::<atom::graphics::backend::vulkan_backend::VulkanBackend>::new(&event_loop);

    event_loop.run_app(&mut app).unwrap();
}
