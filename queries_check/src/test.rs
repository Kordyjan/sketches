use crate::immutable_scenario;
use futures::executor::block_on;
use rand::rng;

#[test]
#[ntest::timeout(3000)]
fn immutable_processes_should_never_deadlock() {
    block_on(immutable_scenario(&mut rng(), 1000, 200_000));
}
