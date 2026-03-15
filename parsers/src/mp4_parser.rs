// The MIT License
//
// Copyright (c) 2026 MGTheTrain
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use chrono::Utc;
use log::info;
use models::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use mp4::{Mp4Track, TrackType};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MP4 error: {0}")]
    Mp4(#[from] mp4::Error),
}

pub struct Mp4Metadata {
    pub container: ContainerMeta,
    pub video: Option<VideoTrack>,
    pub audio: Option<AudioTrack>,
    pub subtitle: Option<SubtitleTrack>,
}

#[derive(Clone, Default)]
pub struct Mp4Parser;

impl Mp4Parser {
    pub fn parse_from_file(path: impl AsRef<Path>) -> Result<Mp4Metadata, ParseError> {
        let f = File::open(&path)?;
        let size = f.metadata()?.len();
        let reader = BufReader::new(f);
        let mp4 = mp4::Mp4Reader::read_header(reader, size)?;

        let video = mp4
            .tracks()
            .values()
            .find(|t| matches!(t.track_type(), Ok(TrackType::Video)))
            .map(Self::get_video_info)
            .transpose()?;
        let audio = mp4
            .tracks()
            .values()
            .find(|t| matches!(t.track_type(), Ok(TrackType::Audio)))
            .map(Self::get_audio_info)
            .transpose()?;

        let subtitle = mp4
            .tracks()
            .values()
            .find(|t| matches!(t.track_type(), Ok(TrackType::Subtitle)))
            .map(Self::get_subtitle_info)
            .transpose()?;

        let now = Utc::now();
        let container = ContainerMeta {
            id: Uuid::new_v4(),
            date_time_created: now,
            date_time_updated: now,
            file_size_in_kb: mp4.size() as i64,
            duration: mp4.duration().as_secs_f64(),
            video_track_id: video.as_ref().map(|v| v.id),
            audio_track_id: audio.as_ref().map(|a| a.id),
            subtitle_track_id: subtitle.as_ref().map(|s| s.id),
            ..Default::default()
        };

        info!("Successfully parsed MP4 file: {}", path.as_ref().display());
        Ok(Mp4Metadata {
            container,
            video,
            audio,
            subtitle,
        })
    }

    fn get_video_info(track: &Mp4Track) -> Result<VideoTrack, ParseError> {
        info!(
            "{} ({}) ({:?}), {}x{}, {} kb/s, {:.2} fps",
            track.media_type()?,
            track.video_profile()?,
            track.box_type()?,
            track.width(),
            track.height(),
            track.bitrate() / 1000,
            track.frame_rate()
        );
        Ok(VideoTrack {
            id: Uuid::new_v4(),
            media_type: track.media_type()?.to_string(),
            width: track.width() as i32,
            height: track.height() as i32,
            bit_rate: (track.bitrate() / 1000) as i32,
            frame_rate: track.frame_rate() as i32,
            ..Default::default()
        })
    }

    fn get_audio_info(track: &Mp4Track) -> Result<AudioTrack, ParseError> {
        let profile = track
            .audio_profile()
            .map(|p| p.to_string())
            .unwrap_or_else(|_| "-".to_string());
        let channel_config = track
            .channel_config()
            .map(|c| c.to_string())
            .unwrap_or_else(|_| "-".to_string());
        info!(
            "{} ({}) ({:?}), {} Hz, {}, {} kb/s",
            track.media_type()?,
            profile,
            track.box_type()?,
            track.sample_freq_index()?.freq(),
            channel_config,
            track.bitrate() / 1000
        );
        Ok(AudioTrack {
            id: Uuid::new_v4(),
            media_type: track.media_type()?.to_string(),
            bit_rate: (track.bitrate() / 1000) as i32,
            channel_config,
            sample_frequency: track.sample_freq_index()?.freq() as i32,
            ..Default::default()
        })
    }

    fn get_subtitle_info(track: &Mp4Track) -> Result<SubtitleTrack, ParseError> {
        info!("{} ({:?})", track.media_type()?, track.box_type()?);
        Ok(SubtitleTrack {
            id: Uuid::new_v4(),
            media_type: track.media_type()?.to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_file() {
        let result = Mp4Parser::parse_from_file("../assets/nature.mp4").unwrap();

        assert_eq!(result.container.file_size_in_kb, 5862561);
        assert!(result.container.video_track_id.is_some());
        assert!(result.container.audio_track_id.is_some());
        assert!(result.container.subtitle_track_id.is_none());

        let video = result.video.unwrap();
        assert_eq!(video.media_type, "h264");
        assert_eq!(video.bit_rate, 1137);
        assert_eq!(video.frame_rate, 25);
        assert_eq!(video.width, 1280);
        assert_eq!(video.height, 720);

        let audio = result.audio.unwrap();
        assert_eq!(audio.media_type, "aac");
        assert_eq!(audio.bit_rate, 0);
        assert_eq!(audio.sample_frequency, 44100);

        assert!(result.subtitle.is_none());
    }
}
