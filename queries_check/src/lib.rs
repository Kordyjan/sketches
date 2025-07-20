use crate::query::{mix, NonlockingProcess, Process, INPUT};
use async_global_executor::spawn;
use async_std::sync::RwLock;
use divisors_fixed::Divisors;
use futures::future::TryJoinAll;
use itertools::Itertools;
use queries::fingerprinting::stamp_with_fingerprint;
use queries::{execution::Reactor, Executor};
use rand::{distr, Rng};
use rustc_hash::FxHashMap;
use std::{collections::HashMap, iter, sync::Arc};

pub(crate) mod query;

#[allow(dead_code)]
async fn immutable_scenario(rng: &mut impl Rng, input_size: usize, process_count: usize) {
    let input: Vec<String> = iter::repeat_with(|| ((rng.random::<u8>() % 26) + b'a') as char)
        .map(|c| c.to_string())
        .take(input_size)
        .collect();
    let reactor = Arc::new(Reactor::new());
    reactor.set_param(&INPUT, input);
    iter::repeat_with(|| rng.sample(distr::Uniform::new(0, input_size).unwrap()))
        .take(process_count)
        .map(|n| {
            let reactor = Arc::clone(&reactor);
            spawn(async move { reactor.execute(NonlockingProcess(n)).await })
        })
        .collect::<TryJoinAll<_>>()
        .await
        .unwrap();
}

#[allow(dead_code)]
async fn mutable_scenario(
    rng: &mut (impl Rng + Clone),
    input_size: usize,
    checked_process_count: usize,
    immutable_process_count: usize,
    modification_count: usize,
    max_weight: usize,
) {
    let mut input: Vec<String> = iter::repeat_with(|| ((rng.random::<u8>() % 26) + b'a') as char)
        .map(|c| c.to_string())
        .take(input_size)
        .collect();

    let checked_processes =
        iter::repeat_with(|| rng.sample(distr::Uniform::new(0, input_size).unwrap()))
            .take(checked_process_count)
            .map(Op::Calculate)
            .collect::<Vec<_>>();

    let immutable_processes =
        iter::repeat_with(|| rng.sample(distr::Uniform::new(0, input_size).unwrap()))
            .take(immutable_process_count)
            .map(Op::CalculateNonlocking)
            .collect::<Vec<_>>();

    let modifications = iter::repeat_with(|| {
        let n = rng.sample(distr::Uniform::new(0, input_size).unwrap());
        let c = ((rng.random::<u8>() % 26) + b'a') as char;
        Op::Replace(n, c.to_string())
    })
    .take(modification_count)
    .collect::<Vec<_>>();

    let groups = checked_processes
        .into_iter()
        .chain(immutable_processes)
        .chain(modifications)
        .map(|op| (rng.sample(distr::Uniform::new(0, max_weight).unwrap()), op))
        .into_group_map();

    let steps = calc_steps(&groups);

    let (output, tracer, chapter_marker) = tracer::create_full("../tmp/trace.json").unwrap();

    let lock = Arc::new(RwLock::new(()));
    spawn(output.run()).detach();
    let reactor = Arc::new(Reactor::with_trace(tracer));
    reactor.set_param(&INPUT, input.clone());

    for Step {
        updates,
        processes,
        nonlocking_processes,
    } in steps
    {
        let guard = lock.write().await;
        let processes_queries = processes
            .clone()
            .into_iter()
            .map(|n| {
                let reactor = Arc::clone(&reactor);
                let lock = Arc::clone(&lock);
                async move { reactor.execute(Process(n, lock)).await }
            })
            .collect::<TryJoinAll<_>>();
        let nonlocking_processes = nonlocking_processes
            .into_iter()
            .map(|n| {
                let reactor = Arc::clone(&reactor);
                async move { reactor.execute(NonlockingProcess(n)).await }
            })
            .collect::<TryJoinAll<_>>();

        let process_task = spawn(processes_queries);
        spawn(nonlocking_processes).detach();

        let old_input = input.clone();
        for (n, v) in updates {
            input[n] = v;
        }
        let (fingerprint, _) = stamp_with_fingerprint(Arc::new(input.clone()));
        chapter_marker.new_chapter(format!("|{}|", input.join("")), format!("{fingerprint:?}"));
        reactor.set_param(&INPUT, input.clone());

        let mut cache = FxHashMap::default();
        let mut old_cache = FxHashMap::default();
        let expectations = processes
            .into_iter()
            .map(|n| {
                (
                    n,
                    check(n, &old_input, &mut old_cache),
                    check(n, &input, &mut cache),
                )
            })
            .collect::<Vec<_>>();

        drop(guard);
        let results = process_task.await.unwrap().into_iter().collect::<Vec<_>>();

        println!("│{}│", (0..10).join(""));
        println!("│{}│", old_input.join(""));
        println!("│{}│\n", input.join(""));

        for ((n, exp1, exp2), result) in expectations.into_iter().zip(results) {
            println!("{n}: ( {exp1} | {exp2} ) {result}");
            assert!(result == Arc::new(exp1) || result == Arc::new(exp2));
        }
        println!();
    }
}

fn calc_steps(groups: &HashMap<usize, Vec<Op>>) -> Vec<Step> {
    let mut steps = Vec::<Step>::new();
    for weight in groups.keys().sorted() {
        let group = &groups[weight];
        let updates = group
            .iter()
            .filter_map(|op| match op {
                Op::Replace(n, v) => Some((*n, v.clone())),
                _ => None,
            })
            .collect();
        let processes = group
            .iter()
            .filter_map(|op| match op {
                Op::Calculate(n) => Some(*n),
                _ => None,
            })
            .collect();
        let nonlocking_processes = group
            .iter()
            .filter_map(|op| match op {
                Op::CalculateNonlocking(n) => Some(*n),
                _ => None,
            })
            .collect();
        steps.push(Step {
            updates,
            processes,
            nonlocking_processes,
        });
    }
    steps
}

fn check(n: usize, input: &[String], cache: &mut FxHashMap<usize, String>) -> String {
    let this = input[n].clone();
    let divisors = (n + 1).divisors();
    let len = divisors.len();
    if len == 1 {
        return this;
    }
    divisors[..(len - 1)]
        .iter()
        .map(|m| {
            let m = *m - 1;
            if let Some(v) = cache.get(&m) {
                v.clone()
            } else {
                let v = check(m, input, cache);
                cache.insert(m, v.clone());
                v
            }
        })
        .fold(this, |acc, it| mix(acc, &it))
}

struct Step {
    updates: Vec<(usize, String)>,
    processes: Vec<usize>,
    nonlocking_processes: Vec<usize>,
}

enum Op {
    Replace(usize, String),
    Calculate(usize),
    CalculateNonlocking(usize),
}

#[cfg(test)]
mod test;
