# Collecting metrics

Most of Durable Inquiry's evidence requests are PromQL range queries. There are three
ways to answer one:

- **Paste the output.** Run the query yourself and paste what Prometheus or
  Grafana returns. See the [operator guide](operator-guide.md).
- **Collect now.** If Durable Inquiry was started with a Prometheus or Grafana
  endpoint, it runs the request's queries itself and adds the result as
  evidence.
- **`durable-inquiry-collect`.** Where Durable Inquiry cannot reach your metrics, run the
  companion where they can be read. It prints a short block of text, a
  *sample block*, to copy from the terminal and paste into the request.

Collect now and `durable-inquiry-collect` produce the same sample block from the same
client code, and Durable Inquiry reads it the same way. In both cases the samples are
observed telemetry, kept exactly as Prometheus returned them.

## Collect now

Start Durable Inquiry with an endpoint:

```bash
durable-inquiry serve --prometheus-url http://prometheus:9090
```

```bash
durable-inquiry serve --grafana-url https://grafana.example.com --grafana-datasource P1809F7CD0C75ACF3
```

| Option | Effect |
| --- | --- |
| `--prometheus-url URL` | The Prometheus HTTP API, or Thanos, Mimir or VictoriaMetrics serving it. A path prefix such as `/prometheus` is kept. |
| `--grafana-url URL` with `--grafana-datasource UID` | A Prometheus data source, read through Grafana's data source proxy (`/api/datasources/proxy/uid/UID`). |
| `--metrics-tenant TENANT` | Sent as `X-Scope-OrgID`, for Mimir, Cortex and Thanos tenants. |
| `--metrics-token-file FILE` | A bearer token, such as a Grafana service account token. Without it, `$DURABLE_INQUIRY_COLLECT_TOKEN` is used. |
| `--metrics-user USER` | Basic authentication. The password comes from `--metrics-password-file` or `$DURABLE_INQUIRY_COLLECT_PASSWORD`. |
| `--metrics-ca-file FILE` | Extra CA certificates to trust, as PEM. |

A request with PromQL queries then shows **Collect now** under its queries.
Durable Inquiry reads each query over the request's window, at a 15-second step for
windows up to an hour, a minute up to six hours, and five minutes beyond.
It adds the samples as one piece of evidence answering the request.

If a query needs a value Durable Inquiry does not know, such as the namespace,
Collect now says so instead. Add the value under **Edit context**: Durable Inquiry
issues the request again with the value filled in. If the endpoint cannot be
reached, refuses the credentials or rejects a query, the request says why,
and nothing is added.

Evidence from Collect now names its origin as `Collector durable-inquiry 0.1.0`. The
raw evidence is the sample block, so it can be decoded, exported and replayed
like any other.

## durable-inquiry-collect

`durable-inquiry-collect` is a single binary. It has no model, agents or knowledge
pack, and it only reads: it sends `GET /api/v1/query_range` and nothing else.
Build it and copy it to a machine that can reach your metrics:

```bash
cargo build --release -p durable-inquiry-collect
```

The binary is `target/release/durable-inquiry-collect`.

### Answering a request

Each request with PromQL queries shows the command to run, with a request
code:

```bash
durable-inquiry-collect --code kyq1.7ZBBa4Mw… --set task_queue=order-fulfillment --url http://prometheus:9090
```

The code carries the request's queries and window. A query that needs a value
Durable Inquiry does not know has a placeholder, such as `<task_queue>`, and the
command ends with a `--set` for each. Put the value after the `=`. A value
with characters other than letters, digits and `_` is matched both as written
and as tally writes it (`order-fulfillment|order_fulfillment`), as Durable Inquiry's
own queries are.

The block goes to standard output; a summary goes to standard error. This
block answers the third request of the `visibility-collected-blocks` fixture,
encoded from saved query output (so its source is `file`):

```text
-----BEGIN KYUMEI SAMPLES v1-----
Collected by durable-inquiry-collect 0.1.0 from file at 2026-10-16 10:45 UTC
Request R3 of inv-golden-visibility-blocks, 2026-10-16 09:50 UTC to 2026-10-16 10:46 UTC, step 1m
Query 1: sum by (operation) (rate(visibility_persistence_errors[1m]))
  operation=RecordWorkflowExecutionStarted  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  min 0  max 0  mean 0  (57 samples)
  operation=UpsertWorkflowExecution         ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  min 0  max 0  mean 0  (57 samples)
  operation=RecordWorkflowExecutionClosed   ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  min 0  max 0  mean 0  (57 samples)
Query 2: sum(rate(dlq_writes{task_category="visibility"}[1m]))
  (no labels)  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  min 0  max 0  mean 0  (57 samples)
:zZIxSgNREIZ3E0E8gY2RJdVusSFBLAStxMouQURElt23k2TYlzfJvLeJiwhp7DxIrLxCLHIDGzvP
:4Al8smAESZtkqmFmfubn57u86bhOLc05TiSEqEY5chEKkhKE8ZqNVqO500UJ89enxax_gGoc9kim
:oMIxakxQoinCRJLIdKV9NF18vc36049ne_v5XnUqpzofeEnh-TQEjg2SCjzfNuAv5ZFdadQGlIAI
:mIn1bWtwFwRVd-9XdtgGQZxeE2ddSZOLexD5z7xjYjaQnpSf5y-7zrrrj8n9q6EGNv9Mboe72ooI
:zyXpjSZ4bCEpoUjlKJowGtAPJtZZJOywR1yc1Ze41B9LOlxng5a_AQ
-----END KYUMEI SAMPLES (358 chars, crc32 691411b8)-----
```

