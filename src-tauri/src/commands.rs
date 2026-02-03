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
use std::time::SystemTime;
use tauri::State;
use tempfile::Builder;
use tokio::time::{sleep, Duration};

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

fn is_pdf_printer(printer_name: &str) -> bool {
    let n = printer_name.to_lowercase();
    n.contains("pdf")
}

fn is_pdfwriter(printer_name: &str) -> bool {
    printer_name.to_lowercase().contains("pdfwriter")
}

fn try_find_latest_pdfwriter_output() -> Option<String> {
    // RWTS PDFwriter typically writes into: /private/var/spool/pdfwriter/<user>/
    // We best-effort pick the most recently modified .pdf.
    let user = std::env::var("USER").ok()?;
    let dir = std::path::Path::new("/private/var/spool/pdfwriter").join(user);
    let entries = std::fs::read_dir(&dir).ok()?;

    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("pdf")) != Some(true) {
            continue;
        }
        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(t) => t,
            Err(_) => continue,
        };

        match &newest {
            None => newest = Some((modified, path)),
            Some((best_time, _)) if modified > *best_time => newest = Some((modified, path)),
            _ => {}
        }
    }

    newest.map(|(_, p)| p.to_string_lossy().to_string())
}

fn try_find_latest_pdfwriter_output_since(since: SystemTime) -> Option<String> {
    let user = std::env::var("USER").ok()?;
    let dir = std::path::Path::new("/private/var/spool/pdfwriter").join(user);
    let entries = std::fs::read_dir(&dir).ok()?;

    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pdf"))
            != Some(true)
        {
            continue;
        }

        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Must be newer than (or equal to) our start time.
        if modified < since {
            continue;
        }

        match &newest {
            None => newest = Some((modified, path)),
            Some((best_time, _)) if modified > *best_time => newest = Some((modified, path)),
            _ => {}
        }
    }

    newest.map(|(_, p)| p.to_string_lossy().to_string())
}

fn cups_job_key(printer_name: &str, job_id: i32) -> Option<String> {
    if job_id <= 0 {
        return None;
    }
    Some(format!("{}-{}", printer_name, job_id))
}

fn cups_job_seen_in_queue(printer_name: &str, job_id: i32) -> Result<bool, String> {
    use std::process::Command;

    let key = match cups_job_key(printer_name, job_id) {
        Some(k) => k,
        None => return Ok(true), // can't verify, don't block
    };

    let output = Command::new("lpstat")
        .args(["-o", printer_name])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}", e))?;

    // If lpstat returns non-zero for an empty queue, treat as not seen.
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&key))
}

fn cups_job_seen_in_completed(printer_name: &str, job_id: i32) -> Result<bool, String> {
    use std::process::Command;

    let key = match cups_job_key(printer_name, job_id) {
        Some(k) => k,
        None => return Ok(true), // can't verify, don't block
    };

    let output = Command::new("lpstat")
        .args(["-W", "completed"])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&key))
}

async fn verify_cups_job_visible(
    printer_name: &str,
    job_id: i32,
    timeout: Duration,
) -> Result<(), String> {
    // Best-effort: only validates that CUPS has *record* of the job (queued or completed).
    // This avoids returning a false "success" when CUPS silently drops a job.
    let key = match cups_job_key(printer_name, job_id) {
        Some(k) => k,
        None => return Ok(()),
    };

    let step = Duration::from_millis(250);
    let mut waited = Duration::from_millis(0);

    while waited < timeout {
        let in_queue = match cups_job_seen_in_queue(printer_name, job_id) {
            Ok(v) => v,
            Err(_e) => {
                return Ok(());
            }
        };

        let in_completed = match cups_job_seen_in_completed(printer_name, job_id) {
            Ok(v) => v,
            Err(_e) => {
                return Ok(());
            }
        };

        if in_queue || in_completed {
            return Ok(());
        }

        sleep(step).await;
        waited += step;
    }

    Err(format!(
        "Print job {} was submitted but not observed in CUPS queue/completed within {:?}",
        key, timeout
    ))
}

