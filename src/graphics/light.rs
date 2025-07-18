enum LightType {
    Point,
    Directional,
    Spot,
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub position: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub enum Light {
    Point(PointLight),
    Directional(DirectionalLight),
}

impl Default for Light {
    fn default() -> Self {
        Light::Point(PointLight {
            position: glam::Vec3::ZERO,
            color: glam::Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        })
    }
}

#[derive(vulkano::buffer::BufferContents, Debug)]
#[repr(C)]
pub struct GpuLight {
    pub position: glam::Vec3,
    pub _padding: f32,
    pub direction: glam::Vec3,
    pub _padding2: f32,
    pub color: glam::Vec3,
    pub intensity: f32,
    pub light_type: u32,
    pub _padding4: [u32; 3],
}

impl Default for GpuLight {
    fn default() -> Self {
        GpuLight {
            position: glam::Vec3::ZERO,
            _padding: 0.0,
            direction: glam::Vec3::ZERO,
            _padding2: 0.0,
            color: glam::Vec3::ZERO,
            intensity: 1.0,
            light_type: 0, // Default to Point light
            _padding4: [0; 3],
        }
    }
}

impl From<&Light> for GpuLight {
    fn from(light: &Light) -> Self {
        match light {
            Light::Point(point_light) => GpuLight {
                position: point_light.position,

                direction: glam::Vec3::ZERO,
                color: point_light.color,
                intensity: point_light.intensity,
                light_type: 0, // Point light type
                ..Default::default()
            },
            Light::Directional(directional_light) => GpuLight {
                position: glam::Vec3::ZERO,

                direction: directional_light.direction,

                color: directional_light.color,
                intensity: directional_light.intensity,
                light_type: 1, // Directional light type
                ..Default::default()
            },
        }
    }
}

impl From<Light> for GpuLight {
    fn from(light: Light) -> Self {
        match light {
            Light::Point(point_light) => GpuLight {
                position: point_light.position,

                direction: glam::Vec3::ZERO,

                color: point_light.color,
                intensity: point_light.intensity,
                light_type: 0, // Point light type
                ..Default::default()
            },
            Light::Directional(directional_light) => GpuLight {
                position: glam::Vec3::ZERO,

                direction: directional_light.direction,

                color: directional_light.color,
                intensity: directional_light.intensity,
                light_type: 1, // Directional light type
                ..Default::default()
            },
        }
    }
}
