use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use tokio::time;

static COUNTER: AtomicUsize = AtomicUsize::new(0usize);

async fn index(_req: HttpRequest) -> impl Responder {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    let counter = COUNTER.load(Ordering::SeqCst);
    format!("{}", counter)
}

async fn do_main(addr: &str) -> std::io::Result<()> {
    let handle = async {
        let mut instant = time::Instant::now();
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let rate = COUNTER.swap(0, Ordering::SeqCst) as f64
                / (instant.elapsed().as_millis() + 1) as f64
                * 1000.0;
            instant = time::Instant::now();
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

    do_main(&args[1]).await
}
