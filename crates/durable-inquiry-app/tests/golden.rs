//! The golden investigation, end to end, with no model and no durable runtime.

use std::sync::Arc;

use durable_inquiry_app::{
    InvestigationService, LocalInvestigations,
    fixture::{self, FixtureClock},
};
use durable_inquiry_knowledge::KnowledgeBase;

#[tokio::test]
async fn golden_signal_flood_investigation() {
    let kb = Arc::new(KnowledgeBase::builtin().expect("knowledge"));
    let fixture = fixture::golden().expect("fixture");
    let clock = Arc::new(FixtureClock::new(fixture.open.at));
    let service = LocalInvestigations::new(kb.clone(), clock.clone());
    let (id, reports) = match fixture::replay(&kb, &service, &fixture, Some(&clock), true).await {
        Ok(r) => r,
        Err(e) => panic!("{e}"),
    };
    for r in &reports {
        eprintln!(
            "{:<40} next={:<22} conclusion={:?} concluded={}",
            r.label,
            r.next_request.clone().unwrap_or_default(),
            r.conclusion,
            r.concluded
        );
    }
    let snapshot = service.snapshot(&id).await.expect("snapshot");
    let conclusion = snapshot.investigation.conclusion.expect("concluded");
    eprintln!("\n{}\n", conclusion.statement);
    assert!(
        conclusion.statement.contains("2,000/s"),
        "{}",
        conclusion.statement
    );
    assert!(
        !conclusion
            .recommendations
            .iter()
            .any(|r| r.text.to_lowercase().starts_with("add worker"))
    );
}

/// The workspace model and the Markdown export render the golden diagnosis,
/// every finding's evidence, and all three diagrams.
#[tokio::test]
async fn golden_workspace_and_export() {
    let kb = Arc::new(KnowledgeBase::builtin().expect("knowledge"));
    let fixture = fixture::golden().expect("fixture");
    let clock = Arc::new(FixtureClock::new(fixture.open.at));
    let service = LocalInvestigations::new(kb.clone(), clock.clone());
    let (id, _) = fixture::replay(&kb, &service, &fixture, Some(&clock), true)
        .await
        .expect("replay");
    let snapshot = service.snapshot(&id).await.expect("snapshot");
    let assessment = durable_inquiry_app::core::current_assessment(&kb, &snapshot.investigation)
        .expect("assessment");
    let ws = durable_inquiry_app::workspace::build(&kb, &snapshot, &assessment);
    assert_eq!(ws.status.token, "concluded");
    assert_eq!(
        ws.diagrams.len(),
        3,
        "diagnostic path, system path and evidence relationships"
    );
    for d in &ws.diagrams {
        assert!(d.svg.contains("role=\"img\""));
        assert!(!d.text.is_empty());
    }
    for f in &ws.findings {
        assert!(!f.basis.is_empty(), "{} has no basis", f.id);
        for b in &f.basis {
            assert!(
                !b.sources.is_empty(),
                "{} basis {} has no source",
                f.id,
                b.signal
            );
        }
    }
    let md = durable_inquiry_app::export::markdown(&ws);
    assert!(md.contains("## Diagnosis"));
    assert!(md.contains("Task Queue dispatch rate limit reached under a Signal flood"));
    std::fs::write(std::env::temp_dir().join("durable-inquiry-golden.md"), &md).expect("write");
    let path_text = &ws.diagrams[0].text;
    eprintln!("{path_text}");
    eprintln!("{}", ws.diagrams[1].text);
    for t in &ws.trail {
        eprintln!("{} {:<11} {}", t.time, t.kind, t.text);
    }
}

/// Values the operator types answer the request they respond to: Durable Inquiry moves
/// on to another request instead of issuing the same one again.
#[tokio::test]
async fn typed_values_answer_their_request() {
    use durable_inquiry_app::{
        Command, EvidenceSubmission, OpenInvestigation, SdkInput, StructuredInput,
    };
    use durable_inquiry_domain::{
        DeclaredContext, InvestigationId, RequestStatus, SignalId, Statistic, SubmissionChannel,
        TimeWindow,
    };

    let kb = Arc::new(KnowledgeBase::builtin().expect("knowledge"));
    let at = |s: &str| -> jiff::Timestamp { s.parse().expect("timestamp") };
    let clock = Arc::new(FixtureClock::new(at("2026-10-13T15:31:38Z")));
    let service = LocalInvestigations::new(kb.clone(), clock);
    let id = service
        .open(OpenInvestigation {
            id: InvestigationId::new("inv-typed-values"),
            title: None,
            description: "Elevated schedule to start latencies".to_owned(),
            signature: None,
            server_version: Some("1.31.2".to_owned()),
            sdks: vec![SdkInput {
                language: "java".to_owned(),
                version: None,
            }],
            topology: None,
            namespaces: Vec::new(),
            task_queues: Vec::new(),
            window_start: Some(at("2026-10-13T11:42:00Z")),
            window_end: Some(at("2026-10-13T11:58:00Z")),
            window_description: None,
            profile: None,
        })
        .await
        .expect("open");
    let first = service
        .snapshot(&id)
        .await
        .expect("snapshot")
        .investigation
        .current_request()
        .cloned()
        .expect("a first request");
    assert_eq!(first.definition.as_str(), "workflow-worker-slots");

    let window =
        TimeWindow::new(at("2026-10-13T11:42:00Z"), at("2026-10-13T11:58:00Z")).expect("window");
    service
        .command(
            &id,
            Command::SubmitEvidence(EvidenceSubmission {
                channel: SubmissionChannel::StructuredEntry,
                text: None,
                bytes_base64: None,
                filename: None,
                media_type: Some("application/json".to_owned()),
                declared: DeclaredContext {
                    signal: Some(SignalId::new("workflow_slots_available")),
                    window: Some(window),
                    ..DeclaredContext::default()
                },
                responds_to: Some(first.id.clone()),
                structured: Some(StructuredInput {
                    signal: "workflow_slots_available".to_owned(),
                    statistic: Statistic::Mean,
                    value: 4769.0,
                    unit: None,
                }),
                origin: None,
                stored: None,
            }),
        )
        .await
        .expect("submit");

    let investigation = service.snapshot(&id).await.expect("snapshot").investigation;
    let answered = investigation
        .requests
        .iter()
        .find(|r| r.id == first.id)
        .expect("the first request");
    assert!(
        matches!(answered.status, RequestStatus::Answered { .. }),
        "{:?}",
        answered.status
    );
    let next = investigation.current_request().expect("a next request");
    assert_ne!(
        next.definition, first.definition,
        "the request the typed values answered was issued again"
    );
}

