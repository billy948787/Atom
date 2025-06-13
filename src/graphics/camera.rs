#[derive(Debug, Clone)]
pub struct Camera {
    position: [f32; 3],
    rotation: [f32; 3],
    fov: f32, // Field of view in degrees
    near_plane: f32,
    far_plane: f32,
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
