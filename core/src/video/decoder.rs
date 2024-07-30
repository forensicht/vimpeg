use std::{collections::HashMap, path::Path};

use anyhow::{self, Context};
use ffmpeg::{format, media::Type, software::scaling, util::frame};
use ffmpeg_next as ffmpeg;
use image::{imageops, DynamicImage, GenericImage, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use rust_embed::RustEmbed;

const FRAME_DIMENSION: u32 = 250;

#[derive(RustEmbed)]
#[folder = "../data/fonts/"]
struct Fonts;

#[derive(Debug)]
pub struct VideoFrame {
    pub data: Vec<u8>,
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
    pub data: Option<Vec<u8>>,
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
        match decoder.receive_frame(&mut decoded) {
            Ok(()) => break decoded,
            _ => {}
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
        data: Some(scaled_frame.data(0).to_owned()),
    };

    Ok(video_thumb)
}

pub fn dump_video_frames_into_image<P: AsRef<Path>>(
    video_path: P,
    image_path: P,
    cols: usize,
    rows: usize,
) -> anyhow::Result<()> {
    let nframes = rows * cols;
    let dump = dump_frame(video_path, nframes)?;
    concat_frames(dump, image_path, cols, rows)?;
    Ok(())
}

pub fn dump_frame<P: AsRef<Path>>(video_path: P, nframes: usize) -> anyhow::Result<VideoDump> {
    ffmpeg::init()?;

    let mut options = ffmpeg::Dictionary::new();
    options.set("framerate", "10");

    let mut input_format_context = ffmpeg::format::input_with_dictionary(&video_path, options)?;

    // shows a dump of the video
    // let video_path = video_path.as_os_str().to_str().unwrap();
    // format::context::input::dump(&input_format_context, 0, Some(video_path));

    let (video_stream_index, frame_rate, time_base, mut decoder) = {
        let stream = input_format_context
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let total_frames = stream.frames();
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

    let mut video_dump = VideoDump::default();
    video_dump.width = decoder.width();
    video_dump.height = decoder.height();

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
                    let video_frame = VideoFrame { data, timestamp };
                    video_dump.frames.insert(frame_index, video_frame);

                    frame_index += 1;
                    processed_frames = 0;
                }

                processed_frames += 1;
            }

            Ok(())
        };

    // decoder.skip_frame(ffmpeg::codec::discard::Discard::All);

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

fn concat_frames<P: AsRef<Path>>(
    dump: VideoDump,
    image_path: P,
    cols: usize,
    rows: usize,
) -> anyhow::Result<()> {
    let frames = frames_to_image(&dump)?;
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

    imgbuf.save(&image_path).context(format!(
        "failed to save image {}",
        image_path.as_ref().display()
    ))?;

    Ok(())
}

fn frames_to_image(dump: &VideoDump) -> anyhow::Result<Vec<DynamicImage>> {
    let width = dump.width;
    let height = dump.height;
    let mut frames = vec![];
    frames.reserve(dump.nframes);

    // font settings
    let (font, font_scale, font_x, font_y) = {
        let font = Fonts::get("DejaVuSans.ttf").unwrap();
        let font = font.data.to_vec();
        // let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
        let font = rusttype::Font::try_from_vec(font).unwrap();
        let font_height = if height > width {
            (24.0 * width as f32) / 360 as f32
        } else {
            (14.0 * width as f32) / 360 as f32
        };
        let font_scale = rusttype::Scale {
            x: font_height * 2.0,
            y: font_height,
        };
        let font_size = text_size(font_scale, &font, "77:77:77.777");
        let font_x = width as i32 - (font_size.0 + 10);
        let font_y = height as i32 - (font_size.1 + 10);

        (font, font_scale, font_x, font_y)
    };

    for i in 0..dump.nframes {
        if let Some(frame) = dump.frames.get(&i) {
            let img_buf =
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, frame.data.to_owned())
                    .unwrap();
            let mut img = DynamicImage::ImageRgba8(img_buf);

            let timestamp = frame.timestamp;
            let seconds = timestamp % 60.0;
            let minutes = ((timestamp / 60.0) % 60.0) as u32;
            let hours = ((timestamp / 60.0) / 60.0) as u32;

            // put timestamp on image
            let text = format!("{:0>2}:{:0>2}:{:0>6.3}", hours, minutes, seconds);
            draw_text_mut(
                &mut img,
                image::Rgba([255u8, 111u8, 0u8, 255u8]),
                font_x,
                font_y,
                font_scale,
                &font,
                text.as_str(),
            );
            // ----------------------------

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

#[cfg(test)]
mod tests {
    use super::*;
    use image;

    #[test]
    fn test_video_thumbnail() {
        let filename = "D:\\video\\vid00.mp4";
        match get_thumbnail(filename) {
            Ok(thumb) => {
                save_file_thumb(&thumb).expect("error saving file thumb");
                assert!(true);
            }
            Err(err) => assert!(false, "{err}"),
        }
    }

    #[test]
    fn test_dump_frames_to_image() {
        let video_path = "D:\\video\\vid00.mp4";
        let image_path = "D:\\video\\frames\\vid00.jpeg";
        let cols = 6;
        let rows: usize = 6;

        match dump_video_frames_into_image(video_path, image_path, cols, rows) {
            Err(err) => assert!(false, "{err}"),
            _ => assert!(true),
        }
    }

    #[test]
    fn test_video_dump_frame() {
        let filename = "D:\\video\\vid00.mp4";
        match dump_frame(filename, 36) {
            Ok(dump) => {
                println!("Frames: {}", dump.nframes);
                save_file_dump_frame(&dump).expect("error saving file dump frame");
                assert!(true);
            }
            Err(err) => assert!(false, "{err}"),
        }
    }

    #[test]
    fn test_video_dump_frame_error() {
        let filename = "D:\\video\\vid00.mp4";
        match dump_frame(filename, 376) {
            Ok(_) => assert!(true),
            Err(err) => {
                eprintln!("{err}");
                assert!(true);
            }
        }
    }

    #[test]
    fn test_video_concat_frames() {
        let filename = "D:\\video\\vid00.mp4";
        let dst_path = "D:\\video\\frames\\vid00.jpeg";
        let cols = 6;
        let rows: usize = 6;
        let nframes = cols * rows;

        if let Ok(dump) = dump_frame(filename, nframes) {
            match concat_frames(dump, dst_path, cols, rows) {
                Err(err) => assert!(false, "{err}"),
                _ => (),
            }
        } else {
            assert!(false);
        }
    }

    fn save_file_dump_frame(dump: &VideoDump) -> anyhow::Result<()> {
        let width = dump.width;
        let height = dump.height;
        let nframes = dump.nframes;
        let frames = &dump.frames;

        println!("nframes: {} - len: {}", nframes, frames.len());

        for index in 0..nframes {
            let path = format!("D:\\video\\frames\\vid{}.jpeg", index);
            if let Some(buffer) = frames.get(&index) {
                image::save_buffer(path, &buffer.data, width, height, image::ColorType::Rgba8)?;
            }
        }

        Ok(())
    }

    fn save_file_thumb(thumb: &VideoThumb) -> anyhow::Result<()> {
        let path = "D:\\video\\vid_thumb.jpeg";
        let width = thumb.width;
        let height = thumb.height;

        if let Some(buffer) = thumb.data.as_ref() {
            image::save_buffer(path, buffer, width, height, image::ColorType::Rgba8)?;
        }

        Ok(())
    }
}
