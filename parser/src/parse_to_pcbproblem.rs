use shared::{
    distinct_color_generator::DistinctColorGenerator,
    pad::{Pad, PadName},
    pcb_problem::{Connection, ConnectionID, NetInfo, NetName, PcbProblem},
    vec2::FloatVec2,
};

// convert_to_problem.rs
use crate::{
    parse_to_display_format::{DisplayFormat, ExtraInfo},
    prim_mst::prim_mst,
};
use std::{collections::HashMap, rc::Rc};

pub struct Converter;

impl Converter {
    /// 将DisplayFormat转换为PcbProblem，应用ExtraInfo中的覆盖设置
    pub fn convert(
        display_format: &DisplayFormat,
        extra_info: &ExtraInfo,
    ) -> Result<PcbProblem, String> {
        // let mut problem = PcbProblem::new(
        //     display_format.width,
        //     display_format.height,
        //     display_format.center,
        //     display_format.num_layers,
        //     display_format.scale_down_factor,
        // );

        // 添加障碍物
        //problem.obstacle_lines = display_format.obstacle_lines;
        //problem.obstacle_polygons = display_format.obstacle_polygons;
        let mut nets: HashMap<NetName, NetInfo> = HashMap::new();
        let mut connection_id_generator = Box::new((0..).map(ConnectionID)); // A generator for ConnectionID, starting from 0
        let mut distinct_color_generator = Box::new(DistinctColorGenerator::new());
        // 处理每个网络
        for (net_name, display_net) in &display_format.nets {
            let source_pad = extra_info.net_name_to_source_pad.get(net_name).cloned();
            let pads: HashMap<PadName, Pad> = display_net
                .pads
                .iter()
                .map(|pad| (pad.name.clone(), pad.clone()))
                .collect();
            let connection_pairs: Vec<(PadName, PadName)> = if let Some(source_pad) = source_pad {
                let mut connection_pairs: Vec<(PadName, PadName)> = Vec::new();
                for pad in pads.values() {
                    if pad.name != source_pad {
                        connection_pairs.push((source_pad.clone(), pad.name.clone()));
                    }
                }
                connection_pairs
            } else {
                // mst
                let pad_positions: HashMap<PadName, FloatVec2> = pads
                    .iter()
                    .map(|(pad_name, pad)| (pad_name.clone(), pad.position))
                    .collect();
                prim_mst(pad_positions)
            };

            let mut connections: HashMap<ConnectionID, Rc<Connection>> = HashMap::new();
            for (start, end) in connection_pairs.iter() {
                let connection_id = connection_id_generator.next().unwrap();
                let connection = Connection {
                    net_name: net_name.clone(),
                    connection_id,
                    start_pad: start.clone(),
                    end_pad: end.clone(),
                };
                connections.insert(connection_id, Rc::new(connection));
            }
            let color = distinct_color_generator.next().unwrap();
            let net_info = NetInfo {
                net_name: net_name.clone(),
                color,
                pads,
                trace_width: display_net.default_trace_width,
                trace_clearance: display_net.default_trace_clearance,
                via_diameter: display_net.via_diameter,
                connections,
            };
            nets.insert(net_name.clone(), net_info);
        }
        let problem = PcbProblem {
            width: display_format.width,
            height: display_format.height,
            center: display_format.center,
            num_layers: display_format.num_layers,
            obstacle_borders: Vec::new(),
            obstacle_border_outlines: Vec::new(),
            obstacle_polygons: Vec::new(),
            nets, // netname, netinfo
            //connection_id_generator: Box::new((0..).map(ConnectionID)),
            scale_down_factor: display_format.scale_down_factor,
        };
        Ok(problem)
    }

    // 确定网络的source pad（优先使用extra_info中的设置）
    // fn determine_source_pad(
    //     net_name: &NetName,
    //     pads: &[Pad],
    //     net_to_source: &HashMap<NetName, PadName>,
    // ) -> Result<Pad, String> {
    //     // 1. 检查是否有用户指定的source pad
    //     if let Some(source_pad_name) = net_to_source.get(net_name) {
    //         return pads
    //             .iter()
    //             .find(|p| &p.name == source_pad_name)
    //             .cloned()
    //             .ok_or_else(|| {
    //                 format!(
    //                     "Specified source pad {} not found in net {}",
    //                     source_pad_name.0, net_name.0
    //                 )
    //             });
    //     }

    //     // 2. 自动选择第一个pad作为source（如果只有1个pad会报错）
    //     if pads.is_empty() {
    //         return Err(format!("Net {} has no pads", net_name.0));
    //     }

    //     if pads.len() == 1 {
    //         eprintln!("Warning: Net {} has only one pad", net_name.0);
    //     }

    //     Ok(pads[0].clone())
    // }

    // 获取trace设置（优先使用extra_info中的覆盖值）

    //  fn get_trace_settings(
    //         pad_name: &PadName,
    //         default_width: f32,
    //         default_clearance: f32,
    //         extra_info: &ExtraInfo,
    //     ) -> (f32, f32) {
    //         (
    //             extra_info
    //                 .pad_name_to_trace_width
    //                 .get(pad_name)
    //                 .copied()
    //                 .unwrap_or(default_width),
    //             extra_info
    //                 .pad_name_to_trace_clearance
    //                 .get(pad_name)
    //                 .copied()
    //                 .unwrap_or(default_clearance),
    //         )
    //     }
}
