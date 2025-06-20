use Atom::graphics;
use Atom::reader;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut app = Atom::app::App::new();

    event_loop.run_app(&mut app);
}
