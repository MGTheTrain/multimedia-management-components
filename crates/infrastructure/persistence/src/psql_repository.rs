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

// Ref:
// - https://github.com/weiznich/diesel_async/blob/main/tests/lib.rs
// - https://github.com/tokio-rs/axum/blob/main/examples/diesel-async-postgres/src/main.rs

use crate::db_models::{AudioTrackRow, ContainerMetaRow, SubtitleTrackRow, VideoTrackRow};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::{Connection, PgConnection};
use diesel_async::pooled_connection::{bb8::Pool, AsyncDieselConnectionManager};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use domain::repositories::{
    AudioTrackRepository, ContainerMetaRepository, RepositoryError, SubtitleTrackRepository,
    VideoTrackRepository,
};
use domain::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use log::info;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
static MIGRATION_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] diesel_async::pooled_connection::PoolError),
    #[error("Migration error: {0}")]
    Migration(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),
}

pub struct PsqlConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct PsqlRepository {
    pool: bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl PsqlRepository {
    pub async fn new(config: PsqlConfig) -> Result<Self, PersistenceError> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);
        let pool = Pool::builder()
            .build(manager)
            .await
            .map_err(PersistenceError::Pool)?;

        let _guard = MIGRATION_LOCK.lock().unwrap();
        let mut conn = PgConnection::establish(&config.database_url)?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(PersistenceError::Migration)?;
        Ok(PsqlRepository { pool })
    }

    async fn get_conn(
        &self,
    ) -> Result<
        bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        RepositoryError,
    > {
        self.pool.get().await.map_err(|e| match e {
            bb8::RunError::User(e) => RepositoryError::Database(e.to_string()),
            bb8::RunError::TimedOut => RepositoryError::Database("Pool timed out".into()),
        })
    }
}

