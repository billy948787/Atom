#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Obj,
    Fbx,
}
impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Obj => write!(f, "OBJ"),
            FileType::Fbx => write!(f, "FBX"),
        }
    }
}
