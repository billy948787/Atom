use std::{collections::HashMap, sync::Arc};

use vulkano::{
    descriptor_set,
    device::{DeviceFeatures, QueueCreateInfo, QueueFlags},
    instance::{InstanceCreateFlags, InstanceCreateInfo},
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
    pub render_contexts:
        HashMap<winit::window::WindowId, crate::graphics::rendering::RenderContext>,
    pub descriptor_set_allocator:
        Arc<vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator>,
    pub queue: Arc<vulkano::device::Queue>,
    pub device: Arc<vulkano::device::Device>,
    pub command_buffer_allocator:
        Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
    pub scene: crate::graphics::scene::Scene,
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

        let scene = crate::reader::obj_reader::read_file("test_model/Ferrari/Ferrari.obj").unwrap();

        return App {
            instance,
            render_contexts: HashMap::new(),
            queue: queues.next().expect("No queues found for the device"),
            device: virtual_device,
            command_buffer_allocator,
            memory_allocator,
            scene,
            descriptor_set_allocator,
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
        let window = event_loop
            .create_window(
                window::Window::default_attributes()
                    .with_title("Atom Engine")
                    .with_resizable(true),
            )
            .unwrap();

        let window_id = window.id();

        self.render_contexts.insert(
            window_id,
            crate::graphics::rendering::create_render_context(
                Arc::new(window),
                self.device.clone(),
                self.instance.clone(),
            )
            .unwrap(),
        );

        println!("Window created with ID: {:?}", window_id);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                println!("Window resized to: {:?}", size);

                if let Some(render_context) = self.render_contexts.get_mut(&window_id) {
                    render_context.recreate_swapchain = true;
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                let aspect_ratio = {
                    let render_context = self.render_contexts.get(&window_id).unwrap();
                    let extent = render_context.window.inner_size();
                    extent.width as f32 / extent.height as f32
                };

                crate::graphics::rendering::draw_scene(
                    self.render_contexts.get_mut(&window_id).unwrap(),
                    self.command_buffer_allocator.clone(),
                    self.descriptor_set_allocator.clone(),
                    self.queue.clone(),
                    crate::graphics::rendering::RenderableScene::from_scene(
                        &self.scene,
                        self.memory_allocator.clone(),
                        aspect_ratio,
                    )
                    .unwrap(),
                )
                .unwrap();
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
                    if let Some(render_context) = self.render_contexts.get_mut(&window_id) {
                        render_context.recreate_swapchain = true;
                        render_context.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
