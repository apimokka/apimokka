//! Recent workspace cards shown on Welcome / Dashboard.

pub fn recent_workspaces() -> Vec<RecentWorkspace> {
    vec![
        RecentWorkspace {
            name: "shop-api-mock".into(),
            path: "~/projects/shop-api-mock".into(),
            last_opened: "today".into(),
            pinned: true,
        },
        RecentWorkspace {
            name: "billing-sandbox".into(),
            path: "~/work/billing-sandbox".into(),
            last_opened: "yesterday".into(),
            pinned: false,
        },
        RecentWorkspace {
            name: "qa-edge-cases".into(),
            path: "~/qa/edge-cases".into(),
            last_opened: "3 days ago".into(),
            pinned: false,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct RecentWorkspace {
    pub name: String,
    pub path: String,
    pub last_opened: String,
    pub pinned: bool,
}
