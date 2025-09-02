use std::{fs::File, io::BufReader, ops::AddAssign, path::Path};

use hound::WavReader;

use crate::engine::{AudioError, FromF64Sample, Resampler, Scale, utils::Utils};

#[derive(Debug, Clone)]
pub struct Clip {
    data: Vec<f64>,
    channel: u16,
    start_time_in_samples: u64,
}

impl Clip {
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        start_time_in_samples: u64,
        timeline_sample_rate: u32,
    ) -> Result<Self, AudioError> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        let samples = Self::decode_samples_to_f64(reader)?;

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

    fn read_samples_as_f64<T>(reader: WavReader<BufReader<File>>, scale: Scale) -> Vec<f64>
    where
        T: hound::Sample + Into<f64> + Default + Copy,
    {
        let samples: Vec<T> = reader
            .into_samples::<T>()
            .map(|s| s.unwrap_or_default())
            .collect();

        Utils::convert_samples_to_f64::<T>(&samples, scale)
    }

    fn decode_samples_to_f64(reader: WavReader<BufReader<File>>) -> Result<Vec<f64>, AudioError> {
        let spec = reader.spec();

        let samples: Vec<f64> = match spec.sample_format {
            hound::SampleFormat::Int => match spec.bits_per_sample {
                8 => Self::read_samples_as_f64::<i8>(reader, Scale::I8),
                16 => Self::read_samples_as_f64::<i16>(reader, Scale::I16),
                24 => Self::read_samples_as_f64::<i32>(reader, Scale::I24),
                32 => Self::read_samples_as_f64::<i32>(reader, Scale::I32),
                other => return Err(AudioError::UnsupportedBitsPerSample(other)),
            },
            hound::SampleFormat::Float => match spec.bits_per_sample {
                32 => Self::read_samples_as_f64::<f32>(reader, Scale::F32),
                other => return Err(AudioError::UnsupportedBitsPerSample(other)),
            },
        };

        Ok(samples)
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

    fn write_to_frame<T>(
        buffer: &mut [T],
        frame_index: usize,
        channel: usize,
        sample: T,
        output_channels: u16,
    ) where
        T: AddAssign + Clone,
    {
        let position = frame_index * output_channels as usize + channel;
        if let Some(s) = buffer.get_mut(position) {
            *s += sample;
        };
    }

    pub fn process_sample<T>(
        &self,
        buffer: &mut [T],
        volume: f32,
        output_channels: u16,
        playhead_position: u64,
        frame_index: usize,
    ) where
        T: FromF64Sample + AddAssign + Clone,
    {
        let frame_within_clip = playhead_position - self.start_time_in_samples;

        if self.is_mono()
            && let Some(sample) = self.data.get(frame_within_clip as usize)
        {
            let sample_t = T::from_f64_sample(sample * volume as f64);
            for channel in 0..output_channels as usize {
                Self::write_to_frame(
                    buffer,
                    frame_index,
                    channel,
                    sample_t.clone(),
                    output_channels,
                );
            }
        }

        if self.is_stereo()
            && let Some(idx) = frame_within_clip.checked_mul(2).map(|idx| idx as usize)
            && let (Some(left), Some(right)) = (self.data.get(idx), self.data.get(idx + 1))
        {
            let left_t = T::from_f64_sample(left * volume as f64);
            let right_t = T::from_f64_sample(right * volume as f64);

            if output_channels == 1 {
                let mono_sample = T::from_f64_sample((left + right) * volume as f64);
                Self::write_to_frame(buffer, frame_index, 0, mono_sample, output_channels);
            } else {
                Self::write_to_frame(buffer, frame_index, 0, left_t, output_channels);
                Self::write_to_frame(buffer, frame_index, 1, right_t, output_channels);
            }
        }
    }
}
