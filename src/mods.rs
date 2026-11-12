use std::path::PathBuf;

pub async fn download_from_github() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let url = inquire::Text::new("Enter the GitHub URL to download from or another site:")
        .prompt()
        .expect("Failed to read input");
    let response = reqwest::get(url).await?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());
    let content_disposition = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok());
    let filename = downloaded_filename(response.url(), content_type, content_disposition);
    let bytes = response.bytes().await?;
    let path = PathBuf::from(filename);
    std::fs::write(&path, &bytes)?;
    Ok(path)
}

fn downloaded_filename(
    url: &reqwest::Url,
    content_type: Option<&str>,
    content_disposition: Option<&str>,
) -> String {
    let filename = content_disposition
        .and_then(content_disposition_filename)
        .and_then(|name| safe_filename(&name))
        .or_else(|| {
            url.path_segments()
                .and_then(|mut segments| segments.next_back())
                .and_then(percent_decode)
                .and_then(|name| safe_filename(&name))
        })
        .unwrap_or_else(|| "download".to_owned());
    let extension = file_extension(url, content_type);
    let is_html = extension == "html";

    if is_html {
        let current_extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|extension| extension.to_str());
        if !current_extension.is_some_and(|ext| ext.eq_ignore_ascii_case("html")) {
            let mut filename = PathBuf::from(filename);
            filename.set_extension("html");
            return filename.to_string_lossy().into_owned();
        }
    } else if std::path::Path::new(&filename).extension().is_none() {
        return format!("{filename}.{extension}");
    }

    filename
}

fn content_disposition_filename(header: &str) -> Option<String> {
    let mut parts = Vec::new();
    let (mut start, mut quoted, mut escaped) = (0, false, false);

    for (index, character) in header.char_indices() {
        if escaped {
            escaped = false;
        } else {
            match character {
                '\\' if quoted => escaped = true,
                '"' => quoted = !quoted,
                ';' if !quoted => {
                    parts.push(&header[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
    }
    parts.push(&header[start..]);

    let mut filename = None;
    for part in parts.into_iter().skip(1) {
        let Some((key, value)) = part.trim().split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("filename*") {
            if let Some(decoded) = decode_extended_filename(value.trim()) {
                return Some(decoded);
            }
        } else if key.trim().eq_ignore_ascii_case("filename") {
            filename = Some(unquote(value.trim()));
        }
    }
    filename
}

fn decode_extended_filename(value: &str) -> Option<String> {
    let mut parts = value.splitn(3, '\'');
    let charset = parts.next()?;
    let _language = parts.next()?;
    let encoded = parts.next()?;

    if !charset.eq_ignore_ascii_case("utf-8") {
        return None;
    }

    percent_decode(encoded)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

fn unquote(value: &str) -> String {
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value);
    let mut unquoted = String::with_capacity(value.len());
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            unquoted.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            unquoted.push(character);
        }
    }

    if escaped {
        unquoted.push('\\');
    }
    unquoted
}

fn safe_filename(filename: &str) -> Option<String> {
    let filename = filename
        .rsplit(|character| character == '/' || character == '\\')
        .next()?
        .trim();

    if filename.is_empty()
        || filename == "."
        || filename == ".."
        || filename.chars().any(char::is_control)
    {
        None
    } else {
        Some(filename.to_owned())
    }
}

fn file_extension(url: &reqwest::Url, content_type: Option<&str>) -> String {
    let media_type = content_type
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .unwrap_or_default();

    if media_type.eq_ignore_ascii_case("text/html") {
        return "html".to_owned();
    }

    if let Some(extension) = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .and_then(|name| std::path::Path::new(name).extension())
        .and_then(|extension| extension.to_str())
    {
        return extension.to_owned();
    }

    match media_type.to_ascii_lowercase().as_str() {
        "application/pdf" => "pdf".to_owned(),
        "application/zip" | "application/x-zip-compressed" => "zip".to_owned(),
        _ => "bin".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{downloaded_filename, file_extension};

    #[test]
    fn html_content_type_uses_html_extension() {
        let url = reqwest::Url::parse("https://example.com/page").unwrap();

        assert_eq!(
            file_extension(&url, Some("text/html; charset=utf-8")),
            "html"
        );
    }

    #[test]
    fn file_url_extension_is_preserved() {
        let url = reqwest::Url::parse("https://example.com/archive.zip?download=1").unwrap();

        assert_eq!(
            file_extension(&url, Some("application/octet-stream")),
            "zip"
        );
    }

    #[test]
    fn pdf_content_type_supplies_missing_extension() {
        let url = reqwest::Url::parse("https://example.com/download").unwrap();

        assert_eq!(file_extension(&url, Some("application/pdf")), "pdf");
    }

    #[test]
    fn content_disposition_filename_is_preserved() {
        let url = reqwest::Url::parse("https://example.com/download").unwrap();

        assert_eq!(
            downloaded_filename(
                &url,
                Some("application/pdf"),
                Some("attachment; filename=\"annual report.pdf\"")
            ),
            "annual report.pdf"
        );
    }

    #[test]
    fn encoded_url_filename_is_preserved() {
        let url = reqwest::Url::parse("https://example.com/releases/my%20archive.zip").unwrap();

        assert_eq!(
            downloaded_filename(&url, Some("application/zip"), None),
            "my archive.zip"
        );
    }

    #[test]
    fn extended_content_disposition_filename_is_decoded() {
        let url = reqwest::Url::parse("https://example.com/download").unwrap();

        assert_eq!(
            downloaded_filename(
                &url,
                Some("application/pdf"),
                Some("attachment; filename*=UTF-8''rapport%20annuel.pdf")
            ),
            "rapport annuel.pdf"
        );
    }
}
