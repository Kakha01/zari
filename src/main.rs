use cpal::{self, SampleFormat, SampleRate};
use zari::engine::AudioEngine;

fn main() -> Result<(), anyhow::Error> {
    let channels = 2;
    let sample_format = SampleFormat::I32;
    let sample_rate = SampleRate(44100);
    let mut audio_engine = AudioEngine::new(channels, sample_format, sample_rate)?;

    let timeline = audio_engine.timeline();

    if let Ok(mut timeline) = timeline.lock() {
        let track_id_1 = timeline.new_track();
        timeline.add_clip(track_id_1, "sample-i24-stereo.wav")?;
    }

    let duration = {
        let timeline = timeline.lock().unwrap();
        timeline.duration_in_seconds()
    };

    audio_engine.start_playing()?;
    std::thread::sleep(std::time::Duration::from_secs(duration.ceil() as u64));

    Ok(())
}
