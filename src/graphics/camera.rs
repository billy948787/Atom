use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32, // Field of view in degrees
    pub near_plane: f32,
    pub far_plane: f32,
    pub up: Vec3, // Up vector for the camera
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Camera {
    pub fn view_matrix(&self) -> glam::Mat4 {
        let forward = Vec3::new(
            self.rotation.y.cos() * self.rotation.x.sin(),
            -self.rotation.x.cos(),
            self.rotation.y.sin() * self.rotation.x.sin(),
        )
        .normalize();

        glam::Mat4::look_at_rh(self.position, self.position + forward, self.up)
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }

    pub fn direction_to_rotation(direction: Vec3) -> Vec3 {
        let dir = direction.normalize();

        let pitch = (-dir.y).acos();

        let yaw = dir.z.atan2(dir.x);

        Vec3::new(pitch, yaw, 0.0)
    }
}
