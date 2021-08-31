use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, error, Error};
use actix_web::guard::Header;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use futures::StreamExt;
use serde::{Deserialize, Serialize};

extern crate dotenv;
use dotenv::dotenv;
use std::env;

extern crate sqlite;
mod utils;

#[get("/log/{log_id}/{field_name}")] // <- define path parameters
async fn getlog(web::Path((log_id, field_name)): web::Path<(i64, String)>) -> Result<String> {
    println!("GETLOG");
    use sqlite::Value;
    let conn = utils::get_dbconnection();
    let mut cursor = conn.prepare("SELECT id, host FROM log WHERE id = :id").unwrap().into_cursor();
    cursor.bind_by_name(vec![(":id", Value::Integer(log_id))]).unwrap();
    let mut host: String = "".to_string();
    let mut id: i64;
    while let Some(row) = cursor.next().unwrap() {
        id = row[0].as_integer().unwrap();
        host = row[1].as_string().unwrap().to_string();
        println!("name = {}", id);
        println!("host = {}", host);
    }
    Ok(format!("Welcome {log_id}, host {host}! field_name {fname}", log_id=log_id, host=host, fname=field_name))
}

#[derive(Serialize, Deserialize)]
struct Log {
    host: String,
    application: String,
    message: String,
    logfile: String
}
const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/savelog")]
async fn savelog(form: web::Form<Log>) -> Result<HttpResponse, Error> {
    let conn = utils::get_dbconnection();
    let mut stmt = conn.prepare("INSERT INTO log(host, application, message, logfile) VALUES(?, ?, ?, ?)").unwrap();
    stmt.bind(1, form.host.as_str()).unwrap();
    stmt.bind(2, form.application.as_str()).unwrap();
    stmt.bind(3, form.message.as_str()).unwrap();
    stmt.bind(4, form.logfile.as_str()).unwrap();
    stmt.next().unwrap();

    Ok(HttpResponse::Ok().body("OK log saved"))
}

#[post("/json/savelog")]
async fn savelog_json(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // let conn = utils::get_dbconnection();
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Log>(&body)?;
    Ok(HttpResponse::Ok().json(obj))
    // Ok(HttpResponse::Ok().json("{\"status\": \"OK\"}")) // <- send response
}

async fn container_status() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // for (key, value) in env::vars() {
    //     println!("{}: {}", key, value);
    // }
    utils::setup_database();
    let server_string: String;

    if let Ok(server_port) = env::var("SERVER_PORT") {
        server_string = format!("0.0.0.0:{}", server_port);
    } else {
        server_string = "0.0.0.0:8080".to_string();
    }
    let secret_header: &'static str;
    if let Ok(_secret_header) = env::var("SERVER_TOKEN") {
        println!("[INFO] found SERVER_TOKEN {}", _secret_header);
        secret_header = string_to_static_str(_secret_header);
    } else {
        secret_header = "1qa2ws";
    }

    println!("server_string is {}", server_string);
    let http_server = HttpServer::new(move || {
        App::new()
        .service(
            web::scope("/")
                .guard(Header("X-Gitlab-Token", secret_header))
                //Use service so we can use the macro at the handler implementation
                .service(savelog)
                .service(getlog)
                .service(savelog_json)
                //Or we can remove the macro and directly use route here
                // .route("/savelog", web::post().to(savelog))
                // .route("/json/savelog", web::post().to(savelog_json))
                // .route("/getlog/{id}/{fname}", web::get().to(getlog))
        )
        // .route("/", web::to(|| HttpResponse::Ok()))
        .route("/container_status", web::get().to(container_status))
    });
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    if let Ok(ssl_key) = env::var("SSL_KEY") {
        if ssl_key != "" {
            builder.set_private_key_file(ssl_key, SslFiletype::PEM).unwrap();
            if let Ok(ssl_cert) = env::var("SSL_CERT") {
                builder.set_certificate_chain_file(ssl_cert).unwrap();
                http_server.bind_openssl(server_string, builder)?.run().await
            } else {
                http_server.bind(server_string,)?.run().await
            }
        } else {
            http_server.bind(server_string,)?.run().await
        }
    } else {
        http_server.bind(server_string,)?.run().await
    }
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