fn normalize_language(language: Option<String>) -> String {
    let raw = language.unwrap_or_else(|| "es".to_string());
    let lower = raw.trim().to_lowercase();
    if lower.starts_with("en") {
        "en".to_string()
    } else if lower.starts_with("fr") {
        "fr".to_string()
    } else {
        "es".to_string()
    }
}

struct TestPageText {
    header_title: String,
    header_status: String,
    label_app: String,
    label_install_status: String,
    install_ok: String,
    label_verification: String,
    verification_ok: String,
    paragraph: String,
    section_details: String,
    label_print_type: String,
    label_printer: String,
    label_profile: String,
    label_paper: String,
    label_size: String,
    label_date: String,
    label_time: String,
    label_os: String,
    print_type_roll: String,
    print_type_sheet: String,
}

fn test_page_text(lang: &str) -> TestPageText {
    match lang {
        "en" => TestPageText {
            header_title: "Print Test - ISIPRINT".to_string(),
            header_status: "INSTALLATION OK".to_string(),
            label_app: "Application".to_string(),
            label_install_status: "Installation Status".to_string(),
            install_ok: "Completed successfully".to_string(),
            label_verification: "System Verification".to_string(),
            verification_ok: "Successful".to_string(),
            paragraph: "An automatic test print will now be performed to confirm that the system works correctly according to the selected configuration.".to_string(),
            section_details: "Test Details".to_string(),
            label_print_type: "Print Type".to_string(),
            label_printer: "Printer".to_string(),
            label_profile: "Profile".to_string(),
            label_paper: "Paper".to_string(),
            label_size: "Size".to_string(),
            label_date: "Date".to_string(),
            label_time: "Time".to_string(),
            label_os: "OS".to_string(),
            print_type_roll: "Roll format".to_string(),
            print_type_sheet: "Sheet format".to_string(),
        },
        "fr" => TestPageText {
            header_title: "Test d'impression - ISIPRINT".to_string(),
            header_status: "INSTALLATION OK".to_string(),
            label_app: "Application".to_string(),
            label_install_status: "\u{00C9}tat de l'installation".to_string(),
            install_ok: "Termin\u{00E9}e avec succ\u{00E8}s".to_string(),
            label_verification: "V\u{00E9}rification du syst\u{00E8}me".to_string(),
            verification_ok: "R\u{00E9}ussie".to_string(),
            paragraph: "Une impression de test automatique va \u{00EA}tre effectu\u{00E9}e afin de confirmer que le syst\u{00E8}me fonctionne correctement selon la configuration s\u{00E9}lectionn\u{00E9}e.".to_string(),
            section_details: "D\u{00E9}tails du test".to_string(),
            label_print_type: "Type d'impression".to_string(),
            label_printer: "Imprimante".to_string(),
            label_profile: "Profil".to_string(),
            label_paper: "Papier".to_string(),
            label_size: "Taille".to_string(),
            label_date: "Date".to_string(),
            label_time: "Heure".to_string(),
            label_os: "OS".to_string(),
            print_type_roll: "Format rouleau".to_string(),
            print_type_sheet: "Format feuille".to_string(),
        },
        _ => TestPageText {
            header_title: "Prueba de Impresi\u{00F3}n - ISIPRINT".to_string(),
            header_status: "INSTALACION CORRECTA".to_string(),
            label_app: "Aplicaci\u{00F3}n".to_string(),
            label_install_status: "Estado de Instalaci\u{00F3}n".to_string(),
            install_ok: "Completada con \u{00E9}xito".to_string(),
            label_verification: "Verificaci\u{00F3}n de Funcionamiento".to_string(),
            verification_ok: "Exitosa".to_string(),
            paragraph: "A continuaci\u{00F3}n se realizar\u{00E1} una prueba de impresi\u{00F3}n autom\u{00E1}tica para confirmar que el sistema funciona correctamente seg\u{00FA}n la configuraci\u{00F3}n seleccionada.".to_string(),
            section_details: "Detalles de la Prueba".to_string(),
            label_print_type: "Tipo de Impresi\u{00F3}n".to_string(),
            label_printer: "Impresora".to_string(),
            label_profile: "Perfil".to_string(),
            label_paper: "Papel".to_string(),
            label_size: "Tama\u{00F1}o".to_string(),
            label_date: "Fecha".to_string(),
            label_time: "Hora".to_string(),
            label_os: "SO".to_string(),
            print_type_roll: "Formato Rollo".to_string(),
            print_type_sheet: "Formato Hoja".to_string(),
        },
    }
}

