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
use validator::Validate;

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct VideoTrack {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    #[validate(length(min = 1, message = "media_type must not be empty"))]
    pub media_type: String,
    #[validate(range(min = 1, message = "width must be positive"))]
    pub width: i32,
    #[validate(range(min = 1, message = "height must be positive"))]
    pub height: i32,
    #[validate(range(min = 1, message = "bit_rate must be positive"))]
    pub bit_rate: i32,
    #[validate(range(min = 1, message = "frame_rate must be positive"))]
    pub frame_rate: i32,
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct AudioTrack {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    #[validate(length(min = 1, message = "media_type must not be empty"))]
    pub media_type: String,
    #[validate(range(min = 0, message = "bit_rate must be non-negative"))]
    pub bit_rate: i32,
    #[validate(length(min = 1, message = "channel_config must not be empty"))]
    pub channel_config: String,
    #[validate(range(min = 1, message = "sample_frequency must be positive"))]
    pub sample_frequency: i32,
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct SubtitleTrack {
    pub id: Uuid,
    pub container_meta_id: Uuid,
    #[validate(length(min = 1, message = "media_type must not be empty"))]
    pub media_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track() {
        let container_meta_id = Uuid::new_v4();

        let video_track = VideoTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("h264"),
            width: 1280,
            height: 720,
            bit_rate: 1835,
            frame_rate: 30,
        };

        assert_eq!(video_track.container_meta_id, container_meta_id);
        assert_eq!(video_track.media_type, "h264");
        assert_eq!(video_track.width, 1280);
        assert_eq!(video_track.height, 720);
        assert_eq!(video_track.bit_rate, 1835);
        assert_eq!(video_track.frame_rate, 30);

        let audio_track = AudioTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("aac"),
            bit_rate: 157,
            channel_config: String::from("stereo"),
            sample_frequency: 48000,
        };

        assert_eq!(audio_track.container_meta_id, container_meta_id);
        assert_eq!(audio_track.media_type, "aac");
        assert_eq!(audio_track.bit_rate, 157);
        assert_eq!(audio_track.channel_config, "stereo");
        assert_eq!(audio_track.sample_frequency, 48000);

        let subtitle_track = SubtitleTrack {
            id: Uuid::new_v4(),
            container_meta_id,
            media_type: String::from("unknown"),
        };

        assert_eq!(subtitle_track.container_meta_id, container_meta_id);
        assert_eq!(subtitle_track.media_type, "unknown");

        assert!(video_track.validate().is_ok());
        assert!(audio_track.validate().is_ok());
        assert!(subtitle_track.validate().is_ok());
    }
}
