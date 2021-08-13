use crate::core::frontend::GenericResult;
use crate::Metadata;
use crate::Result;
use octocrab;
use std::collections::HashMap;

pub fn lookup_pat() -> Result<String> {
    // Tie this back to the user object at some point.
    Ok(std::env::var("github_pat")?)
}

#[derive(Serialize)]
pub struct DispatchWorkflowRequest {
    r#ref: String,
    inputs: HashMap<String, String>,
}

impl DispatchWorkflowRequest {
    pub fn new(git_ref: &str, inputs: Option<HashMap<String, String>>) -> Self {
        Self {
            r#ref: git_ref.to_string(),
            inputs: {
                if let Some(ins) = inputs {
                    ins
                } else {
                    let h: HashMap<String, String> = HashMap::new();
                    h
                }
            },
        }
    }
}

pub fn dispatch_workflow(
    owner: &str,
    repo: &str,
    workflow: &str,
    git_ref: &str,
    inputs: Option<HashMap<String, String>>,
) -> Result<GenericResult> {
    let o = octocrab::OctocrabBuilder::new()
        .personal_token(lookup_pat()?)
        .add_header(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        )
        .build()?;
    let r = tokio::runtime::Runtime::new().unwrap();
    let _guard = r.enter();

    let response = futures::executor::block_on(o._post(
        format!(
            "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
            owner, repo, workflow
        ),
        Some(&DispatchWorkflowRequest::new(git_ref, inputs)),
    ))?;
    let headers = response.headers().clone();
    let status = response.status().as_u16() as usize;
    let body = futures::executor::block_on(response.text())?;

    let mut res = GenericResult::new_success_or_fail(body.is_empty());
    res.set_msg(body);
    res.add_metadata("header", Metadata::String(format!("{:?}", headers)))?;
    res.add_metadata("status", Metadata::Usize(status))?;
    Ok(res)
}
