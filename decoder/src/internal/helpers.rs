use super::super::algo;
use super::super::algo::DecoderResult;

use bytes::*;

pub fn get_ext(filename: &str) -> &str {
    let ext = std::path::Path::new(filename)
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    ext
}

pub fn dec_init(
    infile: Bytes,
    skip_noop: bool,
    ext: &str,
) -> DecoderResult<Box<dyn algo::Decoder>> {
    let all_dec = algo::get_static_decoder_map().get(ext, skip_noop);
    if all_dec.is_empty() {
        return Err(format!("No decoder available for extension: {}", ext).into());
    }
    let dec_params = algo::DecoderParams {
        buffer: infile,
        extension: ext.to_string(),
    };

    let mut dec = None;
    let mut errors = Vec::new();
    for dec_type in all_dec.iter() {
        let mut decoder = dec_type.get_decoder().new_decoder(&dec_params);
        if let Err(e) = decoder.validate() {
            errors.push(format!("{}", e));
        } else {
            dec = Some(decoder);
            break;
        }
    }
    dec.ok_or_else(|| errors.join(", ").into())
}

fn write_id3_tags(
    infile: Bytes,
    metadata: Option<Box<dyn algo::AudioMeta>>,
    cover: Option<Bytes>,
) -> DecoderResult<Bytes> {
    // This is for id3.
    use id3::TagLike;
    if metadata.is_none() && cover.is_none() {
        return Ok(infile);
    }
    let mut infile_reader = std::io::Cursor::new(infile.clone());
    let mut tags = id3::Tag::read_from2(&mut infile_reader)?;

    tags.remove_comment(None, None);
    if let Some(meta) = metadata {
        tags.set_title(meta.get_title());
        tags.set_artist(meta.get_artists().join(","));
        tags.set_album(meta.get_album());
    }
    if let Some(cover) = cover {
        tags.remove_all_pictures();
        tags.add_frame(id3::frame::Picture {
            mime_type: super::sniff::image_mime(&cover).unwrap_or_default(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: String::new(),
            data: cover.to_vec(),
        });
    }
    let out_vec: Vec<u8> = infile.clone().to_vec();
    let mut writer = std::io::Cursor::new(out_vec);
    tags.write_to_file(&mut writer, id3::Version::Id3v24)?;
    Ok(writer.into_inner().into())
}

pub fn get_result(mut dec: Box<dyn algo::Decoder>, filename: Option<&str>) -> DecoderResult<Bytes> {
    let decoded_bytes = dec.decode_bytes()?;
    match super::sniff::audio_extension_with_fallback(&decoded_bytes, String::new()).as_str() {
        ".mp3" | ".wav" => {
            let cover = match dec.get_cover_image() {
                Some(Ok(c)) => Some(c),
                Some(Err(e)) => return Err(e),
                None => None,
            };
            let mut metadata = match dec.get_audio_meta() {
                Some(Ok(meta)) => Some(meta),
                Some(Err(e)) => return Err(e),
                None => None,
            };
            if metadata.is_none() && filename.is_some() {
                metadata = Some(Box::new(
                    super::super::algo::common::meta::parse_filename_meta(filename.unwrap()),
                ));
            }
            write_id3_tags(decoded_bytes.freeze(), metadata, cover)
        }
        _ => Ok(decoded_bytes.freeze()),
    }
}
