use crate::{immutable_scenario, mutable_scenario};
use futures::executor::block_on;
use rand::rng;

#[test]
#[ntest::timeout(3_000)]
fn immutable_processes_should_never_deadlock() {
    block_on(immutable_scenario(&mut rng(), 32, 200_000));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_without_modification() {
    block_on(mutable_scenario(&mut rng(), 32, 10_000, 0, 0, 1_000));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_after_modification_tracable() {
    block_on(mutable_scenario(&mut rng(), 10, 1000, 0, 50, 10));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_after_modification() {
    block_on(mutable_scenario(&mut rng(), 32, 10_000, 0, 500, 1_000));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_after_modification_with_additional_stuff() {
    block_on(mutable_scenario(&mut rng(), 32, 10_000, 1000, 5_000, 100));
}
