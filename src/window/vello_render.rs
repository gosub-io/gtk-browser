use std::cmp;
use std::num::NonZeroUsize;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};
use vello::util::block_on_wgpu;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions};
use wgpu::util::{backend_bits_from_env, dx12_shader_compiler_from_env, gles_minor_version_from_env};
use wgpu::{
    Backends, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, Instance, InstanceDescriptor,
    TextureDescriptor, TextureFormat, TextureUsages,
};

pub async fn render(width: u32, height: u32) -> Vec<u8> {
    dbg!(width, height);
    let instance = Instance::new(InstanceDescriptor {
        backends: backend_bits_from_env().unwrap_or(Backends::all()),
        dx12_shader_compiler: dx12_shader_compiler_from_env().unwrap_or_default(),
        gles_minor_version: gles_minor_version_from_env().unwrap_or_default(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find an appropriate adapter");
    
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: Default::default(),
                required_limits: Default::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Get the GDK surface from the DrawingArea

    let mut renderer = Renderer::new(
        &device,
        RendererOptions {
            surface_format: None,
            use_cpu: false,
            antialiasing_support: AaSupport::all(),
            num_init_threads: NonZeroUsize::new(4),
        },
    )
    .expect("Failed to create Vello renderer");

    // Create a new Vello scene
    let mut scene = vello::Scene::new();

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::AQUAMARINE),
        None,
        &Rect::new(0.0, 0.0, width as f64, height as f64),
    );

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::ORANGE),
        None,
        &Rect::new(
            (cmp::max(width / 2, 50) - 50) as f64,
            (cmp::max(height / 2, 25) - 25) as f64,
            ((width / 2) + 50) as f64,
            ((height / 2) + 25) as f64,
        ),
    );

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let target = device.create_texture(&TextureDescriptor {
        label: Some("Target texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    renderer
        .render_to_texture(
            &device,
            &queue,
            &scene,
            &view,
            &RenderParams {
                height,
                width,
                base_color: Color::WHITE,
                antialiasing_method: AaConfig::Msaa16,
            },
        )
        .expect("Failed to render with Vello");

    let padded_byte_width = (width * 4).next_multiple_of(256);
    let buffer_size = padded_byte_width as u64 * height as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("val"),
        size: buffer_size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Copy out buffer"),
    });
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_byte_width),
                rows_per_image: None,
            },
        },
        size,
    );
    queue.submit([encoder.finish()]);
    let buf_slice = buffer.slice(..);

    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buf_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    if let Some(recv_result) = block_on_wgpu(&device, receiver.receive()) {
        recv_result.unwrap();
    } else {
        panic!("Failed to map buffer");
    }

    let data = buf_slice.get_mapped_range();
    let mut result_unpadded = Vec::<u8>::with_capacity((width * height * 4).try_into().unwrap());
    for row in 0..height {
        let start = (row * padded_byte_width).try_into().unwrap();
        result_unpadded.extend(&data[start..start + (width * 4) as usize]);
    }

    result_unpadded
}
