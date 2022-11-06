use include_dir::{include_dir, Dir};
use std::io::Error;
use std::os::unix::prelude::PermissionsExt;
use std::{env, fs};
use std::{path::PathBuf, process::Command};

static TAILWIND_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/tailwind");
static PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct Tailwind {}

impl Tailwind {
    pub fn build(output_root: &PathBuf) -> Result<(), Error> {
        let tmp = env::temp_dir().join(PKG_NAME);

        if !tmp.exists() {
            fs::create_dir(&tmp)?;
            TAILWIND_DIR
                .extract(&tmp)
                .expect("Could not extract Tailwind files");

            let bin_path = tmp.join("tailwindcss");
            let bin_file = fs::File::open(&bin_path).unwrap();
            let mut perms = bin_file.metadata().unwrap().permissions();

            perms.set_mode(0o755);

            bin_file.set_permissions(perms).unwrap();
        }

        let output_path = output_root.join("tailwind.css");
        let output_path_str = output_path
            .to_str()
            .expect("Could not convert output path to string");
        let input_path_str = tmp.join("input.css");
        let config_path_str = tmp.join("tailwind.config.js");

        let mut cmd = Self::command();
        let output = cmd
            .args([
                "-c",
                config_path_str
                    .to_str()
                    .expect("could not convert config path"),
                "-i",
                input_path_str
                    .to_str()
                    .expect("could not convert input path"),
                "-o",
                output_path_str,
            ])
            .output()
            .expect("failed to execute process");

        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }

    fn command() -> Command {
        let tmp = env::temp_dir().join(PKG_NAME);
        let bin_path = tmp.join("tailwindcss");

        Command::new(bin_path.to_str().unwrap())
    }
}
