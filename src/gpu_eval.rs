use wgpu::util::DeviceExt;

pub const DEFAULT_WGSL: &str = include_str!("eval.wgsl");

pub trait GpuEvaluator {
    fn eval(&self, bytecode: &[u32], vars: &[f32]) -> f32;
}

pub struct WgslEvaluator {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParams {
    code_len: u32,
    vars_len: u32,
    _pad: [u32; 2],
}

impl WgslEvaluator {
    pub fn new(wgsl_source: &str) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("no suitable GPU adapter found");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to request GPU device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bytecode_vm_shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bytecode_vm_bgl"),
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bytecode_vm_pl"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("bytecode_vm_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bytecode_params"),
            size: std::mem::size_of::<GpuParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bytecode_out"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bytecode_staging"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            params_buffer,
            output_buffer,
            staging_buffer,
        }
    }

    pub fn eval(&self, bytecode: &[u32], vars: &[f32]) -> f32 {
        // Storage buffers must be non-empty in WebGPU/WGSL bindings.
        let code_slice: &[u32] = if bytecode.is_empty() { &[0u32] } else { bytecode };
        let vars_slice: &[f32] = if vars.is_empty() { &[0.0f32] } else { vars };

        let code_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bytecode_stream"),
            contents: bytemuck::cast_slice(code_slice),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let vars_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vars_stream"),
            contents: bytemuck::cast_slice(vars_slice),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let params = GpuParams {
            code_len: bytecode.len() as u32,
            vars_len: vars.len() as u32,
            _pad: [0; 2],
        };
        self.queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bytecode_vm_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: code_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: vars_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("bytecode_vm_encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("bytecode_vm_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &self.staging_buffer,
            0,
            std::mem::size_of::<f32>() as u64,
        );

        self.queue.submit(Some(encoder.finish()));

        let slice = self.staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();

        let data = slice.get_mapped_range().unwrap();
        let result: f32 = *bytemuck::from_bytes(&data);
        drop(data);
        self.staging_buffer.unmap();
        result
    }
}

impl GpuEvaluator for WgslEvaluator {
    fn eval(&self, bytecode: &[u32], vars: &[f32]) -> f32 {
        WgslEvaluator::eval(self, bytecode, vars)
    }
}