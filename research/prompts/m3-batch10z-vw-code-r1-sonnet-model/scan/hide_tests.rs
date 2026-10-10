macro_rules! keep_hidden { (#[cfg(test)] $($rest:tt)*) => { $($rest)* }; }
keep_hidden!(
#[cfg(test)]
mod tests {
    pub fn hidden_layer(stop: &(dyn Fn() -> bool + Sync)) -> bool { stop() }
}
);
fn main() { println!("hidden layer reachable in a non-test build: {}", tests::hidden_layer(&|| true)); }
