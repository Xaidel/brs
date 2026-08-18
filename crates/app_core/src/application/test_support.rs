//! Test-only fakes and helpers for the application layer (`#[cfg(test)]`).
//!
//! `backend_arch_docs/testing.md`'s "test-local outbound fakes" rule: these
//! fakes perform no network, filesystem, database, or process I/O — they hold
//! configured results in memory and record calls.

use std::future::Future;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use async_trait::async_trait;
use base64::Engine as _;
use chrono::{TimeZone, Utc};

use crate::domain::value_objects::test_support::grouped;
use crate::domain::value_objects::{
    DatabaseEncryptionKey, LicenseKeyPayload, MachineHardwareId, RecoveryCode, Timestamp,
};
use crate::ports::{
    BackupDestination, BackupError, BackupSnapshotLocation, BackupSnapshotWriter, Clock,
    CredentialError, EncryptionCredentialGateway, HardwareIdError, LicenseGrantRecord,
    LicenseGrantRepository, LicenseSignatureVerifier, MachineHardwareIdSource, RepositoryError,
    SignatureVerificationError,
};

/// Runs an async test future to completion on the current thread.
///
/// `app_core` carries no async-runtime dependency, so tests poll by hand.
/// Test-local fakes are immediately ready (they never await real I/O), so the
/// future completes on its first poll; the bound turns a runaway future into a
/// test failure instead of a hang.
pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    for _ in 0..1_000_000 {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        std::hint::spin_loop();
    }
    panic!("test future did not complete: a fake or use case is awaiting real I/O");
}

/// A clock pinned to a fixed instant, for deterministic `activated_at`
/// assertions.
pub(crate) struct FakeClock {
    now: Timestamp,
}

