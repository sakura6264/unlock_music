#[derive(Clone, Default, PartialEq, Debug)]
pub struct FilenameMeta {
    pub title: String,
    pub artist: Vec<String>,
    pub album: String,
}

impl FilenameMeta {
    pub fn new(title: String, artist: Vec<String>, album: String) -> Self {
        FilenameMeta {
            title,
            artist,
            album,
        }
    }
}
impl super::interface::AudioMeta for FilenameMeta {
    fn get_artists(&self) -> Vec<String> {
        self.artist.clone()
    }
    fn get_album(&self) -> String {
        self.album.clone()
    }
    fn get_title(&self) -> String {
        self.title.clone()
    }
    fn manual_clone(&self) -> Box<dyn super::interface::AudioMeta> {
        Box::new(self.clone())
    }
}

pub fn parse_filename_meta(filename: &str) -> FilenameMeta {
    let part_name = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    // trim them after split
    let items = part_name
        .split_terminator(&['-', '_', ','])
        .map(|v| v.trim())
        .collect::<Vec<_>>();
    let mut ret = FilenameMeta::default();
    match items.len() {
        0 => {}
        1 => ret.title = String::from(items[0]),
        _ => {
            ret.title = String::from(items[items.len() - 1]);
            for v in items.iter().take(items.len() - 1) {
                ret.artist.push(String::from(*v));
            }
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_filename_meta() {
        let tests = vec![
            (
                "test1",
                FilenameMeta {
                    title: String::from("test1"),
                    ..Default::default()
                },
            ),
            (
                "周杰伦 - 晴天.flac",
                FilenameMeta {
                    title: String::from("晴天"),
                    artist: vec![String::from("周杰伦")],
                    ..Default::default()
                },
            ),
            (
                "Alan Walker _ Iselin Solheim - Sing Me to Sleep.flac",
                FilenameMeta {
                    title: String::from("Sing Me to Sleep"),
                    artist: vec![String::from("Alan Walker"), String::from("Iselin Solheim")],
                    ..Default::default()
                },
            ),
            (
                "Christopher,Madcon - Limousine.flac",
                FilenameMeta {
                    title: String::from("Limousine"),
                    artist: vec![String::from("Christopher"), String::from("Madcon")],
                    ..Default::default()
                },
            ),
        ];
        for (filename, expect) in tests {
            assert_eq!(parse_filename_meta(filename), expect);
        }
    }
}
