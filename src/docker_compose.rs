use std::{error::Error, result::Result, str};
use tokio::process::Command;

pub async fn containers() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("docker-compose")
        .args(&["ps", "-q"])
        .current_dir("/home/kodama/work/initial/workspaces/e2e/environments")
        .output()
        .await?;
    Ok(parse_ps_result(str::from_utf8(output.stdout.as_slice())?))
}

fn parse_ps_result(output: &str) -> Vec<String> {
    output.lines().into_iter().map(|s| s.to_string()).collect()
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
        assert_eq!(parse_ps_result(output), expected);
    }
}
