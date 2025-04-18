use super::{App, Pages};
use egui::vec2;

impl App {
    pub(super) fn creation_page(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("creation_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::Frame::none().outer_margin(10.0).show(ui, |ui| {
                    if ui.button("Back").clicked() {
                        self.active_page = Pages::List;
                    }
                })
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::default().inner_margin(10.0).show(ui, |ui| {
                egui::Grid::new("creation_form")
                    .min_col_width(100.0)
                    .num_columns(2)
                    .spacing(vec2(0.0, 10.0))
                    .show(ui, |ui| {
                        let mut replace_on_save: Option<usize> = None;

                        let editing_port = match &mut self.active_page {
                            Pages::List => return,
                            Pages::Creation(new_for) => new_for,
                            Pages::Edit(pos, edit_for) => {
                                replace_on_save = Some(*pos);
                                edit_for
                            }
                        };

                        ui.label("Name: ");
                        ui.text_edit_singleline(&mut editing_port.name);
                        ui.end_row();
                        ui.label("Domain: ");
                        ui.text_edit_singleline(&mut editing_port.target.domain);
                        ui.end_row();
                        ui.label("Port: ");
                        ui.add(
                            egui::DragValue::new(&mut editing_port.target.port).range(0..=65535),
                        );
                        ui.end_row();

                        if let Some(err_msg) = &editing_port.error {
                            ui.label(err_msg);
                            ui.end_row();
                        }

                        if ui.button("Save").clicked() {
                            let mut new_port = editing_port.clone();
                            new_port.error = None;
                            let new_port = new_port;
                            let pos = self.forward_ports.iter().position(|item| *item == new_port);

                            match replace_on_save {
                                Some(replace_index) => {
                                    if let Some(existing_pos) = pos {
                                        if existing_pos != replace_index {
                                            editing_port.error =
                                                Some("Port Already exist".to_string());
                                            return;
                                        }
                                    }

                                    self.forward_ports[replace_index] = new_port;
                                }
                                None => {
                                    let already_exist = pos.is_some();
                                    if !already_exist {
                                        self.forward_ports.push(new_port);
                                    } else {
                                        editing_port.error = Some("Port Already exist".to_string());
                                        return;
                                    };
                                }
                            }
                            self.active_page = Pages::List;
                        }
                        ui.end_row();
                    });
            });
        });
    }
}
