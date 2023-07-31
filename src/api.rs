use actix_web::{web::Path, HttpResponse, Responder};

pub async fn post_vault() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn get_vault_items() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn put_vault_item(_path: Path<String>) -> impl Responder {
    HttpResponse::Ok()
}
