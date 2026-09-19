use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {}

impl TryFrom<&str> for Email {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        anyhow::ensure!(value.contains("@"), "Wrong e-mail address.");

        Ok(Email(value.to_string()))
    }
}

impl From<Email> for String {
    fn from(val: Email) -> Self {
        val.0
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::Email;

    #[test]
    fn email_string_ok() {
        let email_str = "test@test.com";
        let email: Result<Email, _> = email_str.try_into();
        assert!(email.is_ok());

        let email: String = email.unwrap().into();
    }

    #[test]
    fn email_string_err() {
        let email_str = "test.com";
        let email: Result<Email, _> = email_str.try_into();
        assert!(email.is_err());

        let email: String = email.unwrap_err().to_string();
    }
}
