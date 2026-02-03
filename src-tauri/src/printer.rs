// Módulo de manejo de impresoras
use serde::{Deserialize, Serialize};
use std::io::Write;
use tempfile::NamedTempFile;

/// Información de un trabajo de impresión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: i32,
    pub title: String,
    pub user: String,
    pub printer: String,
    pub status: String,
}

/// Comando de corte para impresoras térmicas (ESC/POS)
const CUT_COMMAND: &[u8] = &[0x1D, 0x56, 0x00];

// ==================== macOS / Linux (CUPS) ====================

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn list_printers() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    // Use lpstat -e to list printers (names only, works in any language)
    let output = Command::new("lpstat")
        .args(["-e"])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}", e))?;

    if !output.status.success() {
        // If -e fails, try -a as fallback
        let output_a = Command::new("lpstat")
            .args(["-a"])
            .output()
            .map_err(|e| format!("Error executing lpstat: {}", e))?;
        
        if !output_a.status.success() {
            return Err("Error getting printer list".to_string());
        }
        
        // lpstat -a format: "PRINTER accepting requests since..."
        let stdout = String::from_utf8_lossy(&output_a.stdout);
        let printers: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                // Printer name is the first word
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .collect();
        
        return Ok(printers);
    }

    // lpstat -e only returns printer names, one per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(printers)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn print_file(file_path: &str, printer_name: &str) -> Result<i32, String> {
    use std::process::Command;
    
    // Verify file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(format!("File {} does not exist", file_path));
    }

    // Use lp to print
    let output = Command::new("lp")
        .args(["-d", printer_name, file_path])
        .output()
        .map_err(|e| format!("Error executing lp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Print error: {}", stderr));
    }

    // Extraer job ID de la salida
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Formato: "request id is PRINTER-123 (1 file(s))"
    let job_id = stdout
        .split('-')
        .last()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    Ok(job_id)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn print_file_with_media(
    file_path: &str,
    printer_name: &str,
    media: Option<&str>,
) -> Result<i32, String> {
    use std::process::Command;

    // Verify file exists
    if !std::path::Path::new(file_path).exists() {
        log::error!("File does not exist: {}", file_path);
        return Err(format!("File {} does not exist", file_path));
    }
    

    let mut cmd = Command::new("lp");
    cmd.args(["-d", printer_name]);
    if let Some(media) = media {
        // CUPS option: media=<name>
        cmd.args(["-o", &format!("media={}", media)]);
    }
    cmd.arg(file_path);
    
    let output = cmd
        .output()
        .map_err(|e| format!("Error executing lp: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        return Err(format!("Print error: {}", stderr));
    }

    let job_id = stdout
        .split('-')
        .last()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    
    Ok(job_id)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn send_cut(printer_name: &str) -> Result<(), String> {
    // Create temp file with cut command
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Error creating temp file: {}", e))?;
    
    temp_file
        .write_all(CUT_COMMAND)
        .map_err(|e| format!("Error writing cut command: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();
    print_file(&temp_path, printer_name)?;

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn get_jobs() -> Result<Vec<PrintJob>, String> {
    use std::process::Command;
    
    let output = Command::new("lpstat")
        .args(["-o"])
        .output()
        .map_err(|e| format!("Error executing lpstat: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]); // No jobs
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let jobs: Vec<PrintJob> = stdout
        .lines()
        .filter_map(|line| {
            // Formato: "PRINTER-123 user 1024 Mon Jan 1 12:00:00 2024"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let job_info: Vec<&str> = parts[0].split('-').collect();
                if job_info.len() >= 2 {
                    return Some(PrintJob {
                        id: job_info.last().and_then(|s| s.parse().ok()).unwrap_or(0),
                        title: parts[0].to_string(),
                        user: parts[1].to_string(),
                        printer: job_info[0].to_string(),
                        status: "pending".to_string(),
                    });
                }
            }
            None
        })
        .collect();

    Ok(jobs)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn clear_jobs() -> Result<(), String> {
    use std::process::Command;
    
    let output = Command::new("cancel")
        .args(["-a"])
        .output()
        .map_err(|e| format!("Error executing cancel: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error cancelling jobs: {}", stderr));
    }

    Ok(())
}

// ==================== Windows ====================

#[cfg(target_os = "windows")]
pub fn list_printers() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    // Use wmic to list printers on Windows
    let output = Command::new("wmic")
        .args(["printer", "get", "name"])
        .output()
        .map_err(|e| format!("Error executing wmic: {}", e))?;

    if !output.status.success() {
        return Err("Error getting printer list".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let printers: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(printers)
}

#[cfg(target_os = "windows")]
pub fn print_file(file_path: &str, printer_name: &str) -> Result<i32, String> {
    use std::process::Command;
    
    if !std::path::Path::new(file_path).exists() {
        return Err(format!("File {} does not exist", file_path));
    }

    // Use Windows print command
    let output = Command::new("cmd")
        .args(["/c", "print", &format!("/D:{}", printer_name), file_path])
        .output()
        .map_err(|e| format!("Print error: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Print error: {}", stderr));
    }

    Ok(0) // Windows doesn't easily return job ID
}

#[cfg(target_os = "windows")]
pub fn send_cut(printer_name: &str) -> Result<(), String> {
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| format!("Error creating temp file: {}", e))?;
    
    temp_file
        .write_all(CUT_COMMAND)
        .map_err(|e| format!("Error writing cut command: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();
    print_file(&temp_path, printer_name)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_jobs() -> Result<Vec<PrintJob>, String> {
    use std::process::Command;
    
    let output = Command::new("wmic")
        .args(["printjob", "get", "jobid,document,owner,name,status"])
        .output()
        .map_err(|e| format!("Error executing wmic: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let jobs: Vec<PrintJob> = stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return Some(PrintJob {
                    id: parts[0].parse().unwrap_or(0),
                    title: parts[1].to_string(),
                    user: parts[2].to_string(),
                    printer: parts[3].to_string(),
                    status: parts.get(4).unwrap_or(&"pending").to_string(),
                });
            }
            None
        })
        .collect();

    Ok(jobs)
}

#[cfg(target_os = "windows")]
pub fn clear_jobs() -> Result<(), String> {
    use std::process::Command;
    
    let output = Command::new("net")
        .args(["stop", "spooler"])
        .output()
        .map_err(|e| format!("Error stopping spooler: {}", e))?;

    if !output.status.success() {
        return Err("Error stopping spooler".to_string());
    }

    // Restart spooler
    Command::new("net")
        .args(["start", "spooler"])
        .output()
        .map_err(|e| format!("Error starting spooler: {}", e))?;

    Ok(())
}

/// Download PDF from URL
pub async fn download_pdf(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Error downloading PDF: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Error reading PDF: {}", e))
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_command_bytes() {
        // Verificar que el comando de corte ESC/POS es correcto
        assert_eq!(CUT_COMMAND, &[0x1D, 0x56, 0x00]);
        assert_eq!(CUT_COMMAND.len(), 3);
    }

    #[test]
    fn test_print_job_struct() {
        let job = PrintJob {
            id: 123,
            title: "Test Document".to_string(),
            user: "testuser".to_string(),
            printer: "TestPrinter".to_string(),
            status: "pending".to_string(),
        };

        assert_eq!(job.id, 123);
        assert_eq!(job.title, "Test Document");
        assert_eq!(job.user, "testuser");
        assert_eq!(job.printer, "TestPrinter");
        assert_eq!(job.status, "pending");
    }

    #[test]
    fn test_print_job_serialization() {
        let job = PrintJob {
            id: 1,
            title: "Doc".to_string(),
            user: "user".to_string(),
            printer: "Printer".to_string(),
            status: "completed".to_string(),
        };

        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"title\":\"Doc\""));
    }

    #[test]
    fn test_list_printers_runs() {
        // Este test verifica que la función se ejecuta sin panic
        // El resultado puede ser Ok o Err dependiendo del sistema
        let result = list_printers();
        // Solo verificamos que no hace panic
        match result {
            Ok(printers) => {
                println!("Impresoras encontradas: {:?}", printers);
            }
            Err(e) => {
                println!("Error esperado en entorno sin impresoras: {}", e);
            }
        }
    }

    #[test]
    fn test_get_jobs_runs() {
        // Verificar que la función se ejecuta sin panic
        let result = get_jobs();
        match result {
            Ok(jobs) => {
                println!("Trabajos encontrados: {:?}", jobs);
            }
            Err(e) => {
                println!("Error esperado: {}", e);
            }
        }
    }

    #[test]
    fn test_print_file_nonexistent() {
        // Try to print a file that doesn't exist
        let result = print_file("/nonexistent/path/to/file.pdf", "FakePrinter");
        assert!(result.is_err(), "Should fail with nonexistent file");
        
        let error = result.unwrap_err();
        assert!(
            error.contains("does not exist") || error.contains("Error"),
            "Error should mention file does not exist"
        );
    }

    #[tokio::test]
    async fn test_download_pdf_invalid_url() {
        // Invalid URL
        let result = download_pdf("not-a-valid-url").await;
        assert!(result.is_err(), "Should fail with invalid URL");
    }

    #[tokio::test]
    async fn test_download_pdf_nonexistent_domain() {
        // Non-existent domain
        let result = download_pdf("https://this-domain-does-not-exist-12345.com/file.pdf").await;
        assert!(result.is_err(), "Should fail with nonexistent domain");
    }
}
