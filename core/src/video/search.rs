use anyhow;
use ffmpeg_next::{self as ffmpeg};
use std::path::Path;

const MEDIA_TYPE_VIDEOS: &[&str] = &[
    "mpeg", "mpg", "mp4", "avi", "ogg", "webm", "flv", "mov", "mkv",
];

#[derive(Debug, Clone, Default)]
pub struct Video {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub duration: f64,
    pub rate: f64,
    pub nb_frames: i64,
    pub width: u32,
    pub height: u32,
}

pub async fn search_videos<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Video>> {
    let mut videos = Vec::new();
    let mut dir = tokio::fs::read_dir(path).await?;

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let metadata = tokio::fs::metadata(&path).await?;

        if metadata.is_dir() || !is_video(&path) {
            continue;
        }

        let video_path = entry.path().to_str().unwrap().to_owned();

        let mut video = Video {
            name: entry.file_name().to_str().unwrap().to_owned(),
            path: video_path.clone(),
            size: (metadata.len() as f64 / 1024.0_f64).round() as usize,
            ..Default::default()
        };

        if let Ok(metada) = get_metadata(&video_path) {
            video.duration = metada.duration;
            video.rate = metada.rate;
            video.nb_frames = metada.nb_frames;
            video.width = metada.width;
            video.height = metada.height;
        }

        videos.push(video);
    }

    Ok(videos)
}

fn is_video(entry: &Path) -> bool {
    matches!(
        entry.extension(), 
        Some(ext) if MEDIA_TYPE_VIDEOS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
}

#[derive(Default)]
struct Metadata {
    duration: f64,
    rate: f64,
    nb_frames: i64,
    width: u32,
    height: u32,
}

fn get_metadata(video_path: &str) -> anyhow::Result<Metadata> {
    ffmpeg::init()?;
    let context = ffmpeg::format::input(video_path)?;

    let mut metadata = Metadata {
        // duration (seconds)
        duration: context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE),
        ..Default::default()
    };

    if let Some(video_stream) = context.streams().best(ffmpeg::media::Type::Video) {
        metadata.rate = if video_stream.rate().denominator() > 0 {
            video_stream.rate().numerator() as f64 / video_stream.rate().denominator() as f64
        } else {
            0f64
        };
        metadata.nb_frames = if video_stream.frames() > 0 {
            video_stream.frames()
        } else {
            (metadata.duration * metadata.rate) as i64
        };

        let codec = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
        if codec.medium() == ffmpeg::media::Type::Video {
            if let Ok(video) = codec.decoder().video() {
                metadata.width = video.width();
                metadata.height = video.height();
            }
        }
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_videos() {
        let path = Path::new("../data/video");
        let videos = search_videos(path).await.expect("Failed to search videos.");

        println!("{:#?}", videos);

        // Assert
        assert_eq!(videos.is_empty(), false);
    }
}
