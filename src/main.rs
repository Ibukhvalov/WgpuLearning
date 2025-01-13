mod matrix;


#[cfg(test)]
mod test;

use log::{info, debug};
use matrix::Matrix;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

// max dispatch group size in each dimension is 65535
// max buffer size is 256mb
// max bind group is 128mb
const MATRIX_SIZE: usize = 5000;
const TILE_SIZE: usize = 16;



fn main() {
    env_logger::builder()
        .filter_module("shader_learning", log::LevelFilter::Debug)
        .init();

    info!("Generating matrix data");


    let a = Matrix::new_rand(MATRIX_SIZE);
    let b = Matrix::new_rand(MATRIX_SIZE);

    pollster::block_on(execute_gpu(&a,&b));
}


async fn execute_gpu(a: &Matrix, b: &Matrix) -> Option<Matrix> {

    info!("Getting gpu ready");
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .unwrap();


    let cs_module = device.create_shader_module(include_wgsl!("shader.wgsl"));

    let size = a.data_size() as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let a_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("A Buffer"),
        contents: bytemuck::cast_slice(&a.val),
        usage: wgpu::BufferUsages::STORAGE,
    });

    info!("Creating buffers");

    let b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("B Buffer"),
        contents: bytemuck::cast_slice(&b.val),
        usage: wgpu::BufferUsages::STORAGE,
    });


    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });


    let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Size buffer"),
        contents: bytemuck::bytes_of(&MATRIX_SIZE),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: a_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: size_buffer.as_entire_binding(),
            },
        ],
    });

    info!("Start submitting commands to GPU");

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);

        let workgoup_num = MATRIX_SIZE.div_ceil(TILE_SIZE);

        debug!("Dispatched {} {} {}", workgoup_num as u32, workgoup_num as u32, 1);

        cpass.dispatch_workgroups(workgoup_num as u32, workgoup_num as u32, 1);
    }
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    info!("GPU completed all commands");

    if let Ok(Ok(())) = receiver.recv_async().await {
        let data_view = buffer_slice.get_mapped_range();
        let result = Matrix::from_bytes(&data_view).unwrap();
        drop(data_view);
        staging_buffer.unmap();
        Some(result)
    } else {
        panic!("failed to run compute on gpu!");
    }
}
