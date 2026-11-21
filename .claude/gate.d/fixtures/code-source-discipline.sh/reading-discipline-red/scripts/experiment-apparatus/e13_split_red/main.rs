mod ledger;
fn main() {
    let seed = 7u64;
    let state = seed | 1;
    let counters = ledger::Counters { rows_seen: 0 };
    println!("{} {}", state, counters.rows_seen);
}
