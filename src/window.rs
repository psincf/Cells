use crate::game::Game;
use crate::renderer::{Camera, Renderer};

use crossbeam_utils::sync::Parker;
use euclid::default::{Point2D, Size2D};
use input_events::Events as InputEvents;

#[derive(Default)]
pub struct Events {
    pub parker: std::sync::Mutex<Vec<std::sync::Arc<Parker>>>,
    pub barrier: std::sync::Mutex<Vec<std::sync::Arc<std::sync::Barrier>>>,
    pub buffer_events: Vec<(input_events::event::Event, Option<(Camera, Point2D<i32>)>)>,
    pub input_events: InputEvents,
    pub mouse_events: MouseInfos,
    pub resize_events: ResizeEvents,
}

#[derive(Default)]
pub struct MouseInfos {
    pub mouse_position: Point2D<i32>,
    pub mouse_position_world: Point2D<i32>,
    pub mouse_on_gui: bool,
}

impl MouseInfos {
    pub fn update_mouse_position_world(&mut self, size_window: Size2D<i32>, camera: &Camera) { // TODO: without size_window ?
        if let Some(position) = camera.position_world((self.mouse_position.x, self.mouse_position.y), (size_window.width, size_window.height)) {
            self.mouse_position_world = Point2D::new(position.0, position.1);
        }
    }
}

pub struct ResizeEvents {
    pub resized: std::sync::atomic::AtomicBool,
    pub size: std::sync::Mutex<Size2D<i32>>,
    pub fullscreen: std::sync::Mutex<bool>,
}

impl Default for ResizeEvents {
    fn default() -> ResizeEvents {
        ResizeEvents {
            resized: std::sync::atomic::AtomicBool::new(false),
            size: std::sync::Mutex::new(Size2D::zero()),
            fullscreen: std::sync::Mutex::new(false),
        }
    }
}

pub struct Window {
    pub event_loop: Option<winit::event_loop::EventLoop<()>>,
    pub event_loop_proxy: Option<winit::event_loop::EventLoopProxy<()>>,
    pub window: Box<winit::window::Window>,
    pub events: Events,
}

