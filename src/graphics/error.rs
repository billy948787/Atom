use thiserror::Error;
#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Vulkan initialization failed: {0}")]
    InitializationError(String),

    #[error("Vulkan device creation failed: {0}")]
    DeviceCreationError(String),

    #[error("Vulkan command buffer error: {0}")]
    CommandBufferError(String),

    #[error("Vulkan swapchain error: {0}")]
    SwapchainError(String),

    #[error("Vulkan memory allocation error: {0}")]
    MemoryAllocationError(String),

    #[error("Vulkan synchronization error: {0}")]
    SynchronizationError(String),

    #[error("Vulkan surface creation error: {0}")]
    SurfaceCreationError(String),

    #[error("Shader compilation error: {0}")]
    ShaderCompilationError(String),

    #[error("Vulkan pipeline layout error: {0}")]
    PipelineLayoutError(String),

    #[error("Vulkan pipeline creation error: {0}")]
    PipelineCreationError(String),

    #[error("Buffer creation error: {0}")]
    BufferCreationError(String),
}
#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Vulkan error: {0}")]
    VulkanError(#[from] VulkanError),

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
}
