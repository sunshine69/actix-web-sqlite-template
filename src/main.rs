use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, error, Error};
use actix_web::guard::Header;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use futures::StreamExt;

use serde::{Deserialize, Serialize};
use serde_json::{Value, Map, Number};

extern crate dotenv;
use dotenv::dotenv;

extern crate sqlite;
use sqlite::State;

#[path = "utils.rs"]
mod u;
// curl -X GET 'http://localhost:8080/log/2/sss' -H "Content-Type: multipart/form-data" -H 'X-Gitlab-Token: 1234'
#[get("/log/{log_id}/{field_name}")] // <- define path parameters
async fn getlog(web::Path((log_id, field_name)): web::Path<(i64, String)>) -> Result<String> {
    println!("GETLOG");
    let conn = u::get_dbconnection();
    let mut id: i64;
    let mut host: String = "".to_string();
    for row in conn.prepare("SELECT id, host FROM log WHERE id = ?").unwrap().into_iter()
    .bind((1, log_id)).unwrap().map(|row| row.unwrap() ){
        id = row.read::<i64, _>("id");
        host = row.read::<&str, _>("host").to_owned();
        println!("id = {}", id);
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

// example curl to savelog and use message field as the content of a text file mylog
// curl -X POST 'http://localhost:8080/savelog' -H "Content-Type: multipart/form-data" -H 'X-Gitlab-Token: 1234' -H "Content-Type: application/x-www-form-urlencoded" --data-urlencode 'logfile={"event": "started", "file": "codeception.yml", "error_code": -1}' --data-urlencode 'application={"event": "started", "file": "codeception.yml", "error_code": -1}' --data-urlencode 'host=test host' --data-urlencode "message=$(cat mylog)"
#[post("/savelog")]
async fn savelog(form: web::Form<Log>) -> Result<HttpResponse, Error> {
    let conn = u::get_dbconnection();
    let mut stmt = conn.prepare("INSERT INTO log(host, application, message, logfile) VALUES(?, ?, ?, ?)").unwrap();
    if let Err(err) = stmt.bind((1, form.host.as_str() )) { panic!("Error {}", err) }
    if let Err(err) = stmt.bind((2, form.application.as_str() )) { panic!("Error {}", err) }
    if let Err(err) = stmt.bind((3, form.message.as_str())) { panic!("Error {}", err) }
    if let Err(err) = stmt.bind((4, form.logfile.as_str())) { panic!("Error {}", err) }

    stmt.next().unwrap();
    Ok(HttpResponse::Ok().body("OK log saved"))
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k
#[post("/json/savelog")]
async fn savelog_json(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // let conn = u::get_dbconnection();
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
//Run a select SQL and dump the output. Poor man sql console
#[derive(Serialize, Deserialize)]
struct FormSql {
    sql: String,
}
#[post("/sql")]
async fn runsql(form: web::Form<FormSql>) -> Result<HttpResponse, Error> {
    let conn = u::get_dbconnection();
    let sql = form.sql.as_str();
    let mut statement = conn.prepare(sql).unwrap();
    let mut result_vec: Vec<Map<String, serde_json::Value>> = Vec::new();
    while let State::Row = statement.next().unwrap() {
        let mut _temp = Map::new();
        for colidx in 0..statement.column_count() {
            match statement.column_type(colidx).unwrap() {
                sqlite::Type::String => _temp.insert(statement.column_name(colidx).unwrap().to_string(), Value::String(statement.read::<String, _>(colidx).unwrap()) ),
                sqlite::Type::Integer => _temp.insert(statement.column_name(colidx).unwrap().to_string(), Value::Number( Number::from( statement.read::<i64,_>(colidx).unwrap()) ) ),
                sqlite::Type::Float => {
                    let _myval = Number::from_f64(  statement.read::<f64, _>(colidx).unwrap() ).unwrap();
                    _temp.insert(statement.column_name(colidx).unwrap().to_string(), Value::Number( _myval ) )
                },
                _ => Some(Value::Bool(false) ), //discarded other type
            };
        }
        result_vec.push(_temp);
    }
    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&result_vec).unwrap()))
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
    use u::*;
    setup_database();

    let server_port = get_env("SERVER_PORT", "8080");
    let server_string = format!("0.0.0.0:{}", server_port);

    let secret_header_string = get_env("SERVER_TOKEN", "1qa2ws");
    let secret_header = string_to_static_str(secret_header_string);
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
                .service(runsql)
                //Or we can remove the macro and directly use route here
                // .route("/savelog", web::post().to(savelog))
                // .route("/json/savelog", web::post().to(savelog_json))
                // .route("/getlog/{id}/{fname}", web::get().to(getlog))
        )
        // .route("/", web::to(|| HttpResponse::Ok()))
        .route("/container_status", web::get().to(container_status))
    });
    let ssl_key = get_env("SSL_KEY", "");
    let ssl_cert = get_env("SSL_CERT", "");
    if ssl_key != "" && ssl_cert != "" {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key_file(ssl_key, SslFiletype::PEM).unwrap();
        builder.set_certificate_chain_file(ssl_cert).unwrap();
        http_server.bind_openssl(server_string, builder)?.run().await
    } else {
        http_server.bind(server_string,)?.run().await
    }
}
