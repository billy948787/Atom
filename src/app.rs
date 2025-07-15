use std::{collections::HashMap, sync::Arc};

use egui_winit_vulkano::egui;
use vulkano::{
    descriptor_set,
    device::{DeviceFeatures, QueueCreateInfo, QueueFlags},
    instance::{InstanceCreateFlags, InstanceCreateInfo},
    sync::GpuFuture,
};
use winit::{
    application::ApplicationHandler,
    event::KeyEvent,
    keyboard::{Key, NamedKey},
    raw_window_handle::HasDisplayHandle,
    window::{self},
};

pub struct App {
    pub instance: Arc<vulkano::instance::Instance>,
    pub window_contexts: HashMap<winit::window::WindowId, WindowContext>,
    pub descriptor_set_allocator:
        Arc<vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator>,
    pub queue: Arc<vulkano::device::Queue>,
    pub device: Arc<vulkano::device::Device>,
    pub command_buffer_allocator:
        Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    pub scene: crate::graphics::scene::Scene,
    pub main_editor: crate::editor::Editor,
}

pub struct WindowContext {
    pub render_context: crate::graphics::rendering::RenderContext,
    pub gui: egui_winit_vulkano::Gui,
}

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

impl App {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let instance = Self::create_instance(event_loop);

        let (virtual_device, mut queues) =
            Self::create_virtual_device(Arc::clone(&instance), event_loop);

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

        let scene = crate::reader::obj_reader::read_file("test_model/Soccer/Soccer.obj").unwrap();

