mod tailwind;

use tailwind::Tailwind;

use ammonia::clean;
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, path::Path};
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

    for entry in WalkDir::new(&pages_dir).into_iter().filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        let path = entry.path();

        if f_name.ends_with(".md") {
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

            let md_html = markdown_to_html(&contents, &ComrakOptions::default());
            let unsafe_html = match template {
                Some(template_html) => template_html.replace("{{ content }}", md_html.as_str()),
                None => md_html,
            };
            let safe_html = &unsafe_html;

            println!("{}", out_path.display());

            let mut out_file =
                File::create(out_path.join("index.html")).expect("Could not create file");

            out_file.write_all(safe_html.as_bytes())?;
        }
    }

    Ok(())
}
