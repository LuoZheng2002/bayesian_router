use std::{sync::{Arc, Mutex}, time::Instant};

use cgmath::{Euler, Quaternion};
use shared::{pcb_render_model::PcbRenderModel, prim_shape::{CircleShape, PrimShape, RectangleShape}};

use crate::{
    orthographic_camera::OrthographicCamera, render_context::RenderContext, shape_instance::ShapeInstance, shape_mesh::ShapeMesh, transparent_pipeline::TransparentShapeBatch
};

// model path,
pub struct State {
    // camera stuff
    pub camera: OrthographicCamera,
    // accumulated time
    pub timer: Option<Instant>,
    pub prev_time: Option<f32>,
    pub fps_timer: Option<Instant>,
    pub cursor_timer: Option<Instant>,
    pub accumulated_frame_num: u32,
    pub transparent_shape_submissions: Option<Vec<TransparentShapeBatch>>,
    pub fps: u32,

    // pub pcb_width: f32,
    // pub pcb_height: f32,
}

impl State {
    pub fn init(&mut self) {}

    pub fn update(
        &mut self, 
        render_context: &RenderContext,
        pcb_render_model: Arc<Mutex<PcbRenderModel>>,
    ) {
        // calculate fps every 1 second
        let fps_timer = self.fps_timer.get_or_insert_with(|| Instant::now());
        let cursor_timer = self.cursor_timer.get_or_insert_with(|| Instant::now());
        let current_fps_time = fps_timer.elapsed().as_secs_f32();
        if current_fps_time >= 1.0 {
            // println!("FPS: {}", self.accumulated_frame_num);
            self.fps = self.accumulated_frame_num;
            self.accumulated_frame_num = 0;
            *fps_timer = Instant::now();
        } else {
            self.accumulated_frame_num += 1;
        }
        let current_cursor_time = cursor_timer.elapsed().as_secs_f32();
        // let mut cursor_blink = false;
        if current_cursor_time >= 0.5 {
            // println!("cursor: {:?}", input_context.mouse_position());
            *cursor_timer = Instant::now();
            // cursor_blink = true;
        }
        let timer = self.timer.get_or_insert_with(|| Instant::now());
        let current_time = timer.elapsed().as_secs_f32();
        let prev_time = self.prev_time.get_or_insert(current_time);
        let delta_time = current_time - *prev_time;
        assert!(delta_time >= 0.0);
        // let speed = 0.1;
        // let delta_angle = current_time * speed;
        let pcb_render_model = pcb_render_model.lock().unwrap();
        let pcb_width = pcb_render_model.width;
        let pcb_height = pcb_render_model.height;
        let pcb_center_x = pcb_render_model.center.x;
        let pcb_center_y = pcb_render_model.center.y;
        // update camera
        let pcb_aspect_ratio = pcb_width / pcb_height;
        let screen_aspect_ratio = {
            let size = *render_context.size.borrow();
            size.width as f32 / size.height as f32
        };
        let pcb_margin_scale: f32 = 1.2;
        let (orthographic_width, orthographic_height) = {
            if pcb_aspect_ratio > screen_aspect_ratio {
                let orthographic_width = pcb_width * pcb_margin_scale;
                let orthographic_height = orthographic_width / screen_aspect_ratio;
                (orthographic_width, orthographic_height)
            } else {
                let orthographic_height = pcb_height * pcb_margin_scale;
                let orthographic_width = orthographic_height * screen_aspect_ratio;
                (orthographic_width, orthographic_height)
            }
        };
        // the top left corner of the PCB is (0, 0) in world space
        // self.camera.left = -orthographic_width / 2.0 + self.pcb_width / 2.0;
        // self.camera.right = orthographic_width / 2.0 + self.pcb_width / 2.0;
        // self.camera.bottom = -orthographic_height / 2.0 + self.pcb_height / 2.0;
        // self.camera.top = orthographic_height / 2.0 + self.pcb_height / 2.0;
        self.camera.left = -orthographic_width / 2.0 + pcb_center_x;
        self.camera.right = orthographic_width / 2.0 + pcb_center_x;
        self.camera.bottom = -orthographic_height / 2.0 + pcb_center_y;
        self.camera.top = orthographic_height / 2.0 + pcb_center_y;
        // render submissions
        let circle_mesh = render_context.circle_mesh.clone();
        let rect_mesh = render_context.square_mesh.clone();
        let submissions = pcb_render_model_to_transparent_shape_submissions(&pcb_render_model, circle_mesh, rect_mesh);
        self.transparent_shape_submissions = Some(submissions);
        // self.transparent_shape_submissions = Some(vec![pcb_rect_batch]);
    }
}

