use axum::{
    extract::State,
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::printer;
use crate::state::{AppState, LogEntry};

/// Estado compartido del servidor HTTP
pub struct HttpServerState {
    pub app_state: Arc<RwLock<AppState>>,
}

/// Dominios permitidos para CORS
const DOMINIOS_PERMITIDOS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "*.integrate.com.bo",
    "*.isipass.net",
    "*.adeabordo.com",
    "*.adeabordo.com.bo",
    "*.adesite.net",
    "*.isipass.app",
    "*.pruebas.isipass.net",
];

/// Verifica si un origen está permitido
fn is_origin_allowed(origin: &str) -> bool {
    // Origen vacío se permite (para peticiones directas, curl, Postman, etc.)
    if origin.is_empty() {
        return true;
    }

    // Extraer el hostname del origen
    let host = if let Some(stripped) = origin.strip_prefix("https://") {
        stripped.split('/').next().unwrap_or("")
    } else if let Some(stripped) = origin.strip_prefix("http://") {
        stripped.split('/').next().unwrap_or("")
    } else {
        origin
    };

    // Quitar el puerto si existe
    let host_without_port = host.split(':').next().unwrap_or(host);

    for pattern in DOMINIOS_PERMITIDOS {
        if pattern.starts_with("*.") {
            // Patrón con wildcard
            let suffix = &pattern[1..]; // ".integrate.com.bo"
            if host_without_port.ends_with(suffix) || host_without_port == &pattern[2..] {
                return true;
            }
        } else if host_without_port == *pattern {
            return true;
        }
    }
    false
}

/// Middleware to verify origin
fn verify_origin(headers: &HeaderMap) -> Result<(), (StatusCode, &'static str)> {
    let origin = headers
        .get(header::ORIGIN)
        .or_else(|| headers.get(header::REFERER))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if origin.is_empty() || is_origin_allowed(origin) {
        Ok(())
    } else {
        log::warn!("Unauthorized origin: {}", origin);
        Err((StatusCode::FORBIDDEN, "Unauthorized origin"))
    }
}

/// Verifies if the user is authenticated
async fn verify_auth(state: &Arc<HttpServerState>) -> Result<(), (StatusCode, Json<PrintResponse>)> {
    let app = state.app_state.read().await;
    if !app.auth.is_logged_in {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(PrintResponse {
                success: false,
                message: "Authentication required. Please log in to the application first.".to_string(),
            }),
        ));
    }
    
    // Verify if user has a valid license
    if !app.is_license_valid() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(PrintResponse {
                success: false,
                message: "No valid license found. Please check your subscription.".to_string(),
            }),
        ));
    }
    
    Ok(())
}

/// Crea un LogEntry con timestamp actual
fn create_log_entry(level: &str, message: String) -> LogEntry {
    LogEntry {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        level: level.to_string(),
        message,
    }
}

// ============ Request/Response types ============

#[derive(Debug, Deserialize)]
pub struct PrintRequest {
    #[serde(alias = "impresora")]
    pub printer: Option<String>,
    #[serde(alias = "contenido")]
    pub content: Option<String>,
    #[serde(default = "default_copies")]
    pub copies: u32,
}

fn default_copies() -> u32 { 1 }

#[derive(Debug, Deserialize)]
pub struct PrintPdfRequest {
    #[serde(alias = "impresora")]
    pub printer: Option<String>,
    pub url: Option<String>,
    #[serde(default = "default_copies")]
    pub copies: u32,
}

#[derive(Debug, Serialize)]
pub struct PrintResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PrintersResponse {
    pub printers: Vec<PrinterInfo>,
}

#[derive(Debug, Serialize)]
pub struct PrinterInfo {
    pub name: String,
    #[serde(rename = "default")]
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
pub struct PrintJobsResponse {
    pub jobs: Vec<PrintJobInfo>,
}

#[derive(Debug, Serialize)]
pub struct PrintJobInfo {
    pub id: String,
    pub timestamp: String,
    pub status: String,
    pub message: String,
}

// ============ Route handlers ============

/// GET / - Estado del servidor
async fn index() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "app": "ISIPRINT Client",
        "version": "1.0.0"
    }))
}

