use std::num::NonZeroU8;

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            std140::{AsStd140, Std140},
            AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferSize, BufferUsages, FilterMode, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, TextureSampleType,
            TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};

// This is the struct that will be passed to your shader
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4ee9c361-1124-4113-890e-197d82b00321"]
pub struct EmissiveMaterial {
    pub emissive: Color,
    pub emissive_texture: Option<Handle<Image>>,
}

#[derive(Clone)]
pub struct GpuEmissiveMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

fn get_custom_sampler(render_device: &mut Res<RenderDevice>) -> Sampler {
    let mut sampler_descriptor = SamplerDescriptor::default();

    sampler_descriptor.address_mode_u = AddressMode::Repeat;
    sampler_descriptor.address_mode_v = AddressMode::Repeat;
    sampler_descriptor.mipmap_filter = FilterMode::Linear;
    sampler_descriptor.mag_filter = FilterMode::Linear;
    sampler_descriptor.min_filter = FilterMode::Linear;
    sampler_descriptor.anisotropy_clamp = NonZeroU8::new(16);

    render_device.create_sampler(&sampler_descriptor)
}

// The implementation of [`Material`] needs this impl to work properly.
impl RenderAsset for EmissiveMaterial {
    type ExtractedAsset = EmissiveMaterial;
    type PreparedAsset = GpuEmissiveMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<MaterialPipeline<Self>>,
        SRes<RenderAssets<Image>>,
    );
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, material_pipeline, gpu_images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let emissive = Vec4::from_slice(&material.emissive.as_linear_rgba_f32());
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: emissive.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let (emissive_texture_view, _sampler) = if let Some(result) = material_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.emissive_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let sampler1 = &get_custom_sampler(render_device);

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(emissive_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(sampler1),
                },
            ],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuEmissiveMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for EmissiveMaterial {
    // When creating a custom material, you need to define either a vertex shader, a fragment shader or both.
    // If you don't define one of them it will use the default mesh shader which can be found at
    // <https://github.com/bevyengine/bevy/blob/latest/crates/bevy_pbr/src/render/mesh.wgsl>

    // For this example we don't need a vertex shader
    // fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
    //     // Use the same path as the fragment shader since wgsl let's you define both shader in the same file
    //     Some(asset_server.load("shaders/custom_material.wgsl"))
    // }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        let r = Some(asset_server.load("shaders/emissive_material.wgsl"));
        asset_server.watch_for_changes().unwrap();
        r
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(Vec4::std140_size_static() as u64),
                    },
                    count: None,
                },
                // Emissive Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Emissive Texture Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        })
    }
}
