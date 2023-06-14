use crate::app::{AppCtx, Window};
use crate::async_resource::AsyncResource;
use atletiek_nu_api::AthleteEventResults;
use egui::{Color32, RichText, Ui};
use std::collections::{BTreeMap, HashMap};

pub struct EventResultsWindow {
    id: u32,
    name: String,
    competition_name: String,
    results: AsyncResource<AthleteEventResults>,
}

impl EventResultsWindow {
    pub fn new(id: u32, name: String, competition_name: String) -> Self {
        Self {
            results: AsyncResource::default(),
            id,
            name,
            competition_name,
        }
    }
}

impl Window for EventResultsWindow {
    fn title(&self) -> String {
        format!(
            "Results of {} at {} [{}]",
            self.name, self.competition_name, self.id
        )
    }

    fn on_spawn(&mut self) {
        let id = self.id;
        self.results = AsyncResource::new(move || atletiek_nu_api::get_athlete_event_result(id));
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &mut dyn AppCtx) {
        self.results.poll();

        match &self.results {
            AsyncResource::Pending(_) => {
                ui.centered_and_justified(|ui| ui.spinner());
            }
            AsyncResource::Error(e) => {
                ui.centered_and_justified(|ui| ui.label(RichText::new(e).color(Color32::RED)));
            }
            AsyncResource::Finished(r) => {
                let mut results: BTreeMap<String, Vec<(f64, Option<f64>)>> = BTreeMap::new();
                for i in &r.results {
                    if let Some(res) = results.get_mut(&i.event_name) {
                        res.push((i.result, i.wind_speed));
                    } else {
                        results.insert(i.event_name.to_owned(), vec![(i.result, i.wind_speed)]);
                    }
                }

                ui.label(format!("Name:           {}", self.name));
                ui.label(format!("Competition:    {}", self.competition_name));
                ui.label(format!("Participant ID: {}", self.id));

                for (name, results) in results.iter() {
                    ui.add_space(20.0);
                    ui.strong(name);
                    for (result, wind) in results {
                        let result_text = if result.is_nan() {
                            "x".to_string()
                        } else {
                            format!("{:.2}", result)
                        };

                        let (text, valid) = if let Some(wind) = wind {
                            (
                                format!("{} ({:.1} m/s wind)", result_text, wind),
                                (wind <= &2.0 && wind >= &-2.0),
                            )
                        } else {
                            (result_text, true)
                        };

                        ui.label(RichText::new(text).color(if !valid {
                            Color32::RED
                        } else {
                            Color32::WHITE
                        }));
                    }
                }
            }
            _ => (),
        }
    }
}
