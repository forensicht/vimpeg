use core_vimpeg;

pub const ZOOM_SIZE: i32 = 32;
pub const THUMBNAIL_SIZE: i32 = 160;

#[derive(Debug, Default, Clone)]
pub struct Video {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub duration: String,
    pub rate: String,
    pub nb_frames: i64,
    pub width: u32,
    pub height: u32,
    pub is_selected: bool,
    pub thumbnail_size: i32,
}

impl From<&core_vimpeg::video::Video> for Video {
    fn from(value: &core_vimpeg::video::Video) -> Self {
        let seconds = value.duration as u64 % 60;
        let minutes = (value.duration as u64 / 60) % 60;
        let hours = (value.duration as u64 / 60) / 60;

        Self {
            name: value.name.to_owned(),
            path: value.path.to_owned(),
            size: value.size,
            duration: format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
            rate: format!("{:.2} fps", value.rate),
            nb_frames: value.nb_frames,
            width: value.width,
            height: value.height,
            is_selected: false,
            thumbnail_size: THUMBNAIL_SIZE,
        }
    }
}

#[derive(Debug)]
pub struct VideoFilter {
    pub search_entry: Option<String>,
    pub is_size_0: bool,
    pub is_size_30: bool,
    pub is_size_100: bool,
    pub is_size_500: bool,
    pub is_size_greater_500: bool,
}

impl Default for VideoFilter {
    fn default() -> Self {
        Self {
            search_entry: None,
            is_size_0: true,
            is_size_30: true,
            is_size_100: true,
            is_size_500: true,
            is_size_greater_500: true,
        }
    }
}
