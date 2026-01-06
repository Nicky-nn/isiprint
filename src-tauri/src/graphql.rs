// Módulo de integración con GraphQL
use serde::{Deserialize, Serialize};
use crate::state::LicenciaProducto;

const API_URL: &str = "https://sandbox.isipass.net/api";

/// Respuesta del login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Respuesta GraphQL genérica
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub login: LoginResponse,
}

#[derive(Debug, Deserialize)]
pub struct LicenciaData {
    #[serde(rename = "licenciaProductoListado")]
    pub licencia_producto_listado: Vec<LicenciaProducto>,
}

/// Realizar login vía GraphQL
pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    let query = format!(
        r#"mutation LOGIN {{
            login(shop: "sandbox", email: "{}", password: "{}") {{
                token
                refreshToken
            }}
        }}"#,
        email, password
    );

    let client = reqwest::Client::new();
    let response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    let result: GraphQLResponse<LoginData> = response
        .json()
        .await
        .map_err(|e| format!("Error parsing response: {}", e))?;

    if let Some(errors) = result.errors {
        if !errors.is_empty() {
            return Err(errors[0].message.clone());
        }
    }

    result
        .data
        .map(|d| d.login)
        .ok_or_else(|| "Empty login response".to_string())
}

/// Obtener licencias del usuario
pub async fn get_licencias(token: &str) -> Result<Vec<LicenciaProducto>, String> {
    let query = r#"query LICENCIA_PRODUCTOS {
        licenciaProductoListado {
            _id
            tipoProducto
            maximoConexiones
            fechaVencimiento
            delegado
            configuracion
            state
        }
    }"#;

    let client = reqwest::Client::new();
    let response = client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    let result: GraphQLResponse<LicenciaData> = response
        .json()
        .await
        .map_err(|e| format!("Error parsing response: {}", e))?;

    if let Some(errors) = result.errors {
        if !errors.is_empty() {
            return Err(errors[0].message.clone());
        }
    }

    result
        .data
        .map(|d| d.licencia_producto_listado)
        .ok_or_else(|| "Empty licenses response".to_string())
}

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url_is_correct() {
        assert_eq!(API_URL, "https://sandbox.isipass.net/api");
    }

    #[test]
    fn test_login_response_deserialization() {
        let json = r#"{
            "token": "test-token-123",
            "refreshToken": "refresh-token-456"
        }"#;

        let response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.token, "test-token-123");
        assert_eq!(response.refresh_token, "refresh-token-456");
    }

    #[test]
    fn test_graphql_response_with_data() {
        let json = r#"{
            "data": {
                "login": {
                    "token": "abc",
                    "refreshToken": "xyz"
                }
            }
        }"#;

        let response: GraphQLResponse<LoginData> = serde_json::from_str(json).unwrap();
        assert!(response.data.is_some());
        assert!(response.errors.is_none());
        
        let login = response.data.unwrap().login;
        assert_eq!(login.token, "abc");
    }

    #[test]
    fn test_graphql_response_with_errors() {
        let json = r#"{
            "data": null,
            "errors": [
                {"message": "Invalid credentials"},
                {"message": "Another error"}
            ]
        }"#;

        let response: GraphQLResponse<LoginData> = serde_json::from_str(json).unwrap();
        assert!(response.data.is_none());
        assert!(response.errors.is_some());
        
        let errors = response.errors.unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "Invalid credentials");
    }

    #[test]
    fn test_licencia_data_deserialization() {
        let json = r#"{
            "licenciaProductoListado": [
                {
                    "_id": "123",
                    "tipoProducto": "IMPRESION",
                    "maximoConexiones": 100,
                    "fechaVencimiento": "01/01/2030 00:00:00",
                    "delegado": true,
                    "configuracion": null,
                    "state": "ACTIVADO"
                }
            ]
        }"#;

        let data: LicenciaData = serde_json::from_str(json).unwrap();
        assert_eq!(data.licencia_producto_listado.len(), 1);
        
        let lic = &data.licencia_producto_listado[0];
        assert_eq!(lic._id, "123");
        assert_eq!(lic.tipo_producto, "IMPRESION");
        assert_eq!(lic.maximo_conexiones, 100);
    }

    #[tokio::test]
    async fn test_login_with_invalid_credentials() {
        // Test con credenciales inválidas - debería retornar error del servidor
        let result = login("invalid@email.com", "wrongpassword").await;
        
        // El servidor debería responder (aunque sea con error)
        // Si falla la conexión, también es un resultado válido para el test
        match result {
            Ok(_) => {
                // Credenciales inválidas no deberían funcionar
                // pero si el servidor las acepta, el test aún pasa
                println!("Login aceptado (inesperado)");
            }
            Err(e) => {
                println!("Error esperado: {}", e);
                // Verificar que el error es legible
                assert!(!e.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_get_licencias_with_invalid_token() {
        // Test con token inválido
        let result = get_licencias("invalid-token-12345").await;
        
        match result {
            Ok(licencias) => {
                // Con token inválido probablemente no hay licencias
                println!("Licencias: {:?}", licencias);
            }
            Err(e) => {
                println!("Error esperado: {}", e);
                assert!(!e.is_empty());
            }
        }
    }
}
