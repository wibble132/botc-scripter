use std::fs;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::PathBuf;
use std::process::exit;

#[derive(clap::Parser)]
pub struct Args {
    /// The short name for the script
    pub name: String,

    /// The Minecraft colour name for the script
    pub color: String,

    /// The input json script file. Defaults to stdin.
    #[arg(alias = "in", short, long)]
    input_file: Option<PathBuf>,

    /// The base directory for the outputs.
    ///
    /// Files will be written to `<out>/script/<name>.mcfunction` and `<out>/roles/<name>.mcfunction`.
    ///
    /// Defaults to the current working directory.
    #[arg(alias = "out", short, long)]
    output_dir: Option<PathBuf>,
}

impl Args {
    pub fn input_stream(&self) -> Box<dyn BufRead> {
        if let Some(file_path) = &self.input_file
            && file_path.as_os_str() != "-"
        {
            let file = match File::open(file_path) {
                Err(e) => {
                    eprintln!("Failed to open file {}.", file_path.display());
                    eprintln!("{e}");
                    exit(1);
                }
                Ok(f) => f,
            };

            Box::new(BufReader::new(file))
        } else {
            Box::new(BufReader::new(stdin().lock()))
        }
    }
    pub fn output_script_file(&self) -> File {
        self.output_file("script")
    }
    pub fn output_roles_file(&self) -> File {
        self.output_file("roles")
    }
    fn output_file(&self, dir: &str) -> File {
        let mut base_dir = self
            .output_dir
            .clone()
            .unwrap_or_else(Self::get_current_dir);

        base_dir.push(dir);
        if !fs::exists(&base_dir).unwrap() {
            fs::create_dir(&base_dir).unwrap();
        }
        base_dir.push(format!("{}.mcfunction", self.name));

        match File::create(base_dir) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not create output {dir} file at path {e}");
                eprintln!("{e}");
                exit(1);
            }
        }
    }
    fn get_current_dir() -> PathBuf {
        match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Cannot find current directory. Specify output directory with `--out`.");
                eprintln!("{e}");
                exit(1);
            }
        }
    }
}