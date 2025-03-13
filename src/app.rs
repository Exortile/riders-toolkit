use crate::riders::texture_archive::TextureArchive;
use egui::Color32;
use egui_modal::{Icon, Modal};
use strum::IntoEnumIterator;

#[derive(PartialEq, Clone, Default, strum::Display, strum::EnumIter)]
enum AppTabs {
    #[default]
    Home,
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
        // Set UI zoom
        cc.egui_ctx.set_pixels_per_point(1.5);

        // Set up general style used everywhere
        cc.egui_ctx.style_mut(|style| {
            style.spacing.scroll.floating = false;
            style.spacing.item_spacing = [10.0, 10.0].into();
        });

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

    fn draw_home_tab(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Riders Toolkit");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")))
        });
    }

    fn draw_tex_archive_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut modal = Modal::new(ctx, "generic-texarc-dialog");
        modal.show_dialog();

        if ui.button("Open file...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.picked_file = Some(path.display().to_string());

                let tex_archive = TextureArchive::new(self.picked_file.clone().unwrap());
                if tex_archive.is_err() {
                    modal
                        .dialog()
                        .with_title("Error")
                        .with_body("File could not be opened.")
                        .with_icon(Icon::Error)
                        .open();
                } else {
                    self.picked_tex_archive = Some(tex_archive.unwrap());
                }

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

        if ui.button("Test Dialog").clicked() {
            modal
                .dialog()
                .with_title("Test dialog")
                .with_body("Body test")
                .with_icon(Icon::Info)
                .open();
        }

        if let Some(tex_archive) = &mut self.picked_tex_archive {
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Texture list:");

                // TODO: implement adding textures via file picker
                let _ = ui.button("Add").on_hover_ui(|ui| {
                    ui.label("Adds a new GVR texture to the end of the texture list.");
                });
            });

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    let mut removed_index: Option<usize> = None;
                    let mut moved_up_index: Option<usize> = None;
                    let mut moved_down_index: Option<usize> = None;

                    let mut i = 1; // have to do it without enumerate()
                    let textures_count = tex_archive.textures.len();
                    for tex in &mut tex_archive.textures {
                        ui.horizontal(|ui| {
                            ui.scope(|ui| {
                                ui.style_mut().interaction.selectable_labels = false;
                                ui.add_sized([40.0, 20.0], egui::Label::new(format!("{i}.")));
                            });

                            let _ = ui.add(
                                egui::TextEdit::singleline(&mut tex.name).hint_text("Texture name"),
                            );

                            if ui.add_enabled(i > 1, egui::Button::new("⏶")).clicked() {
                                moved_up_index = Some(i - 1);
                            }
                            if ui
                                .add_enabled(i - 1 < textures_count - 1, egui::Button::new("⏷"))
                                .clicked()
                            {
                                moved_down_index = Some(i - 1);
                            }

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                    Color32::DARK_RED;
                                if ui.button("Remove").clicked() {
                                    removed_index = Some(i - 1);
                                }
                            });
                        });

                        i += 1;
                    }

                    if let Some(idx) = removed_index {
                        tex_archive.textures.remove(idx);
                    }
                    if let Some(idx) = moved_up_index {
                        tex_archive.textures.swap(idx, idx - 1);
                    }
                    if let Some(idx) = moved_down_index {
                        tex_archive.textures.swap(idx, idx + 1);
                    }
                });
        }
    }

    fn draw_current_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        match self.current_tab {
            AppTabs::TextureArchives => self.draw_tex_archive_tab(ctx, ui),
            AppTabs::Home => self.draw_home_tab(ctx, ui),
            _ => {}
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
