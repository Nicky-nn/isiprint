// Tipos para la aplicación

export interface AuthState {
  token: string | null;
  refresh_token: string | null;
  email: string | null;
  is_logged_in: boolean;
}

export interface LicenciaProducto {
  _id: string;
  tipo_producto: string;
  maximo_conexiones: number;
  fecha_vencimiento: string;
  delegado: boolean;
  configuracion: string | null;
  state: string;
}

export interface PrintJob {
  id: number;
  title: string;
  user: string;
  printer: string;
  status: string;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export interface CommandResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export type PaperPreset = 'thermal' | 'carta' | 'oficio' | 'custom';

export interface PrintSettings {
  preset: PaperPreset;
  width_mm?: number;
  height_mm?: number;
}
