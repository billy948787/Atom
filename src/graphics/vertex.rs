#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vertex {
    pub position: crate::math::vector::Vec3,
    pub normal: crate::math::vector::Vec3,
    pub tex_coords: Option<crate::math::vector::Vec2>,
}
