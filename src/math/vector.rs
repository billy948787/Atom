use bytemuck::{Pod, Zeroable};
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl std::cmp::PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl std::cmp::Eq for Vec3 {}

impl std::cmp::PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl std::cmp::Eq for Vec2 {}

impl std::hash::Hash for Vec3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl std::hash::Hash for Vec2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}
