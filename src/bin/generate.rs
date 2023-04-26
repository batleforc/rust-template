extern crate api;
use api::route::apidoc::ApiDoc;
use std::fs;
use utoipa::OpenApi;
fn main() {
    let openapi = ApiDoc::openapi();
    fs::write("./swagger.json", openapi.to_pretty_json().unwrap()).expect("Unable to write file");
}
