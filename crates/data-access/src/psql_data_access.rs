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
use diesel::prelude::*;
use diesel::{Connection, PgConnection};
use diesel_async::pooled_connection::{bb8::Pool, AsyncDieselConnectionManager};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;
use models::{
    container_meta::ContainerMeta,
    tracks::{AudioTrack, SubtitleTrack, VideoTrack},
};
use thiserror::Error;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Error)]
pub enum DataAccessError {
    #[error("Database error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel_async::pooled_connection::PoolError),
    #[error("Migration error: {0}")]
    Migration(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
}

pub struct PsqlDataAccessConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct PsqlDataAccess {
    pub connection_pool: bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl PsqlDataAccess {
    pub async fn new(config: PsqlDataAccessConfig) -> Result<Self, DataAccessError> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);

        let connection_pool = Pool::builder()
            .build(manager)
            .await
            .map_err(DataAccessError::Pool)?;

        let mut migration_conn = PgConnection::establish(&config.database_url)?;

        migration_conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(DataAccessError::Migration)?;

        Ok(PsqlDataAccess { connection_pool })
    }

    async fn get_conn(
        &self,
    ) -> Result<
        bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>,
        DataAccessError,
    > {
        self.connection_pool.get().await.map_err(|e| match e {
            bb8::RunError::User(pool_err) => DataAccessError::Pool(pool_err),
            bb8::RunError::TimedOut => {
                DataAccessError::Pool(diesel_async::pooled_connection::PoolError::ConnectionError(
                    diesel::ConnectionError::BadConnection("Pool timed out".into()),
                ))
            }
        })
    }

    pub async fn insert_video_track(
        &self,
        track: &VideoTrack,
    ) -> Result<VideoTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::video_track::dsl;
        let row = VideoTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: VideoTrackRow = diesel::insert_into(dsl::video_track)
            .values(&row)
            .returning(VideoTrackRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Inserted video track {}", track.id);
        Ok(result.into())
    }

    pub async fn insert_audio_track(
        &self,
        track: &AudioTrack,
    ) -> Result<AudioTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::audio_track::dsl;
        let row = AudioTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: AudioTrackRow = diesel::insert_into(dsl::audio_track)
            .values(&row)
            .returning(AudioTrackRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Inserted audio track {}", track.id);
        Ok(result.into())
    }

    pub async fn insert_subtitle_track(
        &self,
        track: &SubtitleTrack,
    ) -> Result<SubtitleTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::subtitle_track::dsl;
        let row = SubtitleTrackRow::from(track);
        let mut conn = self.get_conn().await?;
        let result: SubtitleTrackRow = diesel::insert_into(dsl::subtitle_track)
            .values(&row)
            .returning(SubtitleTrackRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Inserted subtitle track {}", track.id);
        Ok(result.into())
    }

    pub async fn insert_container_meta(
        &self,
        meta: &ContainerMeta,
    ) -> Result<ContainerMeta, DataAccessError> {
        meta.validate()?;
        use crate::schema::container_meta::dsl;
        let row = ContainerMetaRow::from(meta);
        let mut conn = self.get_conn().await?;
        let result: ContainerMetaRow = diesel::insert_into(dsl::container_meta)
            .values(&row)
            .returning(ContainerMetaRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Inserted container meta {}", meta.id);
        Ok(result.into())
    }

    pub async fn get_video_track_by_id(
        &self,
        track_id: &Uuid,
    ) -> Result<VideoTrack, DataAccessError> {
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: VideoTrackRow = dsl::video_track
            .filter(dsl::id.eq(track_id))
            .select(VideoTrackRow::as_select())
            .first(&mut conn)
            .await?;
        info!("Retrieved video track {}", track_id);
        Ok(row.into())
    }

    pub async fn get_audio_track_by_id(
        &self,
        track_id: &Uuid,
    ) -> Result<AudioTrack, DataAccessError> {
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: AudioTrackRow = dsl::audio_track
            .filter(dsl::id.eq(track_id))
            .select(AudioTrackRow::as_select())
            .first(&mut conn)
            .await?;
        info!("Retrieved audio track {}", track_id);
        Ok(row.into())
    }

    pub async fn get_subtitle_track_by_id(
        &self,
        track_id: &Uuid,
    ) -> Result<SubtitleTrack, DataAccessError> {
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        let row: SubtitleTrackRow = dsl::subtitle_track
            .filter(dsl::id.eq(track_id))
            .select(SubtitleTrackRow::as_select())
            .first(&mut conn)
            .await?;
        info!("Retrieved subtitle track {}", track_id);
        Ok(row.into())
    }

    pub async fn get_container_meta_by_id(
        &self,
        meta_id: &Uuid,
    ) -> Result<ContainerMeta, DataAccessError> {
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        let row: ContainerMetaRow = dsl::container_meta
            .filter(dsl::id.eq(meta_id))
            .select(ContainerMetaRow::as_select())
            .first(&mut conn)
            .await?;
        info!("Retrieved container meta {}", meta_id);
        Ok(row.into())
    }

    pub async fn update_video_track_by_id(
        &self,
        track_id: &Uuid,
        track: &VideoTrack,
    ) -> Result<VideoTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: VideoTrackRow = diesel::update(dsl::video_track.filter(dsl::id.eq(track_id)))
            .set((
                dsl::media_type.eq(&track.media_type),
                dsl::width.eq(track.width),
                dsl::height.eq(track.height),
                dsl::bit_rate.eq(track.bit_rate),
                dsl::frame_rate.eq(track.frame_rate),
            ))
            .returning(VideoTrackRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Updated video track {}", track_id);
        Ok(result.into())
    }

    pub async fn update_audio_track_by_id(
        &self,
        track_id: &Uuid,
        track: &AudioTrack,
    ) -> Result<AudioTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: AudioTrackRow = diesel::update(dsl::audio_track.filter(dsl::id.eq(track_id)))
            .set((
                dsl::media_type.eq(&track.media_type),
                dsl::bit_rate.eq(track.bit_rate),
                dsl::channel_config.eq(&track.channel_config),
                dsl::sample_frequency.eq(track.sample_frequency),
            ))
            .returning(AudioTrackRow::as_returning())
            .get_result(&mut conn)
            .await?;
        info!("Updated audio track {}", track_id);
        Ok(result.into())
    }

