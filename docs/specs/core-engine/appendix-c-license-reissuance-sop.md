# Appendix C

## License Reissuance Standard Operating Procedure

Back to the owning PRD: [Core Engine PRD](PRD.core-engine.md)

The owning PRD remains authoritative for product scope. This appendix clarifies supporting detail and must not broaden or contradict that scope. The binding requirement is [PRD §9.5](PRD.core-engine.md#95-offline-licensing--feature-gating) — that a documented reissuance path must exist and must not require altering existing records. This appendix describes one concrete process satisfying that requirement.

---

## 1. Purpose

Give vendor support and the application a concrete, repeatable process for re-licensing a barangay's installation after a hardware repair or replacement, without requiring any changes to that barangay's existing data (which is guaranteed independently by [Appendix B](appendix-b-key-derivation-and-recovery.md)'s key-derivation approach).

## 2. Process

**Step 1 — Barangay Request:**
On a new or repaired PC, the app detects a Machine Hardware ID mismatch against the currently installed license and displays: *"License Error: Hardware Change Detected. Machine ID: [NEW-MACHINE-ID]."* The Secretary transmits `[NEW-MACHINE-ID]` to vendor support via official email or message.

**Step 2 — Vendor Verification:**
Support staff verifies the barangay's active purchase record in the internal License Registry, then revokes the old Machine ID in that registry.

**Step 3 — Key Regeneration:**
Support executes an internal signing tool, e.g.:

```
bms-keygen --brgy "BRGY-POBLACION-001" --machine "NEW-MACHINE-ID" --features "CORE,KP,TREASURY"
```

and transmits the newly signed Base64 key string back to the barangay.

**Step 4 — Activation:**
The Secretary pastes the new key into Settings → License Activation. Access is unlocked instantly. Because the database encryption key ([Appendix B](appendix-b-key-derivation-and-recovery.md)) does not depend on Machine Hardware ID, existing SQLite database records remain fully readable without alteration.

## 3. Boundary Guidance

- This process is support-driven and manual by design, consistent with the offline-first product principle — it must not require the barangay's PC to contact any server.
- The internal `bms-keygen` tool and License Registry are vendor-side operational tooling, not part of the barangay-facing product; their implementation belongs to internal tooling, not this PRD package.

## 4. Usage Guidance

Downstream TDDs implementing the licensing engine should treat this SOP as the expected support workflow their hardware-mismatch error state must trigger, and should confirm the error message and activation flow match [PRD §10](PRD.core-engine.md#10-api-interaction-or-workflow-requirements)'s license-activation interaction.
