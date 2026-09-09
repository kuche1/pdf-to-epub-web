mod convert_pdf_to_epub;
mod util;
mod web;

use clap::Parser;

/// Web interface for converting pdf to epub
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// Address to run the server on
    #[arg(long, default_value_t = "127.0.0.1:3000".to_string())]
    addr: String,

    /// Number of times to greet
    #[arg(long, default_value_t = 1024 * 1024 * 1)] // 1 MiB
    request_limit_bytes: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    web::main(&args.addr, args.request_limit_bytes).await
}