Copy everything from the `-----BEGIN` line to the `-----END` line and paste it
into the request's **Paste** form. The lines before the `:` lines are for you
to read: each series with a sparkline, its minimum, maximum and mean. Durable Inquiry
reads only the `:` lines.

`--dry-run` prints the filled-in queries and the window, and reads nothing.

### Without a request code

Give the queries and a window yourself:

```bash
durable-inquiry-collect --url http://prometheus:9090 \
  --query 'sum by (operation) (rate(persistence_requests{service_name="history", operation=~"CreateWorkflowExecution|UpdateWorkflowExecution|AddHistoryTasks"}[1m]))' \
  --last 10m --step 15s
```

Read from a Prometheus 3.15 scraping a Temporal dev server under a small
load, that printed:

```text
-----BEGIN KYUMEI SAMPLES v1-----
Collected by durable-inquiry-collect 0.1.0 from prometheus at 2026-10-14 11:13 UTC
2026-10-14 11:03 UTC to 2026-10-14 11:13 UTC, step 15s
Query 1: sum by (operation) (rate(persistence_requests{service_name="history", operation=~"CreateW…
  operation=CreateWorkflowExecution  ▁▁▃▅▇█▆▄▄▄▆████████████▇  min 0  max 0.491  mean 0.37  (24 samples)
  operation=UpdateWorkflowExecution  ▁▃▆▅▃▁▁▃▆██████████▇  min 0.118  max 0.491  mean 0.371  (20 samples)
:dVBtSFNRGD53gqf6kTDCjSK4jaDdlTqDoD8jLISoP0ZlloTN7cgu3u3Oc-61RiWWgTIqicbK2qhd
:sfWlRmT0sWY0SoocQQSJH8hQ_ID9UIeKCnpENxX1_fM87_PyPi_Pe_L8aQbstsrYXCqgLN5RIfPY
:lWURBQFZJNaYnZtt3ObEoh1JNiSTucGxQNAGwHj7c4rjfYvdQxkwDQyR7Wypi9WLToTNEi86OFZP
:CdLTnvBEQg4LKsGoQkZEIlcJwpU8FRxmOzLpbHQuYpfuAJvaNlXpjmFE98-JuLxMEC_nX0EWeXFy
:7azTuqGeZ7UeXzI6YyblRHe9ONd-keNUzNaUq2YTT63yp4MmeTPKgA2L2QPAt8Ya30R16HDsnnsm
:Eb5p_O19Fx_46-d62md_NM77Mv3hWv-TQHR_c-d8c_eHF9yEEvcPB9vUidgX93STR-2J-IKvYy_1
:4aHerpZX1dzM01BvfTSeNq10eO8Of0oHU0kC1kkBZbLF7WWUJRhbVlcH2-QpO97Wd64Eo9C6NqLq
:wX1Tjqrg6xE4UvJTM3siA9YZwjm3d2phbcOpoluIgYnWC_9v3NkF8zUH4aNnBPZ3t-275K2Cv5zb
:6z4WZMBI5Xv58WctPPo9Gmoq2gKgIU9R7zXQKymWJOk1_zKLPUmAhZGyQ0Jh2gI
-----END KYUMEI SAMPLES (671 chars, crc32 9fe692da)-----
```

A block without a request code says no request; Durable Inquiry reads it the same way.

`--query` is repeatable; `--metric NAME` reads a metric as it is. The window
is `--from` and `--to` (RFC 3339), or `--last` ending now; without either it
is the last 30 minutes. The step defaults to the code's, else 15 seconds.
Prometheus returns at most 11,000 samples a series, so a window that would
need more asks for a longer step.

### Endpoints and credentials