fn wrap_text_to_width(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let sep = if line.is_empty() { "" } else { " " };
        if line.len() + sep.len() + word.len() > width {
            if !line.is_empty() {
                out.push(line);
                line = String::new();
            }
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn center_line(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }
    let pad = (width - text.len()) / 2;
    format!("{}{}", " ".repeat(pad), text)
}

async fn verify_pdfwriter_output_visible(since: SystemTime, timeout: Duration) -> Result<String, String> {
    let step = Duration::from_millis(250);
    let mut waited = Duration::from_millis(0);

    while waited < timeout {
        if let Some(out) = try_find_latest_pdfwriter_output_since(since)
            .or_else(try_find_latest_pdfwriter_output)
        {
            return Ok(out);
        }
        sleep(step).await;
        waited += step;
    }

    Err(format!(
        "PDFwriter accepted the job, but no output PDF was detected in spool within {:?}",
        timeout
    ))
}

fn generate_test_page_pdf(
    width_mm: f64,
    height_mm: f64,
    printer_name: &str,
    media: &str,
    preset: &str,
    language: &str,
) -> Result<Vec<u8>, String> {
    let (doc, page1, layer1) = PdfDocument::new(
        "ISIPRINT Test Page",
        Mm(width_mm as f32),
        Mm(height_mm as f32),
        "Layer 1",
    );
    let layer = doc.get_page(page1).get_layer(layer1);
    let font_mono = doc
        .add_builtin_font(BuiltinFont::Courier)
        .map_err(|e| format!("Error loading font: {}", e))?;

    let now = Local::now();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let lang = if language.is_empty() { "es" } else { language };
    let txt = test_page_text(lang);

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

    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));

    // Receipt-style layout (monospace)
    let cols: usize = if width_mm <= 90.0 { 42 } else { 88 };
    let divider = "=".repeat(cols);

    let print_type = if preset.to_lowercase() == "thermal" {
        txt.print_type_roll.clone()
    } else {
        txt.print_type_sheet.clone()
    };

    let size_str = format!("{}x{} mm", width_mm.round() as i32, height_mm.round() as i32);
    let os_str = format!("{} {}", os, arch);
    let date_str = match lang {
        "en" => now.format("%Y-%m-%d").to_string(),
        "fr" => now.format("%d/%m/%Y").to_string(),
        _ => now.format("%d/%m/%Y").to_string(),
    };
    let time_str = match lang {
        "en" => now.format("%H:%M:%S").to_string(),
        "fr" => now.format("%H:%M:%S").to_string(),
        _ => now.format("%H:%M:%S").to_string(),
    };

    let mut lines: Vec<String> = Vec::new();

    // Header (centered)
    lines.push(divider.clone());
    lines.push(center_line(&txt.header_title, cols));
    lines.push(center_line(&txt.header_status, cols));
    lines.push(divider.clone());
    lines.push(String::new());

    // Installation summary
    lines.push(format!("{}: ISIPRINT", txt.label_app));
    lines.push(format!("{}: {}", txt.label_install_status, txt.install_ok));
    lines.push(format!("{}: {}", txt.label_verification, txt.verification_ok));
    lines.push(String::new());

    // Paragraph (wrapped)
    for l in wrap_text_to_width(&txt.paragraph, cols) {
        lines.push(l);
    }
    lines.push(String::new());

    // Details section
    lines.push(divider.clone());
    lines.push(center_line(&txt.section_details, cols));
    lines.push(divider.clone());
    lines.push(String::new());

    // Details list (matches sample style)
    let mut detail = Vec::new();
    detail.push(format!("-{}: {}", txt.label_print_type, print_type));
    detail.push(format!("-{}: {}", txt.label_printer, printer_name));
    detail.push(format!("-{}: {}", txt.label_profile, preset));
    detail.push(format!("-{}: {}", txt.label_paper, media));
    detail.push(format!("-{}: {}", txt.label_size, size_str));
    detail.push(format!("-{}: {}", txt.label_date, date_str));
    detail.push(format!("-{}: {}", txt.label_time, time_str));
    detail.push(format!("-{}: {}", txt.label_os, os_str));

    for l in detail {
        for wrapped in wrap_text_to_width(&l, cols) {
            lines.push(wrapped);
        }
    }

    lines.push(String::new());
    lines.push(divider);

    // Render lines below logo
    let left = 6.0_f32;
    let mut y = (logo_y_mm as f32) - 8.0;
    let font_size = if width_mm <= 90.0 { 9.8 } else { 10.5 };
    let line_h = if width_mm <= 90.0 { 5.5 } else { 6.0 };

    for line in lines {
        layer.use_text(line, font_size, Mm(left), Mm(y), &font_mono);
        y -= line_h;
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
    let since = SystemTime::now();

    match printer::print_file(&file_path, &printer_name) {
        Ok(job_id) => {
            if is_pdfwriter(&printer_name) {
                match verify_pdfwriter_output_visible(since, Duration::from_secs(8)).await {
                    Ok(out) => {
                        let mut app_state = state.write().await;
                        app_state.add_log("INFO", &format!("PDFwriter output detected: {}", out));
                    }
                    Err(e) => {
                        let mut app_state = state.write().await;
                        app_state.add_log("ERROR", &format!("PDFwriter verification failed: {}", e));
                        return Ok(CommandResponse::error(&e));
                    }
                }
            } else if let Err(e) =
                verify_cups_job_visible(&printer_name, job_id, Duration::from_secs(3)).await
            {
                let mut app_state = state.write().await;
                app_state.add_log("ERROR", &format!("CUPS verification failed: {}", e));
                return Ok(CommandResponse::error(&e));
            }

            // Send cut command (only for thermal printers, never for PDF virtual printers)
            if !is_pdf_printer(&printer_name) {
                if let Err(e) = printer::send_cut(&printer_name) {
                    let mut app_state = state.write().await;
                    app_state.add_log("WARN", &format!("Error sending cut: {}", e));
                }
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

    let print_result = if is_pdf_printer(&printer_name) {
        // PDF virtual printers often ignore/reject custom media sizes.
        printer::print_file(&file_path, &printer_name)
    } else {
        printer::print_file_with_media(&file_path, &printer_name, Some(&media))
    };

    let since = SystemTime::now();

    match print_result {
        Ok(job_id) => {
            if is_pdfwriter(&printer_name) {
                if let Err(e) = verify_pdfwriter_output_visible(since, Duration::from_secs(8)).await {
                    let mut app_state = state.write().await;
                    app_state.add_log("ERROR", &format!("PDFwriter verification failed: {}", e));
                    return Ok(CommandResponse::error(&e));
                }
            } else if let Err(e) =
                verify_cups_job_visible(&printer_name, job_id, Duration::from_secs(3)).await
            {
                let mut app_state = state.write().await;
                app_state.add_log("ERROR", &format!("CUPS verification failed: {}", e));
                return Ok(CommandResponse::error(&e));
            }

            // Cut is only meaningful for thermal printers.
            if settings.preset.to_lowercase() == "thermal" && !is_pdf_printer(&printer_name) {
                if let Err(e) = printer::send_cut(&printer_name) {
                    let mut app_state = state.write().await;
                    app_state.add_log("WARN", &format!("Error sending cut: {}", e));
                }
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", &format!("Print started (with settings). Job ID: {}", job_id));

            Ok(CommandResponse::success(format!(
                "Print started. Job ID: {}",
                job_id
            )))
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

        app_state.add_log("INFO", "Downloading PDF");
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

    // Save to temporary file (use a recognizable name for PDF virtual printers)
    let mut temp_file = Builder::new()
        .prefix("isiprint_url_pdf_")
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    let temp_path = temp_file
        .into_temp_path()
        .keep()
        .map_err(|e| format!("Error persisting temp file: {}", e))?;
    
    let temp_path_str = temp_path.to_string_lossy().to_string();

    // Print
    let since = SystemTime::now();

    match printer::print_file(&temp_path_str, &printer_name) {
        Ok(job_id) => {
            if is_pdfwriter(&printer_name) {
                if let Err(e) = verify_pdfwriter_output_visible(since, Duration::from_secs(8)).await {
                    let mut app_state = state.write().await;
                    app_state.add_log("ERROR", &format!("PDFwriter verification failed: {}", e));
                    return Ok(CommandResponse::error(&e));
                }
            } else if let Err(e) =
                verify_cups_job_visible(&printer_name, job_id, Duration::from_secs(3)).await
            {
                let mut app_state = state.write().await;
                app_state.add_log("ERROR", &format!("CUPS verification failed: {}", e));
                return Ok(CommandResponse::error(&e));
            }

            // Don't delete the temp file - let the system clean it up later
            // Virtual printers like PDFwriter need time to process the file
            
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
            let _ = std::fs::remove_file(&temp_path);
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

        app_state.add_log("INFO", "Downloading PDF");
    }

    let pdf_data = match printer::download_pdf(&pdf_url).await {
        Ok(data) => data,
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error downloading PDF: {}", e));
            return Ok(CommandResponse::error(&e));
        }
    };

    let mut temp_file = Builder::new()
        .prefix("isiprint_url_pdf_")
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    let temp_path = temp_file
        .into_temp_path()
        .keep()
        .map_err(|e| format!("Error persisting temp file: {}", e))?;
    
    let temp_path_str = temp_path.to_string_lossy().to_string();
    let (media, _w, _h) = settings_to_media(&settings);

    let print_result = if is_pdf_printer(&printer_name) {
        printer::print_file(&temp_path_str, &printer_name)
    } else {
        printer::print_file_with_media(&temp_path_str, &printer_name, Some(&media))
    };

    let since = SystemTime::now();

    match print_result {
        Ok(job_id) => {
            if is_pdfwriter(&printer_name) {
                if let Err(e) = verify_pdfwriter_output_visible(since, Duration::from_secs(8)).await {
                    let mut app_state = state.write().await;
                    app_state.add_log("ERROR", &format!("PDFwriter verification failed: {}", e));
                    return Ok(CommandResponse::error(&e));
                }
            } else if let Err(e) =
                verify_cups_job_visible(&printer_name, job_id, Duration::from_secs(3)).await
            {
                let mut app_state = state.write().await;
                app_state.add_log("ERROR", &format!("CUPS verification failed: {}", e));
                return Ok(CommandResponse::error(&e));
            }

            // Don't delete the temp file - let the system clean it up later.
            // Virtual printers like PDFwriter need time to process the file.

            if settings.preset.to_lowercase() == "thermal" && !is_pdf_printer(&printer_name) {
                if let Err(e) = printer::send_cut(&printer_name) {
                    let mut app_state = state.write().await;
                    app_state.add_log("WARN", &format!("Error sending cut: {}", e));
                }
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log(
                "INFO",
                &format!("PDF printed from URL (with settings). Job ID: {}", job_id),
            );

            Ok(CommandResponse::success(format!(
                "PDF print started. Job ID: {}",
                job_id
            )))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
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
    language: Option<String>,
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

    // Detectar si es una impresora de red creada por nosotros (Network_Printer_IP_PORT)
    if printer_name.starts_with("Network_Printer_") {
        // Formato: Network_Printer_192_168_1_100_9100
        let parts: Vec<&str> = printer_name.split('_').collect();
        if parts.len() >= 4 {
            // Reconstruir IP (partes 2 a N-1)
            let port_str = parts.last().unwrap();
            
            // La IP son todas las partes intermedias unidas por puntos
            // Network (0), Printer (1), 192 (2), 168 (3), 1 (4), 100 (5), 9100 (6)
            let ip_parts = &parts[2..parts.len()-1];
            let ip = ip_parts.join(".");
            
            if let Ok(port) = port_str.parse::<u16>() {
                // Usar RawPrinter para enviar ESC/POS directo
                let raw_printer = crate::raw_printer::RawPrinter::new(&ip, port);
                
                // Intentar imprimir ticket de prueba ESC/POS
                match raw_printer.print_test_receipt() {
                    Ok(_) => {
                        let mut app_state = state.write().await;
                        app_state.print_count += 1;
                        app_state.add_log("INFO", &format!("RAW Test Print sent to {}:{}", ip, port));
                        return Ok(CommandResponse::success("RAW Test Print sent successfully".to_string()));
                    }
                    Err(e) => {
                         let mut app_state = state.write().await;
                         app_state.add_log("WARN", &format!("RAW Print failed, falling back to CUPS: {}", e));
                         // Fallback a CUPS si falla el RAW directo
                    }
                }
            }
        }
    }

    let (media, width_mm, height_mm) = settings_to_media(&settings);

    let lang = normalize_language(language);
    let pdf_data = generate_test_page_pdf(
        width_mm,
        height_mm,
        &printer_name,
        &media,
        &settings.preset,
        &lang,
    )?;

    let mut temp_file = Builder::new()
        .prefix("isiprint_test_page_")
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| format!("Error creating temp file: {}", e))?;

    temp_file
        .write_all(&pdf_data)
        .map_err(|e| format!("Error writing temp file: {}", e))?;

    // Persist the temp file so it's not deleted when temp_file goes out of scope
    // This is critical for virtual printers (like PDF printers) that need time to read the file
    let pdf_path = temp_file
        .into_temp_path()
        .keep()
        .map_err(|e| format!("Error persisting temp file: {}", e))?;
    
    let pdf_path_str = pdf_path.to_string_lossy().to_string();
    
    // No debug logs here (avoid leaking temp paths)

    // Capture a reference time so we can detect new PDFwriter outputs.
    let pdfwriter_since = SystemTime::now();

    let print_result = if is_pdf_printer(&printer_name) {
        // PDF virtual printers often ignore/reject custom media sizes.
        printer::print_file(&pdf_path_str, &printer_name)
    } else {
        printer::print_file_with_media(&pdf_path_str, &printer_name, Some(&media))
    };

    match print_result {
        Ok(job_id) => {
            // No debug logs here

            // Generic verification for non-PDFwriter printers (avoid false-positive success)
            if !is_pdfwriter(&printer_name) {
                if let Err(e) =
                    verify_cups_job_visible(&printer_name, job_id, Duration::from_secs(3)).await
                {
                    let mut app_state = state.write().await;
                    app_state.add_log("ERROR", &format!("CUPS verification failed: {}", e));
                    return Ok(CommandResponse::error(&e));
                }
            }
            
            // Don't delete the temp file - let the system clean it up later
            // Virtual printers like PDFwriter need time to process the file
            // The /tmp directory is cleaned automatically by the OS
            
            if settings.preset.to_lowercase() == "thermal" && !is_pdf_printer(&printer_name) {
                if let Err(e) = printer::send_cut(&printer_name) {
                    let mut app_state = state.write().await;
                    app_state.add_log("WARN", &format!("Error sending cut: {}", e));
                }
            }

            let mut app_state = state.write().await;
            app_state.print_count += 1;
            app_state.add_log("INFO", &format!("Test page printed successfully. Job ID: {}", job_id));

            // For PDFwriter, don't report success unless we can actually observe an output PDF
            // appear in the expected spool folder. Otherwise, return an error (no false positives).
            if is_pdfwriter(&printer_name) {
                match verify_pdfwriter_output_visible(pdfwriter_since, Duration::from_secs(8)).await {
                    Ok(out) => {
                        let _ = out; // verified, but don't leak path in response
                        return Ok(CommandResponse::success(format!(
                            "Test page printed. Job ID: {}",
                            job_id
                        )));
                    }
                    Err(e) => {
                        let mut app_state = state.write().await;
                        app_state.add_log(
                            "ERROR",
                            &format!(
                                "PDFwriter job {} submitted but no output PDF detected",
                                job_id
                            ),
                        );
                        return Ok(CommandResponse::error(&e));
                    }
                }
            }

            Ok(CommandResponse::success(format!(
                "Test page print started. Job ID: {}",
                job_id
            )))
        }
        Err(e) => {
            // Clean up on error
            let _ = std::fs::remove_file(&pdf_path);
            
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

// ==================== NETWORK DISCOVERY COMMANDS ====================

/// Obtener la IP local del dispositivo
#[tauri::command]
pub async fn get_local_ip(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    match crate::network_discovery::get_local_ip() {
        Ok(ip) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", &format!("Local IP detected: {}", ip));
            Ok(CommandResponse::success(ip))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error getting local IP: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Escanear la red local en busca de impresoras
#[tauri::command]
pub async fn scan_network_printers(
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<Vec<crate::network_discovery::NetworkPrinter>>, String> {
    let mut app_state = state.write().await;
    app_state.add_log("INFO", "Starting network scan for printers...");
    drop(app_state);

    // Get local IP
    let local_ip = match crate::network_discovery::get_local_ip() {
        Ok(ip) => ip,
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Cannot get local IP: {}", e));
            return Ok(CommandResponse::error(&e));
        }
    };

    // Get network range
    let network_range = match crate::network_discovery::get_network_range(&local_ip) {
        Ok(range) => range,
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Cannot determine network range: {}", e));
            return Ok(CommandResponse::error(&e));
        }
    };

    // Scan network
    match crate::network_discovery::scan_network_for_printers(&network_range).await {
        Ok(printers) => {
            let mut app_state = state.write().await;
            app_state.add_log(
                "INFO",
                &format!("Network scan complete. Found {} printers", printers.len()),
            );
            Ok(CommandResponse::success(printers))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Network scan error: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Agregar una impresora de red al sistema
#[tauri::command]
pub async fn add_network_printer(
    printer: crate::network_discovery::NetworkPrinter,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    let mut app_state = state.write().await;
    app_state.add_log(
        "INFO",
        &format!("Adding network printer: {} ({}:{})", printer.name, printer.ip, printer.port),
    );
    drop(app_state);

    match crate::network_discovery::add_network_printer_to_cups(&printer) {
        Ok(message) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", &message);
            Ok(CommandResponse::success(message))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error adding printer: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
}

/// Eliminar una impresora de red
#[tauri::command]
pub async fn remove_network_printer(
    printer_name: String,
    state: State<'_, SharedAppState>,
) -> Result<CommandResponse<String>, String> {
    match crate::network_discovery::remove_network_printer(&printer_name) {
        Ok(()) => {
            let mut app_state = state.write().await;
            app_state.add_log("INFO", &format!("Removed printer: {}", printer_name));
            Ok(CommandResponse::success(format!(
                "Printer {} removed successfully",
                printer_name
            )))
        }
        Err(e) => {
            let mut app_state = state.write().await;
            app_state.add_log("ERROR", &format!("Error removing printer: {}", e));
            Ok(CommandResponse::error(&e))
        }
    }
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
