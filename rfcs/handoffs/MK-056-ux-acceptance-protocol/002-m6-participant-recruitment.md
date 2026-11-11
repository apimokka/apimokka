# Owner task 002 — M6 participant recruitment

**Issued:** 2026-09-16
**Issued by:** programme architect
**Governing design:** `rfcs/accepted/MK-056-ux-acceptance-protocol.md`, decision 4
**Blocks:** M6 exit, and therefore R2
**Delivery:** project owner — this is the one item nobody else can do

---

## What you are recruiting, in one line

**Two people who have never contributed to this repository**: one who has never
used apimock-rs, and one who has hand-written apimock-rs TOML.

That is the whole requirement. Everything below is how to find them, what to
say, and what to book.

---

## ⚠ Updated 2026-11-11 — nothing precedes the sessions any more

**This section used to say three defects came first.** They are long closed, and
so is everything that followed them. As of 2026-11-11:

- **cycles C1 and C2 are closed** (D-12, D-13, D-14 among them);
- **L2 is complete** — all four of decision 7's mandatory combinations have a
  recorded pass (dev-team task 037, four rounds, finished 2026-11-08);
- **the six defects that L2 preparation found are all closed** — D-29, D-30,
  D-31, D-32, D-33 and D-36, four of them S1, the last accepted 2026-11-09;
- **the open register is D-24 (S2), D-28 (S3) and D-35 (S3)**, every one of
  them explicitly deferred to after M6 under decision 8.

**So there is no engineering prerequisite left.** Recruiting was always
unblocked; now the sessions are too. The only thing between this repository and
a first session is the two people.

## Before you contact anyone — the open decision is now settled

**§0 of the session pack asked whether to reclaim L2 and run two sessions
instead of three. Events settled it: L2 was reclaimed and all four combinations
pass**, so participants carry **no screenshot-verification load** — the
sessions measure comprehension and nothing else.

**What L2 did not reclaim is the keyboard session.** MK-056's criterion names a
participant: *"every primary scenario completed by a participant … on both
pointer and keyboard paths"*, and no script stands in for one.

**Recommended shape — two people, three sittings, asked for once:**

| Sitting | Who | Length | What it covers |
|---|---|---:|---|
| S1 | Persona A (newcomer) | ~75 min | Guided, pointer, EN, large window — first boot |
| S2 | Persona B (expert) | ~75 min | Expert, pointer, JA, smallest window — first boot |
| S3 | **Persona B again**, after a break or on another day | ~40 min | keyboard-only, high contrast — the path L2 cannot cover |

S3 does not need first-boot naivety (it is spent by then), which is why it can
reuse a participant rather than cost a third recruit. Persona B is the
suggestion, not a rule — give it to whichever of the two is more comfortable
driving by keyboard. **Ask for both slots in the first invitation**: asking for
more later is worse than asking once.

---

## The two personas, and what actually distinguishes them

MK-056 decision 4 defines them. The distinction is **not** seniority or Rust
skill — it is whether they already hold apimock-rs's model in their head.

### Persona A — Guided newcomer

> Has not used apimock-rs; understands HTTP and JSON; has never seen this
> interface.

They exercise **Guided mode, first-launch, and the wizard**. Their value is that
they cannot fill a gap in the interface from prior knowledge. That is fragile
and spends on first contact — see "one-shot" below.

### Persona B — Expert apimock-rs user

> Has hand-written apimock-rs TOML.

They exercise **Expert mode, the rule builder, and Test Rule**, and they answer
the question only they can: **does the model this GUI presents match the model
the engine actually has?** A newcomer cannot detect a faithful-looking interface
that misrepresents the engine. This is the persona that makes M6 evidence about
correctness and not only about usability.

### The one hard exclusion

**Neither may have contributed to this repository.** Not code, not review, not
design discussion. Someone who has read our RFCs knows what the interface is
trying to do, which is exactly the knowledge we are testing the interface to
supply.

---

## Screening — four questions

Ask these before booking. They take a minute and they prevent the expensive
failure, which is discovering mid-session that you have the wrong persona.

