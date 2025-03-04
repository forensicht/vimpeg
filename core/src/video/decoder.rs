use std::{collections::HashMap, fs, path::Path};

use anyhow::{self, Context};
use bytes::Bytes;
use ffmpeg::{format, media::Type, software::scaling, util::frame};
use ffmpeg_next::{self as ffmpeg};
use image::{imageops, DynamicImage, GenericImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use rust_embed::RustEmbed;

const FRAME_DIMENSION: u32 = 300;

#[derive(RustEmbed)]
#[folder = "../data/fonts/"]
struct Fonts;

#[derive(Debug)]
struct FontSettings {
    font: rusttype::Font<'static>,
    font_scale: rusttype::Scale,
    font_x: i32,
    font_y: i32,
}

#[derive(Debug)]
pub struct VideoFrame {
    pub data: Bytes,
    pub timestamp: f64,
}

#[derive(Debug, Default)]
pub struct VideoDump {
    pub width: u32,
    pub height: u32,
    pub nframes: usize,
    pub frames: HashMap<usize, VideoFrame>,
}

#[derive(Debug, Default)]
pub struct VideoThumb {
    pub width: u32,
    pub height: u32,
    pub data: Option<Bytes>,
}

fn get_font_settings(
    frame_width: i32,
    frame_height: i32,
    font_scale: f32,
) -> anyhow::Result<FontSettings> {
    let font =
        Fonts::get("DejaVuSans.ttf").ok_or_else(|| anyhow::anyhow!("could not load font"))?;
    let font = font.data.to_vec();
    let font = rusttype::Font::try_from_vec(font)
        .ok_or_else(|| anyhow::anyhow!("could not create font"))?;
    let font_height = if frame_height > frame_width {
        (10.0 * font_scale * frame_width as f32) / 360_f32
    } else {
        (5.0 * font_scale * frame_width as f32) / 360_f32
    };
    let font_scale = rusttype::Scale {
        x: font_height * 2.0,
        y: font_height,
    };
    let font_size = text_size(font_scale, &font, "77:77:77.777");
    let font_x = frame_width - (font_size.0 + 5);
    let font_y = frame_height - (font_size.1 + 5);

    Ok(FontSettings {
        font,
        font_scale,
        font_x,
        font_y,
    })
}

fn draw_timestamp(img: &mut DynamicImage, timestamp: f64, font_settings: &FontSettings) {
    let seconds = timestamp % 60.0;
    let minutes = ((timestamp / 60.0) % 60.0) as u32;
    let hours = ((timestamp / 60.0) / 60.0) as u32;

    // put timestamp on image
    let text = format!("{:0>2}:{:0>2}:{:0>6.3}", hours, minutes, seconds);
    draw_text_mut(
        img,
        image::Rgba([255u8, 111u8, 0u8, 255u8]),
        font_settings.font_x,
        font_settings.font_y,
        font_settings.font_scale,
        &font_settings.font,
        text.as_str(),
    );
}

pub fn get_thumbnail<P: AsRef<Path>>(video_path: P) -> anyhow::Result<VideoThumb> {
    ffmpeg::init()?;
    let mut input_format_context = ffmpeg::format::input(&video_path)?;

    let (video_stream_index, mut decoder) = {
        let stream = input_format_context
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let stream_index = stream.index();
        let decode_context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = decode_context.decoder().video()?;

        (stream_index, decoder)
    };

    let cover_frame = loop {
        let mut packet_iter = input_format_context.packets();
        let cover_packet = loop {
            match packet_iter.next() {
                Some((stream, packet)) if stream.index() == video_stream_index => break packet,
                _ => {}
            }
        };

        decoder.send_packet(&cover_packet)?;

        // repeatedly send packet until a frame can be extracted
        let mut decoded = frame::Video::empty();
        if let Ok(()) = decoder.receive_frame(&mut decoded) {
            break decoded;
        }
    };

    let scaled_frame = {
        let mut sws_context = scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            format::Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            scaling::Flags::BILINEAR,
        )
        .context("Invalid swscontext parameter.")?;

        let mut rgb_frame = frame::Video::empty();
        sws_context
            .run(&cover_frame, &mut rgb_frame)
            .context("Error swscontext run.")?;

        rgb_frame
    };

    let video_thumb = VideoThumb {
        width: decoder.width(),
        height: decoder.height(),
        data: Some(Bytes::from(scaled_frame.data(0).to_owned())),
    };

    Ok(video_thumb)
}

