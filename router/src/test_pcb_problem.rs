// use cgmath::Deg;
// use shared::{
//     pad::{Pad, PadLayer, PadName, PadShape},
//     pcb_problem::{NetName, PcbProblem},
//     vec2::FloatVec2,
// };

// pub fn pcb_problem1() -> PcbProblem {
//     let mut pcb_problem = PcbProblem::new(15.0, 15.0, FloatVec2 { x: 0.0, y: 0.0 }, 1, 1.0);
//     let red_net_name = NetName("red".into());
//     pcb_problem.add_net(
//         red_net_name.clone(),
//         Pad {
//             name: PadName("red_source".into()),
//             position: FloatVec2 { x: -6.0, y: 0.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.05,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.05,
//         0.8,
//     );
//     let red_sink_pad1 = Pad {
//         name: PadName("red_sink1".into()),
//         position: FloatVec2 { x: -3.0, y: 5.0 },
//         shape: PadShape::Rectangle {
//             width: 1.0,
//             height: 1.0,
//         },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     let mut red_sink_pad2 = red_sink_pad1.clone();
//     red_sink_pad2.position = FloatVec2 { x: 0.0, y: 5.0 };
//     let mut red_sink_pad3 = red_sink_pad1.clone();
//     red_sink_pad3.position = FloatVec2 { x: 3.0, y: 5.0 };
//     let mut red_sink_pad4 = red_sink_pad1.clone();
//     red_sink_pad4.position = FloatVec2 { x: 6.0, y: 5.0 };
//     pcb_problem.add_connection(red_net_name.clone(), red_sink_pad1, 0.5, 0.05);
//     pcb_problem.add_connection(red_net_name.clone(), red_sink_pad2, 0.5, 0.05);
//     pcb_problem.add_connection(red_net_name.clone(), red_sink_pad3, 0.5, 0.05);
//     pcb_problem.add_connection(red_net_name.clone(), red_sink_pad4, 0.5, 0.05);
//     let purple_net_name = NetName("purple".into());
//     pcb_problem.add_net(
//         purple_net_name.clone(),
//         Pad {
//             name: PadName("purple_source".into()),
//             position: FloatVec2 { x: -6.0, y: -1.0 },
//             shape: PadShape::Circle { diameter: 0.8 },
//             rotation: Deg(0.0),
//             clearance: 0.05,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.05,
//         0.8,
//     );
//     let purple_sink_pad1 = Pad {
//         name: PadName("purple_sink1".into()),
//         position: FloatVec2 { x: -2.0, y: -3.0 },
//         shape: PadShape::Circle { diameter: 0.8 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     let purple_sink_pad2 = Pad {
//         name: PadName("purple_sink2".into()),
//         position: FloatVec2 { x: 4.0, y: -3.0 },
//         shape: PadShape::Circle { diameter: 0.8 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     pcb_problem.add_connection(purple_net_name.clone(), purple_sink_pad1, 0.5, 0.05);
//     pcb_problem.add_connection(purple_net_name.clone(), purple_sink_pad2, 0.5, 0.05);
//     let blue_net_name = NetName("blue".into());
//     pcb_problem.add_net(
//         blue_net_name.clone(),
//         Pad {
//             name: PadName("blue_source".into()),
//             position: FloatVec2 { x: -2.0, y: -1.0 },
//             shape: PadShape::Circle { diameter: 0.8 },
//             rotation: Deg(0.0),
//             clearance: 0.05,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.05,
//         0.8,
//     );
//     let blue_sink_pad1 = Pad {
//         name: PadName("blue_sink1".into()),
//         position: FloatVec2 { x: 0.0, y: 0.0 },
//         shape: PadShape::Circle { diameter: 0.8 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     let blue_sink_pad2 = Pad {
//         name: PadName("blue_sink2".into()),
//         position: FloatVec2 { x: -3.0, y: 0.0 },
//         shape: PadShape::Circle { diameter: 0.8 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     pcb_problem.add_connection(blue_net_name.clone(), blue_sink_pad1, 0.3, 0.05);
//     pcb_problem.add_connection(blue_net_name.clone(), blue_sink_pad2, 0.3, 0.05);
//     let gray_net_name = NetName("gray".into());
//     pcb_problem.add_net(
//         gray_net_name.clone(),
//         Pad {
//             name: PadName("gray_source".into()),
//             position: FloatVec2 { x: -6.0, y: -2.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.05,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.05,
//         0.8,
//     );
//     let gray_sink_pad = Pad {
//         name: PadName("gray_sink".into()),
//         position: FloatVec2 { x: -2.0, y: -2.0 },
//         shape: PadShape::Circle { diameter: 0.6 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     pcb_problem.add_connection(gray_net_name.clone(), gray_sink_pad, 0.2, 0.05);
//     let brown_net_name = NetName("brown".into());

