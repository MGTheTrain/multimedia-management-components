CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE video_track (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_meta_id UUID NOT NULL,
    media_type VARCHAR NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    bit_rate INTEGER NOT NULL,
    frame_rate INTEGER NOT NULL
);

CREATE TABLE audio_track (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_meta_id UUID NOT NULL,
    media_type VARCHAR NOT NULL,
    bit_rate INTEGER NOT NULL,
    channel_config VARCHAR NOT NULL,
    sample_frequency INTEGER NOT NULL
);

CREATE TABLE subtitle_track (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_meta_id UUID NOT NULL,
    media_type VARCHAR NOT NULL
);

CREATE TABLE container_meta (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date_time_created TIMESTAMPTZ NOT NULL,
    date_time_updated TIMESTAMPTZ NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    video_track_id UUID REFERENCES video_track(id) ON DELETE CASCADE,
    audio_track_id UUID REFERENCES audio_track(id) ON DELETE CASCADE,
    subtitle_track_id UUID REFERENCES subtitle_track(id) ON DELETE CASCADE,
    file_size_in_kb BIGINT NOT NULL,
    duration DOUBLE PRECISION NOT NULL
);