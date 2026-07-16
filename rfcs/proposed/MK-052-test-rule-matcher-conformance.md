# RFC MK-052 — Test Rule matcher conformance

**Status.** Proposed
**Tracks.** Stabilization roadmap M2 — Match-test conformance.
**Touches.** Test Rule evaluation and result types, matcher dependency and
provenance, dialog rendering and EN/JA copy, conformance tests, capability
documentation, changelog, and M2 evidence.

## Summary

Replace the Test Rule dialog's best-effort evaluator with a fail-closed
conformance adapter. Supported conditions call the matcher primitives from an
exactly pinned `apimock-routing` 5.10.0 dependency. Conditions which cannot be
verified against that executable engine return `Unsupported`; malformed input
or invalid rule configuration returns `Error`. Neither category may be reduced
to `Matched` or `NoMatch`.

The repository's GUI integration reference names apimock-rs 5.10.1, but no
5.10.1 tag, published routing crate, or immutable engine source was available
during this design investigation. The latest reproducible executable baseline
is the published `apimock-routing` 5.10.0 crate. Its API and behavior do not
implement every operation described by the 5.10.1 reference. This RFC therefore
uses 5.10.0 only as the M2 executable matcher oracle and keeps 5.10.1 as the
intended M3 integration contract. It does not invent the missing semantics.

The initial verified surface is four configured HTTP method constraints, five
of six URL operators, five of nine header operators, and seventeen of eighteen
body operators. A configured PATCH constraint, URL `EndsWith`, header
`EndsWith`/`Regex`/`Exists`/`Absent`, and body `Regex` are explicitly
unsupported. PATCH remains a valid test-request method: for example, configured
GET against request PATCH is a supported, verified non-match. The model retains
unavailable choices so existing rule editing data is not destroyed, but Test
Rule explains why it cannot verify them.

## Context

`App::run_stub_test` currently contains behavior that can produce false results:

- URL `WildCard` always passes;
- header `Regex` and `WildCard` pass whenever the header merely exists;
- body `Regex` always passes;
- `EqualInteger` is evaluated through `f64`, losing precision above 2^53; and
- missing headers are not evaluated with engine semantics.

The public result type has only `Matched`, `NoMatch`, and `Error(String)`, so the
UI cannot distinguish a verified non-match from an unavailable evaluator. This
violates the M2 requirement that no skipped or best-effort branch produce a
match or non-match.

### Executable-baseline investigation

The design investigation established this reproducible baseline:

| Evidence | Observation |
|---|---|
| Upstream tag | Highest available tag: `5.10.0` |
| Upstream commit | `9a220b27cd6058bc5be3ae43d983c61509dfbcb4` |
| Published crate | `apimock-routing` 5.10.0 |
| Crate SHA-256 | `72118fbc81807a3a3e511ec638b3fc798b5eee035c8d287158ae487763003cf1` |
| Declared Rust version | 1.91, matching this workspace's MSRV |
| Missing artifact | no 5.10.1 tag or published `apimock-routing` 5.10.1 crate |

The 5.10.0 source exposes `RuleOp::is_match`, `BodyOperator::is_match`,
`HttpMethod::is_match`, its glob matcher, and dotted-path JSON resolution. Its
`RuleOp` has five variants. Its `HttpMethod` has GET, POST, PUT, and DELETE. Its
body `Regex` variant performs literal substring containment despite its name and
comments, whereas the 5.10.1 integration reference describes regex intent.

The version and checksum above are design provenance, not evidence that a new
dependency has already been installed. Implementation must obtain the crate
through Cargo, commit the resulting lockfile entry, and re-observe its checksum.

## Goals

1. Prevent unsupported, skipped, malformed, or failed evaluations from being
   reported as a verified match or non-match.
2. Reuse the adopted engine's matcher primitives instead of implementing a
   second matcher and testing it against itself.
3. Give every supported operator positive and negative executable coverage.
4. Preserve exact `i64` behavior, including values above 2^53.
5. Make support limitations visible before and after Test Rule execution in EN
   and JA.
6. Record enough engine provenance to reproduce and intentionally upgrade the
   oracle.
7. Keep M2 separate from the M3 editing-boundary decision.

## Non-goals

- Claim conformance to an unavailable apimock-rs 5.10.1 executable.
- Implement regex, new header operators, PATCH support, or other missing engine
  behavior locally.
