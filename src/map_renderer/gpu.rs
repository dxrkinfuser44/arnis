use image::{Rgb, RgbImage};

#[cfg(feature = "gpu")]
use wgpu::util::DeviceExt;

pub struct GpuAdapterInfo {
    pub name: String,
    pub backend: &'static str,
}

/// GPU-accelerated post-process: lightweight gamma + contrast curve.
/// Falls back to CPU if GPU init fails.
pub fn apply_gpu_post_process_with_heights(
    image: &RgbImage,
    heights: &[i32],
) -> Result<(RgbImage, GpuAdapterInfo), String> {
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Ok((
            image.clone(),
            GpuAdapterInfo {
                name: "unknown".to_string(),
                backend: "unknown",
            },
        ));
    }

    let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for pixel in image.pixels() {
        pixels.push(pixel[0]);
        pixels.push(pixel[1]);
        pixels.push(pixel[2]);
        pixels.push(255);
    }

    let mut height_data: Vec<i32> = Vec::with_capacity((width * height) as usize);
    height_data.extend_from_slice(heights);

    let (processed, info) =
        pollster::block_on(process_rgba_gpu(width, height, pixels, height_data))?;

    let mut out = RgbImage::new(width, height);
    let mut idx = 0usize;
    for (_, _, pixel) in out.enumerate_pixels_mut() {
        let r = processed[idx];
        let g = processed[idx + 1];
        let b = processed[idx + 2];
        *pixel = Rgb([r, g, b]);
        idx += 4;
    }

    Ok((out, info))
}

#[cfg(feature = "gpu")]
async fn process_rgba_gpu(
    width: u32,
    height: u32,
    input: Vec<u8>,
    heights: Vec<i32>,
) -> Result<(Vec<u8>, GpuAdapterInfo), String> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| "No compatible GPU adapter found".to_string())?;

    let adapter_info = adapter.get_info();
    let backend_name = match adapter_info.backend {
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        wgpu::Backend::Dx12 => "DX12",
        wgpu::Backend::Gl => "OpenGL",
        wgpu::Backend::BrowserWebGpu => "WebGPU",
        _ => "Unknown",
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("arnis-map-gpu"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .map_err(|e| format!("Failed to request GPU device: {e}"))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("map-post-process"),
        source: wgpu::ShaderSource::Wgsl(include_str!("gpu_post.wgsl").into()),
    });

    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("input-buffer"),
        contents: &input,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let height_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("height-buffer"),
        contents: bytemuck::cast_slice(&heights),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output-buffer"),
        size: input.len() as u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let params = [width, height, 0, 0];
    let params_bytes = bytemuck::cast_slice(&params);
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params-buffer"),
        contents: params_bytes,
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: height_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("post-process"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("post-process-encoder"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("post-process-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_size = 256u32;
        let total_pixels = width * height;
        let groups = total_pixels.div_ceil(workgroup_size);
        pass.dispatch_workgroups(groups, 1, 1);
    }

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging-buffer"),
        size: input.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging, 0, input.len() as u64);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::Maintain::Wait);
    receiver
        .recv()
        .map_err(|_| "Failed to receive map result".to_string())?
        .map_err(|e| format!("Failed to map staging buffer: {e:?}"))?;
    let mapped = buffer_slice.get_mapped_range();
    let output = mapped.to_vec();
    drop(mapped);
    staging.unmap();

    Ok((
        output,
        GpuAdapterInfo {
            name: adapter_info.name,
            backend: backend_name,
        },
    ))
}
