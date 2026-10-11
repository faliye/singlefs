
#[allow(dead_code)]
fn probe_exempt(position: usize, len: usize) -> bool {
    if position + 1 == len {
        return true;
    }
    match std::env::var("PROBE_VARIANT").as_deref() {
        Ok("second_to_last") => position + 2 != len,
        Ok("k2") => position + 3 < len,
        Ok("k3") => position + 4 < len,
        _ => false,
    }
}
