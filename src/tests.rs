#[cfg(test)]
mod tests {
    use opendal_core::Operator;
    use rustic_backend::local::{LocalConfig, LocalSource};
    use rustic_backend::BackendOptions;
    use rustic_core::repofile::SnapshotFile;
    use rustic_core::{BackupOptions, ConfigOptions, Credentials, KeyOptions, PathList, Repository, RepositoryOptions};
    use std::fs;
    use tempfile::tempdir;
    use crate::config::RusticVfsConfig;

    /// Strips the leading separator so `/tmp/foo` becomes `tmp/foo`,
    /// matching rustic's file-structure layout inside the VFS.
    fn vfs_path(host_path: &std::path::Path) -> String {
        host_path
            .to_string_lossy()
            .trim_start_matches('/')
            .replace('\\', "/")
            .replace(':', "")
    }

    #[tokio::test]
    async fn backup_and_read_through_vfs() {
        // ── 1. Repository ────────────────────────────────────────────────────
        let repo_dir = tempdir().expect("repo tempdir");
        let local = LocalConfig::new(repo_dir);

        let backends = BackendOptions::default()
            .with_repo(&local)
            .to_backends()
            .expect("backends");

        let repo = Repository::new(&RepositoryOptions::default(), &backends)
            .unwrap()
            .init(
                &Credentials::Password("testing123456!".into()),
                &KeyOptions::default(),
                &ConfigOptions::default(),
            )
            .unwrap()
            .to_indexed_ids() // need IDs to run a backup
            .unwrap();

        // ── 2. Source data ───────────────────────────────────────────────────
        let src_dir = tempdir().expect("source tempdir");
        let src_file = src_dir.path().join("hello.txt");
        fs::write(&src_file, b"hello from backup").expect("write source file");

        // ── 3. Backup ────────────────────────────────────────────────────────
        let paths = PathList::from_iter([src_dir.path()]);
        let mut file = SnapshotFile::default();
        file.hostname = "testvm".into();
        file.label = "test".into();
        file = repo.backup(&BackupOptions::default(), &LocalSource::new(paths), file)
            .expect("backup");

        // ── 4. VFS operator ──────────────────────────────────────────────────
        let opts = RusticVfsConfig {
            backend: BackendOptions::default().with_repo(&local),
            credentials: Some(Credentials::Password("testing123456!".into())),
            ..Default::default()
        };

        let op = Operator::from_config(opts).expect("VFS init").finish();

        // ── 5. Assert snapshots/latest exists ────────────────────────────────
        let root = format!("/[{}]/[{}]", &file.hostname, &file.label);
        let latest_meta = op
            .stat(&format!("{}/latest", &root))
            .await
            .expect("/snapshots/latest should exist");

        assert!(latest_meta.is_dir(), "snapshots/latest must be a directory");

        // ── 6. Assert the backed-up file is reachable ────────────────────────
        // Rustic stores /tmp/<hash>/hello.txt as  tmp/<hash>/hello.txt
        let relative = format!("{}/latest/{}/hello.txt", &root, vfs_path(src_dir.path()));

        let file_meta = op
            .stat(&relative)
            .await
            .unwrap_or_else(|_| panic!("expected file at VFS path: {relative}"));
        assert!(file_meta.is_file(), "entry should be a regular file");

        // ── 7. Read and verify content ───────────────────────────────────────
        let bytes = op
            .read(&relative)
            .await
            .unwrap_or_else(|_| panic!("failed to read VFS path: {relative}"));

        assert_eq!(bytes.to_bytes(), "hello from backup".as_bytes());
    }
}
