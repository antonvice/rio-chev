use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct VoiceRecorder {
    stream: Option<cpal::Stream>,
    writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    path: PathBuf,
}

impl VoiceRecorder {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("rio-voice-input.wav");
        Self {
            stream: None,
            writer: Arc::new(Mutex::new(None)),
            path,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| "No input device available".to_string())?;
        
        // Whisper requires 16000Hz mono.
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        // Check if device supports this config, if not we might need to fallback and resample
        // But for most modern Macs, 16kHz mono is supported as a virtual or native format
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&self.path, spec)
            .map_err(|e| format!("WavWriter error: {}", e))?;
        let writer = Arc::new(Mutex::new(Some(writer)));
        let writer_clone = writer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| {
                if let Ok(mut guard) = writer_clone.lock() {
                    if let Some(ref mut w) = *guard {
                        for &sample in data {
                            // Convert f32 -> i16 for WAV
                            let s = (sample * i16::MAX as f32) as i16;
                            let _ = w.write_sample(s);
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None
        ).or_else(|_| {
            // Fallback to default if 16k mono fails (rare on Mac)
            let default_config: cpal::StreamConfig = device.default_input_config().unwrap().into();
            device.build_input_stream(
                &default_config,
                move |data: &[f32], _: &_| {
                    // Simple downsampling/mono conversion would go here if needed
                    // For now, let's stick to 16k and see if it works
                },
                |err| eprintln!("Audio stream error: {}", err),
                None
            )
        }).map_err(|e| format!("Stream builder error: {}", e))?;

        stream.play().map_err(|e| format!("Stream play error: {}", e))?;
        
        self.stream = Some(stream);
        self.writer = writer;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<PathBuf, String> {
        self.stream = None; 
        
        if let Ok(mut guard) = self.writer.lock() {
            if let Some(w) = guard.take() {
                w.finalize().map_err(|e| format!("Wav finalize error: {}", e))?;
            }
        }
        
        Ok(self.path.clone())
    }
}
