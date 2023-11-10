pub trait Asset: Send + Sync {
    fn from_bytes(bytes: Vec<u8>, name: &str) -> Self where Self: Sized;
    fn get_name(&self) -> &str;
}

pub struct ImageAsset {
    name: String,
    image: image::RgbaImage,

}

impl Asset for ImageAsset {
    fn from_bytes(bytes: Vec<u8>, name: &str) -> ImageAsset {
        let image = image::load_from_memory(&bytes).unwrap();
        let image = image.to_rgba8();
        ImageAsset {
            name: name.to_string(),
            image,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
