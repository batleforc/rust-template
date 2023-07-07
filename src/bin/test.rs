extern crate api;
use api::model;
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!("No .env file found"),
    }
    let oidc_handler = match model::oidc::Oidc::new() {
        Ok(oidc) => oidc,
        Err(e) => {
            println!("Error: {}", e);
            model::oidc::Oidc::new_disable()
        }
    };
    let token =
        "wXX3BiK_EOhl_ripSPKLW8O_71KhwUWTAFBg0EbYIEcE_QMEomsozj7bS9Yy8ZmEBeAfRB8".to_string();
    match oidc_handler
        .back
        .clone()
        .unwrap()
        .validate_token(token.clone())
    {
        Ok((token, _value)) => {
            if token {
                println!("token: {:?}", token)
            } else {
                println!("You are not authorized")
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    match oidc_handler.back.unwrap().get_user_info(token).await {
        Ok(user_info) => {
            if !user_info.is_null() {
                println!("user_info: {:?}", user_info)
            } else {
                println!("Error while getting userinfo");
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
