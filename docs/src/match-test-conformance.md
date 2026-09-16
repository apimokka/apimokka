# Test Rule matcher conformance

Test Rule is a local diagnostic dry run. It does not send network traffic. Its
supported comparisons call leaf matcher primitives from the **`apimock-routing`
6.0.0** crate fixed in `Cargo.lock`. The manifest accepts the 6.x compatibility
line, but a later resolved artifact is not an adopted matcher until its
provenance and conformance evidence are reviewed — `scripts/check-matcher-oracle.sh`
pins the version and checksum so a drift cannot pass unreviewed.

**Provenance, corrected 2026-09-16.** RFC MK-052 originally adopted
`apimock-routing` 5.10.0, because the repository's intended M3 contract was a
5.10.1 GUI integration reference that **was never published** and had no
reproducible artifact. RFC MK-055 (M7) then adopted the real `apimock-config`
5.10.0 as a test-only dev-dependency and verified the MK-053 editing boundary —
not matching — against it by execution.

**RFC MK-060 (M11) superseded both on 2026-09-05**, carrying `apimock-routing`
and `apimock-config` to **6.0.0** directly, past nine unused 5.x minors. That
retires the unpublished-reference problem entirely: 6.0.0 ships a documented
library API with a stability statement, so the contract is an artifact rather
than prose. MK-052 and MK-055 were both re-run against it. Production source
changed by zero lines; see
[Architecture § Contract provenance](./architecture.md#contract-provenance).

Test Rule's matching contract below is unaffected by the config crate —
`apimock-config` does not own request matching.

**One boundary this document should state.** Our match-test represents request
header values as UTF-8 `String`s, so a header value that is not valid UTF-8
cannot be expressed here at all. apimock-rs 6.1.0 (RFC 072) changed the engine's
behaviour for exactly that input — such a condition used to match
unconditionally and now fails closed. The change is unreachable from this
surface, so it is a **limit of what we can specify**, not a divergence. Recorded
2026-09-16 so the conformance claim is read with its edge visible.

Test Rule fails closed:

- **Matched** — every applicable condition was evaluated and passed.
- **No match** — every condition was evaluable and at least one failed.
- **Unable to verify** — at least one condition has no adopted matcher primitive.
- **Error** — request input or configured rule data is invalid or ambiguous.

The aggregate order is Error, Unable to verify, No match, then Matched. A failed
supported condition cannot hide an unsupported condition and produce a
misleading No match.

## Capability matrix

Configured method constraints are different from request methods. GET, POST,
PUT, and DELETE constraints are supported. A configured PATCH, HEAD, OPTIONS,
CONNECT, TRACE, or extension method is unavailable in the adopted engine and is
reported as Unable to verify. A request may use any syntactically valid standard
or extension method: configured GET against request PATCH is a verified No
match. An empty configured method means Any and passes every valid request
method.

The current dialog exposes GET, POST, PUT, PATCH, and DELETE request buttons.
Other valid methods can still reach the evaluator through restored or internal
state, but the dialog does not currently provide free-form method entry.

| Family | Supported | Unable to verify |
|---|---|---|
| Configured method | Any, GET, POST, PUT, DELETE | PATCH and every other valid but unmapped standard/extension constraint |
| URL path | Equal, StartsWith, Contains, WildCard, NotEqual | EndsWith |
| Header | Equal, Contains, StartsWith, NotEqual, WildCard | EndsWith, Regex, Exists, Absent |
| JSON body | Equal, EqualString, Contains, StartsWith, EndsWith, EqualTyped, ArrayContains, EqualNumber, GreaterThan, LessThan, GreaterOrEqual, LessOrEqual, EqualInteger, ArrayLengthEqual, ArrayLengthAtLeast, Exists, Absent | Regex |

Body Regex is unavailable because the engine's implementation performs literal
substring containment despite the regex operation name. Test Rule does not
present containment as regex behavior.

**Re-derived against 6.0.0 at M11** (RFC MK-060) and unchanged. 6.0.0 added
`Regex`/`NotRegex` to `UrlPathOp` and `NotRegex` to `HeaderOp`, but `BodyOp`'s
seven additions are the four `Not*` families, `MapHasKey`, `MapDoesNotHaveKey`
and `StructuralContains` — **no new body regex operator**, so the reason above
still holds. Every verdict in the matrix above was re-derived at that upgrade;
none moved. Our operator enums remain strict subsets of the engine's in all
three families, so this surface cannot express a condition the engine would
reject.

## Input and matching details

- Wildcard uses the engine grammar: `*` matches zero or more characters and `?`
  matches one Unicode scalar value.
- Header names are case-insensitive and values are case-sensitive. Missing
  headers fail every supported header comparison, including NotEqual.
- Header input is one `name: value` entry per line. The first colon separates
  name and value; blank lines are ignored. Duplicate names are errors rather
  than being joined or overwritten.
- Body input must be valid JSON when the selected rule has body conditions.
- A malformed request method or demanded body produces one authoritative
  request diagnostic. A supported condition that depends on that unavailable
  parsed value is omitted rather than adding a derivative condition error;
  independent configured errors and unsupported conditions are still retained.
- Body paths use dotted object keys and numeric array segments, such as
  `user.id` or `items.2.name`. This is not JSONPath.
- Exists passes for a resolved value including `null`; Absent passes only for a
  missing path.
- EqualNumber uses the engine's `f64` semantics. Configured numeric values must
  be finite.
- EqualInteger uses exact `i64` matching and preserves integers above 2^53,
  including `9007199254740993`.
- EqualTyped and ArrayContains configured values must be valid JSON. Invalid
  typed values are errors instead of silent non-matches or string fallbacks.
- An unused URL field emits no synthetic condition result. Header/body request
  text is ignored when the selected rule has no condition in that family.

Unavailable operations remain editable so the mockup does not destroy rule
data. The dialog displays a warning before running and lists the unavailable
conditions after running. A later engine version expands this matrix only after
its immutable version/checksum, MSRV, executable behavior, tests, UI copy, and
independent conformance review are recorded. Programme gates use `--locked` so
the build uses the checked-in resolution, and the matcher-oracle guard verifies
that the checked-in versions, registry sources, checksums, and activated
features still match the reviewed contract. A compatible lockfile update cannot
pass that guard silently.