| # | Question | Why it discriminates |
|---|---|---|
| 1 | "Have you used apimock-rs before? If yes, have you written its TOML by hand?" | The **only** question that assigns persona. "Used it via someone else's config" is Persona A, not B. |
| 2 | "Have you looked at the apimokka repository — code, RFCs, or issues?" | The exclusion. A yes disqualifies for either persona. |
| 3 | "Are you comfortable reading an HTTP request and a JSON body?" | Persona A's floor. Below it, findings are about HTTP, not about the interface. |
| 4 | "Do you normally work on Linux, and can you sit at a Linux desktop for the session?" | Sessions are Linux-only by MK-056's own decision — see below. Not a persona filter; a logistics one. |

**Do not screen for opinions about UI, design experience, or willingness to
give feedback.** A participant recruited for being opinionated produces
opinions; we want observed behaviour.

---

## What to tell them — and what not to

### Say

- It is a **mockup** of a desktop GUI for apimock-rs: it looks and behaves like
  the real thing but **opens no files and starts no server**. Nothing they do
  can damage anything.
- You want them to **do tasks and think aloud**. You will watch and take notes.
- **You are testing the interface, not them.** Where they get stuck is the
  finding; that is the point of the session.
- It takes **about 75 minutes** (or 60 — pending §0), and you will send the
  build details but nothing to prepare.
- They can stop at any time, and can ask you to strike anything from the notes.

### Do not say

- **Anything about what the interface can do, or what it is called.** Not
  "there's a command palette", not "it has Guided and Expert modes". The first
  screen of the application is itself under test.
- **Anything about text size, colour, or contrast.** MK-056's readability probe
  (decision 6 amendment) depends on the question being unprompted: asking
  teaches them to look for a problem they had not noticed, and the answer is
  then worthless.
- **Do not promise the findings will be acted on.** Some will be deferred under
  MK-056 decision 8, and saying otherwise is a promise you do not control.

---

## The one-shot rule, and why it constrains scheduling

**First-boot naivety cannot be recovered.** Once someone has seen the mode
picker and the first-launch flow, they cannot un-see it — not after a week, not
on a different machine. MK-056 makes this a rule, not a preference:
**first-boot scenarios run first, before anything spends it.**

Concretely:

- **Do not demo the application to them beforehand.** Not a screenshot, not "it
  looks like this".
- **Do not let them "have a quick look" while you set up.** Set up before they
  arrive, with the app not yet launched.
- **You have exactly two first-boot observations in the entire programme** — one
  per participant. Session 3, if run, cannot produce another.

This is the single easiest thing to destroy by accident and the single hardest
to replace.

---

## Logistics

| Item | Value | Source |
|---|---|---|
| Platform | **Linux only** | MK-056 rejected human sessions on all three platforms: L1 and L2 supply platform evidence, leaving sessions to measure comprehension, which does not vary materially by platform. Do not recruit for Windows or macOS. |
| Location | In person, or screen-share with audio | You must see hesitation and hear think-aloud. Text chat is not sufficient. |
| Duration | ~75 min per session, plus 15 min setup | Six primary scenarios with think-aloud. Pending §0. |
| Build | Record the **commit SHA** at session start | MK-056 decision 7: "A finding without its configuration is not reproducible and is not evidence." Head is `0570b18` as of 2026-11-11; **I freeze and confirm the identity once you give me a date**, so do not copy this one into a session log. |
| Recording | Optional, and ask | Notes are sufficient — the facilitator log is the evidence artifact, not a video. |
| Compensation | Your call | Not a protocol matter, but 75 minutes of an expert's time is a real ask. |

---

## How many people, and why not more

**Two is the designed minimum and is sufficient for M6's exit**, because MK-056
judges coverage **across entry states, not across headcount**:

> Coverage is judged across states, not across headcount. A scenario verified
> only from a populated, clean workspace is not verified for the empty or dirty
> case.

A third participant of either persona would add confidence to *interpretive*
findings — those escalate from S2 to S1 only on a second sighting — but it is
not required, and it costs a third booking. **Recruit two. Add a third only if
the first two produce contradictory interpretive findings**, which is a decision
you can take after session 2 with real information rather than now without any.

---

## What to hand back

Nothing formal. Tell me when you have:

1. **who** (persona A / persona B — names are not needed in the record);
2. **when**, so I can confirm the build is frozen and the session pack matches
   the tree they will see;
3. **the shape you booked** — whether S3 rides with one of the two sittings
   above or became a third booking.

I will then confirm the environment, freeze the build identity, and walk the
pack with you before the first session. **You facilitate; I do not attend** —
MK-056 requires participants who have not contributed to this repository, and
an architect in the room is a rescue waiting to happen.
