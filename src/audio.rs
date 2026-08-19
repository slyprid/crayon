use rodio::{source::SineWave, OutputStream, OutputStreamHandle, Sink, Source};
use std::time::Duration;

pub struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl Audio {
    pub fn new() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| format!("Audio init failed: {e}"))?;
        Ok(Self { _stream: stream, handle })
    }

    pub fn play_tone_hz(&self, hz: u32, ms: u64, volume: f32) -> Result<(), String> {
        let sink = Sink::try_new(&self.handle).map_err(|e| format!("Sink failed: {e}"))?;
        let src = SineWave::new(hz as f32)
            .take_duration(Duration::from_millis(ms))
            .amplify(volume);
        sink.append(src);
        sink.detach(); // fire-and-forget
        Ok(())
    }

    pub fn sound_to_hz(n: u8) -> u32 {
        let min_hz = 87.0f32;
        let max_hz = 13_000.0f32;
        let t = (n.saturating_sub(1)) as f32 / 254.0;
        (min_hz + t * (max_hz - min_hz)).round() as u32
    }

    pub fn sound_len_to_ms(len: u8) -> u64 {
        // 100 -> 6000 ms
        (len as u64) * 60
    }
}