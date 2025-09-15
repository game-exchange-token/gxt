use std::collections::BTreeMap;

use egui_dnd::dnd;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct GxtViewerApp {
    keys: BTreeMap<String, String>,
}

impl GxtViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for GxtViewerApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            });
        });

        // egui::CentralPanel::default().show(ctx, |ui| dnd(ui, "file_drop").show_vec(items, item_ui));
    }
}
