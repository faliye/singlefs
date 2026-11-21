mod ledger;
fn main() {
    let mut ledger = ledger::Ledger { rows_counted: 0 };
    let measured = 3 + 4;
    ledger.rows_counted = measured;
    println!("{}", ledger.rows_counted);
}
