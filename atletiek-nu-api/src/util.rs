use regex::Regex;

const REGEX_CLEAN_HTML: &'static str = "<(.|\n)*?>";

/// Removes everything except the plaintext from the html
#[allow(dead_code)]
pub fn clean_html_re(html: &str) -> String {
    let re = Regex::new(REGEX_CLEAN_HTML).unwrap();
    re.replace(html, "").to_string()
}

/// Keeps only the top-level plaintext
pub fn clean_html(html: &str) -> String {
    let mut res = String::new();

    let mut is_in_tag = false;
    let mut closing_tag = false;
    let mut tag_depth = 0;

    for c in html.chars() {
        match c {
            '<' => {
                is_in_tag = true;
            }
            '>' => {
                if closing_tag {
                    tag_depth -= 1;
                } else {
                    tag_depth += 1;
                }

                closing_tag = false;
                is_in_tag = false;
            }
            c => {
                if is_in_tag && c == '/' {
                    closing_tag = true;
                }

                if tag_depth == 0 && !is_in_tag {
                    res.push(c);
                }
            }
        }
    }

    res
}
