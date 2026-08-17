# Hexon Architecture Decision Records

This directory contains the definitive Hexon Architecture Decision Records (HADRs)
for the reusable Rust Hexon baseline. Application-specific Architecture Decision
Records (ADRs) belong in [`docs/adr/`](../../docs/adr/README.md).

## Reading Order

Read the baseline in identifier order. It moves from specification authority through
architecture, crate boundaries, construction, boundary policy, and the five-gate
reference delivery workflow.

| HADR | Title | Status | Relationships |
| --- | --- | --- | --- |
| [HADR-0001](HADR-0001-govern-product-and-implementation-specifications.md) | Govern Product and Implementation Specifications | Accepted | Specification authority |
| [HADR-0002](HADR-0002-adopt-rust-hexon-architecture.md) | Adopt Rust Hexon Architecture | Accepted | Architecture and composition model |
| [HADR-0003](HADR-0003-define-crate-and-module-responsibilities.md) | Define Crate and Module Responsibilities | Accepted | Crate, module, and privacy model |
| [HADR-0004](HADR-0004-build-and-test-capabilities-from-the-core-outward.md) | Build and Test Capabilities From the Core Outward | Accepted | Core-outward construction order |
| [HADR-0005](HADR-0005-define-boundary-and-consistency-policies.md) | Define Boundary and Consistency Policies | Accepted | Boundary and consistency policy |
| [HADR-0006](HADR-0006-govern-core-delivery-through-three-approval-gates.md) | Govern Core Delivery Through Three Approval Gates | Accepted | Completes specification and core review mechanics; precedes HADR-0007 |
| [HADR-0007](HADR-0007-govern-contract-authority-and-external-delivery-through-two-approval-gates.md) | Govern Contract Authority and External Delivery Through Two Approval Gates | Accepted | Depends on HADR-0006; retains HADR-0002 through HADR-0005 |

`HADR-0008` is permanently reserved because the identifier exists in Git history.
Identifiers are never reused; the next HADR is `HADR-0009`.

## When to Write a HADR

Write a HADR when a decision intended for every derived service
materially affects architecture, crate boundaries, contracts, construction, testing,
or governance. Product scope belongs in an application PRD. Application implementation
design belongs in a technical design document. Application-only architecture decisions
belong in `docs/adr/`.

## Creation and Lifecycle

1. Copy `template.md` to `HADR-NNNN-short-kebab-case-title.md`.
2. Allocate the next four-digit identifier after the highest issued or reserved HADR;
   identifiers in this baseline are never reused. `HADR-0008` is reserved, so the next
   identifier is `HADR-0009`.
3. Default an unresolved record to `Proposed`.
4. Project maintainers accept, reject, or withdraw proposals through review.
5. Update this index and every affected contract in the same accepted change.

| Status | Meaning |
| --- | --- |
| `Proposed` | Under discussion and not authoritative |
| `Accepted` | Authoritative for its stated scope |
| `Rejected` | Deliberately not adopted |
| `Withdrawn` | Removed from consideration without a merits decision |
| `Deprecated` | Still applicable but being phased out |
| `Retired` | No longer applicable because its capability was removed |
| `Superseded` | Replaced by a linked HADR |

Accepted and rejected HADRs are durable evidence. Change an accepted decision with a
new HADR and synchronize its relationships, this index, and affected contracts.