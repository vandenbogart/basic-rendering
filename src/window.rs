use std::time::Duration;

use winit::{
    event::{Event as WinEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

pub enum Event {
    Resize { width: u32, height: u32 },
    Keyboard { key: KeyboardInput },
    Redraw,
    Loop { delta_time: Duration },
}

impl Window {
    pub fn new() -> Window {
        // env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        Window { event_loop, window }
    }
    pub fn run(self, mut runner: impl 'static + FnMut(Event)) -> ! {
        let mut last_update_inst = std::time::Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = match event {
                WinEvent::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    runner(Event::Resize {
                        width: size.width,
                        height: size.height,
                    });
                    ControlFlow::Poll
                }
                WinEvent::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => ControlFlow::Exit,
                WinEvent::RedrawRequested(_) => {
                    runner(Event::Redraw);
                    ControlFlow::Poll
                }
                WinEvent::MainEventsCleared => {
                    let delta_time = last_update_inst.elapsed();
                    runner(Event::Loop { delta_time });
                    self.window.request_redraw();
                    last_update_inst = std::time::Instant::now();
                    ControlFlow::Poll
                }
                WinEvent::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    runner(Event::Keyboard { key: input });
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            }
        });
    }
}
