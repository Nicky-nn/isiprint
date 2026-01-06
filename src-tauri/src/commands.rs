// Comandos Tauri - equivalentes a los endpoints de Flask
use crate::graphql;
use crate::persistence;
use crate::printer;
use crate::state::{AuthState, LogEntry};
use crate::SharedAppState;
use chrono::Local;
use printpdf::{BuiltinFont, Color, Mm, PdfDocument, Pt, Rgb};
use printpdf::svg::{Svg, SvgTransform};
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use tauri::State;
use tempfile::NamedTempFile;

const ISIPRINT_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1021 793">
    <g transform="translate(0,793) scale(0.1,-0.1)" fill="#000000">
    <path d="M5080 7233 c-81 -8 -312 -54 -395 -78 -436 -125 -833 -377 -1128
-714 -334 -383 -532 -843 -579 -1347 -12 -134 -5 -507 37 -2024 35 -1232 35
-1276 20 -1355 -55 -289 -322 -494 -615 -472 -142 11 -266 70 -370 176 -111
113 -164 249 -163 416 2 261 189 490 451 551 94 22 225 15 321 -19 l73 -26 -7
327 c-4 180 -8 328 -10 330 -9 10 -150 24 -255 24 -606 2 -1114 -449 -1190
-1056 -44 -349 73 -707 319 -968 161 -172 358 -288 595 -350 114 -29 367 -37
488 -14 446 84 809 402 942 824 58 185 59 210 47 947 -6 369 -14 705 -16 748
-3 42 -2 77 3 77 4 0 21 -14 38 -31 16 -17 76 -70 134 -118 285 -238 598 -395
965 -485 131 -32 593 -159 720 -198 147 -45 209 -80 292 -163 114 -116 167
-249 166 -415 -3 -300 -241 -551 -538 -567 -118 -7 -192 8 -294 58 -155 76
-279 230 -311 386 -17 78 -8 252 16 323 10 30 40 87 66 125 25 39 44 73 40 76
-6 6 -608 159 -624 159 -3 0 -22 -38 -42 -85 -185 -430 -94 -940 229 -1282
238 -254 536 -385 875 -386 202 -1 377 41 563 135 150 75 333 237 434 384 145
212 208 420 208 689 -1 258 -65 469 -208 679 -106 156 -310 328 -488 410 -67
31 -346 116 -794 242 -93 26 -171 51 -173 55 -2 4 -49 22 -105 39 -552 172
-988 624 -1136 1180 -113 424 -68 849 131 1230 227 436 611 742 1088 869 257
68 562 71 815 9 327 -80 584 -225 825 -467 241 -242 401 -550 468 -896 13 -69
16 -303 22 -1705 7 -1808 1 -1656 76 -1872 118 -340 359 -616 674 -773 107
-54 198 -83 328 -105 257 -45 504 -9 737 105 268 132 472 352 587 633 62 152
82 259 82 442 0 210 -30 348 -111 519 -237 496 -775 770 -1318 669 l-65 -11 0
-161 c0 -89 3 -238 7 -333 l6 -172 36 18 c70 36 142 51 246 51 172 0 285 -47
405 -169 122 -123 170 -238 170 -406 0 -163 -54 -296 -165 -408 -112 -113
-249 -168 -416 -169 -251 0 -468 158 -545 396 l-26 81 -19 830 c-10 457 -23
1168 -28 1580 -6 413 -16 806 -22 873 -47 553 -285 1061 -684 1457 -371 369
-846 603 -1358 670 -108 14 -445 19 -547 8z" />
    <path d="M5787 5400 c-104 -27 -179 -90 -226 -190 -20 -44 -25 -70 -25 -132 1
-134 69 -238 194 -297 46 -22 68 -26 135 -25 119 2 207 50 269 147 82 130 71
277 -30 395 -68 80 -215 127 -317 102z" />
    <path d="M4731 5379 c-86 -17 -181 -92 -223 -178 -18 -38 -23 -63 -23 -141 0
