use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::fmt;

// Custom error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
}

impl ErrorResponse {
    pub fn new(message: &str) -> Self {
        Self {
            error: ErrorDetails {
                message: message.to_string(),
                r#type: None,
                param: None,
                code: None,
            },
        }
    }

    pub fn with_type(message: &str, error_type: &str) -> Self {
        Self {
            error: ErrorDetails {
                message: message.to_string(),
                r#type: Some(error_type.to_string()),
                param: None,
                code: None,
            },
        }
    }

    pub fn with_code(message: &str, code: i32) -> Self {
        Self {
            error: ErrorDetails {
                message: message.to_string(),
                r#type: None,
                param: None,
                code: Some(code),
            },
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

// For specific error status codes, we can create a wrapper or use a different approach.
// But the requirement is to have a custom error response format.
// We'll also implement a way to set the status code.

#[derive(Debug)]
pub struct ErrorResponseWithStatus {
    pub status: StatusCode,
    pub inner: ErrorResponse,
}

impl ErrorResponseWithStatus {
    pub fn new(status: StatusCode, message: &str) -> Self {
        Self {
            status,
            inner: ErrorResponse::new(message),
        }
    }

    pub fn with_type(status: StatusCode, message: &str, error_type: &str) -> Self {
        Self {
            status,
            inner: ErrorResponse::with_type(message, error_type),
        }
    }

    pub fn with_code(status: StatusCode, message: &str, code: i32) -> Self {
        Self {
            status,
            inner: ErrorResponse::with_code(message, code),
        }
    }
}

impl IntoResponse for ErrorResponseWithStatus {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.inner)).into_response()
    }
}

// SSE Event format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<u32>,
}

impl SseEvent {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            data: Some(data.into()),
            event: None,
            id: None,
            retry: None,
        }
    }

    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn retry(mut self, retry: u32) -> Self {
        self.retry = Some(retry);
        self
    }
}

// Implement IntoResponse for SseEvent so it can be used directly in handlers
// However, Axum's Sse type expects a stream of SseEvent.
// We'll leave the conversion to the handler, but we can provide a helper.

impl From<SseEvent> for axum::response::sse::Event {
    fn from(event: SseEvent) -> Self {
        let mut e = axum::response::sse::Event::default();
        if let Some(data) = event.data {
            e = e.data(data);
        }
        if let Some(event) = event.event {
            e = e.event(event);
        }
        if let Some(id) = event.id {
            e = e.id(id);
        }
        if let Some(retry) = event.retry {
            e = e.retry(retry);
        }
        e
    }
}
