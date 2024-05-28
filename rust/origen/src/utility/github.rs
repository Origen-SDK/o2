use origen_metal::{Result, Outcome, octocrab, futures};
use std::collections::HashMap;

pub fn with_blocking_calls<F, V>(mut f: F) -> Result<V>
where
    F: FnMut() -> Result<V>,
{
    let r = tokio::runtime::Runtime::new().unwrap();
    let _guard = r.enter();
    f()
}

macro_rules! block_on {
    ($call:expr) => {
        futures::executor::block_on($call)
    }
}

pub fn lookup_pat() -> Result<String> {
    // TODO Publishing Tie this back to the user object at some point.
    match std::env::var("github_pat") {
        Ok(v) => Ok(v),
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                bail!("Environment variable 'github_pat' was not found")
            },
            _ => return Err(e.into())
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Actor {
    pub login: String,
    pub id: usize,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRuns {
    pub total_count: usize,
    workflow_runs: Vec<WorkflowRun>
}

impl WorkflowRuns {
    pub fn get_only(mut self) -> Result<WorkflowRun> {
        let l = self.workflow_runs.len();
        if l != 1 {
            bail!("Expected a single workflow run but found {}", l)
        }
        Ok(self.workflow_runs.pop().unwrap())
    }
}

#[derive(Deserialize, Debug)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: String,
    pub html_url: String,
    pub run_attempt: u8,
    pub path: String,
    pub event: String,
    // TODO parse these as DATETIMEs? use time::PrimitiveDateTime as DateTime;
    pub created_at: String,
    pub updated_at: String,
    pub run_started_at: String,
    pub triggering_actor: Actor,
    pub cancel_url: String,
    pub rerun_url: String,
}

impl WorkflowRun {
    pub fn was_cancelled(&self) -> bool {
        self.conclusion.as_ref().map_or( false, |c| c == "cancelled")
    }

    pub fn cancel(&self) -> Result<()> {
        send_post_request(|| { Ok(octocrab::OctocrabBuilder::new().personal_token(lookup_pat()?).build()?) }, &self.cancel_url, None::<()>)?;
        Ok(())
    }

    pub fn completed(&self) -> bool {
        self.status == "completed"
    }

    pub fn refresh(&self) -> Result<Self> {
        Ok(serde_json::from_str(&send_get_request(|| new_crab(None), &self.url)?)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Enabled {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BranchProtections {
    pub url: String,
    // pub required_signatures: ?,
    // pub enforce_admins
    pub required_linear_history: Enabled,
    pub allow_force_pushes: Enabled,
    pub allow_deletions: Enabled,
    pub required_conversation_resolution: Enabled,
    pub lock_branch: Enabled,
    pub allow_fork_syncing: Enabled,
}

impl BranchProtections {
    pub fn is_locked(&self) -> bool {
        self.lock_branch.enabled
    }
}

pub fn send_get_request<F>(crab: F, uri: &str) -> Result<String>
where
    F: Fn() -> Result<octocrab::Octocrab>,
{
    with_blocking_calls( || {
        let c = crab()?;
        log_trace!("Sending GET request to GA: {}", uri);
        let response = futures::executor::block_on(c._get(uri))?;
        let body = futures::executor::block_on(c.body_to_string(response))?;
        log_trace!("Received pre-processed body:\n{}", body);
        Ok(body)
    })
}

pub fn send_post_request<F, H>(crab: F, uri: &str, inputs: Option<H>) -> Result<String>
where
    F: Fn() -> Result<octocrab::Octocrab>,
    H: serde::Serialize + Sized
{
    with_blocking_calls( || {
        let c = crab()?;
        log_trace!("Sending POST request to GA: {}", uri);
        let response = block_on!(c._post(uri, inputs.as_ref()))?;
        let body = block_on!(c.body_to_string(response))?;
        log_trace!("Received pre-processed body:\n{}", body);
        Ok(body)
    })
}

pub fn send_put_request<F, H>(crab: F, uri: &str, inputs: Option<H>) -> Result<String>
where
    F: Fn() -> Result<octocrab::Octocrab>,
    H: serde::Serialize + Sized
{
    with_blocking_calls( || {
        let c = crab()?;
        log_trace!("Sending POST request to GA: {}", uri);
        let response = block_on!(c._put(uri, inputs.as_ref()))?;
        let body = block_on!(c.body_to_string(response))?;
        log_trace!("Received pre-processed body:\n{}", body);
        Ok(body)
    })
}

pub enum GithubAuth {
    PersonalAccessToken
}

pub fn new_crab(auth: Option<GithubAuth>) -> Result<octocrab::Octocrab> {
    Ok(if let Some(a) = auth {
        match a {
            GithubAuth::PersonalAccessToken => octocrab::Octocrab::builder().personal_token(lookup_pat()?).build()?
        }
    } else {
        octocrab::OctocrabBuilder::new().build()?
    })
}

pub fn get_latest_workflow_dispatch(owner: &str, repo: &str, workflow: Option<&str>) -> Result<WorkflowRun> {
    let mut uri = "https://api.github.com/repos".to_string();
    if let Some(w) = workflow {
        uri = format!("{}/{}/{}/actions/workflows/{}/runs", uri, owner, repo, w);
    } else {
        uri = format!("{}/{}/{}/actions/runs", uri, owner, repo);
    }
    uri += "?per_page=1";
    let body = send_get_request(|| new_crab(None), &uri)?;
    let runs = serde_json::from_str::<WorkflowRuns>(&body)?;
    runs.get_only()
}

pub fn get_workflow_run_by_id(owner: &str, repo: &str, run_id: u64) -> Result<WorkflowRun> {
    Ok(serde_json::from_str(&send_get_request(
        || new_crab(None),
        &format!("https://api.github.com/repos/{}/{}/actions/runs/{}", owner, repo, run_id)
    )?)?)
}

pub fn dispatch_workflow<H>(
    owner: &str,
    repo: &str,
    workflow: &str,
    git_ref: &str,
    inputs: Option<H>,
) -> Result<Outcome>
where
    H: serde::Serialize + Sized
{
    with_blocking_calls(|| {
        let crab = new_crab(Some(GithubAuth::PersonalAccessToken))?;
        let actions = crab.actions();
        let mut workflow = actions.create_workflow_dispatch(owner, repo, workflow, git_ref);
        if let Some(ins) = inputs.as_ref() {
            workflow = workflow.inputs(serde_json::json!(ins));
        }
        // TODO PublishingO2 Add checks for errors in response
        block_on!(workflow.send())?;
        Ok(())
    })?;
    let res = Outcome::new_success();
    Ok(res)
}

pub fn get_branch_protections(owner: &str, repo: &str, branch: &str) -> Result<BranchProtections> {
    let resp = &send_get_request(
        || new_crab(Some(GithubAuth::PersonalAccessToken)),
        &format!("https://api.github.com/repos/{owner}/{repo}/branches/{branch}/protection")
    )?;
    match serde_json::from_str(resp) {
        Ok(retn) => Ok(retn),
        Err(e) => {
            bail!("Error building branch protection struct: {}\nUnexpected response:\n{}", e, resp);
        }
    }
}

#[derive(Serialize, Debug)]
pub struct UpdateBranchProtectionRule {
    pub lock_branch: bool,
    pub enforce_admins: bool,
    pub required_pull_request_reviews: Option<HashMap<String, String>>,
    pub required_status_checks: Option<HashMap<String, String>>,
    pub restrictions: Option<HashMap<String, String>>,
}

impl UpdateBranchProtectionRule {
    pub fn lock_branch() -> Self {
        Self {
            lock_branch: true,
            enforce_admins: true,
            required_pull_request_reviews: None,
            required_status_checks: None,
            restrictions: None,
        }
    }

    pub fn unlock_branch() -> Self {
        Self {
            lock_branch: false,
            enforce_admins: false,
            required_pull_request_reviews: None,
            required_status_checks: None,
            restrictions: None,
        }
    }
}

pub fn update_branch_protection(owner: &str, repo: &str, branch: &str, new_protections: UpdateBranchProtectionRule) -> Result<BranchProtections> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/branches/{branch}/protection");
    let res = send_put_request(
        || new_crab(Some(GithubAuth::PersonalAccessToken)),
        &url,
        Some(new_protections),
    )?;
    match serde_json::from_str(&res) {
        Ok(retn) => Ok(retn),
        Err(e) => {
            bail!("Error building branch protection struct: {}\nUnexpected response:\n{}", e, res);
        }
    }
}

pub fn lock_branch(owner: &str, repo: &str, branch: &str) -> Result<BranchProtections> {
    update_branch_protection(owner, repo, branch, UpdateBranchProtectionRule::lock_branch())
}

pub fn unlock_branch(owner: &str, repo: &str, branch: &str) -> Result<BranchProtections> {
    update_branch_protection(owner, repo, branch, UpdateBranchProtectionRule::unlock_branch())
}