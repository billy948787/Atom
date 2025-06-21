use std::{
    collections::{HashMap, hash_map},
    sync::Arc,
};

use vulkano::instance::{InstanceCreateFlags, InstanceCreateInfo};
use winit::{
    application::ApplicationHandler,
    window::{self, WindowAttributes},
};

pub struct App {
    instance: Arc<vulkano::instance::Instance>,
    render_context: HashMap<winit::window::WindowId, crate::graphics::rendering::RenderContext>,
    // queue: Arc<vulkano::device::Queue>,
    // device: Arc<vulkano::device::Device>,
}

impl App {
    pub fn new() -> Self {
        let library = vulkano::library::VulkanLibrary::new().unwrap();
        let instance = vulkano::instance::Instance::new(
            library,
            InstanceCreateInfo {
                application_name: Some("Atom Engine".to_string()),
                // max_api_version: Some(vulkano::Version::V1_0),
                enabled_extensions: vulkano::instance::InstanceExtensions{
                    khr_portability_enumeration: true,
                    ..Default::default()
                },
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .unwrap();
        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::empty()
        };

        let devices = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices");

        let suitable_device = devices
            .into_iter()
            .min_by_key(|device| match device.properties().device_type {
                vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 0,
                vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 1,
                vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 2,
                vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
                vulkano::device::physical::PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("No suitable device found");

        println!("Using device: {}", suitable_device.properties().device_name);

        // let virtual_device = vulkano::device::Device::new(
        //     suitable_device,
        //     vulkano::device::DeviceCreateInfo{
                
        //     }
        // );



        return App {
            instance,
            render_context: HashMap::new(),
        };
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
