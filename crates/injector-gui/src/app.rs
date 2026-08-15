use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use eframe::egui::{self, Align, Color32, Layout, RichText, ScrollArea, TextEdit};
use eframe::{App, CreationContext, Frame, NativeOptions};
use injector_core::{
    inject, list_processes, InjectRequest, InjectionMethod, InjectionOptions, ProcessInfo,
};

const APP_TITLE: &str = "Injector";

pub fn run() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([560.0, 420.0])
            .with_title(APP_TITLE),
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(InjectorApp::new(cc)))),
    )
}

enum WorkerMsg {
    Progress(String),
    Done(Result<(), String>),
}

struct InjectorApp {
    processes: Vec<ProcessInfo>,
    filter: String,
    selected_pid: Option<u32>,
    dll_path: String,
    method: InjectionMethod,
    clear_path: bool,
    settings_open: bool,
    status: Status,
    rx: Option<Receiver<WorkerMsg>>,
}

enum Status {
    Idle,
    Busy(String),
    Ok(String),
    Err(String),
}

impl InjectorApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        setup_style(&cc.egui_ctx);
        let processes = list_processes().unwrap_or_default();
        Self {
            processes,
            filter: String::new(),
            selected_pid: None,
            dll_path: String::new(),
            method: InjectionMethod::default(),
            clear_path: false,
            settings_open: false,
            status: Status::Idle,
            rx: None,
        }
    }

    fn refresh(&mut self) {
        self.processes = list_processes().unwrap_or_default();
    }

    fn poll_worker(&mut self) {
        let Some(rx) = self.rx.take() else { return };
        let mut finished = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                WorkerMsg::Progress(s) => self.status = Status::Busy(s),
                WorkerMsg::Done(Ok(())) => {
                    self.status = Status::Ok("Injection succeeded".into());
                    finished = true;
                }
                WorkerMsg::Done(Err(e)) => {
                    self.status = Status::Err(e);
                    finished = true;
                }
            }
        }
        if !finished {
            self.rx = Some(rx);
        }
    }

    fn start_inject(&mut self) {
        let Some(pid) = self.selected_pid else {
            self.status = Status::Err("Select a process".into());
            return;
        };
        let dll = self.dll_path.trim();
        if dll.is_empty() {
            self.status = Status::Err("Enter a DLL path".into());
            return;
        }
        let path = PathBuf::from(dll);
        let method = self.method;
        let opts = InjectionOptions {
            clear_path_after: self.clear_path,
        };
        let (tx, rx): (Sender<WorkerMsg>, Receiver<WorkerMsg>) = channel();
        self.rx = Some(rx);
        self.status = Status::Busy("Injecting...".into());

        thread::spawn(move || {
            let _ = tx.send(WorkerMsg::Progress("Injecting...".into()));
            let req = InjectRequest {
                pid,
                dll_path: &path,
                method,
                options: opts,
            };
            let res = inject(&req).map_err(|e| e.to_string());
            let _ = tx.send(WorkerMsg::Done(res));
        });
    }
}

impl App for InjectorApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut Frame) {
        self.poll_worker();
        if self.rx.is_some() {
            ctx.request_repaint();
        }

        let mut open = self.settings_open;
        egui::SidePanel::right("settings")
            .resizable(false)
            .default_width(260.0)
            .show_animated(ctx, open, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Settings");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("close").clicked() {
                            open = false;
                        }
                    });
                });
                ui.separator();
                ui.label("Injection method");
                for m in InjectionMethod::ALL {
                    ui.radio_value(&mut self.method, *m, m.display_name());
                }
                ui.add_space(12.0);
                ui.label("Advanced");
                ui.checkbox(&mut self.clear_path, "Clear DLL path from remote memory");
            });
        self.settings_open = open;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(APP_TITLE);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("settings").clicked() {
                        self.settings_open = !self.settings_open;
                    }
                });
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Process");
                ui.add(
                    TextEdit::singleline(&mut self.filter)
                        .hint_text("filter by name or pid")
                        .desired_width(ui.available_width()),
                );
            });

            let needle = self.filter.to_ascii_lowercase();
            let mut selected = self.selected_pid;
            ScrollArea::vertical()
                .id_salt("proc_list")
                .max_height(220.0)
                .show(ui, |ui| {
                    for p in &self.processes {
                        let matches = needle.is_empty()
                            || p.name.to_ascii_lowercase().contains(&needle)
                            || p.pid.to_string().contains(&needle);
                        if !matches {
                            continue;
                        }
                        let arch = p
                            .architecture
                            .map(|a| format!("{a:?}"))
                            .unwrap_or_else(|| "?".into());
                        let label = format!("{:>6}  {:<4}  {}", p.pid, arch, p.name);
                        let resp = ui.selectable_label(
                            selected == Some(p.pid),
                            RichText::new(label).monospace(),
                        );
                        if resp.clicked() {
                            selected = Some(p.pid);
                        }
                    }
                });
            self.selected_pid = selected;

            if ui.button("Refresh").clicked() {
                self.refresh();
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("DLL");
                let browse_clicked = ui.add(egui::Button::new("Browse")).clicked();
                ui.add(
                    TextEdit::singleline(&mut self.dll_path)
                        .hint_text(r"C:\path\to\payload.dll")
                        .desired_width(ui.available_width()),
                );
                if browse_clicked {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("DLL", &["dll"])
                        .pick_file()
                    {
                        self.dll_path = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.add_space(12.0);
            let can_inject = self.rx.is_none();
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(RichText::new("Inject").strong().size(16.0))
                    .min_size(egui::vec2(180.0, 34.0));
                if ui.add_enabled(can_inject, btn).clicked() {
                    self.start_inject();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            let (label, color) = match &self.status {
                Status::Idle => ("Ready".into(), Color32::GRAY),
                Status::Busy(s) => (s.clone(), Color32::LIGHT_BLUE),
                Status::Ok(s) => (s.clone(), Color32::LIGHT_GREEN),
                Status::Err(s) => (s.clone(), Color32::LIGHT_RED),
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new("Status:").strong());
                ui.label(RichText::new(label).color(color));
            });
        });
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    ctx.set_style(style);
    ctx.set_visuals(egui::Visuals::dark());
}
