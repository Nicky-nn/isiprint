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
}
