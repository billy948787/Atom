#[derive(
    Debug,
    Clone,
    PartialEq,
    Copy,
    vulkano::buffer::BufferContents,
    vulkano::pipeline::graphics::vertex_input::Vertex,
)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: glam::Vec3,
    #[format(R32G32B32_SFLOAT)]
    pub normal: glam::Vec3,
    #[format(R32G32_SFLOAT)]
    pub tex_coord: glam::Vec2,
}

impl std::cmp::Eq for Vertex {}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);
        self.normal.x.to_bits().hash(state);
        self.normal.y.to_bits().hash(state);
        self.normal.z.to_bits().hash(state);
        self.tex_coord.x.to_bits().hash(state);
        self.tex_coord.y.to_bits().hash(state);
    }
}
