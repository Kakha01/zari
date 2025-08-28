mod clip;
mod error;
mod resampler;
mod timeline;
mod track;

use clip::Clip;
use error::AudioError;
use resampler::Resampler;
use track::Track;
use track::TrackId;

const I8_SCALE: f32 = 1.0 / (i8::MAX as f32 + 1.0);
const I16_SCALE: f32 = 1.0 / (i16::MAX as f32 + 1.0);
const I24_SCALE: f32 = 1.0 / ((1 << 23) as f32 + 1.0);
const I32_SCALE: f32 = 1.0 / (i32::MAX as f32 + 1.0);

pub use timeline::Timeline;
