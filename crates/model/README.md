# apimokka-model

UI-facing workspace, rendering, validation, and trace data types for apimokka.

The crate does not mirror an executable apimock-rs crate. Its authoritative
editing boundary is the local `WorkspacePort` contract adopted by RFC MK-053:

- `EditIntent` and `EditTransaction` describe typed UI mutations;
- `PortSnapshot` carries canonical rule/condition views beside a legacy render
  projection;
- `EditOutcome`, `SaveOutcome`, and `SaveFailure` return complete post-attempt
  snapshots and typed effects; and
- `MemoryWorkspace` implements the contract without filesystem or server I/O.

The mapping is based on the documented apimock-rs 5.10.1 GUI integration
reference. That version was never published. RFC MK-055 adopted the real
`apimock-config` 5.10.0 crate as a **test-only dev-dependency**
(`tests/engine_conformance/`, pinned by
`scripts/check-engine-oracle.sh`) and verified the mapping by executing
against it — not merely by reading its source. This crate claims conformance
to the reviewed local mapping and both the in-memory and 5.10.0-executed
contract suites, not source or binary compatibility with an engine
implementation: `MemoryWorkspace` remains the only implementation, and no
production adapter has been built.

Older `EditCommand`, `WorkspaceSnapshot`, `RulePayload`, `RespondPayload`,
`SaveResult`, and trace types remain as mockup render/prototype vocabulary.
They are deliberately not the canonical adapter contract, and lossy render
values must not be used to reconstruct canonical state or history.

Known `ReferenceGap`s are documented in
[`docs/src/architecture.md`](../../docs/src/architecture.md#reference-gap-inventory).
