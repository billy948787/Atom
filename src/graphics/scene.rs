use glam::Vec3;

use crate::graphics::{self, primitives};
#[derive(Debug, Clone)]
pub struct Scene {
    pub objects: Vec<graphics::mesh::Mesh>,
    pub cameras: Vec<graphics::camera::Camera>,
    pub lights: Vec<graphics::light::Light>,
    pub main_camera_index: usize,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            main_camera_index: 0,
        }
    }

    pub fn merge(&mut self, other: Scene) {
        self.objects.extend(other.objects);
        self.cameras.extend(other.cameras);
        self.lights.extend(other.lights);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let cameras = vec![graphics::camera::Camera::default()];
        let lights = vec![graphics::light::Light::default()];
        let default_object = primitives::create_sphere(Vec3::ZERO, 1.0, 32).mesh;
        Self {
            objects: vec![default_object],
            cameras,
            lights,
            main_camera_index: 0,
        }
    }
}
