use crate::app::{
    AXIS_ANGLE_DECIMALS, AXIS_ANGLE_SPEED, COLAT_DECIMALS, COLAT_SPEED, COLAT_STEP, MAX_COLAT,
    MAX_N, MIN_COLAT, MIN_N, PuzzleApp,
};
use puzzle_explorer_math::geometry::derive_axis_angle;

pub fn build_puzzle_params_window(app: &mut PuzzleApp, ctx: &egui::Context) {
    let buttons_enabled = app.anim.is_none();

    egui::Window::new("Puzzle Parameters")
        .default_pos([50.0, 50.0])
        .show(ctx, |ui| {
            // Bigger slider than default
            ui.spacing_mut().slider_width = 250.0;

            let mut changed = false;

            ui.horizontal(|ui| {
                if ui
                    .add(crate::gui::toggle(&mut app.params.show_axes))
                    .changed()
                    && let Some(three) = &app.three
                {
                    let axes = app.build_axes();
                    three.update_axis_indicators(&axes, app.params.show_axes);
                }
                ui.label("Show axes");
            });

            ui.horizontal(|ui| {
                ui.label("nA:");
                egui::ComboBox::from_id_salt("nA")
                    .selected_text(format!("{}", app.params.n_a))
                    .show_ui(ui, |ui| {
                        for i in MIN_N..=MAX_N {
                            if ui
                                .selectable_value(&mut app.params.n_a, i, format!("{}", i))
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
                ui.label("nB:");
                egui::ComboBox::from_id_salt("nB")
                    .selected_text(format!("{}", app.params.n_b))
                    .show_ui(ui, |ui| {
                        for i in MIN_N..=MAX_N {
                            if ui
                                .selectable_value(&mut app.params.n_b, i, format!("{}", i))
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Manual Axis Angle");
                if ui
                    .add(crate::gui::toggle(&mut app.params.manual_axis_angle))
                    .changed()
                {
                    // Sync: when switching to manual, populate from current p/q
                    if app.params.manual_axis_angle
                        && let Some(ang) = derive_axis_angle(
                            app.params.n_a,
                            app.params.n_b,
                            app.params.p,
                            app.params.q,
                        )
                    {
                        app.params.manual_axis_angle_deg =
                            (ang.to_degrees() * 10000.0).round() / 10000.0;
                    }

                    changed = true;
                }
            });

            if app.params.manual_axis_angle {
                ui.horizontal(|ui| {
                    ui.label("Axis Angle:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.manual_axis_angle_deg)
                                .range(0.0..=180.0)
                                .speed(AXIS_ANGLE_SPEED)
                                .fixed_decimals(AXIS_ANGLE_DECIMALS)
                                .suffix("°"),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    ui.separator();
                    ui.label("Max Iterations:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.manual_max_iterations)
                                .range(1..=150)
                                .speed(0.1),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("p/q:");

                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.p)
                                .range(1..=20)
                                .speed(0.02),
                        )
                        .changed()
                    {
                        changed = true;
                    }

                    ui.label("/");

                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.q)
                                .range(2..=30)
                                .speed(0.02),
                        )
                        .changed()
                    {
                        changed = true;
                    }

                    if let Some(ang) = derive_axis_angle(
                        app.params.n_a,
                        app.params.n_b,
                        app.params.p,
                        app.params.q,
                    ) {
                        ui.label(format!("Cut: {:.4}\u{00B0}", ang.to_degrees()));
                    }
                });
            }

            ui.separator();

            if ui
                .checkbox(&mut app.params.lock_cuts, "Lock cuts together")
                .changed()
                && app.params.lock_cuts
            {
                app.params.colat_b = app.params.colat_a;
                for ea in &mut app.params.extra_axes {
                    ea.colat = app.params.colat_a;
                }
                changed = true;
            }

            ui.label(format!("Cut A: {:.1}\u{00B0}", app.params.colat_a));
            if ui
                .add(
                    egui::Slider::new(&mut app.params.colat_a, MIN_COLAT..=MAX_COLAT)
                        .smallest_positive(COLAT_STEP)
                        .fixed_decimals(COLAT_DECIMALS)
                        .step_by(COLAT_STEP)
                        .drag_value_speed(COLAT_SPEED)
                        .show_value(true)
                        .trailing_fill(true),
                )
                .changed()
            {
                if app.params.lock_cuts {
                    app.params.colat_b = app.params.colat_a;
                    for ea in &mut app.params.extra_axes {
                        ea.colat = app.params.colat_a;
                    }
                }
                changed = true;
            }

            ui.label(format!("Cut B: {:.1}\u{00B0}", app.params.colat_b));
            ui.add_enabled_ui(!app.params.lock_cuts, |ui| {
                if ui
                    .add(
                        egui::Slider::new(&mut app.params.colat_b, MIN_COLAT..=MAX_COLAT)
                            .smallest_positive(COLAT_STEP)
                            .fixed_decimals(COLAT_DECIMALS)
                            .step_by(COLAT_STEP)
                            .drag_value_speed(COLAT_SPEED)
                            .show_value(true)
                            .trailing_fill(true),
                    )
                    .changed()
                {
                    if app.params.lock_cuts {
                        app.params.colat_a = app.params.colat_b;
                        for ea in &mut app.params.extra_axes {
                            ea.colat = app.params.colat_b;
                        }
                    }
                    changed = true;
                }
            });

            if changed {
                app.spawn_geometry_worker();
            }

            ui.separator();

            // --- Additional Axes (Experimental) ---
            let mut extra_changed = false;
            ui.horizontal(|ui| {
                ui.label("Additional axes:");
                if ui
                    .add(
                        egui::DragValue::new(&mut app.params.num_extra_axes)
                            .range(0..=5)
                            .speed(0.05),
                    )
                    .changed()
                {
                    let n = app.params.num_extra_axes as usize;
                    while app.params.extra_axes.len() < n {
                        let mut new_axis = crate::gui::ExtraAxisParams::default();
                        if app.params.lock_cuts {
                            new_axis.colat = app.params.colat_a;
                        }
                        app.params.extra_axes.push(new_axis);
                    }
                    app.params.extra_axes.truncate(n);
                    extra_changed = true;
                }
                ui.label("(experimental)");
            });

            let axis_labels = ['C', 'D', 'E', 'F', 'G'];
            for idx in 0..app.params.extra_axes.len() {
                ui.separator();

                let label = axis_labels.get(idx).copied().unwrap_or('?');
                ui.horizontal(|ui| {
                    ui.label(format!("n{}:", label));
                    egui::ComboBox::from_id_salt(format!("n{}", label))
                        .selected_text(format!("{}", app.params.extra_axes[idx].n))
                        .show_ui(ui, |ui| {
                            for i in MIN_N..=MAX_N {
                                if ui
                                    .selectable_value(
                                        &mut app.params.extra_axes[idx].n,
                                        i,
                                        format!("{}", i),
                                    )
                                    .changed()
                                {
                                    extra_changed = true;
                                }
                            }
                        });
                    ui.label("Pitch:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.extra_axes[idx].pitch_deg)
                                .range(0.0..=180.0)
                                .speed(AXIS_ANGLE_SPEED)
                                .fixed_decimals(AXIS_ANGLE_DECIMALS)
                                .suffix("°"),
                        )
                        .changed()
                    {
                        extra_changed = true;
                    }
                    ui.label("Yaw:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut app.params.extra_axes[idx].yaw_deg)
                                .range(-180.0..=180.0)
                                .speed(AXIS_ANGLE_SPEED)
                                .fixed_decimals(AXIS_ANGLE_DECIMALS)
                                .suffix("°"),
                        )
                        .changed()
                    {
                        extra_changed = true;
                    }
                });

                ui.label(format!(
                    "Cut {}: {:.1}\u{00B0}",
                    label, app.params.extra_axes[idx].colat
                ));
                ui.add_enabled_ui(!app.params.lock_cuts, |ui| {
                    if ui
                        .add(
                            egui::Slider::new(
                                &mut app.params.extra_axes[idx].colat,
                                MIN_COLAT..=MAX_COLAT,
                            )
                            .smallest_positive(COLAT_STEP)
                            .fixed_decimals(COLAT_DECIMALS)
                            .step_by(COLAT_STEP)
                            .drag_value_speed(COLAT_SPEED)
                            .show_value(true)
                            .trailing_fill(true),
                        )
                        .changed()
                    {
                        extra_changed = true;
                    }
                });
            }

            if extra_changed {
                app.spawn_geometry_worker();
                if let Some(three) = &app.three {
                    let axes = app.build_axes();
                    three.update_axis_indicators(&axes, app.params.show_axes);
                }
            }

            ui.separator();

            ui.add_enabled_ui(buttons_enabled, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Rotate A").clicked() {
                        app.start_rotation(0, true);
                    }
                    if ui.button("A'").clicked() {
                        app.start_rotation(0, false);
                    }
                    if ui.button("Rotate B").clicked() {
                        app.start_rotation(1, true);
                    }
                    if ui.button("B'").clicked() {
                        app.start_rotation(1, false);
                    }
                });
            });
        });
}
