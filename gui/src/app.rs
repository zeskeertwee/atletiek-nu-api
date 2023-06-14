use crate::windows::requests::RequestsWindow;
use crate::windows::search_athletes::AthleteSearchWindow;
use crate::windows::search_competitions::SearchCompetitionsWindow;
use eframe::{CreationContext, Frame};
use egui::{Context, Id, TopBottomPanel, Ui};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc::{sync_channel, Receiver};
use std::time::{Duration, Instant};

pub struct App {
    windows: Vec<(Id, Box<dyn Window>)>,
    event_queue: VecDeque<AppEvent>,
    counter: usize,
    req_rx: Receiver<(usize, atletiek_nu_api::Request)>,
    sta_rx: Receiver<(usize, atletiek_nu_api::StatusCode)>,
    requests: HashMap<usize, RequestState>,
}

pub enum AppEvent {
    SpawnWindow(Box<dyn Window>),
    PopWindow(Id),
}

pub enum RequestState {
    Pending {
        req: atletiek_nu_api::Request,
        started_at: Instant,
    },
    Finished {
        req: atletiek_nu_api::Request,
        code: atletiek_nu_api::StatusCode,
        duration: Duration,
    },
}

pub trait Window {
    fn title(&self) -> String;
    fn draw(&mut self, ui: &mut Ui, ctx: &mut dyn AppCtx);
    fn on_spawn(&mut self) {}
}

pub trait AppCtx {
    fn push_event(&mut self, event: AppEvent);
    fn get_requests(&self) -> &HashMap<usize, RequestState>;
}

struct AppCtxProvider<'a> {
    requests: &'a HashMap<usize, RequestState>,
    events: Vec<AppEvent>,
}

impl App {
    pub fn new(_: &CreationContext) -> Self {
        let (req_tx, req_rx) = sync_channel::<(usize, atletiek_nu_api::Request)>(1000);
        let (sta_tx, sta_rx) = sync_channel::<(usize, atletiek_nu_api::StatusCode)>(1000);

        atletiek_nu_api::set_request_callback(req_tx, sta_tx);

        Self {
            windows: Vec::new(),
            event_queue: VecDeque::new(),
            counter: 0,
            req_rx,
            sta_rx,
            requests: HashMap::new(),
        }
    }

    pub fn process_events(&mut self) {
        if self.event_queue.len() == 0 {
            return;
        }

        let mut queue = VecDeque::new();
        std::mem::swap(&mut queue, &mut self.event_queue);

        for i in queue {
            match i {
                AppEvent::SpawnWindow(window) => self.add_boxed_window(window),
                AppEvent::PopWindow(id) => {
                    self.windows.swap_remove(
                        self.windows
                            .iter()
                            .enumerate()
                            .find(|(_, w)| w.0 == id)
                            .unwrap()
                            .0,
                    );
                }
            }
        }
    }

    fn update_requests(&mut self) {
        for i in self.req_rx.try_iter() {
            self.requests.insert(
                i.0,
                RequestState::Pending {
                    req: i.1,
                    started_at: Instant::now(),
                },
            );
            log::info!("Inserted new request {}", i.0);
        }

        for i in self.sta_rx.try_iter() {
            let req = self.requests.remove(&i.0).unwrap();
            let new_req = match req {
                RequestState::Pending { req, started_at } => RequestState::Finished {
                    req,
                    code: i.1,
                    duration: started_at.elapsed(),
                },
                other => {
                    log::warn!("Tried to update non-pending request!");
                    other
                }
            };
            self.requests.insert(i.0, new_req);

            log::info!("Updated request {} status to {}", i.0, i.1);
        }
    }

    pub fn add_window<W: Window + 'static>(&mut self, window: W) {
        self.add_boxed_window(Box::new(window))
    }

    pub fn add_boxed_window(&mut self, mut window: Box<dyn Window>) {
        window.on_spawn();
        self.windows
            .push((Id::new(format!("window-{}", self.counter)), window));
        self.counter += 1;
    }

    pub fn draw_top_bar(ctx: &Context, event_queue: &mut VecDeque<AppEvent>) {
        TopBottomPanel::top("top-menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Search competitions").clicked() {
                    event_queue.push_back(AppEvent::SpawnWindow(Box::new(
                        SearchCompetitionsWindow::default(),
                    )));
                }

                if ui.button("Search athletes").clicked() {
                    event_queue.push_back(AppEvent::SpawnWindow(Box::new(
                        AthleteSearchWindow::default(),
                    )));
                }

                if ui.button("View requests").clicked() {
                    event_queue
                        .push_back(AppEvent::SpawnWindow(Box::new(RequestsWindow::default())));
                }
            });
        });
    }
}

impl<'a> AppCtx for AppCtxProvider<'a> {
    fn get_requests(&self) -> &HashMap<usize, RequestState> {
        self.requests
    }

    fn push_event(&mut self, event: AppEvent) {
        self.events.push(event)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        self.process_events();
        self.update_requests();

        App::draw_top_bar(ctx, &mut self.event_queue);

        let mut provider = AppCtxProvider {
            events: Vec::new(),
            requests: &self.requests,
        };

        for (window_id, window) in self.windows.iter_mut() {
            let mut open = true;

            egui::Window::new(window.title())
                .open(&mut open)
                .id(*window_id)
                .show(ctx, |ui| {
                    window.draw(ui, &mut provider);
                });

            if !open {
                self.event_queue
                    .push_back(AppEvent::PopWindow(window_id.to_owned()))
            }
        }

        self.event_queue.extend(provider.events);
    }
}
