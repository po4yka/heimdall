# CloudKit Sync — Runbook

Operational guide for the CloudKit-backed snapshot sync that powers the
Heimdall iPhone dashboard and shared-zone collaboration between Macs.

## Containers

| Environment | Container identifier |
|---|---|
| Production | `iCloud.dev.heimdall.heimdallbar` |
| Development (unsigned builds) | disabled by `MacPlatformFactory.shouldEnableCloudKitSnapshotSync` |

Both macOS and iOS apps target the same container; the iOS app joins
snapshots published by one or more Macs via zone shares.

Dashboard: <https://icloud.developer.apple.com/dashboard/> — select the
team, then the container above.

## Architecture at a glance

- **Custom zone:** `heimdall-sync-space` in the user's private database.
- **Sync engine:** `SnapshotCloudEngine` (actor) owns one `CKSyncEngine`
  per container, configured against the private database.
  `macos/HeimdallBar/Services/Sources/SnapshotCloudEngine.swift`
- **Record type:** `SyncedInstallationSnapshot` — one record per Mac,
  record name = the Mac's installation ID (Keychain-stored,
  reinstall-safe). Schema centralised in
  `macos/HeimdallBar/Services/Sources/SnapshotCloudRecord.swift`.
- **Payload encryption:** `record.encryptedValues["payload"]` (per-field
  E2E, iOS 15+ / macOS 12+). Metadata fields
  (`installationID`, `sourceDevice`, `publishedAt`) stay plain so
  CloudKit can index/query them.
- **Engine state:** serialized via
  `FileBackedCloudKitSyncEngineStateStore` to
  `~/Library/Application Support/HeimdallSync/sync-engine-state.json`
  (macOS) or the iOS app sandbox equivalent.
- **Sharing:** owner Macs create a zone-level `CKShare` with
  `publicPermission = .readOnly`; participants read from
  `sharedDatabase` via raw `CKDatabase.records(...)` (CKSyncEngine does
  not manage shares).
- **Account observer:** `CloudKitAccountObserver` watches
  `Notification.Name.CKAccountChanged` and purges cached aggregate on
  sign-in / sign-out / account switch.

## CloudKit Console metrics & alert thresholds

Configure in the CloudKit Console → Telemetry → Alerts:

| Metric | Threshold | Rationale |
|---|---|---|
| Operation error rate | > 2% over 10 min | Transient spikes are normal; sustained errors indicate schema drift or outage |
| p95 latency (modifyRecords) | > 3 s | Saves are small; anything slower signals backend degradation |
| Storage quota | > 80% | Snapshot records are small; quota growth means accidental mass writes |
| `quotaExceeded` per hour | > 0 | Should never happen at steady state |

Use `Group By: platform` to detect iOS/macOS divergence early.

When an alert fires, drill from Telemetry → "Query in Logs" to see the
originating userID and operation scope.

## Schema evolution rules

**CloudKit schemas in production are effectively append-only.** Apple's
docs warn that removing or retyping a field that is live in production
can break older clients indefinitely.

- **Add fields** freely as optional. Older clients ignore them.
- **Never remove** a field that was ever saved in production.
- **Never retype** a field — bump to a new field name instead.
- **Never reuse** a record type or field name for a different purpose.
- **Add** new record types rather than repurposing existing ones.
- **Bump** `contractVersion` on `SyncedInstallationSnapshot` when the
  encoded payload's JSON schema changes; older readers should soft-fail
  on decode and fall back to the last-known-good snapshot.

Field names currently in use: see
`SnapshotCloudRecord.Field` in `SnapshotCloudRecord.swift`.

## Expected event flow

```
Mac app save → SnapshotCloudEngine.saveInstallationSnapshot
            → engine.state.add(pendingRecordZoneChanges: [.saveRecord])
            → engine.sendChanges()
                → nextRecordZoneChangeBatch (delegate callback)
                → handleEvent(.sentRecordZoneChanges)
            → handleEvent(.stateUpdate) → persist serialization

iPhone silent push (UIBackgroundModes=remote-notification)
   → MobileAppDelegate.didReceiveRemoteNotification
   → MobileDashboardModel.refresh(reason: .manual)
   → SnapshotCloudEngine.fetchInstallationSnapshots
   → engine.fetchChanges()
   → handleEvent(.fetchedRecordZoneChanges) — cache updated
   → handleEvent(.stateUpdate) — persist serialization
```

## Common issues

### `CKError.changeTokenExpired` spike

