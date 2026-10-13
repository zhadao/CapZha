use cap_enc_gif::{GifEncoderWrapper, GifQuality};
use ffmpeg::{codec, format, frame, media, software};
use std::path::Path;

const MAX_GIF_SECONDS: f64 = 60.0;
const INSTANT_GIF_FPS: u32 = 12;

pub fn convert_instant_gif(input: &Path, output: &Path) -> Result<(), String> {
    convert_mp4_to_gif(input, output, INSTANT_GIF_FPS, GifQuality::default())
}

pub fn convert_mp4_to_gif(
    input: &Path,
    output: &Path,
    fps: u32,
    quality: GifQuality,
) -> Result<(), String> {
    let fps = fps.max(1);
    let frame_interval = 1.0 / fps as f64;

    let mut ictx = format::input(input).map_err(|e| format!("Failed to open input: {e}"))?;

    let stream_index;
    let time_base;
    let width;
    let height;
    let mut decoder;

    {
        let stream = ictx
            .streams()
            .best(media::Type::Video)
            .ok_or_else(|| "No video stream found".to_string())?;

        stream_index = stream.index();
        time_base = stream.time_base();

        decoder = codec::Context::from_parameters(stream.parameters())
            .map_err(|e| format!("Failed to create decoder context: {e}"))?
            .decoder()
            .video()
            .map_err(|e| format!("Failed to create video decoder: {e}"))?;

        decoder.set_time_base(time_base);

        width = decoder.width();
        height = decoder.height();
    }

    if width == 0 || height == 0 {
        return Err("Invalid video dimensions".to_string());
    }

    let mut scaler = software::converter((width, height), decoder.format(), format::Pixel::RGBA)
        .map_err(|e| format!("Failed to create frame converter: {e}"))?;

    let mut encoder = GifEncoderWrapper::new_with_quality(output, width, height, fps, quality)
        .map_err(|e| format!("Failed to create GIF encoder: {e}"))?;

    let mut rgba = frame::Video::new(format::Pixel::RGBA, width, height);
    let mut decoded = frame::Video::empty();
    let mut next_frame_time = 0.0f64;
    let mut decoded_index = 0u64;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }

        if decoder.send_packet(&packet).is_err() {
            continue;
        }

        while decoder.receive_frame(&mut decoded).is_ok() {
            let seconds = frame_seconds(&decoded, time_base, decoded_index, frame_interval);
            decoded_index += 1;

            if seconds + 1e-6 >= next_frame_time {
                if next_frame_time > MAX_GIF_SECONDS {
                    tracing::warn!(
                        "GIF conversion truncated at {MAX_GIF_SECONDS}s for '{}'",
                        input.display()
                    );
                    encoder
                        .finish()
                        .map_err(|e| format!("Failed to finish GIF: {e}"))?;
                    return Ok(());
                }

                write_frame(&mut scaler, &mut rgba, &mut encoder, &decoded)?;
                next_frame_time += frame_interval;
            }
        }
    }

    let _ = decoder.send_eof();

    while decoder.receive_frame(&mut decoded).is_ok() {
        let seconds = frame_seconds(&decoded, time_base, decoded_index, frame_interval);
        decoded_index += 1;

        if seconds + 1e-6 >= next_frame_time {
            write_frame(&mut scaler, &mut rgba, &mut encoder, &decoded)?;
            next_frame_time += frame_interval;
        }
    }

    encoder
        .finish()
        .map_err(|e| format!("Failed to finish GIF: {e}"))?;

    Ok(())
}

fn frame_seconds(
    frame: &frame::Video,
    time_base: ffmpeg::Rational,
    decoded_index: u64,
    frame_interval: f64,
) -> f64 {
    match frame.pts() {
        Some(pts) => {
            pts as f64 * time_base.numerator() as f64 / time_base.denominator().max(1) as f64
        }
        None => decoded_index as f64 * frame_interval,
    }
}

fn write_frame(
    scaler: &mut software::scaling::Context,
    rgba: &mut frame::Video,
    encoder: &mut GifEncoderWrapper,
    decoded: &frame::Video,
) -> Result<(), String> {
    scaler
        .run(decoded, rgba)
        .map_err(|e| format!("Failed to convert frame: {e}"))?;

    encoder
        .add_frame(rgba.data(0), rgba.stride(0))
        .map_err(|e| format!("Failed to add GIF frame: {e}"))
}