#[async_trait]
impl VideoTrackRepository for PsqlRepository {
    async fn insert(&self, track: &VideoTrack) -> Result<VideoTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::video_track::dsl;
        let row = VideoTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: VideoTrackRow = diesel::insert_into(dsl::video_track)
            .values(&row)
            .returning(VideoTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Inserted video track {}", track.id);
        Ok(result.into())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<VideoTrack, RepositoryError> {
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: VideoTrackRow = dsl::video_track
            .filter(dsl::id.eq(id))
            .select(VideoTrackRow::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| RepositoryError::NotFound)?;
        Ok(row.into())
    }

    async fn update(&self, id: &Uuid, track: &VideoTrack) -> Result<VideoTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: VideoTrackRow = diesel::update(dsl::video_track.filter(dsl::id.eq(id)))
            .set((
                dsl::media_type.eq(&track.media_type),
                dsl::width.eq(track.width),
                dsl::height.eq(track.height),
                dsl::bit_rate.eq(track.bit_rate),
                dsl::frame_rate.eq(track.frame_rate),
            ))
            .returning(VideoTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(result.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        diesel::delete(dsl::video_track.filter(dsl::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl AudioTrackRepository for PsqlRepository {
    async fn insert(&self, track: &AudioTrack) -> Result<AudioTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::audio_track::dsl;
        let row = AudioTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: AudioTrackRow = diesel::insert_into(dsl::audio_track)
            .values(&row)
            .returning(AudioTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Inserted audio track {}", track.id);
        Ok(result.into())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<AudioTrack, RepositoryError> {
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: AudioTrackRow = dsl::audio_track
            .filter(dsl::id.eq(id))
            .select(AudioTrackRow::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| RepositoryError::NotFound)?;
        Ok(row.into())
    }

    async fn update(&self, id: &Uuid, track: &AudioTrack) -> Result<AudioTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: AudioTrackRow = diesel::update(dsl::audio_track.filter(dsl::id.eq(id)))
            .set((
                dsl::media_type.eq(&track.media_type),
                dsl::bit_rate.eq(track.bit_rate),
                dsl::channel_config.eq(&track.channel_config),
                dsl::sample_frequency.eq(track.sample_frequency),
            ))
            .returning(AudioTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Updated audio track {}", id);
        Ok(result.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        diesel::delete(dsl::audio_track.filter(dsl::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Deleted audio track {}", id);
        Ok(())
    }
}

#[async_trait]
impl SubtitleTrackRepository for PsqlRepository {
    async fn insert(&self, track: &SubtitleTrack) -> Result<SubtitleTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::subtitle_track::dsl;
        let row = SubtitleTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: SubtitleTrackRow = diesel::insert_into(dsl::subtitle_track)
            .values(&row)
            .returning(SubtitleTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Inserted subtitle track {}", track.id);
        Ok(result.into())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<SubtitleTrack, RepositoryError> {
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: SubtitleTrackRow = dsl::subtitle_track
            .filter(dsl::id.eq(id))
            .select(SubtitleTrackRow::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| RepositoryError::NotFound)?;
        Ok(row.into())
    }

    async fn update(
        &self,
        id: &Uuid,
        track: &SubtitleTrack,
    ) -> Result<SubtitleTrack, RepositoryError> {
        track
            .validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: SubtitleTrackRow = diesel::update(dsl::subtitle_track.filter(dsl::id.eq(id)))
            .set(dsl::media_type.eq(&track.media_type))
            .returning(SubtitleTrackRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Updated subtitle track {}", id);
        Ok(result.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        diesel::delete(dsl::subtitle_track.filter(dsl::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Deleted subtitle track {}", id);
        Ok(())
    }
}

#[async_trait]
impl ContainerMetaRepository for PsqlRepository {
    async fn insert(&self, meta: &ContainerMeta) -> Result<ContainerMeta, RepositoryError> {
        meta.validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::container_meta::dsl;
        let row = ContainerMetaRow::from(meta);
        let mut conn = self.get_conn().await?;
        let result: ContainerMetaRow = diesel::insert_into(dsl::container_meta)
            .values(&row)
            .returning(ContainerMetaRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Inserted container meta {}", meta.id);
        Ok(result.into())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<ContainerMeta, RepositoryError> {
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        let row: ContainerMetaRow = dsl::container_meta
            .filter(dsl::id.eq(id))
            .select(ContainerMetaRow::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| RepositoryError::NotFound)?;
        info!("Retrieved container meta {}", id);
        Ok(row.into())
    }

    async fn update(
        &self,
        id: &Uuid,
        meta: &ContainerMeta,
    ) -> Result<ContainerMeta, RepositoryError> {
        meta.validate()
            .map_err(|e| RepositoryError::Validation(e.to_string()))?;
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        let result: ContainerMetaRow = diesel::update(dsl::container_meta.filter(dsl::id.eq(id)))
            .set((
                dsl::date_time_updated.eq(meta.date_time_updated),
                dsl::title.eq(&meta.title),
                dsl::description.eq(&meta.description),
                dsl::tags.eq(&meta.tags),
                dsl::video_track_id.eq(meta.video_track_id),
                dsl::audio_track_id.eq(meta.audio_track_id),
                dsl::subtitle_track_id.eq(meta.subtitle_track_id),
                dsl::file_size_in_kb.eq(meta.file_size_in_kb),
                dsl::duration.eq(meta.duration),
            ))
            .returning(ContainerMetaRow::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Updated container meta {}", id);
        Ok(result.into())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), RepositoryError> {
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        diesel::delete(dsl::container_meta.filter(dsl::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        info!("Deleted container meta {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::tracks::VideoTrack;
    use uuid::Uuid;

    async fn setup() -> PsqlRepository {
        let _ = env_logger::try_init();
        PsqlRepository::new(PsqlConfig {
            database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
        })
        .await
        .expect("Failed to connect")
    }

    #[tokio::test]
    async fn test_video_track_crud() -> Result<(), Box<dyn std::error::Error>> {
        let repo = setup().await;
        let r = &repo as &dyn VideoTrackRepository;

        let mut track = VideoTrack {
            id: Uuid::new_v4(),
            container_meta_id: Uuid::new_v4(),
            media_type: String::from("h264"),
            width: 1280,
            height: 720,
            bit_rate: 1850,
            frame_rate: 60,
        };

        // insert
        let inserted = r.insert(&track).await?;
        assert_eq!(inserted.id, track.id);
        assert_eq!(inserted.media_type, "h264");

        // get
        let fetched = r.get_by_id(&track.id).await?;
        assert_eq!(fetched.width, 1280);
        assert_eq!(fetched.height, 720);

        // update
        track.width = 1920;
        track.height = 1080;
        let updated = r.update(&track.id, &track).await?;
        assert_eq!(updated.width, 1920);
        assert_eq!(updated.height, 1080);

        // delete
        r.delete(&track.id).await?;
        let result = r.get_by_id(&track.id).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        Ok(())
    }

    #[tokio::test]
    async fn test_audio_track_crud() -> Result<(), Box<dyn std::error::Error>> {
        let repo = setup().await;
        let r = &repo as &dyn AudioTrackRepository;

        let mut track = domain::tracks::AudioTrack {
            id: Uuid::new_v4(),
            container_meta_id: Uuid::new_v4(),
            media_type: String::from("aac"),
            bit_rate: 128,
            channel_config: String::from("stereo"),
            sample_frequency: 44100,
        };

        let inserted = r.insert(&track).await?;
        assert_eq!(inserted.media_type, "aac");

        let fetched = r.get_by_id(&track.id).await?;
        assert_eq!(fetched.sample_frequency, 44100);

        track.bit_rate = 256;
        let updated = r.update(&track.id, &track).await?;
        assert_eq!(updated.bit_rate, 256);

        r.delete(&track.id).await?;
        assert!(matches!(
            r.get_by_id(&track.id).await,
            Err(RepositoryError::NotFound)
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_container_meta_crud() -> Result<(), Box<dyn std::error::Error>> {
        let repo = setup().await;
        let video_r = &repo as &dyn VideoTrackRepository;
        let audio_r = &repo as &dyn AudioTrackRepository;
        let meta_r = &repo as &dyn ContainerMetaRepository;

        let video = VideoTrack {
            id: Uuid::new_v4(),
            container_meta_id: Uuid::new_v4(),
            media_type: String::from("h264"),
            width: 1920,
            height: 1080,
            bit_rate: 5000,
            frame_rate: 30,
        };
        let audio = domain::tracks::AudioTrack {
            id: Uuid::new_v4(),
            container_meta_id: Uuid::new_v4(),
            media_type: String::from("aac"),
            bit_rate: 128,
            channel_config: String::from("stereo"),
            sample_frequency: 48000,
        };
        video_r.insert(&video).await?;
        audio_r.insert(&audio).await?;

        use chrono::Utc;
        let now = Utc::now();
        let mut meta = ContainerMeta {
            id: Uuid::new_v4(),
            date_time_created: now,
            date_time_updated: now,
            title: String::from("test.mp4"),
            description: String::from("test"),
            tags: vec![Some(String::from("test"))],
            video_track_id: Some(video.id),
            audio_track_id: Some(audio.id),
            subtitle_track_id: None,
            file_size_in_kb: 1000,
            duration: 10.0,
        };

        let inserted = meta_r.insert(&meta).await?;
        assert_eq!(inserted.title, "test.mp4");

        let fetched = meta_r.get_by_id(&meta.id).await?;
        assert_eq!(fetched.file_size_in_kb, 1000);

        meta.title = String::from("updated.mp4");
        meta.date_time_updated = Utc::now();
        let updated = meta_r.update(&meta.id, &meta).await?;
        assert_eq!(updated.title, "updated.mp4");

        meta_r.delete(&meta.id).await?;
        assert!(matches!(
            meta_r.get_by_id(&meta.id).await,
            Err(RepositoryError::NotFound)
        ));

        video_r.delete(&video.id).await?;
        audio_r.delete(&audio.id).await?;

        Ok(())
    }
}
