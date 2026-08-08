use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum TrayCommand {
    AnswerCall,
    EndCall,
    ToggleMute,
    OpenSettings,
    Quit,
}

pub struct TrayUI {
    cmd_rx: Receiver<TrayCommand>,
    _tray: tray_icon::TrayIcon,
}

impl TrayUI {
    pub fn new() -> Result<(Self, Sender<TrayCommand>), String> {
        let (cmd_tx, cmd_rx) = channel::<TrayCommand>();
        let cmd_tx_clone = cmd_tx.clone();

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

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("PhoneBridge - Wireless Headset")
            .build()
            .map_err(|e| format!("Failed to create tray icon: {}", e))?;

        std::thread::spawn(move || {
            let menu_channel = MenuEvent::receiver();
            loop {
                if let Ok(event) = menu_channel.try_recv() {
                    if event.id == answer_item.id() {
                        let _ = cmd_tx_clone.send(TrayCommand::AnswerCall);
                    } else if event.id == end_item.id() {
                        let _ = cmd_tx_clone.send(TrayCommand::EndCall);
                    } else if event.id == mute_item.id() {
                        let _ = cmd_tx_clone.send(TrayCommand::ToggleMute);
                    } else if event.id == settings_item.id() {
                        let _ = cmd_tx_clone.send(TrayCommand::OpenSettings);
                    } else if event.id == quit_item.id() {
                        let _ = cmd_tx_clone.send(TrayCommand::Quit);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        Ok((Self { cmd_rx, _tray: tray }, cmd_tx))
    }

    pub fn poll_commands(&self) -> Vec<TrayCommand> {
        let mut cmds = Vec::new();
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            cmds.push(cmd);
        }
        cmds
    }
}

pub struct AsyncTrayUI {
    inner: std::sync::Arc<tokio::sync::Mutex<TrayUI>>,
}

impl AsyncTrayUI {
    pub fn new() -> Result<(Self, Sender<TrayCommand>), String> {
        let (tray, tx) = TrayUI::new()?;
        Ok((Self {
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(tray)),
        }, tx))
    }

    pub async fn poll(&self) -> Vec<TrayCommand> {
        let tray = self.inner.lock().await;
        tray.poll_commands()
    }

    pub async fn notify(&self, title: &str, message: &str) {
        let tray = self.inner.lock().await;
        log::info!("[NOTIFY] {}: {}", title, message);
    }
}
