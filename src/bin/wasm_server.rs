use std::{net::SocketAddr, path::Path};

use axum::{
    Router,
    body::Body,
    http::{
        Request, Response,
        header::{CONTENT_LENGTH, CONTENT_TYPE},
    },
};
use clap::Parser;
use mime_guess::Mime;
use tokio::io::BufReader;
use tokio::{fs, net::TcpListener};
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

const ASSETS_DIR: &str = "./www/";

#[derive(clap::Parser)]
struct MArgs {
    #[clap(short, long, help = "Bind address", default_value_t = {"0.0.0.0:8000".parse().unwrap()})]
    bind: SocketAddr,
}

fn not_found_response() -> Response<Body> {
    Response::builder().status(404).body(Body::empty()).unwrap()
}

// const fn map_mime(ext: &str) -> Option<&str> {
//     match ext {
//         _ => None,
//     }
// }

async fn files(req: Request<Body>) -> Response<Body> {
    let file_path = Path::new(ASSETS_DIR).join(req.uri().path().trim_start_matches('/'));
    info!("file req: {file_path:?}.");
    if let Ok(metadata) = file_path.metadata()
        && metadata.is_file()
    {
        let Ok(file) = fs::OpenOptions::new().read(true).open(&file_path).await else {
            warn!("cannot open file.");
            return not_found_response();
        };

        let reader = BufReader::new(file);
        let rs = ReaderStream::new(reader);

        info!("file {:?} response, len: {}.", file_path, metadata.len());
        Response::builder()
            .status(200)
            .header(
                CONTENT_TYPE,
                mime_guess::from_path(file_path)
                    .first_or_octet_stream()
                    .essence_str(),
            )
            .header(CONTENT_LENGTH, metadata.len())
            .body(Body::from_stream(rs))
            .unwrap_or_else(|_| not_found_response())
    } else {
        warn!("not found or not file.");
        not_found_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let args = MArgs::parse();

    let listener = TcpListener::bind(args.bind).await.unwrap();
    let app = Router::new().fallback(files);
    info!("WASM server start at: http://{}", args.bind);
    axum::serve(listener, app).await.unwrap();
}
