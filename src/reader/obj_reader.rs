use glam::{Vec2, Vec3};

use crate::graphics::material::Material;
use crate::graphics::{self, vertex::Vertex};
use crate::reader::error::FileError;
use std::collections::HashMap;
use std::fs::{self};

pub fn read_file(path: &str) -> Result<graphics::scene::Scene, FileError> {
    // Read the OBJ file and populate the Scene

    let file = fs::read_to_string(path).map_err(|e| FileError::IoError(e))?;

    Ok(parse_file(path, &file)?)
}
fn parse_file(path: &str, file: &str) -> Result<graphics::scene::Scene, FileError> {
    // Parse the file content and populate the Scene
    let mut scene = graphics::scene::Scene::new();

    let mut hash_map = std::collections::HashMap::<graphics::vertex::Vertex, u32>::new();

    // Temporary vectors to hold positions, normals, and texture coordinates
    let mut positions = std::vec::Vec::<Vec3>::new();
    let mut normals = std::vec::Vec::<Vec3>::new();
    let mut tex_coords = std::vec::Vec::<Vec2>::new();

    let mut mesh = graphics::mesh::Mesh {
        submeshes: Vec::new(),
        world_transform: glam::Mat4::IDENTITY,
    };

    let mut material_map: HashMap<String, graphics::material::Material> = HashMap::new();
    for (line_number, line) in file.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue; // Skip comments and empty lines
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        let line_number = line_number + 1;

        match parts[0] {
            "v" => {
                // Vertex position
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid vertex position".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }
                let x = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                let y = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                let z = parts[3].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                positions.push(Vec3 { x, y, z });
            }
            "vt" => {
                // Texture coordinates
                if parts.len() < 3 {
                    return Err(FileError::FormatError(
                        "Invalid texture coordinates".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }
                let u = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                let v = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                tex_coords.push(Vec2 { x: u, y: v });
            }

            "vn" => {
                // Vertex normal
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid vertex normal".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }
                let x = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                let y = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                let z = parts[3].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    )
                })?;
                normals.push(Vec3 { x, y, z });
            }

            "f" => {
                // Face definition
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid face definition".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }

                let mut face_vertex_indices: Vec<u32> = Vec::new();
                // get submesh from previous created from usemtl

                // get last submesh
                let last_submesh = match mesh.submeshes.last_mut() {
                    Some(submesh) => submesh,
                    None => {
                        mesh.submeshes.push(graphics::mesh::SubMesh {
                            vertices: Vec::new(),
                            indices: Vec::new(),
                            material: Material::default(),
                        });
                        mesh.submeshes.last_mut().unwrap()
                    }
                };

                for part in &parts[1..] {
                    let indices: Vec<usize> = part
                        .split('/')
                        .map(|s| s.parse::<usize>().unwrap_or(1) - 1)
                        .collect();

                    if indices.len() < 1 || indices[0] >= positions.len() {
                        return Err(FileError::FormatError(
                            "Invalid vertex index".to_string(),
                            crate::reader::FileType::Obj,
                            line_number,
                        ));
                    }

                    let position = positions[indices[0]];
                    let normal = if indices.len() > 2 && indices[2] < normals.len() {
                        normals[indices[2]]
                    } else {
                        Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }
                    };

                    let tex_coord = indices
                        .get(1)
                        .and_then(|&i| tex_coords.get(i))
                        .cloned()
                        .unwrap_or(Vec2 { x: 0.0, y: 0.0 });

                    let vertex = Vertex {
                        position,
                        normal,
                        tex_coord,
                    };

                    let index = hash_map.entry(vertex).or_insert_with(|| {
                        let index = last_submesh.vertices.len() as u32;
                        last_submesh.vertices.push(vertex);
                        index
                    });

                    face_vertex_indices.push(*index);
                }
                let first_index = face_vertex_indices[0];
                for i in 1..(face_vertex_indices.len() - 1) {
                    last_submesh.indices.push(first_index);
                    last_submesh.indices.push(face_vertex_indices[i]);
                    last_submesh.indices.push(face_vertex_indices[i + 1]);
                }
            }
            "mtllib" => {
                if parts.len() < 2 {
                    return Err(FileError::FormatError(
                        "Invalid material library".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }
                // create path to mtl file

                let mtl_path = format! {
                    "{}/{}",
                    path.rfind('/')
                        .map_or("", |i| &path[..i]),
                    parts[1]
                };
                println!("Loading MTL file: {}", mtl_path);
                match parse_mtl_file(&mtl_path) {
                    Ok(materials) => {
                        material_map.extend(materials);
                    }
                    Err(_) => {
                        println!("Failed to load MTL file: {}", mtl_path);
                    }
                }
            }
            "usemtl" => {
                if parts.len() < 2 {
                    return Err(FileError::FormatError(
                        "Invalid material name".to_string(),
                        crate::reader::FileType::Obj,
                        line_number,
                    ));
                }
                let material_name = parts[1].to_string();

                // Check if the material exists in the map
                if let Some(material) = material_map.get(&material_name) {
                    // Create a submesh with the current vertices and indices
                    mesh.submeshes.push(graphics::mesh::SubMesh {
                        vertices: Vec::new(),
                        indices: Vec::new(),
                        material: material.clone(),
                    });
                } else {
                    // Material not found, create a default submesh
                    mesh.submeshes.push(graphics::mesh::SubMesh {
                        vertices: Vec::new(),
                        indices: Vec::new(),
                        material: Material::default(),
                    });
                }
            }
            _ => {}
        }
    }

    mesh.normalize();

    scene.objects.push(mesh);

    // check if camera exists
    // if no camera exists, create a default camera
    if scene.cameras.is_empty() {
        scene.cameras.push(graphics::camera::Camera {
            position: Vec3::new(0.0, 1.0, 5.0),
            target: Vec3::new(0.0, 0.0, 0.0), // Look at the origin
            up: Vec3::Y,                      // Set the up vector to Y axis
            fov: 30.0,
            near_plane: 0.1,
            far_plane: 100.0,
        });
    }

    // let camera look at object center
    if let Some(camera) = scene.cameras.first_mut() {
        // Get the first object immutably, as we only need to read vertex data.
        if let Some(first_object) = scene.objects.first() {
            let all_vertices: Vec<Vec3> = first_object
                .submeshes
                .iter()
                .flat_map(|submesh| submesh.vertices.iter().map(|v| v.position))
                .collect();

            if !all_vertices.is_empty() {
                let total_vertices = all_vertices.len() as f32;
                let sum_of_positions = all_vertices.into_iter().fold(Vec3::ZERO, |acc, v| acc + v);
                let center = sum_of_positions / total_vertices;

                // Position the camera slightly away from the center and make it look at the center.
                camera.position = center + Vec3::new(0.0, 0.0, 5.0);
                camera.target = center; // The target is the center of the object.
            }
        }
    }

    if scene.lights.is_empty() {}

    Ok(scene)
}

