//! 攻方腿的模型（m4-gpugc-publish-growth-r1）：量「单元字节换成共享不可变字节（甲档）之后，每次发布那一组深拷还剩多少」。
//! 只用 std，不连 crates/：照 `crates/singlefs-core/src/transaction.rs` 的 `PublishedUnit`（落点、身份、字节）造同形的单元，
//! 一次「发布」照今天的四步走：照抄上一版每个单元（重写其中一个）、整版再拷一份交给调用方、换掉现行那一版（旧的释放）、丢掉交回的那一份。
//! 另量甲档碰不到的那几张表：每个单元一条带映射 key 的表、映射 key 表、分配记录表（条数取单元数的十倍，同产物里 R/U 约 10）。
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

const DATA_UNIT_BYTES: usize = 32 * 1024;
const METADATA_UNIT_BYTES: usize = 16 * 1024;
const MAPPING_KEY_BYTES: usize = 27;
const ALLOCATION_RECORD_BYTES: usize = 40;

trait UnitBytes: Clone {
    fn from_vec(bytes: Vec<u8>) -> Self;
    fn byte_count(&self) -> usize;
}
impl UnitBytes for Vec<u8> {
    fn from_vec(bytes: Vec<u8>) -> Self { bytes }
    fn byte_count(&self) -> usize { self.len() }
}
impl UnitBytes for Arc<[u8]> {
    fn from_vec(bytes: Vec<u8>) -> Self { Arc::from(bytes) }
    fn byte_count(&self) -> usize { self.len() }
}

#[derive(Clone)]
struct Unit<Bytes: UnitBytes> {
    slot: u64,
    identity: [u64; 2],
    bytes: Bytes,
}

fn fresh_unit<Bytes: UnitBytes>(index: usize, salt: u64) -> Unit<Bytes> {
    // 十一个单元里一个是 16 KiB 的元数据节点，其余是 32 KiB 的数据单元（同 P=2048 交错写末窗口约 9% 的元数据单元）。
    let length = if index % 11 == 10 { METADATA_UNIT_BYTES } else { DATA_UNIT_BYTES };
    let fill = u8::try_from((index as u64 ^ salt) % 251).expect("小于 251");
    Unit { slot: index as u64 * 2 + salt, identity: [index as u64, salt], bytes: Bytes::from_vec(vec![fill; length]) }
}

fn minor_faults() -> u64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let after = stat.rsplit_once(')').map_or("", |(_, rest)| rest);
    after.split_whitespace().nth(7).and_then(|field| field.parse().ok()).unwrap_or(0)
}

fn milliseconds(started: Instant) -> f64 { started.elapsed().as_secs_f64() * 1000.0 }

/// 一种字节持有方式、一个单元数：照今天的四步连发 `publishes` 次（先不计时地发 `warm_up` 次），打每一步每次的毫秒。
fn publishes_of_the_deep_group<Bytes: UnitBytes>(mode: &str, units: usize, warm_up: usize, publishes: usize) -> f64 {
    let mut current: Vec<Unit<Bytes>> = (0..units).map(|index| fresh_unit(index, 0)).collect();
    let (mut carry, mut clone_out, mut replace, mut drop_out) = (0.0, 0.0, 0.0, 0.0);
    let faults_before = minor_faults();
    for round in 0..warm_up + publishes {
        let rewritten = (round * 7919) % units;
        let started = Instant::now();
        let mut next: Vec<Unit<Bytes>> = Vec::with_capacity(units);
        for (index, unit) in current.iter().enumerate() {
            next.push(if index == rewritten { fresh_unit(index, round as u64 + 1) } else { unit.clone() });
        }
        let carry_ms = milliseconds(started);
        let started = Instant::now();
        let handed_out = black_box(next.clone());
        let clone_ms = milliseconds(started);
        let started = Instant::now();
        current = next;
        let replace_ms = milliseconds(started);
        let started = Instant::now();
        drop(handed_out);
        let drop_ms = milliseconds(started);
        if round >= warm_up {
            carry += carry_ms; clone_out += clone_ms; replace += replace_ms; drop_out += drop_ms;
        }
    }
    let faults = minor_faults() - faults_before;
    let count = publishes as f64;
    let total = (carry + clone_out + replace + drop_out) / count;
    let bytes: usize = current.iter().map(|unit| unit.bytes.byte_count()).sum();
    let slots: u64 = current.iter().map(|unit| unit.slot ^ unit.identity[0]).sum();
    black_box(slots);
    println!(
        "name=model_publish mode={mode} units={units} unit_bytes={bytes} publishes={publishes} carry_ms={:.4} clone_ms={:.4} replace_ms={:.4} drop_ms={:.4} deep_group_ms={total:.4} ns_per_unit={:.1} minor_faults_per_publish={:.0}",
        carry / count, clone_out / count, replace / count, drop_out / count,
        total * 1.0e6 / units as f64, faults as f64 / (warm_up + publishes) as f64
    );
    total
}

