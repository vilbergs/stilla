mod tailwind;

use ammonia;
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use serde::Serialize;
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

#[derive(Serialize)]
struct Page {
    name: String,
}

#[derive(Serialize)]
struct Context {
    pages: HashMap<String, Vec<Page>>,
    content: String,
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

    let mut context = Context {
        pages: HashMap::new(),
        content: String::new(),
    };

    for entry in WalkDir::new(&pages_dir).into_iter().filter_map(|e| e.ok()) {
        let mut tt = TinyTemplate::new();

        let f_name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().is_dir();
        let path = entry.path();

        // println!("{}", &out_path.display());

        if is_dir {
            context.pages.insert(f_name.to_owned(), Vec::new());
        }

        if f_name.ends_with(".md") && !f_name.starts_with("#") {
            let relative_path = path
                .strip_prefix(&pages_dir)
                .expect("Could not get relative path to file");

            let contents =
                fs::read_to_string(&path).expect("Should have been able to read the file");

            let html_file = relative_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let out_path = match html_file.as_str() {
                "index" => out_dir.join(relative_path.parent().unwrap()),
                _ => out_dir.join(relative_path.parent().unwrap().join(&html_file)),
            };

            std::fs::create_dir_all(&out_path).unwrap();

            let template = match html_file.as_str() {
                "index" => &templates.index,
                _ => &templates.page,
            };

            let md_html = ammonia::clean(&markdown_to_html(&contents, &ComrakOptions::default()));
            context.content = md_html.to_owned();

            let html = match template {
                Some(template_html) => template_html.replace("{{ content }}", md_html.as_str()),
                None => md_html,
            };

            tt.add_template(out_path.to_str().unwrap(), &html).unwrap();

            let mut out_file =
                File::create(&out_path.join("index.html")).expect("Could not create file");

            let parent = entry
                .path()
                .parent()
                .unwrap()
                .to_str()
                .unwrap()
                .split("/")
                .last()
                .unwrap();

            println!("{}", parent);
            if let Some(page_vec) = context.pages.get_mut(parent) {
                page_vec.push(Page {
                    name: f_name.to_owned(),
                })
            }

            out_file.write_all(
                tt.render(out_path.to_str().unwrap(), &context)
                    .unwrap()
                    .as_bytes(),
            )?;
        }
    }

    Ok(())
}
