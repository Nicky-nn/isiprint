import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type {
  AuthState,
  CommandResponse,
  LicenciaProducto,
  LogEntry,
  PrintJob,
  PrintSettings,
} from "./types";

// Check if we're running inside Tauri
// Uses multiple methods to detect Tauri environment
export const isTauri = (): boolean => {
  if (typeof window === 'undefined') return false;
  
  // Check for __TAURI__ global (Tauri v1)
  if ('__TAURI__' in window) return true;
  
  // Check for __TAURI_INTERNALS__ (Tauri v2)
  if ('__TAURI_INTERNALS__' in window) return true;
  
  // Check user agent
  if (navigator.userAgent.includes('Tauri')) return true;
  
  return false;
};

// Safe invoke wrapper that checks if Tauri is available
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    console.warn(`Tauri not available, cannot invoke: ${cmd}`);
    throw new Error("Tauri API not available - running in browser mode");
  }
  return await tauriInvoke<T>(cmd, args);
}

// API para comunicarse con el backend de Rust

export async function login(email: string, password: string): Promise<CommandResponse<AuthState>> {
  return await invoke("login", { email, password });
}

export async function logout(): Promise<CommandResponse<string>> {
  return await invoke("logout");
}

export async function getAuthState(): Promise<AuthState> {
  return await invoke("get_auth_state");
}

export async function verifySession(): Promise<CommandResponse<AuthState>> {
  return await invoke("verify_session");
}

export async function getLicencias(): Promise<CommandResponse<LicenciaProducto[]>> {
  return await invoke("get_licencias");
}

export async function getPrinters(): Promise<CommandResponse<string[]>> {
  return await invoke("get_printers");
}

export async function printPdf(filePath: string, printerName: string): Promise<CommandResponse<string>> {
  return await invoke("print_pdf", { filePath, printerName });
}

export async function printPdfWithSettings(
  filePath: string,
  printerName: string,
  settings: PrintSettings
): Promise<CommandResponse<string>> {
  return await invoke("print_pdf_with_settings", { filePath, printerName, settings });
}

export async function printPdfFromUrl(pdfUrl: string, printerName: string): Promise<CommandResponse<string>> {
  return await invoke("print_pdf_from_url", { pdfUrl, printerName });
}

export async function printPdfFromUrlWithSettings(
  pdfUrl: string,
  printerName: string,
  settings: PrintSettings
): Promise<CommandResponse<string>> {
  return await invoke("print_pdf_from_url_with_settings", { pdfUrl, printerName, settings });
}

export async function printTestPage(
  printerName: string,
  settings: PrintSettings
): Promise<CommandResponse<string>> {
  return await invoke("print_test_page", { printerName, settings });
}

export async function getPrintJobs(): Promise<CommandResponse<PrintJob[]>> {
  return await invoke("get_print_jobs");
}

export async function clearPrintJobs(): Promise<CommandResponse<string>> {
  return await invoke("clear_print_jobs");
}

export async function sendCutCommand(printerName: string): Promise<CommandResponse<string>> {
  return await invoke("send_cut_command", { printerName });
}

export async function getLogs(): Promise<LogEntry[]> {
  return await invoke("get_logs");
}

export async function validateDomain(origin: string): Promise<boolean> {
  return await invoke("validate_domain", { origin });
}
