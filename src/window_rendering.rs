use std::{sync::Arc, time::{Instant, Duration}};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::{movement::*};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    var_count: u32,
}

#[derive(Default, Clone)]
pub struct Scene {
    pub vars: Vec<f32>,
    pub camera: Vec<f32>,
}

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    vars_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vars: Vec<f32>,
    camera: Vec<f32>,
    start_time: Instant,
}

impl State {
    async fn new(window: Arc<Window>, scene: Scene, src_shader: &str) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                ..Default::default()
            })
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| matches!(f, wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Rgba8UnormSrgb))
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        };
        surface.configure(&device, &config);

        let initial_uniforms = Uniforms {
            resolution: [config.width as f32, config.height as f32],
            time: 0.0,
            var_count: scene.vars.len() as u32,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniforms"),
            contents: bytemuck::bytes_of(&initial_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let vars_slice: &[f32] = if scene.vars.is_empty() { &[0.0] } else { &scene.vars };
        let vars_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vars"),
            contents: bytemuck::cast_slice(vars_slice),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let cam_slice: &[f32] = if scene.camera.is_empty() { &[0.0] } else { &scene.camera };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera"),
            contents: bytemuck::cast_slice(cam_slice),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let storage_read_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: None,
    entries: &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        storage_read_entry(1), // vars
        storage_read_entry(2), // camera
    ],
});

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vars_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sdf shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(src_shader)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            uniform_buffer,
            vars_buffer,
            camera_buffer,
            bind_group,
            vars: scene.vars,
            camera: scene.camera,
            start_time: Instant::now(),
        }
    }

    fn push_vars(&self) {
        let slice: &[f32] = if self.vars.is_empty() { &[0.0] } else { &self.vars };
        self.queue
            .write_buffer(&self.vars_buffer, 0, bytemuck::cast_slice(slice));
    }

    fn push_camera(&self) {
        let slice: &[f32] = if self.camera.is_empty() { &[0.0] } else { &self.camera };
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(slice));
    }


    /// Syncs a `Scene` snapshot into `State` and uploads changed buffers to the GPU.
    /// Call this every frame once your event/async source produces a `Scene`.

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Placeholder per-frame update until wired to your event/async thread:
        // let next_scene = Scene { vars: self.vars.clone(), camera: self.camera.clone(), bytecode: self.bytecode.clone() };
        // self.update_scene(&next_scene);

        let uniforms = Uniforms {
            resolution: [self.config.width as f32, self.config.height as f32],
            time: elapsed,
            var_count: self.vars.len() as u32,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        self.window.request_redraw();
    }
}

struct App<'a> {
    state: Option<State>,
    scene: Scene,
    shader_src: &'a str,
    fps_frames: u32,
    fps_last: Instant,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // unchanged
        if self.state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Shader Window"))
                    .unwrap(),
            );
            let scene = std::mem::take(&mut self.scene);
            self.state = Some(pollster::block_on(State::new(window, scene, self.shader_src)));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.render();

                // --- FPS counter ---
                self.fps_frames += 1;
                let elapsed = self.fps_last.elapsed();
                if elapsed >= Duration::from_secs(1) {
                    let secs = elapsed.as_secs_f64();
                    let fps = self.fps_frames as f64 / secs;
                    println!("FPS: {:.1}  ({:.2} ms/frame)", fps, 1000.0 / fps);
                    self.fps_frames = 0;
                    self.fps_last = Instant::now();
                }
            },
                        WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let now = Instant::now();
                let dt = now.duration_since(self.fps_last).as_secs_f32().min(0.1);
                let speed = 0.5;
                let pressed = key_state == ElementState::Pressed;
                match code {
                    KeyCode::KeyW => {
                        state.camera.move_forward(speed, dt);
                        state.push_camera();
                    },
                    KeyCode::KeyS => {
                        state.camera.move_backward(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::KeyA => {
                        state.camera.move_left(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::KeyD => {
                        state.camera.move_right(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::ArrowLeft => {
                        state.camera.turn_left(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::ArrowRight => {
                        state.camera.turn_right(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::ArrowUp => {
                        state.camera.look_up(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::ArrowDown => {
                        state.camera.look_down(speed, dt);
                        state.push_camera();
                    },
                    KeyCode::Space => {
                        state.camera.move_up(speed, dt);
                        state.push_camera();
                    },
                    KeyCode::ShiftLeft => {
                        state.camera.move_down(speed, dt);
                        state.push_camera();
                    }
                    KeyCode::Escape if pressed && !repeat => event_loop.exit(),
                    _ => {}
                }
            }
            // ... KeyboardInput arm unchanged ...
            _ => {}
        }
    }
}

pub fn run_window(scene: &Scene, shader_src: &str) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        state: None,
        scene: scene.clone(),
        fps_frames: 0,
        fps_last: Instant::now(),
        shader_src: shader_src
    };
    event_loop.run_app(&mut app).unwrap();
}