#[derive(Debug)]
pub enum EditorCommand {
    UpdateObjectTransform {
        object_index: usize,
        new_transform: glam::Mat4,
    },
}
