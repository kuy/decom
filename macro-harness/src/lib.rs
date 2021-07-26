use anyhow::Result;
use console::style;
use std::path::{Path, PathBuf};

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
    std::panic::set_hook(Box::new(|_| {}));

    let path = path.as_ref().to_owned();
    std::panic::catch_unwind(|| {
        // TODO: Collect unknown errors and report
        inner_run(path).unwrap();
    })
    .unwrap_or_else(|_cause| {
        panic!("Failed");
    })
}

fn inner_run(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref().to_owned();
    let context = Context {
        project_dir: path::project_dir()?,
    };

    let source_path = context.project_dir.join(path);
    let stdout_path = path::with_extension(&source_path, "stdout");
    let expected = helper::read_expected(stdout_path)?;

    let template_path = source_path
        .parent()
        .unwrap_or_else(|| {
            panic!("Failed to traverse parent dir");
        })
        .join("Cargo.toml.template");
    let (temp_manifest_path, _) = manifest::prepare_manifest_file(&template_path, &source_path)?;

    let output = cargo::expand(&temp_manifest_path)?;
    let actual = String::from_utf8(output.stdout)?;

    // TODO: cleanup temp dir

    if expected == actual {
        println!(
            "{} {}",
            style("Pass:").green(),
            style(format!("{:?}", source_path.as_path())).dim()
        );
    } else {
        println!("{} {:?}", style("Failed:").red(), source_path.as_path());
        diff::print(&expected, &actual);
        panic!();
    }

    Ok(())
}
