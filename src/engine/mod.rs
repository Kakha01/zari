mod clip;
mod error;
mod resampler;
mod timeline;
mod track;
mod utils;

use clip::Clip;
use error::AudioError;
use resampler::Resampler;
use track::Track;
use track::TrackId;

#[derive(Clone, Copy)]
enum Scale {
    I8,
    I16,
    I24,
    I32,
    F32,
}

impl Scale {
    const I8_SCALE: f64 = 1.0 / i8::MAX as f64;
    const I16_SCALE: f64 = 1.0 / i16::MAX as f64;
    const I24_SCALE: f64 = 1.0 / ((1 << 23) - 1) as f64;
    const I32_SCALE: f64 = 1.0 / i32::MAX as f64;

    fn get_f64_scale(&self) -> f64 {
        match self {
            Scale::I8 => Self::I8_SCALE,
            Scale::I16 => Self::I16_SCALE,
            Scale::I24 => Self::I24_SCALE,
            Scale::I32 => Self::I32_SCALE,
            Scale::F32 => 1.0,
        }
    }
}

pub use timeline::Timeline;
