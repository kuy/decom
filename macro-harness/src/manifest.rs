use crate::path;
use anyhow::Result;
use cargo_toml::{self, Dependency, DependencyDetail, Edition, Product};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

pub fn prepare_manifest_file(
    template_path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf)> {
    // Parepare temporary directory
    let dir = path::create_temp_dir();
    let temp_manifest_path = dir.join("Cargo.toml");

    // Deserialize template manifest file
    let content = fs::read_to_string(&template_path)?;
    let mut manifest = cargo_toml::Manifest::from_slice(content.as_bytes()).unwrap_or_else(|err| {
        panic!(
            "Failed to load manifest: {:?}\n  Error: {:?}",
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
                    let crate_dir = path::canonicalize(&manifest_dir, &rel_path);
                    let detail = DependencyDetail {
                        path: Some(String::from(crate_dir.to_str().unwrap())),
                        ..detail
                    };
                    (crate_name, Dependency::Detailed(detail))
                } else {
                    (crate_name, dep)
                }
            }
            _ => (crate_name, dep),
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
        panic!("Failed to serialize manifest\n  Error: {:?}", err);
    });
    let mut temp_manifest = File::create(&temp_manifest_path).unwrap_or_else(|err| {
        panic!(
            "Failed to create manifest file: {:?}\n  Error: {:?}",
            temp_manifest_path, err
        );
    });
    temp_manifest.write_all(content.as_bytes())?;
    temp_manifest.flush()?;

    Ok((temp_manifest_path, dir))
}
