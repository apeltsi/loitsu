use super::texture_asset::TextureFormat;

#[cfg_attr(
    feature = "json_preference_parse",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, bitcode::Encode, bitcode::Decode)]
pub enum AssetMeta {
    None, // used mainly in asset-gen to indicate that the asset doesn't have any form of metadata
    TextureMeta(TextureMetadata),
}

#[cfg_attr(
    feature = "json_preference_parse",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, bitcode::Encode, bitcode::Decode)]
pub struct TextureMetadata {
    pub resolution_multiplier: Option<f32>,
    pub include_alpha: Option<bool>,
    pub uv: Option<(f32, f32, f32, f32)>,
    pub target: String,
}

impl TextureMetadata {
    pub fn get_resolution_multiplier(&self) -> f32 {
        self.resolution_multiplier.unwrap_or(1.0)
    }

    pub fn get_include_alpha(&self) -> bool {
        self.include_alpha.unwrap_or(false)
    }

    pub fn get_target(&self) -> &str {
        &self.target
    }

    pub fn get_uv(&self) -> (f32, f32, f32, f32) {
        self.uv.unwrap_or((0.0, 0.0, 1.0, 1.0))
    }

    pub fn get_format(&self) -> TextureFormat {
        if let Some(include_alpha) = self.include_alpha {
            if include_alpha {
                return TextureFormat::RGBA8;
            } else {
                return TextureFormat::RGB8;
            }
        } else {
            TextureFormat::RGB8
        }
    }
}