/// 甲档碰不到的那几张表：整版拷一份、换掉旧的、丢掉交回的那一份（同上面后三步），打每次的毫秒。
fn publishes_of_the_tables_left_by_shared_bytes(units: usize, warm_up: usize, publishes: usize) {
    let records = units * 10;
    let make = |salt: u8| -> (Vec<([u64; 2], Vec<u8>)>, Vec<Vec<u8>>, Vec<[u8; ALLOCATION_RECORD_BYTES]>) {
        (
            (0..units).map(|index| ([index as u64, 1], vec![salt; MAPPING_KEY_BYTES])).collect(),
            (0..units).map(|_| vec![salt; MAPPING_KEY_BYTES]).collect(),
            vec![[salt; ALLOCATION_RECORD_BYTES]; records],
        )
    };
    let mut current = make(1);
    let (mut clone_out, mut replace_and_drop) = (0.0, 0.0);
    for round in 0..warm_up + publishes {
        let started = Instant::now();
        let next = black_box(current.clone());
        let handed_out = black_box(next.clone());
        let clone_ms = milliseconds(started);
        let started = Instant::now();
        current = next;
        drop(handed_out);
        let drop_ms = milliseconds(started);
        if round >= warm_up { clone_out += clone_ms; replace_and_drop += drop_ms; }
    }
    let count = publishes as f64;
    println!(
        "name=model_tables_left units={units} records={records} two_copies_ms={:.4} two_drops_ms={:.4} total_ms={:.4}",
        clone_out / count, replace_and_drop / count, (clone_out + replace_and_drop) / count
    );
}

fn list(argument: Option<String>, default: &[usize]) -> Vec<usize> {
    argument.map_or_else(|| default.to_vec(), |text| text.split(',').map(|item| item.parse().expect("单元数是整数")).collect())
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let owned_byte_units = list(arguments.next(), &[3386, 6724]);
    let shared_byte_units = list(arguments.next(), &[3386, 6724, 60500]);
    let publishes: usize = arguments.next().map_or(20, |text| text.parse().expect("次数是整数"));
    let warm_up = 3;
    for &units in &owned_byte_units {
        let owned = publishes_of_the_deep_group::<Vec<u8>>("owned_vec", units, warm_up, publishes);
        let shared = publishes_of_the_deep_group::<Arc<[u8]>>("shared_arc", units, warm_up, publishes);
        println!("name=model_ratio units={units} owned_over_shared={:.1}", owned / shared);
    }
    for &units in &shared_byte_units {
        if !owned_byte_units.contains(&units) {
            publishes_of_the_deep_group::<Arc<[u8]>>("shared_arc", units, warm_up, publishes);
        }
        publishes_of_the_tables_left_by_shared_bytes(units, warm_up, publishes);
    }
    println!("name=done");
}
