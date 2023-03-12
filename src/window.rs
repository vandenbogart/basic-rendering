
use std::{time::{Instant}};

use winit::{
    event::{
        ElementState, Event as WinEvent, KeyboardInput, ModifiersState, MouseButton, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
};
pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: winit::window::Window,
}

pub enum Event {
    Resize {
        width: u32,
        height: u32,
    },
    CursorMove {
        x: f32,
        y: f32,
        modifiers: ModifiersState,
    },
    CursorInput {
        state: ElementState,
        button: MouseButton,
    },
    Keyboard {
        key: KeyboardInput,
    },
    Redraw,
    Loop {
        delta_time: f32,
        elapsed: f32,
    },
}


pub struct Clock {
    start: Instant,
    now: f32,
}
impl Clock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            now: 0.0,
        }
    }
    pub fn advance(&mut self) -> (f32, f32) {
        let elapsed = self.start.elapsed().as_secs_f32();
        let delta = elapsed - self.now;
        self.now = elapsed;
        (delta, self.now)
    }
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
        let mut clock = Clock::new();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = match event {
                WinEvent::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                            modifiers: _,
                        },
                    ..
                } => {
                    runner(Event::CursorInput { state, button });
                    ControlFlow::Poll
                }
                WinEvent::WindowEvent {
                    event:
                        WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                            modifiers,
                        },
                    ..
                } => {
                    runner(Event::CursorMove {
                        x: position.x as f32,
                        y: position.y as f32,
                        modifiers,
                    });
                    ControlFlow::Poll
                }
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
                    let (delta_time, elapsed) = clock.advance();
                    let _target = 1.0 / 60.0;
                    runner(Event::Loop { delta_time, elapsed });
                    self.window.request_redraw();
                    ControlFlow::Poll
                }
                WinEvent::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    match input.virtual_keycode {
                        Some(key) => {
                            if key == winit::event::VirtualKeyCode::Escape {
                                ControlFlow::Exit
                            }
                            else {
                                runner(Event::Keyboard { key: input });
                                ControlFlow::Poll
                            }
                        }
                        None => ControlFlow::Poll
                    }
                }
                _ => ControlFlow::Poll,
            }
        });
    }
}
