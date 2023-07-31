use std::fmt::Display;

use actix_governor::{
    governor::{
        clock::{Clock, DefaultClock, QuantaInstant},
        NotUntil,
    },
    KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use serde::{Deserialize, Serialize};

/// Custom ErrorMessage for PerPathBearerTokenKeyExtractor
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ErrorMessage {
    pub code: u64,
    #[serde(skip_serializing_if = "is_default")]
    pub msg: String,
    #[serde(skip_serializing_if = "is_default")]
    pub wait_time_ms: u128,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// PerPathBearerTokenKeyExtractor is an actix_governor KeyExtractor that extracts the bearer token from Authorization cookie
/// together with the URI path as the key for the rate limiter. This allows the underlying rate limiter works seperately for each
/// (bearer_token, uri_path) pair.
#[derive(Clone)]
pub struct PerPathBearerTokenKeyExtractor;

impl KeyExtractor for PerPathBearerTokenKeyExtractor {
    /// The return key is (bearer_token, uri_path) for making the key different for each token and path.
    type Key = (String, String);

    type KeyExtractionError = SimpleKeyExtractionError<ErrorMessage>;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        let bearer_token = req
            .cookie("Authorization")
            .as_ref()
            .map(|token| token.value())
            .and_then(|token| token.strip_prefix("Bearer "))
            .map(|token| token.trim().to_owned())
            .ok_or_else(|| {
                Self::KeyExtractionError::new(ErrorMessage {
                    code: 401,
                    msg: "Invalid authorization token".to_owned(),
                    ..Default::default()
                })
                .set_content_type(ContentType::json())
                .set_status_code(StatusCode::UNAUTHORIZED)
            })?;
        let path = req.path();
        Ok((path.to_owned(), bearer_token))
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait_time = negative
            .wait_time_from(DefaultClock::default().now())
            .as_millis();
        response
            .content_type(ContentType::json())
            .json(&ErrorMessage {
                code: 429,
                msg: "too many retries".to_owned(),
                wait_time_ms: wait_time,
            })
    }
}

#[cfg(test)]
mod test {
    use actix_governor::KeyExtractor;
    use actix_web::{cookie::Cookie, http::header::ContentType, test::TestRequest};

    use crate::key_extractor::ErrorMessage;

    use super::PerPathBearerTokenKeyExtractor;

    #[test]
    fn test_per_path_bearer_token_key_extractor_extract() {
        let extractor = PerPathBearerTokenKeyExtractor;
        let req = TestRequest::default()
            .cookie(Cookie::new("Authorization", "Bearer AAAAAA"))
            .uri("/path")
            .to_srv_request();
        let resp = extractor.extract(&req);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), ("/path".to_string(), "AAAAAA".to_string()));

        let req = TestRequest::default()
            .cookie(Cookie::new("Authorization", "Bearer BBBBBB\r\n"))
            .uri("/path")
            .to_srv_request();
        let resp = extractor.extract(&req);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap(), ("/path".to_string(), "BBBBBB".to_string()));

        // Invalid Authorization cookie
        let req = TestRequest::default()
            .cookie(Cookie::new("Authorization", "AAAAAA"))
            .uri("/path")
            .to_srv_request();
        let resp = extractor.extract(&req);
        assert!(resp.is_err());
        let error = resp.unwrap_err();
        assert_eq!(error.content_type, ContentType::json());
        assert_eq!(
            error.body,
            ErrorMessage {
                code: 401,
                msg: "Invalid authorization token".to_owned(),
                wait_time_ms: 0
            }
        );
        assert_eq!(error.status_code, 401);

        // No Authorization cookie
        let req = TestRequest::default().uri("/path").to_srv_request();
        let resp = extractor.extract(&req);
        assert!(resp.is_err());
        let error = resp.unwrap_err();
        assert_eq!(error.content_type, ContentType::json());
        assert_eq!(
            error.body,
            ErrorMessage {
                code: 401,
                msg: "Invalid authorization token".to_owned(),
                wait_time_ms: 0
            }
        );
        assert_eq!(error.status_code, 401);
    }
}