- Change the rule model or remove unavailable operator choices.
- Define the production workspace adapter, server process, file I/O, trace
  transport, or command mapping; those belong to M3 and later work.
- Change apimock-rs or publish a replacement routing crate.
- Make the full routing engine or `RuleSet::find_matched` the Test Rule boundary.
  That would prematurely choose M3's model mapping.
- Resolve pre-existing clippy or audit findings assigned to M4, except for a new
  finding introduced by this dependency change.
- Create a release, commit, tag, archive, or push.

## Decision

### 1. Pin and call the real matcher

Implementation adds these exact workspace dependencies, with no semver range:

```toml
apimock-routing = "=5.10.0"
http = "=1.4.2"
```

The app's conformance adapter maps supported GUI operations to the public
5.10.0 matcher primitives and calls them in production. It may perform request
input parsing and result aggregation locally, but it must not reproduce glob,
string, numeric, typed JSON, array, presence, or exact-integer comparison logic.

`http::Method::from_bytes`, `http::HeaderName`, and `http::HeaderValue` are the
authoritative request-input parsers/types. A direct `http` dependency is required
rather than relying on apimock-routing's transitive Hyper dependency. Hyper 1
re-exports the same `http::Method` type expected by the engine's public matcher.

The exact deep public engine imports are:

```rust
apimock_routing::rule_set::rule::when::request::http_method::HttpMethod
apimock_routing::rule_set::rule::when::request::rule_op::RuleOp
apimock_routing::rule_set::rule::when::request::body::body_operator::BodyOperator
apimock_routing::util::json::json_value_by_jsonpath
```

These paths are an accepted version-pinned integration risk, not a stable API
claim beyond 5.10.0. A path change is handled by the same explicit upgrade rule
as a behavior change.

The full `RuleSet` matcher is not reused because constructing engine rule sets
from the GUI model would establish the editing mapping that M3 is scheduled to
decide. M2 reuses only leaf matcher semantics and records the small orchestration
boundary explicitly.

`Cargo.lock` is the installed-version authority. The implementation review
package records the resolved version and checksum and compares them with the
provenance table. A version change, feature change, patch, fork, or source
override requires an RFC amendment and a fresh conformance review.

### 2. Separate verification outcomes

The new domain result is structurally typed rather than a display string:

```rust
pub struct TestRuleResult {
    pub outcome: TestRuleOutcome,
    pub conditions: Vec<ConditionResult>,
    pub diagnostics: Vec<RequestDiagnostic>,
}

pub enum TestRuleOutcome {
    Matched,
    NoMatch,
    Unsupported,
    Error,
}

pub struct ConditionResult {
    pub condition: ConditionIdentity,
    pub outcome: ConditionOutcome,
}

pub enum ConditionOutcome {
    Passed,
    Failed,
    Unsupported { reason: UnsupportedReason },
    Error { reason: EvaluationError },
}

pub struct RequestDiagnostic {
    pub scope: DiagnosticScope,
    pub reason: EvaluationError,
}
```

`ConditionIdentity` names Method, URL path, header index/name, or body
index/path. `DiagnosticScope` names Selection, RequestMethod,
RequestHeaderLine(line number) or RequestBody.
Condition errors remain attached to the relevant condition; request/global
errors use `diagnostics`. Reasons are enums with data, not pre-localized strings.
The screen maps them to localized copy. Debug detail may be derived for tests,
but user copy does not become application state.

All conditions are evaluated where safe so the dialog can show a useful list.
The aggregate precedence is mandatory:

1. any request diagnostic or condition `Error` produces
   `TestRuleOutcome::Error`;
2. otherwise, any `Unsupported` produces `TestRuleOutcome::Unsupported`;
3. otherwise, any `Failed` produces `TestRuleOutcome::NoMatch`;
4. otherwise the outcome is `Matched`.

This means a failed supported condition does not hide an unsupported sibling.
The request as a whole was not fully verifiable, so `NoMatch` would overclaim.

### 3. Distinguish unsupported behavior from invalid data

`Unsupported` means the selected operation has no accepted executable oracle.
Its capability classification is deterministic and independent of request
parsing, but it does not suppress demanded-family validation. The reason
identifies the condition, operation, adopted oracle version, and the limitation
in stable terms suitable for EN/JA rendering.