/// The journal Kyumei 0.1.0 wrote for the reference investigation, as recorded
/// in `fixtures/recorded/kyumei-0.1.0`: opened with `serve --in-process --demo 0`,
/// the first request read by Collect now, the next four answered by pasting,
/// and the simulation the fifth calibrates. One request is still open.
const RECORDED_JOURNAL: &str = "../recorded/kyumei-0.1.0/journal/inv-golden-signal-flood.json";

/// A data directory written before the rename opens as it is, reads the same
/// and continues. What it recorded under the earlier names (the engine, the
/// collector, the simulator) is neither needed nor rewritten.
#[tokio::test]
async fn a_journal_written_before_the_rename_opens_and_continues() {
    use durable_inquiry_domain::{EvidenceOrigin, RecordedEvent};

    let kb = Arc::new(KnowledgeBase::builtin().expect("knowledge"));
    let bytes = std::fs::read(fixture::fixtures_dir().join(RECORDED_JOURNAL)).expect("recorded");
    let recorded: Vec<RecordedEvent> = serde_json::from_slice(&bytes).expect("events");
    // A copy, since the service writes to its journal: the recording stays as it was.
    let dir = std::env::temp_dir().join(format!(
        "durable-inquiry-recorded-journal-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create");
    std::fs::write(dir.join("inv-golden-signal-flood.json"), &bytes).expect("copy");
    let open = || {
        LocalInvestigations::new(kb.clone(), Arc::new(durable_inquiry_app::SystemClock))
            .with_simulation()
            .with_journal(dir.clone())
            .expect("the journal opens")
    };

    // It is listed, and every event reads as it was written.
    let service = open();
    let listed = service.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    let id = listed[0].id.clone();
    assert_eq!(id.as_str(), "inv-golden-signal-flood");
    assert_eq!(listed[0].evidence, 6, "five answers and a simulation");
    assert_eq!(listed[0].conclusion, None);
    let snapshot = service.snapshot(&id).await.expect("snapshot");
    let investigation = &snapshot.investigation;
    assert_eq!(investigation.events, recorded);
    let origin = |e: &str| {
        investigation
            .evidence(&e.into())
            .expect("evidence")
            .origin
            .describe()
    };
    assert_eq!(origin("E1"), "Collector kyumei 0.1.0");
    assert_eq!(origin("E6"), "Simulation by tempdes 22b8dc9, seed 1");
    let last = investigation.latest_assessment().expect("an assessment");
    assert_eq!(last.engine, "kyumei-diagnostics/0.1.0");

    // It reads the same: the engine's assessment now is the one recorded then.
    let now = durable_inquiry_app::core::current_assessment(&kb, investigation).expect("assess");
    assert_eq!(
        now.hypotheses
            .iter()
            .map(|h| (h.id.clone(), (h.state, h.contested)))
            .collect::<std::collections::BTreeMap<_, _>>(),
        last.states
            .iter()
            .map(|(id, s)| (id.clone(), (s.state, s.contested)))
            .collect()
    );
    assert_eq!(
        now.findings.iter().map(|f| &f.id).collect::<Vec<_>>(),
        last.findings.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        now.next().map(|p| &p.definition),
        last.next_request.as_ref()
    );
    assert_eq!(
        now.conclusion.as_ref().map(|c| &c.id),
        last.conclusion.as_ref()
    );
    assert!(
        now.simulations.is_empty(),
        "the simulation recorded then is not due again"
    );

    // It continues: the reference fixture's last step answers the open request.
    let golden = fixture::golden().expect("fixture");
    fixture::step(
        &kb,
        &service,
        &golden,
        &id,
        golden.steps.len() - 1,
        None,
        true,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));
    let done = service.snapshot(&id).await.expect("snapshot").investigation;
    assert_eq!(
        done.conclusion.as_ref().map(|c| c.id.as_str()),
        Some("dispatch-ceiling-under-signal-flood")
    );
    assert_eq!(
        done.events[..recorded.len()],
        recorded[..],
        "nothing rewritten"
    );
    let simulations = |events: &[RecordedEvent]| {
        durable_inquiry_domain::Investigation::replay(events)
            .expect("replay")
            .evidence
            .iter()
            .filter(|e| matches!(e.origin, EvidenceOrigin::Simulation { .. }))
            .count()
    };
    assert_eq!(simulations(&done.events), simulations(&recorded));
    assert_eq!(
        done.latest_assessment().map(|a| a.engine.as_str()),
        Some(durable_inquiry_diagnostics::ENGINE)
    );

    // And what was written now opens again.
    let again = open().snapshot(&id).await.expect("snapshot").investigation;
    assert_eq!(again.events, done.events);
    let _ = std::fs::remove_dir_all(&dir);
}
