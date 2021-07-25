use anyhow::{Error, Result};
use cargo_toml::{self, Dependency, DependencyDetail, Edition, Product};
use console::{style, Style};
use rand;
use similar::{ChangeTag, TextDiff};
use std::{
    env, fmt,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use toml;

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
        project_dir: project_dir()?,
    };

    let source_path = context.project_dir.join(path);
    let stdout_path = to_stdout_path(&source_path);
    let expected = read_expected(stdout_path)?;

    let template_path = source_path
        .parent()
        .ok_or(Error::msg("failed to traverse parent dir"))?
        .join("Cargo.toml.template");
    let (temp_manifest_path, _) = prepare_manifest_file(&template_path, &source_path)?;

    let output = cargo_expand(&temp_manifest_path)?;
    let actual = String::from_utf8(output.stdout)?;

    if expected == actual {
        println!(
            "{} {}",
            style("Pass:").green(),
            style(format!("{:?}", source_path.as_path())).dim()
        );
    } else {
        println!("{} {:?}", style("Failed:").red(), source_path.as_path());
        print_diff(&expected, &actual);
        panic!();
    }

    Ok(())
}

fn cargo_expand(manifest_path: impl AsRef<Path>) -> Result<Output> {
    let manifest_path = manifest_path.as_ref().to_owned();
    let output = Command::new("cargo")
        .args(&[
            "expand",
            "--manifest-path",
            manifest_path.as_path().to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to run 'cargo expand': {:?}\n  Error: {:?}",
                manifest_path, err
            );
        });
    if !output.status.success() {
        let msg = String::from_utf8(output.stderr).expect("should be convert");
        panic!("Failed to expand\n  Error: {}", msg);
    }
    Ok(output)
}

fn read_expected(path: PathBuf) -> Result<String> {
    let copy = path.clone();
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!("Failed to read expected file: {:?}\nError: {:?}", copy, err);
    });
    Ok(content)
}

fn to_stdout_path(path: &PathBuf) -> PathBuf {
    let mut path = path.clone();
    path.set_extension("stdout");
    path
}

fn project_dir() -> Result<PathBuf> {
    env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(Error::msg("failed to get source dir"))
}

fn create_temp_dir() -> PathBuf {
    let dir = env::temp_dir();
    let n: u128 = rand::random();
    let path = dir.join(format!("macro-harness-build.{}", n));

    let msg = format!("Failed to create temporary directory: {:?}", path);
    fs::create_dir(&path).unwrap_or_else(|err| {
        panic!("{}\n  Error: {:?}", msg, err);
    });

    path
}

fn prepare_manifest_file(
    template_path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf)> {
    // Parepare temporary directory
    let dir = create_temp_dir();
    let temp_manifest_path = dir.join("Cargo.toml");

    // Deserialize template manifest file
    let content = fs::read_to_string(&template_path)?;
    let mut manifest = cargo_toml::Manifest::from_slice(content.as_bytes()).unwrap_or_else(|err| {
        panic!(
            "Failed load manifest: {:?}\n  Error: {:?}",
            template_path.as_ref(),
            err
        );
    });

    // Apply modifications: deps, lib
    let manifest_path = template_path.as_ref().to_path_buf();
    manifest.dependencies = manifest
        .dependencies
        .into_iter()
        .map(|(crate_name, dep)| match dep.clone() {
            Dependency::Detailed(detail) => {
                if let Some(rel_path) = detail.clone().path {
                    let manifest_dir = manifest_path.parent().expect("should be have parent");
                    let crate_dir = canonicalize_path(&manifest_dir, &rel_path);
                    let detail = DependencyDetail {
                        path: Some(String::from(crate_dir.to_str().unwrap())),
                        ..detail.clone()
                    };
                    (crate_name, Dependency::Detailed(detail))
                } else {
                    (crate_name, dep.clone())
                }
            }
            _ => (crate_name, dep.clone()),
        })
        .collect();

    let source_path = String::from(source_path.as_ref().to_str().unwrap());
    manifest.lib = Some(Product {
        path: Some(source_path),
        edition: Some(Edition::E2018),
        ..Default::default()
    });

    // Write out manifest
    let content = toml::to_string(&manifest).unwrap_or_else(|err| {
        panic!("Failed serialize manifest\n  Error: {:?}", err);
    });
    let mut temp_manifest = File::create(&temp_manifest_path).unwrap_or_else(|err| {
        panic!(
            "Failed create manifest file: {:?}\n  Error: {:?}",
            temp_manifest_path, err
        );
    });
    temp_manifest.write_all(content.as_bytes())?;
    temp_manifest.flush()?;

    Ok((temp_manifest_path, dir))
}

fn canonicalize_path(base_path: impl AsRef<Path>, rel_path: impl AsRef<Path>) -> PathBuf {
    let base = base_path.as_ref().to_path_buf();

    let msg = format!(
        "Failed to canonicalize: {:?}, {:?}",
        &base,
        rel_path.as_ref()
    );
    base.join(rel_path).canonicalize().unwrap_or_else(|err| {
        panic!("{}\n  Error: {:?}", msg, err);
    })
}

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn print_diff(expected: &String, actual: &String) {
    let diff = TextDiff::from_lines(expected, actual);
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                print!(
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                );
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        print!("{}", s.apply_to(value).underlined().on_black());
                    } else {
                        print!("{}", s.apply_to(value));
                    }
                }
                if change.missing_newline() {
                    println!();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_stdout_path() {
        assert_eq!(
            to_stdout_path(&PathBuf::from("tests/ui/test_foo_bar.rb")),
            PathBuf::from("tests/ui/test_foo_bar.stdout")
        );
    }
}