`Error` means evaluation could have been supported, but the supplied request or
configured condition is invalid or ambiguous. At minimum this includes:

- a malformed request-header line or invalid header name/value;
- a duplicate request-header name, because the current text form does not
  define whether duplicates are joined, overwritten, or independently matched;
- invalid request JSON when at least one selected body condition needs it;
- a malformed configured value for `EqualTyped`, `ArrayContains`, numeric,
  exact-integer, or array-length operations; and

The engine leaf functions sometimes return `false` for malformed configured
values. Test Rule is a diagnostic workflow, so a preflight validation error is
more precise and cannot create a false match or non-match. Valid inputs still
use the engine primitive for the actual comparison. Validation rules must be
covered separately from matching rules.

An empty body is an input error when any selected body condition requires JSON.
Body `Exists` and `Absent` still require valid JSON because their question is
path presence within a JSON value.

Input families are demand-parsed after selection succeeds. The request method is
always parsed because every Test Rule input represents an HTTP request. Header
text is ignored when the selected rule has no header conditions and otherwise
the complete header block is validated, including irrelevant names. Body text is
ignored when the rule has no body conditions and otherwise must be valid JSON,
even when all selected body operations are unsupported. Therefore malformed JSON
plus body `Regex` aggregates to Error, not Unsupported. A selection failure
produces only the Selection diagnostic because no rule-owned demand can be
established safely.

Diagnostics have deterministic order: Selection first when present; otherwise
RequestMethod, RequestHeaderLine in ascending one-based line order,
RequestBody, then condition results in Method, URL, header-vector order, and
body-vector order. Duplicate header
diagnostics attach to the later line and name the first line.

A malformed request method or demanded request body produces its authoritative
request diagnostic without a second derivative condition error. A supported
method or body condition that needs the unavailable parsed value is omitted;
configured validation errors and unsupported conditions remain independent and
are retained. This keeps the report complete without repeating one input fault.

### 4. Request-header grammar

When header input is demanded:

- split with Rust `str::lines`; a line whose ASCII-space/tab-trimmed content is
  empty is ignored;
- split each nonblank line at its first ASCII colon, preserving later colons in
  the value;
- trim only ASCII space and tab around the name and at both ends of the value;
- validate the name with `http::HeaderName` and value with `http::HeaderValue`,
  then require `HeaderValue::to_str` because the engine leaf matcher accepts
  `&str`; a validated but non-visible-ASCII value is Error rather than an
  invented byte-to-text conversion;
- allow an empty validated value;
- normalize names through `HeaderName` and reject case-insensitive duplicates;
  no joining, overwriting, or multi-value interpretation is performed; and
- retain the normalized name and validated value bytes for engine-compatible
  comparison.

Line numbers refer to the original text including blank lines. Missing colons,
empty/invalid names, invalid values, and duplicates are request diagnostics.

### 5. Configured-value validation

Validation is deliberately precise and bounded:

- `EqualTyped` and `ArrayContains` expected values must parse as one complete
  `serde_json::Value`; malformed JSON is Error. This intentionally rejects
  5.10.0 `ArrayContains`'s malformed-JSON string fallback.
- Numeric expected values must parse as finite `f64`; NaN and positive/negative
  infinity are Error. Resolved request values, including numeric strings, are
  passed to the engine unchanged; its conversion/non-match behavior remains the
  oracle.
- `EqualInteger` expected values must parse in `i64::MIN..=i64::MAX`; fractions
  and out-of-range values are Error. Invalid or out-of-range resolved request
  values remain verified non-matches through the engine.
- Array-length expected values must parse as `usize` for the current compilation
  target, hence `0..=usize::MAX`; negative, fractional, whitespace-padded, and
  out-of-range values are Error. Implementation uses `parse::<usize>()` so its
  accepted lexical form matches the engine.

Each validation boundary has accepted-minimum, accepted-maximum, malformed, and
out-of-range tests where applicable.

### 6. Preserve engine request semantics

- Multiple conditions are ANDed subject to the aggregate precedence above.
- HTTP methods are compared through the engine `HttpMethod` primitive.
- `url_path_op: None` is an absent URL constraint and emits no condition result;
  it is not displayed as a synthetic Passed condition.
