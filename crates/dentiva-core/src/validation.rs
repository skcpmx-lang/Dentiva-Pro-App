use crate::error::{invalid, Result};
use chrono::{NaiveDate, Utc};

pub(crate) fn text(value: &str, max: usize, required: bool) -> Result<String> {
    let value = value.trim();
    if (required && value.is_empty()) || value.chars().count() > max
        || value.chars().any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(invalid("A text field is empty, too long, or contains unsupported control characters."));
    }
    Ok(value.to_owned())
}

pub(crate) fn username(value: &str) -> Result<String> {
    let value = value.trim().to_ascii_lowercase();
    if !(3..=64).contains(&value.len())
        || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(invalid("Use 3–64 letters, digits, dots, underscores or hyphens for the username."));
    }
    Ok(value)
}

pub(crate) fn password(value: &str) -> Result<()> {
    if value.chars().filter(|c| !c.is_whitespace()).count() < 12 || value.chars().count() > 128 {
        return Err(invalid("Use a password with at least 12 non-space characters and at most 128 characters."));
    }
    Ok(())
}

pub(crate) fn phone(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() { return Ok(String::new()); }
    if trimmed.len() > 32 || !trimmed.bytes().all(|b| b.is_ascii_digit() || b" +-()".contains(&b)) {
        return Err(invalid("Enter a valid phone number, including country code if needed."));
    }
    let digits: String = trimmed.chars().filter(char::is_ascii_digit).collect();
    if !(7..=15).contains(&digits.len()) || trimmed.matches('+').count() > 1
        || (trimmed.contains('+') && !trimmed.starts_with('+')) {
        return Err(invalid("A phone number must contain 7–15 digits."));
    }
    Ok(if trimmed.starts_with('+') { format!("+{digits}") } else { digits })
}

pub(crate) fn birth_date(value: &Option<String>) -> Result<()> {
    if let Some(value) = value {
        let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| invalid("Enter a valid date of birth."))?;
        if date.format("%Y-%m-%d").to_string() != *value || date > Utc::now().date_naive()
            || date < NaiveDate::from_ymd_opt(1850,1,1).expect("valid boundary") {
            return Err(invalid("Date of birth must be between 1850 and today."));
        }
    }
    Ok(())
}

pub(crate) fn request_key(value: &str) -> Result<()> {
    uuid::Uuid::parse_str(value).map(|_| ()).map_err(|_| invalid("The request identifier is invalid. Reopen the form."))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_and_controls() {
        assert_eq!(text("  রোগীর নাম  ",200,true).unwrap(), "রোগীর নাম");
        assert!(text("\0",20,false).is_err());
        assert!(text("  ",20,true).is_err());
    }
    #[test]
    fn validates_identity_inputs() {
        assert_eq!(phone("+880 1712-345678").unwrap(), "+8801712345678");
        for p in ["123", "1234567+", "1234567abc", "++1234567"] { assert!(phone(p).is_err()); }
        assert!(birth_date(&Some("2025-02-30".into())).is_err());
        assert!(birth_date(&Some("2999-01-01".into())).is_err());
        assert!(username("a' OR 1=1").is_err());
        assert!(password("            ").is_err());
    }
}