    pub async fn update_subtitle_track_by_id(
        &self,
        track_id: &Uuid,
        track: &SubtitleTrack,
    ) -> Result<SubtitleTrack, DataAccessError> {
        track.validate()?;
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        let result: SubtitleTrackRow =
            diesel::update(dsl::subtitle_track.filter(dsl::id.eq(track_id)))
                .set((dsl::media_type.eq(&track.media_type),))
                .returning(SubtitleTrackRow::as_returning())
                .get_result(&mut conn)
                .await?;
        info!("Updated subtitle track {}", track_id);
        Ok(result.into())
    }

    pub async fn update_container_meta_by_id(
        &self,
        meta_id: &Uuid,
        meta: &ContainerMeta,
    ) -> Result<ContainerMeta, DataAccessError> {
        meta.validate()?;
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        let result: ContainerMetaRow =
            diesel::update(dsl::container_meta.filter(dsl::id.eq(meta_id)))
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
                .await?;
        info!("Updated container meta {}", meta_id);
        Ok(result.into())
    }

    pub async fn delete_video_track_by_id(&self, track_id: &Uuid) -> Result<(), DataAccessError> {
        use crate::schema::video_track::dsl;
        let mut conn = self.get_conn().await?;
        let rows = diesel::delete(dsl::video_track.filter(dsl::id.eq(track_id)))
            .execute(&mut conn)
            .await?;
        if rows == 0 {
            return Err(DataAccessError::Diesel(diesel::result::Error::NotFound));
        }
        info!("Deleted video track {}", track_id);
        Ok(())
    }

    pub async fn delete_audio_track_by_id(&self, track_id: &Uuid) -> Result<(), DataAccessError> {
        use crate::schema::audio_track::dsl;
        let mut conn = self.get_conn().await?;
        let rows = diesel::delete(dsl::audio_track.filter(dsl::id.eq(track_id)))
            .execute(&mut conn)
            .await?;
        if rows == 0 {
            return Err(DataAccessError::Diesel(diesel::result::Error::NotFound));
        }
        info!("Deleted audio track {}", track_id);
        Ok(())
    }

    pub async fn delete_subtitle_track_by_id(
        &self,
        track_id: &Uuid,
    ) -> Result<(), DataAccessError> {
        use crate::schema::subtitle_track::dsl;
        let mut conn = self.get_conn().await?;
        let rows = diesel::delete(dsl::subtitle_track.filter(dsl::id.eq(track_id)))
            .execute(&mut conn)
            .await?;
        if rows == 0 {
            return Err(DataAccessError::Diesel(diesel::result::Error::NotFound));
        }
        info!("Deleted subtitle track {}", track_id);
        Ok(())
    }

    pub async fn delete_container_meta_by_id(&self, meta_id: &Uuid) -> Result<(), DataAccessError> {
        use crate::schema::container_meta::dsl;
        let mut conn = self.get_conn().await?;
        let rows = diesel::delete(dsl::container_meta.filter(dsl::id.eq(meta_id)))
            .execute(&mut conn)
            .await?;
        if rows == 0 {
            return Err(DataAccessError::Diesel(diesel::result::Error::NotFound));
        }
        info!("Deleted container meta {}", meta_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_psql_data_access_methods_for_track() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init();

        let config = PsqlDataAccessConfig {
            database_url: String::from("postgres://user:password@localhost:5432/diesel-demo"),
        };
        let psql_data_access = PsqlDataAccess::new(config).await?;

        let mut video_track = VideoTrack {
            id: Uuid::new_v4(),
            container_meta_id: Uuid::new_v4(),
            media_type: String::from("h264"),
            width: 1280,
            height: 720,
            bit_rate: 1850,
            frame_rate: 60,
        };

        let result = psql_data_access.insert_video_track(&video_track).await;
        assert!(result.is_ok());

        let fetched = psql_data_access
            .get_video_track_by_id(&video_track.id)
            .await?;
        assert_eq!(fetched.container_meta_id, video_track.container_meta_id);
        assert_eq!(fetched.width, video_track.width);
        assert_eq!(fetched.height, video_track.height);
        assert_eq!(fetched.bit_rate, video_track.bit_rate);
        assert_eq!(fetched.frame_rate, video_track.frame_rate);

        video_track.width = 1920;
        video_track.height = 1080;
        let updated = psql_data_access
            .update_video_track_by_id(&video_track.id, &video_track)
            .await?;
        assert_eq!(updated.width, 1920);
        assert_eq!(updated.height, 1080);

        let delete_result = psql_data_access
            .delete_video_track_by_id(&video_track.id)
            .await;
        assert!(delete_result.is_ok());

        Ok(())
    }
}
