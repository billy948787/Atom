#[derive(Debug, Clone)]
pub struct Camera {
  pub position: [f32; 3],
  pub rotation: [f32; 3],
  pub fov: f32,      // Field of view in degrees
  pub near_plane: f32,
  pub far_plane: f32,
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
