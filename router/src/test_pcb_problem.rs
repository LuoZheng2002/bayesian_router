use cgmath::Deg;
use shared::pcb_problem::PcbProblem;






pub fn pcb_problem1()->PcbProblem{
    let mut pcb_problem = PcbProblem::new(15.0, 15.0);
    let red_net_id = pcb_problem.add_net(Color{r: 255, g: 0, b: 0});
    let red_source_pad = Pad{
        position: FloatVec2{x: -6.0, y: 0.0},
        shape: PadShape::Circle { diameter: 0.6},
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let red_sink_pad1 = Pad { 
        position: FloatVec2 { x: -3.0, y: 5.0 }, 
        shape: PadShape::Square { 
            side_length: 1.0,
        },
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let mut red_sink_pad2 = red_sink_pad1.clone();
    red_sink_pad2.position = FloatVec2 { x: 0.0, y: 5.0 };
    let mut red_sink_pad3 = red_sink_pad1.clone();
    red_sink_pad3.position = FloatVec2 { x: 3.0, y: 5.0 };
    let mut red_sink_pad4 = red_sink_pad1.clone();
    red_sink_pad4.position = FloatVec2 { x: 6.0, y: 5.0 };
    pcb_problem.add_connection(
        red_net_id, 
        red_source_pad.clone(),
        red_sink_pad1,
        0.5,
        0.05,
    );
    pcb_problem.add_connection(
        red_net_id, 
        red_source_pad.clone(),
        red_sink_pad2,
        0.5,
        0.05,
    );
    pcb_problem.add_connection(
        red_net_id, 
        red_source_pad.clone(),
        red_sink_pad3,
        0.5,
        0.05,
    );
    pcb_problem.add_connection(
        red_net_id, 
        red_source_pad.clone(),
        red_sink_pad4,
        0.5,
        0.05,
    );
    let purple_net_id = pcb_problem.add_net(Color{r: 128, g: 0, b: 128});
    let purple_source_pad = Pad{
        position: FloatVec2{x: -6.0, y: -1.0},
        shape: PadShape::Circle { diameter: 0.8 },
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let mut purple_sink_pad1 = purple_source_pad.clone();
    purple_sink_pad1.position = FloatVec2 { x: -2.0, y: -3.0 };
    let mut purple_sink_pad2 = purple_sink_pad1.clone();
    purple_sink_pad2.position = FloatVec2 { x: 4.0, y: -3.0 };
    pcb_problem.add_connection(
        purple_net_id, 
        purple_source_pad.clone(),
        purple_sink_pad1,
        0.5,
        0.05,
    );
    pcb_problem.add_connection(
        purple_net_id, 
        purple_source_pad.clone(),
        purple_sink_pad2,
        0.5,
        0.05,
    );
    let blue_net_id = pcb_problem.add_net(Color{r: 0, g: 0, b: 255});
    let blue_source_pad = Pad{
        position: FloatVec2{x: -2.0, y: -1.0},
        shape: PadShape::Circle { diameter: 0.8 },
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let mut blue_sink_pad1 = blue_source_pad.clone();
    blue_sink_pad1.position = FloatVec2 { x: 0.0, y: 0.0 };
    let mut blue_sink_pad2 = blue_sink_pad1.clone();
    blue_sink_pad2.position = FloatVec2 { x: -3.0, y: 0.0 };
    pcb_problem.add_connection(
        blue_net_id, 
        blue_source_pad.clone(),
        blue_sink_pad1,
        0.3,
        0.05,
    );
    pcb_problem.add_connection(
        blue_net_id, 
        blue_source_pad.clone(),
        blue_sink_pad2,
        0.3,
        0.05,
    );
    let gray_net_id = pcb_problem.add_net(Color{r: 128, g: 128, b: 128});
    let gray_source_pad = Pad{
        position: FloatVec2{x: -6.0, y: -2.0},
        shape: PadShape::Circle { diameter: 0.6 },
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let mut gray_sink_pad = gray_source_pad.clone();
    gray_sink_pad.position = FloatVec2 { x: -2.0, y: -2.0};
    pcb_problem.add_connection(
        gray_net_id, 
        gray_source_pad.clone(),
        gray_sink_pad,
        0.2,
        0.05,
    );

    let brown_net_id = pcb_problem.add_net(Color{r: 165, g: 42, b: 42});
    let brown_source_pad = Pad{
        position: FloatVec2{x: -6.0, y: -3.0},
        shape: PadShape::Circle { diameter: 0.8 },
        rotation: Deg(0.0),
        clearance: 0.05,
    };
    let mut brown_sink_pad = brown_source_pad.clone();
    brown_sink_pad.position = FloatVec2 { x: 4.0, y: -2.0};
    pcb_problem.add_connection(
        brown_net_id, 
        brown_source_pad.clone(),
        brown_sink_pad,
        0.2,
        0.05,
    );
    pcb_problem
}

pub fn pcb_problem2()->PcbProblem{
    let mut pcb_problem = PcbProblem::new(20.0, 20.0);
    let red_net_id = pcb_problem.add_net(Color{r: 255, g: 0, b: 0});
    let green_net_id = pcb_problem.add_net(Color{r: 0, g: 255, b: 0});
    let blue_net_id = pcb_problem.add_net(Color{r: 0, g: 0, b: 255});
    let yellow_net_id = pcb_problem.add_net(Color{r: 255, g: 255, b: 0});
    let pad = Pad{
        position: FloatVec2{x: 0.0, y: 0.0},
        shape: PadShape::Circle { diameter: 0.6},
        rotation: Deg(0.0),
        clearance: 0.1,
    };
    let mut red_source_pad = pad.clone();
    red_source_pad.position = FloatVec2 { x: -6.0, y: 3.0 };
    red_source_pad.clearance = 0.2;
    red_source_pad.shape = PadShape::Square { side_length: 0.8 };
    let mut red_sink_pad = pad.clone();
    red_sink_pad.clearance = 0.2;
    red_sink_pad.shape = PadShape::Square { side_length: 0.8 };
    red_sink_pad.position = FloatVec2 { x: 6.0, y: 3.0 };
    let mut green_source_pad = pad.clone();
    green_source_pad.position = FloatVec2 { x: -6.0, y: -3.0 };
    let mut green_sink_pad = pad.clone();
    green_sink_pad.position = FloatVec2 { x: 6.0, y: -3.0 };
    let mut blue_source_pad = pad.clone();
    blue_source_pad.position = FloatVec2 { x: -3.0, y: 6.0 };
    blue_source_pad.clearance = 0.15;
    let mut blue_sink_pad = pad.clone();    
    blue_sink_pad.position = FloatVec2 { x: -3.0, y: -6.0 };
    blue_sink_pad.clearance = 0.15;
    let mut yellow_source_pad = pad.clone();
    yellow_source_pad.position = FloatVec2 { x: 3.0, y: 6.0 };
    let mut yellow_sink_pad = pad.clone();
    yellow_sink_pad.position = FloatVec2 { x: 3.0, y: -6.0 };
    pcb_problem.add_connection(
        red_net_id, 
        red_source_pad.clone(),
        red_sink_pad,
        0.5,
        0.2,
    );
    pcb_problem.add_connection(
        green_net_id, 
        green_source_pad.clone(),
        green_sink_pad,
        0.7,
        0.05,
    );
    pcb_problem.add_connection(
        blue_net_id, 
        blue_source_pad.clone(),
        blue_sink_pad,
        0.6,
        0.3,
    );
    pcb_problem.add_connection(
        yellow_net_id, 
        yellow_source_pad.clone(),
        yellow_sink_pad,
        0.4,
        0.1,
    );
    pcb_problem
}