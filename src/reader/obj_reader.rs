use crate::graphics::{self, scene::Scene, vertex::Vertex};
use crate::math::{
    matrix::Matrix,
    vector::{Vec2, Vec3},
};
use crate::reader::error::FileError;
use std::fs::{self, File};

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
        world_transform: Matrix::<f32>::default(4, 4),
    };
    for line in file.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue; // Skip comments and empty lines
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        match parts[0] {
            "v" => {
                // Vertex position
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid vertex position".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    ));
                }
                let x = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                let y = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                let z = parts[3].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                positions.push(Vec3 { x, y, z });
            }
            "vt" => {
                // Texture coordinates
                if parts.len() < 3 {
                    return Err(FileError::FormatError(
                        "Invalid texture coordinates".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    ));
                }
                let u = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                let v = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                tex_coords.push(Vec2 { x: u, y: v });
            }

            "vn" => {
                // Vertex normal
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid vertex normal".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    ));
                }
                let x = parts[1].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                let y = parts[2].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                let z = parts[3].parse::<f32>().map_err(|_| {
                    FileError::FormatError(
                        "Invalid float".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
                    )
                })?;
                normals.push(Vec3 { x, y, z });
            }

            "f" => {
                // Face definition
                if parts.len() < 4 {
                    return Err(FileError::FormatError(
                        "Invalid face definition".to_string(),
                        crate::reader::file_type::FileType::Obj,
                        file.lines().position(|l| l == line).unwrap_or(0),
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
                            crate::reader::file_type::FileType::Obj,
                            file.lines().position(|l| l == line).unwrap_or(0),
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
                    let tex_coord = if indices.len() > 1 && indices[1] < tex_coords.len() {
                        Some(tex_coords[indices[1]])
                    } else {
                        None
                    };

                    let vertex = Vertex {
                        position,
                        normal,
                        tex_coords: tex_coord,
                    };

                    // Insert vertex into the hash map if it doesn't already exist
                    if !hash_map.contains_key(&vertex) {
                        hash_map.insert(vertex.clone(), hash_map.len() as u32);
                        mesh.vertices.push(vertex);
                        mesh.indices.push(hash_map.len() as u32 - 1);
                    } else {
                        mesh.indices.push(*hash_map.get(&vertex).unwrap());
                    }
                }
            }
            // TODO: Handle material
            _ => {}
        }
    }

    scene.objects.push(mesh);

    Ok(scene)
}
