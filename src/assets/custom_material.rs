use std::ops::RangeInclusive;

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
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferSize, BufferUsages, SamplerBindingType,
            ShaderStages, TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};
use bevy_egui::egui;

use crate::{
    sampler_binding, sampler_group_layout, texture_binding, texture_group_layout, texture_view,
};

use super::material_util::get_sampler;

#[derive(Debug, Clone, Copy, AsStd140)]
pub struct MaterialSetProp {
    pub scale: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub blend: f32,
}

#[allow(dead_code)]
pub fn log_slider<Num: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    value: &mut Num,
    range: RangeInclusive<Num>,
    text: &str,
) {
    ui.add(egui::Slider::new(value, range).logarithmic(true).text(text));
}

#[allow(dead_code)]
pub fn slider<Num: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    value: &mut Num,
    range: RangeInclusive<Num>,
    text: &str,
) {
    ui.add(egui::Slider::new(value, range).text(text));
}
impl MaterialSetProp {
    #[allow(dead_code)]
    pub fn build_ui(&mut self, ui: &mut egui::Ui, label: &str) {
        ui.label(label);
        log_slider(ui, &mut self.scale, 0.0..=100.0, "scale");
        log_slider(ui, &mut self.contrast, 0.0..=10.0, "contrast");
        log_slider(ui, &mut self.brightness, 0.0..=40.0, "brightness");
        log_slider(ui, &mut self.blend, 0.0..=1.0, "blend");
    }
}

#[derive(Debug, Clone, Copy, AsStd140)]
pub struct MaterialProperties {
    pub lightmap: MaterialSetProp,
    pub base_a: MaterialSetProp,
    pub base_b: MaterialSetProp,
    pub vary_a: MaterialSetProp,
    pub vary_b: MaterialSetProp,
    pub reflection: MaterialSetProp,
    pub walls: MaterialSetProp,
    pub reflection_mask: MaterialSetProp,
    pub mist: MaterialSetProp,
    pub directional_light_blend: f32,
    pub flags: u32,
    // pub directional_light_color: Vec3,
}

impl MaterialProperties {
    pub fn build_ui(&mut self, ui: &mut egui::Ui) {
        if ui.button("Debug Print").clicked() {
            dbg!(&self);
        }
        self.lightmap.build_ui(ui, "lightmap");
        self.base_a.build_ui(ui, "base_a");
        self.base_b.build_ui(ui, "base_b");
        self.vary_a.build_ui(ui, "vary_a");
        self.vary_b.build_ui(ui, "vary_b");
        self.reflection.build_ui(ui, "reflection");
        self.reflection_mask.build_ui(ui, "reflection_mask");
        self.walls.build_ui(ui, "walls");
        self.mist.build_ui(ui, "mist");
        ui.label("-------------");
        ui.add(
            egui::Slider::new(&mut self.directional_light_blend, 0.0..=5.0)
                .text("directional_light_blend"),
        );
    }
}

// #[derive(Debug, Clone)]
// pub struct MaterialTexture {
//     pub handle: Handle<Image>,
// }

// impl MaterialTexture {
// pub fn build_ui(&mut self, ui: &mut egui::Ui, asset_server: &Res<AssetServer>) {
//     ui.label(&self.name);
//     ui.horizontal(|ui| {
//         ui.text_edit_singleline(&mut self.path);
//         if ui.button("LOAD").clicked() {
//             self.handle = asset_server.load(&self.path);
//         }
//     });
// }
// }

// This is the struct that will be passed to your shader
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4ee9c361-1124-4113-890e-197d82b00123"]
pub struct CustomMaterial {
    pub material_properties: MaterialProperties,
    pub textures: [Handle<Image>; 5],
}

// impl CustomMaterial {
//     pub fn build_ui(&mut self, ui: &mut egui::Ui, asset_server: &Res<AssetServer>) {
//         //self.material_properties.build_ui(ui);
//         ui.label("CustomMaterial");
//         if ui.button("Print Paths").clicked() {
//             for texture in &self.textures {
//                 println!("{}", texture.path);
//             }
//         }
//         for texture in &mut self.textures {
//             texture.build_ui(ui, asset_server)
//         }
//     }
// }

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct CustomMaterialFlags: u32 {
        const SHADOWS                      = (1 << 0);
        const POTATO                       = (1 << 1);
        //const METALLIC_ROUGHNESS_TEXTURE = (1 << 2);
        //const OCCLUSION_TEXTURE          = (1 << 3);
        //const DOUBLE_SIDED               = (1 << 4);
        //const UNLIT                      = (1 << 5);
        //const ALPHA_MODE_OPAQUE          = (1 << 6);
        //const ALPHA_MODE_MASK            = (1 << 7);
        //const ALPHA_MODE_BLEND           = (1 << 8);
        const NONE                         = 0;
        //const UNINITIALIZED              = 0xFFFF;
    }
}

#[derive(Clone)]
pub struct GpuCustomMaterial {
    _buffer: Buffer,
    _flags: u32,
    bind_group: BindGroup,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
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

        let texviews = [
            texture_view!(&material.textures[0], gpu_images, material),
            texture_view!(&material.textures[1], gpu_images, material),
            texture_view!(&material.textures[2], gpu_images, material),
            texture_view!(&material.textures[3], gpu_images, material),
            texture_view!(&material.textures[4], gpu_images, material),
        ];

        let samplers = [
            get_sampler(render_device, AddressMode::Repeat),
            get_sampler(render_device, AddressMode::Repeat),
            get_sampler(render_device, AddressMode::Repeat),
            get_sampler(render_device, AddressMode::Repeat),
            get_sampler(render_device, AddressMode::Repeat),
        ];

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                texture_binding!(texviews[0], 1),
                sampler_binding!(samplers[0], 2),
                texture_binding!(texviews[1], 3),
                sampler_binding!(samplers[1], 4),
                texture_binding!(texviews[2], 5),
                sampler_binding!(samplers[2], 6),
                texture_binding!(texviews[3], 7),
                sampler_binding!(samplers[3], 8),
                texture_binding!(texviews[4], 9),
                sampler_binding!(samplers[4], 10),
            ],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuCustomMaterial {
            _buffer: buffer,
            _flags: material_properties.flags,
            bind_group,
        })
    }
}

impl Material for CustomMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        let r = Some(asset_server.load("shaders/custom_material.wgsl"));
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
                        min_binding_size: BufferSize::new(0),
                    },
                    count: None,
                },
                texture_group_layout!(1),
                sampler_group_layout!(2),
                texture_group_layout!(3),
                sampler_group_layout!(4),
                texture_group_layout!(5),
                sampler_group_layout!(6),
                texture_group_layout!(7),
                sampler_group_layout!(8),
                texture_group_layout!(9),
                sampler_group_layout!(10),
            ],
            label: None,
        })
    }
}
