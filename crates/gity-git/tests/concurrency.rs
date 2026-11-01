use gity_git::{CreateFileCommitInput, GitService};

fn input(path: String) -> CreateFileCommitInput {
    CreateFileCommitInput {
        branch_name: "main".into(),
        file_path: path,
        content: "content".into(),
        message: "Concurrent addition".into(),
        author_name: "Tester".into(),
        author_email: "test@example.test".into(),
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_file_commits_cannot_overwrite_successful_writers() {
    let root = tempfile::tempdir().unwrap();
    let service = GitService::new(root.path(), "git");
    service.init_bare("repository.git", "main").await.unwrap();
    service
        .create_file_commit("repository.git", input("base".into()))
        .await
        .unwrap();
    let mut tasks = Vec::new();
    for number in 0..8 {
        let service = service.clone();
        let path = format!("file-{number}");
        tasks.push(tokio::spawn(async move {
            let success = service
                .create_file_commit("repository.git", input(path.clone()))
                .await
                .is_ok();
            (path, success)
        }));
    }
    let mut successes = Vec::new();
    for task in tasks {
        let (path, success) = task.await.unwrap();
        if success {
            successes.push(path);
        }
    }
    assert!(!successes.is_empty());
    let tree = service
        .list_tree("repository.git", "main", "", "")
        .await
        .unwrap();
    assert_eq!(tree.as_array().unwrap().len(), successes.len() + 1);
    for path in &successes {
        assert!(service
            .file_exists("repository.git", "main", "", path)
            .await
            .unwrap());
    }
    assert_eq!(
        service
            .list_commits("repository.git", "main", "", 100)
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        successes.len() + 1
    );
}
