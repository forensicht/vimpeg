pub const ZOOM_SIZE: i32 = 32;
pub const THUMBNAIL_SIZE: i32 = 160;

#[derive(Debug, Default, Clone)]
pub struct Image {
    pub name: String,
    pub path: String,
    pub thumbnail_size: i32,
}
