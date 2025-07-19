use std::sync::{Arc, Mutex};

use crate::{prim_shape::PrimShape, vec2::FloatVec2};

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
    pub center: FloatVec2,
    pub trace_shape_renderables: Vec<RenderableBatch>,
    pub pad_shape_renderables: Vec<ShapeRenderable>,
    pub other_shape_renderables: Vec<ShapeRenderable>,
}


pub trait UpdatePcbRenderModel {
    fn update_pcb_render_model(&self, pcb_render_model: PcbRenderModel);
}