pub fn dump_video_frames_into_image<P: AsRef<Path>>(
    video_path: P,
    image_path: P,
    cols: usize,
    rows: usize,
    show_timestamp: bool,
) -> anyhow::Result<()> {
    let nframes = rows * cols;
    let dump = frame_dump(video_path, nframes)?;
    let img = concat_frames(dump, cols, rows, show_timestamp)?;
    img.save(&image_path).context(format!(
        "failed to save image {}",
        image_path.as_ref().display()
    ))?;
    Ok(())
}

#[derive(Debug)]
pub struct VideoDumpResult {
    pub file_name: String,
    pub image_paths: Vec<String>,
}

pub fn dump_video_frames_by_time<P: AsRef<Path>>(
    video_path: P,
    save_path: P,
    time_start: f64,
    time_end: f64,
    frame_rate: u32,
    show_timestamp: bool,
) -> anyhow::Result<VideoDumpResult> {
    let video_name = video_path
        .as_ref()
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("invalid video path"))?
        .to_str()
        .unwrap_or("");
    let save_path = save_path.as_ref().join(video_name);
    if !save_path.exists() {
        fs::create_dir_all(&save_path)
            .with_context(|| format!("Could not create `{}` path", save_path.display()))?;
    }

    let dump = frame_dump_by_time(&video_path, time_start, time_end, frame_rate)?;
    let image_paths = frames_to_image(&dump, video_name, &save_path, show_timestamp)?;

    Ok(VideoDumpResult {
        file_name: video_name.to_owned(),
        image_paths,
    })
}

pub fn frame_dump<P: AsRef<Path>>(video_path: P, nframes: usize) -> anyhow::Result<VideoDump> {
    ffmpeg::init()?;

    let options = ffmpeg::Dictionary::new();
    let mut input_format_context = ffmpeg::format::input_with_dictionary(&video_path, options)?;

    // shows a dump of the video
    // let video_path = video_path.as_ref().as_os_str().to_str().unwrap();
    // format::context::input::dump(&input_format_context, 0, Some(video_path));

    let (video_stream_index, frame_rate, time_base, mut decoder) = {
        let stream = input_format_context
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let total_frames = if stream.frames() > 0 {
            stream.frames()
        } else if stream.rate().0 > 0 && input_format_context.duration() > 0 {
            // calculates the number of frames
            let frame_rate = if stream.rate().denominator() > 0 {
                stream.rate().numerator() as f64 / stream.rate().denominator() as f64
            } else {
                0f64
            };
            let duration =
                input_format_context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE); // to seconds
            let nb_frames = duration * frame_rate;
            nb_frames as i64
        } else {
            0
        };

        if nframes as i64 > total_frames {
            anyhow::bail!(
                "nframes must be smaller than the total video frames [{}]",
                total_frames
            );
        }

        let frame_rate = total_frames / nframes as i64;
        let time_base = f64::from(stream.time_base());
        let stream_index = stream.index();

        let decode_context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = decode_context.decoder().video()?;

        (stream_index, frame_rate, time_base, decoder)
    };

    let mut video_dump = VideoDump {
        width: decoder.width(),
        height: decoder.height(),
        ..Default::default()
    };

    let mut sws_context = scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        format::Pixel::RGBA,
        decoder.width(),
        decoder.height(),
        scaling::Flags::BILINEAR,
    )
    .context("invalid swscontext parameter")?;

    let mut frame_index = 0;
    let mut processed_frames = 0;

    let mut receive_and_process_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = frame::Video::empty();

            while decoder.receive_frame(&mut decoded).is_ok() {
                if (frame_index < nframes)
                    && (processed_frames == 0 || processed_frames == frame_rate)
                {
                    let mut rgb_frame = frame::Video::empty();
                    sws_context.run(&decoded, &mut rgb_frame)?;

                    let timestamp = if let Some(timestamp) = decoded.timestamp() {
                        timestamp as f64 * time_base
                    } else {
                        0f64
                    };

                    let data = rgb_frame.data(0).to_owned();
                    let video_frame = VideoFrame {
                        data: Bytes::from(data),
                        timestamp,
                    };
                    video_dump.frames.insert(frame_index, video_frame);

                    frame_index += 1;
                    processed_frames = 0;
                }

                processed_frames += 1;
            }

            Ok(())
        };

    for (stream, packet) in input_format_context.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_frames(&mut decoder)?;
        }
    }

    decoder.send_eof()?;
    receive_and_process_frames(&mut decoder)?;

    video_dump.nframes = frame_index;

    Ok(video_dump)
}