pub fn pcb_render_model_to_transparent_shape_submissions(
    pcb_render_model: &PcbRenderModel,
    circle_mesh: Arc<ShapeMesh>,
    rect_mesh: Arc<ShapeMesh>,
) -> Vec<TransparentShapeBatch> {
    
    let mut submissions = Vec::new();

    // Add PCB rectangle
    let pcb_center_x = pcb_render_model.center.x;
    let pcb_center_y = pcb_render_model.center.y;
    let pcb_rect_instance = ShapeInstance {
        position: [pcb_center_x, pcb_center_y, 0.0].into(),
        rotation: Quaternion::from(Euler::new(
            cgmath::Deg(0.0),
            cgmath::Deg(0.0),
            cgmath::Deg(0.0),
        )),
        scale: cgmath::Vector3::new(pcb_render_model.width, pcb_render_model.height, 1.0),
        color: [1.0, 1.0, 1.0, 0.3],
    };
    let pcb_rect_batch = TransparentShapeBatch(vec![(rect_mesh.clone(), vec![pcb_rect_instance])]);
    submissions.push(pcb_rect_batch);

   
    // add traces
    for trace in &pcb_render_model.trace_shape_renderables {
        let mut circle_instances: Vec<ShapeInstance> = Vec::new();
        let mut rect_instances: Vec<ShapeInstance> = Vec::new();
        for renderable in &trace.0{
            let color = renderable.color;
            match &renderable.shape{
                PrimShape::Circle(circle_shape) =>{
                    let CircleShape { diameter, position } = circle_shape;
                    let circle_instance = ShapeInstance {
                        position: [position.x, position.y, 0.0].into(),
                        rotation: Quaternion::from(Euler::new(
                            cgmath::Deg(0.0),
                            cgmath::Deg(0.0),
                            cgmath::Deg(0.0),
                        )),
                        scale: cgmath::Vector3::new(*diameter, *diameter, 1.0),
                        color,
                    };
                    circle_instances.push(circle_instance);
                },
                PrimShape::Rectangle(rect_shape) => {
                    let RectangleShape { width, height , position, rotation} = rect_shape;
                    let rect_instance = ShapeInstance {
                        position: [position.x, position.y, 0.0].into(),
                        rotation: Quaternion::from(Euler::new(
                            cgmath::Deg(0.0),
                            cgmath::Deg(0.0),
                            *rotation,
                        )),
                        scale: cgmath::Vector3::new(*width, *height, 1.0),
                        color,
                    };
                    rect_instances.push(rect_instance);
                },
            }
        }      
        let mut batch_contents = Vec::new();
        if !circle_instances.is_empty() {
            batch_contents.push((circle_mesh.clone(), circle_instances));
        }
        if !rect_instances.is_empty() {
            batch_contents.push((rect_mesh.clone(), rect_instances));
        }  
        if !batch_contents.is_empty() {
            let trace_batch = TransparentShapeBatch(batch_contents);
            submissions.push(trace_batch);
        }
    }
     // Add pads
    for renderable in &pcb_render_model.pad_shape_renderables {
        let color = renderable.color;
        match &renderable.shape{
            PrimShape::Circle(circle_shape) =>{
                let CircleShape { diameter, position } = circle_shape;
                let circle_instance = ShapeInstance {
                    position: [position.x, position.y, 0.0].into(),
                    rotation: Quaternion::from(Euler::new(
                        cgmath::Deg(0.0),
                        cgmath::Deg(0.0),
                        cgmath::Deg(0.0),
                    )),
                    scale: cgmath::Vector3::new(*diameter, *diameter, 1.0),
                    color,
                };
                let circle_batch = TransparentShapeBatch(vec![(circle_mesh.clone(), vec![circle_instance])]);
                submissions.push(circle_batch);
            },
            PrimShape::Rectangle(rect_shape) => {
                let RectangleShape { width, height , position, rotation} = rect_shape;
                let rect_instance = ShapeInstance {
                    position: [position.x, position.y, 0.0].into(),
                    rotation: Quaternion::from(Euler::new(
                        cgmath::Deg(0.0),
                        cgmath::Deg(0.0),
                        *rotation,
                    )),
                    scale: cgmath::Vector3::new(*width, *height, 1.0),
                    color,
                };
                let rect_batch = TransparentShapeBatch(vec![(rect_mesh.clone(), vec![rect_instance])]);
                submissions.push(rect_batch);
            },
        }
    }
    submissions
}

impl Default for State {
    fn default() -> Self {
        State {
            camera: OrthographicCamera::new(
                cgmath::Point3::new(0.0, 0.0, 1.0),
                cgmath::Point3::new(0.0, 0.0, 0.0),
                cgmath::Vector3::new(0.0, 1.0, 0.0),
                -1.0,
                1.0,
                -1.0,
                1.0,
                -10.0,
                10.0,
            ),
            timer: None,
            prev_time: None,
            fps_timer: None,
            cursor_timer: None,
            accumulated_frame_num: 0,
            transparent_shape_submissions: None,
            fps: 0,
            // pcb_width: 15.0,
            // pcb_height: 10.0,
        }
    }
}
