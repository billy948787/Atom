use crate::graphics;
use crate::math;
#[derive(Debug, Clone, Default)]
pub struct Scene {
    objects: Vec<graphics::mesh::Mesh>,
    cameras: Vec<graphics::camera::Camera>,
    lights: Vec<graphics::light::Light>,
    world_transform: math::matrix::Matrix<f32>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            world_transform: math::matrix::Matrix::default(4, 4),
        }
    }

    fn merge(&mut self, other: Scene) {
        self.objects.extend(other.objects);
        self.cameras.extend(other.cameras);
        self.lights.extend(other.lights);
    }
}
