use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<crate::graphics::vertex::Vertex>,
    pub indices: Vec<u32>,
    pub world_transform: glam::Mat4,
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Mesh {
    pub fn normalize(&mut self) {
        let mut min_point = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_point = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for vertex in &self.vertices {
            min_point = min_point.min(vertex.position);
            max_point = max_point.max(vertex.position);
        }

        let bbox_size = max_point - min_point;
        let center = (min_point + max_point) / 2.0;
        let max_side_length = bbox_size.x.max(bbox_size.y).max(bbox_size.z);

        if max_side_length == 0.0 {
            // Avoid division by zero if all vertices are at the same point
            return;
        }

        for vertex in &mut self.vertices {
            vertex.position = (vertex.position - center) / max_side_length;
        }
    }
}

pub struct SubMesh {}
