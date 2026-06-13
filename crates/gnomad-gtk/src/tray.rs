use std::sync::mpsc;
use std::thread;

use ksni::blocking::TrayMethods;
use ksni::menu::{MenuItem, StandardItem};
use ksni::{ToolTip, Tray};

use crate::icons;
use crate::state::UiCommand;

const STUDIO_URL: &str = "https://gnomadstudio.org";

struct GnomadTray {
    cmd_tx: mpsc::Sender<UiCommand>,
}

impl Tray for GnomadTray {
    const MENU_ON_ACTIVATE: bool = false;

    fn id(&self) -> String {
        "com.gnomadstudio.gnomad".into()
    }

    fn title(&self) -> String {
        "Gnomad".into()
    }

    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        icons::tray_icons()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: String::new(),
            icon_pixmap: icons::tray_icons(),
            title: "Gnomad".into(),
            description: "Left-click: open window · Right-click: menu".into(),
        }
    }

    fn activate(&mut self, x: i32, y: i32) {
        let _ = self.cmd_tx.send(UiCommand::TrayActivate { x, y });
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            MenuItem::Standard(StandardItem {
                label: "Show Gnomad — Open or focus the standard window".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::ShowWindowed);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Pop Out — Floating window that stays above others".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::ShowFloating);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Settings — Agent, secrets, knowledge, and API keys".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::OpenSettings);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Hide — Minimize to tray (app keeps running)".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::HidePanel);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Gnomad Studio — Visit gnomadstudio.org".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::OpenUrl(STUDIO_URL));
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit — Exit Gnomad completely".into(),
                activate: Box::new({
                    let tx = self.cmd_tx.clone();
                    move |_| {
                        let _ = tx.send(UiCommand::Quit);
                    }
                }),
                ..Default::default()
            }),
        ]
    }
}

pub fn spawn_tray(cmd_tx: mpsc::Sender<UiCommand>) {
    thread::spawn(move || {
        let tray = GnomadTray { cmd_tx };
        if let Err(e) = tray.spawn() {
            eprintln!("gnomad-gtk: tray failed to start: {e}");
        }
    });
}
