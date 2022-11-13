mod page;
mod tailwind;

use clap::Parser;
use page::{Page, PageContext};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, path::Path};
use tailwind::Tailwind;
use tinytemplate::TinyTemplate;
use walkdir::WalkDir;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

struct Templates {
    index: Option<String>,
    page: Option<String>,
}

struct PageWrapper {
    filename: String,
    out_path: PathBuf,
    file_path: PathBuf,
    template_html: String,
    page: Page,
}

fn main() -> std::io::Result<()> {
    let cwd = env::current_dir()?;
    let cwd_path = cwd.as_path();
    let args = Args::parse();
    let path = Path::new(&args.path);

    let project_path = cwd_path.join(path);

    let pages_dir = project_path.join(Path::new("pages"));
    let templates_dir = project_path.join(Path::new("templates"));
    let out_dir = project_path.join("dist");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("Could not remove output directory");
    }

    fs::create_dir(&out_dir)?;

    Tailwind::build(&out_dir)?;

    let templates = Templates {
        index: match fs::read_to_string(templates_dir.join("index.html")) {
            Ok(html) => Some(html),
            Err(_) => None,
        },
        page: match fs::read_to_string(templates_dir.join("page.html")) {
            Ok(html) => Some(html),
            Err(_) => None,
        },
    };

    let root_dir = WalkDir::new(&pages_dir).into_iter().filter_map(|e| e.ok());

    let mut pages = HashMap::new();

    let mut wrappers: Vec<PageWrapper> = Vec::new();

    for entry in root_dir {
        let mut tt = TinyTemplate::new();
        let f_name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().is_dir();
        let path = entry.path();

        if is_dir {
            pages.insert(f_name.to_owned(), Vec::new());
        }

        if f_name.ends_with(".md") && !f_name.starts_with("#") {
            let relative_path = path
                .strip_prefix(&pages_dir)
                .expect("Could not get relative path to file");

            let page = Page::from_md_file(path);

            let html_file = relative_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let parent_path = relative_path.parent().unwrap();
            let out_path = match html_file.as_str() {
                "index" => out_dir.join(&parent_path),
                _ => out_dir.join(&parent_path.join(&html_file)),
            };
            let p = out_path.to_string_lossy().to_string().clone();

            std::fs::create_dir_all(&out_path).unwrap();

            let template = match html_file.as_str() {
                "index" => {
                    let parent_dir_name = parent_path.to_str().unwrap().rsplit("/").next().unwrap();
                    let maybe_dir_template_path =
                        templates_dir.join(format!("{}.html", parent_dir_name));

                    let index_template = match fs::read_to_string(maybe_dir_template_path) {
                        Ok(section_template) => Some(section_template),
                        _ => templates.index.to_owned(),
                    };

                    index_template
                }
                _ => templates.page.to_owned(),
            };

            let md_html = page.html_content();
            let content = md_html.to_owned();

            let template_html = match template {
                Some(template_html) => template_html,
                None => md_html,
            };

            let out_file_path = out_path.join("index.html");

            let parent = entry
                .path()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .split("/")
                .last()
                .unwrap();

            if let Some(page_vec) = pages.get_mut(parent) {
                if html_file != "index" {
                    page_vec.push(page.clone())
                }
            }

            wrappers.push(PageWrapper {
                filename: html_file,
                out_path,
                file_path: out_file_path,
                template_html,
                page,
            })
        }
    }

    // TODO: Implement sorting
    for vec in pages.values_mut() {
        vec.sort_by(|a, b| {
            let ordering = match (a.date, b.date) {
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some(date_a), Some(date_b)) => date_b.cmp(&date_a), // Most recent first
                _ => Ordering::Equal,
            };

            return ordering;
        })
    }

    for wrapper in wrappers {
        let mut out_file =
            File::create(wrapper.out_path.join("index.html")).expect("Could not create page file");

        let ctx = PageContext {
            pages: pages.clone(),
            content: wrapper.page.html_content(),
        };

        out_file
            .write_all(
                wrapper
                    .page
                    .render_from_template(&wrapper.template_html, &ctx)
                    .as_bytes(),
            )
            .expect("Could not write_template");
    }

    Ok(())
}
