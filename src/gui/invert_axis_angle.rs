use crate::app::PuzzleApp;
use crate::gui::{AXIS_ANGLE_SPEED, INVERT_AXIS_ANGLE_POS, INVERT_AXIS_ANGLE_WIDTH};
use puzzle_explorer_math::geometry::{derive_axis_angle, invert_axis_angle};

fn format_results(angle_deg: f64, epsilon_decimal_places: f64) -> Vec<String> {
    let epsilon_deg = 10f64.powf(-epsilon_decimal_places);
    let epsilon_rad = epsilon_deg.to_radians();
    let results = invert_axis_angle(angle_deg.to_radians(), epsilon_rad);

    let mut lines = vec![
        format!("Input angle: {:.9}°", angle_deg),
        format!("Epsilon: ±{:.6}°", epsilon_deg),
        format!("Matches: {}", results.len()),
    ];

    if results.is_empty() {
        lines.push(String::from("No matches found within epsilon."));
        return lines;
    }

    lines.push(String::new());
    for (idx, (a, b, p, q, diff_rad)) in results.into_iter().enumerate() {
        let derived_deg = derive_axis_angle(a, b, p, q)
            .map(f64::to_degrees)
            .unwrap_or(angle_deg);
        lines.push(format!(
            "{}. a={}, b={}, p/q={}/{}, derived={:.9}°, error={:.9}°",
            idx + 1,
            a,
            b,
            p,
            q,
            derived_deg,
            diff_rad.to_degrees()
        ));
    }

    lines
}

pub fn build_invert_axis_angle_window(app: &mut PuzzleApp, ctx: &egui::Context) {
    egui::Window::new("Invert Axis Angle")
        .open(&mut app.window_state.show_invert_axis_angle)
        .resizable(true)
        .default_pos(INVERT_AXIS_ANGLE_POS)
        .default_width(INVERT_AXIS_ANGLE_WIDTH)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Angle (deg):");
                ui.add(
                    egui::DragValue::new(&mut app.invert_axis_angle_state.angle_deg_input)
                        .range(0.0..=180.0)
                        .speed(AXIS_ANGLE_SPEED)
                        .fixed_decimals(9)
                        .suffix("°"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Allowable error:");
                ui.spacing_mut().slider_width = 200.0;
                if ui
                    .add(
                        egui::Slider::new(
                            &mut app.invert_axis_angle_state.epsilon_decimal_places,
                            0.0..=6.0,
                        )
                        .show_value(false),
                    )
                    .changed()
                {
                    // Snap epsilon to one significant digit, e.g. 0.94 -> 0.9, 0.0034 -> 0.003.
                    let epsilon = 10f64.powf(-app.invert_axis_angle_state.epsilon_decimal_places);
                    let exponent = epsilon.log10().floor();
                    let scale = 10f64.powf(exponent);
                    let snapped = (epsilon / scale).floor() * scale;
                    app.invert_axis_angle_state.epsilon_decimal_places = -snapped.log10();
                }
                ui.label(format!(
                    "±{:.6}°",
                    10f64.powf(-app.invert_axis_angle_state.epsilon_decimal_places)
                ));
            });

            let mut compute_clicked = false;
            ui.horizontal(|ui| {
                if ui.button("Compute").clicked() {
                    compute_clicked = true;
                }
            });

            if compute_clicked {
                app.invert_axis_angle_state.output_lines = format_results(
                    app.invert_axis_angle_state.angle_deg_input,
                    app.invert_axis_angle_state.epsilon_decimal_places,
                );
                app.invert_axis_angle_state.error_message = None;
            }

            if let Some(error_message) = &app.invert_axis_angle_state.error_message {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(255, 80, 80), error_message);
            }

            ui.separator();
            egui::ScrollArea::vertical().vscroll(true).show(ui, |ui| {
                if app.invert_axis_angle_state.output_lines.is_empty()
                    && app.invert_axis_angle_state.error_message.is_none()
                {
                    ui.label("Enter an angle and press Compute.");
                    return;
                }

                for line in &app.invert_axis_angle_state.output_lines {
                    if line.is_empty() {
                        ui.add_space(4.0);
                    } else {
                        ui.label(line);
                    }
                }
            });
        });
}
