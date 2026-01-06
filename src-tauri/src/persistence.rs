// Persistence module - Save and load authentication state
use crate::state::{AuthState, LicenciaProducto};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Saved session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub auth: AuthState,
    pub licencias: Vec<LicenciaProducto>,
}

/// Get the path to the session file
fn get_session_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|mut path| {
        path.push("ISIPRINT");
        path.push("session.json");
        path
    })
}

/// Save session to disk
pub fn save_session(auth: &AuthState, licencias: &[LicenciaProducto]) -> Result<(), String> {
    let path = get_session_path().ok_or("Could not determine data directory")?;
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let session = SavedSession {
        auth: auth.clone(),
        licencias: licencias.to_vec(),
    };
    
    let json = serde_json::to_string_pretty(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write session file: {}", e))?;
    
    log::info!("Session saved to {:?}", path);
    Ok(())
}

/// Load session from disk
pub fn load_session() -> Option<SavedSession> {
    let path = get_session_path()?;
    
    if !path.exists() {
        log::info!("No saved session found");
        return None;
    }
    
    match fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str::<SavedSession>(&json) {
                Ok(session) => {
                    log::info!("Session loaded from {:?}", path);
                    Some(session)
                }
                Err(e) => {
                    log::warn!("Failed to parse session: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to read session file: {}", e);
            None
        }
    }
}

/// Clear saved session
pub fn clear_session() -> Result<(), String> {
    let path = get_session_path().ok_or("Could not determine data directory")?;
    
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete session file: {}", e))?;
        log::info!("Session cleared");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_session_path() {
        let path = get_session_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("ISIPRINT"));
        assert!(path.to_string_lossy().contains("session.json"));
    }
    
    #[test]
    fn test_saved_session_serialization() {
        let session = SavedSession {
            auth: AuthState {
                token: Some("test_token".to_string()),
                refresh_token: Some("test_refresh".to_string()),
                email: Some("test@example.com".to_string()),
                is_logged_in: true,
            },
            licencias: vec![],
        };
        
        let json = serde_json::to_string(&session).unwrap();
        let parsed: SavedSession = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.auth.token, Some("test_token".to_string()));
        assert_eq!(parsed.auth.email, Some("test@example.com".to_string()));
        assert!(parsed.auth.is_logged_in);
    }
}