pub fn frame_dump_by_time<P: AsRef<Path>>(
    video_path: P,
    time_start: f64,
    time_end: f64,
    frame_rate: u32,
) -> anyhow::Result<VideoDump> {
    ffmpeg::init()?;

    let options = ffmpeg::Dictionary::new();
    let mut input_format_context = ffmpeg::format::input_with_dictionary(&video_path, options)?;

    let (video_stream_index, time_base, frame_rate, mut decoder) = {
        let stream = input_format_context
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let rate = if stream.rate().denominator() > 0 {
            stream.rate().numerator() as f64 / stream.rate().denominator() as f64
        } else {
            0f64
        };

        let frame_rate = (rate / frame_rate as f64).round() as u32;
        let time_base = f64::from(stream.time_base());
        let stream_index = stream.index();
        let decode_context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        let decoder = decode_context.decoder().video()?;

        (stream_index, time_base, frame_rate, decoder)
    };

    let mut video_dump = VideoDump {
        width: decoder.width(),
        height: decoder.height(),
        ..Default::default()
    };

    let mut sws_context = scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        format::Pixel::RGBA,
        decoder.width(),
        decoder.height(),
        scaling::Flags::BILINEAR,
    )
    .context("invalid swscontext parameter")?;

    let mut frame_index = 0;
    let mut processed_frames = 0;

    let mut receive_and_process_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = frame::Video::empty();

            while decoder.receive_frame(&mut decoded).is_ok() {
                if processed_frames == 0 || processed_frames == frame_rate {
                    let timestamp = if let Some(timestamp) = decoded.timestamp() {
                        timestamp as f64 * time_base
                    } else {
                        0f64
                    };

                    if timestamp >= time_start && timestamp <= time_end {
                        let mut rgb_frame = frame::Video::empty();
                        sws_context.run(&decoded, &mut rgb_frame)?;

                        let data = rgb_frame.data(0).to_owned();
                        let video_frame = VideoFrame {
                            data: Bytes::from(data),
                            timestamp,
                        };
                        video_dump.frames.insert(frame_index, video_frame);

                        frame_index += 1;
                    }

                    processed_frames = 0;
                }

                processed_frames += 1;
            }

            Ok(())
        };

    for (stream, packet) in input_format_context.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            receive_and_process_frames(&mut decoder)?;
        }
    }

    decoder.send_eof()?;
    receive_and_process_frames(&mut decoder)?;

    video_dump.nframes = frame_index;

    Ok(video_dump)
}

