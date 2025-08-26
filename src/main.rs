use cpal::{
    self, SampleRate, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::path::Path;
use zari::engine::Timeline;

fn main() -> Result<(), anyhow::Error> {
    let sample_rate = 48000;
    let mut timeline = Timeline::new(sample_rate);

    let track_id_1 = timeline.new_track();

    timeline.add_clip(track_id_1, Path::new("sample-i16-stereo.wav"))?;

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();

    let stream = output_device
        .build_output_stream(
            &StreamConfig {
                buffer_size: cpal::BufferSize::Default,
                channels: 2,
                sample_rate: SampleRate(sample_rate),
            },
            move |data: &mut [f32], _| {
                let num_frames = data.len() / 2;

                let audio_block = timeline.process(num_frames);

                for (i, frame) in audio_block.iter().enumerate() {
                    data[i * 2] = frame[0]; // Left channel
                    data[i * 2 + 1] = frame[1]; // Right channel
                }
            },
            |err| eprintln!("{err:?}"),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
