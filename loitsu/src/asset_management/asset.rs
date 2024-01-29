pub use super::image_asset::ImageAsset;
use super::AssetError;
pub enum Asset {
    None, // used
    Image(ImageAsset),
}

impl Asset {
    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), AssetError> {
        match self {
            Asset::Image(image) => image.initialize(device, queue),
            _ => Ok(()),
        }
    }
}

pub fn image_from_bytes(bytes: Vec<u8>, name: &str) -> Asset {
    Asset::Image(ImageAsset::from_bytes(bytes, name))
}
