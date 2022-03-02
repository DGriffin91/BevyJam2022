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
use bevy_egui::egui;

use super::custom_material::{slider, MaterialSetProp};

#[derive(Debug, Clone, Copy, Default, AsStd140)]
pub struct OrbProperties {
    pub orb: MaterialSetProp,
    pub speed: Vec3,
    pub color_tint: Vec3,
    pub radius: f32,
    pub inner_radius: f32,
    pub alpha: f32,
    pub time: f32,
}

impl OrbProperties {
    #[allow(dead_code)]
    pub fn build_ui(&mut self, ui: &mut egui::Ui) {
        if ui.button("Debug Print").clicked() {
            dbg!(&self);
        }
        slider(ui, &mut self.color_tint.x, 0.0..=1.0, "r");
        slider(ui, &mut self.color_tint.y, 0.0..=1.0, "g");
        slider(ui, &mut self.color_tint.z, 0.0..=1.0, "b");
        slider(ui, &mut self.speed.x, -10.0..=10.0, "speed x");
        slider(ui, &mut self.speed.y, -10.0..=10.0, "speed y");
        slider(ui, &mut self.speed.z, -10.0..=10.0, "speed z");
        slider(ui, &mut self.radius, -2.0..=2.0, "radius");
        slider(ui, &mut self.inner_radius, -2.0..=2.0, "inner_radius");
        slider(ui, &mut self.alpha, -2.0..=2.0, "alpha");
        self.orb.build_ui(ui, "orb");
        ui.label("-------------");
    }
}

pub fn update_orb_material_time(time: Res<Time>, mut orb_materials: ResMut<Assets<OrbMaterial>>) {
    for (_, mat) in orb_materials.iter_mut() {
        mat.material_properties.time = time.seconds_since_startup() as f32;
    }
}

// This is the struct that will be passed to your shader
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "1ef7c361-1344-4729-790e-117d82b126c1"]
pub struct OrbMaterial {
    pub material_properties: OrbProperties,
    pub noise_texture: Option<Handle<Image>>,
}

#[derive(Clone)]
pub struct GpuOrbMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

fn get_custom_sampler(render_device: &mut Res<RenderDevice>) -> Sampler {
    let sampler_descriptor = SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        mipmap_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        anisotropy_clamp: NonZeroU8::new(16),
        ..Default::default()
    };

    render_device.create_sampler(&sampler_descriptor)
}

// The implementation of [`Material`] needs this impl to work properly.
impl RenderAsset for OrbMaterial {
    type ExtractedAsset = OrbMaterial;
    type PreparedAsset = GpuOrbMaterial;
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
        let material_properties = &material.material_properties;
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: material_properties.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let (orb_texture_view, _sampler) = if let Some(result) = material_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.noise_texture)
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
                    resource: BindingResource::TextureView(orb_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(sampler1),
                },
            ],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuOrbMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for OrbMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        let r = Some(asset_server.load("shaders/orb_material.wgsl"));
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
                        min_binding_size: BufferSize::new(0), //BufferSize::new(Vec4::std140_size_static() as u64),
                    },
                    count: None,
                },
                // Noise Texture
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
                // Noise Texture Sampler
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

    #[inline]
    fn alpha_mode(_render_asset: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        AlphaMode::Blend
    }
}