| Option | Environment | Effect |
| --- | --- | --- |
| `--url URL` | `DURABLE_INQUIRY_COLLECT_URL` | The Prometheus HTTP API, or Thanos, Mimir or VictoriaMetrics. |
| `--grafana URL --datasource UID` | `DURABLE_INQUIRY_COLLECT_GRAFANA`, `DURABLE_INQUIRY_COLLECT_DATASOURCE` | A Prometheus data source, through Grafana's data source proxy. |
| `--tenant TENANT` | `DURABLE_INQUIRY_COLLECT_TENANT` | Sent as `X-Scope-OrgID`. |
| `--token-file FILE` | `DURABLE_INQUIRY_COLLECT_TOKEN` | A bearer token. |
| `--user USER`, `--password-file FILE` | `DURABLE_INQUIRY_COLLECT_USER`, `DURABLE_INQUIRY_COLLECT_PASSWORD` | Basic authentication. |
| `--ca-file FILE` | | Extra CA certificates to trust, as PEM. |

Secrets are read from files or the environment, never from the command line,
where other users of the machine can see them. A URL with credentials in it
is refused. Errors never repeat a token or a password, nor the request URL,
which carries the query.

### Smaller blocks: rounding

Values are exact by default. `--round N` rounds them to N significant digits
(1 to 17), which shortens blocks of noisy measurements. The block records
that it was rounded, and Durable Inquiry says so on the evidence.

The record lag of the visibility reference fixture (three series, 843 samples, a
41,676-byte JSON response):

| Rounding | Characters | Lines |
| --- | --- | --- |
| Exact | 3,296 | 44 |
| 4 digits | 2,647 | 35 |
| 3 digits | 2,156 | 29 |
| 2 digits | 1,432 | 19 |

Counts, gauges that change little, and flat series compress much further: the
same fixture's refusals (519 samples, 22,710 bytes of JSON) take 5 lines.

### Labels you would rather not share: redaction

`--redact namespace,task_queue` replaces those labels' values with stable
pseudonyms such as `r_3f9c07a1d2`, in the samples and in the queries. A
pseudonym is an HMAC-SHA256 of the label and its value under a random key
kept on the collecting machine: the same value of the same label always gets
the same pseudonym there, and only that machine's table maps it back. The key
and the table are in `~/.config/durable-inquiry-collect` (or `--state-dir`), readable
only by you.

```bash
durable-inquiry-collect reveal r_3f9c07a1d2
```

Durable Inquiry notes on the evidence which labels are pseudonyms. A finding that
quotes a redacted label quotes the pseudonym.

### Other subcommands

- `durable-inquiry-collect encode FILE… --code CODE` writes saved `query_range` output,
  one JSON file per query in the code's order, as a block. Use it when you
  can run queries in Grafana's query inspector or with `curl` but cannot run
  the collector against the endpoint. The block's window is the range the
  files cover, unless `--from` or `--last` gives one.
- `durable-inquiry-collect decode [FILE]` prints every sample of a block (or of
  standard input), to check what it holds.
- `--qr` also prints the block as a QR code, without its readable lines, when
  it fits in one (at most 2,953 bytes).

## Request codes

A request code is `kyq1.` followed by base64url, without padding, of a raw
deflate stream of this JSON object:

| Key | Value |
| --- | --- |
| `i` | The investigation's id. |
| `r` | The request's id, such as `R3`. |
| `q` | The PromQL queries, in order. `<name>` is a placeholder for a value Durable Inquiry does not know: letters, digits and `_` between angle brackets. |
| `s`, `e` | The window's start and end, in Unix milliseconds. |
| `p` | The step, in milliseconds. |

A code with no queries, a window that ends before it starts, or a zero step is
refused, as is one that inflates to more than 1 MiB.

## The sample block format, version 1

A block is text:

```text
-----BEGIN KYUMEI SAMPLES v1-----
(readable lines)
:(payload line)
:(payload line)
-----END KYUMEI SAMPLES (N chars, crc32 XXXXXXXX)-----
```

- **Armour.** The first line names the format version. The last line states
  the payload's length in characters and the CRC-32 (IEEE) of the compressed
  payload, as eight hexadecimal digits.
- **Readable lines** are everything between the first line and the first
  line that starts with `:`. Readers ignore them.
- **Payload lines** start with `:` and hold up to 76 characters of base64url,
  without padding, of a raw deflate stream. A line of base64url characters
  without the `:`, after the payload has begun, continues it: terminals that
  wrap long lines when copying are tolerated. Spaces around lines and CRLF
  line endings are ignored.
- **Text around a block** is ignored, and a paste may hold several blocks.

A reader rejects a block whose last line is missing, whose payload is shorter
or longer than the last line states, whose checksum does not match, or whose
version is newer than it reads, and says which. Nothing is read from a block
that is rejected. A payload may not inflate to more than 64 MiB.

### The payload

Integers are LEB128 varints: `uvarint` unsigned, `ivarint` zigzag-encoded
signed. A string is a `uvarint` length and that many bytes of UTF-8. Signed
arithmetic on deltas wraps.