//     pcb_problem.add_net(
//         brown_net_name.clone(),
//         Pad {
//             name: PadName("brown_source".into()),
//             position: FloatVec2 { x: -6.0, y: -3.0 },
//             shape: PadShape::Circle { diameter: 0.8 },
//             rotation: Deg(0.0),
//             clearance: 0.05,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.05,
//         0.8,
//     );
//     let brown_sink_pad = Pad {
//         name: PadName("brown_sink".into()),
//         position: FloatVec2 { x: 4.0, y: -2.0 },
//         shape: PadShape::Circle { diameter: 0.8 },
//         rotation: Deg(0.0),
//         clearance: 0.05,
//         pad_layer: PadLayer::Front,
//     };
//     pcb_problem.add_connection(brown_net_name, brown_sink_pad, 0.2, 0.05);
//     pcb_problem
// }

// pub fn pcb_problem2() -> PcbProblem {
//     let mut pcb_problem = PcbProblem::new(20.0, 20.0, FloatVec2 { x: 0.0, y: 0.0 }, 1, 1.0);
//     let red_net_name = NetName("red".into());
//     let green_net_name = NetName("green".into());
//     let blue_net_name = NetName("blue".into());
//     let yellow_net_name = NetName("yellow".into());
//     pcb_problem.add_net(
//         red_net_name.clone(),
//         Pad {
//             name: PadName("red_source".into()),
//             position: FloatVec2 { x: -6.0, y: 3.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.1,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.2,
//         0.8,
//     );
//     pcb_problem.add_net(
//         green_net_name.clone(),
//         Pad {
//             name: PadName("green_source".into()),
//             position: FloatVec2 { x: -6.0, y: -3.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.1,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.2,
//         0.8,
//     );
//     pcb_problem.add_net(
//         blue_net_name.clone(),
//         Pad {
//             name: PadName("blue_source".into()),
//             position: FloatVec2 { x: -3.0, y: 6.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.1,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.2,
//         0.8,
//     );
//     pcb_problem.add_net(
//         yellow_net_name.clone(),
//         Pad {
//             name: PadName("yellow_source".into()),
//             position: FloatVec2 { x: 3.0, y: 6.0 },
//             shape: PadShape::Circle { diameter: 0.6 },
//             rotation: Deg(0.0),
//             clearance: 0.1,
//             pad_layer: PadLayer::Front,
//         },
//         0.5,
//         0.2,
//         0.8,
//     );

//     let red_sink_pad = Pad {
//         name: PadName("red_sink".into()),
//         position: FloatVec2 { x: 6.0, y: 3.0 },
//         shape: PadShape::Rectangle {
//             width: 0.8,
//             height: 0.8,
//         },
//         rotation: Deg(0.0),
//         clearance: 0.2,
//         pad_layer: PadLayer::Front,
//     };
//     let green_sink_pad = Pad {
//         name: PadName("green_sink".into()),
//         position: FloatVec2 { x: 6.0, y: -3.0 },
//         shape: PadShape::Circle { diameter: 0.6 },
//         rotation: Deg(0.0),
//         clearance: 0.1,
//         pad_layer: PadLayer::Front,
//     };
//     let blue_sink_pad = Pad {
//         name: PadName("blue_sink".into()),
//         position: FloatVec2 { x: -3.0, y: -6.0 },
//         shape: PadShape::Circle { diameter: 0.6 },
//         rotation: Deg(0.0),
//         clearance: 0.15,
//         pad_layer: PadLayer::Front,
//     };
//     let yellow_sink_pad = Pad {
//         name: PadName("yellow_sink".into()),
//         position: FloatVec2 { x: 3.0, y: -6.0 },
//         shape: PadShape::Circle { diameter: 0.6 },
//         rotation: Deg(0.0),
//         clearance: 0.1,
//         pad_layer: PadLayer::Front,
//     };
//     pcb_problem.add_connection(red_net_name.clone(), red_sink_pad, 0.5, 0.2);
//     pcb_problem.add_connection(green_net_name.clone(), green_sink_pad, 0.7, 0.05);
//     pcb_problem.add_connection(blue_net_name.clone(), blue_sink_pad, 0.6, 0.3);
//     pcb_problem.add_connection(yellow_net_name.clone(), yellow_sink_pad, 0.4, 0.1);
//     pcb_problem
// }
