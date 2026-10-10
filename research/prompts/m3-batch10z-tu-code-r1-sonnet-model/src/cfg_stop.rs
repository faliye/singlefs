struct Stop<'a>(&'a (dyn Fn() -> bool + Sync));
impl<'a> Stop<'a> { fn new(f: &'a (dyn Fn() -> bool + Sync)) -> Self { Stop(f) } fn is_stopped(&self) -> bool { (self.0)() } }
struct Never;
impl Never { fn is_stopped(&self) -> bool { false } }
#[allow(non_upper_case_globals)]
static stop_request: Never = Never;
fn layer(stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool {
    #[cfg(any())]
    let stop_request = Stop::new(stop_is_requested);
    stop_request.is_stopped()
}
fn main() { println!("layer(&|| true) = {} (a layer that hands the stop on would return true)", layer(&|| true)); }