| Field | Encoding |
| --- | --- |
| Magic | the bytes `KYS` |
| Version | `u8`, 1 |
| Rounded to | `u8`: significant digits, or 0 for exact values |
| Collector | string, such as `durable-inquiry-collect 0.1.0` |
| Source | string: `prometheus`, `grafana` or `file` |
| Collected at | `ivarint`, Unix milliseconds |
| Investigation | string, empty for none |
| Request | string, empty for none |
| Window | start and end as `ivarint` Unix milliseconds, then the step as `uvarint` milliseconds |
| Redacted labels | `uvarint` count, then each label name as a string |
| Queries | `uvarint` count, then each query |

A query is its PromQL as a string, then a `uvarint` count of series. A series
is a `uvarint` count of labels, each a name and a value string in name order
(`__name__` included when Prometheus returns it), then a `uvarint` count of
points. A series with points then has its timestamps, then its values.

**Timestamps** are Unix milliseconds: the first as an `ivarint`, the second
as its gap from the first, then each gap's change from the previous gap. An
even step costs one byte a sample; gaps and irregular steps cost more.

**Values** start with a `u8` saying how they are written. The writer tries
every encoding that represents the series exactly and keeps the smallest;
each is checked by decoding before it is chosen.

| Code | Encoding |
| --- | --- |
| 0, integers | Every value is a whole number up to 2^53 (and not -0): each value's change from the previous, as `ivarint`s, starting from 0. |
| 1, decimals | Every value is finite and not -0: one `ivarint` power of ten common to all, then each value times ten to minus that power, as changes from the previous `ivarint`. |
| 2, float bits | The first value's IEEE 754 bits, 8 bytes little-endian; then for each value, its bits XOR the previous value's. A zero XOR is one byte 0; any other is a byte `(t << 4) \| m`, where `t` is the number of trailing zero bytes and `m` the number of bytes left, then those `m` bytes little-endian. |
| 3, significant | Every value is finite and not -0. Each is its shortest decimal mantissa and a power of ten, which stays the same from one value to the next while the value can be written exactly with it and the mantissa grows at most a hundredfold. First the powers: a `uvarint` count of runs, then each run's length (`uvarint`) and its power's change from the previous run's (`ivarint`). Then each mantissa: as its change from the previous mantissa while the power is the same, else whole, as `ivarint`s. |

NaN, the infinities, -0 and NaN payloads are written as float bits. A decoded
block holds exactly the values and timestamps Prometheus returned: values bit
for bit, timestamps to the millisecond.

The payload must end after the last query; trailing bytes are rejected.

### Names in the format

The format was defined when Durable Inquiry was called Kyumei, and version 1
keeps the names it was given then: `KYUMEI SAMPLES` on a block's first and
last lines, the payload's magic `KYS`, and the request code's prefix `kyq1.`.
They are the format's fixed markers, not labels. A collector often runs on
another machine, where its output can only be copied as text, and is seldom
upgraded with the application. Changing a marker would make each refuse what
the other writes, and blocks already collected would stop reading.

So `kyumei-collect` and `durable-inquiry-collect` read the same codes and
write the same blocks, and Kyumei and Durable Inquiry read blocks from either.
The collector's own name, in the *Collector* field and on the `Collected by`
line, is the only difference, and no reader checks it.

## What Durable Inquiry does with a block

Durable Inquiry recognises a block in a paste or an upload by its first line, and reads
each query of each block as if its output had been pasted with that query:
the query tells it which metric a series is, and the window is the block's.

On the evidence it notes:

- who collected the block, from what, and when;
- if the block was collected for another request than the one it is pasted
  into, to check;
- if the values were rounded, or labels redacted;
- each query that returned no series. An empty result is not evidence that
  the value was zero, so nothing is inferred from it.

The fixture `visibility-collected-blocks` replays the visibility reference
investigation with every answer pasted as a block. A test checks that its
blocks hold that fixture's query output exactly, under each request's code.

## Checking against a real Prometheus

`crates/durable-inquiry-collect/tests/real_prometheus.rs` reads a set of queries from
a Prometheus you name, compares every sample with Prometheus's own JSON, and
checks that a block returns them bit for bit:

```bash
DURABLE_INQUIRY_PROMETHEUS_URL=http://127.0.0.1:9090 \
  cargo test -p durable-inquiry-collect --test real_prometheus -- --ignored --nocapture
```

Its queries read Prometheus's own metrics, chosen to return NaN, the
infinities, -0 and gaps, and a Temporal Server's counters when Prometheus
scrapes one. Run against Prometheus 3.15.0 scraping a Temporal dev server
that was restarted during the window, it read 2,168 samples, among them 49
NaN, 40 infinities, 22 negative zeros, 68 gaps and 67 counter resets, and
every one came back from the block exactly.
