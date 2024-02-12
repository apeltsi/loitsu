use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{asset::Asset, asset_reference::AssetReference, AssetError};

pub struct TextureAsset {
    name: String,
    rgba: image::RgbaImage,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    dimensions: (u32, u32),
}

impl TextureAsset {
    pub fn get_texture(&self) -> Option<&wgpu::Texture> {
        self.texture.as_ref()
    }

    pub fn get_texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    pub fn from_bytes(bytes: Vec<u8>, name: &str) -> TextureAsset {
        let image = image::load_from_memory(&bytes).unwrap();
        let image = image.to_rgba8();
        let dimensions = image.dimensions();
        TextureAsset {
            name: name.to_string(),
            rgba: image,
            dimensions,
            texture: None,
            texture_view: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), AssetError> {
        if self.texture.is_some() {
            return Ok(());
        }
        #[cfg(feature = "editor")]
        #[cfg(target = "wasm32")]
        crate::web::add_editor_loading_task("Loading assets");
        let texture_size = wgpu::Extent3d {
            width: self.dimensions.0,
            height: self.dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(self.name.as_str()),
            view_formats: &[],
        });
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
            self.texture_view = Some(
                self.texture
                    .as_ref()
                    .expect(
                        "Couldn't create textureview from texture. The texture might not be valid",
                    )
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
        }

        #[cfg(feature = "editor")]
        #[cfg(target = "wasm32")]
        crate::web::remove_editor_loading_task("Loading assets");
        Ok(())
    }
}

#[derive(Clone)]
pub struct TextureMeta {
    target: String,
    texture: Arc<Mutex<AssetReference>>,
    uv: (f32, f32, f32, f32),
    format: TextureFormat,
}

impl TextureMeta {
    pub fn new(target: &str, uv: (f32, f32, f32, f32), format: TextureFormat) -> TextureMeta {
        TextureMeta {
            target: target.to_string(),
            texture: Arc::new(Mutex::new(AssetReference::new(Arc::new(Mutex::new(
                Asset::None,
            ))))),
            uv,
            format,
        }
    }

    pub fn initialize(
        &mut self,
        assets: &HashMap<String, Arc<Mutex<AssetReference>>>,
    ) -> Result<(), AssetError> {
        self.texture = assets.get(&self.target).expect("TextureMeta target not found! The texture meta's target should be in the same shard as the meta").clone();
        Ok(())
    }

    pub fn get_texture(&self) -> Result<Arc<Mutex<Asset>>, AssetError> {
        let texture = self.texture.lock().unwrap();
        Ok(texture.get_asset())
    }

    pub fn get_texture_version(&self) -> u32 {
        let texture = self.texture.lock().unwrap();
        return texture.get_version();
    }

    pub fn get_uv(&self) -> (f32, f32, f32, f32) {
        self.uv
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    RGBA8,
    RGBA16,
    RGBA32,
    RGB8,
    RGB16,
    RGB32,
}

// i know this looks bad just lemme cook
// (really feel like this will be a problem in the future)
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TextureAsset {}
#[cfg(target_arch = "wasm32")]
unsafe impl Send for TextureAsset {}
