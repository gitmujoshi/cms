use serde::{Deserialize, Serialize};
use web_sys::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    TrainingDataProvider,
    CleanRoomProvider,
    DataConsumer,
    SystemAdministrator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub organization_name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    user: Option<User>,
    token: Option<String>,
}

impl AuthContext {
    pub fn new() -> Self {
        // Try to load from localStorage
        if let Ok(Some(storage)) = web_sys::window().and_then(|win| win.local_storage()) {
            if let Ok(Some(token)) = storage.get_item("auth_token") {
                if let Ok(Some(user_json)) = storage.get_item("auth_user") {
                    if let Ok(user) = serde_json::from_str(&user_json) {
                        return Self {
                            user: Some(user),
                            token: Some(token),
                        };
                    }
                }
            }
        }
        Self::default()
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() && self.user.is_some()
    }

    pub fn user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn role(&self) -> Option<&Role> {
        self.user.as_ref().map(|u| &u.role)
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn login(&mut self, email: &str, password: &str) -> Result<(), String> {
        // Call login API
        let response = api::post("/api/auth/login", &serde_json::json!({
            "email": email,
            "password": password,
        }))
        .await
        .map_err(|e| e.to_string())?;

        let auth_data: AuthResponse = response.json()
            .await
            .map_err(|e| e.to_string())?;

        // Store in localStorage
        if let Ok(Some(storage)) = web_sys::window().and_then(|win| win.local_storage()) {
            storage.set_item("auth_token", &auth_data.token)
                .map_err(|e| e.to_string())?;
            storage.set_item("auth_user", &serde_json::to_string(&auth_data.user)
                .map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }

        self.token = Some(auth_data.token);
        self.user = Some(auth_data.user);

        Ok(())
    }

    pub async fn logout(&mut self) -> Result<(), String> {
        // Call logout API if needed
        if let Some(token) = &self.token {
            let _ = api::post("/api/auth/logout", &()).await;
        }

        // Clear localStorage
        if let Ok(Some(storage)) = web_sys::window().and_then(|win| win.local_storage()) {
            storage.remove_item("auth_token").map_err(|e| e.to_string())?;
            storage.remove_item("auth_user").map_err(|e| e.to_string())?;
        }

        self.token = None;
        self.user = None;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
    user: User,
} 