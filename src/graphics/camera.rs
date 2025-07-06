use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32, // Field of view in degrees
    pub near_plane: f32,
    pub far_plane: f32,
    pub look_at: Vec3,
    pub up: Vec3, // Up vector for the camera
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Camera {
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.position, self.look_at, self.up)
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }
}
