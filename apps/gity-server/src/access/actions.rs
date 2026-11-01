/// Project operations have separate user-role and project-token scope rules.
#[derive(Clone, Copy, Debug)]
pub enum ProjectAction {
    Read,
    Write,
    Delete,
    RepositoryRead,
    RepositoryPush,
    RepositoryAdmin,
    IssueCreate,
    IssueWrite,
    IssueComment,
    MergeRequestCreate,
    MergeRequestWrite,
    MergeRequestComment,
    MergeRequestMerge,
    PackageRead,
    PackageWrite,
    WikiRead,
    WikiWrite,
    JobRead,
    JobWrite,
    RunnerRead,
    RunnerAdmin,
}
impl ProjectAction {
    pub(super) fn minimum_role(self) -> i64 {
        match self {
            Self::Read | Self::RepositoryRead | Self::PackageRead | Self::WikiRead => 0,
            Self::IssueCreate
            | Self::IssueComment
            | Self::MergeRequestComment
            | Self::JobRead
            | Self::RunnerRead => 20,
            Self::Write
            | Self::RepositoryPush
            | Self::IssueWrite
            | Self::MergeRequestCreate
            | Self::MergeRequestWrite
            | Self::PackageWrite
            | Self::WikiWrite
            | Self::JobWrite => 30,
            Self::RepositoryAdmin | Self::MergeRequestMerge | Self::RunnerAdmin => 40,
            Self::Delete => 50,
        }
    }
    pub(super) fn token_scopes(self) -> &'static [&'static str] {
        match self {
            Self::Read | Self::RepositoryRead => &[
                "read_repository",
                "write_repository",
                "read_api",
                "write_api",
            ],
            Self::RepositoryPush => &["write_repository"],
            Self::PackageRead => &["read_package", "write_package"],
            Self::PackageWrite => &["write_package"],
            Self::IssueCreate
            | Self::IssueWrite
            | Self::IssueComment
            | Self::MergeRequestCreate
            | Self::MergeRequestWrite
            | Self::MergeRequestComment
            | Self::WikiRead
            | Self::JobRead
            | Self::RunnerRead => &["read_api", "write_api"],
            Self::Write | Self::WikiWrite | Self::JobWrite => &["write_api"],
            Self::Delete | Self::RepositoryAdmin | Self::MergeRequestMerge | Self::RunnerAdmin => {
                &[]
            }
        }
    }
}
