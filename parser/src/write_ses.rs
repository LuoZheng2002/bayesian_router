use crate::dsn_struct::{DsnStruct, Library, Network, Shape};
use shared::pcb_problem::{FixedTrace, PcbSolution};
use shared::trace_path::Via;
use shared::vec2::FixedVec2;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};

fn generate_placement<W: Write>(file: &mut W, dsn: &DsnStruct) -> Result<()> {
    writeln!(file, "  (placement")?;
    writeln!(
        file,
        "    (resolution {} {})",
        dsn.resolution.unit, dsn.resolution.value
    )?;
    for component in &dsn.placement.components {
        writeln!(file, "    (component \"{}\"", component.name).unwrap();

        for inst in &component.instances {
            writeln!(
                file,
                "      (place {} {:.6} {:.6} {:?} {:.6})",
                inst.reference,
                inst.position.x,
                inst.position.y,
                inst.placement_layer.as_str(),
                inst.rotation
            )?;
        }
        writeln!(file, "    )\n")?;
    }
    writeln!(file, "  )\n")?;
    Ok(())
}

pub struct ViaSES {
    name: String,
    shape: String,
    through_hole: bool,
    diameter: f32,
}

impl ViaSES {
    fn to_ses_string(&self, layers: &[String]) -> String {
        let shape = &self.shape;
        let dia_int = (self.diameter * 10000.0).round() as i32;
        let mut s = format!("      (padstack \"{}\"\n", self.name);

        if self.through_hole {
            for layer in layers {
                s += &format!(
                    "        (shape\n          ({} {} {} 0 0)\n        )\n",
                    shape, layer, dia_int
                );
            }
        } else {
            let layer = &layers[0];
            s += &format!(
                "        (shape\n          ({} {} {} 0 0)\n        )\n",
                shape, layer, dia_int
            );
        }

        s += &format!("        (attach off)\n      )\n");

        s
    }
}

fn via_info(dsn: &DsnStruct) -> Vec<ViaSES> {
    dsn.library
        .pad_stacks
        .iter()
        .filter_map(|(name, pad)| {
            if name.starts_with("Via") {
                if let Shape::Circle { diameter } = pad.shape {
                    Some(ViaSES {
                        name: name.clone(),
                        shape: "circle".to_string(),
                        through_hole: pad.through_hole,
                        diameter,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn extract_fixed_vec2(v: &FixedVec2) -> (f32, f32) {
    (v.x.to_num::<f32>(), v.y.to_num::<f32>())
}

fn find_via_name(netname: &String, dsn: &DsnStruct) -> Option<String> {
    for netclass in dsn.network.netclasses.values() {
        if netclass.net_names.contains(netname) {
            return Some(netclass.via_name.clone());
        }
    }
    None
}

fn generate_network<W: Write>(
    file: &mut W,
    dsn: &DsnStruct,
    solution: &PcbSolution,
    layers: &Vec<String>,
    vias: &Vec<ViaSES>,
) -> Result<()> {
    // This function will generate the network information based on the PcbProblem and PcbSolution
    // The implementation will depend on the specific requirements of the network format

    let mut nets: HashMap<&String, Vec<&FixedTrace>> = HashMap::new();
    for trace in solution.determined_traces.values() {
        nets.entry(&trace.net_name.0).or_default().push(trace);
    }

    for (net_name, traces) in nets {
        writeln!(file, "  (net \"{}\")", net_name).unwrap();
        let via_name = find_via_name(&net_name, &dsn).unwrap_or("default_via".to_string());

        for trace in traces {
            for via in &trace.trace_path.vias {
                let (x, y) = extract_fixed_vec2(&via.position);
                writeln!(file, "    (via {} {} {})", via_name, x, y)?;
            }
            for segment in &trace.trace_path.segments {
                let (start_x, start_y) = extract_fixed_vec2(&segment.start);
                let (end_x, end_y) = extract_fixed_vec2(&segment.end);
                let layer_name = layers[segment.layer].as_str();
                writeln!(
                    file,
                    "        (wire\n          (path {} {}\n            {} {}\n            {} {}))",
                    layer_name, // 0 = front, highest = back
                    segment.width,
                    start_x,
                    start_y,
                    end_x,
                    end_y
                )?;
            }
        }
        writeln!(file, "    )")?;
    }
    writeln!(file, "  )")?;
    Ok(())
}

pub fn write_ses(dsn: &DsnStruct, solution: &PcbSolution, output: &str) -> Result<()> {
    // This function will convert the PcbProblem and PcbSolution into a SES format string
    // The implementation will depend on the specific requirements of the SES format
    let mut ses = File::create(output.to_string() + ".ses")?;
    //let mut ses = String::new();

    let layer_names: Vec<String> = dsn.get_layer_names();

    writeln!(ses, "(session {}.ses)", output)?;
    writeln!(ses, "  (base_design {}.dsn)", output)?;

    generate_placement(&mut ses, &dsn)?;

    writeln!(ses, "  (was_is")?;
    writeln!(ses, "  )")?;
    writeln!(ses, "  (routes")?;
    writeln!(
        ses,
        "    (resolution {} {})",
        dsn.resolution.unit, dsn.resolution.value
    )?;
    writeln!(ses, "    (parser")?;
    writeln!(ses, "      (host_cad \"KiCad's Pcbnew\")")?;
    writeln!(ses, "      (host_version 9.0.2)")?;
    writeln!(ses, "    )")?;

    // via
    writeln!(ses, "    (library_out")?;
    let vias = via_info(&dsn);
    for via in &vias {
        let via_str = via.to_ses_string(&layer_names);
        write!(ses, "{}", via_str)?;
    }
    writeln!(ses, "    )")?;

    // net
    writeln!(ses, "    (network_out")?;

    generate_network(&mut ses, &dsn, &solution, &layer_names, &vias)?;
    writeln!(ses, "  )")?;
    writeln!(ses, ")")?;
    Ok(())
}
