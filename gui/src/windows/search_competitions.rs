use crate::app::{AppCtx, AppEvent, Window};
use crate::async_resource::AsyncResource;
use crate::windows::registrations::RegistrationsWindow;
use atletiek_nu_api::chrono::{self, Months, NaiveDate};
use atletiek_nu_api::models::competitions_list_web::CompetitionsWebList;
use eframe::emath::Align;
use egui::{Color32, Key, Layout, RichText, TextEdit, Ui};
use egui_datepicker::DatePicker;
use egui_extras::{Column, TableBuilder};

pub struct SearchCompetitionsWindow {
    search: AsyncResource<CompetitionsWebList>,
    search_field: String,
    start: NaiveDate,
    end: NaiveDate,
}

impl Default for SearchCompetitionsWindow {
    fn default() -> Self {
        let now = chrono::offset::Local::now().date_naive();

        Self {
            search: AsyncResource::default(),
            search_field: String::new(),
            start: now,
            end: now.checked_add_months(Months::new(1)).unwrap(),
        }
    }
}

impl Window for SearchCompetitionsWindow {
    fn title(&self) -> String {
        "Search competitions".to_string()
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &mut dyn AppCtx) {
        self.search.poll();

        ui.horizontal(|ui| {
            let pressed_enter = ui
                .add(
                    TextEdit::singleline(&mut self.search_field)
                        .interactive(!self.search.is_pending()),
                )
                .lost_focus()
                && ui.input(|i| i.key_pressed(Key::Enter));

            ui.label("Start");
            ui.add(DatePicker::new("start-picker", &mut self.start));
            ui.add_space(10.0);
            ui.label("End");
            ui.add(DatePicker::new("end-picker", &mut self.end));

            if self.search.is_pending() {
                ui.spinner();
            } else {
                if ui.button("Search").clicked() || pressed_enter {
                    let query = self.search_field.clone();
                    let start = self.start.clone();
                    let end = self.end.clone();
                    self.search = AsyncResource::new(move || {
                        atletiek_nu_api::search_competitions_for_time_period(start, end, &query)
                    });
                }
            }
        });

        match &self.search {
            AsyncResource::Finished(r) => {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .columns(Column::auto().resizable(true), 5);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("ID");
                        });
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Location");
                        });
                        header.col(|ui| {
                            ui.strong("Registrations");
                        });
                        header.col(|ui| {
                            ui.strong("");
                        });
                    })
                    .body(|body| {
                        body.rows(20.0, r.len(), |index, mut row| {
                            let i = &r[index];

                            row.col(|ui| {
                                ui.label(i.competition_id.to_string());
                            });
                            row.col(|ui| {
                                ui.label(&i.name);
                            });
                            row.col(|ui| {
                                ui.strong(&i.location);
                            });
                            row.col(|ui| {
                                ui.strong(format!("{} registrations", i.registrations));
                            });
                            row.col(|ui| {
                                if i.registrations == 0 {
                                    ui.label("No registrations!");
                                } else {
                                    if ui.button("List registrations").clicked() {
                                        ctx.push_event(AppEvent::SpawnWindow(Box::new(
                                            RegistrationsWindow::new(
                                                i.name.to_owned(),
                                                i.competition_id,
                                                i.results_availible,
                                            ),
                                        )));
                                    }
                                }
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
