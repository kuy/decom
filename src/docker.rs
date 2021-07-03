use futures::prelude::*;
use std::{collections::HashMap, error::Error, process::Stdio, result::Result, str};
use tokio::process::Command;
use tokio_util::codec::{FramedRead, LinesCodec};

pub async fn names(container_ids: Vec<String>) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let output = Command::new("docker")
        .args(&["ps", "--format", "{{.ID}},{{.Names}}", "--no-trunc"])
        .output()
        .await?;
    let output = str::from_utf8(output.stdout.as_slice())?;
    let dict = parse_ps_result(output);
    Ok(map_id_and_name(container_ids, dict))
}

fn parse_ps_result(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .into_iter()
        .filter_map(|s| {
            s.split_once(',')
                .map(|(id, name)| (id.to_string(), name.to_string()))
        })
        .collect()
}

fn map_id_and_name(ids: Vec<String>, dict: HashMap<String, String>) -> HashMap<String, String> {
    ids.into_iter()
        .map(|id| (id.clone(), dict.get(&id)))
        .filter_map(|(id, name)| match name {
            Some(name) => Some((id, name.clone())),
            _ => None,
        })
        .collect()
}

pub async fn logs(container_id: String) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("docker")
        .args(&["logs", "-f", container_id.as_str()])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let stdout = child.stdout.take().expect("child output");
    let mut reader = FramedRead::new(stdout, LinesCodec::new());
    while let Some(line) = reader.next().await {
        println!("{}", line?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ps_result() {
        let output = "7d7b045685ce0704de380ad30424cdcdde79448c6b78e967f69997db69678fc4,environments_apple_1\n\
            f087f5679cea32b36414923f62d0d30b851cfeebbac46cac440068fcdc6043eb,environments_banana-api_1\n\
            2e5aeea126fe2ce71c65501f428a2880664cd001fd6ec84cae688cec45a57794,environments_coconut-api_1";
        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert(
            "7d7b045685ce0704de380ad30424cdcdde79448c6b78e967f69997db69678fc4".into(),
            "environments_apple_1".into(),
        );
        expected.insert(
            "f087f5679cea32b36414923f62d0d30b851cfeebbac46cac440068fcdc6043eb".into(),
            "environments_banana-api_1".into(),
        );
        expected.insert(
            "2e5aeea126fe2ce71c65501f428a2880664cd001fd6ec84cae688cec45a57794".into(),
            "environments_coconut-api_1".into(),
        );
        assert_eq!(parse_ps_result(output), expected);
    }

    #[test]
    fn test_map_id_and_name() {
        let ids = vec![
            "7d7b045685ce0704de380ad30424cdcdde79448c6b78e967f69997db69678fc4".into(),
            "2e5aeea126fe2ce71c65501f428a2880664cd001fd6ec84cae688cec45a57794".into(),
        ];
        let mut dict: HashMap<String, String> = HashMap::new();
        dict.insert(
            "7d7b045685ce0704de380ad30424cdcdde79448c6b78e967f69997db69678fc4".into(),
            "environments_apple_1".into(),
        );
        dict.insert(
            "f087f5679cea32b36414923f62d0d30b851cfeebbac46cac440068fcdc6043eb".into(),
            "environments_banana-api_1".into(),
        );
        dict.insert(
            "2e5aeea126fe2ce71c65501f428a2880664cd001fd6ec84cae688cec45a57794".into(),
            "environments_coconut-api_1".into(),
        );

        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert(
            "7d7b045685ce0704de380ad30424cdcdde79448c6b78e967f69997db69678fc4".into(),
            "environments_apple_1".into(),
        );
        expected.insert(
            "2e5aeea126fe2ce71c65501f428a2880664cd001fd6ec84cae688cec45a57794".into(),
            "environments_coconut-api_1".into(),
        );
        assert_eq!(map_id_and_name(ids, dict), expected);
    }
}
