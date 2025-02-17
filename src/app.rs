use strum::IntoEnumIterator;

#[derive(PartialEq, Clone, Default, strum::Display, strum::EnumIter)]
enum AppTabs {
    #[default]
    #[strum(to_string = "Texture Archives")]
    TextureArchives,
    #[strum(to_string = "Graphical Archives")]
    GraphicalArchives,
    #[strum(to_string = "PackMan Archives")]
    PackManArchives,
    #[strum(to_string = "Text Files")]
    TextFiles,
}

#[derive(Default)]
pub struct EguiApp {
    current_tab: AppTabs,
    picked_file: Option<String>,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.5);
        Self::default()
    }

    fn draw_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for tab in AppTabs::iter() {
                ui.selectable_value(&mut self.current_tab, tab.clone(), tab.to_string());
            }
        });

        ui.separator();
    }

    fn draw_current_tab(&mut self, ui: &mut egui::Ui) {
        if self.current_tab == AppTabs::TextureArchives {
            ui.label("Texture archives");

            if ui.button("Open file...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_file = Some(path.display().to_string());
                }
            }

            if let Some(picked_file) = &self.picked_file {
                ui.label("Picked file:");
                ui.monospace(picked_file);
            }
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_tab_bar(ui);
            self.draw_current_tab(ui);
        });
    }
}
