use glam::Vec3;

#[derive(Debug, Clone, vulkano::buffer::BufferContents)]
#[repr(C)]
pub struct MaterialProperties {
    pub ambient_color: Vec3,
    pub diffuse_color: Vec3,
    pub specular_color: Vec3,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        MaterialProperties {
            ambient_color: Vec3::new(0.2, 0.2, 0.2),
            diffuse_color: Vec3::new(1.0, 1.0, 1.0),
            specular_color: Vec3::new(1.0, 1.0, 1.0),
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

#[derive(vulkano::buffer::BufferContents)]
#[repr(C)]
pub struct GpuMaterial {
    properties: MaterialProperties,
}

impl From<Material> for GpuMaterial {
    fn from(material: Material) -> Self {
        GpuMaterial {
            properties: material.properties,
        }
    }
}

impl From<&Material> for GpuMaterial {
    fn from(material: &Material) -> Self {
        GpuMaterial {
            properties: material.properties.clone(),
        }
    }
}
