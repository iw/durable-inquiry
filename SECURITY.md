# Security policy

## Reporting a vulnerability

Please report a vulnerability privately, through this repository's
[private vulnerability reporting](https://github.com/iw/durable-inquiry/security/advisories/new),
not in a public issue. Say what you found, how to reproduce it, and the commit you
tested. The report stays private while a fix is prepared.

## Supported versions

Durable Inquiry has no releases yet. Fixes land on `main`.

## Deployment model

- **No authentication.** Anyone who can reach Durable Inquiry can read and change its
  investigations. When it is started with a Prometheus or Grafana endpoint, they can also
  have it run its requests' queries with that endpoint's credentials. `durable-inquiry serve`
  listens on 127.0.0.1 unless `--addr` says otherwise. Run it where only people you trust
  can reach it, or behind a proxy that authenticates them.
- **Read-only collection.** Collect now, linked panels and `durable-inquiry-collect` only read:
  they run PromQL queries against the endpoint you give them.
- **The explainer.** With `--explainer claude` or `--explainer codex`, Durable Inquiry runs that
  CLI, logged in as the user running Durable Inquiry, in an empty `agent-sandbox` directory inside
  the data directory. It sends the model the investigation's current assessment, with
  the values drawn from your evidence, so the model's provider sees them.
- **What leaves the machine.** Without the explainer, Durable Inquiry sends nothing beyond the
  queries it runs against your Prometheus or Grafana, and the release checks that
  `durable-inquiry knowledge updates` makes on GitHub when you run it.
