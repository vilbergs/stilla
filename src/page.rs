use comrak::{markdown_to_html, ComrakOptions};

use regex::Regex;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::{collections::BTreeMap, fs, path::Path};
use time::format_description;
use time::Date;
use tinytemplate::{format_unescaped, TinyTemplate};

#[derive(Serialize)]
pub struct PageContext {
    pub pages: HashMap<String, Vec<Page>>,
    pub content: String,
}

fn round_serialize<S>(d: &Option<Date>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(date) = d {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let formatted = date.format(&format).unwrap().to_owned();

        return s.serialize_str(&formatted);
    }

    return s.serialize_none();
}

#[derive(Serialize, Debug, Clone)]
pub struct Page {
    frontmatter: Option<BTreeMap<String, String>>,
    content: String,
    #[serde(serialize_with = "round_serialize")]
    pub date: Option<Date>,
    pub filename: String,
}

impl Page {
    pub fn from_md_file(path: &Path) -> Page {
        let md_content = fs::read_to_string(&path).expect("Should have been able to read the file");

        let date_regex = Regex::new(r"^\d{4}-(0[1-9]|1[012])-(0[1-9]|[12][0-9]|3[0-1])").unwrap();
        let mut filename = path
            .strip_prefix(path.parent().unwrap())
            .unwrap()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut maybe_date: Option<Date> = None;

        if date_regex.is_match(&filename) {
            let new_filename = filename.clone();
            let mut parts = new_filename.split("_").into_iter();
            let date_str = parts.next().unwrap();
            filename = parts.next().unwrap().to_string();

            let format = format_description::parse("[year]-[month]-[day]").unwrap();

            maybe_date = Some(Date::parse(date_str, &format).unwrap());
        }

        if !md_content.starts_with("---") {
            return Self {
                frontmatter: None,
                content: md_content.trim().to_string(),
                date: maybe_date,
                filename,
            };
        }

        let (content, frontmatter) = Self::extract_frontmatter(&md_content);

        let s = Self {
            frontmatter: Some(frontmatter),
            content: content,
            date: maybe_date,
            filename,
        };

        return s;
    }

    pub fn html_content(&self) -> String {
        ammonia::clean(&markdown_to_html(&self.content, &ComrakOptions::default()))
    }

    pub fn render_from_template(&self, template_html: &str, context: &PageContext) -> String {
        let mut templating_engine = TinyTemplate::new();
        templating_engine.set_default_formatter(&format_unescaped);

        templating_engine
            .add_template("name", &template_html)
            .unwrap();

        templating_engine.render("name", context).unwrap()
    }

    fn extract_frontmatter(md_content: &str) -> (String, BTreeMap<String, String>) {
        let mut lines = md_content.lines();

        lines.next(); // Skip the first "---" and consume it

        let frontmatter_lines = lines.by_ref().take_while(|item| item != &"---");
        let yaml = frontmatter_lines.collect::<Vec<&str>>().join("\n");
        let frontmatter: BTreeMap<String, String> = serde_yaml::from_str(&yaml).unwrap();

        let content = lines.collect::<Vec<&str>>().join("\n").trim().to_string();

        (content, frontmatter)
    }
}
