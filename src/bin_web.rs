#[tokio::main]
async fn main() {
    if let Err(error) = threecx_log_explorer::web::run().await {
        eprintln!("Falha ao iniciar servidor web: {error}");
        std::process::exit(1);
    }
}
