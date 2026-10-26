use vstd::prelude::*;
verus! {
pub enum Refusal { Bad }
pub fn step(a: u64) -> (r: Result<u64, Refusal>) { if a == 0 { Err(Refusal::Bad) } else { Ok(a) } }
// transaction.rs:9928 的形状：循环里 push(f(...)?)
pub fn all(planned: &Vec<u64>) -> (r: Result<Vec<u64>, Refusal>) {
    let mut out: Vec<u64> = Vec::new();
    let mut i: usize = 0;
    while i < planned.len()
        invariant i <= planned.len(), out.len() == i,
        decreases planned.len() - i,
    {
        out.push(step(planned[i])?);
        i += 1;
    }
    Ok(out)
}
}
fn main() {}