fn concat_frames(
    dump: VideoDump,
    cols: usize,
    rows: usize,
    show_timestamp: bool,
) -> anyhow::Result<DynamicImage> {
    let frames = frames_to_thumbnail(&dump, show_timestamp)?;
    let img_width_out: u32 = frames.iter().map(|img| img.width()).take(cols).sum();
    let img_height_out: u32 = frames.iter().map(|img| img.height()).take(rows).sum();

    // Initialize an image buffer with the appropriate size.
    let mut imgbuf = ImageBuffer::new(img_width_out, img_height_out);
    let mut accumulated_width = 0;
    let mut accumulated_heigth = 0;

    // Copy each input image at the correct location in the output image.
    for img in frames {
        if accumulated_width == img_width_out {
            accumulated_width = 0;
            accumulated_heigth += img.height();
        }

        imgbuf.copy_from(&img, accumulated_width, accumulated_heigth)?;
        accumulated_width += img.width();
    }

    let dynamic_img = DynamicImage::ImageRgba8(imgbuf);

    Ok(dynamic_img)
}

fn frames_to_thumbnail(
    dump: &VideoDump,
    show_timestamp: bool,
) -> anyhow::Result<Vec<DynamicImage>> {
    let width = dump.width;
    let height = dump.height;
    let font_settings = get_font_settings(width as i32, height as i32, 2.0)?;
    let mut frames = Vec::with_capacity(dump.nframes);

    for i in 0..dump.nframes {
        if let Some(frame) = dump.frames.get(&i) {
            let img_buf =
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, frame.data.to_vec())
                    .unwrap();
            let mut img = DynamicImage::ImageRgba8(img_buf);

            if show_timestamp {
                draw_timestamp(&mut img, frame.timestamp, &font_settings);
            }

            let img = img.resize(
                FRAME_DIMENSION,
                FRAME_DIMENSION,
                imageops::FilterType::Lanczos3,
            );
            frames.push(img);
        }
    }

    Ok(frames)
}

fn frames_to_image<P: AsRef<Path>>(
    dump: &VideoDump,
    video_name: &str,
    save_path: P,
    show_timestamp: bool,
) -> anyhow::Result<Vec<String>> {
    let mut image_paths = Vec::with_capacity(dump.nframes);

    let width = dump.width;
    let height = dump.height;
    let font_settings = get_font_settings(width as i32, height as i32, 0.7)?;

    for i in 0..dump.nframes {
        if let Some(frame) = dump.frames.get(&i) {
            let image_name = save_path
                .as_ref()
                .join(format!("{}-{}.jpeg", i, video_name));
            let img_buf =
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, frame.data.to_vec())
                    .unwrap();
            let mut img = DynamicImage::ImageRgba8(img_buf);

            if show_timestamp {
                draw_timestamp(&mut img, frame.timestamp, &font_settings);
            }

            img.save(&image_name).context(format!(
                "failed to save image {}",
                save_path.as_ref().display()
            ))?;

            image_paths.push(image_name.to_str().unwrap().to_string());
        }
    }

    Ok(image_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_thumbnail() {
        let filename = "../data/video/vid.mp4";
        let video_thumb = get_thumbnail(filename).expect("Failed to get thumbnail.");

        // Assert
        assert_ne!(video_thumb.data, None);
    }

    #[test]
    fn test_video_frame_dump() {
        let filename = "../data/video/vid.mp4";
        let video_dump = frame_dump(filename, 36).expect("Failed to dump frame.");

        // Assert
        assert_eq!(video_dump.nframes, 36);
    }

    #[test]
    fn test_video_frame_dump_by_time() {
        let filename = "../data/video/vid.mp4";
        let video_dump =
            frame_dump_by_time(filename, 3.0, 10.0, 15).expect("Failed to dump frame.");

        // Assert
        assert_eq!(video_dump.nframes, 105);
    }

    #[test]
    fn test_video_frame_dump_error() {
        let filename = "../data/video/vid.mp4";
        let is_error = frame_dump(filename, 400).is_err();

        // Assert
        assert_eq!(is_error, true);
    }

    #[test]
    fn test_video_concat_frames() {
        let filename = "../data/video/vid.mp4";
        let cols = 6;
        let rows: usize = 6;
        let nframes = cols * rows;

        let dump = frame_dump(filename, nframes).expect("Failed to dump frame.");
        let _ = concat_frames(dump, cols, rows, true).expect("Failed to concat frames.");

        // Assert
        assert!(true);
    }
}
