# HADR-0001: Govern Product and Implementation Specifications

- **Date**: 2026-07-20
- **Status**: Accepted
- **Deciders**: Project maintainers

## Context and Problem Statement

Architecture and implementation need distinct sources for durable product direction,
bounded requirements, and engineering design. Reusable Rust Hexon guidance also needs
an unambiguous boundary from the application built with it.

## Decision Drivers

- Separate vision, committed product scope, and implementation design.
- Prevent exploratory material from authorizing delivery.
- Give implementation slices explicit prerequisites and acceptance criteria.
- Keep reusable architecture separate from application documentation.
- Avoid artificial document and phase ceremony.

## Decision

We will govern delivery through this sequence:

```text
Master PRD (Vision) -> focused PRD (Normative)
                    -> technical design (Normative)
                    -> core-outward implementation -> acceptance evidence
```

The master PRD owns durable direction and does not authorize implementation. Every
initiative requires a normative focused PRD for bounded product requirements and a
normative owning technical design for implementation boundaries, contracts, tests,
and acceptance.

Use one complete design for a coherent capability. Use phases only for real
predecessor relationships, independently verifiable outcomes, material risk, or
separate handoffs. Every requirement is covered, deferred, excluded with
justification, or blocked by a recorded question.

`arch_docs/` owns reusable Rust Hexon architecture and decisions. `docs/` owns
application requirements, technical designs, decisions, and operations. Material
belongs in `arch_docs/` when it remains applicable after replacing the Notes domain,
HTTP API, and memory repository.

## Alternatives Considered

### Implement From Vision

- Benefits: minimal planning overhead.
- Costs and risks: exploratory direction becomes accidental scope and implementation
  acceptance remains implicit.

### Governed PRD and Technical-Design Pipeline

- Benefits: separates authority and creates reviewable delivery gates.
- Costs and risks: documents require maintenance and unjustified phasing adds ceremony.

## Consequences

### Positive

- Scope and implementation authority are explicit.
- Reusable architecture and application documentation have clear owners.

### Negative

- Meaningful changes require maintained specifications before coding.

### Neutral / Risks

- Status labels become ceremonial unless review verifies scope and evidence.

## Confirmation

- `docs/specs/README.md` defines the taxonomy and authority model.
- Each implementation initiative maps to normative product and design sources.
- Documentation follows the replacement test in this decision.

## References

- [Specification governance](../../docs/specs/README.md)
- [Development workflow](../development-workflow.md)
- [HADR-0004](HADR-0004-build-and-test-capabilities-from-the-core-outward.md)