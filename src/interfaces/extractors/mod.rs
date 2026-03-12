use axum::{
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
};
use std::convert::Infallible;

// Extractor for API key from Authorization header
#[derive(Debug, Clone)]
pub struct ApiKeyExtractor(pub String);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for ApiKeyExtractor
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract the Authorization header
        if let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state).await.ok() {
            return Ok(ApiKeyExtractor(bearer.token().to_string()));
        }
        
        // If not found, return empty string (or you could return an error, but we'll let handlers handle missing keys)
        Ok(ApiKeyExtractor(String::new()))
    }
}

// Extractor for application state (already provided by Axum's State, but we can create a wrapper if needed)
// Actually, Axum's State extractor is sufficient, so we don't need a custom one for state.
// However, if we want to extract a specific part of the state, we can do so.
// For simplicity, we'll just use Axum's State extractor directly in handlers.
// But let's create a StateExtractor that extracts the entire state for consistency with the requirement.

use axum::extract::State;

// We'll re-export State as StateExtractor for clarity, but it's the same.
// Alternatively, we can create a wrapper if we want to add logic, but for now, we'll just use State.
// Since the requirement says "StateExtractor: extrae estado de la aplicación", we can define:

pub type StateExtractor<T> = State<T>;

// However, to strictly follow the requirement, we'll create a newtype wrapper if needed.
// But Axum's State is already an extractor. Let's just use it and note in comments.

// If we really want a custom extractor for state (though unnecessary), we could do:
// But it's redundant. We'll leave it as a comment and use State directly.

// For the purpose of the exercise, we'll define StateExtractor as an alias.