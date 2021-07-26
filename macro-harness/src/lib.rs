use anyhow::Result;
use console::style;
use std::{
    fs::OpenOptions,
    io::Write,
    panic,
    path::{Path, PathBuf},
};

mod cargo;
mod diff;
mod helper;
mod manifest;
mod path;

struct Context {
    project_dir: PathBuf,
}

pub fn run(path: impl AsRef<Path>) {
    // Suppress stack backtrace output
    panic::set_hook(Box::new(|_| {}));

    let path = path.as_ref().to_owned();
    let overwrite = std::env::var("MACRO_HARNESS")
        .map(|value| value.to_lowercase() == "overwrite")
        .unwrap_or(false);
    panic::catch_unwind(|| {
        // TODO: Collect unknown errors and report
        inner_run(path, overwrite).unwrap();
    })
    .unwrap_or_else(|_cause| {
        panic!("Failed");
    })
}

fn inner_run(path: PathBuf, overwrite: bool) -> Result<()> {
    let context = Context {
        project_dir: path::project_dir()?,
    };

    let source_path = context.project_dir.join(path);
    let stdout_path = path::with_extension(&source_path, "stdout");
    let expected = helper::read_expected(&stdout_path)?;

    let template_path = source_path
        .parent()
        .unwrap_or_else(|| {
            panic!("Failed to traverse parent dir");
        })
        .join("Cargo.template.toml");
    let (temp_manifest_path, _) = manifest::prepare_manifest_file(&template_path, &source_path)?;

    let output = cargo::expand(&temp_manifest_path)?;
    let actual = String::from_utf8(output.stdout)?;

    // TODO: cleanup temp dir

    if expected == actual {
        println!(
            "{} {}",
            style("Pass:").green(),
            style(path::to_string(source_path)).dim()
        );
    } else {
        println!(
            "{} {}",
            style("Failed:").red(),
            path::to_string(source_path)
        );
        diff::print(&expected, &actual);

        if overwrite {
            let stdout_path = path::to_string(stdout_path);
            let msg = format!("Failed to open stdout file: {}", stdout_path);
            let mut stdout_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&stdout_path)
                .unwrap_or_else(|err| {
                    panic!("{}\n  {:?}", msg, err);
                });
            stdout_file.write_all(actual.as_bytes())?;
            stdout_file.flush()?;

            println!(
                "{} {}",
                style("└ Overwritten:").yellow(),
                style(stdout_path).dim()
            );
        }

        panic!();
    }

    Ok(())
}
