//! Request id propagation (`x-request-id`).
//!
//! Incoming ids are reused, missing ids are generated. The id is returned to the
//! caller so a log line can be correlated with a client report.

use axum::http::HeaderName;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn header_name() -> HeaderName {
    HeaderName::from_static(REQUEST_ID_HEADER)
}

/// Generates an id when the request does not carry one.
pub fn set_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    SetRequestIdLayer::new(header_name(), MakeRequestUuid)
}

/// Copies the id from the request onto the response.
pub fn propagate_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::new(header_name())
}
