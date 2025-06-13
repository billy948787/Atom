use Atom::graphics;
use Atom::reader;

fn main() {
    graphics::rendering::create_window(
        800,
        800,
        "selector",
        || {
            return true;
        },
        |event| {
            use glfw::{Action, Key, WindowEvent};

            match event {
                WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    println!("Space key pressed.");
                }
                _ => {}
            }
        },
    );

    let scene = match reader::obj_reader::read_file("test_model/Cube.obj") {
        Ok(scene) => scene,
        Err(e) => {
            eprintln!("Error reading file: {:?}", e);
            return;
        }
    };

    println!("Scene loaded successfully: {:#?}", scene);
}
