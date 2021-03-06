use cgmath::*;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod adaptive;
mod app;
mod compute;
mod cpu_octree;
mod gpu;
mod octree;
mod procedural;
mod render;
mod world;
use adaptive::*;
use app::*;
use compute::*;
use cpu_octree::*;
use gpu::*;
use octree::*;
use procedural::*;
use render::*;
use world::*;

#[tokio::main]
async fn main() {
    println!("octree-tracer v0.1.0");

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut app = pollster::block_on(App::new(&window));

    let now = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        app.render.egui_platform.handle_event(&event);
        app.input(&window, &event);
        match event {
            Event::RedrawRequested(_) => {
                match app.render.render(&app.gpu, &window) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => app.render.resize(&app.gpu, app.render.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
                app.update(now.elapsed().as_secs_f64());
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        app.render.resize(&app.gpu, *physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        app.render.resize(&app.gpu, **new_inner_size);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            _ => {}
        }
    });
}

pub struct Input {
    forward: bool,
    backward: bool,
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    mouse_delta: Vector2<f32>,
}

impl Input {
    fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            right: false,
            left: false,
            up: false,
            down: false,
            mouse_delta: Vector2::zero(),
        }
    }
}

pub struct Settings {
    octree_depth: u32,
    fov: f32,
    sensitivity: f32,
}

pub struct Character {
    pos: Point3<f32>,
    look: Vector3<f32>,
    cursour_grabbed: bool,
    speed: f32,
}

impl Character {
    fn new() -> Self {
        Self {
            pos: Point3::new(0.1, 0.2, -1.5),
            look: -Vector3::new(0.0, 0.0, -1.5),
            cursour_grabbed: true,
            speed: -5.0,
        }
    }
}

fn create_proj_matrix(fov: f32, aspect: f32) -> Matrix4<f32> {
    let s = 1.0 / ((fov / 2.0) * (std::f32::consts::PI / 180.0)).tan();
    Matrix4::new(
        aspect * s,
        0.0,
        0.0,
        0.0,
        //
        0.0,
        s,
        0.0,
        0.0,
        //
        0.0,
        0.0,
        -1.0,
        0.0,
        //
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