impl FakeClock {
    /// A clock that always reports `now`.
    pub(crate) fn new(now: Timestamp) -> Self {
        Self { now }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

/// An in-memory `MachineHardwareIdSource` returning a configured result.
pub(crate) struct FakeMachineHardwareIdSource {
    result: Result<MachineHardwareId, HardwareIdError>,
    call_count: AtomicUsize,
}

impl FakeMachineHardwareIdSource {
    /// A source returning `result` from every `current()` call.
    pub(crate) fn returning(result: Result<MachineHardwareId, HardwareIdError>) -> Self {
        Self {
            result,
            call_count: AtomicUsize::new(0),
        }
    }

    /// The number of `current()` calls so far.
    pub(crate) fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl MachineHardwareIdSource for FakeMachineHardwareIdSource {
    async fn current(&self) -> Result<MachineHardwareId, HardwareIdError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}

/// An in-memory `LicenseSignatureVerifier` returning a configured result.
pub(crate) struct FakeLicenseSignatureVerifier {
    result: Result<(), SignatureVerificationError>,
    call_count: AtomicUsize,
}

impl FakeLicenseSignatureVerifier {
    /// A verifier that accepts every signature.
    pub(crate) fn succeeding() -> Self {
        Self::returning(Ok(()))
    }

    /// A verifier that rejects every signature.
    pub(crate) fn failing() -> Self {
        Self::returning(Err(SignatureVerificationError::InvalidSignature))
    }

    /// A verifier returning `result` from every `verify()` call.
    pub(crate) fn returning(result: Result<(), SignatureVerificationError>) -> Self {
        Self {
            result,
            call_count: AtomicUsize::new(0),
        }
    }

    /// The number of `verify()` calls so far.
    pub(crate) fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl LicenseSignatureVerifier for FakeLicenseSignatureVerifier {
    async fn verify(&self, _payload: &LicenseKeyPayload) -> Result<(), SignatureVerificationError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.result
    }
}

/// An in-memory `LicenseGrantRepository` with configured results and a call
/// log. `save` records only successful inserts, mirroring the port's
/// insert-only contract.
pub(crate) struct FakeLicenseGrantRepository {
    save_result: Result<(), RepositoryError>,
    find_result: Result<Option<LicenseGrantRecord>, RepositoryError>,
    saved: Mutex<Vec<LicenseGrantRecord>>,
    save_count: AtomicUsize,
    find_count: AtomicUsize,
}

impl FakeLicenseGrantRepository {
    /// A repository that accepts inserts and finds no current grant.
    pub(crate) fn new() -> Self {
        Self::with_results(Ok(()), Ok(None))
    }

    /// A repository whose `save()` returns `save_result`.
    pub(crate) fn with_save_result(save_result: Result<(), RepositoryError>) -> Self {
        Self::with_results(save_result, Ok(None))
    }

    /// A repository whose `find_current()` returns `find_result`.
    pub(crate) fn with_find_result(
        find_result: Result<Option<LicenseGrantRecord>, RepositoryError>,
    ) -> Self {
        Self::with_results(Ok(()), find_result)
    }

    /// A repository with fully configured results.
    pub(crate) fn with_results(
        save_result: Result<(), RepositoryError>,
        find_result: Result<Option<LicenseGrantRecord>, RepositoryError>,
    ) -> Self {
        Self {
            save_result,
            find_result,
            saved: Mutex::new(Vec::new()),
            save_count: AtomicUsize::new(0),
            find_count: AtomicUsize::new(0),
        }
    }

    /// The records successfully saved so far, in order.
    pub(crate) fn saved_records(&self) -> Vec<LicenseGrantRecord> {
        self.saved.lock().unwrap().clone()
    }
}

impl Default for FakeLicenseGrantRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LicenseGrantRepository for FakeLicenseGrantRepository {
    async fn save(&self, record: LicenseGrantRecord) -> Result<(), RepositoryError> {
        self.save_count.fetch_add(1, Ordering::SeqCst);
        if self.save_result.is_ok() {
            self.saved.lock().unwrap().push(record);
        }
        self.save_result
    }

    async fn find_current(&self) -> Result<Option<LicenseGrantRecord>, RepositoryError> {
        self.find_count.fetch_add(1, Ordering::SeqCst);
        self.find_result.clone()
    }
}

/// An in-memory `EncryptionCredentialGateway` with configured results and a
/// call log.
pub(crate) struct FakeEncryptionCredentialGateway {
    establish_result: Result<RecoveryCode, CredentialError>,
    recover_result: Result<DatabaseEncryptionKey, CredentialError>,
    received_recovery_code: Mutex<Option<String>>,
    establish_count: AtomicUsize,
    recover_count: AtomicUsize,
}

impl FakeEncryptionCredentialGateway {
    /// A gateway whose `establish()` returns `establish_result`.
    pub(crate) fn with_establish_result(
        establish_result: Result<RecoveryCode, CredentialError>,
    ) -> Self {
        Self::with_results(establish_result, Ok(database_key()))
    }

    /// A gateway whose `recover_database_key()` returns `recover_result`.
    pub(crate) fn with_recover_result(
        recover_result: Result<DatabaseEncryptionKey, CredentialError>,
    ) -> Self {
        Self::with_results(Ok(recovery_code()), recover_result)
    }

    /// A gateway with fully configured results.
    pub(crate) fn with_results(
        establish_result: Result<RecoveryCode, CredentialError>,
        recover_result: Result<DatabaseEncryptionKey, CredentialError>,
    ) -> Self {
        Self {
            establish_result,
            recover_result,
            received_recovery_code: Mutex::new(None),
            establish_count: AtomicUsize::new(0),
            recover_count: AtomicUsize::new(0),
        }
    }

    /// The number of `establish()` calls so far.
    pub(crate) fn establish_count(&self) -> usize {
        self.establish_count.load(Ordering::SeqCst)
    }

    /// The canonical form of the Recovery Code most recently passed to
    /// `recover_database_key()`, if any.
    pub(crate) fn received_recovery_code(&self) -> Option<String> {
        self.received_recovery_code.lock().unwrap().clone()
    }
}

#[async_trait]
impl EncryptionCredentialGateway for FakeEncryptionCredentialGateway {
    async fn establish(&self) -> Result<RecoveryCode, CredentialError> {
        self.establish_count.fetch_add(1, Ordering::SeqCst);
        self.establish_result.clone()
    }

    async fn load_database_key(&self) -> Result<DatabaseEncryptionKey, CredentialError> {
        unreachable!(
            "load_database_key is composition-root plumbing (§7.6), not exercised by Phase 1 use cases"
        )
    }

    async fn recover_database_key(
        &self,
        recovery_code: &RecoveryCode,
    ) -> Result<DatabaseEncryptionKey, CredentialError> {
        self.recover_count.fetch_add(1, Ordering::SeqCst);
        *self.received_recovery_code.lock().unwrap() = Some(recovery_code.as_str().to_string());
        self.recover_result
    }

    fn bootstrap_file_path(&self) -> String {
        unreachable!(
            "bootstrap_file_path is consumed by infra_backup (§4.7), not by Phase 1 use cases"
        )
    }
}

/// An in-memory `BackupSnapshotWriter` returning a configured result and
/// recording destinations.
pub(crate) struct FakeBackupSnapshotWriter {
    result: Result<BackupSnapshotLocation, BackupError>,
    destinations: Mutex<Vec<BackupDestination>>,
    call_count: AtomicUsize,
}

impl FakeBackupSnapshotWriter {
    /// A writer that confirms every snapshot.
    pub(crate) fn succeeding() -> Self {
        Self::returning(Ok(BackupSnapshotLocation {
            path: "C:\\BarangayMS\\backups\\initial.bmsbak".to_string(),
            created_at: "2026-08-18T00:00:00Z".to_string(),
        }))
    }

    /// A writer that fails every snapshot.
    pub(crate) fn failing() -> Self {
        Self::returning(Err(BackupError::WriteFailed))
    }

    /// A writer returning `result` from every `take_snapshot()` call.
    pub(crate) fn returning(result: Result<BackupSnapshotLocation, BackupError>) -> Self {
        Self {
            result,
            destinations: Mutex::new(Vec::new()),
            call_count: AtomicUsize::new(0),
        }
    }

    /// The number of `take_snapshot()` calls so far.
    pub(crate) fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    /// The destinations passed to `take_snapshot()` so far, in order.
    pub(crate) fn destinations(&self) -> Vec<BackupDestination> {
        self.destinations.lock().unwrap().clone()
    }
}

#[async_trait]
impl BackupSnapshotWriter for FakeBackupSnapshotWriter {
    async fn take_snapshot(
        &self,
        destination: BackupDestination,
    ) -> Result<BackupSnapshotLocation, BackupError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.destinations.lock().unwrap().push(destination);
        self.result.clone()
    }
}

/// A sample 13-group Machine Hardware ID.
pub(crate) fn machine_id() -> MachineHardwareId {
    MachineHardwareId::parse(&grouped(13)).unwrap()
}

/// A whole-object Base64 License Key envelope (`tdd.phase-1` §11) binding
/// `machine_hardware_id` to `flags`, carrying a placeholder 64-byte signature.
pub(crate) fn raw_license_key(machine_hardware_id: &str, flags: &[&str]) -> String {
    let obj = serde_json::json!({
        "machine_hardware_id": machine_hardware_id,
        "feature_flags": flags,
        "signature": base64::engine::general_purpose::STANDARD.encode([7u8; 64]),
    });
    base64::engine::general_purpose::STANDARD.encode(obj.to_string().as_bytes())
}

/// A valid all-zero Recovery Code (28 `0` data symbols, check symbol `0`).
pub(crate) fn recovery_code() -> RecoveryCode {
    RecoveryCode::parse("0000-0000-0000-0000-0000-0000-0000-0").unwrap()
}

/// A fixed derived key for assertions (not real key material).
pub(crate) fn database_key() -> DatabaseEncryptionKey {
    DatabaseEncryptionKey::from_bytes([7u8; 32])
}

/// A fixed UTC instant for deterministic clock and `activated_at` assertions.
pub(crate) fn timestamp() -> Timestamp {
    Timestamp::new(Utc.timestamp_opt(1_700_000_000, 0).unwrap())
}
