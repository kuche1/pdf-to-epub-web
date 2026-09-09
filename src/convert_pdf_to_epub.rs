use std::{
    path::Path,
    process::{Command, Stdio},
};
use tempfile::TempPath;

use crate::util;

const PROGRAM: &str = "ebook-convert";

pub struct PdfToEpub {}

impl PdfToEpub {
    pub fn new() -> Result<Self, String> {
        let status = Command::new(PROGRAM)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(v) => {
                if !v.success() {
                    return Err(format!(
                        "Unsuccessfull return code for program `{}`",
                        PROGRAM
                    ));
                }
            }
            Err(e) => return Err(format!("Cannot find program `{}`: {}", PROGRAM, e)),
        };

        Ok(Self {})
    }

    pub fn main(&self, pdf: &Path) -> Result<TempPath, String> {
        if !util::suffix_is_pdf(pdf) {
            return Err("Pdf suffix is incorrect".to_string());
        }

        let output_epub = match util::generate_temp_file(".epub") {
            Ok(v) => v,
            Err(e) => return Err(format!("Could not create temporary `.epub`: {}", e)),
        };

        let status = Command::new(PROGRAM)
            .arg(pdf)
            .arg(&output_epub)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let status = match status {
            Ok(v) => v,
            Err(e) => return Err(format!("Could not convert pdf to epub: {}", e)),
        };

        if !status.success() {
            // TODO: maybe show the stdout
            return Err("Could not convert pdf to epub".to_string());
        }

        Ok(output_epub)
    }
}