/// GET /printers - List available printers
async fn get_printers(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    match printer::list_printers() {
        Ok(printers) => {
            let printer_list: Vec<PrinterInfo> = printers
                .into_iter()
                .enumerate()
                .map(|(i, name)| PrinterInfo {
                    name,
                    is_default: i == 0, // First printer as default
                })
                .collect();

            // Log
            let log_entry = create_log_entry("info", format!("Listed {} printers", printer_list.len()));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            Json(PrintersResponse { printers: printer_list }).into_response()
        }
        Err(e) => {
            log::error!("Error listing printers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: e,
                }),
            )
                .into_response()
        }
    }
}

/// POST /print - Print RAW content (ESC/POS, text, etc.)
async fn print_raw(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
    Json(payload): Json<PrintRequest>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    let content = match payload.content {
        Some(c) if !c.is_empty() => c,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PrintResponse {
                    success: false,
                    message: "Content required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let printer_name = match payload.printer {
        Some(p) if !p.is_empty() => p,
        _ => {
            // Get default printer
            match printer::list_printers() {
                Ok(printers) if !printers.is_empty() => printers[0].clone(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(PrintResponse {
                            success: false,
                            message: "No printers available".to_string(),
                        }),
                    )
                        .into_response()
                }
            }
        }
    };

    // Create temp file with content
    use std::io::Write;
    use tempfile::NamedTempFile;

    let temp_result = NamedTempFile::new();
    let mut temp_file = match temp_result {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error creating temp file: {}", e),
                }),
            )
                .into_response()
        }
    };

    if let Err(e) = temp_file.write_all(content.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PrintResponse {
                success: false,
                message: format!("Error writing content: {}", e),
            }),
        )
            .into_response();
    }

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    let path = match temp_file.into_temp_path().keep() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error persisting temp file: {}", e),
                }),
            )
                .into_response();
        }
    };
    let path_str = path.to_string_lossy().to_string();

    match printer::print_file(&path_str, &printer_name) {
        Ok(job_id) => {
            // Don't delete the temp file - let the system clean it up later
            // Virtual printers like PDFwriter need time to process the file
            
            let msg = format!("Printed successfully. Job ID: {}", job_id);
            let log_entry = create_log_entry("success", format!("RAW print: {}", msg));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            Json(PrintResponse {
                success: true,
                message: msg,
            })
            .into_response()
        }
        Err(e) => {
            let _ = std::fs::remove_file(&path);
            
            let log_entry = create_log_entry("error", format!("RAW print error: {}", e));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: e,
                }),
            )
                .into_response()
        }
    }
}

/// POST /printPDF - Download and print a PDF from URL
async fn print_pdf(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
    Json(payload): Json<PrintPdfRequest>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    let url = match payload.url {
        Some(u) if !u.is_empty() => u,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PrintResponse {
                    success: false,
                    message: "URL required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let printer_name = match payload.printer {
        Some(p) if !p.is_empty() => p,
        _ => {
            match printer::list_printers() {
                Ok(printers) if !printers.is_empty() => printers[0].clone(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(PrintResponse {
                            success: false,
                            message: "No printers available".to_string(),
                        }),
                    )
                        .into_response()
                }
            }
        }
    };

    // Download PDF
    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error downloading PDF: {}", e),
                }),
            )
                .into_response()
        }
    };

    let pdf_bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error reading PDF: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Save to temporary file
    use tempfile::Builder;

    let temp_file = match Builder::new()
        .prefix("isiprint_http_pdf_")
        .suffix(".pdf")
        .tempfile()
    {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error creating temp file: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    let path = match temp_file.into_temp_path().keep() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error persisting temp file: {}", e),
                }),
            )
                .into_response();
        }
    };
    let path_str = path.to_string_lossy().to_string();
    
    if let Err(e) = std::fs::write(&path, &pdf_bytes) {
        let _ = std::fs::remove_file(&path);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PrintResponse {
                success: false,
                message: format!("Error saving PDF: {}", e),
            }),
        )
            .into_response();
    }

    match printer::print_file(&path_str, &printer_name) {
        Ok(job_id) => {
            // Don't delete the temp file - let the system clean it up later
            // Virtual printers like PDFwriter need time to process the file
            
            let msg = format!("PDF printed successfully. Job ID: {}", job_id);
            let log_entry = create_log_entry("success", format!("PDF print: {}", msg));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            Json(PrintResponse {
                success: true,
                message: msg,
            })
            .into_response()
        }
        Err(e) => {
            let _ = std::fs::remove_file(&path);
            
            let log_entry = create_log_entry("error", format!("PDF print error: {}", e));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: e,
                }),
            )
                .into_response()
        }
    }
}

