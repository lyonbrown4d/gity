use super::*;
#[test]
fn accepts_only_git_services_and_safe_quoted_or_absolute_repository_paths() {
    for command in [
        "git-upload-pack 'team/demo.git'",
        "git-upload-pack '/team/demo.git'",
        "git-upload-pack team/demo.git",
    ] {
        assert_eq!(
            parse(command.as_bytes()).unwrap(),
            (GitRpc::UploadPack, "team/demo".into())
        );
    }
    assert_eq!(
        parse(b"git-receive-pack 'team/demo.git'").unwrap().0,
        GitRpc::ReceivePack
    );
    for command in [
        "sh -c id",
        "git-upload-pack '../team/demo.git'",
        "git-upload-pack 'team/demo.git';id",
        "git-upload-pack 'team//demo.git'",
        "git-upload-pack 'team/demo.git\n'",
        "git-upload-pack 'team/--upload-pack.git'\\x",
        "git-upload-pack team/demo.git\0",
    ] {
        assert!(parse(command.as_bytes()).is_err(), "{command}");
    }
}
