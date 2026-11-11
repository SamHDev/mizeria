//! Sequential and concurrent scenario schedulers.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use futures_util::stream::{FuturesUnordered, StreamExt};

use super::jobs::ScenarioJob;
use super::report::{Reporter, SharedReporter};
use super::{ScenarioStatus, Suite};
use crate::outcome::Outcome;

pub(super) fn apply_status(outcome: &mut Outcome, status: ScenarioStatus) {
    match status {
        ScenarioStatus::Passed => outcome.passed += 1,
        ScenarioStatus::Failed => outcome.failed += 1,
        ScenarioStatus::Undefined => outcome.undefined += 1,
    }
}

/// Walk jobs in order, emitting feature/rule brackets around contiguous groups.
pub(super) fn run_jobs_sequential_sync<W>(
    suite: &Suite<W>,
    jobs: &[ScenarioJob<'_>],
    reporter: &mut Reporter<'_>,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    let mut open_feature: Option<usize> = None;
    let mut open_rule: Option<usize> = None;
    let mut last_feature_job: Option<&ScenarioJob<'_>> = None;
    let mut last_rule_job: Option<&ScenarioJob<'_>> = None;

    for job in jobs {
        if open_feature != Some(job.feature_key) {
            if let Some(prev) = last_rule_job.take()
                && let Some(event) = prev.rule_event()
            {
                reporter.on_rule_finished(&event);
            }
            open_rule = None;
            if let Some(prev) = last_feature_job.take() {
                reporter.on_feature_finished(&prev.feature_event());
            }
            reporter.on_feature_started(&job.feature_event());
            open_feature = Some(job.feature_key);
            last_feature_job = Some(job);
        }

        match (job.rule_key, open_rule) {
            (Some(rk), Some(or)) if rk != or => {
                if let Some(prev) = last_rule_job.take()
                    && let Some(event) = prev.rule_event()
                {
                    reporter.on_rule_finished(&event);
                }
                if let Some(event) = job.rule_event() {
                    reporter.on_rule_started(&event);
                }
                open_rule = Some(rk);
                last_rule_job = Some(job);
            }
            (Some(rk), None) => {
                if let Some(event) = job.rule_event() {
                    reporter.on_rule_started(&event);
                }
                open_rule = Some(rk);
                last_rule_job = Some(job);
            }
            (None, Some(_)) => {
                if let Some(prev) = last_rule_job.take()
                    && let Some(event) = prev.rule_event()
                {
                    reporter.on_rule_finished(&event);
                }
                open_rule = None;
            }
            _ => {}
        }

        let status = suite.run_job_sync(job, reporter);
        apply_status(outcome, status);
    }

    if let Some(prev) = last_rule_job.take()
        && let Some(event) = prev.rule_event()
    {
        reporter.on_rule_finished(&event);
    }
    if let Some(prev) = last_feature_job.take() {
        reporter.on_feature_finished(&prev.feature_event());
    }
}

pub(super) async fn run_jobs_sequential_async<W>(
    suite: &Suite<W>,
    jobs: &[ScenarioJob<'_>],
    reporter: &mut Reporter<'_>,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    let mut open_feature: Option<usize> = None;
    let mut open_rule: Option<usize> = None;
    let mut last_feature_job: Option<&ScenarioJob<'_>> = None;
    let mut last_rule_job: Option<&ScenarioJob<'_>> = None;

    for job in jobs {
        if open_feature != Some(job.feature_key) {
            if let Some(prev) = last_rule_job.take()
                && let Some(event) = prev.rule_event()
            {
                reporter.on_rule_finished(&event);
            }
            open_rule = None;
            if let Some(prev) = last_feature_job.take() {
                reporter.on_feature_finished(&prev.feature_event());
            }
            reporter.on_feature_started(&job.feature_event());
            open_feature = Some(job.feature_key);
            last_feature_job = Some(job);
        }

        match (job.rule_key, open_rule) {
            (Some(rk), Some(or)) if rk != or => {
                if let Some(prev) = last_rule_job.take()
                    && let Some(event) = prev.rule_event()
                {
                    reporter.on_rule_finished(&event);
                }
                if let Some(event) = job.rule_event() {
                    reporter.on_rule_started(&event);
                }
                open_rule = Some(rk);
                last_rule_job = Some(job);
            }
            (Some(rk), None) => {
                if let Some(event) = job.rule_event() {
                    reporter.on_rule_started(&event);
                }
                open_rule = Some(rk);
                last_rule_job = Some(job);
            }
            (None, Some(_)) => {
                if let Some(prev) = last_rule_job.take()
                    && let Some(event) = prev.rule_event()
                {
                    reporter.on_rule_finished(&event);
                }
                open_rule = None;
            }
            _ => {}
        }

        let status = suite.run_job_async(job, reporter).await;
        apply_status(outcome, status);
    }

    if let Some(prev) = last_rule_job.take()
        && let Some(event) = prev.rule_event()
    {
        reporter.on_rule_finished(&event);
    }
    if let Some(prev) = last_feature_job.take() {
        reporter.on_feature_finished(&prev.feature_event());
    }
}

struct BracketTracker {
    feature_remaining: Vec<AtomicUsize>,
    feature_started: Vec<AtomicBool>,
    rule_remaining: Vec<AtomicUsize>,
    rule_started: Vec<AtomicBool>,
}

impl BracketTracker {
    fn new(jobs: &[ScenarioJob<'_>]) -> Self {
        let max_feature = jobs
            .iter()
            .map(|j| j.feature_key)
            .max()
            .map(|k| k + 1)
            .unwrap_or(0);
        let max_rule = jobs
            .iter()
            .filter_map(|j| j.rule_key)
            .max()
            .map(|k| k + 1)
            .unwrap_or(0);

        let mut feature_counts = vec![0usize; max_feature];
        let mut rule_counts = vec![0usize; max_rule];
        for job in jobs {
            feature_counts[job.feature_key] += 1;
            if let Some(rk) = job.rule_key {
                rule_counts[rk] += 1;
            }
        }

        Self {
            feature_remaining: feature_counts.into_iter().map(AtomicUsize::new).collect(),
            feature_started: (0..max_feature).map(|_| AtomicBool::new(false)).collect(),
            rule_remaining: rule_counts.into_iter().map(AtomicUsize::new).collect(),
            rule_started: (0..max_rule).map(|_| AtomicBool::new(false)).collect(),
        }
    }

    fn on_start(&self, job: &ScenarioJob<'_>, reporter: &SharedReporter) {
        if !self.feature_started[job.feature_key].swap(true, Ordering::AcqRel) {
            reporter.emit(|l| l.on_feature_started(&job.feature_event()));
        }
        if let Some(rk) = job.rule_key
            && !self.rule_started[rk].swap(true, Ordering::AcqRel)
            && let Some(event) = job.rule_event()
        {
            reporter.emit(|l| l.on_rule_started(&event));
        }
    }

    fn on_finish(&self, job: &ScenarioJob<'_>, reporter: &SharedReporter) {
        if let Some(rk) = job.rule_key
            && self.rule_remaining[rk].fetch_sub(1, Ordering::AcqRel) == 1
            && let Some(event) = job.rule_event()
        {
            reporter.emit(|l| l.on_rule_finished(&event));
        }
        if self.feature_remaining[job.feature_key].fetch_sub(1, Ordering::AcqRel) == 1 {
            reporter.emit(|l| l.on_feature_finished(&job.feature_event()));
        }
    }
}

/// Run concurrent batches split by `@serial` jobs.
pub(super) async fn run_jobs_concurrent_async<W>(
    suite: &Suite<W>,
    jobs: &[ScenarioJob<'_>],
    max: usize,
    reporter: &SharedReporter,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    let max = max.max(1);
    let tracker = BracketTracker::new(jobs);
    let mut index = 0usize;

    while index < jobs.len() {
        if jobs[index].serial {
            let job = &jobs[index];
            tracker.on_start(job, reporter);
            let mut rep = reporter.as_reporter();
            let status = suite.run_job_async(job, &mut rep).await;
            apply_status(outcome, status);
            tracker.on_finish(job, reporter);
            index += 1;
            continue;
        }

        let batch_start = index;
        while index < jobs.len() && !jobs[index].serial {
            index += 1;
        }
        let batch = &jobs[batch_start..index];
        run_async_batch(suite, batch, max, &tracker, reporter, outcome).await;
    }
}

async fn run_async_batch<W>(
    suite: &Suite<W>,
    batch: &[ScenarioJob<'_>],
    max: usize,
    tracker: &BracketTracker,
    reporter: &SharedReporter,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    let mut in_flight = FuturesUnordered::new();
    let mut next = 0usize;

    while next < batch.len() || !in_flight.is_empty() {
        while in_flight.len() < max && next < batch.len() {
            let job = &batch[next];
            next += 1;
            tracker.on_start(job, reporter);
            let reporter = reporter.clone();
            in_flight.push(async move {
                let mut rep = reporter.as_reporter();
                let status = suite.run_job_async(job, &mut rep).await;
                tracker.on_finish(job, &reporter);
                status
            });
        }

        if let Some(status) = in_flight.next().await {
            apply_status(outcome, status);
        }
    }
}

pub(super) fn run_jobs_threaded<W>(
    suite: &Suite<W>,
    jobs: &[ScenarioJob<'_>],
    max: usize,
    reporter: &SharedReporter,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    let max = max.max(1);
    let tracker = BracketTracker::new(jobs);
    let mut index = 0usize;

    while index < jobs.len() {
        if jobs[index].serial {
            let job = &jobs[index];
            tracker.on_start(job, reporter);
            let mut rep = reporter.as_reporter();
            let status = suite.run_job_sync(job, &mut rep);
            apply_status(outcome, status);
            tracker.on_finish(job, reporter);
            index += 1;
            continue;
        }

        let batch_start = index;
        while index < jobs.len() && !jobs[index].serial {
            index += 1;
        }
        let batch = &jobs[batch_start..index];
        run_threaded_batch(suite, batch, max, &tracker, reporter, outcome);
    }
}

fn run_threaded_batch<W>(
    suite: &Suite<W>,
    batch: &[ScenarioJob<'_>],
    max: usize,
    tracker: &BracketTracker,
    reporter: &SharedReporter,
    outcome: &mut Outcome,
) where
    W: Send + Sync + 'static,
{
    if batch.is_empty() {
        return;
    }

    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(batch.len()));
    let workers = max.min(batch.len()).max(1);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= batch.len() {
                        break;
                    }
                    let job = &batch[i];
                    tracker.on_start(job, reporter);
                    let mut rep = reporter.as_reporter();
                    let status = suite.run_job_sync(job, &mut rep);
                    tracker.on_finish(job, reporter);
                    results.lock().unwrap().push(status);
                }
            });
        }
    });

    for status in results.into_inner().unwrap() {
        apply_status(outcome, status);
    }
}

pub(super) fn take_shared_listener(
    listener: &mut Option<Box<dyn crate::listener::Listener>>,
) -> (SharedReporter, bool) {
    match listener.take() {
        Some(l) => (SharedReporter::from_listener(l), true),
        None => (SharedReporter::nop(), false),
    }
}

pub(super) fn restore_listener(
    listener: &mut Option<Box<dyn crate::listener::Listener>>,
    shared: SharedReporter,
    had_listener: bool,
) {
    if had_listener {
        *listener = shared.into_listener();
    }
}
