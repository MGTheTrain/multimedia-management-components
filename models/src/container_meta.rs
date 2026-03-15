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

use uuid::Uuid;

use chrono::{DateTime, Utc};

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerMeta {
    pub id: Uuid,
    pub date_time_created: DateTime<Utc>,
    pub date_time_updated: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub tags: Vec<Option<String>>,
    pub video_track_id: Option<Uuid>,
    pub audio_track_id: Option<Uuid>,
    pub subtitle_track_id: Option<Uuid>,
    pub file_size_in_kb: i64,
    pub duration: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracks::{AudioTrack, SubtitleTrack, VideoTrack};

    // In order to run the test execute: `RUST_LOG=info cargo test`
    #[test]
    fn test_container_meta() {
        let container_meta_id = Uuid::new_v4();

        let video_track = VideoTrack {
            id: Uuid::new_v4(),
            name: String::from("simple_video.h264"),
            container_meta_id: container_meta_id,
            media_type: String::from("h264"),
            width: 1280,
            height: 720,
            bit_rate: 1835,
            frame_rate: 30,
        };

        let audio_track = AudioTrack {
            id: Uuid::new_v4(),
            name: String::from("simple_audio.aac"),
            container_meta_id: container_meta_id,
            media_type: String::from("aac"),
            bit_rate: 157,
            channel_config: String::from("stereo"),
            sample_frequency: 48000,
        };

        let subtitle_track = SubtitleTrack {
            id: Uuid::new_v4(),
            name: String::from("simple_subtitle.unkown"),
            container_meta_id: container_meta_id,
            media_type: String::from("unkown"),
        };

        let current_date_time = Utc::now();
        let tags: Vec<Option<String>> = vec![
            Some(String::from("entertainment")),
            Some(String::from("music")),
        ];

        let container_meta = ContainerMeta {
            id: Uuid::new_v4(),
            date_time_created: current_date_time,
            date_time_updated: current_date_time,
            title: String::from("simple_container.mov"),
            description: String::from("This is a sample container with video and audio to be stored in Youtube or Netflix shared container platform"),
            tags: tags,
            video_track_id: Some(video_track.id),
            audio_track_id: Some(audio_track.id),
            subtitle_track_id: Some(subtitle_track.id),
            file_size_in_kb: 100000,
            duration: 200.23,
        };

        assert_eq!(container_meta.date_time_created, current_date_time);
        assert_eq!(container_meta.date_time_updated, current_date_time);
        assert_eq!(container_meta.title, String::from("simple_container.mov"));
        assert_eq!(container_meta.description, "This is a sample container with video and audio to be stored in Youtube or Netflix shared container platform");
        assert_eq!(container_meta.tags.len(), 2);
        assert_eq!(container_meta.video_track_id, Some(video_track.id));
        assert_eq!(container_meta.audio_track_id, Some(audio_track.id));
        assert_eq!(container_meta.subtitle_track_id, Some(subtitle_track.id));
        assert_eq!(container_meta.file_size_in_kb, 100000);
        assert_eq!(container_meta.duration, 200.23);
    }
}
