//! Helpers that keep credentials out of logs and API responses.

/// Mask the password part of a connection URL.
///
/// `postgres://user:s3cret@host:5432/db` becomes `postgres://user:***@host:5432/db`.
/// URLs without credentials, or with an empty password, are returned unchanged.
pub fn redact_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_owned();
    };
    let credentials_start = scheme_end + 3;
    let Some(at_offset) = url[credentials_start..].find('@') else {
        return url.to_owned();
    };
    let at = credentials_start + at_offset;
    let userinfo = &url[credentials_start..at];

    let Some(colon_offset) = userinfo.find(':') else {
        return url.to_owned();
    };
    let password_start = credentials_start + colon_offset + 1;

    if password_start >= at {
        return url.to_owned();
    }

    let mut redacted = String::with_capacity(url.len());
    redacted.push_str(&url[..password_start]);
    redacted.push_str("***");
    redacted.push_str(&url[at..]);
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_postgres_password() {
        assert_eq!(
            redact_url("postgres://cngokz:s3cret@localhost:5432/cngokzapi"),
            "postgres://cngokz:***@localhost:5432/cngokzapi"
        );
    }

    #[test]
    fn redacts_redis_password() {
        assert_eq!(
            redact_url("redis://:topsecret@redis:6379/0"),
            "redis://:***@redis:6379/0"
        );
    }

    #[test]
    fn keeps_urls_without_password_untouched() {
        assert_eq!(
            redact_url("redis://localhost:6379"),
            "redis://localhost:6379"
        );
        assert_eq!(
            redact_url("postgres://user@localhost/db"),
            "postgres://user@localhost/db"
        );
        assert_eq!(redact_url("not-a-url"), "not-a-url");
    }

    #[test]
    fn does_not_panic_on_empty_password() {
        assert_eq!(
            redact_url("redis://user:@localhost:6379"),
            "redis://user:@localhost:6379"
        );
    }
}
