# Test Rule matcher conformance

Test Rule is a local diagnostic dry run. It does not send network traffic. Its
supported comparisons call leaf matcher primitives from the exactly pinned
`apimock-routing` 5.10.0 crate. The repository's 5.10.1 GUI integration
reference remains the intended M3 contract, but no reproducible 5.10.1 engine
artifact was available when RFC MK-052 adopted the executable M2 oracle.

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

Body Regex is unavailable because the 5.10.0 implementation performs literal
substring containment despite the regex operation name. Test Rule does not
present containment as regex behavior.

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
independent conformance review are recorded.
