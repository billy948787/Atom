pub mod error;
pub mod obj_reader;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Obj,
    Fbx,
    Mtl,
}
impl Default for FileType {
    fn default() -> Self {
        FileType::Obj
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Obj => write!(f, "OBJ"),
            FileType::Fbx => write!(f, "FBX"),
            FileType::Mtl => write!(f, "MTL"),
        }
    }
}
