// Estado global de la aplicación
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Información de licencia del producto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenciaProducto {
    pub _id: String,
    #[serde(rename = "tipoProducto")]
    pub tipo_producto: String,
    #[serde(rename = "maximoConexiones")]
    pub maximo_conexiones: i32,
    #[serde(rename = "fechaVencimiento")]
    pub fecha_vencimiento: String,
    pub delegado: bool,
    pub configuracion: Option<String>,
    pub state: String,
}

/// Estado de autenticación
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthState {
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub email: Option<String>,
    pub is_logged_in: bool,
}

/// Registro de log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Estado global de la aplicación
#[derive(Debug, Default)]
pub struct AppState {
    pub auth: AuthState,
    pub licencias: Vec<LicenciaProducto>,
    pub logs: VecDeque<LogEntry>,
    pub print_count: i32,
}

impl AppState {
    /// Agregar un log al estado
    pub fn add_log(&mut self, level: &str, message: &str) {
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };
        
        // Mantener solo los últimos 100 logs
        if self.logs.len() >= 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(entry);
    }

    /// Verificar si la licencia está vigente
    pub fn is_license_valid(&self) -> bool {
        if let Some(licencia) = self.licencias.iter().find(|l| l.tipo_producto == "IMPRESION") {
            if licencia.state != "ACTIVADO" {
                return false;
            }
            
            // Parsear fecha de vencimiento (formato: "DD/MM/YYYY HH:MM:SS")
            if let Ok(fecha) = chrono::NaiveDateTime::parse_from_str(
                &licencia.fecha_vencimiento,
                "%d/%m/%Y %H:%M:%S"
            ) {
                let now = chrono::Local::now().naive_local();
                return fecha > now;
            }
        }
        false
    }

    /// Verificar si se puede imprimir (límite de conexiones)
    pub fn can_print(&self) -> bool {
        if let Some(licencia) = self.licencias.iter().find(|l| l.tipo_producto == "IMPRESION") {
            return self.print_count < licencia.maximo_conexiones;
        }
        false
    }
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: crear una licencia de prueba válida
    fn create_test_license(fecha_vencimiento: &str, state: &str, max_conexiones: i32) -> LicenciaProducto {
        LicenciaProducto {
            _id: "test123".to_string(),
            tipo_producto: "IMPRESION".to_string(),
            maximo_conexiones: max_conexiones,
            fecha_vencimiento: fecha_vencimiento.to_string(),
            delegado: true,
            configuracion: None,
            state: state.to_string(),
        }
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.auth.is_logged_in);
        assert!(state.auth.token.is_none());
        assert!(state.licencias.is_empty());
        assert!(state.logs.is_empty());
        assert_eq!(state.print_count, 0);
    }

    #[test]
    fn test_add_log() {
        let mut state = AppState::default();
        
        state.add_log("INFO", "Test message");
        assert_eq!(state.logs.len(), 1);
        
        let log = state.logs.front().unwrap();
        assert_eq!(log.level, "INFO");
        assert_eq!(log.message, "Test message");
    }

    #[test]
    fn test_add_log_max_100() {
        let mut state = AppState::default();
        
        // Agregar 105 logs
        for i in 0..105 {
            state.add_log("INFO", &format!("Message {}", i));
        }
        
        // Solo deben quedar 100
        assert_eq!(state.logs.len(), 100);
        
        // El primer mensaje debe ser el #5 (los primeros 5 fueron eliminados)
        let first_log = state.logs.front().unwrap();
        assert_eq!(first_log.message, "Message 5");
    }

    #[test]
    fn test_is_license_valid_with_future_date() {
        let mut state = AppState::default();
        
        // Licencia que vence en el futuro
        state.licencias.push(create_test_license(
            "01/01/2030 00:00:00",
            "ACTIVADO",
            100
        ));
        
        assert!(state.is_license_valid(), "Licencia con fecha futura debería ser válida");
    }

    #[test]
    fn test_is_license_valid_with_past_date() {
        let mut state = AppState::default();
        
        // Licencia que ya venció
        state.licencias.push(create_test_license(
            "01/01/2020 00:00:00",
            "ACTIVADO",
            100
        ));
        
        assert!(!state.is_license_valid(), "Licencia vencida NO debería ser válida");
    }

    #[test]
    fn test_is_license_valid_inactive() {
        let mut state = AppState::default();
        
        // Licencia desactivada
        state.licencias.push(create_test_license(
            "01/01/2030 00:00:00",
            "INACTIVO",
            100
        ));
        
        assert!(!state.is_license_valid(), "Licencia inactiva NO debería ser válida");
    }

    #[test]
    fn test_is_license_valid_no_license() {
        let state = AppState::default();
        assert!(!state.is_license_valid(), "Sin licencia NO debería ser válido");
    }

    #[test]
    fn test_can_print_within_limit() {
        let mut state = AppState::default();
        state.licencias.push(create_test_license(
            "01/01/2030 00:00:00",
            "ACTIVADO",
            100
        ));
        state.print_count = 50;
        
        assert!(state.can_print(), "Debería poder imprimir (50 < 100)");
    }

    #[test]
    fn test_can_print_at_limit() {
        let mut state = AppState::default();
        state.licencias.push(create_test_license(
            "01/01/2030 00:00:00",
            "ACTIVADO",
            100
        ));
        state.print_count = 100;
        
        assert!(!state.can_print(), "NO debería poder imprimir (100 >= 100)");
    }

    #[test]
    fn test_can_print_over_limit() {
        let mut state = AppState::default();
        state.licencias.push(create_test_license(
            "01/01/2030 00:00:00",
            "ACTIVADO",
            100
        ));
        state.print_count = 150;
        
        assert!(!state.can_print(), "NO debería poder imprimir (150 > 100)");
    }

    #[test]
    fn test_can_print_no_license() {
        let state = AppState::default();
        assert!(!state.can_print(), "Sin licencia NO debería poder imprimir");
    }

    #[test]
    fn test_auth_state_default() {
        let auth = AuthState::default();
        assert!(auth.token.is_none());
        assert!(auth.refresh_token.is_none());
        assert!(auth.email.is_none());
        assert!(!auth.is_logged_in);
    }
}
