use crate::docker::{self, Container};
use std::{error::Error, result::Result, str};
use tokio::process::Command;

pub async fn containers() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("docker-compose")
        .args(&["ps", "-q"])
        .current_dir("/home/kodama/work/kuy/periodic-output")
        .output()
        .await?;
    Ok(parse_lines(str::from_utf8(output.stdout.as_slice())?))
}

fn parse_lines(output: &str) -> Vec<String> {
    output.lines().into_iter().map(|s| s.to_string()).collect()
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub service_name: String,
    pub container_name: String,
    pub container_id: String,
}

pub async fn services() -> Result<Vec<Service>, Box<dyn Error>> {
    let output = Command::new("docker-compose")
        .args(&["config", "--services"])
        .current_dir("/home/kodama/work/kuy/periodic-output")
        .output()
        .await?;
    let service_names = parse_lines(str::from_utf8(output.stdout.as_slice())?);

    let ids = containers().await?;
    let containers = docker::names(ids).await?;

    Ok(merge_service_names(service_names, containers))
}

fn merge_service_names(service_names: Vec<String>, containers: Vec<Container>) -> Vec<Service> {
    service_names
        .into_iter()
        .filter_map(|service_name| {
            let container = containers.iter().find(|&c| c.name.contains(&service_name));
            match container {
                Some(c) => Some(Service {
                    service_name: service_name.clone(),
                    container_name: c.name.clone(),
                    container_id: c.id.clone(),
                }),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ps_result() {
        let output = "88c904a13f039e3eca4ff145cb06d8244548a341e56212f7e43598c2c4f62e8a\n\
            f087f5679cea32b36414923f62d0d30b851cfeebbac46cac440068fcdc6043eb\n\
            98d9a30d2e5676539dd92ef986ed2159e85af80ab77eaf2ac6f311a71ecca458\n";
        let expected = vec![
            "88c904a13f039e3eca4ff145cb06d8244548a341e56212f7e43598c2c4f62e8a".to_owned(),
            "f087f5679cea32b36414923f62d0d30b851cfeebbac46cac440068fcdc6043eb".to_owned(),
            "98d9a30d2e5676539dd92ef986ed2159e85af80ab77eaf2ac6f311a71ecca458".to_owned(),
        ];
        assert_eq!(parse_lines(output), expected);
    }

    #[test]
    fn test_merge_service_names() {
        let service_names = vec!["service1".into(), "service2".into(), "service3".into()];
        let containers = vec![
            Container {
                id: "98d9a30d2e5676539dd92ef986ed2159e85af80ab77eaf2ac6f311a71ecca458".into(),
                name: "parent-dir_service1_1".into(),
            },
            Container {
                id: "88c904a13f039e3eca4ff145cb06d8244548a341e56212f7e43598c2c4f62e8a".into(),
                name: "parent-dir_service2_1".into(),
            },
        ];
        let expected = vec![
            Service {
                service_name: "service1".into(),
                container_id: "98d9a30d2e5676539dd92ef986ed2159e85af80ab77eaf2ac6f311a71ecca458"
                    .into(),
                container_name: "parent-dir_service1_1".into(),
            },
            Service {
                service_name: "service2".into(),
                container_id: "88c904a13f039e3eca4ff145cb06d8244548a341e56212f7e43598c2c4f62e8a"
                    .into(),
                container_name: "parent-dir_service2_1".into(),
            },
        ];
        assert_eq!(merge_service_names(service_names, containers), expected);
    }
}
