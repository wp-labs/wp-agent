use std::path::Path;

use super::{LogCheckpointState, ObservedFileIdentity, upsert_checkpoint};

#[test]
fn upsert_checkpoint_does_not_merge_fingerprint_only_entries_from_other_paths() {
    let observed_at = "2026-04-13T00:00:00Z";
    let mut state = LogCheckpointState::new("input-app".to_string(), observed_at.to_string());

    upsert_checkpoint(
        &mut state,
        Path::new("/tmp/a.log"),
        &ObservedFileIdentity {
            device_id: None,
            inode: None,
            fingerprint: Some("616263".to_string()),
            file_len: 3,
        },
        3,
        observed_at,
        None,
    );
    upsert_checkpoint(
        &mut state,
        Path::new("/tmp/b.log"),
        &ObservedFileIdentity {
            device_id: None,
            inode: None,
            fingerprint: Some("616263".to_string()),
            file_len: 3,
        },
        3,
        observed_at,
        None,
    );

    assert_eq!(state.files.len(), 2);
    assert_ne!(state.files[0].file_id, state.files[1].file_id);
}