-111 19 -162 88 -231 66 -66 135 -94 232 -94 100 1 163 26 228 93 75 77 92
121 92 232 0 110 -17 154 -88 228 -50 52 -93 76 -165 91 -60 12 -78 12 -141 0z" />
    </g>
</svg>"##;

fn mm_to_pt(mm: f64) -> Pt {
    Pt((mm as f32) * 72.0 / 25.4)
}

/// Dominios permitidos para las solicitudes
const DOMINIOS_PERMITIDOS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "*.integrate.com.bo",
    "*.isipass.net",
    "*.isipay.me",
    "*.idematica.net",
    "*.quickpay.com.bo",
    "*.isipass.com.bo",
];

/// Respuesta genérica para los comandos
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    pub preset: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
}

fn clamp_mm(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

fn settings_to_media(settings: &PrintSettings) -> (String, f64, f64) {
    // Returns: (cups_media, width_mm, height_mm)
    let preset = settings.preset.to_lowercase();

    match preset.as_str() {
        "carta" | "letter" => ("Letter".to_string(), 215.9, 279.4),
        // "Oficio" in many LATAM contexts is 216x330mm (8.5"x13")
        "oficio" => ("Custom.216x330mm".to_string(), 216.0, 330.0),
        "custom" | "personalizado" => {
            let w = clamp_mm(settings.width_mm.unwrap_or(80.0), 20.0, 500.0);
            let h = clamp_mm(settings.height_mm.unwrap_or(200.0), 20.0, 1000.0);
            (format!("Custom.{}x{}mm", w.round() as i32, h.round() as i32), w, h)
        }
        // thermal default
        _ => {
            let w = clamp_mm(settings.width_mm.unwrap_or(80.0), 20.0, 200.0);
            let h = clamp_mm(settings.height_mm.unwrap_or(200.0), 40.0, 1000.0);
            (format!("Custom.{}x{}mm", w.round() as i32, h.round() as i32), w, h)
        }
    }
}

fn generate_test_page_pdf(
    width_mm: f64,
    height_mm: f64,
    printer_name: &str,
    media: &str,
) -> Result<Vec<u8>, String> {
    let (doc, page1, layer1) = PdfDocument::new(
        "ISIPRINT Test Page",
        Mm(width_mm as f32),
        Mm(height_mm as f32),
        "Layer 1",
    );
    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Courier)
        .map_err(|e| format!("Error loading font: {}", e))?;

    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:%M:%S").to_string();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Logo SVG real (vector) - centrado arriba
    let logo_svg = Svg::parse(ISIPRINT_LOGO_SVG)
        .map_err(|e| format!("Error parsing logo SVG: {}", e))?;
    let logo_xobject = logo_svg.into_xobject(&layer);

    let logo_w_mm = if width_mm <= 90.0 { 26.0 } else { 34.0 };
    let logo_h_mm = logo_w_mm * (793.0 / 1021.0);
    let logo_margin_top_mm = 10.0;
    let logo_x_mm = ((width_mm - logo_w_mm) / 2.0).max(0.0);
    let logo_y_mm = (height_mm - logo_margin_top_mm - logo_h_mm).max(0.0);

    let base_w_pt = logo_xobject.width.into_pt(72.0).0;
    let desired_w_pt = mm_to_pt(logo_w_mm).0;
    let scale = if base_w_pt > 0.0 {
        desired_w_pt / base_w_pt
    } else {
        1.0
    };

    logo_xobject
        .clone()
        .add_to_layer(
            &layer,
            SvgTransform {
                translate_x: Some(mm_to_pt(logo_x_mm)),
                translate_y: Some(mm_to_pt(logo_y_mm)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(72.0),
                ..Default::default()
            },
        );

    // Basic ASCII banner (safe for thermal widths too)
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    let mut y = (logo_y_mm as f32) - 10.0;
    let left = 6.0_f32;

    layer.use_text("ISIPRINT", 14.0, Mm(left), Mm(y), &font);
    y -= 7.0;
    layer.use_text("====================", 12.0, Mm(left), Mm(y), &font);
    y -= 7.0;
    layer.use_text("TEST PRINT", 12.0, Mm(left), Mm(y), &font);
    y -= 10.0;

    let lines = vec![
        format!("Printer: {}", printer_name),
        format!("Paper: {} ({}x{} mm)", media, width_mm.round() as i32, height_mm.round() as i32),
        format!("OS: {} {}", os, arch),
        format!("Date: {}", date),
        format!("Time: {}", time),
        "".to_string(),
        "If you can read this, the printer is OK.".to_string(),
    ];

    for line in lines {
        layer.use_text(line, 10.0, Mm(left), Mm(y), &font);
        y -= 6.0;
        if y < 10.0 {
            break;
        }
    }

    let mut out = BufWriter::new(Vec::<u8>::new());
    doc.save(&mut out)
        .map_err(|e| format!("Error saving PDF: {}", e))?;
    let bytes = out
        .into_inner()
        .map_err(|e| format!("Error finalizing PDF buffer: {}", e))?;

    Ok(bytes)
}

/// Login with GraphQL
#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<AuthState>, String> {
    log::info!("Login attempt for: {}", email);

    match graphql::login(&email, &password).await {
        Ok(response) => {
            // Get licenses after login
            let licencias = graphql::get_licencias(&response.token).await.unwrap_or_default();

            let mut app_state = state.write().await;
            app_state.auth = AuthState {
                token: Some(response.token),
                refresh_token: Some(response.refresh_token),
                email: Some(email.clone()),
                is_logged_in: true,
            };
            app_state.licencias = licencias.clone();
            app_state.add_log("INFO", &format!("Login successful for {}", email));

            // Save session to disk for next app start
            if let Err(e) = persistence::save_session(&app_state.auth, &app_state.licencias) {
                log::warn!("Failed to save session: {}", e);
            }

            Ok(CommandResponse::success(app_state.auth.clone()))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Login error: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Obtener licencias del usuario
#[tauri::command]
pub async fn get_licencias(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<Vec<crate::state::LicenciaProducto>>, String> {
    let app_state = state.read().await;

    if !app_state.auth.is_logged_in {
        return Ok(CommandResponse::error("Not logged in"));
    }

    Ok(CommandResponse::success(app_state.licencias.clone()))
}

/// Obtener lista de impresoras - equivalente a /printers
#[tauri::command]
pub async fn get_printers(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<Vec<String>>, String> {
    let mut app_state = state.write().await;

    match printer::list_printers() {
        Ok(printers) => {
            app_state.add_log("INFO", &format!("Printers listed: {:?}", printers));
            Ok(CommandResponse::success(printers))
        }
        Err(e) => {
            app_state.add_log("ERROR", &format!("Error listing printers: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Imprimir PDF desde archivo - equivalente a /print
#[tauri::command]
pub async fn print_pdf(
    file_path: String,
    printer_name: String,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    {
        let mut app_state = state.write().await;

        // Verify license
        if !app_state.is_license_valid() {
            app_state.add_log("ERROR", "License expired or invalid");
            return Ok(CommandResponse::error("License expired or invalid"));
        }

        // Verify print limit
        if !app_state.can_print() {
            app_state.add_log("ERROR", "Print limit reached");
            return Ok(CommandResponse::error("Print limit reached"));
        }

        app_state.add_log(
            "INFO",
            &format!("Printing {} on {}", file_path, printer_name),
        );
    }

    // Imprimir archivo
    match printer::print_file(&file_path, &printer_name) {
        Ok(job_id) => {
            // Send cut command
            if let Err(e) = printer::send_cut(&printer_name) {
                let mut app_state = state.write().await;
                app_state.add_log("WARN", &format!("Error sending cut: {}", e));
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", &format!("Print started. Job ID: {}", job_id));

            Ok(CommandResponse::success("Print started".to_string()))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Print error: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Imprimir PDF desde archivo con settings (media/tamaño)
#[tauri::command]
pub async fn print_pdf_with_settings(
    file_path: String,
    printer_name: String,
    settings: PrintSettings,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    {
        let mut app_state = state.write().await;

        if !app_state.is_license_valid() {
            app_state.add_log("ERROR", "License expired or invalid");
            return Ok(CommandResponse::error("License expired or invalid"));
        }

        if !app_state.can_print() {
            app_state.add_log("ERROR", "Print limit reached");
            return Ok(CommandResponse::error("Print limit reached"));
        }

        app_state.add_log(
            "INFO",
            &format!("Printing {} on {} (with settings)", file_path, printer_name),
        );
    }

    let (media, _w, _h) = settings_to_media(&settings);

    match printer::print_file_with_media(&file_path, &printer_name, Some(&media)) {
        Ok(_job_id) => {
            if let Err(e) = printer::send_cut(&printer_name) {
                let mut app_state = state.write().await;
                app_state.add_log("WARN", &format!("Error sending cut: {}", e));
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", "Print started (with settings)");

            Ok(CommandResponse::success("Print started".to_string()))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Print error: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Imprimir PDF desde URL - equivalente a /printPDF
#[tauri::command]
pub async fn print_pdf_from_url(
    pdf_url: String,
    printer_name: String,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    {
        let mut app_state = state.write().await;

        if !app_state.is_license_valid() {
            app_state.add_log("ERROR", "License expired or invalid");
            return Ok(CommandResponse::error("License expired or invalid"));
        }

        if !app_state.can_print() {
            app_state.add_log("ERROR", "Print limit reached");
            return Ok(CommandResponse::error("Print limit reached"));
        }

        app_state.add_log("INFO", &format!("Downloading PDF from {}", pdf_url));
    }

    // Download PDF
    let pdf_data = match printer::download_pdf(&pdf_url).await {
        Ok(data) => data,
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error downloading PDF: {}", e));
            return Ok(CommandResponse::error(&e));
        }
    };

    // Save to temporary file
    let mut temp_file = NamedTempFile::with_suffix(".pdf")
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Print
    match printer::print_file(&temp_path, &printer_name) {
        Ok(job_id) => {
            if let Err(e) = printer::send_cut(&printer_name) {
                let mut app_state = state.write().await;
                app_state.add_log("WARN", &format!("Error sending cut: {}", e));
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log(
                "INFO",
                &format!("PDF printed from URL. Job ID: {}", job_id),
            );

            Ok(CommandResponse::success(
                "PDF print started".to_string(),
            ))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error printing PDF: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Imprimir PDF desde URL con settings (media/tamaño)
#[tauri::command]
pub async fn print_pdf_from_url_with_settings(
    pdf_url: String,
    printer_name: String,
    settings: PrintSettings,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    {
        let mut app_state = state.write().await;

        if !app_state.is_license_valid() {
            app_state.add_log("ERROR", "License expired or invalid");
            return Ok(CommandResponse::error("License expired or invalid"));
        }

        if !app_state.can_print() {
            app_state.add_log("ERROR", "Print limit reached");
            return Ok(CommandResponse::error("Print limit reached"));
        }

        app_state.add_log("INFO", &format!("Downloading PDF from {}", pdf_url));
    }

    let pdf_data = match printer::download_pdf(&pdf_url).await {
        Ok(data) => data,
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error downloading PDF: {}", e));
            return Ok(CommandResponse::error(&e));
        }
    };

    let mut temp_file = NamedTempFile::with_suffix(".pdf")
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();
    let (media, _w, _h) = settings_to_media(&settings);

    match printer::print_file_with_media(&temp_path, &printer_name, Some(&media)) {
        Ok(_job_id) => {
            if let Err(e) = printer::send_cut(&printer_name) {
                let mut app_state = state.write().await;
                app_state.add_log("WARN", &format!("Error sending cut: {}", e));
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", "PDF printed from URL (with settings)");

            Ok(CommandResponse::success("PDF print started".to_string()))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error printing PDF: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Imprimir una página de prueba (PDF generado) con settings
#[tauri::command]
pub async fn print_test_page(
    printer_name: String,
    settings: PrintSettings,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    {
        let mut app_state = state.write().await;

        if !app_state.is_license_valid() {
            app_state.add_log("ERROR", "License expired or invalid");
            return Ok(CommandResponse::error("License expired or invalid"));
        }

        if !app_state.can_print() {
            app_state.add_log("ERROR", "Print limit reached");
            return Ok(CommandResponse::error("Print limit reached"));
        }

        app_state.add_log("INFO", &format!("Test print on {}", printer_name));
    }

    let (media, width_mm, height_mm) = settings_to_media(&settings);
    let pdf_data = generate_test_page_pdf(width_mm, height_mm, &printer_name, &media)?;

    let mut temp_file = NamedTempFile::with_suffix(".pdf")
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    let pdf_path = temp_file.path().to_string_lossy().to_string();

    match printer::print_file_with_media(&pdf_path, &printer_name, Some(&media)) {
        Ok(_job_id) => {
            if let Err(e) = printer::send_cut(&printer_name) {
                let mut app_state = state.write().await;
                app_state.add_log("WARN", &format!("Error sending cut: {}", e));
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", "Test page printed");

            Ok(CommandResponse::success("Test page print started".to_string()))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Test print error: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Obtener trabajos de impresión - equivalente a /print_jobs
#[tauri::command]
pub async fn get_print_jobs(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<Vec<printer::PrintJob>>, String> {
    match printer::get_jobs() {
        Ok(jobs) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", &format!("Jobs in queue: {}", jobs.len()));
            Ok(CommandResponse::success(jobs))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error getting jobs: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Limpiar cola de impresión - equivalente a /clear_jobs
#[tauri::command]
pub async fn clear_print_jobs(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    match printer::clear_jobs() {
        Ok(()) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", "Print queue cleared");
            Ok(CommandResponse::success(
                "All print jobs in queue have been cancelled".to_string(),
            ))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error clearing queue: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Enviar comando de corte
#[tauri::command]
pub async fn send_cut_command(
    printer_name: String,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    match printer::send_cut(&printer_name) {
        Ok(()) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", &format!("Cut command sent to {}", printer_name));
            Ok(CommandResponse::success(
                "Cut command sent".to_string(),
            ))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error sending cut: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Obtener logs del sistema
#[tauri::command]
pub async fn get_logs(state: State<'_, SharedAppState>) -> Result<Vec<LogEntry>, String> {
    let app_state = state.read().await;
    Ok(app_state.logs.iter().cloned().collect())
}

/// Validar dominio - equivalente a verificar_dominio en Python
#[tauri::command]
pub fn validate_domain(origin: String) -> Result<bool, String> {
    if let Ok(parsed) = url::Url::parse(&origin) {
        if let Some(host) = parsed.host_str() {
            // Permitir localhost
            if host == "localhost" || host == "127.0.0.1" {
                return Ok(true);
            }

            // Verificar dominios permitidos con fnmatch
            for pattern in DOMINIOS_PERMITIDOS {
                if fnmatch_regex::glob_to_regex(pattern)
                    .map(|re| re.is_match(host))
                    .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Get authentication state
#[tauri::command]
pub async fn get_auth_state(state: State<'_, SharedAppState>) -> Result<AuthState, String> {
    let app_state = state.read().await;
    Ok(app_state.auth.clone())
}

/// Verify if saved session is still valid by checking with the API
/// Returns the auth state if valid, or clears and returns default if expired
#[tauri::command]
pub async fn verify_session(state: State<'_, SharedAppState>) -> Result<CommandResponse<AuthState>, String> {
    let app_state = state.read().await;
    
    // If not logged in, nothing to verify
    if !app_state.auth.is_logged_in {
        return Ok(CommandResponse::success(AuthState::default()));
    }
    
    let token = match &app_state.auth.token {
        Some(t) => t.clone(),
        None => {
            drop(app_state);
            // No token, clear state
            let mut app_state = state.write().await;
            app_state.auth = AuthState::default();
            app_state.licencias.clear();
            let _ = persistence::clear_session();
            return Ok(CommandResponse::success(AuthState::default()));
        }
    };
    
    drop(app_state);
    
    // Try to get licenses with the saved token - if it works, token is valid
    match graphql::get_licencias(&token).await {
        Ok(licencias) => {
            // Token is valid, update licenses (they might have changed)
            let mut app_state = state.write().await;
            app_state.licencias = licencias.clone();
            app_state.add_log("INFO", "Session restored from saved data");
            
            // Update saved session with fresh license data
            if let Err(e) = persistence::save_session(&app_state.auth, &app_state.licencias) {
                log::warn!("Failed to update saved session: {}", e);
            }
            
            Ok(CommandResponse::success(app_state.auth.clone()))
        }
        Err(e) => {
            // Token expired or invalid, clear session
            log::info!("Saved token is invalid/expired: {}", e);
            let mut app_state = state.write().await;
            app_state.auth = AuthState::default();
            app_state.licencias.clear();
            app_state.add_log("INFO", "Session expired, please log in again");
            
            // Clear saved session
            let _ = persistence::clear_session();
            
            Ok(CommandResponse::error("Session expired, please log in again"))
        }
    }
}

/// Close session
#[tauri::command]
pub async fn logout(state: State<'_, SharedAppState>) -> Result<CommandResponse<String>, String> {
    let mut app_state = state.write().await;
    app_state.auth = AuthState::default();
    app_state.licencias.clear();
    app_state.add_log("INFO", "Session closed");
    
    // Clear saved session from disk
    if let Err(e) = persistence::clear_session() {
        log::warn!("Failed to clear saved session: {}", e);
    }
    
    Ok(CommandResponse::success("Session closed".to_string()))
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Validación de dominios permitidos
    #[test]
    fn test_validate_domain_localhost() {
        // localhost debe ser permitido
        let result = validate_domain_internal("http://localhost:1420");
        assert!(result, "localhost debería estar permitido");

        let result = validate_domain_internal("http://127.0.0.1:7777");
        assert!(result, "127.0.0.1 debería estar permitido");
    }

    #[test]
    fn test_validate_domain_allowed_patterns() {
        // Dominios permitidos con wildcard
        let result = validate_domain_internal("https://app.integrate.com.bo");
        assert!(result, "*.integrate.com.bo debería estar permitido");

        let result = validate_domain_internal("https://sandbox.isipass.net");
        assert!(result, "*.isipass.net debería estar permitido");

        let result = validate_domain_internal("https://api.isipay.me");
        assert!(result, "*.isipay.me debería estar permitido");
    }

    #[test]
    fn test_validate_domain_denied() {
        // Dominios NO permitidos
        let result = validate_domain_internal("https://malicious-site.com");
        assert!(!result, "malicious-site.com NO debería estar permitido");

        let result = validate_domain_internal("https://google.com");
        assert!(!result, "google.com NO debería estar permitido");

        let result = validate_domain_internal("https://fake-isipass.net.evil.com");
        assert!(!result, "fake-isipass.net.evil.com NO debería estar permitido");
    }

    #[test]
    fn test_validate_domain_edge_cases() {
        // Casos límite
        let result = validate_domain_internal("");
        assert!(!result, "URL vacía NO debería estar permitida");

        let result = validate_domain_internal("not-a-valid-url");
        assert!(!result, "URL inválida NO debería estar permitida");
    }

    /// Test: CommandResponse
    #[test]
    fn test_command_response_success() {
        let response: CommandResponse<String> = CommandResponse::success("test".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_command_response_error() {
        let response: CommandResponse<String> = CommandResponse::error("error message");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("error message".to_string()));
    }

    /// Función interna para tests (sin State de Tauri)
    fn validate_domain_internal(origin: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(origin) {
            if let Some(host) = parsed.host_str() {
                // Permitir localhost
                if host == "localhost" || host == "127.0.0.1" {
                    return true;
                }

                // Verificar dominios permitidos con fnmatch
                for pattern in DOMINIOS_PERMITIDOS {
                    // Coincidencia exacta primero
                    if *pattern == host {
                        return true;
                    }
                    // Luego wildcard
                    if fnmatch_regex::glob_to_regex(pattern)
                        .map(|re| re.is_match(host))
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}
