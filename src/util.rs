use humansize::{BINARY, format_size};
use std::path::Path;
use tempfile::Builder;
use tempfile::TempPath;

pub fn suffix_is_pdf<P: AsRef<Path>>(path: P) -> bool {
    suffix_is(path, "pdf")
}

fn suffix_is<P: AsRef<Path>>(path: P, suffix: &str) -> bool {
    path.as_ref() // this SHOULD cast a `&str` into `&Path` literally for free
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext.eq_ignore_ascii_case(suffix))
}

pub fn generate_temp_file(suffix: &str) -> Result<TempPath, String> {
    let output_epub = match Builder::new().suffix(suffix).tempfile() {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    Ok(output_epub.into_temp_path()) // TODO: supposedly this automatically deletes the temporary file when the variable gets dropped, need to test this
}

#[allow(non_snake_case)]
pub fn format_XiB(unformatted: usize) -> String {
    format_size(unformatted, BINARY)
}
