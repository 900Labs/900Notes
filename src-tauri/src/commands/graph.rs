use tauri::State;

use crate::models::*;
use crate::AppState;

#[tauri::command]
pub fn get_graph_data(state: State<'_, AppState>) -> Result<GraphData, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_graph_data().map_err(|e| e.to_string())
}
