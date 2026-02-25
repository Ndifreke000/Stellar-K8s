History archives are the backbone of Stellar's decentralized history. We need a controller that periodically verifies the integrity of these archives by checking for stale ledgers or missing checkpoints.

✅ Acceptance Criteria
Add a new ArchiveCheck routine in the operator (or integrate into archive_health.rs).
Periodically (every 1 hour) download the stellar-history.json from the configured historyArchiveUrls.
Compare the ledger sequence in the archive with the actual network state.
If the archive is lagging significantly, update the StellarNodeStatus with a Degraded condition and fire a Prometheus alert.
Unit test the edge cases (archive unreachable, malformed JSON, sync lag detection).
📚 Resources
src/controller/archive_health.rs
Stellar History Archives
