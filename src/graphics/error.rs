use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Vulkan error: {0}")]
    VulkanError(#[from] crate::graphics::backend::error::VulkanError),
    
    #[error("Invalid texture format: {0}")]
    InvalidTextureFormat(String),

    #[error("Shader compilation error: {0}")]
    ShaderCompilationError(String),

    #[error("Mesh loading error: {0}")]
    MeshLoadingError(String),

    #[error("Material not found: {0}")]
    MaterialNotFound(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),

    #[error("Camera setup error: {0}")]
    CameraSetupError(String),

    #[error("No mesh data found")]
    NoMeshDataFound,

    #[error("No camera found")]
    NoCameraFound,
}
