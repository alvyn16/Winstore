/// Keep native menus and dialogs consistent with the app's themed window controls.
#[tauri::command]
pub fn set_window_appearance(window: tauri::Window, dark: bool) -> Result<(), String> {
    window
        .set_theme(Some(if dark {
            tauri::Theme::Dark
        } else {
            tauri::Theme::Light
        }))
        .map_err(|error| error.to_string())
}
