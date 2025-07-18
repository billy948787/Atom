use crate::graphics::{
    material::Material,
    mesh::{Mesh, SubMesh},
    vertex::Vertex,
};

pub struct Sphere {
    pub mesh: Mesh,
    pub position: glam::Vec3,
    pub radius: f32,
}

pub fn create_sphere(position: glam::Vec3, radius: f32, segments: u32) -> Sphere {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::new();
    let vertical_segments = segments;
    let horizontal_segments = segments;

    // Generate vertices
    for i in 0..=vertical_segments {
        let v = i as f32 / vertical_segments as f32;
        let theta = v * std::f32::consts::PI; // Vertical angle

        for j in 0..=horizontal_segments {
            let u = j as f32 / horizontal_segments as f32;
            let phi = u * 2.0 * std::f32::consts::PI; // Horizontal angle

            let x = radius * theta.sin() * phi.cos();
            let y = radius * theta.cos();
            let z = radius * theta.sin() * phi.sin();

            let pos = glam::Vec3::new(x, y, z);
            vertices.push(Vertex {
                position: pos + position,
                normal: pos.normalize(),
                tex_coord: glam::Vec2::new(u, v),
            });
        }
    }

    // Generate indices
    for i in 0..vertical_segments {
        for j in 0..horizontal_segments {
            let first = (i * (horizontal_segments + 1)) + j;
            let second = first + horizontal_segments + 1;

            indices.push(first as u32);
            indices.push(second as u32);
            indices.push(first as u32 + 1);

            indices.push(second as u32);
            indices.push(second as u32 + 1);
            indices.push(first as u32 + 1);
        }
    }

    let submesh = SubMesh {
        vertices,
        indices,
        material: Material::default(),
    };

    Sphere {
        mesh: Mesh {
            submeshes: vec![submesh],
            world_transform: glam::Mat4::IDENTITY,
        },
        position,
        radius,
    }
}
