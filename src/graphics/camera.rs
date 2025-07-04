use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32, // Field of view in degrees
    pub near_plane: f32,
    pub far_plane: f32,
    pub look_at: Vec3,
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
