use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use shared::pcb_render_model::PcbRenderModel;

use crate::{input_context::InputContext, render_context::RenderContext, state::State};

#[derive(Default)]
pub struct Context {
    pub render_context: Option<RenderContext>,
    pub state: RefCell<State>,
    pub input_context: RefCell<InputContext>,
    pub pcb_render_model: Arc<Mutex<Option<PcbRenderModel>>>,
    pub working_thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    pub command_thread: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}
