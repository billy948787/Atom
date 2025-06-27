#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    vulkano::buffer::BufferContents,
    vulkano::pipeline::graphics::vertex_input::Vertex,
)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: crate::math::vector::Vec3,
    #[format(R32G32B32_SFLOAT)]
    pub normal: crate::math::vector::Vec3,
    #[format(R32G32_SFLOAT)]
    pub tex_coord: crate::math::vector::Vec2,
}
