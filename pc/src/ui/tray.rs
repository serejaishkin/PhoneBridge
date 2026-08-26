//! System tray integration for Windows and macOS.
//!
//! The tray must be created on the OS main thread (NSStatusItem requirement on
//! macOS), so PhoneBridgeApp constructs it inside its eframe creator closure
//! and drains menu events every UI frame.
//!
//! Linux is intentionally not covered here yet: tray-icon's Linux backend needs
//! a GTK main loop, which conflicts with winit owning the main thread of this
//! app. On Linux the eframe window alone provides the interface today.

use std::sync::mpsc::{channel, Receiver};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

#[derive(Debug, Clone)]
pub enum TrayCommand {
    AnswerCall,
    EndCall,
    ToggleMute,
    OpenSettings,
    Quit,
}

/// Builds the tray icon image at runtime: a small blue disc with a lighter
/// ring, so no binary asset is needed.
fn build_icon() -> Result<Icon, String> {
    const SIZE: usize = 32;
    let mut rgba = Vec::with_capacity(SIZE * SIZE * 4);
    let center = (SIZE as f32 - 1.0) / 2.0;
    let radius = SIZE as f32 / 2.0 - 1.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let (r, g, b, a) = if dist <= radius {
                if dist >= radius - 4.0 { (230, 240, 255, 255) } else { (30, 100, 220, 255) }
            } else {
                (0, 0, 0, 0)
            };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }
    Icon::from_rgba(rgba, SIZE as u32, SIZE as u32).map_err(|e| e.to_string())
}

pub struct TrayUI {
    cmd_rx: Receiver<TrayCommand>,
    _tray: tray_icon::TrayIcon,
}

impl TrayUI {
    /// Must be called on the OS main thread (see module docs).
    pub fn new() -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = channel::<TrayCommand>();

        let menu = Menu::new();
        let answer_item = MenuItem::new("Answer Call", true, None);
        let end_item = MenuItem::new("End Call", true, None);
        let mute_item = MenuItem::new("Mute Mic", true, None);
        let settings_item = MenuItem::new("Settings...", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&answer_item).map_err(|e| e.to_string())?;
        menu.append(&end_item).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
        menu.append(&mute_item).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
        menu.append(&settings_item).map_err(|e| e.to_string())?;
        menu.append(&quit_item).map_err(|e| e.to_string())?;

        let icon = build_icon()?;
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("PhoneBridge")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("failed to create tray icon: {e}"))?;

        // Menu items are !Send (Rc-backed), so only their cheap ids cross the
        // thread boundary; ids are plain string wrappers.
        let answer_id = answer_item.id().clone();
        let end_id = end_item.id().clone();
        let mute_id = mute_item.id().clone();
        let settings_id = settings_item.id().clone();
        let quit_id = quit_item.id().clone();

        // Forwarding thread: muda delivers menu events through a global channel;
        // reading it from a helper thread keeps the UI frame loop allocation-free.
        std::thread::spawn(move || {
            let menu_channel = MenuEvent::receiver();
            while let Ok(event) = menu_channel.recv() {
                let command = if event.id == answer_id {
                    Some(TrayCommand::AnswerCall)
                } else if event.id == end_id {
                    Some(TrayCommand::EndCall)
                } else if event.id == mute_id {
                    Some(TrayCommand::ToggleMute)
                } else if event.id == settings_id {
                    Some(TrayCommand::OpenSettings)
                } else if event.id == quit_id {
                    Some(TrayCommand::Quit)
                } else {
                    None
                };
                if let Some(command) = command {
                    let quitting = matches!(command, TrayCommand::Quit);
                    let _ = cmd_tx.send(command);
                    if quitting {
                        break;
                    }
                }
            }
        });

        Ok(Self { cmd_rx, _tray: tray })
    }

    /// Non-blocking drain of pending commands, called once per UI frame.
    pub fn poll_commands(&self) -> Vec<TrayCommand> {
        let mut cmds = Vec::new();
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            cmds.push(cmd);
        }
        cmds
    }
}
