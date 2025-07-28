use std::{collections::HashMap, sync::Arc};

use winit::{
    application::ApplicationHandler,
    window::{self, Fullscreen},
};

use crate::{graphics::backend::RenderContext, reader::obj_reader};

pub struct App<B: crate::graphics::backend::RenderBackend> {
    pub render_backend: B,
    pub main_editor: crate::editor::Editor,
    pub window_contexts: HashMap<winit::window::WindowId, B::Context>,
}

impl<B: crate::graphics::backend::RenderBackend> App<B> {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        return App {
            render_backend: B::new(event_loop).unwrap(),
            main_editor: crate::editor::Editor::default(),
            window_contexts: HashMap::new(),
        };
    }
}

impl<B: crate::graphics::backend::RenderBackend> ApplicationHandler for App<B> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // create a default window
        let window = Arc::new(
            event_loop
                .create_window(
                    window::Window::default_attributes()
                        .with_title("Atom Engine")
                        // .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
                        .with_maximized(true)
                        .with_resizable(true),
                )
                .unwrap(),
        );

        let window_id = window.id();

        let render_context = self
            .render_backend
            .create_window_context(&event_loop, window.clone())
            .unwrap();

        self.window_contexts.insert(window_id, render_context);

        println!("Window created with ID: {:?}", window_id);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let window_context = self.window_contexts.get_mut(&window_id).unwrap();
        if window_context.gui_update(&event) {
            window_context.window().request_redraw();
            return; // If the GUI handled the event, skip further processing
        }
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(new_size) => {
                println!("Window resized to: {:?}", new_size);

                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }

                if let Some(window_context) = self.window_contexts.get_mut(&window_id) {
                    window_context.resize().unwrap();
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                // check the window size
                if let Some(window_context) = self.window_contexts.get_mut(&window_id) {
                    if window_context.window().inner_size().width == 0
                        || window_context.window().inner_size().height == 0
                    {
                        return;
                    }
                }

                if let Some(window_context) = self.window_contexts.get_mut(&window_id) {
                    let scene = self.main_editor.scene.clone();

                    self.render_backend
                        .draw_frame(
                            window_context,
                            |context| {
                                self.main_editor.ui(&context);
                            },
                            &scene,
                        )
                        .unwrap();
                    window_context.window().request_redraw();
                }
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match event.physical_key {
                _ => {}
            },
            _ => {}
        }
    }
}
