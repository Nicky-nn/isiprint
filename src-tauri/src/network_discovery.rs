// Módulo de descubrimiento de impresoras en red
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, TcpStream};
use std::time::Duration;
use tokio::task;

/// Información de una impresora descubierta en la red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPrinter {
    pub ip: String,
    pub port: u16,
    pub protocol: String, // "ipp", "raw", "http"
    pub name: String,
    pub is_online: bool,
}

/// Obtener la IP local del dispositivo
pub fn get_local_ip() -> Result<String, String> {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| format!("Error getting local IP: {}", e))
}

/// Obtener el rango de red local (e.g., "192.168.1.0/24")
pub fn get_network_range(local_ip: &str) -> Result<String, String> {
    let ip: IpAddr = local_ip
        .parse()
        .map_err(|e| format!("Invalid IP address: {}", e))?;
    
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // Asume máscara /24 (255.255.255.0) que es la más común en redes domésticas
            Ok(format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]))
        }
        IpAddr::V6(_) => Err("IPv6 not supported yet".to_string()),
    }
}

/// Verificar si un puerto está abierto en un host
fn is_port_open(ip: &str, port: u16, timeout_ms: u64) -> bool {
    let addr = format!("{}:{}", ip, port);
    TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(timeout_ms),
    )
    .is_ok()
}

/// Escanear un rango de IPs buscando puertos comunes de impresoras
pub async fn scan_network_for_printers(network_range: &str) -> Result<Vec<NetworkPrinter>, String> {
    log::info!("Scanning network range: {}", network_range);
    
    // Parse network range
    let network: ipnetwork::IpNetwork = network_range
        .parse()
        .map_err(|e| format!("Invalid network range: {}", e))?;
    
    let mut tasks = Vec::new();
    
    // Puertos comunes de impresoras:
    // 9100 - HP JetDirect / Raw printing
    // 631  - IPP (Internet Printing Protocol)
    // 515  - LPD (Line Printer Daemon)
    let printer_ports = vec![9100, 631, 515];
    
    // Escanear todas las IPs en el rango
    for ip in network.iter() {
        if let IpAddr::V4(ipv4) = ip {
            let ip_str = ipv4.to_string();
            
            // Evitar IPs de broadcast y network address
            let last_octet = ipv4.octets()[3];
            if last_octet == 0 || last_octet == 255 {
                continue;
            }
            
            for &port in &printer_ports {
                let ip_clone = ip_str.clone();
                let task = task::spawn_blocking(move || {
                    if is_port_open(&ip_clone, port, 100) {
                        Some((ip_clone, port))
                    } else {
                        None
                    }
                });
                tasks.push(task);
            }
        }
    }
    
    // Esperar resultados
    let mut printers = Vec::new();
    let mut printer_count = 0;
    
    for task in tasks {
        if let Ok(Some((ip, port))) = task.await {
            let protocol = match port {
                631 => "ipp",
                9100 => "raw",
                515 => "lpd",
                _ => "unknown",
            };
            
            let name = format!("Network_Printer_{}_{}", ip.replace('.', "_"), port);
            
            printers.push(NetworkPrinter {
                ip: ip.clone(),
                port,
                protocol: protocol.to_string(),
                name: name.clone(),
                is_online: true,
            });
            
            printer_count += 1;
            log::info!("Found printer: {} at {}:{} ({})", name, ip, port, protocol);
        }
    }
    
    log::info!("Network scan complete. Found {} printers.", printer_count);
    Ok(printers)
}

