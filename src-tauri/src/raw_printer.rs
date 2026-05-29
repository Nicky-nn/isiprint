use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::fs::File;
use std::io::Read;

/// Constantes ESC/POS
const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;

/// Inicializar impresora
pub const INIT: &[u8] = &[ESC, b'@'];
/// Cortar papel
pub const CUT: &[u8] = &[GS, b'V', 66, 0];
/// Negrita ON
pub const BOLD_ON: &[u8] = &[ESC, b'E', 1];
/// Negrita OFF
pub const BOLD_OFF: &[u8] = &[ESC, b'E', 0];
/// Alineación Centro
pub const ALIGN_CENTER: &[u8] = &[ESC, b'a', 1];
/// Alineación Izquierda
pub const ALIGN_LEFT: &[u8] = &[ESC, b'a', 0];
/// Alineación Derecha
pub const ALIGN_RIGHT: &[u8] = &[ESC, b'a', 2];

/// Estructura para conexión directa a impresora
pub struct RawPrinter {
    address: String,
    timeout: Duration,
}

impl RawPrinter {
    pub fn new(ip: &str, port: u16) -> Self {
        Self {
            address: format!("{}:{}", ip, port),
            timeout: Duration::from_secs(5),
        }
    }

    /// Enviar bytes crudos a la impresora
    pub fn print_bytes(&self, data: &[u8]) -> Result<(), String> {
        let addr = self.address.to_socket_addrs()
            .map_err(|e| format!("Invalid address: {}", e))?
            .next()
            .ok_or("Could not resolve address")?;

        let mut stream = TcpStream::connect_timeout(&addr, self.timeout)
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        // Escribir datos
        stream.write_all(data)
            .map_err(|e| format!("Write failed: {}", e))?;
            
        // Asegurar que se enviaron
        stream.flush()
            .map_err(|e| format!("Flush failed: {}", e))?;

        Ok(())
    }

    /// Imprimir archivo local raw (enviar bytes tal cual)
    pub fn print_file(&self, path: &str) -> Result<(), String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Would not open file: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Read failed: {}", e))?;

