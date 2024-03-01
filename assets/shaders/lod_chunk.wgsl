#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_bindings,
    mesh_functions,
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#import bevy_render::instance_index::get_instance_index;

@group(1) @binding(100)
var<uniform> my_extended_material: LodChunkMaterial;
struct LodChunkMaterial {
    dummy: u32,
}

// TODO: UPDATE THIS OFC
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(1) @interpolate(flat) hack_ass_vert: vec2<u32>,
};

struct MyVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

// TODO: UPDATE THIS OFC
@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    // Hack layout:
    //                       Negative normal?
    //                              \_/
    // U32:(0xxxxx,0y)(yyyy,0zzz)(zz,nnnn,0r)(rrgggbbb)
    //       ^---^  ^-----^  ^-----^  ^-^  ^---^^-^^-^
    //        X        Y        Z    Norml   R   G  B
    // 3x5 bits = 0-31 for each position component
    // 3 bits for each axis of normal, 1 bit for negative.
    // 2x5 bits = 0-31 for quad size to determine UV
    // 4 bytes for U32 to represent atlas index
    var hack = vertex.hack_ass_vert.x;
    var in_pos_x = hack >> 26u;
    var in_pos_y = (hack << 6u) >> 26u;
    var in_pos_z = (hack << 12u) >> 26u;
    var is_normal_neg = (hack << 18u) >> 31u;
    var normal_x = (hack << 19u) >> 31u;
    var normal_y = (hack << 20u) >> 31u;
    var normal_z = (hack << 21u) >> 31u;
    var color_r = (hack << 23u) >> 29u;
    var color_g = (hack << 26u) >> 29u;
    var color_b = (hack << 29u) >> 29u;

    // 1 -> 0 -> 0 -> -1
    // 0 -> 1 -> 2 -> 1
    var pre_normal_mul = (2 * (1 - i32(is_normal_neg))) - 1;
    var made_normal = f32(pre_normal_mul) * vec3<f32>(
        f32(normal_x),
        f32(normal_y),
        f32(normal_z)
    );

    var out: MyVertexOutput;

    var model = mesh_functions::get_model_matrix(vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(
        model,
        vec4<f32>(
            f32(in_pos_x),
            f32(in_pos_y),
            f32(in_pos_z),
            1.0
        )
    );
    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        made_normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        get_instance_index(vertex.instance_index)
    );
    out.color = vec3<f32>(
        f32(color_r),
        f32(color_g),
        f32(color_b)
    );

    return out;
}

@fragment
fn fragment(
    input: MyVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_vert_out: VertexOutput;
    pbr_vert_out.position = input.position;
    pbr_vert_out.world_position = input.world_position;
    pbr_vert_out.world_normal = input.world_normal;

    // Generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(pbr_vert_out, is_front);

    // Alpha discard
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color
    );
    pbr_input.material.base_color *= input.color;

    var out: FragmentOutput;

    // Apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // Apply in-shader post processing.
    // Ex: fog, alpha-premultiply, etc. For non-hdr cameras: tonemapping and debanding
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
