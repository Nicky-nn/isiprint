// Módulos del proyecto
mod commands;
mod graphql;
mod http_server;
mod network_discovery;
mod persistence;
mod printer;
mod raw_printer;
mod state;

use state::AppState;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::RwLock;

/// Estado compartido envuelto en Arc<RwLock> para ser accesible desde Tauri y HTTP server
pub type SharedAppState = Arc<RwLock<AppState>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();

    // Try to load saved session
    let initial_state = if let Some(saved_session) = persistence::load_session() {
        log::info!("Found saved session for: {:?}", saved_session.auth.email);
        let mut state = AppState::default();
        state.auth = saved_session.auth;
        state.licencias = saved_session.licencias;
        state
    } else {
        log::info!("No saved session, starting fresh");
        AppState::default()
    };

    // Shared state for the whole application
    let shared_state: SharedAppState = Arc::new(RwLock::new(initial_state));
    let http_state = shared_state.clone();

    // Start HTTP server in a separate thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(http_server::start_http_server(http_state));
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // If another instance tries to open, show the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(shared_state)
        .setup(|app| {
            // macOS: ocultar el icono del Dock (tray-only)
            #[cfg(target_os = "macos")]
            {
                let _ = app.handle().set_dock_visibility(false);
            }

            // Crear menú del tray
            let show_item = MenuItem::with_id(app, "show", "Abrir ISIPRINT", true, None::<&str>)?;
            let printers_item = MenuItem::with_id(app, "printers", "Ver Impresoras", true, None::<&str>)?;
            let separator = MenuItem::with_id(app, "sep", "─────────────", false, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &printers_item, &separator, &quit_item])?;

            // Usar el icono del proyecto (no el default)
            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

            // Crear el tray icon
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon.into())
                .menu(&menu)
                // Click izquierdo solo abre el menú, no la ventana (en todos los OS)
                .show_menu_on_left_click(true)
                .tooltip("ISIPRINT")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "printers" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            // Emitir evento para cambiar a pestaña de impresoras
                            let _ = window.emit("navigate", "printers");
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Startup log
            if let Some(state) = app.try_state::<SharedAppState>() {
                let mut app_state = state.blocking_write();
                app_state.add_log("INFO", "ISIPRINT started successfully");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::get_licencias,
            commands::get_printers,
            commands::print_pdf,
            commands::print_pdf_from_url,
            commands::print_pdf_with_settings,
            commands::print_pdf_from_url_with_settings,
            commands::print_test_page,
            commands::get_print_jobs,
            commands::clear_print_jobs,
            commands::send_cut_command,
            commands::get_logs,
            commands::validate_domain,
            commands::get_auth_state,
            commands::verify_session,
            commands::logout,
            // Network discovery commands
            commands::get_local_ip,
            commands::scan_network_printers,
            commands::add_network_printer,
            commands::remove_network_printer,
        ])
        .on_window_event(|window, event| {
            // Al cerrar la ventana, solo ocultarla (no cerrar la app)
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
