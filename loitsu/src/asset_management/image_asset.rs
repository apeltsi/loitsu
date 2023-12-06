use super::AssetError;

pub struct ImageAsset {
    name: String,
    rgba: image::RgbaImage,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    dimensions: (u32, u32)
}

impl ImageAsset {
    pub fn get_texture(&self) -> Option<&wgpu::Texture> {
        self.texture.as_ref()
    }

    pub fn get_texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }
    pub fn from_bytes(bytes: Vec<u8>, name: &str) -> ImageAsset {
        let image = image::load_from_memory(&bytes).unwrap();
        let image = image.to_rgba8();
        let dimensions = image.dimensions();
        ImageAsset {
            name: name.to_string(),
            rgba: image,
            dimensions,
            texture: None,
            texture_view: None 
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn initialize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), AssetError> {
        if self.texture.is_some() {
            return Ok(());
        }
        let texture_size = wgpu::Extent3d {
            width: self.dimensions.0,
            height: self.dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some(self.name.as_str()),
                view_formats: &[],
            }
        );
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &self.rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.dimensions.0),
                rows_per_image: Some(self.dimensions.1),
            },
            texture_size,
        );
        {
            self.texture = Some(texture);
        }
        {
            self.texture_view = Some(self.texture.as_ref()
                                     .expect("Couldn't create textureview from texture. The texture might not be valid")
                                     .create_view(&wgpu::TextureViewDescriptor::default()));
        }
        Ok(())
    }
}

// i know this looks bad just lemme cook
// (really feel like this will be a problem in the future)
#[cfg(target_arch = "wasm32")] 
unsafe impl Sync for ImageAsset {}
#[cfg(target_arch = "wasm32")]
unsafe impl Send for ImageAsset {}
