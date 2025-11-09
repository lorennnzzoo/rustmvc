use rustmvc::*;
use RouteRules::*;

use crate::config::get_auth_config;

mod config {
    use std::sync::Arc;

    use rustmvc::authentication::AuthConfig;

    pub fn get_auth_config() -> Arc<AuthConfig> {
        Arc::new(AuthConfig::new("123456789"))
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    server.add_middleware(move |mut ctx, next| {
        let auth_config = get_auth_config();
        if ctx.rules.contains(&Authorize) {
            match ctx.headers.get("Authorization") {
                Some(auth_header) => {
                    let token = auth_header.to_str().unwrap_or("").replace("Bearer ", "");
                    match auth_config.validate_token(&token) {
                        Ok(token_data) => {
                            ctx.user = Some(User {
                                name: token_data.claims.sub,
                                roles: token_data.claims.roles,
                            });
                            next(ctx)
                        }
                        Err(_) => ActionResult::UnAuthorized("Invalid token".into()),
                    }
                }
                None => ActionResult::UnAuthorized("Missing token".into()),
            }
        } else {
            next(ctx)
        }
    });
    server.add_route(
        "/login",
        providers::custom_provider,
        HttpMethod::POST,
        vec![AllowAnonymous],
    );
    server.add_route("/", routes::home, HttpMethod::GET, vec![Authorize]);
    server.start("127.0.0.1:8080").await
}

mod mock_database {
    pub struct User {
        pub username: String,
        pub password: String,
    }

    impl User {
        pub fn mock_users() -> Vec<User> {
            (0..10)
                .map(|i| User {
                    username: format!("user{}", i.to_string()),
                    password: "12345678".to_string(),
                })
                .collect()
        }

        pub fn get(username: String) -> Option<User> {
            User::mock_users()
                .into_iter()
                .find(|user| user.username == username)
        }
    }
}

mod routes {
    use rustmvc::{ActionResult, RequestContext};

    pub fn home(ctx: RequestContext) -> ActionResult {
        if let Some(user) = &ctx.user {
            ActionResult::Ok(format!("Hello, {}!", user.name))
        } else {
            ActionResult::Ok("Hello, anonymous!".to_string())
        }
    }
}

mod providers {
    use rustmvc::{ActionResult, RequestContext};

    use crate::{config::get_auth_config, mock_database};
    pub fn custom_provider(ctx: RequestContext) -> ActionResult {
        match (ctx.params.get("username"), ctx.params.get("password")) {
            (Some(u), Some(p)) => {
                let user = mock_database::User::get(u.to_string());
                match user {
                    Some(user) => {
                        if user.password == *p {
                            let auth_config = get_auth_config();

                            let token =
                                auth_config.generate_token(&user.username, vec!["user".into()], 60);

                            return ActionResult::Ok(format!("{:?}", token));
                        } else {
                            return ActionResult::BadRequest(
                                "username and password not valid".into(),
                            );
                        }
                    }
                    None => {
                        return ActionResult::BadRequest("username and password not valid".into())
                    }
                }
            }

            (None, Some(_)) => ActionResult::BadRequest("username field required".into()),

            (Some(_), None) => ActionResult::BadRequest("password field required".into()),

            (None, None) => {
                ActionResult::BadRequest("username and password fields required".into())
            }
        }
    }
}
