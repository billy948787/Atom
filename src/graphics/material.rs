use glam::Vec3;

#[derive(Debug, Clone, vulkano::buffer::BufferContents)]
#[repr(C)]
pub struct MaterialProperties {
    pub ambient_color: Vec3,
    _padding1: u32, // Padding to ensure proper alignment
    pub diffuse_color: Vec3,
    _padding2: u32, // Padding to ensure proper alignment
    pub specular_color: Vec3,
    _padding3: u32, // Padding to ensure proper alignment
}

impl Default for MaterialProperties {
    fn default() -> Self {
        MaterialProperties {
            ambient_color: Vec3::new(0.2, 0.2, 0.2),
            diffuse_color: Vec3::new(1.0, 1.0, 1.0),
            specular_color: Vec3::new(1.0, 1.0, 1.0),
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,

    pub properties: MaterialProperties,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            name: "Default Material".to_string(),
            properties: MaterialProperties::default(),
        }
    }
}
