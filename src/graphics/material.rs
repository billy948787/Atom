use glam::Vec3;

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub name: String,

    pub diffuse_color: Vec3,

    pub specular_color: Vec3,

    pub ambient_color: Vec3,
}
#[derive(Debug, Clone, Default, vulkano::buffer::BufferContents)]
#[repr(C)]
pub struct GpuMaterial {
    pub ambient_color: Vec3,
    pub diffuse_color: Vec3,
    pub specular_color: Vec3,
}

impl From<Material> for GpuMaterial {
    fn from(material: Material) -> Self {
        GpuMaterial {
            diffuse_color: material.diffuse_color,
            specular_color: material.specular_color,
            ambient_color: material.ambient_color,
        }
    }
}

impl From<&Material> for GpuMaterial {
    fn from(material: &Material) -> Self {
        GpuMaterial {
            diffuse_color: material.diffuse_color,
            specular_color: material.specular_color,
            ambient_color: material.ambient_color,
        }
    }
}
