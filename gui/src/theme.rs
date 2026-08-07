const STORAGE_KEY: &str = "tabletennis-tournament-dark-mode";

pub(crate) fn load_dark_mode() -> bool {
    storage()
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .is_some_and(|value| value == "true")
}

pub(crate) fn store_dark_mode(enabled: bool) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(STORAGE_KEY, if enabled { "true" } else { "false" });
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
}
