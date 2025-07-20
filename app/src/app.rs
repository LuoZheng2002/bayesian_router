use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalPosition,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{command_thread, context::Context, render_context::RenderContext, working_thread_fn};

// thread_local! {
//     pub static RENDER_CONTEXT: RefCell<Option<RenderContext>> = RefCell::new(None);
//     pub static STATE: RefCell<State> = RefCell::new(State::default());
//     pub static INPUT_CONTEXT: RefCell<InputContext> = RefCell::new(InputContext::default());
// }

#[derive(Default)]
pub struct App {
    pub window: Option<Arc<Window>>,
    pub context: Context,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .with_position(LogicalPosition::new(0, 0));
        let window = event_loop.create_window(attributes).unwrap();
        let window = Arc::new(window);
        self.context.render_context = Some(RenderContext::new(window.clone()));
        let mut state = self.context.state.borrow_mut();
        state.init();
        self.window = Some(window);
        let pcb_render_model = self.context.pcb_render_model.clone();
        let mut working_thread = self.context.working_thread.lock().unwrap();
        *working_thread = Some(std::thread::spawn(move || {
            working_thread_fn::working_thread_fn(pcb_render_model);
        }));
        let mut command_thread = self.context.command_thread.lock().unwrap();
        *command_thread = Some(std::thread::spawn(move || {
            command_thread::command_thread_fn();
        }));
    }
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let mut input_context = self.context.input_context.borrow_mut();
        input_context.handle_device_event(&event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut input_context = self.context.input_context.borrow_mut();
        input_context.handle_window_event(&event);
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                // std::process::abort();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
                let render_context = self.context.render_context.as_ref().unwrap();
                let size = *render_context.size.borrow();
                let mut state = self.context.state.borrow_mut();
                state.update(render_context, self.context.pcb_render_model.clone());
                match render_context.render(&state) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        render_context.resize(size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        let mut working_thread = self.context.working_thread.lock().unwrap();
                        let working_thread = working_thread.take().unwrap();
                        working_thread.join().unwrap();
                        event_loop.exit();
                    }

                    // This happens when the a frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                let render_context = self.context.render_context.as_mut().unwrap();
                render_context.resize(new_size);
            }
            _ => (),
        }
    }
}
