use crate::riders::texture_archive::TextureArchive;
use egui::ScrollArea;
use egui_modal::{Icon, Modal};
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
    picked_tex_archive: Option<TextureArchive>,
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

    fn draw_tex_archive_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut modal = Modal::new(ctx, "test");
        modal.show_dialog();
        ui.label("Texture archives");

        if ui.button("Open file...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_file = Some(path.display().to_string());
                self.picked_tex_archive = Some(
                    TextureArchive::new(self.picked_file.clone().unwrap())
                        .expect("File could not be opened."),
                );

                if let Err(err_str) = &self.picked_tex_archive.as_mut().unwrap().read() {
                    modal
                        .dialog()
                        .with_title("Error")
                        .with_body(err_str)
                        .with_icon(Icon::Error)
                        .open();
                }
            }
        }

        if let Some(picked_file) = &self.picked_file {
            ui.label("Picked file:");
            ui.monospace(picked_file.to_string());
        }

        if ui
            .add_enabled(
                self.picked_tex_archive.is_some(),
                egui::Button::new("Tex Arc Button"),
            )
            .clicked()
        {
            println!("Button clicked!");
        }

        if ui.button("Test").clicked() {
            modal
                .dialog()
                .with_title("Test dialog")
                .with_body("Body test")
                .with_icon(Icon::Info)
                .open();
        }

        if let Some(_tex_archive) = &self.picked_tex_archive {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("Test 1");
                ui.label("Test 2");
                ui.label("Test 3");
                for i in 0..10 {
                    ui.label("More test");
                }
            });
        }
    }

    fn draw_current_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.current_tab == AppTabs::TextureArchives {
            self.draw_tex_archive_tab(ctx, ui);
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_tab_bar(ui);
            self.draw_current_tab(ctx, ui);
        });
    }
}