- Header names are parsed case-insensitively. Values remain case-sensitive.
- For supported header operations, a missing named header is `Failed`, including
  `NotEqual`, matching the engine's header-condition short circuit.
- The engine glob grammar is authoritative: `*` matches zero or more characters
  and `?` matches one Unicode scalar value.
- Body paths use the engine's dotted-path resolver. `Exists` passes for any
  resolved value including `null`; `Absent` passes only when resolution fails.
- Body `EqualInteger` delegates to the engine's `i64` matcher. It never shares
  the `f64` path used by `EqualNumber`.
- A rule with zero conditions other than its supported method/URL fields is
  evaluated normally; there is no implicit success for a skipped collection.

## Operator conformance matrix

“Supported” means the production adapter calls the named 5.10.0 primitive.
“Unsupported” means Test Rule returns only the explicit unsupported outcome.

### HTTP method

Configured rule method and Test Rule request method are separate domains. An
empty configured `RulePayload.method` is Any/no constraint. Nonempty configured
and request strings are validated without whitespace trimming by
`http::Method::from_bytes`. After syntactic validation, configured
GET/POST/PUT/DELETE are recognized ASCII-case-insensitively and mapped to the
engine variants. Configured PATCH or any other syntactically valid but unmapped
standard or extension method has no 5.10.0 configured-method variant and is
Unsupported. The request method may
be any syntactically valid standard or extension method because the engine
primitive compares a configured variant with a general `http::Method`.
The current Test Rule dialog offers GET, POST, PUT, PATCH, and DELETE request
buttons rather than free-form entry. Other valid methods remain evaluator inputs
for restored/internal state and executable conformance coverage.

| Configured rule method | Same valid request | Other standard request, including PATCH | Unrelated valid extension request | Malformed request |
|---|---|---|---|---|
| empty (Any) | Passed | Passed | Passed | Error: RequestMethod |
| GET/POST/PUT/DELETE | Passed through corresponding `HttpMethod::is_match` | Failed through `is_match` | Failed through `is_match` | Error: RequestMethod |
| PATCH | Unsupported | Unsupported | Unsupported | Error takes precedence over Unsupported |
| other valid standard or extension method | Unsupported | Unsupported | Unsupported | Error takes precedence over Unsupported |
| malformed nonempty configured value | Error: Method condition | Error: Method condition | Error: Method condition | both method errors retained; aggregate Error |

Configured Any still emits a Passed Method condition so the diagnostic order is
stable. A valid configured name's case does not affect recognition. The 5.10.0
engine comparison lowercases both sides, so a valid request extension token such
as lowercase `get` passes configured GET even though HTTP method tokens are
normally case-sensitive; an unrelated extension such as PURGE fails. M2 records
and tests the executable engine behavior rather than silently correcting it.
Required tests cover every configured standard variant plus Any, PATCH, a valid
unknown standard/extension method, and malformed text against the request categories in this
table, including lowercase-standard request tokens.

### URL path

| GUI operator | M2 state | 5.10.0 oracle |
|---|---|---|
| Equal | Supported | `RuleOp::Equal::is_match` |
| StartsWith | Supported | `RuleOp::StartsWith::is_match` |
| Contains | Supported | `RuleOp::Contains::is_match` |
| EndsWith | Unsupported | no `RuleOp` variant |
| WildCard | Supported | `RuleOp::WildCard::is_match` |
| NotEqual | Supported | `RuleOp::NotEqual::is_match` |

### Header

| GUI operator | M2 state | 5.10.0 oracle |
|---|---|---|
| Equal | Supported | `RuleOp::Equal::is_match` |
| Contains | Supported | `RuleOp::Contains::is_match` |
| StartsWith | Supported | `RuleOp::StartsWith::is_match` |
| EndsWith | Unsupported | no `RuleOp` variant |
| Regex | Unsupported | no accepted regex primitive |
| Exists | Unsupported | no `RuleOp` variant |
| Absent | Unsupported | no `RuleOp` variant |
| NotEqual | Supported | `RuleOp::NotEqual::is_match` after engine-compatible presence check |
| WildCard | Supported | `RuleOp::WildCard::is_match` |

### JSON body

