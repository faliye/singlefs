//! 对照样本：两个读数都先写成 0 再真的写进值，恒为字面量 0 那一条不该碰它们——
//! `peak_writes` 由同名局部变量经字段简写初始化，`rounds_seen` 经方法调用的结果写字段。

struct Reading {
    peak_writes: u64,
    rounds_seen: u64,
}

fn empty_reading() -> Reading {
    Reading { peak_writes: 0, rounds_seen: 0 }
}

fn reading_slot(readings: &mut [Reading], index: usize) -> &mut Reading {
    &mut readings[index]
}

fn reading_with_peak(round_count: u64) -> Reading {
    let peak_writes = round_count * 3;
    Reading {
        peak_writes,
        rounds_seen: 0,
    }
}

fn measure(round_count: u64) -> Reading {
    let mut readings = vec![empty_reading(), reading_with_peak(round_count)];
    reading_slot(&mut readings, 1).rounds_seen = round_count;
    readings.remove(1)
}

fn main() {
    let reading = measure(4);
    println!("peak_writes={} rounds_seen={}", reading.peak_writes, reading.rounds_seen);
}

#[cfg(test)]
mod tests {
    #[test]
    fn four_rounds_write_twelve() { assert_eq!(super::measure(4).peak_writes, 12); }
}