impl Window {
    pub fn new() -> Window {
        let icon_data = include_bytes!("../assets/icon/icon.png");
        let icon = image::load_from_memory(icon_data).unwrap();
        let icon = icon.into_rgba8();
        let icon_pixels: Vec<u8> = icon.as_raw().clone();


        let event_loop = Some(winit::event_loop::EventLoop::new());
        let event_loop_proxy = Some(event_loop.as_ref().unwrap().create_proxy());
        #[cfg(not(feature = "shipping"))]
        let window = Box::new(winit::window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(event_loop.as_ref().unwrap().primary_monitor())))
            //.with_fullscreen(Some(winit::window::Fullscreen::Exclusive(event_loop.as_ref().unwrap().primary_monitor().video_modes().next().unwrap())))
            .with_title("Cells Demo")
            .with_transparent(false)
            .with_window_icon(Some(winit::window::Icon::from_rgba(icon_pixels, icon.width(), icon.height()).unwrap()))
            .build(event_loop.as_ref().unwrap())
            .unwrap());

        let events = Events::default();

        *events.resize_events.size.lock().unwrap() = Size2D::new(window.inner_size().width as i32, window.inner_size().height as i32);

        Window {
            event_loop,
            event_loop_proxy,
            window,
            events,
        }
    }

    pub fn poll_events(&mut self, game: &Game, renderer: &mut Renderer) {
        let mut last_event_time = std::time::Instant::now();
        let mut last_camera_move_time = std::time::Instant::now();
        let mut event_loop = self.event_loop.take().unwrap();
        use winit::platform::run_return::EventLoopExtRunReturn;
        event_loop.run_return(|event, _event_loop, control_flow| {
            use winit::event::Event;
            use winit::event_loop::ControlFlow;
            match &event {
                Event::WindowEvent{event, ..} => {
                    use winit::event::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }

                        WindowEvent::Resized(size) => {
                            // TODO: refactor this
                            let resize_event = self.events.resize_events.size.lock().unwrap().clone();
                            if size.width as i32 == resize_event.width && size.height as i32 == resize_event.height { return }

                            self.events.resize_events.resized.store(true, std::sync::atomic::Ordering::Relaxed);
                            *self.events.resize_events.size.lock().unwrap() = Size2D::new(size.width as i32, size.height as i32);
                            renderer.core.resize_window(Size2D::new(size.width as u32, size.height as u32));
                        }

                        WindowEvent::MouseInput{state: _, button: _, ..} => {
                        }

                        WindowEvent::CursorMoved{position, ..} => {
                            let position = Point2D::new(position.x as i32, position.y as i32);
                            self.events.mouse_events.mouse_position = position.clone();
                            if game.gui.is_on_gui(position) {
                                self.events.mouse_events.mouse_on_gui = true;
                            } else {
                                self.events.mouse_events.mouse_on_gui = false;
                                self.events.mouse_events.update_mouse_position_world(*self.events.resize_events.size.lock().unwrap(), &renderer.camera);
                            }
                        }

                        WindowEvent::MouseWheel{delta, ..} => {
                            use winit::event::MouseScrollDelta;
                            if let MouseScrollDelta::LineDelta(_x, z) = delta {
                                let wheel_count = *z as i32;
                                renderer.camera_future.size = renderer.camera_future.size * (1.0 + (wheel_count as f32) * -0.1);
                                renderer.camera_future.size = renderer.camera_future.size.max(1.0);
                                renderer.camera_future.size = renderer.camera_future.size.min(game.settings.max_camera);
                            }
                        }

                        WindowEvent::KeyboardInput{input, ..} => {
                            use winit::event::ElementState;
                            use winit::event::VirtualKeyCode;
                            match input.state {
                                ElementState::Pressed => {
                                    if let Some(keycode) = input.virtual_keycode {
                                        match keycode {
                                            #[cfg(not(feature = "shipping"))]
                                            VirtualKeyCode::RControl => { *crate::DEBUG.get_mut() = !crate::DEBUG.get(); }
                                            //VirtualKeyCode::Escape => { *control_flow = ControlFlow::Exit }
                                            _ => {}
                                        }
                                    }
                                }

                                ElementState::Released => { }
                            }
                        }
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    let waiting_thread = game.step.waiting.lock();
                    if let Some(thread) = waiting_thread.as_ref() {
                        if thread.0.elapsed() > thread.1 && thread.0.elapsed() > thread.2 { thread.3.send(()).unwrap(); }
                    }

                    {
                        let mut parkers = self.events.parker.lock().unwrap();
                        for parker in parkers.iter() {
                            parker.park();
                        }
                        parkers.clear();
                    }

                    {
                        let mut barriers = self.events.barrier.lock().unwrap();
                        for barrier in barriers.iter() {
                            barrier.wait();
                        }
                        barriers.clear();
                    }
                    *control_flow = ControlFlow::Poll;
                }
                
                Event::RedrawRequested(_) => {
                    //dbg!();
                }
                _ => {}
            }

            if let Some(event) = self.events.input_events.update_and_return_event(&event) {
                let camera;
                match event {
                    input_events::event::Event::Keyboard(_) => { camera = Some((renderer.camera.clone(), self.events.mouse_events.mouse_position_world)); }
                    input_events::event::Event::MouseButton(_) => { camera = Some((renderer.camera.clone(), self.events.mouse_events.mouse_position_world)); }
                    input_events::event::Event::MouseMoved(_) => { camera =  Some((renderer.camera.clone(), self.events.mouse_events.mouse_position_world)); }
                    input_events::event::Event::MouseWheel(_) => { camera =  Some((renderer.camera.clone(), self.events.mouse_events.mouse_position_world)); }
                }

                self.events.buffer_events.push((event.clone(), camera));
            }
                
            if game.state == crate::game::GameState::Editor {
                if last_camera_move_time.elapsed() > std::time::Duration::from_micros(100) {
                    if self.events.input_events.state.keyboard.is_pressed(winit::event::VirtualKeyCode::Up) { renderer.camera_future.y -= (1_000.0 * renderer.camera_future.size * last_event_time.elapsed().as_secs_f32()).ceil() as i32; }
                    if self.events.input_events.state.keyboard.is_pressed(winit::event::VirtualKeyCode::Down) { renderer.camera_future.y += (1_000.0 * renderer.camera_future.size * last_event_time.elapsed().as_secs_f32()).ceil() as i32; }
                    if self.events.input_events.state.keyboard.is_pressed(winit::event::VirtualKeyCode::Left) { renderer.camera_future.x -= (1_000.0 * renderer.camera_future.size * last_event_time.elapsed().as_secs_f32()).ceil() as i32; }
                    if self.events.input_events.state.keyboard.is_pressed(winit::event::VirtualKeyCode::Right) { renderer.camera_future.x += (1_000.0 * renderer.camera_future.size * last_event_time.elapsed().as_secs_f32()).ceil() as i32; }
                    last_camera_move_time = std::time::Instant::now();
                }
            }

            // Handle event on imgui
            game.gui.handle_events(&self.window, event);

            if !crate::APP.get().app_runner_infos.running() {
                *control_flow = ControlFlow::Exit;
            }

            last_event_time = std::time::Instant::now();
        });
    }
}