/// POST /printPDF/upload with multipart (file uploaded directly)
async fn print_pdf_multipart(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    let mut pdf_data: Option<Vec<u8>> = None;
    let mut printer_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" | "pdf" => {
                if let Ok(data) = field.bytes().await {
                    pdf_data = Some(data.to_vec());
                }
            }
            "impresora" | "printer" => {
                if let Ok(text) = field.text().await {
                    printer_name = Some(text);
                }
            }
            _ => {}
        }
    }

    let data = match pdf_data {
        Some(d) if !d.is_empty() => d,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PrintResponse {
                    success: false,
                    message: "PDF file required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let printer = match printer_name {
        Some(p) if !p.is_empty() => p,
        _ => {
            match printer::list_printers() {
                Ok(printers) if !printers.is_empty() => printers[0].clone(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(PrintResponse {
                            success: false,
                            message: "No printers available".to_string(),
                        }),
                    )
                        .into_response()
                }
            }
        }
    };

    // Save to temporary file
    use tempfile::Builder;

    let temp_file = match Builder::new()
        .prefix("isiprint_http_pdf_")
        .suffix(".pdf")
        .tempfile()
    {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error creating temp file: {}", e),
                }),
            )
                .into_response()
        }
    };

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    let path = match temp_file.into_temp_path().keep() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: format!("Error persisting temp file: {}", e),
                }),
            )
                .into_response();
        }
    };
    let path_str = path.to_string_lossy().to_string();
    
    if let Err(e) = std::fs::write(&path, &data) {
        let _ = std::fs::remove_file(&path);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PrintResponse {
                success: false,
                message: format!("Error saving PDF: {}", e),
            }),
        )
            .into_response();
    }

    match printer::print_file(&path_str, &printer) {
        Ok(job_id) => {
            // Don't delete the temp file - let the system clean it up later
            // Virtual printers like PDFwriter need time to process the file
            
            let msg = format!("Multipart PDF printed. Job ID: {}", job_id);
            let log_entry = create_log_entry("success", format!("Multipart PDF print: {}", msg));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            Json(PrintResponse {
                success: true,
                message: msg,
            })
            .into_response()
        }
        Err(e) => {
            let _ = std::fs::remove_file(&path);
            
            let log_entry = create_log_entry("error", format!("Multipart PDF print error: {}", e));
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: e,
                }),
            )
                .into_response()
        }
    }
}

/// GET /print_jobs - List print jobs (recent logs)
async fn get_print_jobs(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    let app = state.app_state.read().await;
    let jobs: Vec<PrintJobInfo> = app.logs
        .iter()
        .filter(|log| log.message.to_lowercase().contains("print") || log.message.to_lowercase().contains("printed"))
        .enumerate()
        .map(|(i, log)| PrintJobInfo {
            id: format!("{}", i),
            timestamp: log.timestamp.clone(),
            status: log.level.clone(),
            message: log.message.clone(),
        })
        .collect();

    Json(PrintJobsResponse { jobs }).into_response()
}

