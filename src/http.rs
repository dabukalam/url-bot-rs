extern crate curl;
extern crate htmlescape;

use self::curl::easy::{Easy2, Handler, WriteError, List};
use self::htmlescape::decode_html;
use std::time::Duration;
use itertools::Itertools;
use regex::Regex;

#[derive(Debug)]
struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

pub fn resolve_url(url: &str, lang: &str) -> Option<String> {
    eprintln!("RESOLVE {}", url);

    let mut easy = Easy2::new(Collector(Vec::new()));

    easy.get(true).unwrap();
    easy.url(url).unwrap();
    easy.follow_location(true).unwrap();
    easy.max_redirections(10).unwrap();
    easy.timeout(Duration::from_secs(5)).unwrap();
    easy.max_recv_speed(10 * 1024 * 1024).unwrap();
    easy.useragent("url-bot-rs/0.1").unwrap();

    let mut headers = List::new();
    let lang = format!("Accept-Language: {}", lang);
    headers.append(&lang).unwrap();
    easy.http_headers(headers).unwrap();

    easy.perform().ok()?;

    let contents = String::from_utf8_lossy(&easy.get_ref().0);

    parse_content(&contents)
}

fn parse_content(page_contents: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new("<title>((.|\n)*?)</title>").unwrap();
    }
    let title_enc = RE.captures(page_contents)?.get(1)?.as_str();
    let title_dec = decode_html(title_enc).ok()?;

    // make any multi-line title string into a single line,
    // trim leading and trailing whitespace
    let title_one_line = title_dec
        .trim()
        .lines()
        .map(|line| line.trim())
        .join(" ");

    if title_one_line.is_empty() {
        return None;
    }

    eprintln!("SUCCESS \"{}\"", title_one_line);
    Some(title_one_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_urls() {
        assert_ne!(None, resolve_url("https://youtube.com", "en"));
        assert_ne!(None, resolve_url("https://google.co.uk", "en"));
    }

    #[test]
    fn parse_contents() {
        assert_eq!(None, parse_content(&"".to_string()));
        assert_eq!(None, parse_content(&"    ".to_string()));
        assert_eq!(None, parse_content(&"<title></title>".to_string()));
        assert_eq!(None, parse_content(&"<title>    </title>".to_string()));
        assert_eq!(None,
             parse_content(&"floofynips, not a real webpage".to_string()));
        assert_eq!(Some("cheese is nice".to_string()),
            parse_content(&"<title>cheese is nice</title>".to_string()));
        assert_eq!(Some("squanch".to_string()),
            parse_content(&"<title>     squanch</title>".to_string()));
        assert_eq!(Some("squanch".to_string()),
            parse_content(&"<title>squanch     </title>".to_string()));
        assert_eq!(Some("squanch".to_string()),
            parse_content(&"<title>\nsquanch</title>".to_string()));
        assert_eq!(Some("squanch".to_string()),
            parse_content(&"<title>\n  \n  squanch</title>".to_string()));
        assert_eq!(Some("we like the moon".to_string()),
            parse_content(&"<title>\n  \n  we like the moon</title>".to_string()));
        assert_eq!(Some("&hello123&<>''~".to_string()),
            parse_content(&"<title>&amp;hello123&amp;&lt;&gt;''~</title>".to_string()));
        assert_eq!(Some("CVE - CVE-2018-11235".to_string()),
            parse_content(&"<title>CVE -\nCVE-2018-11235\n</title>".to_string()));
    }
}

