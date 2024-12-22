use bytemuck::{Pod, Zeroable};
use rand::{thread_rng, Rng};
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;


// max dispatch group size in each dimension is 65535
const MATRIX_SIZE: usize = 100;

#[repr(C)]
#[derive(Copy, Clone)]
struct Matrix {
    val: [f32;MATRIX_SIZE*MATRIX_SIZE],
}

unsafe impl Pod for Matrix {}
unsafe impl Zeroable for Matrix {}

impl Matrix {
    fn new_rand() -> Self {
        let mut rng = thread_rng();

        let mut mat = Self { val: [0f32; MATRIX_SIZE*MATRIX_SIZE] };

        for i in 0..mat.val.len(){
            mat.val[i] = rng.gen_range(0f32..10f32);
        }

        mat
    }


    /*fn print(&self) {
        let size = MATRIX_SIZE;
        for i in 0..size {
            for j in 0..size {
                print!{"{} ", self.val[i*size + j]};
            }
            println!();
        }
        println!();
    }*/
}



fn main() {
    pollster::block_on(run());
}


async fn run() {
    let a = Matrix::new_rand();
    let b = Matrix::new_rand();


    let result = execute_gpu(&a, &b).await.unwrap();

/*    if MATRIX_SIZE < 8 {
        a.print();
        b.print();
        result.print();
    }*/
}

async fn execute_gpu(a: &Matrix, b: &Matrix) -> Option<Matrix> {
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
        contents: bytemuck::bytes_of(a),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("B Buffer"),
        contents: bytemuck::bytes_of(b),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Size buffer"),
        contents: bytemuck::bytes_of(&MATRIX_SIZE),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
        },  wgpu::BindGroupEntry {
                binding: 3,
                resource: size_buffer.as_entire_binding(),
        },]
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(MATRIX_SIZE as u32, MATRIX_SIZE as u32, 1);
    }
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);
    queue.submit(Some(encoder.finish()));


    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap() );

    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    if let Ok(Ok(())) = receiver.recv_async().await {
        let data_view = buffer_slice.get_mapped_range();
        let result_view = bytemuck::cast_slice::<_, Matrix>(&data_view);
        let result = result_view[0].clone();
        drop(data_view);
        staging_buffer.unmap();
        Some(result)
    } else {
        panic!("failed to run compute on gpu!");
    }
}


