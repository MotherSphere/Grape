use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, source::Source};

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
    NoTrackLoaded,
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::Io(err) => write!(f, "io error: {err}"),
            PlayerError::DecoderError(err) => write!(f, "decoder error: {err}"),
            PlayerError::StreamError(err) => write!(f, "stream error: {err}"),
            PlayerError::PlayError(err) => write!(f, "play error: {err}"),
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

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    state: PlaybackState,
    current_track: Option<PathBuf>,
    position: Duration,
}

impl Player {
    pub fn new() -> Result<Self, PlayerError> {
        let (stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;
        Ok(Self {
            _stream: stream,
            handle,
            sink,
            state: PlaybackState::Stopped,
            current_track: None,
            position: Duration::ZERO,
        })
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), PlayerError> {
        let path = path.as_ref().to_path_buf();
        self.current_track = Some(path.clone());
        self.position = Duration::ZERO;
        self.sink.stop();
        self.sink = Sink::try_new(&self.handle)?;
        let source = self.decode_source(&path)?;
        self.sink.append(source);
        self.sink.pause();
        self.state = PlaybackState::Paused;
        Ok(())
    }

    pub fn play(&mut self) {
        self.sink.play();
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.state = PlaybackState::Paused;
    }

    pub fn seek(&mut self, position: Duration) -> Result<(), PlayerError> {
        let path = self
            .current_track
            .clone()
            .ok_or(PlayerError::NoTrackLoaded)?;
        self.position = position;
        self.sink.stop();
        self.sink = Sink::try_new(&self.handle)?;
        let source = self.decode_source(&path)?.skip_duration(position);
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
        let file = File::open(path)?;
        Ok(Decoder::new(io::BufReader::new(file))?)
    }
}
