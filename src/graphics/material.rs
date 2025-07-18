use glam::Vec3;

#[derive(Debug, Clone, vulkano::buffer::BufferContents)]
#[repr(C)]
pub struct GpuMaterials {
    pub ambient_color: Vec3,
    _padding1: u32, // Padding to ensure proper alignment
    pub diffuse_color: Vec3,
    _padding2: u32, // Padding to ensure proper alignment
    pub specular_color: Vec3,
    pub specular_exponent: f32, // Specular exponent for shininess
}

impl Default for GpuMaterials {
    fn default() -> Self {
        GpuMaterials {
            ambient_color: Vec3::new(0.2, 0.2, 0.2),
            diffuse_color: Vec3::new(1.0, 1.0, 1.0),
            specular_color: Vec3::new(1.0, 1.0, 1.0),
            specular_exponent: 32.0, // Default specular exponent
            _padding1: 0,
            _padding2: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,

    pub properties: GpuMaterials,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            name: "Default Material".to_string(),
            properties: GpuMaterials::default(),
        }
    }
}
