use cpal::{
    self, BuildStreamError, SampleFormat, SampleRate,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use zari::engine::Timeline;

fn main() -> Result<(), anyhow::Error> {
    let channels = 2;
    let sample_format = SampleFormat::F32;
    let sample_rate = SampleRate(44100);
    let mut timeline = Timeline::new(sample_rate.0);

    let track_id_1 = timeline.new_track();

    timeline.add_clip(track_id_1, "sample-u8-stereo.wav")?;

    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .expect("Output device not found");

    let mut supported_output_configs = output_device.supported_output_configs()?;

    let supported_config = supported_output_configs.find(|c| {
        c.channels() == channels
            && c.sample_format() == sample_format
            && c.min_sample_rate() < sample_rate
            && c.max_sample_rate() > sample_rate
    });

    let config = supported_config
        .expect("Config is not supported")
        .with_sample_rate(sample_rate);

    let err = move |err| eprintln!("{err:?}");

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let config = config.config();
            output_device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let num_samples = data.len() / config.channels as usize;
                    let audio_block = timeline.process(num_samples, config.channels);

                    for (i, frame) in audio_block.iter().enumerate() {
                        for channel_idx in 0..config.channels as usize {
                            if let Some(sample) = frame.get(channel_idx) {
                                data[i * config.channels as usize + channel_idx] = *sample;
                            }
                        }
                    }
                },
                err,
                None,
            )
        }
        _ => Err(BuildStreamError::StreamConfigNotSupported),
    }?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
