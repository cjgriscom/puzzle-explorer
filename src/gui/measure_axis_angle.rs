use crate::app::PuzzleApp;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MeasureAxisAngleWindowState {
    pub axis_a: String,
    pub axis_b: String,
}

pub fn build_measure_axis_angle_window(app: &mut PuzzleApp, ctx: &egui::Context) {
    let mut open = app.window_state.show_measure_axis_angle;
    egui::Window::new("Measure Axis Angle")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            let available = app.axis_defs.available_axis_names();

            ui.horizontal(|ui| {
                ui.label("Axis A:");
                axis_combo_box(
                    ui,
                    "measure_a",
                    &mut app.measure_axis_angle_state.axis_a,
                    &available,
                );
            });
            ui.horizontal(|ui| {
                ui.label("Axis B:");
                axis_combo_box(
                    ui,
                    "measure_b",
                    &mut app.measure_axis_angle_state.axis_b,
                    &available,
                );
            });

            ui.separator();

            let vec_a = app
                .axis_defs
                .get_resolved_vector(&app.measure_axis_angle_state.axis_a);
            let vec_b = app
                .axis_defs
                .get_resolved_vector(&app.measure_axis_angle_state.axis_b);

            match (vec_a, vec_b) {
                (Some(a), Some(b)) => {
                    let a_n = a.normalize();
                    let b_n = b.normalize();
                    let dot = a_n.dot(b_n).clamp(-1.0, 1.0);
                    let angle_rad = dot.acos();
                    let angle_deg = angle_rad.to_degrees();
                    ui.label(format!("Angle: {:.8}°", angle_deg));
                }
                _ => {
                    let mut parts = Vec::new();
                    if vec_a.is_none() && !app.measure_axis_angle_state.axis_a.is_empty() {
                        parts.push(format!(
                            "'{}' not resolved",
                            app.measure_axis_angle_state.axis_a
                        ));
                    }
                    if vec_b.is_none() && !app.measure_axis_angle_state.axis_b.is_empty() {
                        parts.push(format!(
                            "'{}' not resolved",
                            app.measure_axis_angle_state.axis_b
                        ));
                    }
                    if parts.is_empty() {
                        ui.label("Select two axes to measure");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 80, 80), parts.join(", "));
                    }
                }
            }
        });
    app.window_state.show_measure_axis_angle = open;
}

fn axis_combo_box(ui: &mut egui::Ui, id_salt: &str, selected: &mut String, available: &[String]) {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(if selected.is_empty() {
            "(none)"
        } else {
            selected.as_str()
        })
        .show_ui(ui, |ui| {
            for name in available {
                ui.selectable_value(selected, name.clone(), name);
            }
        });
}
