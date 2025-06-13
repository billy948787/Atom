use glfw::{Action, Context, Key, PWindow, WindowEvent};

pub fn create_window(
    height: u32,
    width: u32,
    title: &str,
    window_control_cb: fn() -> bool,
    event_control_cb: fn(WindowEvent),
) {
    use glfw::fail_on_errors;
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    let (mut window, events) = glfw
        .create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);

    while !window.should_close() && window_control_cb() {
        // Swap front and back buffers
        window.swap_buffers();

        // Poll for and process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            event_control_cb(event);
        }
    }
}
