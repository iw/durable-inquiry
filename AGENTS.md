# Durable Inquiry contributor guidance

These instructions are for coding agents working in this repository, and apply to all of
it. Read [README.md](README.md) first, then [docs/architecture.md](docs/architecture.md)
and [docs/knowledge.md](docs/knowledge.md) before changing a crate's boundaries or a
knowledge pack's format. [docs/decisions.md](docs/decisions.md) says why things are as
they are.

## What Durable Inquiry is

Durable Inquiry helps an operator find out why a running Temporal deployment is misbehaving. It
keeps competing hypotheses open, asks for the one piece of evidence that best tells them
apart, and traces every conclusion to the evidence behind it. Keep it that way:

- The engine has no Temporal-specific code. Metric names, settings, API fields,
  behaviours, thresholds, hypotheses, rules and requests are data, in the knowledge packs
  under `knowledge/temporal/`.
- Reasoning is deterministic. A model may reword an explanation, which is then checked
  against the assessment. It never decides a fact, a rule's outcome or a diagram's
  content.
- Durable Inquiry is not a dashboard and not a chatbot. It shows metrics inside Durable Inquiry, drawn as
  the matching Grafana panel would draw them, rather than sending the operator elsewhere
  to see them.

## Sources of truth

- Temporal's own sources, at the revisions each pack pins in `sources/sources.toml`: the
  server, the API, the SDKs and the CLI. A claim about Temporal is verified by reading
  that source, and cites the path, the line and a verbatim excerpt.
- Documentation is not verification. A claim known only from documentation is marked
  `unverified`, with the reason.
- Durable Inquiry's own thresholds are heuristics. They cite the `durable-inquiry-heuristics` source and
  are never presented as Temporal behaviour.
- [tsurdilo/temporal-server-operations](https://github.com/tsurdilo/temporal-server-operations)
  has no licence. Cite and link its runbooks, playbooks and dashboards; never copy their
  text or dashboards into this repository.

## Architecture boundaries

The Workspace table in the README says what each crate owns.

- `durable-inquiry-domain` is the pure model, with no I/O.
- Diagnosis runs in activities, never in workflow code, and commands reach an
  investigation as workflow updates (`durable-inquiry-odori`). The in-process service
  (`--in-process`) must reach the same assessments.
- `knowledge/temporal/*/generated/` is produced by the `durable-inquiry-extract` generators. Never
  edit it by hand: change the generator and regenerate.
- Durable Envision is used only through `durable-inquiry-sim`, pinned by Git revision. Don't copy its code.
- A new dependency is a design decision, and a Git dependency needs a pinned revision.

## Names from before the rename

Durable Inquiry was called Kyumei. A few names keep that spelling, because something
written earlier depends on each of them. Don't rename them:

- the sample block format's markers (`KYUMEI SAMPLES`, the magic `KYS`) and the request
  code's prefix (`kyq1.`), in `durable-inquiry-samples`: collectors and applications of
  different releases read each other's output;
- the task queues `kyumei-investigations` and `kyumei-agents` and the agent
  `kyumei-explainer`, which the engine's snapshot records for every investigation;
- the time zone cookie `kyumei_tz`, which browsers hold;
- the simulated scenario's namespace and Task Queue, `kyumei`: the simulator hashes
  names, so others would give other numbers.

Durable Envision is pinned to a commit where its package is named `tempdes`. The
dependency's `package` in `Cargo.toml`, the profile key and `Cargo.lock` keep that name
until the pin moves.

Elsewhere the earlier name appears only in the README's sentence about it, in "Upgrading
from Kyumei" in the operator guide, in the hints for a variable or directory left under
it, and in fixtures recorded then (`fixtures/recorded/` and the sample blocks of
`visibility-collected-blocks`), which stay as recorded. These are the exceptions to the
writing rule about history below.

## Knowledge changes

- A new or changed claim needs its verification method and date, and provenance with the
  path, line and excerpt from a pinned source. Its scope lists the releases it holds in.
- Run `durable-inquiry knowledge check`, and `durable-inquiry knowledge verify --source <id>=<path> …` for
  every source you have locally; [docs/knowledge.md](docs/knowledge.md) shows how.
- When a diagnosis changes, add or update a fixture under `fixtures/investigations/`.
  Every fixture replays in process and on Odori.
- After a change, recheck any count the docs quote: claims, citations, sources, fixtures.

## Writing

- Docs and comments describe the current state: no history of earlier versions, no dated
  anecdotes, no "since" or "used to". Rewrite the affected text rather than appending to
  it.
- British English spelling: behaviour, licence.
- Describe a situation concretely. Never use the phrase "locked-down environment".
- Comments explain why a decision is necessary, not what the next line does.

## Rust standards

- Edition 2024. The toolchain is pinned in `rust-toolchain.toml` (1.98.1), and the
  workspace declares `rust-version` 1.98. Don't change either as a side effect of other
  work.
- `unsafe_code` is forbidden across the workspace.
- Avoid `unwrap` and `panic` outside tests. When an invariant makes failure impossible,
  use `expect` with a message that states the invariant.
- Use `--locked` with Cargo. A dependency change is deliberate and reviewed.
- Don't sleep in tests; wait for the condition the test needs to observe.

## Verification

Run from the repository root before asking for a commit:

```console
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p durable-inquiry --locked -- knowledge check
```

CI runs the first three on every push to `main` and every pull request. The test suite
includes the durable Odori tests and takes several minutes; a quiet build is not a
stalled one.

## Data and safety

- An operator's Durable Inquiry data directory holds real investigations. Back it up, rehearse on
  a copy, and get explicit approval before anything rewrites it. Demos and screenshots
  use a throwaway `--data-dir` and an empty `--config`.
- Never put credentials, real cluster names, hosts or personal paths in fixtures,
  screenshots, docs or code.
- Ask before downloading anything (a clone, a release binary, a module), saying what,
  from where and how large. To read another repository at a revision, archive it into a
  scratch directory; never change someone's existing checkout.

## Git

- Treat uncommitted changes you didn't make as someone's work. Don't discard, overwrite
  or fold them in without approval.
- Commit and push only when the maintainer asks, and keep each commit to one coherent
  change.
- Never rewrite published history without the maintainer's approval of the exact
  operation.

## Commit attribution (required)

The maintainer is the Git author. Agents are credited with trailers at the end of the
message, after a blank line:

- `Co-Authored-By: <Agent> <email>` for every agent that authored part of the change:
  code, knowledge, documentation or tests.
- `Assisted-by: <Agent> <email>` for every agent that assisted without authoring: review,
  verification or research.

Claude's identity is `Claude <noreply@anthropic.com>`, and its trailers here name the
model, as in `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Pull requests
name the agents that took part.

## Definition of done

A change is done when its claims are verified with provenance, the knowledge and the docs
describe the current state, the fixtures and tests that cover it pass, formatting, Clippy
and the knowledge check are clean, and agent attribution is recorded.
