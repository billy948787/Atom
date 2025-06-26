use std::{
    collections::{HashMap, hash_map},
    sync::Arc,
};

use vulkano::{
    device::{DeviceFeatures, QueueCreateInfo, QueueFlags},
    instance::{InstanceCreateFlags, InstanceCreateInfo},
    pipeline::graphics,
};
use winit::{
    application::ApplicationHandler,
    raw_window_handle::HasDisplayHandle,
    window::{self, WindowAttributes},
};

pub struct App {
    pub instance: Arc<vulkano::instance::Instance>,
    pub render_context: HashMap<winit::window::WindowId, crate::graphics::rendering::RenderContext>,
    pub queue: Arc<vulkano::device::Queue>,
    pub device: Arc<vulkano::device::Device>,
    pub command_buffer_allocator:
        Arc<vulkano::command_buffer::allocator::StandardCommandBufferAllocator>,
    pub memory_allocator: Arc<vulkano::memory::allocator::StandardMemoryAllocator>,
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

        let memory_allocator = Arc::new(vulkano::memory::allocator::StandardMemoryAllocator::new(
            Arc::clone(&virtual_device),
            Default::default(),
        ));

        let command_buffer_allocator = Arc::new(
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator::new(
                Arc::clone(&virtual_device),
                Default::default(),
            ),
        );

        return App {
            instance,
            render_context: HashMap::new(),
            queue: queues.next().expect("No queues found for the device"),
            device: virtual_device,
            command_buffer_allocator,
            memory_allocator,
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
                        queue_family.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && device
                                .presentation_support(*index as u32, event_loop)
                                .unwrap()
                    })
                    .map(|(index, _)| index as u32)
                    .collect::<Vec<_>>();

                (device, indexs)
            })
            .min_by_key(
                |(device, queue_family_index)| match device.properties().device_type {
                    vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 0,
                    vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 1,
                    vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 2,
                    vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
                    vulkano::device::physical::PhysicalDeviceType::Other => 4,
                    _ => 5,
                },
            )
            .expect("No suitable device found");

        println!("Using device: {}", suitable_device.properties().device_name);

        if suitable_device.api_version() < vulkano::Version::V1_3 {
            device_extensions.khr_dynamic_rendering = true;
        }

        println!("{:?}", queue_family_indexs);

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
            }
            winit::event::WindowEvent::RedrawRequested => {}
            _ => {}
        }
    }
}
