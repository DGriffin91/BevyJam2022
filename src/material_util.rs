use std::num::NonZeroU8;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, FilterMode, Sampler, SamplerDescriptor, TextureAspect, TextureFormat,
            TextureView, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};

pub fn get_sampler(render_device: &mut Res<RenderDevice>, address_mode: AddressMode) -> Sampler {
    let sampler_descriptor = SamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        mipmap_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        anisotropy_clamp: NonZeroU8::new(16),
        ..Default::default()
    };

    render_device.create_sampler(&sampler_descriptor)
}

pub fn get_image_texture_cube(
    gpu_images: &RenderAssets<Image>,
    texture: &Handle<Image>,
) -> Option<TextureView> {
    gpu_images.get(texture).map(|gpu_image| {
        gpu_image.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Rgba8UnormSrgb),
            dimension: Some(TextureViewDimension::Cube),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        })
    })
}

#[macro_export]
macro_rules! texture_view {
    ( $texture:expr, $gpu_images:expr, $material:expr ) => {{
        if let Some(result) = $gpu_images.get($texture) {
            result.texture_view.clone()
        } else {
            return Err(PrepareAssetError::RetryNextUpdate($material.clone()));
        }
    }};
}

#[macro_export]
macro_rules! cubetex_view {
    ( $texture:expr, $gpu_images:expr, $material:expr ) => {{
        if let Some(texture_view) =
            $crate::material_util::get_image_texture_cube($gpu_images, $texture)
        {
            texture_view
        } else {
            return Err(PrepareAssetError::RetryNextUpdate($material));
        }
    }};
}

#[macro_export]
macro_rules! texture_binding {
    ( $view:expr, $binding:expr ) => {{
        BindGroupEntry {
            binding: $binding,
            resource: bevy::render::render_resource::BindingResource::TextureView(&$view),
        }
    }};
}

#[macro_export]
macro_rules! sampler_binding {
    ( $sampler:expr, $binding:expr ) => {{
        BindGroupEntry {
            binding: $binding,
            resource: bevy::render::render_resource::BindingResource::Sampler(&$sampler),
        }
    }};
}

#[macro_export]
macro_rules! cubetex_group_layout {
    ( $binding:expr ) => {{
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                sample_type: bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                },
                view_dimension: bevy::render::render_resource::TextureViewDimension::Cube,
            },
            count: None,
        }
    }};
}

#[macro_export]
macro_rules! texture_group_layout {
    ( $binding:expr ) => {{
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }
    }};
}

#[macro_export]
macro_rules! sampler_group_layout {
    ( $binding:expr ) => {{
        BindGroupLayoutEntry {
            binding: $binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        }
    }};
}
