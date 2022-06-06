use core::time;
use std::{borrow::Cow, iter, mem, thread};

use rand::{distributions::Uniform, prelude::IteratorRandom, thread_rng, Rng, SeedableRng};
use smaa::SmaaTarget;
use wgpu::{util::DeviceExt, BindGroup, ComputePipeline};
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
    cube::Cube,
    scene::{Scene, Vertex},
    texture::Texture,
};

const GRID_WIDTH: u32 = 15;
const TOTAL_CELLS: u32 = GRID_WIDTH * GRID_WIDTH * GRID_WIDTH;

pub struct State {
    cell_bind_groups: Vec<wgpu::BindGroup>,
    cell_buffers: Vec<wgpu::Buffer>,
    compute_pipeline: ComputePipeline,
    frame_num: usize,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    depth_texture: Texture,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub smaa_target: SmaaTarget,
    scene: Scene,
}

impl State {
    pub async fn new(window: &Window, scene: Scene) -> Self {
        let (_instance, surface, adapter, device, queue) = State::create_iadq(window).await;
        let size = window.inner_size();
        let config = State::configure_surface(&surface, &adapter, size);
        surface.configure(&device, &config);
        let shader = State::get_shader(&device);
        let camera = Camera::new(&config);
        let camera_controller = CameraController::new(0.2);
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let smaa_target = SmaaTarget::new(
            &device,
            &queue,
            window.inner_size().width,
            window.inner_size().height,
            config.format,
            smaa::SmaaMode::Smaa1X,
        );

        let (
            camera_bind_group,
            camera_buffer,
            camera_uniform,
            _render_pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        ) = State::setup_render_pipeline(&device, &shader, &config, &camera);

        let (cell_bind_groups, cell_buffers, compute_pipeline) =
            State::setup_compute_pipeline(&device);

        Self {
            cell_bind_groups,
            cell_buffers,
            compute_pipeline,
            frame_num: 0,
            surface,
            device,
            queue,
            size,
            config,
            camera,
            camera_bind_group,
            camera_buffer,
            camera_controller,
            camera_uniform,
            depth_texture,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            smaa_target,
            scene,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.smaa_target
                .resize(&self.device, new_size.width, new_size.height);
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let smaa_frame = self
            .smaa_target
            .start_frame(&self.device, &self.queue, &view);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Recalculate Vertices
        let computed_cell_buffer = &self.cell_buffers[(self.frame_num + 1) % 2];
        let cell_buffer_slice = computed_cell_buffer.slice(..);
        let cell_buffer_future = cell_buffer_slice.map_async(wgpu::MapMode::Read);
        self.device.poll(wgpu::Maintain::Wait);

        let mut scene = Scene::new();
        if let Ok(()) = cell_buffer_future.await {
            // Gets contents of buffer
            let data = cell_buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to i32
            let result: Vec<i32> = bytemuck::cast_slice(&data).to_vec();

            for result_chunk in result.chunks(4) {
                let state = result_chunk[0] as f32;
                let x = result_chunk[1] as f32;
                let y = result_chunk[2] as f32;
                let z = result_chunk[3] as f32;
                // println!("{}: ({} {} {})", state, x, y, z);

                if state == 1f32 {
                    scene.add_cube(Cube::new(
                        x,
                        y,
                        z,
                        1.,
                        [
                            x / GRID_WIDTH as f32,
                            y / GRID_WIDTH as f32,
                            z / GRID_WIDTH as f32,
                        ],
                    ))
                }
            }

            drop(data);
            computed_cell_buffer.unmap();
        }

        let (vertices, indices) = scene.get_vertices_and_indices();
        println!(
            "Num cubes: {} fn: {}",
            self.scene.cubes.len(),
            self.frame_num
        );

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        self.scene = scene;

        // Run Compute Pass
        encoder.push_debug_group("compute boid movement");
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.cell_bind_groups[self.frame_num % 2], &[]);
            cpass.dispatch(TOTAL_CELLS, 1, 1);
        }
        encoder.pop_debug_group();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &*smaa_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let (vertices, indices) = self.scene.get_vertices_and_indices();

            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));

        smaa_frame.resolve();
        output.present();

        self.frame_num += 1;

        thread::sleep(time::Duration::from_millis(500));

        Ok(())
    }

    async fn create_iadq(
        window: &Window,
    ) -> (
        wgpu::Instance,
        wgpu::Surface,
        wgpu::Adapter,
        wgpu::Device,
        wgpu::Queue,
    ) {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        (instance, surface, adapter, device, queue)
    }

    fn configure_surface(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        }
    }

    fn get_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        })
    }

    fn setup_compute_pipeline(
        device: &wgpu::Device,
    ) -> (Vec<wgpu::BindGroup>, Vec<wgpu::Buffer>, ComputePipeline) {
        // Compute
        let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
        });

        // Following Death/Survive/Birth -> 0/1/2
        let birth_list: Vec<u32> = vec![4, 5, 6, 7, 8, 9, 10];
        let survive_list: Vec<u32> = vec![4, 5, 6, 7, 8];
        let mut ruleset_list: Vec<u32> = vec![0; 27];
        for birth in birth_list {
            ruleset_list[birth as usize] = 2;
        }

        for survive in survive_list {
            ruleset_list[survive as usize] = 1;
        }
        let rulset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Conway Birth List"),
            contents: bytemuck::cast_slice(&ruleset_list),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (ruleset_list.len() * mem::size_of::<u32>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            // min_binding_size: None,
                            min_binding_size: wgpu::BufferSize::new((TOTAL_CELLS * 16) as _),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            // min_binding_size: None,
                            min_binding_size: wgpu::BufferSize::new((TOTAL_CELLS * 16) as _),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        // Setting up pipeline

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        // Setting up initial cell data data
        let mut rng = thread_rng();
        let mut initial_cell_state: Vec<i32> = (0..(TOTAL_CELLS * 4) as usize)
            .map(|_| if rng.gen_bool(0.5) { 0 } else { 1 })
            .collect();

        let mut chunked_initial_cell_state = initial_cell_state.chunks_mut(4);
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_WIDTH {
                for z in 0..GRID_WIDTH {
                    let cell_instance_chunk = chunked_initial_cell_state.next().unwrap();
                    // cell_instance_chunk[0] = 1;
                    cell_instance_chunk[1] = x as i32;
                    cell_instance_chunk[2] = y as i32;
                    cell_instance_chunk[3] = z as i32;
                }
            }
        }

        // Create two buffers of cell state
        let mut cell_buffers = Vec::<wgpu::Buffer>::new();
        let mut cell_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            cell_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Cell Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_cell_state),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::MAP_READ,
                }),
            );
        }

        // Create 2 bind groups one for each buffer
        for i in 0..2 {
            cell_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: rulset_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: cell_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cell_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }

        (cell_bind_groups, cell_buffers, compute_pipeline)
    }

    fn setup_render_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
    ) -> (
        wgpu::BindGroup,
        wgpu::Buffer,
        CameraUniform,
        wgpu::PipelineLayout,
        wgpu::RenderPipeline,
        wgpu::Buffer,
        wgpu::Buffer,
    ) {
        // Camera Logic!!!
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (TOTAL_CELLS as wgpu::BufferAddress) * 100,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (TOTAL_CELLS as wgpu::BufferAddress) * 100,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        (
            camera_bind_group,
            camera_buffer,
            camera_uniform,
            render_pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        )
    }
}
