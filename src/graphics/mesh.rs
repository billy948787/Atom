#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<crate::graphics::vertex::Vertex>,
    pub indices: Vec<u32>,
    pub world_transform: crate::math::matrix::Matrix<f32>,
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

pub struct SubMesh {}
    