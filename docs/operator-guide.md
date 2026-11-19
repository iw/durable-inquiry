# Operator guide

Durable Inquiry tells you exactly what evidence to look at next, and why. Started with
your Prometheus or Grafana endpoint, it reads the metrics a request asks for
itself; you bring the rest, such as CLI and API output and configuration. It
keeps track of what the evidence shows, and says plainly what it does not know.

## Running Durable Inquiry

```bash
cargo run --release -p durable-inquiry -- serve
```

Open <http://127.0.0.1:7373/>. These options change how it runs:

| Option | Effect |
| --- | --- |
| `--addr 0.0.0.0:7373` | Listen elsewhere. There is no authentication, so only expose Durable Inquiry on a network you trust. |
| `--data-dir DIR` | Where investigations live. The default is `durable-inquiry` in your platform's local data directory; see [Configuration and data](#configuration-and-data). |
| `--config FILE` | Read preferred time zones and environment profiles from FILE. See [Configuration and data](#configuration-and-data). |
| `--in-process` | Run without the durable engine. Investigations are journalled as JSON. The explainer needs the engine, so Durable Inquiry refuses `--explainer` with this option. |
| `--snapshot-interval N` | Seconds between engine snapshots. The default is 5. |
| `--explainer claude` / `codex` | Have a model reword explanations. Off by default, and refused with `--in-process`. See below. |
| `--no-simulate` | Don't run the simulations that check whether a cause is big enough. See [Simulations](#simulations). |
| `--demo N` | Preload the reference Workflow Task investigation after N evidence steps. |
| `--demo-fixture NAME` | Also preload that fixture's investigation, with all its evidence. Repeatable. |
| `--knowledge-dir DIR` | Use knowledge packs from DIR instead of the built-in ones. |
| `--prometheus-url URL`, or `--grafana-url URL --grafana-datasource UID` | Let **Collect now** read requests' PromQL queries from there. Credentials come from files or the environment; see [collector.md](collector.md). |
| `--dashboards DIR` | Draw evidence as the panels of these Grafana dashboards draw it. See [Dashboards and runbooks](#dashboards-and-runbooks). |
| `--grafana-url URL` | Link panels into your Grafana, over the incident window. |

Stop the server with Ctrl-C. It brings every investigation's projection up to
date, drains its workers and writes a final engine snapshot. Start it again
with the same `--data-dir` and every investigation is where you left it. If the
process was killed instead, recent commands are restored from on-disk
projections, and the workspace says so.

Stop Durable Inquiry this way before you upgrade it. A new engine release may not read
the old engine's snapshot; it then rebuilds every investigation from its
projection, keeps the old snapshot as `engine.snapshot.unreadable-<UTC time>`,
and asks for model explanations again.

## Configuration and data

Durable Inquiry keeps its investigations, engine snapshots and uploaded dashboards in a
data directory, and reads an optional configuration file. Both follow your
platform's conventions:

| | Data directory | Configuration file |
| --- | --- | --- |
| Linux | `~/.local/share/durable-inquiry` (`$XDG_DATA_HOME`) | `~/.config/durable-inquiry/config.toml` (`$XDG_CONFIG_HOME`) |
| macOS | `~/Library/Application Support/durable-inquiry` | `~/Library/Application Support/durable-inquiry/config.toml` |
| Windows | `%LOCALAPPDATA%\durable-inquiry` | `%APPDATA%\durable-inquiry\config.toml` |

`--data-dir` and `--config` choose others, as does `$DURABLE_INQUIRY_CONFIG` for the
configuration.

The configuration file sets the time zones offered in the zone selector and
the environment profiles that fill in a new investigation:

```toml
# The zones offered besides UTC. Leave it out to offer every zone.
time_zones = ["Europe/London", "America/New_York"]

[[profile]]
name = "local-dev"
description = "Temporal dev server on this machine"
server_version = "1.25.1"
namespace = "orders"
task_queue = "fulfil-q"
deployment = "self-hosted"      # kubernetes, self-hosted or temporal-cloud
frontend_hosts = 1
history_hosts = 1
matching_hosts = 1
persistence = "SQLite (dev server)"
```

A profile may also set `sdk_language` (`go`, `java`, `typescript`, `python`,
`dotnet`, `ruby`, `rust` or `php`), `sdk_version` and `worker_processes`. Durable Inquiry
checks the file when it starts: an unknown zone, field or value stops it with
the problem named. The startup lines say which file was read.

## Upgrading from Kyumei

Durable Inquiry was called Kyumei (究明). The rename changed the names below and
nothing else: the same investigations, evidence and knowledge, under new names.

**Getting and building it.** The repository is
<https://github.com/iw/durable-inquiry>; GitHub redirects the old address, and
`git remote set-url origin https://github.com/iw/durable-inquiry` points an
existing clone at the new one. Every package took the new name:

| Before | Now |
| --- | --- |
| `cargo run -p kyumei -- serve` | `cargo run -p durable-inquiry -- serve` |
| `cargo build --release -p kyumei-collect` | `cargo build --release -p durable-inquiry-collect` |
| `cargo build --release --locked -p kyumei-demo` | `cargo build --release --locked -p durable-inquiry-demo` |
| `cargo run -p kyumei-extract --bin gen-dc-claims` | `cargo run -p durable-inquiry-extract --bin gen-dc-claims` |
| `cargo test -p kyumei-odori` | `cargo test -p durable-inquiry-odori` |

**Commands.**

| Before | Now |
| --- | --- |
| `kyumei` | `durable-inquiry` |
| `kyumei-collect` | `durable-inquiry-collect` |
| `kyumei-demo` | `durable-inquiry-demo` |

Their subcommands and options are the same, and so are `gen-dc-claims`,
`gen-metric-claims` and `gen-sdk-metric-claims`.

**Environment variables.** `KYUMEI_` became `DURABLE_INQUIRY_` in every
variable:

| Before | Now |
| --- | --- |
| `KYUMEI_CONFIG` | `DURABLE_INQUIRY_CONFIG` |
| `KYUMEI_COLLECT_URL`, `KYUMEI_COLLECT_GRAFANA`, `KYUMEI_COLLECT_DATASOURCE`, `KYUMEI_COLLECT_TENANT` | `DURABLE_INQUIRY_COLLECT_URL`, `DURABLE_INQUIRY_COLLECT_GRAFANA`, `DURABLE_INQUIRY_COLLECT_DATASOURCE`, `DURABLE_INQUIRY_COLLECT_TENANT` |
| `KYUMEI_COLLECT_TOKEN`, `KYUMEI_COLLECT_USER`, `KYUMEI_COLLECT_PASSWORD` | `DURABLE_INQUIRY_COLLECT_TOKEN`, `DURABLE_INQUIRY_COLLECT_USER`, `DURABLE_INQUIRY_COLLECT_PASSWORD` |
| The tests' switches, such as `KYUMEI_OPERATIONS_SOURCE` and `KYUMEI_PROMETHEUS_URL` | `DURABLE_INQUIRY_OPERATIONS_SOURCE`, `DURABLE_INQUIRY_PROMETHEUS_URL` and so on |

A variable set under its old name is not read. `durable-inquiry serve` and
`durable-inquiry-collect` say so in one line, naming the new variable and
never the value. A test whose switch is set under the old name is skipped as
if it were unset. Log targets follow the crates, so `RUST_LOG=kyumei_odori=info`
is now `RUST_LOG=durable_inquiry_odori=info`.

**Default directories.** The directory `kyumei` became `durable-inquiry`
wherever a default is used:

| | Before | Now |
| --- | --- | --- |
| Data directory | `~/.local/share/kyumei`, `~/Library/Application Support/kyumei`, `%LOCALAPPDATA%\kyumei` | `durable-inquiry` in the same place |
| Configuration file | `~/.config/kyumei/config.toml`, `~/Library/Application Support/kyumei/config.toml`, `%APPDATA%\kyumei\config.toml` | `durable-inquiry/config.toml` in the same place |
| The collector's redaction key and pseudonyms | `~/.config/kyumei-collect` | `~/.config/durable-inquiry-collect` |
| The demo's data and configuration | `.kyumei-demo`, `demo/kyumei-config.toml` | `.durable-inquiry-demo`, `demo/durable-inquiry-config.toml` |

Durable Inquiry does not read the old directories and never moves or rewrites
them. Move them yourself, before you first start it. Stop Kyumei with Ctrl-C,
so that it writes its last snapshot, then rename each directory you have:

```bash
mv ~/.local/share/kyumei ~/.local/share/durable-inquiry
mv ~/.config/kyumei ~/.config/durable-inquiry
mv ~/.config/kyumei-collect ~/.config/durable-inquiry-collect
```

On macOS the data directory holds the configuration file too, so one rename
moves both; the collector's directory is under `~/.config` there as well:

```bash
mv ~/Library/Application\ Support/kyumei ~/Library/Application\ Support/durable-inquiry
mv ~/.config/kyumei-collect ~/.config/durable-inquiry-collect
```

A directory you named yourself, with `--data-dir`, `--config` or
`--state-dir`, stays where it is: give the same option. If Durable Inquiry
starts without one and finds the old default but not the new, it says so in
one line and goes on with an empty new directory; stop it, put the old
directory in the new one's place, and start again. With the collector's state
directory left behind, `--redact` would write other pseudonyms for the same
values, and `reveal` would not find the earlier ones.

The demo's Compose project is now `durable-inquiry-demo`. Take a stack that
Kyumei's demo started down under its own name, `docker compose -p kyumei-demo
down -v`, before starting the new one.

**What keeps working as it was.**

- **Your investigations.** A data directory Kyumei wrote opens as it is: every
  investigation is listed, reads the same and continues, on the durable engine
  and with `--in-process`. Evidence keeps what it recorded when it was added:
  a block says `Collected by kyumei-collect`, Collect now's evidence names the
  collector `kyumei`, a simulation names `tempdes`, and requests and notes
  written then still say Kyumei.
- **Blocks and request codes.** The sample block format and the request code
  did not change, so the application and the collector need not be upgraded
  together. `kyumei-collect` runs a code that Durable Inquiry shows, Durable
  Inquiry reads a block `kyumei-collect` printed, and Kyumei reads one from
  `durable-inquiry-collect`. See [collector.md](collector.md#names-in-the-format).
- **Pseudonyms**, once the collector's state directory is moved: the key is
  the same, so the same value gets the same pseudonym.
- **The time zone each browser chose**, the configuration file's format, the
  address and port, the pages' addresses, and the knowledge packs.

**What else reads differently.** New evidence from Collect now names its
origin `Collector durable-inquiry`, and a new simulation `durable-envision`,
the simulator's new name ([Durable Envision](https://github.com/iw/durable-envision)).
An export is saved as `durable-inquiry-<investigation>.md`. The collector
sends the User-Agent `durable-inquiry-collect/<version>`, and
`durable-inquiry knowledge updates` sends `durable-inquiry/<version>`, which
matters only to a proxy that filters on it.

## Time zones

The selector at the top right sets the zone every page shows times in, for
this browser. It starts at UTC and offers UTC with the configured zones, or
every zone when none are configured.

- **Shown.** Times carry the zone's abbreviation, such as `14:21 BST`, or its
  offset, such as `10:21 UTC−03:00`, when the zone has no letter abbreviation.
  Lists of times, such as the trail, name the zone once in their heading.
  Hover over a time to see it in UTC.
- **Typed.** Dates and times you type into forms are read in the chosen zone,
  and each field's label names it. A time skipped by a change to summer time
  moves forward; a time that happens twice takes the first.
- **Everywhere.** Charts, request windows, notes and explanations follow the
  zone, and so does the Markdown export.
- **Not changed.** Durable Inquiry stores and reasons in UTC. Evidence as you submitted
  it, the sample block format and `durable-inquiry-collect` stay in UTC, and so does
  your own description of the symptom.

## Opening an investigation

On the landing page, describe what you are seeing in your own words, as you
would to a colleague. Include numbers if you have them ("about 30 seconds"),
a time ("since 13:35 UTC") and anything that changed.

The optional fields help Durable Inquiry ask precise questions. Leave any you don't
know empty, and Durable Inquiry will not guess them. With profiles configured, choose
an environment above the fields to fill them from its profile. Change anything
that differs: what you submit is recorded as your assertion, with the profile's
name.

- **Temporal Server version.** This chooses the knowledge pack: Durable Inquiry has one
  for 1.31, verified for 1.31.0 to 1.31.4, and one for 1.32, verified for
  1.32.0 and 1.32.1. For 1.27.0 to 1.30.8 the 1.31 pack is used: the names,
  types and defaults of settings and metrics are checked for your exact
  release, and where a default differed Durable Inquiry shows yours; behaviours were
  read for 1.31, and each claim says where it was verified. Left empty, the
  newest pack is used and the version is shown as unknown. For other versions
  every claim is marked unverified.
- **Worker SDK and version.** This decides metric names and units. Core-based
  SDKs (Python, TypeScript, .NET, Ruby, Rust) export durations in milliseconds
  by default. The Go and Java SDKs export them in seconds. Java, through
  Micrometer's Prometheus registry, adds `_total` to counters and `_seconds`
  to durations. Leave the SDK unknown, and every request shows a query for each
  SDK, labelled.
- **Namespace and Task Queue.** These are filled into every query.
- **Incident window.** In the zone chosen at the top of the page. It starts as
  the last hour, from an hour ago to now: move it to when the symptom began and
  ended, or clear it if you don't know. It decides the range of every query,
  and separates the incident from the baseline before it. Collect now needs
  both ends.
- **Topology.** The deployment kind, host counts and persistence.

Durable Inquiry records your description as evidence `E0`. Numbers in it are your
**assertions**: they count, but never as much as measured telemetry. A rule
that rests only on your assertions can support or weaken a hypothesis, but can
never rule it in or out.

You can change the context later under **Edit context**. If the change alters
the open request, the request is issued again. That happens when its window
changes, when a query changes, or when it should now be written for a
different SDK.

## The workspace

The workspace has four parts.

- **Situation.** The symptom in your words, the scope, the version and
  whether Durable Inquiry's knowledge is verified for it, and what is still unknown.
- **Current assessment.**
  - A summary.
  - *What Durable Inquiry knows*: each measured or asserted quantity, with its standing
    and the evidence it came from.
  - The hypotheses that are leading, open and ruled out.
  - The diagnosis, once there is one.
  - The remaining uncertainty.
  - The diagrams.
- **Next evidence.** The primary action.
- **Investigation trail.** Everything that happened, in order.

### Hypothesis states

Durable Inquiry never shows a confidence percentage. A hypothesis is in one of these
states:

| State | Meaning |
| --- | --- |
| candidate | Possible; nothing has moved it yet |
| supported | Some evidence points towards it |
| weakened | Some evidence points away from it |
| eliminated | Evidence rules it out |
| established | Evidence shows it holds |

*Contested* means evidence points both ways. Every state links to the findings
behind it. Every finding links to the facts it used, whether each fact was
observed or asserted, the evidence and observation it came from, and the
knowledge claims it relied on. Each claim records which Temporal versions it
was verified against.

A diagnosis is shown as **provisional** while a more specific explanation is
still possible. It becomes final only when nothing more specific could hold.

## Answering a request

*Next evidence* asks for one thing at a time. It tells you:

- what to collect;
- why it matters;
- the exact query or command, rendered for your SDK, namespace and Task Queue;
- the time window and grouping;
- which hypotheses the answer will tell apart.

Where Durable Inquiry lacks a value, the query shows a placeholder such as
`<task_queue>`. Replace it yourself, or add the value under **Edit context**:
Durable Inquiry then asks again with it filled in.

Commands run as shown in bash or zsh. A namespace or Task Queue name with a
space, a quote or another character the shell reads specially is quoted, and
inside grpcurl's JSON it is also escaped as a JSON string.

There are four ways to answer.

- **Paste.** Paste raw output: a Prometheus API JSON response
  (`/api/v1/query_range`), exposition text, a Grafana CSV export,
  `timestamp value` lines, `temporal` CLI or grpcurl JSON output,
  dynamic-config YAML or logs. Also paste the query or command you ran. Durable Inquiry
  uses it to understand the output, for example the labels you filtered on.
  *Describe it* lets you state the form, the unit or the capture time when
  they cannot be detected.
- **Upload.** Upload the same kinds of file, or a screenshot. Durable Inquiry keeps a
  screenshot but cannot read numbers from it.
- **Enter values.** Type a value for a signal, such as a minimum of 1,071 free
  slots over a window. Typed values are recorded as your assertion.
- **Observation.** A written note, such as "Workers restarted at 13:40".

For PromQL queries there are two more ways, described in
[collector.md](collector.md):

- **Collect now.** If Durable Inquiry was started with a Prometheus or Grafana
  endpoint, it runs the queries over the request's window and adds the
  samples as evidence. The notice that follows says what was read (series,
  samples and the time they cover), the values, what changed in the
  assessment and what comes next, and the next-evidence box keeps a line
  saying which request was answered and by what. Without an incident window
  with both ends, the box says so instead of offering the button. A request's
  queries can read only part of what it asks for: the backlog request's
  fallback reads only the backlog's age. The box then says what they give,
  and once Durable Inquiry has it, it stops offering them and points to the other
  ways.
- **durable-inquiry-collect.** Copy the command shown under the queries and run it
  where your metrics can be read. Paste the block of text it prints under
  **Paste**. The block holds the samples exactly, and says which request it
  answers.

If you cannot get the evidence, use **Can't get this** and say why. Durable Inquiry
keeps the affected hypotheses open, since missing evidence rules nothing out.
It records the gap as uncertainty and asks for the next most useful evidence,
which is often the same question asked another way.

Durable Inquiry treats an empty query result differently from a zero. If a query
returns no series, say so in an observation; do not type 0. A sample block
records an empty result as an observation with no value, so Durable Inquiry doesn't ask
for it again. When the Workers' metrics for the Task Queue all come back
empty, no Worker may be polling it: the DescribeTaskQueue capture settles that,
since Matching lists every poller of the last five minutes.

The exception is a failure counter shown with a second query, its witness. The
SDKs and the server create such a counter at its first increment. So while the
witness (every poll, or every request, from the same Workers or namespace) has
samples, no series means nothing failed, and Durable Inquiry reads it as zero. Collect
now and `durable-inquiry-collect` run both queries. When you collect by hand, run both
and paste the results together.

Durable Inquiry asks for DescribeTaskQueue through grpcurl first. The Frontend
serves gRPC reflection, so grpcurl needs only its address, and the response
is the API's own JSON: Matching's backlog and rates, the effective rate
limit and where it comes from, and the pollers of the last five minutes. The
`temporal task-queue describe` command is the second way. The CLI prints the
backlog, the rates and the pollers, each poller with the Task Queue rate its
Worker sends, but neither the effective rate limit nor its source, and only
for the unversioned queue. When every poller sends the same rate and dispatch
runs at it, with a backlog waiting or tasks arriving faster, Durable Inquiry takes the
Workers' rate as the limit in force. A limit set through the API, or the
system default, would have to equal that rate, so Durable Inquiry leaves both weakened
rather than ruled out; DescribeTaskQueue's source for the limit settles them.
Keep the CLI's `--task-queue-type`: without it the CLI lists the Workflow and
the Activity Task Queue together, and Durable Inquiry leaves the rows unread.

Each piece of evidence under **Evidence** shows the query or command you gave
with it, and its own page shows what Durable Inquiry read from it.

On Temporal 1.32, DescribeTaskQueue's statistics also say whether the rate
limiter held dispatch back in the 30 seconds before the capture
(`rateLimitingActive`). The CLI does not print it. Protobuf JSON leaves the
field out when it is false, so the grpcurl call runs with `-emit-defaults`,
which prints it, and a capture without it is not read as false.

## Simulations

For some causes Durable Inquiry checks whether the cause, at the size your evidence
shows, is big enough to explain the symptom. Once the evidence it needs is in,
it simulates the part of the cluster the cause is in with
[Durable Envision](https://github.com/iw/durable-envision), calibrated on that evidence, and
compares the latency the run gives with the one observed. For Workflow Task
schedule-to-start, it runs the Task Queue with its observed add rate against
its dispatch limit and partitions, with nothing else in the way.

The run is evidence of its own, marked as a simulation. Its page shows the
hypothesis it tests, the Durable Envision commit and seed, the evidence it was
calibrated on, its inputs and results, and the scenario exactly as run. Its
notes give the fit: the simulated latency against the one observed.

What it found is a **simulated fact**. Like your typed values, it can support
or weaken a hypothesis but never establish or rule one out: a run that
reproduces the latency shows the cause is consistent with it, not that it
caused it. Within half to twice the observed latency supports the cause, below
a fifth weakens it, and anything else counts for nothing. Findings that rest
on one say so.

Simulations run only for self-hosted Temporal 1.31, which Durable Envision models, and
need an incident window with both ends. Each takes a few seconds and runs once
for the evidence it was calibrated on. Start Durable Inquiry with `--no-simulate` to
turn them off.

## Evidence and corrections

Every piece of evidence has its own page, reached from the trail or from any
finding. It has two parts.

- **Raw.** Exactly what you submitted, byte for byte, downloadable.
- **Interpretation.** What Durable Inquiry read from it:
  - each observation, with its subject (series and labels, API field or
    configuration key), its value, its window, and the signal it informs;
  - the basis for that binding;
  - any caution, such as an assumed unit or an assumed capture time.

If Durable Inquiry misread something, correct it. You can say an observation:

- is about a different signal, or about none;
- has a different unit;
- covers a different window;
- should be disregarded, or used again.

A correction never changes the raw evidence. It is recorded as its own event,
and the investigation is reassessed at once. If a correction takes away the
evidence a request was answered with, Durable Inquiry asks for it again.

## Dashboards and runbooks

Durable Inquiry links each diagnosis, hypothesis and request to the operations material
that bears on it, and draws evidence the way your dashboards do.

**Runbooks, playbooks and panels.** The alert runbooks, playbook sections and
dashboard panels of
[tsurdilo/temporal-server-operations](https://github.com/tsurdilo/temporal-server-operations)
appear where they apply:

- under the symptom, the runbooks for the alerts it would fire;
- beside a hypothesis, the runbook or playbook section that covers it;
- under **Where operators look** in the next evidence, the panels that show
  what the request asks for, on the dashboard for your SDK, and the runbooks
  that check it;
- under the diagnosis, **Watch it recover** lists the panels that draw the
  symptom, on the dashboard for your SDK, to watch it recover once you make
  the change, and **Runbooks and playbooks** lists the runbooks and playbook
  sections for what was found.

Each one says why it applies, and links to the repository at the commit Durable Inquiry
checked it at. Durable Inquiry does not copy the repository's text: follow the link.

**Charts.** Start Durable Inquiry with `--dashboards DIR`, a directory of Grafana
dashboard JSON (such as a clone of the repository's `observability/dashboards`,
or dashboards exported from your own Grafana), or upload dashboards on the
**Dashboards** page. Uploads are kept in the data directory and load again at
startup.

An evidence page then draws its series as the panel that reads the same
quantity draws them: the panel's title, its unit and decimals, its thresholds,
its legend names and statistics (min, max, mean, last), in a chart with a text
version beside it. The panel is chosen in this order:

1. one the request links to;
2. one on a dashboard for your SDK;
3. one whose query groups by the labels the series carry.

A panel qualifies only if its query reads that metric alone (not a ratio of
two), as a quantile when the evidence is one, and keeps at least one of the
series through its filters. Under each chart Durable Inquiry says which panel, which
dashboard and version, and where the dashboard came from. With `--grafana-url`,
a link opens the panel in your Grafana over the evidence's window. The
investigation's **Evidence** panel names the panel beside each evidence Durable Inquiry
draws this way.

Values are written in the panel's unit: a lag Durable Inquiry holds in seconds is drawn
in milliseconds on a milliseconds panel. A legend that names series by a label
the evidence does not carry is filled from a label that carries the same value,
and the chart says so. Thresholds outside the evidence's range are listed under
the chart rather than stretching it.

**Checks.** The **Dashboards** page lists what is loaded and the panels Durable Inquiry
cannot read, and checks every chart panel's queries against what the newest
pack's knowledge says about the metrics they read (choose another pack above
the findings):

- metric names no claim covers;
- labels a metric does not have;
- labels that carry the same value as the one Durable Inquiry reads;
- units that differ from the metric's;
- thresholds a hundredfold or more from other dashboards' for the same metric.

`durable-inquiry dashboards check DIR` prints the same report for every pack.

**Linked panels, live.** Every panel Durable Inquiry links to opens in Durable Inquiry, with live
data: follow its title, or **open with live data**, wherever a panel is
listed, the **Evidence** panel included, and **Open the panel with live data**
under an evidence chart. The
panel page runs the panel's queries through the endpoint Collect now uses and
draws them as the panel does, with its legend, unit, thresholds and legend
statistics.

- Inside an investigation, `$namespace` and the Task Queue come from its
  context, and the range is its incident window with the margins its requests
  use (10 minutes before, 5 after). On its own, a panel reads the last hour.
- Choose the range (incident window, last hour, 6 hours or 24 hours) and the
  panel's variables, such as the quantile `$p`, above the chart.
  `$__rate_interval` is worked out as Grafana does, assuming a 15-second
  scrape.
- **Add as evidence** records what the page read as a sample block citing the
  panel. When the panel is linked to the open request, it answers it. Just
  looking at a panel never changes the investigation.
- Without a metrics endpoint, the page shows the filled-in queries and the
  `durable-inquiry-collect` command to read them elsewhere. When the panel's dashboard
  isn't loaded, it says which file of the repository to load.

**Versions.** The operations repository has no tags or releases, so citations
carry the versions it declares itself, with the commit and its date: a panel's
dashboard by its title's version (`Temporal Server Dashboard v2.25.0`), and a
playbook by the dashboard version it needs (`needs the Temporal Server
dashboard v2.15.0+`). Runbooks declare none.

## Explanations from a model

The explainer is off by default, so every explanation is written by the
deterministic engine. With `--explainer claude`, Claude Code rewrites the prose
after each assessment. It runs through your local `claude` CLI, which must be
logged in. `--explainer codex` uses the Codex CLI instead. The explainer runs
on Odori, so Durable Inquiry refuses `--explainer` with `--in-process` rather than start
without it.

The model sees the structured assessment and may only reword it. A draft is
rejected if it does any of the following:

- introduces a number that is not in the evidence;
- contradicts a hypothesis state;
- refers to a finding that does not exist.

When a draft is rejected, the deterministic text stays and the workspace shows
why. The model never decides anything: states, findings, requests and the
diagnosis all come from the engine.

## Exporting

**Export** produces a Markdown report: the situation, the diagnosis and its
recommendations, what Durable Inquiry knows, the hypotheses, every finding with its
facts and claims, the diagrams as text, the evidence index and the full trail.
It is meant to be pasted into an incident document.

## Knowledge

**Knowledge** holds every claim Durable Inquiry can rely on, in a pack for each minor
release of Temporal Server. That includes every dynamic configuration setting
and server metric of Temporal 1.27.0 to 1.32.1, so the page opens with counts
by kind rather than a list. It shows the newest pack; choose another above the
search. Search by key, metric name or words, and narrow the results by kind,
by verification status, or to claims a diagnostic uses. An investigation's
links open the claims in the pack it uses.

Each claim page shows:

- the claim itself, its category, and the other packs that have it;
- for a server metric, its definition in the Temporal source, the packages
  that record it, and every name and unit it can be exported under, with the
  metrics configuration that produces each;
- the versions it was verified against;
- what uses it: each diagnostic, with the signals, requests, rules and
  conclusions that read it, and any heuristic that relies on it;
- notes, including those kept in annotations beside generated claims;
- each source, as repository, revision, path and line, with the verbatim
  excerpt and a link to it.

Durable Inquiry's own thresholds are listed separately as heuristics. They are
judgements, not facts about Temporal.

`durable-inquiry knowledge updates` lists what is newer upstream than the sources the
knowledge pins: Temporal Server and SDK releases after the pinned ones, and
commits after pinned commits, with the cited files and dashboard versions that
changed in the operations repository. It needs Git and network access to the
repositories, and changes nothing.

## What Durable Inquiry can diagnose

Durable Inquiry recognises twelve symptoms, on Temporal Server 1.31 and 1.32. Describe the one
you see; if it mentions more than one, the more specific symptom wins.

| Symptom | Say something like | What it looks at |
| --- | --- | --- |
| Java Workflow thread exhaustion | "our Java Workers log WorkflowRejectedExecutionError", or "the Workers keep evicting cached Workflows to free threads" | The Java Workers' Workflow threads in use, evictions to free a thread, Workflow Task failures by reason, and the cached Workflows |
| Activity schedule-to-start | "activities are taking 2 minutes to start" | Activity slots, pollers, poll failures, DescribeTaskQueue for the Activity Task Queue, which limit caps dispatch and, on 1.32, whether the rate limiter held it back, and how many Activities Workflows schedule |
| Workflow Task schedule-to-start | "workflow tasks wait 30 seconds before a worker picks them up" | Workflow Task slots, pollers, poll failures, DescribeTaskQueue (on 1.32, with whether the rate limiter held dispatch back), partitions and the Signal rate |
| History task processing lag | "timers have been firing 30 to 90 seconds late" | History's task load latency, scheduler wait and attempt latency, and failed attempts by cause |
| History Workflow cache pressure | "History's GetWorkflowExecution calls tripled and its latency is up" | History's reads of Workflow state, its Workflow cache's lookups and misses, and how full the cache is |
| Shard ownership churn | "during every History deploy clients get 'shard unavailable, please backoff and retry'" | ShardOwnershipLost errors at the Frontend, History's membership changes, and how long taking moved shards takes |
| Visibility lag and errors | "completed workflows still show as Running in the Web UI", or "temporal workflow list fails with namespace rate limit exceeded" | How late History writes visibility records, the Frontend's rejections of listing calls, the visibility persistence limits, and the visibility store's errors, DLQ writes and latency by call |
| Persistence latency | "History's persistence latency p99 is around 900ms" | History's SQL connection pools, persistence errors, persistence calls and latency against their baseline |
| ResourceExhausted rejections | "clients get ResourceExhausted, namespace rate limit exceeded" | The Frontend's rejections by cause and scope, and the namespace's requests against their baseline |
| History limits | "workflows are terminated by history-service: history size exceeds limit" | One affected Workflow's status, event count and history size, and the terminated Workflow's last event |
| Workflow cache misses and replays | "since the campaign our workers spend seconds replaying workflows" | Replay latency, sticky cache hits and misses, the Workers' cache size, sticky timeouts and completions without sticky execution |
| Workflow Task failures | "since the deploy, running workflows stopped progressing; queries fail with Workflow Task in failed state" | Failed Workflow Task attempts by reason, History's Workflow Task timeouts, and the attempt a stuck Workflow's task has reached |

Some requests need the server's own metrics, not only the Workers'. Each
request says so. The visibility store's metrics and History's DLQ count have
no namespace label, so those requests cover every namespace. If you cannot collect one, decline it and Durable Inquiry asks for the
next best thing.

A namespace or Task Queue whose name has characters other than letters,
digits and `_` is matched in two spellings, for example
`namespace=~"orders-prod|orders_prod"`. Tally, the server's default metrics
framework, writes those characters as `_`, and so do Go Workers that use the
tally handler. Queries for Core-based and Java Workers use the name as it is.

When an answer breaks a metric down by a label, such as ResourceExhausted
errors by cause, Durable Inquiry reads a value missing from it as zero: Prometheus
returns no series for something that never happened. The evidence shows a
note saying so, which you can correct. An empty answer is still not zero:
Durable Inquiry records that it found no series, and reads the signal as unknown.

## Limits

If your description does not match a symptom Durable Inquiry knows, the investigation
still opens. It says that no diagnostic matched, and asks for nothing.
**Edit context** opens so you can choose the closest symptom, or leave it.
Durable Inquiry never invents a diagnostic.

See [decisions.md](decisions.md) for the known limitations.
