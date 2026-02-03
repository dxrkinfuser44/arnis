use super::operator::operator_vec_from_json;
use crate::coordinate_system::cartesian::XZBBox;
use crate::debug;
use crate::ground::Ground;
use crate::info;
use crate::osm_parser::ProcessedElement;
use crate::progress::emit_gui_progress_update;

pub fn transform_map(
    elements: &mut Vec<ProcessedElement>,
    xzbbox: &mut XZBBox,
    ground: &mut Ground,
) {
    info!("[4/7] Transforming map");
    emit_gui_progress_update(20.0, "Transforming map...");

    let opjson_string = include_str!("../../tests/map_transformation/example_transformations.json");
    let opjson = serde_json::from_str(opjson_string)
        .expect("Failed to parse map transformations config json");

    let ops = operator_vec_from_json(&opjson)
        .map_err(|e| format!("Map transformations json format error:\n{e}"))
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            panic!();
        });

    let nop: usize = ops.len();
    let mut iop: usize = 1;

    let progress_increment_prcs: f64 = 5.0 / nop as f64;

    for op in ops {
        let current_progress_prcs = 20.0 + (iop as f64 * progress_increment_prcs);
        emit_gui_progress_update(current_progress_prcs, "");

        iop += 1;

        let op_name = op.repr();
        debug!("Applying transformation {}/{}: {}", iop - 1, nop, op_name);
        op.operate(elements, xzbbox, ground);
    }

    emit_gui_progress_update(25.0, "");
}
