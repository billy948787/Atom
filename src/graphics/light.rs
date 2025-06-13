#[derive(Debug, Clone)]
pub struct Light {
    position: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}

impl std::fmt::Display for Light {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
