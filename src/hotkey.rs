use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

pub struct HotkeyManager {
    _manager: GlobalHotKeyManager,
    pub summon_hotkey: HotKey,
}

impl HotkeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;

        // Raccourci par défaut style Antinote : Alt + A (ou Option + A sur Mac)
        let summon_hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyA);

        manager.register(summon_hotkey)?;

        Ok(Self {
            _manager: manager,
            summon_hotkey,
        })
    }
}

pub fn poll_hotkey_events(hotkey: &HotkeyManager) -> bool {
    if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        if event.id == hotkey.summon_hotkey.id() && event.state == global_hotkey::HotKeyState::Released {
            return true;
        }
    }
    false
}
