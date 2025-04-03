use std::io::Cursor;

use crate::riders::{
    gvr_texture::GVRTexture,
    packman_archive::{PackManArchive, PackManFolder},
    texture_archive::TextureArchive,
};
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
struct GraphicalArchiveContext {
    picked_file: Option<String>,
}

#[derive(Default)]
struct TextureArchiveContext {
    picked_file: Option<String>,
    archive: Option<TextureArchive>,
}

#[derive(Default)]
struct PackManArchiveContext {
    picked_file: Option<String>,
    archive: Option<PackManArchive>,
}

#[derive(Default)]
pub struct EguiApp {
    current_tab: AppTabs,

    texture_archive_ctx: TextureArchiveContext,
    graphical_archive_ctx: GraphicalArchiveContext,
    packman_archive_ctx: PackManArchiveContext,
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

    fn draw_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tab-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for tab in AppTabs::iter() {
                    ui.selectable_value(&mut self.current_tab, tab.clone(), tab.to_string());
                }
            });
            ui.add_space(1.);
        });
    }

    fn draw_side_bars(&mut self, ctx: &egui::Context) {
        if self.current_tab == AppTabs::GraphicalArchives {
            egui::SidePanel::left("graphical-left-sidebar").show(ctx, |ui| {
                ui.small("No objects.");
            });
        }
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

        ui.horizontal(|ui| {
            if ui
                .button("Open file...")
                .on_hover_ui(|ui| {
                    ui.label("Opens a pre-existing GVR texture archive.");
                })
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.texture_archive_ctx.picked_file = Some(path.display().to_string());

                    let tex_archive = TextureArchive::new(self.texture_archive_ctx.picked_file.clone().unwrap());
                    if tex_archive.is_err() {
                        modal
                            .dialog()
                            .with_title("Error")
                            .with_body("File could not be opened.")
                            .with_icon(Icon::Error)
                            .open();
                    } else {
                        self.texture_archive_ctx.archive = Some(tex_archive.unwrap());
                    }

                    if let Err(err_str) = &self.texture_archive_ctx.archive.as_mut().unwrap().read() {
                        modal
                            .dialog()
                            .with_title("Error")
                            .with_body(err_str)
                            .with_icon(Icon::Error)
                            .open();
                    }
                }
            }

            if ui.button("Create new...").on_hover_ui(|ui| {
                ui.label("Makes a new empty texture archive, where you can start adding textures into.");
            }).clicked() {
                self.texture_archive_ctx.archive = Some(TextureArchive::new_empty());
            }

            let is_archive_exportable = self.texture_archive_ctx.archive.is_some()
                && !self
                    .texture_archive_ctx.archive
                    .as_ref()
                    .unwrap()
                    .textures
                    .is_empty();

            if ui
                .add_enabled(
                    is_archive_exportable,
                    egui::Button::new("Export archive..."),
                ).on_hover_ui(|ui| {
                    ui.label("Exports all the given textures in the list as a GVR texture archive.");
                })
                .clicked()
            {
                if let Some(rfd_path) = rfd::FileDialog::new().save_file() {
                    if self
                        .texture_archive_ctx.archive
                        .as_ref()
                        .unwrap()
                        .export(&rfd_path.display().to_string())
                        .is_ok()
                    {
                        modal
                            .dialog()
                            .with_title("Success")
                            .with_body("Texture archive exported successfully!")
                            .with_icon(Icon::Success)
                            .open();
                    } else {
                        modal
                            .dialog()
                            .with_title("Error")
                            .with_body("Texture archive export failed.")
                            .with_icon(Icon::Error)
                            .open();
                    }
                }
            }
        });

        if let Some(picked_file) = &self.texture_archive_ctx.picked_file {
            ui.label("Picked file:");
            ui.monospace(picked_file.to_string());
        }

        if let Some(tex_archive) = &mut self.texture_archive_ctx.archive {
            ui.separator();

            ui.checkbox(&mut tex_archive.is_without_model, "Is without a model")
                .on_hover_ui(|ui| {
                    ui.label(
                        "Whether or not this texture archive is associated with a 3D model or not.",
                    );
                });

            ui.horizontal(|ui| {
                ui.heading("Texture list:");

                if ui
                    .button("Add")
                    .on_hover_ui(|ui| {
                        ui.label("Adds a new GVR texture(s) to the end of the texture list.");
                    })
                    .clicked()
                {
                    if let Some(files) = rfd::FileDialog::new().pick_files() {
                        let mut broken_file: Option<String> = None;

                        for file in files {
                            let path = file.display().to_string();
                            let mut cursor = Cursor::new(std::fs::read(&path).unwrap());
                            let texture = GVRTexture::new_from_cursor(
                                file.file_stem()
                                    .unwrap()
                                    .to_os_string()
                                    .into_string()
                                    .unwrap(),
                                &mut cursor,
                            );

                            if let Ok(valid_tex) = texture {
                                tex_archive.textures.push(valid_tex);
                            } else {
                                broken_file = Some(
                                    file.file_name()
                                        .unwrap()
                                        .to_os_string()
                                        .into_string()
                                        .unwrap(),
                                );
                                break;
                            }
                        }

                        if let Some(file) = broken_file {
                            modal
                                .dialog()
                                .with_title("Error")
                                .with_body(format!("File {} is not a valid GVR texture.", file))
                                .with_icon(Icon::Error)
                                .open();
                        } else {
                            modal
                                .dialog()
                                .with_title("Success")
                                .with_body("Texture(s) added succesfully!")
                                .with_icon(Icon::Success)
                                .open();
                        }
                    }
                }
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

                            ui.spacing_mut().button_padding = [1., 0.].into();
                            ui.scope(|ui| {
                                ui.style_mut().spacing.item_spacing = [10., 0.].into();
                                //ui.spacing_mut().button_padding.y = 2.;
                                ui.vertical(|ui| {
                                    ui.add_enabled_ui(i > 1, |ui| {
                                        let button =
                                            ui.add_sized([1., 1.], egui::Button::new("⏶").small());
                                        if button.clicked() {
                                            moved_up_index = Some(i - 1);
                                        }
                                    });
                                    if ui
                                        .add_enabled(
                                            i - 1 < textures_count - 1,
                                            egui::Button::new("⏷").small(),
                                        )
                                        .clicked()
                                    {
                                        moved_down_index = Some(i - 1);
                                    }
                                });
                            });

                            ui.scope(|ui| {
                                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                    Color32::DARK_RED;
                                if ui
                                    .button("Remove")
                                    .on_hover_ui(|ui| {
                                        ui.label("Removes this texture from the list.");
                                    })
                                    .clicked()
                                {
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

    fn draw_graphical_archive_tab(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.graphical_archive_ctx.picked_file = Some(path.display().to_string());
            }
        }

        if let Some(picked_file) = &self.graphical_archive_ctx.picked_file {
            ui.label("Picked file:");
            ui.monospace(picked_file);
        }
    }

    fn draw_packman_archive_tab(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open file...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.packman_archive_ctx.picked_file = Some(path.display().to_string());
                    if let Ok(mut archive) =
                        PackManArchive::new(self.packman_archive_ctx.picked_file.as_ref().unwrap())
                    {
                        archive.read().unwrap();
                        self.packman_archive_ctx.archive = Some(archive);

                        // Clear data so collapsing header state doesn't persist
                        ui.data_mut(|data| {
                            data.clear();
                        });
                    }
                }
            }

            if ui.button("Create new...").clicked() {
                self.packman_archive_ctx.archive = Some(PackManArchive::new_empty());
            }

            let mut export_enabled = false;
            if let Some(archive) = &self.packman_archive_ctx.archive {
                export_enabled =
                    !archive.folders.is_empty() && archive.folders.iter().all(|f| f.is_id_valid);
            }
            if ui
                .add_enabled(export_enabled, egui::Button::new("Export archive..."))
                .clicked()
            {
                todo!("Export a PackMan archive");
            }
        });

        if let Some(archive) = &mut self.packman_archive_ctx.archive {
            ui.separator();
            ui.label(format!("Folder count: {}", archive.folders.len()));

            if ui.button("Add folder").clicked() {
                archive.folders.push(PackManFolder::new(0));
            }

            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_size(ui.max_rect().size());

                let mut removed_folder_idx: Option<usize> = None;

                for (i, folder) in archive.folders.iter_mut().enumerate() {
                    ui.collapsing(format!("Folder {i}"), |ui| {
                        ui.label(format!("ID: (valid: {})", folder.is_id_valid));

                        // Handle editing of the ID properly with validation checks
                        ui.scope(|ui| {
                            let folder_hash = egui::Id::new(format!("packman-id-textedit{}", i));

                            if !folder.is_id_valid {
                                // Text edit background color
                                ui.visuals_mut().extreme_bg_color = Color32::from_rgb(30, 8, 5);

                                ui.visuals_mut().widgets.hovered.bg_stroke.color =
                                    Color32::DARK_RED;

                                let mut empty = String::new();

                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut empty).id(folder_hash));
                                    ui.visuals_mut().override_text_color = Some(Color32::RED);
                                    ui.label("Please specify an ID number.");
                                });

                                if let Ok(result) = empty.parse() {
                                    folder.is_id_valid = true;
                                    folder.id = result;
                                }
                            } else {
                                let mut tmp_value = format!("{}", &folder.id);
                                ui.add(egui::TextEdit::singleline(&mut tmp_value).id(folder_hash));

                                if let Ok(result) = tmp_value.parse() {
                                    folder.is_id_valid = true;
                                    folder.id = result;
                                } else if tmp_value.is_empty() {
                                    folder.is_id_valid = false;
                                    folder.id = 0;
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Add files...").clicked() {
                                if let Some(files) = rfd::FileDialog::new().pick_files() {
                                    for file in files {
                                        folder.files.push(
                                            std::fs::read(file.display().to_string()).unwrap(),
                                        );
                                    }
                                }
                            }
                            if ui.button("Add empty file...").clicked() {
                                folder.files.push(Vec::new());
                            }
                            if ui.button("Remove folder").clicked() {
                                removed_folder_idx = Some(i);
                            }
                        });
                        ui.separator();

                        let mut deleted_idx: Option<usize> = None;
                        for (i, file) in folder.files.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("File {i}:"));
                                ui.label(format!("Size: {:#x}", file.len()));
                            });
                            ui.horizontal(|ui| {
                                if ui.button("Replace").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        *file = std::fs::read(path.display().to_string()).unwrap();
                                    }
                                }
                                if ui.button("Clear").clicked() {
                                    file.clear();
                                }
                                if ui.button("Remove").clicked() {
                                    deleted_idx = Some(i);
                                }
                            });
                            ui.add_space(8.0);
                        }

                        if let Some(idx) = deleted_idx {
                            folder.files.remove(idx);
                        }
                    });
                }

                if let Some(idx) = removed_folder_idx {
                    archive.folders.remove(idx);
                }
            });
        }
    }

    fn draw_current_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        match self.current_tab {
            AppTabs::Home => self.draw_home_tab(ctx, ui),
            AppTabs::TextureArchives => self.draw_tex_archive_tab(ctx, ui),
            AppTabs::GraphicalArchives => self.draw_graphical_archive_tab(ctx, ui),
            AppTabs::PackManArchives => self.draw_packman_archive_tab(ctx, ui),
            _ => {}
        }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_tab_bar(ctx);
        self.draw_side_bars(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_current_tab(ctx, ui);
        });
    }
}
