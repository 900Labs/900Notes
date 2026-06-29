use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn create_audio_note(
    input: CreateAudioNoteInput,
    state: State<'_, AppState>,
) -> Result<AudioNote, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_audio_note(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_audio_note(id: String, state: State<'_, AppState>) -> Result<AudioNote, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_audio_note(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_audio_notes_for_page(
    page_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AudioNote>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_audio_notes_for_page(&page_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_audio_note(
    input: UpdateAudioNoteInput,
    state: State<'_, AppState>,
) -> Result<AudioNote, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_audio_note(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_audio_note(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_audio_note(&id).map_err(|e| e.to_string())
}
