use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use std::ffi::OsString;
use std::fs::{self, read_dir, File, ReadDir};
use std::io::Write;
use std::{
    env,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String,
}

fn main() -> std::io::Result<()> {
    let cwd = env::current_dir()?;
    let cwd_path = cwd.as_path();
    let args = Args::parse();
    let path = Path::new(&args.path);

    let project_path = cwd_path.join(path);

    let pages_dir = project_path.join(Path::new("pages"));
    let out_dir = project_path.join("dist");

    if !out_dir.exists() {
        fs::create_dir(&out_dir)?;
    }

    for entry in WalkDir::new(&pages_dir).into_iter().filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        let path = entry.path();

        if f_name.ends_with(".md") {
            let relative_path = path
                .strip_prefix(&pages_dir)
                .expect("Could not get relative path to file");

            let contents =
                fs::read_to_string(&path).expect("Should have been able to read the file");

            let mut html_file = relative_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let out_path = match html_file.as_str() {
                "index" => out_dir.join(relative_path.parent().unwrap()),
                _ => out_dir.join(relative_path.parent().unwrap().join(html_file)),
            };

            std::fs::create_dir_all(&out_path).unwrap();

            let html = markdown_to_html(&contents, &ComrakOptions::default());

            println!("{}", out_path.display());

            let mut out_file =
                File::create(out_path.join("index.html")).expect("Could not create file");

            out_file.write_all(html.as_bytes());
        }
    }

    Ok(())
}
