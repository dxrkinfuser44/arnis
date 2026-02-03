struct Params {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0)
var<storage, read> input_data: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_data: array<u32>;

@group(0) @binding(2)
var<storage, read> heights: array<i32>;

@group(0) @binding(3)
var<uniform> params: Params;

fn unpack_color(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

fn pack_color(color: vec4<f32>) -> u32 {
    let r = u32(clamp(color.r, 0.0, 1.0) * 255.0) & 0xFFu;
    let g = u32(clamp(color.g, 0.0, 1.0) * 255.0) & 0xFFu;
    let b = u32(clamp(color.b, 0.0, 1.0) * 255.0) & 0xFFu;
    let a = u32(clamp(color.a, 0.0, 1.0) * 255.0) & 0xFFu;
    return r | (g << 8u) | (b << 16u) | (a << 24u);
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let total = params.width * params.height;
    if (index >= total) {
        return;
    }

    let packed = input_data[index];
    let color = unpack_color(packed);

    // Simple contrast + gamma-ish adjustment
    let contrast = 1.08;
    let gamma = 0.95;
    var rgb = pow(color.rgb, vec3<f32>(gamma));
    rgb = ((rgb - vec3<f32>(0.5)) * contrast) + vec3<f32>(0.5);

    // Elevation shading using height buffer
    let height = heights[index];
    if (height > -100000) {
        let normalized = clamp(f32(height) / 100.0, -1.0, 1.0);
        let elevation_adjust = select(normalized * 0.20, normalized * 0.20, normalized >= 0.0);
        let multiplier = 1.10 + elevation_adjust;
        rgb = clamp(rgb * multiplier, vec3<f32>(0.0), vec3<f32>(1.0));
    }

    output_data[index] = pack_color(vec4<f32>(rgb, color.a));
}
