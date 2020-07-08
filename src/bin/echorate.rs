use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{env, net};

use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use tokio::time;

static DEFAULT_ADDR: &str = "127.0.0.1:8080";
static COUNTER: AtomicUsize = AtomicUsize::new(0usize);

async fn index(_req: HttpRequest) -> impl Responder {
    format!("{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

async fn do_main<T: net::ToSocketAddrs>(addr: T) -> std::io::Result<()> {
    let handle = async {
        let mut instant = Instant::now();
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let rate = COUNTER.swap(0, Ordering::SeqCst) as f64
                / (instant.elapsed().as_millis() + 1) as f64
                * 1000.0;
            instant = Instant::now();
            println!("Rate: {:.3}/s", rate);
        }
    };

    let server = HttpServer::new(|| App::new().service(web::resource("/").to(index)))
        .bind(addr)?
        .run();
    tokio::select! {
        _ = handle => {
            println!("ticker finished");
        }
        _ = server => {
            println!("server finished");
        }
    }
    Ok(())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let addr = args.get(1).map(String::as_str).unwrap_or(DEFAULT_ADDR);

    println!("listening on {}", addr);
    do_main(addr).await
}
