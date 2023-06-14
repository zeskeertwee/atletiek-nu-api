use crate::app::{AppCtx, Window};
use crate::async_resource::AsyncResource;
use atletiek_nu_api::models::athlete_list::AthleteList;
use egui::{Align, Color32, Key, Layout, RichText, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

pub struct AthleteSearchWindow {
    search: AsyncResource<AthleteList>,
    search_field: String,
}

impl Default for AthleteSearchWindow {
    fn default() -> Self {
        Self {
            search: AsyncResource::default(),
            search_field: String::new(),
        }
    }
}

impl Window for AthleteSearchWindow {
    fn title(&self) -> String {
        "Search athletes".to_string()
    }

    fn draw(&mut self, ui: &mut Ui, _: &mut dyn AppCtx) {
        self.search.poll();

        ui.horizontal(|ui| {
            let pressed_enter = ui
                .add(
                    TextEdit::singleline(&mut self.search_field)
                        .interactive(!self.search.is_pending()),
                )
                .lost_focus()
                && ui.input(|i| i.key_pressed(Key::Enter));

            if self.search.is_pending() {
                ui.spinner();
            } else {
                if ui.button("Search").clicked() || pressed_enter {
                    let query = self.search_field.clone();
                    self.search =
                        AsyncResource::new(move || atletiek_nu_api::search_athletes(&query));
                }
            }
        });

        match &self.search {
            AsyncResource::Finished(r) => {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .columns(Column::auto().resizable(true), 4);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("ID");
                        });
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Club");
                        });
                        header.col(|ui| {
                            ui.strong("Age");
                        });
                    })
                    .body(|body| {
                        body.rows(20.0, r.len(), |index, mut row| {
                            let i = &r[index];

                            row.col(|ui| {
                                ui.label(i.id.to_string());
                            });
                            row.col(|ui| {
                                ui.label(&i.name);
                            });
                            row.col(|ui| {
                                ui.label(&i.club_name);
                            });
                            row.col(|ui| {
                                ui.label(format!("{} yr", i.age));
                            });
                        })
                    });
            }
            AsyncResource::Error(e) => {
                ui.label(RichText::new(format!("Error: {}", e)).color(Color32::RED));
            }
            _ => (),
        }
    }
}
