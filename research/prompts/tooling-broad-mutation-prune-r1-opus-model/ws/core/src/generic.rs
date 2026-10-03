/// S8 泛型（在调用方 crate 里单态化）、`#[inline]`、闭包：量覆盖把它们记在哪个文件。
pub fn largest<T: Ord + Copy>(items: &[T]) -> T {
    let mut best = items[0];
    for &item in items {
        if item > best {
            best = item;
        }
    }
    best
}

#[inline]
pub fn halve(value: u64) -> u64 {
    value / 2
}

pub fn apply_twice(value: u64) -> u64 {
    let add_one = |x: u64| x + 1;
    add_one(add_one(value))
}
