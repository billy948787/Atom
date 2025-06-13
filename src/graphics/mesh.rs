#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
