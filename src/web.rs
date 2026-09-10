use axum::body::Bytes;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::{
    Router,
    extract::{Multipart, State},
    response::Html,
    routing::{get, post},
};

use crate::convert_pdf_to_epub::PdfToEpub;
use crate::util;
use maud::html;
use std::sync::Arc;

struct AppState {
    pdf_to_epub: PdfToEpub,
    request_limit_bytes: usize,
}

impl AppState {
    fn new(request_limit_bytes: usize) -> Result<Self, String> {
        let pdf_to_epub = match PdfToEpub::new() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                return Err(e);
            }
        };

        Ok(Self {
            pdf_to_epub: pdf_to_epub,
            request_limit_bytes: request_limit_bytes,
        })
    }
}

pub async fn main(addr: &str, request_limit_bytes: usize) {
    let shared_state = AppState::new(request_limit_bytes);

    let app = {
        match shared_state {
            Ok(ss) => Router::new()
                .route("/", get(handle_main))
                .route("/upload", post(handle_upload))
                .with_state(Arc::new(ss)),
            Err(e) => Router::new()
                .route("/", get(handle_main_error))
                .with_state(e),
        }
    };

    let app = app.layer(DefaultBodyLimit::max(request_limit_bytes));

    println!("Binding on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_main() -> Html<String> {
    let markup = html! {
        script src="https://unpkg.com/htmx.org@2.0.10" {}
        h1 { "Convert pdf to epub" }
        form action="/upload" method="post" enctype="multipart/form-data" {
            input type="file" name="myfile";
            button type="submit" { "Upload & Convert" }
        }
    };
    Html(markup.into_string())
}

async fn handle_upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut downloaded_pdf_name = String::from("downloaded_file");
    let mut downloaded_pdf_data: Option<Bytes> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.file_name() {
            downloaded_pdf_name = name.to_string();
        }
        if let Ok(bytes) = field.bytes().await {
            downloaded_pdf_data = Some(bytes);
            break;
        }
    }

    let downloaded_pdf_data = match downloaded_pdf_data {
        Some(data) => data,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "No file uploaded OR file size (including request body data) exceeded the limit of {}",
                    util::format_XiB(state.request_limit_bytes)
                ),
            ));
        }
    };

    //////////

    if !util::suffix_is_pdf(&downloaded_pdf_name) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "You need to upload a pdf".to_string(), // this shows up if you select "upload" without having selected a file
        ));
    }

    let new_len = downloaded_pdf_name.len().saturating_sub(4);
    let output_epub_name = format!("{}.epub", &downloaded_pdf_name[..new_len]);

    let downloaded_pdf = match util::generate_temp_file(".pdf") {
        Ok(v) => v,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", e),
            ));
        }
    };

    //////////
    // download to disk

    tokio::fs::write(&downloaded_pdf, &downloaded_pdf_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    //////////

    let output_epub = match state.pdf_to_epub.main(&downloaded_pdf) {
        Ok(v) => v,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    };

    let output_epub_bytes = tokio::fs::read(&output_epub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    //////////
    // set the required headers

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", &output_epub_name)
            .parse()
            .unwrap(),
    );

    //////////
    // send the actual data

    Ok((headers, output_epub_bytes))
}

async fn handle_main_error(State(error): State<String>) -> Html<String> {
    let markup = html! {
        script src="https://unpkg.com/htmx.org@2.0.10" {}
        h1 { "Internal Server Error" }
        pre { (error) } // `pre` preserves `\n`
    };
    Html(markup.into_string())
}