/// Agregar una impresora de red a CUPS (macOS/Linux)
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn add_network_printer_to_cups(printer: &NetworkPrinter) -> Result<String, String> {
    use std::process::Command;
    
    // Construir URI según el protocolo
    let uri = match printer.protocol.as_str() {
        "ipp" => format!("ipp://{}/ipp/print", printer.ip),
        "raw" | "9100" => format!("socket://{}:{}", printer.ip, printer.port),
        "lpd" => format!("lpd://{}/queue", printer.ip),
        _ => format!("socket://{}:{}", printer.ip, printer.port),
    };
    
    log::info!("Adding printer {} with URI: {}", printer.name, uri);
    
    // Verificar si la impresora ya existe
    let check = Command::new("lpstat")
        .args(["-p", &printer.name])
        .output();
    
    if let Ok(output) = check {
        if output.status.success() {
            log::info!("Printer {} already exists, skipping", printer.name);
            return Ok(format!("Printer {} already installed", printer.name));
        }
    }
    
    // Determinar el mejor driver según el protocolo y el OS
    let driver = if cfg!(target_os = "macos") {
        // macOS ya no soporta "raw" desde Big Sur
        // Usar IPP Everywhere para todo - maneja PDFs correctamente
        "everywhere"
    } else {
        // Linux también usa IPP Everywhere
        "everywhere"
    };
    
    log::info!("Using driver: {} for printer: {}", driver, printer.name);
    
    // Agregar la impresora usando lpadmin
    let mut cmd = Command::new("lpadmin");
    cmd.args(["-p", &printer.name, "-v", &uri, "-E"]);
    
    // Agregar el driver apropiado
    if driver == "everywhere" {
        // IPP Everywhere no usa -m, usa -m everywhere
        cmd.args(["-m", "everywhere"]);
    } else {
        cmd.args(["-m", driver]);
    }
    
    let output = cmd
        .output()
        .map_err(|e| format!("Error executing lpadmin: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Si falla con el driver seleccionado, intentar con alternativas
        if cfg!(target_os = "macos") && !stderr.is_empty() {
            log::warn!("IPP Everywhere failed, trying PostScript/LaserJet driver...");
            
            // Intentar con otro driver genérico
            let output2 = Command::new("lpadmin")
                .args([
                    "-p", &printer.name,
                    "-v", &uri,
                    "-E",
                    "-m", "drv:///sample.drv/laserjet.ppd", // Generic LaserJet (PostScript)
                ])
                .output()
                .map_err(|e| format!("Error executing lpadmin (retry): {}", e))?;
            
            if !output2.status.success() {
                let stderr2 = String::from_utf8_lossy(&output2.stderr);
                
                // Último intento: sin especificar driver (CUPS auto-detecta)
                let output3 = Command::new("lpadmin")
                    .args([
                        "-p", &printer.name,
                        "-v", &uri,
                        "-E",
                    ])
                    .output()
                    .map_err(|e| format!("Error executing lpadmin (final retry): {}", e))?;
                
                if !output3.status.success() {
                    let stderr3 = String::from_utf8_lossy(&output3.stderr);
                    return Err(format!("Error adding printer after all attempts:\n1. {}\n2. {}\n3. {}", 
                        stderr, stderr2, stderr3));
                }
            }
        } else {
            return Err(format!("Error adding printer: {}", stderr));
        }
    }
    
    log::info!("Successfully added printer: {}", printer.name);
    Ok(format!("Printer {} added successfully", printer.name))
}

/// Agregar impresora de red en Windows
#[cfg(target_os = "windows")]
pub fn add_network_printer_to_cups(printer: &NetworkPrinter) -> Result<String, String> {
    use std::process::Command;
    
    let port_name = format!("IP_{}_{}", printer.ip.replace('.', "_"), printer.port);
    let printer_name = &printer.name;
    
    // Crear puerto de impresora TCP/IP
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Add-PrinterPort -Name '{}' -PrinterHostAddress '{}'",
                port_name, printer.ip
            ),
        ])
        .output()
        .map_err(|e| format!("Error creating printer port: {}", e))?;
    
    if !output.status.success() {
        log::warn!("Port creation warning (may already exist)");
    }
    
    // Agregar la impresora
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Add-Printer -Name '{}' -PortName '{}' -DriverName 'Generic / Text Only'",
                printer_name, port_name
            ),
        ])
        .output()
        .map_err(|e| format!("Error adding printer: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error adding printer: {}", stderr));
    }
    
    Ok(format!("Printer {} added successfully", printer_name))
}

/// Eliminar una impresora de CUPS
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn remove_network_printer(printer_name: &str) -> Result<(), String> {
    use std::process::Command;
    
    let output = Command::new("lpadmin")
        .args(["-x", printer_name])
        .output()
        .map_err(|e| format!("Error executing lpadmin: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error removing printer: {}", stderr));
    }
    
    log::info!("Successfully removed printer: {}", printer_name);
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_network_printer(printer_name: &str) -> Result<(), String> {
    use std::process::Command;
    
    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!("Remove-Printer -Name '{}'", printer_name),
        ])
        .output()
        .map_err(|e| format!("Error removing printer: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error removing printer: {}", stderr));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_ip() {
        let result = get_local_ip();
        println!("Local IP: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_network_range() {
        let result = get_network_range("192.168.1.100");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.0/24");
    }

    #[test]
    fn test_network_printer_serialization() {
        let printer = NetworkPrinter {
            ip: "192.168.1.100".to_string(),
            port: 9100,
            protocol: "raw".to_string(),
            name: "Network_Printer_1".to_string(),
            is_online: true,
        };
        
        let json = serde_json::to_string(&printer).unwrap();
        assert!(json.contains("192.168.1.100"));
        assert!(json.contains("9100"));
    }
}
