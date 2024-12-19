use rand::{thread_rng, Rng};
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;


// max dispatch group size in each dimension is 65535
const VECTOR_SIZE: usize = 60_000;


fn main() {
    pollster::block_on(run());
}


async fn run() {
    let mut rng = thread_rng();
    let mut a: [f32; VECTOR_SIZE] = [0f32; VECTOR_SIZE];
    let mut b: [f32; VECTOR_SIZE] = [0f32; VECTOR_SIZE];

    rng.fill(&mut a[..]);
    rng.fill(&mut b[..]);


    let result = execute_gpu(&a, &b).await.unwrap();


    for i in 0..VECTOR_SIZE {
        if a[i] + b[i] != result[i] {
            panic!("{} + {} != {}", a[i], b[i], result[i]);
        }
    }
    println!("All computations completed  correctly!");
    let rand_index = rng.gen_range(0..VECTOR_SIZE);
    println!("A random example of result:\n{} + {} = {}", a[rand_index], b[rand_index], a[rand_index]+b[rand_index]);


}

async fn execute_gpu(a: &[f32], b: &[f32]) -> Option<Vec<f32>> {
    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
            None).await.unwrap();

    let cs_module = device.create_shader_module(include_wgsl!("shader.wgsl"));


    let size = size_of_val(a) as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let a_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("A Buffer"),
        contents: bytemuck::cast_slice(&a),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("B Buffer"),
        contents: bytemuck::cast_slice(&b),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });


    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
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
        },]
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(VECTOR_SIZE as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));


    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap() );

    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    if let Ok(Ok(())) = receiver.recv_async().await {
        let data_view = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data_view).to_vec();

        drop(data_view);
        staging_buffer.unmap();

        Some(result)
    } else {
        panic!("failed to run compute on gpu!");
    }
}


