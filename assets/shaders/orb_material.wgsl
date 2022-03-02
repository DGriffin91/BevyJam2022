#import bevy_pbr::mesh_view_bind_group
#import "shaders/util.wgsl"

struct MaterialSetProp {
    scale: f32;
    contrast: f32;
    brightness: f32;
    blend: f32;
};

struct MaterialProperties {
    orb: MaterialSetProp;
    speed: vec3<f32>;
    color_tint: vec3<f32>;
    radius: f32;
    inner_radius: f32;
    time: f32;
};

[[group(1), binding(0)]]
var<uniform> ma: MaterialProperties;
[[group(1), binding(1)]]
var noise_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var noise_sampler: sampler;

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    var col = vec3<f32>(0.0,0.0,0.0);

    var N: vec3<f32> = normalize(in.world_normal);
    var V = normalize(view.world_position.xyz - in.world_position.xyz);
    let NdotV = max(dot(N, V), 0.0001);
    
    //TODO use with billboard
    //let dist = distance(V.xy, vec2<f32>(0.5));
    //let fade = 1.0 - smoothStep(0.0, ma.inner_radius, abs(ma.radius-dist));

    let fade = clamp(pow(NdotV, 7.0) * 1.0, 0.0, 1.0);

    return vec4<f32>(ma.color_tint, fade);
}