| GUI operator | M2 state | 5.10.0 oracle |
|---|---|---|
| Equal | Supported | `BodyOperator::Equal::is_match` |
| EqualString | Supported | `BodyOperator::EqualString::is_match` |
| Contains | Supported | `BodyOperator::Contains::is_match` |
| StartsWith | Supported | `BodyOperator::StartsWith::is_match` |
| EndsWith | Supported | `BodyOperator::EndsWith::is_match` |
| Regex | Unsupported | 5.10.0 performs literal containment; conflicts with reference intent |
| EqualTyped | Supported | `BodyOperator::EqualTyped::is_match` |
| ArrayContains | Supported | `BodyOperator::ArrayContains::is_match` |
| EqualNumber | Supported | `BodyOperator::EqualNumber::is_match` |
| GreaterThan | Supported | `BodyOperator::GreaterThan::is_match` |
| LessThan | Supported | `BodyOperator::LessThan::is_match` |
| GreaterOrEqual | Supported | `BodyOperator::GreaterOrEqual::is_match` |
| LessOrEqual | Supported | `BodyOperator::LessOrEqual::is_match` |
| EqualInteger | Supported | `BodyOperator::EqualInteger::is_match` |
| ArrayLengthEqual | Supported | `BodyOperator::ArrayLengthEqual::is_match` |
| ArrayLengthAtLeast | Supported | `BodyOperator::ArrayLengthAtLeast::is_match` |
| Exists | Supported | engine dotted-path resolution plus `BodyOperator::Exists` |
| Absent | Supported | engine dotted-path resolution plus `BodyOperator::Absent` |

The unsupported list is a capability statement, not a permanent product
decision. Support may expand only after an executable engine artifact with the
required primitive is explicitly adopted and its positive/negative matrix is
reviewed.

## Architecture and file boundaries

The implementation extracts matching from the already oversized reducer:

```text
crates/app/src/match_test.rs
crates/app/src/match_test/input.rs
crates/app/src/match_test/engine.rs
crates/app/src/match_test/result.rs
crates/app/src/match_test/tests.rs
crates/app/src/match_test/tests/input.rs
crates/app/src/match_test/tests/matrix.rs
crates/app/src/match_test/tests/body.rs
crates/app/src/match_test/tests/aggregation.rs
crates/app/src/match_test/tests/screen.rs
docs/src/match-test-conformance.md
```

`match_test.rs` owns orchestration, capability lookup, and preflight. `result.rs`
owns domain results, condition identities, diagnostics, and aggregation.
`input.rs` parses the dialog request. `engine.rs` contains only audited
GUI-to-engine enum mappings and leaf calls. The files in the block are the
expected new tracked-file inventory. Test modules remain
outside implementation modules and may be consolidated if the implementation
review inventory explains the final path and each file stays cohesive and within
repository size guidance. Any other new tracked file must be added to that
inventory before review.

`App::update` delegates Test Rule execution to this module. `run_stub_test` and
its best-effort helpers and inline tests are removed. `message.rs` imports the
domain result rather than defining a UI-only string result.

`screens/test_rule.rs` renders the aggregate result, request/global diagnostics,
and compact per-condition outcomes in the stable order defined above. It does
not recompute capability or matching decisions. The dialog
shows an “Unable to verify” warning before execution whenever the selected rule
contains an unsupported method or operator, and the Run action remains
available so the user can obtain the complete diagnostic list. Unsupported is
not styled as success or ordinary non-match and is not conveyed by color alone.

New user-visible labels, reasons, and summaries receive both English and
Japanese keys. Tests verify key presence and screen rendering; exact prose may
be refined without changing the result enum.

## Conformance test design

The production dependency is also the external oracle. Tests must not compare a
local matcher copy with the adapter. Instead they exercise GUI enum mapping and
aggregation while independently invoking the corresponding public engine
primitive for expected leaf results.

### Required matrix

Every supported method/operator has at least one passing and one failing case.
Every unsupported method/operator has a case proving the aggregate can only be
`Unsupported`, including when another condition fails. The suite additionally
covers:

- the complete configured-method/request-method categories in the HTTP matrix,
  including configured Any, every supported standard constraint, configured
  PATCH, valid unknown standard/extension methods, and malformed values;
- wildcard `*`, `?`, repeated wildcards, empty matches, and Unicode scalars;
- header-name case folding, case-sensitive values, missing headers including
  `NotEqual`, blank lines, first-colon splitting, ASCII whitespace, empty values,
  invalid names/values, case-folded duplicates, and original line numbers;
