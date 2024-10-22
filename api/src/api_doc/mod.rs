use crate::routers::{accounts, auth, transactions, users};
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::authorize_user,
        users::get_users,
        users::get_user,
        users::create_user,
        users::update_user,
        users::delete_user,
        accounts::get_accounts,
        accounts::get_account,
        accounts::create_account,
        accounts::delete_account,
        transactions::get_account_transactions,
        transactions::create_account_transaction,
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}
