use crate::engine::TrackId;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio file error: {0}")]
    FileError(#[from] hound::Error),

    #[error("Resampling failed: {0}")]
    ResampleError(#[from] rubato::ResampleError),

    #[error("Resample construction failed: {0}")]
    ResamplerConstructionError(#[from] rubato::ResamplerConstructionError),

    #[error("Track not found: {0}")]
    TrackNotFound(TrackId),

    #[error("Invalid volume: {0} (must be between 0.0 and 1.0)")]
    InvalidVolume(f32),

    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),

    #[error("Unsupported bits per sample: {0}")]
    UnsupportedBitsPerSample(u16),
}