- string coercion versus typed JSON equality;
- numeric JSON values, numeric strings, non-numeric values, and invalid expected
  values, including configured NaN and infinities;
- exact integers `9007199254740992`, `9007199254740993`, adjacent non-matches,
  `i64::MIN`, `i64::MAX`, fractional values, and out-of-range values;
- dotted object and array paths, missing paths, and resolved `null`;
- array typed containment, malformed `ArrayContains` JSON, and both length
  operations at zero, `usize::MAX`, malformed, negative, and out-of-range values;
- empty or malformed JSON when body evaluation is required;
- ignored malformed header/body text when the selected rule has no condition in
  that family, and demanded validation when it has supported or unsupported
  conditions;
- Selection, RequestMethod, RequestHeaderLine, and RequestBody diagnostics plus
  their deterministic ordering with condition results;
- multiple-condition AND behavior and Error > Unsupported > Failed > Passed
  aggregation precedence; and
- an invariant test that enumerates `UrlPathOp::all`, `HeaderOp::all`, and
  `BodyOp::all` so a newly added model variant cannot silently bypass the
  capability table.

Expected matcher behavior is derived from the pinned engine calls. Assertions
also name stable known cases so an upstream behavior change cannot be normalized
away by changing both sides unnoticed. The exact dependency pin and checksum
make that upstream change an explicit repository diff.

Screen-flow tests cover Matched, NoMatch, Unsupported, and Error in both
locales, plus keyboard dismissal and a rule with multiple displayed issues.

## Documentation and disclosure

Implementation adds a short `docs/src/match-test-conformance.md` capability
page and links it from the documentation navigation. It records:

- the adopted 5.10.0 executable oracle and why it differs from the 5.10.1
  integration reference;
- every supported and unsupported method/operator;
- wildcard, missing-header, dotted-path, numeric, and exact-integer semantics;
- the distinction between NoMatch, Unsupported, and Error; and
- the process for adopting a newer engine artifact.

The README claim that Test Rule evaluates all nine header and nineteen body
operations is replaced with an honest summary and a link to the matrix. The
body model has eighteen operations, so the incorrect count is removed even
apart from support status. Any existing dialog copy that implies complete
evaluation is updated in EN and JA. `CHANGELOG.md` records the change under
Unreleased after implementation.

## Upgrade rule

An executable 5.10.1 or later engine does not become the oracle merely because
it exists. Adoption requires:

1. immutable source/version/checksum provenance;
2. confirmation that its declared Rust version satisfies the workspace policy;
3. an updated operator matrix based on public executable primitives;
4. focused positive and negative tests for changed behavior;
5. documentation and UI capability updates; and
6. independent conformance review.

If the future artifact's body `Regex` behavior remains literal containment, it
remains unsupported under that name unless the integration contract is also
revised. M2 must not label substring behavior as regex.

## Implementation sequence

1. Add the exact dependencies, resolve the lockfile, and record
   versions/checksums.
2. Add domain results, capability tables, input parsing, and aggregation.
3. Map supported operations directly to engine matcher primitives.
4. Replace reducer stub evaluation and delete best-effort branches/tests.
5. Render all four outcomes and pre-run limitations with EN/JA copy.
6. Add the complete operator, boundary, aggregation, and screen-flow matrix.
7. Correct README claims and add the conformance page and changelog entry.
8. Run the focused suite, RFC integrity checks, and programme-wide gates.
9. Produce implementation evidence for independent review.

The sequence is small enough and the mappings are fully enumerated here, so a
separate developer handoff is not required. If implementation is delegated
across multiple developers or the accepted design changes to full `RuleSet`
reuse, create a handoff before coding.

## Verification and gates

Focused verification includes the match-test module tests, relevant app
screen-flow tests, i18n checks, and the RFC integrity checker. Milestone evidence
also observes, without overclaiming, the programme commands:

```sh
cargo fmt --check
cargo test --workspace --lib --bins --locked
cargo build --workspace --locked
cargo +1.91 test --workspace --lib --bins --locked
cargo +1.91 build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
bash scripts/check-rfcs.sh
bash scripts/check-rfcs-self-test.sh
git diff --check
```

