#import bevy_pbr::mesh_view_bind_group
#import "shaders/util.wgsl"

struct MaterialSetProp {
    scale: f32;
    contrast: f32;
    brightness: f32;
    blend: f32;
};

struct MaterialProperties {
    shaft: MaterialSetProp;
    noise_a: MaterialSetProp;
    noise_b: MaterialSetProp;
    speed: vec3<f32>;
    color_tint: vec3<f32>;
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
    let distance = distance(view.world_position.xyz, in.world_position.xyz);
    let distance_fade = clamp(pow(distance * 0.012, 3.0), 0.0, 1.0);
    let depth_scale_var = in.world_position.z * 0.0002;

    var offset = ma.speed.xy * ma.time;
    offset = offset * in.world_normal.x * -2.0; //hack to made both sides animate in the same direction
    offset = offset + depth_scale_var * 10.0;

    let noise_b = textureSample(noise_texture, noise_sampler, in.uv * ma.noise_b.scale * vec2<f32>(1.0,20.0) + vec2<f32>(depth_scale_var, 0.0)).rgb;
    col = mix(col, col + pow(noise_b, vec3<f32>(ma.noise_b.contrast)) * ma.noise_b.brightness, ma.noise_b.blend);

    let noise_a = textureSample(noise_texture, noise_sampler, (in.uv + offset) * ma.noise_a.scale).rgb;
    col = mix(col, col * pow(noise_a, vec3<f32>(ma.noise_a.contrast)) * ma.noise_a.brightness, ma.noise_a.blend);

    var fade = in.uv.x * in.uv.y * (1.0 - in.uv.y) * (1.0 - in.uv.x);
    fade = pow(fade, ma.shaft.contrast) * ma.shaft.brightness;

    col = col * vec3<f32>(ma.color_tint.x, ma.color_tint.y, ma.color_tint.z);

    fade = fade * (col.x + col.y + col.z) * distance_fade;

    return vec4<f32>(vec3<f32>(col), fade);
}