fn parse_mtl_file(path: &str) -> Result<HashMap<String, graphics::material::Material>, FileError> {
    let file = fs::read_to_string(path).map_err(|e| FileError::IoError(e))?;
    let mut current_material: Option<graphics::material::Material> = Option::None;
    let mut materials = HashMap::new();

    for (line_index, line) in file.lines().enumerate() {
        let line_number = line_index + 1;
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }

        match parts[0] {
            "newmtl" => {
                // Start a new material
                if let Some(material) = current_material.take() {
                    materials.insert(material.name.clone(), material);
                }
                if parts.len() < 2 {
                    return Err(FileError::FormatError(
                        "Invalid material name".to_string(),
                        crate::reader::FileType::Mtl,
                        line_number,
                    ));
                }
                current_material = Some(graphics::material::Material {
                    name: parts[1].to_string(),
                    properties: graphics::material::MaterialProperties::default(),
                });
            }

            "Ka" => {
                // Ambient color
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid ambient color".to_string(),
                        crate::reader::FileType::Mtl,
                        line_number,
                    ));
                }
                if let Some(material) = &mut current_material {
                    material.properties.ambient_color = Vec3 {
                        x: parts[1].parse().unwrap_or(0.0),
                        y: parts[2].parse().unwrap_or(0.0),
                        z: parts[3].parse().unwrap_or(0.0),
                    };
                }
            }

            "Kd" => {
                // Diffuse color
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid diffuse color".to_string(),
                        crate::reader::FileType::Mtl,
                        line_number,
                    ));
                }
                if let Some(material) = &mut current_material {
                    material.properties.diffuse_color = Vec3 {
                        x: parts[1].parse().unwrap_or(0.0),
                        y: parts[2].parse().unwrap_or(0.0),
                        z: parts[3].parse().unwrap_or(0.0),
                    };
                }
            }

            "Ks" => {
                // Specular color
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid specular color".to_string(),
                        crate::reader::FileType::Mtl,
                        line_number,
                    ));
                }
                if let Some(material) = &mut current_material {
                    material.properties.specular_color = Vec3 {
                        x: parts[1].parse().unwrap_or(0.0),
                        y: parts[2].parse().unwrap_or(0.0),
                        z: parts[3].parse().unwrap_or(0.0),
                    };
                }
            }

            _ => {}
        }
    }
    // Insert the last material if it exists
    if let Some(material) = current_material {
        materials.insert(material.name.clone(), material);
    }
    println!("materials: {:#?}", materials);
    Ok(materials)
}
