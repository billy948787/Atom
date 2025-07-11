use glam::Vec3;

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub name: String,
    pub diffuse_color: Vec3,
    pub specular_color: Vec3,
    pub ambient_color: Vec3,
}
