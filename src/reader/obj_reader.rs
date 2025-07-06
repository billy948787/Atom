use glam::{Vec2, Vec3};

use crate::graphics::{self, vertex::Vertex};
use crate::reader::error::FileError;
use std::fs::{self};

pub fn read_file(path: &str) -> Result<graphics::scene::Scene, FileError> {
    // Read the OBJ file and populate the Scene

    let file = fs::read_to_string(path).map_err(|e| FileError::IoError(e))?;

    Ok(parse_file(&file)?)
}

fn parse_file(file: &str) -> Result<graphics::scene::Scene, FileError> {
    // Parse the file content and populate the Scene
    let mut scene = graphics::scene::Scene::new();

    let mut hash_map = std::collections::HashMap::<graphics::vertex::Vertex, u32>::new();

    // Temporary vectors to hold positions, normals, and texture coordinates
    let mut positions = std::vec::Vec::<Vec3>::new();
    let mut normals = std::vec::Vec::<Vec3>::new();
    let mut tex_coords = std::vec::Vec::<Vec2>::new();

    let mut mesh = graphics::mesh::Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        world_transform: glam::Mat4::IDENTITY,
    };
    for line in file.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue; // Skip comments and empty lines
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        let line_number = file.lines().position(|l| l == line).unwrap_or(0) + 1;

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
                for part in &parts[1..] {
                    let indices: Vec<usize> = part
                        .split('/')
                        .map(|s| s.parse::<usize>().unwrap_or(0) - 1)
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
                        let index = mesh.vertices.len() as u32;
                        mesh.vertices.push(vertex);
                        index
                    });

                    mesh.indices.push(*index);
                }
            }
            // TODO: Handle material
            _ => {}
        }
    }

    println!("vertices: {:?}", mesh.vertices);

    mesh.normalize();

    println!("normalized vertices: {:?}", mesh.vertices);

    scene.objects.push(mesh);

    // check if camera exists
    // if no camera exists, create a default camera
    if scene.cameras.is_empty() {
        scene.cameras.push(graphics::camera::Camera {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            fov: 90.0,
            near_plane: 0.1,
            up: Vec3::new(0.0, 1.0, 0.0),
            far_plane: 100.0,
        });
    }

    Ok(scene)
}
