//! Health check handlers
//!
//! Provides health check endpoints for monitoring.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::presentation::AppState;

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
}

/// Detailed health response.
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub providers: ProviderHealth,
    pub accounts: AccountHealthSummary,
}

/// Provider health summary.
#[derive(Debug, Serialize)]
pub struct ProviderHealth {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
}

/// Account health summary.
#[derive(Debug, Serialize)]
pub struct AccountHealthSummary {
    pub total: usize,
    pub active: usize,
    pub inactive: usize,
}

/// Handler for GET /health
pub async fn health() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: current_timestamp(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Json(response)
}

/// Handler for GET /health/detail
pub async fn health_detail(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DetailedHealthResponse>, StatusCode> {
    // Get account stats
    let accounts = state
        .account_repo
        .find_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_accounts = state
        .account_repo
        .find_active()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Group accounts by provider
    let mut provider_map: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    for account in &accounts {
        let entry = provider_map
            .entry(account.provider_id.clone())
            .or_insert((0, 0));
        entry.0 += 1; // total accounts for provider
        if account.is_active {
            entry.1 += 1; // active accounts
        }
    }

    let response = DetailedHealthResponse {
        status: "healthy".to_string(),
        timestamp: current_timestamp(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        providers: ProviderHealth {
            total: provider_map.len(),
            enabled: provider_map
                .iter()
                .filter(|(_, (_, active))| *active > 0)
                .count(),
            disabled: provider_map
                .iter()
                .filter(|(_, (_, active))| *active == 0)
                .count(),
        },
        accounts: AccountHealthSummary {
            total: accounts.len(),
            active: active_accounts.len(),
            inactive: accounts.len() - active_accounts.len(),
        },
    };

    Ok(Json(response))
}

/// Handler for GET /accounts
pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AccountInfo>>, StatusCode> {
    let accounts = state
        .account_repo
        .find_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_infos: Vec<AccountInfo> = accounts
        .into_iter()
        .map(|a| AccountInfo {
            id: a.id,
            provider_id: a.provider_id,
            is_active: a.is_active,
            priority: a.priority,
            api_key_prefix: a.api_key.chars().take(8).collect(),
        })
        .collect();

    Ok(Json(account_infos))
}

/// Account information (without full API key).
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub id: String,
    pub provider_id: String,
    pub is_active: bool,
    pub priority: i32,
    pub api_key_prefix: String,
}

/// Returns current Unix timestamp.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
