use crate::domain::{AuthError, SessionCookie, Username};
use async_trait::async_trait;

#[async_trait]
pub trait Authelia: Send + Sync {
    async fn first_factor(
        &self,
        username: &Username,
        password: &str,
    ) -> Result<SessionCookie, AuthError>;

    async fn second_factor_totp(
        &self,
        session: &SessionCookie,
        token: &str,
    ) -> Result<(), AuthError>;
}