        return App {
            instance,
            window_contexts: HashMap::new(),
            queue: queues.next().expect("No queues found for the device"),
            device: virtual_device,
            command_buffer_allocator,
            memory_allocator,
            scene,
            descriptor_set_allocator,
            main_editor: crate::editor::Editor::default(),
        };
    }

    fn create_instance(event_loop: &impl HasDisplayHandle) -> Arc<vulkano::instance::Instance> {
        let library = vulkano::library::VulkanLibrary::new().unwrap();
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
            InstanceCreateInfo {
                application_name: Some("Atom Engine".to_string()),
                // max_api_version: Some(vulkano::Version::V1_0),
                enabled_extensions: required_extensions,

                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,

                ..Default::default()
            },
        )
        .unwrap();
    }

    fn create_virtual_device(
        instance: Arc<vulkano::instance::Instance>,
        event_loop: &impl HasDisplayHandle,
    ) -> (
        Arc<vulkano::device::Device>,
        impl ExactSizeIterator<Item = Arc<vulkano::device::Queue>>,
    ) {
        let mut device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            // khr_dynamic_rendering: true,
            ..vulkano::device::DeviceExtensions::empty()
        };

        let devices = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices");

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

                        queue_family.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && device
                                .presentation_support(*index as u32, event_loop)
                                .unwrap()
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

        let (device, queues) = vulkano::device::Device::new(
            suitable_device,
            vulkano::device::DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: queue_family_indexs
                    .iter()
                    .map(|index| QueueCreateInfo {
                        queue_family_index: *index,
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
                enabled_features: DeviceFeatures {
                    dynamic_rendering: true,
                    multi_draw_indirect: true,
                    shader_draw_parameters: true,
                    ..DeviceFeatures::empty()
                },
                ..Default::default()
            },
        )
        .unwrap();
        println!("Number of created queues: {}", queues.len());
        (device, queues)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // create a default window
        let window = Arc::new(
            event_loop
                .create_window(
                    window::Window::default_attributes()
                        .with_title("Atom Engine")
                        .with_resizable(true),
                )
                .unwrap(),
        );

        let window_id = window.id();

        let render_context = crate::graphics::rendering::create_render_context(
            window.clone(),
            self.device.clone(),
            self.instance.clone(),
            self.memory_allocator.clone(),
        )
        .unwrap();

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "jetbrains_mono".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/JetBrainsMono-Regular.ttf"
            ))),
        );

        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "jetbrains_mono".to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional) // 使用 Proportional 以便預設套用
            .unwrap()
            .insert(0, "jetbrains_mono".to_owned());

        let gui = egui_winit_vulkano::Gui::new(
            event_loop,
            vulkano::swapchain::Surface::from_window(self.instance.clone(), window.clone())
                .unwrap(),
            self.queue.clone(),
            render_context.swapchain.image_format(),
            egui_winit_vulkano::GuiConfig {
                is_overlay: true,
                ..Default::default()
            },
        );

        gui.egui_ctx.set_fonts(fonts);

        self.window_contexts.insert(
            window_id,
            WindowContext {
                render_context,
                gui,
            },
        );

        println!("Window created with ID: {:?}", window_id);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let window_context = self.window_contexts.get_mut(&window_id).unwrap();
        let captured_by_gui = window_context.gui.update(&event);

        if captured_by_gui {
            window_context.render_context.window.request_redraw();
            return;
        }
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                println!("Window resized to: {:?}", size);

                if let Some(window_context) = self.window_contexts.get_mut(&window_id) {
                    window_context.render_context.recreate_swapchain = true;
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                let aspect_ratio = {
                    let window_context = self.window_contexts.get(&window_id).unwrap();
                    let extent = window_context.render_context.window.inner_size();
                    extent.width as f32 / extent.height as f32
                };

                let (image_index, future) = crate::graphics::rendering::draw_scene(
                    &mut self
                        .window_contexts
                        .get_mut(&window_id)
                        .unwrap()
                        .render_context,
                    self.command_buffer_allocator.clone(),
                    self.descriptor_set_allocator.clone(),
                    self.memory_allocator.clone(),
                    self.queue.clone(),
                    crate::graphics::rendering::RenderableScene::from_scene(
                        &self.scene,
                        self.memory_allocator.clone(),
                        aspect_ratio,
                    )
                    .unwrap(),
                )
                .unwrap();

                let window_context = self.window_contexts.get_mut(&window_id).unwrap();

                window_context.gui.immediate_ui(|ui| {
                    let ctx = ui.context();
                    egui::Window::new("Hello Egui").show(&ctx, |ui| {
                        ui.label("This is a label inside a window.");
                        if ui.button("Click me").clicked() {
                            println!("Button clicked!");
                        }
                    });
                });

                let ui_future = window_context.gui.draw_on_image(
                    future,
                    window_context.render_context.image_views[image_index as usize].clone(),
                );

                let _ = ui_future
                    .then_swapchain_present(
                        self.queue.clone(),
                        vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
                            window_context.render_context.swapchain.clone(),
                            image_index,
                        ),
                    )
                    .then_signal_fence_and_flush()
                    .unwrap();

                window_context.render_context.window.request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let mut need_redraw = true;
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW) => {
                        self.scene.cameras[0].position.z -= 0.1;
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS) => {
                        self.scene.cameras[0].position.z += 0.1;
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA) => {
                        self.scene.cameras[0].position.x -= 0.1;
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD) => {
                        self.scene.cameras[0].position.x += 0.1;
                    }
                    // rotate camera
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) => {
                        self.scene.cameras[0].rotate(glam::Vec3::new(0.0, -0.1, 0.0));
                    }

                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft) => {
                        self.scene.cameras[0].rotate(glam::Vec3::new(0.0, 0.1, 0.0));
                    }

                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp) => {
                        self.scene.cameras[0].rotate(glam::Vec3::new(-0.1, 0.0, 0.0));
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown) => {
                        self.scene.cameras[0].rotate(glam::Vec3::new(0.1, 0.0, 0.0));
                    }

                    _ => {
                        need_redraw = false;
                    }
                }

                if need_redraw {
                    if let Some(window_context) = self.window_contexts.get_mut(&window_id) {
                        window_context.render_context.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
