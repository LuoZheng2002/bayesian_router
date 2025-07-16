use std::sync::{Arc, Mutex};

use crate::prim_shape::PrimShape;

#[derive(Debug, Clone)]
pub struct ShapeRenderable{
    pub shape: PrimShape,
    pub color: [f32; 4], // RGBA color
}

#[derive(Debug, Clone)]
pub struct RenderableBatch(pub Vec<ShapeRenderable>);

#[derive(Default)]
pub struct PcbRenderModel {
    pub width: f32,
    pub height: f32,
    pub trace_shape_renderables: Vec<RenderableBatch>,
    pub pad_shape_renderables: Vec<ShapeRenderable>,
}


pub trait UpdatePcbRenderModel {
    fn update_pcb_render_model(&self, pcb_render_model: PcbRenderModel);
}

impl UpdatePcbRenderModel for Arc<Mutex<PcbRenderModel>> {
    fn update_pcb_render_model(&self, pcb_render_model: PcbRenderModel) {
        let mut model = self.lock().unwrap();
        *model = pcb_render_model;
    }
}
