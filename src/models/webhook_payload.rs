
// Event Types
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Push,
    PullRequest,
}

// Pull request action type
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestAction {
    Opened,
    Closed,
    Assigned,
    Reopened,
    Synchronize,
}

// Root model
#[derive(serde::Deserialize, Debug)]
pub struct WebhookPayload {
    #[serde(rename = "ref")]
    pub(crate) git_ref: Option<String>,
    action: Option<PullRequestAction>,
    pub(crate) repository: Option<Repository>,
    pusher: Option<Pusher>,
    organization: Option<Organization>,
    created: bool, // new branch/tag created
    deleted: bool, // branch/tag deleted
    forced: bool, // force-push happened

}


#[derive(serde::Deserialize, Debug)]
pub(crate) struct Repository {
    pub(crate) name: String,
    pub(crate) full_name: String,
    pub(crate) private: bool,
    pub(crate) git_url: String,
    pub(crate) ssh_url: String,
}

#[derive(serde::Deserialize, Debug)]
struct Organization {
    pub(crate) login: String,
    pub(crate) id: u32,
}

#[derive(serde::Deserialize, Debug)]
struct Pusher {
    name: String,
    email: String,
}

#[derive(serde::Deserialize, Debug)]
struct PullRequest {
    url: String,
    id: u32,
    title: String,
    number: u32,
    merged: bool, // If action == closed => {true => merged | false => closed}
    mergeable: bool,

}