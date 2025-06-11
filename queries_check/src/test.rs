use crate::{immutable_scenario, mutable_scenario};
use futures::executor::block_on;
use rand::rng;

#[test]
#[ntest::timeout(3000)]
fn immutable_processes_should_never_deadlock() {
    block_on(immutable_scenario(&mut rng(), 1000, 200_000));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_without_modification() {
    block_on(mutable_scenario(&mut rng(), 100, 10_000, 0, 0, 1_000));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_after_modification() {
    block_on(mutable_scenario(&mut rng(), 10, 100, 0, 50, 10));
}

#[test]
#[ntest::timeout(3000)]
fn processes_should_return_correct_values_after_modification_with_additional_stuff() {
    block_on(mutable_scenario(&mut rng(), 100, 10_000, 0, 5_000, 1_00));
}
