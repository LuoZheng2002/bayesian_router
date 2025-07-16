// convert_to_problem.rs
use crate::pad::Pad;
use crate::parse_to_display_format::{DisplayFormat, ExtraInfo};
use crate::{
    pad::PadName,
    pcb_problem::{Connection, ConnectionID, NetInfo, NetName, PcbProblem},
    shapes::{Line, Polygon},
};
use std::collections::HashMap;

pub struct Converter;

impl Converter {
    /// 将DisplayFormat转换为PcbProblem，应用ExtraInfo中的覆盖设置
    pub fn convert(
        display_format: DisplayFormat,
        extra_info: ExtraInfo,
    ) -> Result<PcbProblem, String> {
        let mut problem = PcbProblem::new(
            display_format.width,
            display_format.height,
            display_format.center,
        );

        // 添加障碍物
        //problem.obstacle_lines = display_format.obstacle_lines;
        //problem.obstacle_polygons = display_format.obstacle_polygons;

        // 处理每个网络
        for (net_name, display_net) in display_format.nets {
            // 确定source pad（优先使用extra_info中的设置）
            let source_pad = Self::determine_source_pad(
                &net_name,
                &display_net.pads,
                &extra_info.net_name_to_source_pad,
            )?;

            // 获取trace宽度和clearance（优先使用extra_info中的覆盖值）
            let (source_width, source_clearance) = Self::get_trace_settings(
                &source_pad.name,
                display_net.default_trace_width,
                display_net.default_trace_clearance,
                &extra_info,
            );

            // 添加网络
            problem.add_net(
                net_name.clone(),
                source_pad.clone(),
                source_width,
                source_clearance,
            );

            // 添加连接（sink pads）
            for pad in display_net.pads {
                // 跳过source pad
                if pad.name == source_pad.name {
                    continue;
                }

                let (sink_width, sink_clearance) = Self::get_trace_settings(
                    &pad.name,
                    display_net.default_trace_width,
                    display_net.default_trace_clearance,
                    &extra_info,
                );

                problem.add_connection(net_name.clone(), pad, sink_width, sink_clearance);
            }
        }

        Ok(problem)
    }

    /// 确定网络的source pad（优先使用extra_info中的设置）
    fn determine_source_pad(
        net_name: &NetName,
        pads: &[Pad],
        net_to_source: &HashMap<NetName, PadName>,
    ) -> Result<Pad, String> {
        // 1. 检查是否有用户指定的source pad
        if let Some(source_pad_name) = net_to_source.get(net_name) {
            return pads
                .iter()
                .find(|p| &p.name == source_pad_name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "Specified source pad {} not found in net {}",
                        source_pad_name.0, net_name.0
                    )
                });
        }

        // 2. 自动选择第一个pad作为source（如果只有1个pad会报错）
        if pads.is_empty() {
            return Err(format!("Net {} has no pads", net_name.0));
        }

        if pads.len() == 1 {
            eprintln!("Warning: Net {} has only one pad", net_name.0);
        }

        Ok(pads[0].clone())
    }

    /// 获取trace设置（优先使用extra_info中的覆盖值）
    fn get_trace_settings(
        pad_name: &PadName,
        default_width: f32,
        default_clearance: f32,
        extra_info: &ExtraInfo,
    ) -> (f32, f32) {
        (
            extra_info
                .pad_name_to_trace_width
                .get(pad_name)
                .copied()
                .unwrap_or(default_width),
            extra_info
                .pad_name_to_trace_clearance
                .get(pad_name)
                .copied()
                .unwrap_or(default_clearance),
        )
    }
}
