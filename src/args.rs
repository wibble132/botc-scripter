use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

#[derive(clap::Parser, Clone)]
pub struct Args {
    /// The short name for the script
    ///
    /// If set to `all`, this will run on every script
    pub name: String,

    /// The version of the script, e.g. "1.0.0", "3.2.1".
    ///
    /// Can be omitted for online scripts, in which case the latest version will be used.
    #[arg(short, long)]
    pub version: Option<String>,

    /// The input json script file. Defaults to stdin.
    #[arg(alias = "in", short, long)]
    pub input_file: Option<PathBuf>,

    /// The base directory for the outputs.
    ///
    /// Files will be written to `<out>/script/<name>.mcfunction` and `<out>/roles/<name>.mcfunction`.
    ///
    /// Defaults to `./out`.
    #[arg(alias = "out", short, long)]
    pub output_dir: Option<PathBuf>,
}

impl Args {
    pub fn output_script_file(&self) -> File {
        self.output_file("script")
    }
    pub fn output_roles_file(&self) -> File {
        self.output_file("roles")
    }
    fn output_file(&self, dir: &str) -> File {
        let mut base_dir = self.base_dir();

        if !fs::exists(&base_dir).unwrap() {
            fs::create_dir(&base_dir).unwrap();
        }

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
    pub fn base_dir(&self) -> PathBuf {
        self.output_dir
            .clone()
            .unwrap_or_else(Self::get_default_out)
    }
    fn get_default_out() -> PathBuf {
        let mut current_dir = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Cannot find current directory. Specify output directory with `--out`.");
                eprintln!("{e}");
                exit(1);
            }
        };
        current_dir.push("out");
        current_dir
    }
}