Meaning: the server change token CloudKit gave us for incremental fetch
has been rotated (usually after a long idle or an account recovery).

Recovery path is automatic with `CKSyncEngine` — the engine nils the
token internally and does a full zone re-fetch on the next
`fetchChanges` call. If you see rates above the baseline sustained for
more than an hour, suspect:

1. Account recovery on a user's iCloud — check that `accountChange`
   events are firing (search logs for `subsystem:dev.heimdall.heimdallbar
   category:CloudKit`).
2. App reinstall storm — expected bump after a release; should subside
   within a day.
3. Clock drift on the device — if the client clock is too far skewed
   CloudKit considers the token expired.

### User reports "my iPhone isn't syncing"

Triage checklist in order:

1. **Account status:** open macOS Settings → Debug → Cloud Sync
   Diagnostics (W3.3) and confirm `CKAccountStatus.available` on both
   devices. `.restricted` and `.noAccount` show clear messaging.
2. **Share acceptance:** the iPhone must have accepted the owner Mac's
   zone share (deep link from macOS Settings → Copy share link →
   Messages / AirDrop → tap on iPhone). Check
   `cloudSyncState.status == .participantJoined` in the debug panel.
3. **Push delivery:** iOS silent pushes are best-effort; ensure
   Background App Refresh is on for Heimdall in iOS Settings. Force a
   foreground refresh by pulling down on the dashboard.
4. **Quota:** unlikely given snapshot size, but check CloudKit Console
   → Telemetry → Storage.
5. **Schema drift:** after a release that added a `contractVersion`
   bump, older clients may silently drop payloads that don't decode.
   Check `category:CloudKit` for `decode failed` errors on the
   affected installation.

### New Mac won't publish after install

The installation ID now lives in the Keychain
(`KeychainInstallationIDStore`), but Keychain access requires a signed
build. Unsigned Debug builds run through
`MacPlatformFactory.shouldEnableCloudKitSnapshotSync` which bails out
when the bundle path contains `/.derived/`, `/DerivedData/`, or
`/Build/Products/Debug/`. This is intentional — local builds don't
pollute the user's real iCloud container.

## CKSyncEngine delegate lifecycle notes

Events arrive on the engine's internal actor; our
`SnapshotCloudEngine` is itself an actor, so mutations are naturally
serialized. Key rules:

- **Never call `engine.fetchChanges()` or `engine.sendChanges()` inside
  a delegate event handler.** This creates an infinite loop. Only call
  them from external triggers (user action, silent push, foreground).
- **Always persist `stateSerialization` on `.stateUpdate`.** Without
  this, the next launch re-fetches everything from scratch and ignores
  the server change token.
- **`accountChange` is coalesced** — it fires once on the first launch
  after a sign-in / sign-out, not mid-session.
- **One engine per database.** Running two `CKSyncEngine` instances
  against the same private database causes immediate conflicts. If you
  need to partition data, use separate zones inside one engine.

## Observability

Logs use `Logger(subsystem: "dev.heimdall.heimdallbar", category:
"CloudKit")`. To filter in Console.app:

```
subsystem:dev.heimdall.heimdallbar category:CloudKit
```

Notable log lines to watch:

- `SnapshotCloudEngine created for container …` — engine instantiation
- `event: accountChange — clearing local sync state` — iCloud account
  transition
- `save failed recordName=… code=N` — individual save errors; `N` is
  the raw `CKError.Code`
- `CKAccountChanged fired` — posted by the system, handled by
  `CloudKitAccountObserver`

See Apple's [CloudKit error codes reference](https://developer.apple.com/documentation/cloudkit/ckerror/code)
for translating numeric codes to cases.

## Related files

- `macos/HeimdallBar/Services/Sources/SnapshotCloudEngine.swift` — engine
- `macos/HeimdallBar/Services/Sources/SnapshotCloudRecord.swift` — schema
- `macos/HeimdallBar/Services/Sources/CloudKitSyncEngineStateStore.swift` — state persistence
- `macos/HeimdallBar/Services/Sources/CloudKitAccountObserver.swift` — account notifications
- `macos/HeimdallBar/Services/Sources/InstallationIDStore.swift` — Keychain-backed installation ID
- `macos/HeimdallBar/Services/Sources/CloudKitSnapshotSyncStore.swift` — high-level `SnapshotSyncStore` wrapper
- `macos/HeimdallBar/iOS/App/Sources/HeimdallMobileApp.swift` — silent-push handler
