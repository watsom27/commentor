use reqwest::{blocking::Client, StatusCode};
use serde_json::{json, Value};

static API_URL: &str = "https://api.github.com";

#[derive(Debug, Default)]
pub struct PullRequestIdentifier {
    owner:  String,
    repo:   String,
    number: String, // shh
}

pub fn parse_url(pr_url: &str) -> Result<PullRequestIdentifier, &'static str> {
    if pr_url.starts_with("http://github.com") {
        Err("Non-https link, I don't support those!")
    } else {
        let trimmed = pr_url.trim_start_matches("https://github.com/");
        let mut split = trimmed.split('/');
        let mut result = PullRequestIdentifier::default();

        if let Some(owner) = split.next() {
            result.owner = owner.to_owned();
        }

        if let Some(repo) = split.next() {
            result.repo = repo.to_owned();
        }

        // discard the `pull` or `issue`
        let _ = split.next();

        if let Some(number) = split.next() {
            result.number = number.to_owned();
        }

        log::trace!("Parsed pr identifier: {result:?}");
        Ok(result)
    }
}

#[derive(Debug)]
pub struct PullRequestDescription {
    pub url:    String,
    pub title:  String,
    pub author: String,
    pub state:  String,
}

/// Request pr details from github, will panic if the json received is badly formatted.
pub fn get_pull_request(
    client: &Client,
    pr: &PullRequestIdentifier,
    token: &str,
) -> anyhow::Result<Option<PullRequestDescription>> {
    let response = client
        .get(format!(
            "{API_URL}/repos/{}/{}/pulls/{}",
            pr.owner, pr.repo, pr.number
        ))
        .header("Accept", "application/json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "commentor")
        .bearer_auth(token)
        .send()?;

    let mut result = None;

    if response.status() == 200 {
        let json = response.text()?;
        let parsed_json: Value = serde_json::from_str(&json)?;

        let title = parsed_json
            .get("title")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let url = parsed_json.get("url").unwrap().as_str().unwrap().to_owned();
        let author = parsed_json
            .get("user")
            .unwrap()
            .get("login")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let state = parsed_json
            .get("state")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();

        result = Some(PullRequestDescription {
            url,
            title,
            author,
            state,
        });
    }

    Ok(result)
}

pub fn post_comment(
    client: &Client,
    pr: &PullRequestIdentifier,
    token: &str,
    comment: &str,
) -> anyhow::Result<StatusCode> {
    let response = client
        .post(format!(
            "{API_URL}/repos/{}/{}/issues/{}/comments",
            pr.owner, pr.repo, pr.number
        ))
        .header("Accept", "application/json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "commentor")
        .bearer_auth(token)
        .body(
            json!({
                "body": comment.to_owned(),
            })
            .to_string(),
        )
        .send()?;

    Ok(response.status())
}
