#import bevy_pbr::mesh_view_bind_group
#import "shaders/util.wgsl"

struct MaterialSetProp {
    scale: f32;
    contrast: f32;
    brightness: f32;
    blend: f32;
};

let SHADOWS: u32 = 1u;
let POTATO: u32 = 2u;

struct MaterialProperties {
    lightmap: MaterialSetProp;
    base_a: MaterialSetProp;
    base_b: MaterialSetProp;
    vary_a: MaterialSetProp;
    vary_b: MaterialSetProp;
    reflection: MaterialSetProp;
    walls: MaterialSetProp;
    reflection_mask: MaterialSetProp;
    mist: MaterialSetProp;
    directional_light_blend: f32;
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32;
};

[[group(1), binding(0)]]
var<uniform> ma: MaterialProperties;
[[group(1), binding(1)]]
var lightmap_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var lightmap_sampler: sampler;
[[group(1), binding(3)]]
var base_texture: texture_2d<f32>;
[[group(1), binding(4)]]
var base_sampler: sampler;
[[group(1), binding(5)]]
var vary_texture: texture_2d<f32>;
[[group(1), binding(6)]]
var vary_sampler: sampler;
[[group(1), binding(7)]]
var reflection_texture: texture_2d<f32>;
[[group(1), binding(8)]]
var reflection_sampler: sampler;
[[group(1), binding(9)]]
var walls_texture: texture_2d<f32>;
[[group(1), binding(10)]]
var walls_sampler: sampler;

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {

    //------------------------------------------
    var N: vec3<f32> = normalize(in.world_normal);
    var V = normalize(view.world_position.xyz - in.world_position.xyz);
    // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
    let NdotV = max(dot(N, V), 0.0001);
    //get the dot product between the normal and the view direction
    var fresnel = NdotV;
    //invert the fresnel so the big values are on the outside
    fresnel = clamp(1.0 - fresnel, 0.0, 1.0);
    //------------------------------------------
    var col = vec3<f32>(1.0);

    let lightmap = textureSample(lightmap_texture, lightmap_sampler, in.uv * ma.lightmap.scale).rgb;
    col = mix(col, col * pow(lightmap, vec3<f32>(ma.lightmap.contrast)) * ma.lightmap.brightness, ma.lightmap.blend);
    if ((ma.flags & POTATO) == 0u) {
        let base_tex_a = textureSample(base_texture, base_sampler, in.uv * ma.base_a.scale).rgb;
        col = mix(col, col * pow(base_tex_a, vec3<f32>(ma.base_a.contrast)) * ma.base_a.brightness, ma.base_a.blend);

        let base_tex_b = textureSample(base_texture, base_sampler, in.uv * ma.base_b.scale).rgb;
        col = mix(col, col * pow(base_tex_b, vec3<f32>(ma.base_b.contrast)) * ma.base_b.brightness, ma.base_b.blend);

        let var_tex_a = textureSample(vary_texture, vary_sampler, in.uv * ma.vary_a.scale).rgb;
        col = mix(col, col * pow(var_tex_a, vec3<f32>(ma.vary_a.contrast)) * ma.vary_a.brightness, ma.vary_a.blend);

        let var_tex_b = textureSample(vary_texture, vary_sampler, in.uv * ma.vary_b.scale).rgb;
        col = mix(col, col * pow(var_tex_b, vec3<f32>(ma.vary_b.contrast)) * ma.vary_b.brightness, ma.vary_b.blend);
        //Use variation textures to create ripples in the water UVs
        var ref_uv = normalize(reflect(V, N)).xy * ma.reflection.scale + var_tex_a.y * 0.01 + var_tex_b.y * 0.01; 

        var ref_sample = textureSample(reflection_texture, reflection_sampler, ref_uv * ma.reflection.scale).rgb;
        ref_sample = mix(ref_sample, ref_sample * pow(base_tex_a, vec3<f32>(ma.base_a.contrast)) * ma.base_a.brightness, ma.base_a.blend);
        var ref = mix(col, pow(ref_sample, vec3<f32>(ma.reflection.contrast)) * ma.reflection.brightness, ma.reflection.blend);


        var puddle_mask = textureSample(vary_texture, vary_sampler, in.uv * ma.reflection_mask.scale + vec2<f32>(0.2, 0.0)).g;
        puddle_mask = 1.0-clamp(pow(puddle_mask, ma.reflection_mask.contrast)*ma.reflection_mask.brightness, 0.0, 1.0);
        ref = mix(col, ref, vec3<f32>(puddle_mask*fresnel*0.9));

        var walls_mask = abs(in.world_normal.z + in.world_normal.x);
        var walls_tex = textureSample(walls_texture, walls_sampler, in.uv.yx * ma.walls.scale).rgb;
        walls_tex = pow(walls_tex, vec3<f32>(ma.walls.contrast)) * ma.walls.brightness;
        col = mix(col, walls_tex * col, walls_mask * ma.walls.blend);

        col = mix(col, ref, clamp(ceil((in.world_normal.y - 0.99))*100.0,0.0,1.0));

        var mist = pow(clamp(ma.mist.scale-in.frag_coord.w,0.0,1.0), ma.mist.contrast) * ma.mist.brightness;

        if ((ma.flags & SHADOWS) != 0u) {
            //var shadow = fetch_point_shadow(get_light_id(0u), in.world_position, in.world_normal);
            var shadow = fetch_directional_shadow(0u, in.world_position, in.world_normal);
            col = col + col * shadow * vec3<f32>(1.0,0.9,0.5) * 4.0 * ma.directional_light_blend;
        }

        col = mix(col, col + mist, ma.mist.blend);
    }
    return vec4<f32>(aces(col), 1.0);
    //return vec4<f32>(vec3<f32>(fresnel), 1.0);
}
