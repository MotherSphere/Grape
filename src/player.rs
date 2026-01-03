#![allow(dead_code)]

use std::fmt;
use std::fs::File;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::{Decoder, OutputStream, Sink, source::Source};
use tracing::{error, info};

mod audio_options {
    use rodio::{OutputStream, OutputStreamBuilder, cpal};
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    use tracing::{info, warn};

    use crate::config::{AudioOutputDevice, UserSettings};
    use crate::player::PlayerError;

    #[derive(Debug, Clone)]
    pub struct AudioOptions {
        pub output_device: AudioOutputDevice,
        pub sample_rate_hz: Option<u32>,
    }

    impl AudioOptions {
        pub fn from_settings(settings: &UserSettings) -> Self {
            Self {
                output_device: settings.output_device,
                sample_rate_hz: settings.output_sample_rate_hz,
            }
        }

        pub fn open_stream(&self) -> Result<OutputStream, PlayerError> {
            let builder = self.builder()?;
            match builder.open_stream() {
                Ok(stream) => Ok(stream),
                Err(err) => {
                    if self.is_default() {
                        Err(err.into())
                    } else {
                        warn!(error = %err, "Failed to open stream with custom options, falling back");
                        OutputStreamBuilder::open_default_stream().map_err(PlayerError::from)
                    }
                }
            }
        }

        fn builder(&self) -> Result<OutputStreamBuilder, PlayerError> {
            let builder = if let Some(device) = self.resolve_device()? {
                OutputStreamBuilder::from_device(device)?
            } else {
                OutputStreamBuilder::from_default_device()?
            };
            let builder = if let Some(sample_rate) = self.sample_rate_hz {
                builder.with_sample_rate(sample_rate)
            } else {
                builder
            };
            Ok(builder)
        }

        fn resolve_device(&self) -> Result<Option<cpal::Device>, PlayerError> {
            match self.output_device {
                AudioOutputDevice::System => Ok(None),
                AudioOutputDevice::UsbHeadset => {
                    let host = cpal::default_host();
                    let devices = host.output_devices().map_err(PlayerError::from)?;
                    for device in devices {
                        let name = device.name().unwrap_or_default();
                        let lowered = name.to_lowercase();
                        if lowered.contains("usb") || lowered.contains("headset") {
                            info!(device = %name, "Selected USB headset output device");
                            return Ok(Some(device));
                        }
                    }
                    warn!("USB headset output device not found, using default");
                    Ok(None)
                }
            }
        }

        fn is_default(&self) -> bool {
            self.output_device == AudioOutputDevice::System && self.sample_rate_hz.is_none()
        }
    }

    impl Default for AudioOptions {
        fn default() -> Self {
            Self {
                output_device: AudioOutputDevice::System,
                sample_rate_hz: None,
            }
        }
    }
}

pub use audio_options::AudioOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub duration_secs: u32,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub now_playing: Option<NowPlaying>,
    pub is_playing: bool,
    pub position_secs: u32,
}

impl PlayerState {
    pub fn placeholder() -> Self {
        Self {
            now_playing: None,
            is_playing: false,
            position_secs: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Paused,
    Playing,
}

#[derive(Debug)]
pub enum PlayerError {
    Io(io::Error),
    DecoderError(rodio::decoder::DecoderError),
    StreamError(rodio::StreamError),
    PlayError(rodio::PlayError),
    DeviceError(rodio::cpal::DevicesError),
    NoTrackLoaded,
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::Io(err) => write!(f, "io error: {err}"),
            PlayerError::DecoderError(err) => write!(f, "decoder error: {err}"),
            PlayerError::StreamError(err) => write!(f, "stream error: {err}"),
            PlayerError::PlayError(err) => write!(f, "play error: {err}"),
            PlayerError::DeviceError(err) => write!(f, "device error: {err}"),
            PlayerError::NoTrackLoaded => write!(f, "no track loaded"),
        }
    }
}

impl std::error::Error for PlayerError {}

impl From<io::Error> for PlayerError {
    fn from(err: io::Error) -> Self {
        PlayerError::Io(err)
    }
}

impl From<rodio::decoder::DecoderError> for PlayerError {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        PlayerError::DecoderError(err)
    }
}

impl From<rodio::StreamError> for PlayerError {
    fn from(err: rodio::StreamError) -> Self {
        PlayerError::StreamError(err)
    }
}

impl From<rodio::PlayError> for PlayerError {
    fn from(err: rodio::PlayError) -> Self {
        PlayerError::PlayError(err)
    }
}

impl From<rodio::cpal::DevicesError> for PlayerError {
    fn from(err: rodio::cpal::DevicesError) -> Self {
        PlayerError::DeviceError(err)
    }
}

pub struct Player {
    stream: OutputStream,
    sink: Sink,
    state: PlaybackState,
    current_track: Option<PathBuf>,
    position: Duration,
}

impl Player {
    pub fn new() -> Result<Self, PlayerError> {
        Self::new_with_options(AudioOptions::default())
    }

    pub fn new_with_options(options: AudioOptions) -> Result<Self, PlayerError> {
        let stream = options.open_stream()?;
        let sink = Sink::connect_new(stream.mixer());
        Ok(Self {
            stream,
            sink,
            state: PlaybackState::Stopped,
            current_track: None,
            position: Duration::ZERO,
        })
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), PlayerError> {
        let path = path.as_ref().to_path_buf();
        info!(path = %path.display(), "Loading track");
        self.current_track = Some(path.clone());
        self.position = Duration::ZERO;
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream.mixer());
        let source = self.decode_source(&path).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to load track");
            err
        })?;
        self.sink.append(source);
        self.sink.pause();
        self.state = PlaybackState::Paused;
        Ok(())
    }

    pub fn play(&mut self) {
        info!("Playback start");
        self.sink.play();
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        info!("Playback pause");
        self.sink.pause();
        self.state = PlaybackState::Paused;
    }

    pub fn seek(&mut self, position: Duration) -> Result<(), PlayerError> {
        info!(position_secs = position.as_secs(), "Seeking");
        let path = self
            .current_track
            .clone()
            .ok_or(PlayerError::NoTrackLoaded)?;
        self.position = position;
        self.sink.stop();
        self.sink = Sink::connect_new(self.stream.mixer());
        let source = self
            .decode_source(&path)
            .map_err(|err| {
                error!(error = %err, path = %path.display(), "Failed to seek");
                err
            })?
            .skip_duration(position);
        self.sink.append(source);
        match self.state {
            PlaybackState::Playing => self.sink.play(),
            _ => self.sink.pause(),
        }
        Ok(())
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn position(&self) -> Duration {
        self.position
    }

    fn decode_source(&self, path: &Path) -> Result<Decoder<io::BufReader<File>>, PlayerError> {
        let file = File::open(path).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to open track file");
            err
        })?;
        Decoder::new(io::BufReader::new(file)).map_err(|err| {
            error!(error = %err, path = %path.display(), "Failed to decode track");
            err.into()
        })
    }
}
