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

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

// VideoTrackRow

#[derive(Insertable, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::video_track)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VideoTrackRow {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    pub media_type: String,
    pub width: i32,
    pub height: i32,
    pub bit_rate: i32,
    pub frame_rate: i32,
}

impl From<&domain::tracks::VideoTrack> for VideoTrackRow {
    fn from(t: &domain::tracks::VideoTrack) -> Self {
        VideoTrackRow {
            id: t.id,
            container_meta_id: t.container_meta_id,
            media_type: t.media_type.clone(),
            width: t.width,
            height: t.height,
            bit_rate: t.bit_rate,
            frame_rate: t.frame_rate,
        }
    }
}

impl From<VideoTrackRow> for domain::tracks::VideoTrack {
    fn from(row: VideoTrackRow) -> Self {
        domain::tracks::VideoTrack {
            id: row.id,
            container_meta_id: row.container_meta_id,
            media_type: row.media_type,
            width: row.width,
            height: row.height,
            bit_rate: row.bit_rate,
            frame_rate: row.frame_rate,
        }
    }
}

// AudioTrackRow

#[derive(Insertable, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::audio_track)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AudioTrackRow {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    pub media_type: String,
    pub bit_rate: i32,
    pub channel_config: String,
    pub sample_frequency: i32,
}

impl From<&domain::tracks::AudioTrack> for AudioTrackRow {
    fn from(t: &domain::tracks::AudioTrack) -> Self {
        AudioTrackRow {
            id: t.id,
            container_meta_id: t.container_meta_id,
            media_type: t.media_type.clone(),
            bit_rate: t.bit_rate,
            channel_config: t.channel_config.clone(),
            sample_frequency: t.sample_frequency,
        }
    }
}

impl From<AudioTrackRow> for domain::tracks::AudioTrack {
    fn from(row: AudioTrackRow) -> Self {
        domain::tracks::AudioTrack {
            id: row.id,
            container_meta_id: row.container_meta_id,
            media_type: row.media_type,
            bit_rate: row.bit_rate,
            channel_config: row.channel_config,
            sample_frequency: row.sample_frequency,
        }
    }
}

// SubtitleTrackRow

#[derive(Insertable, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::subtitle_track)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubtitleTrackRow {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    pub media_type: String,
}

impl From<&domain::tracks::SubtitleTrack> for SubtitleTrackRow {
    fn from(t: &domain::tracks::SubtitleTrack) -> Self {
        SubtitleTrackRow {
            id: t.id,
            container_meta_id: t.container_meta_id,
            media_type: t.media_type.clone(),
        }
    }
}

impl From<SubtitleTrackRow> for domain::tracks::SubtitleTrack {
    fn from(row: SubtitleTrackRow) -> Self {
        domain::tracks::SubtitleTrack {
            id: row.id,
            container_meta_id: row.container_meta_id,
            media_type: row.media_type,
        }
    }
}

// ContainerMetaRow

#[derive(Insertable, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::container_meta)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContainerMetaRow {
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

impl From<&domain::container_meta::ContainerMeta> for ContainerMetaRow {
    fn from(m: &domain::container_meta::ContainerMeta) -> Self {
        ContainerMetaRow {
            id: m.id,
            date_time_created: m.date_time_created,
            date_time_updated: m.date_time_updated,
            title: m.title.clone(),
            description: m.description.clone(),
            tags: m.tags.clone(),
            video_track_id: m.video_track_id,
            audio_track_id: m.audio_track_id,
            subtitle_track_id: m.subtitle_track_id,
            file_size_in_kb: m.file_size_in_kb,
            duration: m.duration,
        }
    }
}

impl From<ContainerMetaRow> for domain::container_meta::ContainerMeta {
    fn from(row: ContainerMetaRow) -> Self {
        domain::container_meta::ContainerMeta {
            id: row.id,
            date_time_created: row.date_time_created,
            date_time_updated: row.date_time_updated,
            title: row.title,
            description: row.description,
            tags: row.tags,
            video_track_id: row.video_track_id,
            audio_track_id: row.audio_track_id,
            subtitle_track_id: row.subtitle_track_id,
            file_size_in_kb: row.file_size_in_kb,
            duration: row.duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_video_track_row_roundtrip() {
        let container_meta_id = Uuid::new_v4();
        let domain = domain::tracks::VideoTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("h264"),
            width: 1280,
            height: 720,
            bit_rate: 1835,
            frame_rate: 30,
        };

        let row = VideoTrackRow::from(&domain);
        assert_eq!(row.id, domain.id);
        assert_eq!(row.media_type, domain.media_type);
        assert_eq!(row.width, domain.width);
        assert_eq!(row.height, domain.height);
        assert_eq!(row.bit_rate, domain.bit_rate);
        assert_eq!(row.frame_rate, domain.frame_rate);

        let roundtrip = domain::tracks::VideoTrack::from(row);
        assert_eq!(roundtrip, domain);
    }

    #[test]
    fn test_audio_track_row_roundtrip() {
        let container_meta_id = Uuid::new_v4();
        let domain = domain::tracks::AudioTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("aac"),
            bit_rate: 157,
            channel_config: String::from("stereo"),
            sample_frequency: 48000,
        };

        let row = AudioTrackRow::from(&domain);
        assert_eq!(row.id, domain.id);
        assert_eq!(row.media_type, domain.media_type);
        assert_eq!(row.bit_rate, domain.bit_rate);
        assert_eq!(row.channel_config, domain.channel_config);
        assert_eq!(row.sample_frequency, domain.sample_frequency);

        let roundtrip = domain::tracks::AudioTrack::from(row);
        assert_eq!(roundtrip, domain);
    }

    #[test]
    fn test_subtitle_track_row_roundtrip() {
        let container_meta_id = Uuid::new_v4();
        let domain = domain::tracks::SubtitleTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("vtt"),
        };

        let row = SubtitleTrackRow::from(&domain);
        assert_eq!(row.id, domain.id);
        assert_eq!(row.media_type, domain.media_type);

        let roundtrip = domain::tracks::SubtitleTrack::from(row);
        assert_eq!(roundtrip, domain);
    }

    #[test]
    fn test_container_meta_row_roundtrip() {
        let now = Utc::now();
        let domain = domain::container_meta::ContainerMeta {
            id: Uuid::new_v4(),
            date_time_created: now,
            date_time_updated: now,
            title: String::from("simple_container.mov"),
            description: String::from("A sample container"),
            tags: vec![
                Some(String::from("entertainment")),
                Some(String::from("music")),
            ],
            video_track_id: Some(Uuid::new_v4()),
            audio_track_id: Some(Uuid::new_v4()),
            subtitle_track_id: Some(Uuid::new_v4()),
            file_size_in_kb: 100000,
            duration: 200.23,
        };

        let row = ContainerMetaRow::from(&domain);
        assert_eq!(row.id, domain.id);
        assert_eq!(row.date_time_created, domain.date_time_created);
        assert_eq!(row.date_time_updated, domain.date_time_updated);
        assert_eq!(row.title, domain.title);
        assert_eq!(row.description, domain.description);
        assert_eq!(row.tags, domain.tags);
        assert_eq!(row.video_track_id, domain.video_track_id);
        assert_eq!(row.audio_track_id, domain.audio_track_id);
        assert_eq!(row.subtitle_track_id, domain.subtitle_track_id);
        assert_eq!(row.file_size_in_kb, domain.file_size_in_kb);
        assert_eq!(row.duration, domain.duration);

        let roundtrip = domain::container_meta::ContainerMeta::from(row);
        assert_eq!(roundtrip, domain);
    }
}
