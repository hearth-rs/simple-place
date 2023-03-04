use egui_wgpu::renderer::Renderer as EguiRenderer;
use egui_winit::winit;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub mod ui;

struct App {
    window: Window,
    size: PhysicalSize<u32>,
    device: Device,
    queue: Queue,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    egui_state: egui_winit::State,
    egui_ctx: egui::Context,
    egui_renderer: EguiRenderer,
    ui: ui::UserInterface,
}

impl App {
    pub async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Simple Place")
            .build(&event_loop)
            .unwrap();

        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let formats = surface.get_capabilities(&adapter).formats;
        let format = egui_wgpu::preferred_framebuffer_format(&formats);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![format],
        };

        surface.configure(&device, &surface_config);

        let egui_state = egui_winit::State::new(event_loop);
        let egui_ctx = egui::Context::default();
        let egui_renderer = EguiRenderer::new(&device, format, None, 1);

        let ui = ui::UserInterface::new();

        Self {
            window,
            size,
            device,
            queue,
            surface,
            surface_config,
            egui_state,
            egui_ctx,
            egui_renderer,
            ui,
        }
    }

    pub fn update(&mut self) {
        self.window.request_redraw();
    }

    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(st) => st,
            Err(SurfaceError::Lost) => {
                self.on_resize(self.size);
                return;
            }
            Err(err) => {
                eprintln!("Surface error: {:?}", err);
                return;
            }
        };

        let raw_input = self.egui_state.take_egui_input(&self.window);
        let output = self.egui_ctx.run(raw_input, |ctx| self.ui.show(ctx));

        self.egui_state.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            output.platform_output,
        );

        let ren = &mut self.egui_renderer;
        let meshes = self.egui_ctx.tessellate(output.shapes);

        for (id, image_delta) in output.textures_delta.set.iter() {
            ren.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let screen_desc = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: self.egui_ctx.pixels_per_point(),
        };

        let mut cmds = self.device.create_command_encoder(&Default::default());
        ren.update_buffers(&self.device, &self.queue, &mut cmds, &meshes, &screen_desc);

        let output_view = surface_texture.texture.create_view(&Default::default());

        {
            let mut rp = cmds.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // non-egui rendering goes here
            // TODO use egui custom render callbacks instead?

            ren.render(&mut rp, &meshes, &screen_desc);
        }

        for id in output.textures_delta.free.iter() {
            ren.free_texture(id);
        }

        self.queue.submit(Some(cmds.finish())); // Option implements IntoIterator
        surface_texture.present();
    }

    pub fn should_quit(&self) -> bool {
        false
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut app = pollster::block_on(App::new(&event_loop));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => app.render(),
        Event::MainEventsCleared => {
            app.update();

            if app.should_quit() {
                control_flow.set_exit();
            }
        }
        Event::WindowEvent { event, .. } => {
            match &event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(size) => app.on_resize(*size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    app.on_resize(**new_inner_size)
                }
                _ => {}
            }

            let _response = app.egui_state.on_event(&app.egui_ctx, &event);
        }
        _ => {}
    });
}