        self.print_bytes(&buffer)
    }

    /// Generar y enviar página de prueba estilo ticket (ESC/POS)
    pub fn print_test_receipt(&self) -> Result<(), String> {
        let mut buffer = Vec::new();

        // 1. Inicializar
        buffer.extend_from_slice(INIT);

        // 2. Encabezado Centrado
        buffer.extend_from_slice(ALIGN_CENTER);
        buffer.extend_from_slice(BOLD_ON);
        // Doble altura y ancho
        buffer.extend_from_slice(&[GS, b'!', 0x11]); 
        buffer.extend_from_slice(b"ISIPRINT\n");
        // Reset tamaño
        buffer.extend_from_slice(&[GS, b'!', 0x00]);
        buffer.extend_from_slice(b"Prueba de Conexion\n");
        buffer.extend_from_slice(BOLD_OFF);
        buffer.extend_from_slice(b"--------------------------------\n");

        // 3. Info Sistema (Izquierda)
        buffer.extend_from_slice(ALIGN_LEFT);
        buffer.extend_from_slice(b"Estado: ");
        buffer.extend_from_slice(BOLD_ON);
        buffer.extend_from_slice(b"CONECTADO\n");
        buffer.extend_from_slice(BOLD_OFF);
        
        buffer.extend_from_slice(b"IP Impresora: ");
        buffer.extend_from_slice(self.address.as_bytes());
        buffer.extend_from_slice(b"\n");
        
        buffer.extend_from_slice(b"Modo: ");
        buffer.extend_from_slice(b"RAW SOCKET / ESC-POS\n");
        
        let now = chrono::Local::now();
        let date_str = now.format("%d/%m/%Y %H:%M:%S").to_string();
        buffer.extend_from_slice(b"Fecha: ");
        buffer.extend_from_slice(date_str.as_bytes());
        buffer.extend_from_slice(b"\n\n");

        // 4. Mensaje
        buffer.extend_from_slice(ALIGN_CENTER);
        buffer.extend_from_slice(b"Esta es una prueba de impresion\n");
        buffer.extend_from_slice(b"Directa sin Drivers (Driverless)\n");
        buffer.extend_from_slice(b"usando protocolo TCP/IP port 9100\n");
        buffer.extend_from_slice(b"\n");

        // 5. Pie de página
        buffer.extend_from_slice(b"--------------------------------\n");
        buffer.extend_from_slice(b"Desarrollado por IsiPrint\n");
        buffer.extend_from_slice(b"\n\n\n\n"); // Feed

        // 6. Corte de papel
        buffer.extend_from_slice(CUT);

        // 7. Enviar
        self.print_bytes(&buffer)
    }

    /// Imprimir una imagen (convirtiéndola a ESC/POS Raster)
    pub fn print_image(&self, img: &image::DynamicImage) -> Result<(), String> {
        let width = img.width();
        let height = img.height();
        
        // Convertir a escala de grises (luma8)
        let gray_img = img.to_luma8();
        
        // Ancho en bytes (8 pixeles por byte)
        let width_bytes = (width + 7) / 8;
        
        // Buffer de comandos
        let mut buffer = Vec::new();
        buffer.extend_from_slice(INIT);
        buffer.extend_from_slice(ALIGN_CENTER);
        
        // Comando GS v 0 (Raster Bit Image)
        // GS v 0 m xL xH yL yH d1...dk
        // m=0 (density normal), xL/xH = width bytes, yL/yH = height lines
        
        buffer.extend_from_slice(&[GS, b'v', b'0', 0]);
        buffer.push((width_bytes & 0xFF) as u8);
        buffer.push(((width_bytes >> 8) & 0xFF) as u8);
        buffer.push((height & 0xFF) as u8);
        buffer.push(((height >> 8) & 0xFF) as u8);
        
        for y in 0..height {
            for x_byte in 0..width_bytes {
                let mut byte = 0u8;
                for bit in 0..8 {
                    let x = x_byte * 8 + bit;
                    if x < width {
                        // Obtener pixel (negro es < 128)
                        let pixel = gray_img.get_pixel(x, y)[0];
                        if pixel < 128 {
                            byte |= 1 << (7 - bit);
                        }
                    }
                }
                buffer.push(byte);
            }
        }
        
        buffer.extend_from_slice(b"\n\n\n\n"); // Feed
        buffer.extend_from_slice(CUT);
        
        self.print_bytes(&buffer)
    }
    
    /// Imprimir PDF renderizándolo primero a imagen (macOS/native)
    pub fn print_pdf_renderer(&self, pdf_path: &str) -> Result<(), String> {
        use std::process::Command;
        use std::fs;
        use std::path::Path;

        // Verificar que el PDF de entrada existe y tiene contenido
        let pdf_meta = fs::metadata(pdf_path)
            .map_err(|e| format!("Input PDF check failed: {}", e))?;
        if pdf_meta.len() == 0 {
            return Err("Input PDF file is empty".to_string());
        }
        
        println!("DEBUG: Input PDF path: {}, Size: {}", pdf_path, pdf_meta.len());

        let temp_dir = std::env::temp_dir();
        let uuid = uuid::Uuid::new_v4();
        
        // COPIA DE SEGURIDAD: Copiar el PDF a un nuevo archivo temporal
        // Esto evita problemas si el archivo original tiene permisos raros o está en uso
        let safe_pdf_name = format!("isiprint_input_{}.pdf", uuid);
        let safe_pdf_path = temp_dir.join(safe_pdf_name);
        fs::copy(pdf_path, &safe_pdf_path)
            .map_err(|e| format!("Failed to copy input PDF to safe location: {}", e))?;
            
        let safe_pdf_path_str = safe_pdf_path.to_string_lossy().to_string();
        println!("DEBUG: Safe PDF path: {}", safe_pdf_path_str);

        // 1. Generar ruta temporal para PNG
        let png_name = format!("isiprint_render_{}.png", uuid);
        let png_path = temp_dir.join(png_name);
        let png_path_str = png_path.to_string_lossy().to_string();
        
        println!("DEBUG: Output PNG path: {}", png_path_str);
        
        // 2. Usar 'sips' (macOS) para convertir PDF a PNG
        let mut conversion_success = false;
        
        #[cfg(target_os = "macos")]
        {
            // Intento 1: sips
            println!("DEBUG: Trying sips conversion...");
            let output = Command::new("sips")
                .args(["-s", "format", "png", &safe_pdf_path_str, "--out", &png_path_str])
                .output()
                .map_err(|e| format!("Error executing sips: {}", e))?;
                
            if output.status.success() {
                conversion_success = true;
                println!("DEBUG: sips conversion successful");
            } else {
                println!("DEBUG: sips failed. Stderr: {}", String::from_utf8_lossy(&output.stderr));
                println!("DEBUG: Trying qlmanage fallback...");
                
                // Intento 2: qlmanage (QuickLook)
                // qlmanage -t -s 1000 -o <dir> <file>
                // Genera <file>.png en <dir>
                let output_ql = Command::new("qlmanage")
                    .args(["-t", "-s", "1000", "-o", temp_dir.to_str().unwrap(), &safe_pdf_path_str])
                    .output()
                    .map_err(|e| format!("Error executing qlmanage: {}", e))?;
                    
                // qlmanage es ruidoso, verificamos si el archivo esperado existe
                // El nombre generado suele ser "<nombre_pdf>.png"
                let expected_ql_output = format!("{}.png", safe_pdf_path_str);
                let expected_path = Path::new(&expected_ql_output);
                
                if expected_path.exists() {
                     // Mover al path esperado (png_path)
                     fs::rename(expected_path, &png_path)
                        .map_err(|e| format!("Error moving qlmanage output: {}", e))?;
                     conversion_success = true;
                     println!("DEBUG: qlmanage conversion successful");
                } else {
                     println!("DEBUG: qlmanage failed. Output file not found: {}", expected_ql_output);
                     println!("DEBUG: qlmanage stderr: {}", String::from_utf8_lossy(&output_ql.stderr));
                     
                     // Limpiar intentos fallidos
                     let _ = fs::remove_file(&png_path);
                }
            }
        }

        // Limpiar el PDF seguro temporal
        let _ = fs::remove_file(&safe_pdf_path);
        
        #[cfg(not(target_os = "macos"))]
        return Err("Internal PDF rendering only supported on macOS currently".to_string());
        
        if !conversion_success {
             return Err("PDF rendering failed with both sips and qlmanage".to_string());
        }
        
        // 3. Cargar imagen con librería 'image'
        let img = image::open(&png_path)
            .map_err(|e| {
                let _ = fs::remove_file(&png_path); // Limpiar
                format!("Error loading rendered image: {}", e)
            })?;
        
        // Limpiar archivo temporal PNG una vez cargado en memoria
        let _ = fs::remove_file(&png_path);
            
        // 4. Redimensionar si es muy ancha (max 512px o 576px para térmicas 80mm standard)
        let target_width = 570;
        let final_img = if img.width() > target_width {
            img.resize(target_width, (target_width * img.height()) / img.width(), image::imageops::FilterType::Triangle)
        } else {
            img
        };
        
        // 5. Imprimir imagen procesada
        self.print_image(&final_img)
    }
}
