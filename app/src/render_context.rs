use std::{
    cell::RefCell,
    sync::Arc,
};

use wgpu::{CompositeAlphaMode, PollType, util::DeviceExt};
use winit::window::Window;

use crate::{
    // model_data::MyMesh,
    // model_instance::ModelInstance,
    camera_uniform::CameraUniform,
    my_texture::MyTexture,
    shape_mesh::ShapeMesh,
    state::State,
    transparent_pipeline::{TransparentPipeline},
    vertex::Vertex, // ui_pipeline::UIPipeline,
};

pub struct RenderContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: RefCell<wgpu::SurfaceConfiguration>,
    pub size: RefCell<winit::dpi::PhysicalSize<u32>>,
    pub depth_texture: RefCell<MyTexture>,
    pub transparent_pipeline: TransparentPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,

    pub square_mesh: Arc<ShapeMesh>,
    pub circle_mesh: Arc<ShapeMesh>,
}

impl RenderContext {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // instance represents the connection to the graphics API and system GPU drivers
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        // surface only depends on window
        let surface = instance.create_surface(window).unwrap();
        // adapter represents a GPU
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }))
            .unwrap();

        let _surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .find(|f| f.is_srgb())
        //     .copied()
        //     .unwrap_or(surface_caps.formats[0]);
        let surface_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        // define how the surface creates its underlying SurfaceTextures
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // enable vsync with fifo present mode
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("view_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture = MyTexture::create_depth_texture(&device, &config, "depth texture");

        let transparent_pipeline =
            TransparentPipeline::new(&device, &config, &camera_bind_group_layout);

        let square_mesh = create_square_mesh(&device);
        let circle_mesh = create_circle_mesh(&device, 32);
        RenderContext {
            surface,
            device,
            queue,
            config: RefCell::new(config),
            size: RefCell::new(size),
            depth_texture: RefCell::new(depth_texture),
            transparent_pipeline,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            square_mesh,
            circle_mesh,
        }
    }
    pub fn resize(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        *self.size.borrow_mut() = new_size;
        let mut config = self.config.borrow_mut();
        config.width = new_size.width;
        config.height = new_size.height;
        self.surface.configure(&self.device, &config);
        *self.depth_texture.borrow_mut() =
            MyTexture::create_depth_texture(&self.device, &config, "depth texture");
    }

    pub fn render(&self, state: &State) -> Result<(), wgpu::SurfaceError> {
        if state.transparent_shape_submissions.is_none() {
            println!("No transparent shape submissions, skipping render");
            return Ok(());
        }
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // update camera transform
        // let aspect = self.config.width as f32 / self.config.height as f32;
        // to do
        let camera_uniform = state.camera.to_uniform();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // convert model instances to mesh instances
        let transparent_shape_submissions = &state.transparent_shape_submissions.as_ref().unwrap();
        let depth_texture = self.depth_texture.borrow();
        self.transparent_pipeline.render(
            &transparent_shape_submissions,
            &mut encoder,
            &self.device,
            &self.queue,
            &view,
            &depth_texture.view,
            &self.camera_bind_group,
        );

        // submit will accept anything that implements IntoIter

        // std::thread::sleep(std::time::Duration::from_millis(500));
        //     let mut input = String::new();
        // std::io::stdin()
        //     .read_line(&mut input)
        //     .expect("Failed to read line");
        // panic!("render");
        self.queue.submit(std::iter::once(encoder.finish()));
        let _ = self.device.poll(PollType::Wait).unwrap();
        output.present();
        Ok(())
    }
}

pub fn create_square_mesh(device: &wgpu::Device) -> Arc<ShapeMesh> {
    let vertices = vec![
        Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        },
    ];
    let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Square Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Square Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let shape_mesh = ShapeMesh {
        vertex_buffer,
        index_buffer,
        num_indices: indices.len() as u32,
    };
    Arc::new(shape_mesh)
}

pub fn create_circle_mesh(device: &wgpu::Device, segments: u16) -> Arc<ShapeMesh> {
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let radius = 0.5;
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        vertices.push(Vertex {
            position: [x, y, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        });
        if i > 1 {
            indices.push(0);
            indices.push(i - 1);
            indices.push(i);
        }
    }
    // close the circle
    indices.push(0);
    indices.push(segments - 1);
    indices.push(1);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Circle Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Circle Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let shape_mesh = ShapeMesh {
        vertex_buffer,
        index_buffer,
        num_indices: indices.len() as u32,
    };
    Arc::new(shape_mesh)
}
