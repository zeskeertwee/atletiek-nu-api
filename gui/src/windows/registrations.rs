use crate::app::{AppCtx, AppEvent, Window};
use crate::async_resource::AsyncResource;
use crate::windows::event_results::EventResultsWindow;
use atletiek_nu_api::models::registrations_list::{RegistrationsList, RegistrationsListElement};
use eframe::emath::Align;
use egui::{Color32, Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};

pub struct RegistrationsWindow {
    registrations: AsyncResource<RegistrationsList>,
    is_team: bool,
    competition_name: String,
    competition_id: u32,
    search_text: String,
    results_availible: bool,
}

impl RegistrationsWindow {
    pub fn new(name: String, id: u32, results_availible: bool) -> Self {
        Self {
            registrations: AsyncResource::default(),
            is_team: false,
            competition_name: name,
            competition_id: id,
            search_text: String::new(),
            results_availible,
        }
    }
}

impl Window for RegistrationsWindow {
    fn title(&self) -> String {
        format!(
            "Registrations for {} [{}]",
            self.competition_name, self.competition_id
        )
    }

    fn on_spawn(&mut self) {
        let id = self.competition_id;
        self.registrations =
            AsyncResource::new(move || atletiek_nu_api::get_competition_registrations(&id))
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &mut dyn AppCtx) {
        match self.registrations.poll() {
            Some(r) => self.is_team = r.iter().find(|v| v.team_name.is_some()).is_some().clone(),
            None => (),
        };

        match &self.registrations {
            AsyncResource::Finished(r) => {
                if r.len() == 0 {
                    ui.centered_and_justified(|ui| ui.label("No registrations!"));
                    return;
                }

                ui.text_edit_singleline(&mut self.search_text);

                // TODO: fix this
                let r: Vec<&RegistrationsListElement> = r
                    .iter()
                    .filter(|v| {
                        v.name.to_lowercase().contains(&self.search_text)
                            || v.events.contains(&self.search_text)
                            || v.club_name.to_lowercase().contains(&self.search_text)
                            || v.category.to_lowercase().contains(&self.search_text)
                            || if let Some(team) = &v.team_name {
                                team.to_lowercase().contains(&self.search_text)
                            } else {
                                false
                            }
                    })
                    .collect();

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .columns(
                        Column::auto().resizable(true),
                        6 + self.is_team as usize + self.results_availible as usize,
                    );

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("ID");
                        });
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Category");
                        });
                        header.col(|ui| {
                            ui.strong("Club");
                        });
                        if self.is_team {
                            header.col(|ui| {
                                ui.strong("Team");
                            });
                        }
                        header.col(|ui| {
                            ui.strong("Events");
                        });
                        if self.results_availible {
                            header.col(|ui| {
                                ui.strong("");
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(20.0, r.len(), |index, mut row| {
                            let i = &r[index];

                            row.col(|ui| {
                                ui.label(i.participant_id.to_string());
                            });
                            row.col(|ui| {
                                ui.label(&i.name);
                            });
                            row.col(|ui| {
                                ui.strong(&i.category);
                            });
                            row.col(|ui| {
                                ui.strong(&i.club_name);
                            });
                            if self.is_team {
                                row.col(|ui| {
                                    ui.label(i.team_name.as_ref().unwrap_or(&"".to_string()));
                                });
                            }
                            row.col(|ui| {
                                ui.label(i.events.join(", "));
                            });
                            if self.results_availible {
                                row.col(|ui| {
                                    if ui.button("Results").clicked() {
                                        ctx.push_event(AppEvent::SpawnWindow(Box::new(
                                            EventResultsWindow::new(
                                                i.participant_id,
                                                i.name.clone(),
                                                self.competition_name.clone(),
                                            ),
                                        )));
                                    }
                                });
                            }
                        })
                    });
            }
            AsyncResource::Error(e) => {
                ui.label(RichText::new(format!("Error: {}", e)).color(Color32::RED));
            }
            AsyncResource::Pending(_) => {
                ui.centered_and_justified(|ui| ui.spinner());
            }
            _ => (),
        }
    }
}
