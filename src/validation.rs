use crate::models::{FieldError, SendRequest};

pub fn validate_send_request(request: &SendRequest) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();

    if !is_well_formed_email(&request.recipient_email) {
        errors.push(FieldError {
            field: "recipient_email",
            message: "Recipient email must be a well-formed email address.",
        });
    }

    if request.subject.trim().is_empty() {
        errors.push(FieldError {
            field: "subject",
            message: "Subject must not be empty.",
        });
    }

    if request.message.trim().is_empty() {
        errors.push(FieldError {
            field: "message",
            message: "Message must not be empty.",
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn is_well_formed_email(email: &str) -> bool {
    if email.trim() != email || email.is_empty() || email.chars().any(char::is_whitespace) {
        return false;
    }

    let mut parts = email.split('@');
    let Some(local_part) = parts.next() else {
        return false;
    };
    let Some(domain) = parts.next() else {
        return false;
    };

    if parts.next().is_some() || local_part.is_empty() || domain.is_empty() {
        return false;
    }

    if local_part.starts_with('.') || local_part.ends_with('.') || local_part.contains("..") {
        return false;
    }

    if !local_part.chars().all(is_valid_local_part_character) {
        return false;
    }

    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return false;
    }

    labels.iter().all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '-'
            })
    }) && labels
        .last()
        .is_some_and(|top_level_domain| top_level_domain.len() >= 2)
}

fn is_valid_local_part_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '-'
                | '/'
                | '='
                | '?'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
                | '.'
        )
}

#[cfg(test)]
mod tests {
    use super::validate_send_request;
    use crate::models::SendRequest;

    #[test]
    fn accepts_valid_payload() {
        let request = send_request("person@example.com", "Hello", "A short message");

        assert!(validate_send_request(&request).is_ok());
    }

    #[test]
    fn rejects_malformed_email() {
        let request = send_request("not-an-email", "Hello", "A short message");

        let errors = validation_errors(&request);

        assert!(errors
            .iter()
            .any(|error| error.field == "recipient_email"));
    }

    #[test]
    fn rejects_blank_subject_and_message() {
        let request = send_request("person@example.com", "  ", "\n\t");

        let errors = validation_errors(&request);

        assert!(errors.iter().any(|error| error.field == "subject"));
        assert!(errors.iter().any(|error| error.field == "message"));
    }

    fn send_request(recipient_email: &str, subject: &str, message: &str) -> SendRequest {
        SendRequest {
            recipient_email: recipient_email.to_string(),
            subject: subject.to_string(),
            message: message.to_string(),
        }
    }

    fn validation_errors(request: &SendRequest) -> Vec<crate::models::FieldError> {
        match validate_send_request(request) {
            Ok(()) => panic!("expected validation errors"),
            Err(errors) => errors,
        }
    }
}
