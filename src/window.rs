use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use crate::{core::State, scene::Scene};

pub fn run(fullscreen: bool) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window_builder = WindowBuilder::new();

    if fullscreen {
        window_builder = window_builder.with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
        window_builder = window_builder.with_inner_size(LogicalSize::<f64>::new(800., 600.))
    }

    let window = window_builder.build(&event_loop).unwrap();

    // Use to test repeating pattern
    let _scene = Scene::new_tube();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window, None));

    // main()
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match pollster::block_on(state.render()) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
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
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
