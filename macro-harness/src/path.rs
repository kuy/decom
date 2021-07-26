use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn with_extension(path: &PathBuf, ext: &str) -> PathBuf {
    let mut path = path.clone();
    path.set_extension(ext);
    path
}

pub fn project_dir() -> Result<PathBuf> {
    let dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            panic!("Failed to get manifest dir");
        });
    Ok(dir)
}

pub fn create_temp_dir() -> PathBuf {
    let dir = env::temp_dir();
    let n: u128 = rand::random();
    let path = dir.join(format!("macro-harness-build.{}", n));

    let msg = format!("Failed to create temporary directory: {:?}", path);
    fs::create_dir(&path).unwrap_or_else(|err| {
        panic!("{}\n  Error: {:?}", msg, err);
    });

    path
}

pub fn canonicalize(base_path: impl AsRef<Path>, rel_path: impl AsRef<Path>) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_stdout_path() {
        assert_eq!(
            with_extension(&PathBuf::from("tests/ui/test_foo_bar.rb"), "stdout"),
            PathBuf::from("tests/ui/test_foo_bar.stdout")
        );
    }
}