/// POST /clear_jobs - Clear print jobs
async fn clear_print_jobs(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    {
        let mut app = state.app_state.write().await;
        app.logs.clear();
    }

    // También limpiar cola del sistema
    let _ = printer::clear_jobs();

    Json(PrintResponse {
        success: true,
        message: "Print jobs cleared".to_string(),
    })
    .into_response()
}

/// POST /cut - Send cut command
async fn send_cut(
    headers: HeaderMap,
    State(state): State<Arc<HttpServerState>>,
    Json(payload): Json<PrintRequest>,
) -> impl IntoResponse {
    if let Err(e) = verify_origin(&headers) {
        return e.into_response();
    }

    // Verify authentication
    if let Err(e) = verify_auth(&state).await {
        return e.into_response();
    }

    let printer_name = match payload.printer {
        Some(p) if !p.is_empty() => p,
        _ => {
            match printer::list_printers() {
                Ok(printers) if !printers.is_empty() => printers[0].clone(),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(PrintResponse {
                            success: false,
                            message: "No printers available".to_string(),
                        }),
                    )
                        .into_response()
                }
            }
        }
    };

    match printer::send_cut(&printer_name) {
        Ok(()) => {
            let log_entry = create_log_entry("success", "Cut command sent".to_string());
            if let Ok(mut app) = state.app_state.try_write() {
                if app.logs.len() >= 100 {
                    app.logs.pop_front();
                }
                app.logs.push_back(log_entry);
            }

            Json(PrintResponse {
                success: true,
                message: "Cut command sent".to_string(),
            })
            .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PrintResponse {
                    success: false,
                    message: e,
                }),
            )
                .into_response()
        }
    }
}

/// Inicia el servidor HTTP en el puerto 7777
pub async fn start_http_server(app_state: Arc<RwLock<AppState>>) {
    let state = Arc::new(HttpServerState { app_state });

    // Configurar CORS permisivo (la validación se hace en cada endpoint)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/printers", get(get_printers))
        .route("/print", post(print_raw))
        .route("/printPDF", post(print_pdf))
        .route("/printPDF/upload", post(print_pdf_multipart))
        .route("/print_jobs", get(get_print_jobs))
        .route("/clear_jobs", post(clear_print_jobs))
        .route("/cut", post(send_cut))
        .layer(cors)
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:7777").await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Error binding to port 7777: {}", e);
            return;
        }
    };

    log::info!("HTTP server started at http://127.0.0.1:7777");
    println!("ISIPRINT HTTP Server running at http://127.0.0.1:7777");

    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Error en servidor HTTP: {}", e);
    }
}

// ============ Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_localhost_allowed() {
        assert!(is_origin_allowed("http://localhost"));
        assert!(is_origin_allowed("http://localhost:3000"));
        assert!(is_origin_allowed("http://127.0.0.1"));
        assert!(is_origin_allowed("http://127.0.0.1:8080"));
    }

    #[test]
    fn test_origin_isipass_allowed() {
        assert!(is_origin_allowed("https://app.isipass.net"));
        assert!(is_origin_allowed("https://sandbox.isipass.net"));
        assert!(is_origin_allowed("https://isipass.net"));
    }

    #[test]
    fn test_origin_integrate_allowed() {
        assert!(is_origin_allowed("https://pos.integrate.com.bo"));
        assert!(is_origin_allowed("https://integrate.com.bo"));
    }

    #[test]
    fn test_origin_malicious_rejected() {
        assert!(!is_origin_allowed("https://evil.com"));
        assert!(!is_origin_allowed("https://fakeisipass.net"));
        assert!(!is_origin_allowed("https://isipass.net.evil.com"));
    }

    #[test]
    fn test_origin_adeabordo_allowed() {
        assert!(is_origin_allowed("https://app.adeabordo.com"));
        assert!(is_origin_allowed("https://adeabordo.com.bo"));
    }

    #[test]
    fn test_origin_empty_allowed() {
        // Origen vacío se permite (para peticiones directas, curl, etc.)
        assert!(is_origin_allowed(""));
    }
}
