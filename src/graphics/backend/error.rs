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

    #[error("Image creation error: {0}")]
    ImageCreationError(String),

    #[error("Image view creation error: {0}")]
    ImageViewCreationError(String),

    #[error("Instance creation error: {0}")]
    InstanceCreationError(String),

    #[error("Presentation support error: {0}")]
    PresentationSupportError(String),

    #[error("Scene rendering error: {0}")]
    SceneError(String),
}
