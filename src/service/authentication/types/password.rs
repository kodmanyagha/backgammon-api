use std::fmt::Display;

use anyhow::anyhow;
use entity::utils::password_helper::{hash_password, verify_password};

#[derive(Clone, Debug, Default)]
pub struct Password(String);

impl Password {}

impl TryFrom<&str> for Password {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().len() < 6 {
            return Err(anyhow!("Password must be minimum 6 characters."));
        }
        let value = value.trim().to_string();

        Ok(Password(hash_password(&value)?))
    }
}

impl TryFrom<String> for Password {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Password::try_from(value.as_str())
    }
}

impl From<Password> for String {
    fn from(val: Password) -> Self {
        val.0
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        verify_password(&other.0, &self.0)
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::Password;
    use anyhow::Error;
    use entity::utils::password_helper::verify_password;

    #[test]
    fn password_ok_1() {
        let password_str = "123456";
        let password: Result<Password, Error> = match Password::try_from(password_str) {
            Ok(password) => {
                assert!(verify_password(password_str, &password.to_string()));
                Ok(password)
            }
            Err(err) => {
                log::info!(">>>>>>>>>> {password_str}, {}", err);
                Err(err)
            }
        };

        dbg!(password_str);
        assert!(password.is_ok());
    }

    #[test]
    fn password_err_1() {
        let password_str = "12345";
        let password: Result<Password, _> = password_str.try_into();
        assert!(password.is_err());
    }
}
