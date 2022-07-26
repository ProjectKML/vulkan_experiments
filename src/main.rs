use glam::Vec3;
use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, VirtualKeyCode};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use crate::render::{render_ctx::RenderCtx, renderer, Camera};

pub mod render;

fn main() {
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan experiments")
        .with_inner_size(Size::Physical(PhysicalSize::new(1600, 900)))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut render_ctx = RenderCtx::new(&window);

    let mut frame_count = 0;
    let mut frame_index = 0;

    let mut running = true;

    let mut pressed_keys = HashSet::new();
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

    while running {
        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, window_id } => {
                    if window.id() == window_id {
                        match event {
                            WindowEvent::Resized(size) => {} //TODO: handle
                            WindowEvent::CloseRequested => running = false,
                            WindowEvent::KeyboardInput {
                                device_id,
                                input,
                                is_synthetic,
                            } => {
                                if let Some(key_code) = input.virtual_keycode {
                                    if key_code == VirtualKeyCode::Escape {
                                        running = false;
                                    }

                                    match input.state {
                                        ElementState::Pressed => {
                                            if !pressed_keys.contains(&key_code) {
                                                pressed_keys.insert(key_code);
                                            }
                                        }
                                        ElementState::Released => {
                                            if pressed_keys.contains(&key_code) {
                                                pressed_keys.remove(&key_code);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::DeviceEvent { device_id, event } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        camera.move_mouse(delta.0, delta.1);
                    }
                    _ => {}
                },

                _ => {}
            }
        });

        unsafe {
            let delta = 1.0 / 165.0; //TODO: dont do this
            camera.update(&pressed_keys, delta);
            renderer::render_frame(&mut render_ctx, &mut frame_index, &camera);
        }

        frame_count += 1;
        frame_index = frame_count % render_ctx.frames.len();
    }
}
