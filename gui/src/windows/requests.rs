use crate::app::{AppCtx, RequestState, Window};
use egui::{Align, Layout, Ui};
use egui_extras::{Column, TableBuilder};

pub struct RequestsWindow;

impl Default for RequestsWindow {
    fn default() -> Self {
        RequestsWindow
    }
}

impl Window for RequestsWindow {
    fn title(&self) -> String {
        "Web requests".to_string()
    }

    fn draw(&mut self, ui: &mut Ui, ctx: &mut dyn AppCtx) {
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
                    ui.strong("URL");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("Duration (ms)");
                });
            })
            .body(|body| {
                body.rows(20.0, ctx.get_requests().len(), |index, mut row| {
                    let request = ctx.get_requests().get(&index).unwrap();
                    let (id, url, status, duration) = match request {
                        RequestState::Pending { req, started_at } => (
                            index,
                            req.url().as_str(),
                            "Pending",
                            started_at.elapsed().as_millis(),
                        ),
                        RequestState::Finished {
                            req,
                            code,
                            duration,
                        } => (
                            index,
                            req.url().as_str(),
                            code.as_str(),
                            duration.as_millis(),
                        ),
                    };

                    row.col(|ui| {
                        ui.label(id.to_string());
                    });
                    row.col(|ui| {
                        ui.label(url);
                    });
                    row.col(|ui| {
                        ui.label(status);
                    });
                    row.col(|ui| {
                        ui.label(duration.to_string());
                    });
                })
            })
    }
}
