use crate::nonlocking_scenario;
use futures::executor::block_on;
use rand::rng;

#[test]
#[ntest::timeout(3000)]
fn nonlocking_processes_should_never_deadlock() {
    block_on(nonlocking_scenario(&mut rng(), 1000, 200_000));
}
