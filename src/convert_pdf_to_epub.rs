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

        let output = Command::new(PROGRAM)
            .arg(pdf)
            .arg(&output_epub)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let output = match output {
            Ok(v) => v,
            Err(e) => return Err(format!("Could not convert pdf to epub: {}", e)),
        };

        if !output.status.success() {
            //// this may leek some metadata on the server, such as the path to the executable
            // let stdout = String::from_utf8_lossy(&output.stdout);
            // let stderr = String::from_utf8_lossy(&output.stderr);
            // return Err(format!("Conversion failed:\n\n{}\n\n{}", stdout, stderr));

            return Err("Conversions failed, is the uploaded file really a pdf?".to_string());
        }

        Ok(output_epub)
    }
}