Before implementation review, compare the actual new tracked files with the
expected inventory above. For every still-untracked file, run and record:

```sh
git diff --no-index --check /dev/null -- <new-file>
```

No whitespace diagnostics are permitted; exit status 1 is expected because the
files differ. `git diff --check` remains required for tracked/index-visible
changes. The implementation review package lists both the inventory comparison
and each observed no-index result, so new Rust, test, or documentation files are
not omitted from the whitespace gate.

Because `Cargo.lock` changes, `cargo audit` is mandatory. Pre-existing M1
clippy/audit failures remain truthful M4 inputs, but any new warning or advisory
introduced by `apimock-routing` blocks M2 exit unless remediated or accepted
through the repository-owned exception policy.

## Review and milestone closure

The lifecycle is intentionally staged:

1. independent review accepts this design or requests changes;
2. the project owner explicitly authorizes implementation and names the delivery
   person before product files or dependencies change;
3. implementation review checks the accepted design, executable matrix, gates,
   documentation, and dependency provenance;
4. after implementation acceptance, a closure-candidate patch moves this RFC to
   `done/`, sets `Implemented (Unreleased)`, updates the index, and leaves M2 `In
   review` with the implementation-review evidence path and closure pending;
5. an independent closure confirmation checks the candidate's lifecycle,
   status, index, gates, and roadmap agreement;
6. the project owner records evidence-approver acceptance of that closure
   confirmation; and
7. only then does a separate finalization patch change M2 from `In review` to
   `Complete` and record both accepted evidence paths, followed by the RFC
   checker, self-test, and tracked/new-file whitespace checks.

The finalization patch is mechanical and cannot add product, dependency, RFC
design, or capability changes. If it does, M2 remains `In review` and targeted
independent review is required. Design acceptance alone does not make an
implementation or completion claim.

## Alternatives considered

### Implement all reference operations locally

Rejected. There is no executable 5.10.1 oracle against which to prove exact
equivalence, and local regex/glob/numeric code tested against local expectations
would repeat the current self-oracle problem.

### Treat 5.10.0 body Regex as supported containment

Rejected. The operator name and integration reference promise regex behavior;
returning substring results would be predictably misleading.

### Call the full 5.10.0 RuleSet matcher

Deferred to M3. It would require choosing a GUI-to-engine editing mapping and
could erase the distinction between Test Rule conformance and production rule
serialization.

### Keep only Error and encode unavailability as an error string

Rejected. Unsupported capability is stable and actionable, while malformed
input is exceptional. A distinct variant permits unambiguous UI, tests, and
telemetry without parsing prose.

### Disable the Run action for unsupported rules

Rejected. Users benefit from seeing every unsupported or invalid condition in
one diagnostic run. The action remains available, but it cannot return a
verified match/non-match until every condition is supported and valid.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| 5.10.0 differs from intended 5.10.1 | Name both contracts, limit M2 to the executable baseline, and require explicit future adoption |
| Dependencies increase build/security surface | Exact pins, lockfile checksums, MSRV build, audit, and review |
| Adapter mapping drifts when model variants change | Exhaustive capability mapping plus `all()` coverage tests |
| Unsupported sibling is hidden by an early failure | Evaluate safely and enforce aggregate precedence |
| Validation diverges from engine false-return behavior | Limit preflight to invalid/ambiguous inputs and test it separately; valid matching still calls engine |
| UI becomes noisy for many conditions | Compact condition list, stable identities, aggregate summary, accessible non-color state |
| M2 accidentally decides M3 | Reuse leaf matchers only; exclude serialization and full RuleSet construction |

## Open review decisions

The independent reviewer and project owner must explicitly confirm:

1. 5.10.0 is acceptable as the reproducible M2 executable oracle while 5.10.1
   remains the intended M3 integration reference;
2. the configured-method/request-method distinction and supported/unsupported
   matrix are conservative enough, especially configured PATCH, header presence
   operations, and body `Regex`;
3. `Error > Unsupported > Failed > Passed` is the correct fail-closed aggregate
   order; and
4. request/global diagnostics, demand parsing, header grammar, and stable
   diagnostic ordering are sufficiently explicit; and
5. configuration preflight errors should remain distinct from the engine's
   leaf-level `false` behavior for malformed expected values.

Implementation is not authorized by creation of this proposed RFC.
