use egui::{warn_if_debug_build, Align, Margin, RichText, Ui};

use super::{App, ForwardPort, Pages};
use crate::widgets::Toggle;

impl App {
    pub(super) fn list_page(&mut self, ctx: &egui::Context) {
        self.top_band(ctx);
        self.center_panel(ctx);
    }
    fn top_band(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::Frame::default().inner_margin(10.0).show(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        egui::widgets::global_dark_light_mode_switch(ui);
                        if ui.button("Reset").clicked() {
                            *self = App::default();
                        }
                    });
                })
            });
        });
    }

    fn center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            error_warning(ui, &self.error);

            egui::Frame::default().inner_margin(10.0).show(ui, |ui| {
                ui.heading("From");

                ui.horizontal(|ui| {
                    ui.label("Port: ");
                    ui.add_enabled(
                        !self.is_enabled,
                        egui::DragValue::new(&mut self.listen_port).range(0..=65535),
                    );

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(Toggle::new(&mut self.is_enabled)).clicked() {
                            self.update_hyper();
                        };
                    });
                });
                ui.add_space(10.0);

                ui.separator();

                egui::Frame::none()
                    .outer_margin(Margin {
                        top: 10.0,
                        bottom: 10.0,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.heading("To");
                    });

                for (index, forward_port) in self.forward_ports.clone().iter().enumerate() {
                    let mut is_active = if let Some(cur_port) = &self.active_forward_port {
                        cur_port == forward_port
                    } else {
                        false
                    };
                    ui.horizontal(|ui| {
                        ui.label(&forward_port.name);
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add_enabled(!is_active, egui::Button::new("Edit"))
                                .clicked()
                            {
                                self.active_page = Pages::Edit(index, forward_port.clone());
                            };
                            if ui.add_enabled(!is_active, egui::Button::new("x")).clicked() {
                                self.forward_ports.retain(|port| port != forward_port);
                            };
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Port: ");
                        ui.label(format!("{}", forward_port.port));

                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            let is_already_active = is_active;
                            if ui.add(Toggle::new(&mut is_active)).clicked() {
                                if is_already_active {
                                    self.active_forward_port = None;
                                } else {
                                    self.active_forward_port = Some(forward_port.clone());
                                }
                                self.update_hyper();
                            };
                        });
                    });
                    ui.add_space(10.0);
                }

                ui.separator();

                if ui.button("Add New").clicked() {
                    self.active_page = Pages::Creation(ForwardPort::default());
                }

                warn_if_debug_build(ui)
            });
        });
    }
}

fn error_warning(ui: &mut Ui, error: &Option<String>) {
    if let Some(err_str) = error {
        egui::Frame::default().inner_margin(10.0).show(ui, |ui| {
            ui.label(
                RichText::new(format!("⚠ {} ⚠", err_str))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        });
    }
}
