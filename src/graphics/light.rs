enum LightType {
    Point,
    Directional,
    Spot,
}

#[derive(Debug, Clone)]
pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl std::fmt::Display for Light {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
