use std::path::Path;

use crate::engine::{AudioError, I8_SCALE, I16_SCALE, I24_SCALE, I32_SCALE, Resampler};

pub struct Clip {
    data: Vec<f32>,
    channel: u16,
    start_time_in_samples: u64,
}

impl Clip {
    pub fn new(data: Vec<f32>, channel: u16, start_time_in_samples: u64) -> Self {
        Clip {
            data,
            channel,
            start_time_in_samples,
        }
    }

    pub fn duration_in_samples(&self) -> u64 {
        self.sample_count() as u64 / self.channel as u64
    }

    pub fn start_time_in_samples(&self) -> u64 {
        self.start_time_in_samples
    }

    pub fn end_time_in_samples(&self) -> u64 {
        self.start_time_in_samples + self.duration_in_samples()
    }

    pub fn is_mono(&self) -> bool {
        self.channel == 1
    }

    pub fn is_stereo(&self) -> bool {
        self.channel == 2
    }

    pub fn sample_count(&self) -> usize {
        self.data.len()
    }

    pub fn from_path<P: AsRef<Path>>(
        path: P,
        start_time_in_samples: u64,
        timeline_sample_rate: u32,
    ) -> Result<Self, AudioError> {
        let reader = hound::WavReader::open(path)?;
        let spec: hound::WavSpec = reader.spec();
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => match spec.bits_per_sample {
                8 => reader
                    .into_samples::<i8>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I8_SCALE))
                    .collect(),
                16 => reader
                    .into_samples::<i16>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I16_SCALE))
                    .collect(),
                24 => reader
                    .into_samples::<i32>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I24_SCALE))
                    .collect(),
                32 => reader
                    .into_samples::<i32>()
                    .map(|sample| sample.map_or(0.0, |s| s as f32 * I32_SCALE))
                    .collect(),
                other => return Err(AudioError::UnsupportedBitsPerSample(other)),
            },
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => reader
                    .into_samples::<f32>()
                    .map(|sample| sample.unwrap_or(0.0))
                    .collect(),
                other => return Err(AudioError::UnsupportedBitsPerSample(other)),
            },
        };

        let data = if spec.sample_rate != timeline_sample_rate {
            Resampler::resample(
                samples,
                timeline_sample_rate,
                spec.sample_rate,
                spec.channels,
            )?
        } else {
            samples
        };

        Ok(Clip {
            data,
            channel: spec.channels,
            start_time_in_samples,
        })
    }

    pub fn process_sample(
        &self,
        mix_buffer: &mut [f32],
        volume: f32,
        output_channels: u16,
        playhead_position: u64,
    ) {
        let frame_within_clip = playhead_position - self.start_time_in_samples;

        if self.is_mono()
            && let Some(sample) = self.data.get(frame_within_clip as usize)
        {
            for item in mix_buffer.iter_mut().take(output_channels as usize) {
                *item += sample * volume;
            }
        }

        if self.is_stereo()
            && let Some(idx) = frame_within_clip.checked_mul(2).map(|idx| idx as usize)
            && let (Some(left), Some(right)) = (self.data.get(idx), self.data.get(idx + 1))
        {
            let left = left * volume;
            let right = right * volume;

            if output_channels == 1 {
                mix_buffer[0] += left + right;
            } else {
                mix_buffer[0] += left;
                mix_buffer[1] += right;
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn
// }
