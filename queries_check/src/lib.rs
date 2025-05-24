pub(crate) mod query;

struct Step;

enum Op {
    Replace(u64, u64),
    Calculate(u64),
}

#[cfg(test)]
mod test;
