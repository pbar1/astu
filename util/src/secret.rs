use redact::Secret;
use zeroize::Zeroizing;

/// String that is redacted when printed and zeroed when it goes out of scope.
pub struct SecureString(Zeroizing<Secret<String>>);

impl<S> From<S> for SecureString
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        SecureString(Zeroizing::new(Secret::new(value.as_ref().into())))
    }
}
