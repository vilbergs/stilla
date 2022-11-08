use std::collections::BTreeMap;

use comrak::{markdown_to_html, ComrakOptions};
use serde::Serialize;

#[derive(Serialize)]
pub struct Page {
    frontmatter: Option<BTreeMap<String, String>>,
    content: String,
}

impl Page {
    pub fn from_md(md_content: String) -> Page {
        if !md_content.starts_with("---") {
            return Self {
                frontmatter: None,
                content: md_content,
            };
        }

        let (content, frontmatter) = Self::extract_frontmatter(&md_content);

        return Self {
            frontmatter: Some(frontmatter),
            content,
        };
    }

    pub fn html_content(&self) -> String {
        ammonia::clean(&markdown_to_html(&self.content, &ComrakOptions::default()))
    }

    fn extract_frontmatter(md_content: &str) -> (String, BTreeMap<String, String>) {
        let mut lines = md_content.lines();

        lines.next(); // Skip the first "---" and consume it

        let frontmatter_lines = lines.by_ref().take_while(|item| item != &"---");
        let yaml = frontmatter_lines.collect::<Vec<&str>>().join("\n");
        let frontmatter: BTreeMap<String, String> = serde_yaml::from_str(&yaml).unwrap();

        let content = lines.collect::<Vec<&str>>().join("\n");

        (content, frontmatter)
    }
}
