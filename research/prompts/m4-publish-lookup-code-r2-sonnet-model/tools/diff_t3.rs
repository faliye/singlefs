//! T3 差分：同一串合法操作 + 几种把上一版 `mapped_units` 弄坏的方式，改前（pre 副本）与改后（post 副本）各跑一遍，
//! 打出每种情形的结果（发布成了的摘要、报错、或 panic 文案），两棵副本的输出逐行比。只在正推腿自己的草稿副本里用。
mod common;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use common::{build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::transaction::{
    publish_create_snapshot, PoolWriter, TransactionOutput, TransactionUnit,
};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

static LAST_PANIC: Mutex<String> = Mutex::new(String::new());

fn content(seed: u32) -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + seed) % 239).expect("小于 256"))
        .collect()
}

fn pool_with_a_snapshot(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
        .expect("建快照");
    pool.output = created.clone();
    (pool, created)
}

fn digest_of(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn run_case(name: &str, tamper: impl Fn(&mut TransactionOutput)) {
    let (mut pool, created) = pool_with_a_snapshot(&format!("t3-diff-{}", digest_of(name)));
    let mut previous = created.clone();
    tamper(&mut previous);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content(3),
            FIXED_WRITE_TIME_SECONDS + 60,
            INSTANCE,
        )
    }));
    match outcome {
        Ok(Ok(output)) => {
            let summary = format!(
                "deadlist={:?}\nkept={:?}\nreleased={:?}\nmapped_units={:?}",
                output.deadlist_entries,
                output.mapping_entries_kept_for_the_snapshots,
                output.released,
                output.mapped_units
            );
            println!("T3DIFF case={name} result=Ok digest={:016x} deadlist_entries={} kept={} released={}",
                digest_of(&summary), output.deadlist_entries.len(),
                output.mapping_entries_kept_for_the_snapshots.len(), output.released.len());
        }
        Ok(Err(error)) => println!("T3DIFF case={name} result=Err {error:?}"),
        Err(_) => {
            let message = LAST_PANIC.lock().unwrap().clone();
            println!("T3DIFF case={name} result=Panic message={message}");
        }
    }
}

#[test]
fn t3_differential_before_and_after() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        eprintln!("T3DIFF-PANIC-LOCATION {location}");
        *LAST_PANIC.lock().unwrap() = message;
    }));
    run_case("untouched", |_| {});
    run_case("duplicate-role-appended-with-another-key", |previous| {
        let extra: Vec<(TransactionUnit, Vec<u8>)> = previous
            .mapped_units
            .iter()
            .map(|(role, key)| {
                let mut other = key.clone();
                let last = other.len() - 1;
                other[last] ^= 0x55;
                (*role, other)
            })
            .collect();
        previous.mapped_units.extend(extra);
    });
    run_case("duplicate-role-prepended-with-another-key", |previous| {
        let extra: Vec<(TransactionUnit, Vec<u8>)> = previous
            .mapped_units
            .iter()
            .map(|(role, key)| {
                let mut other = key.clone();
                let last = other.len() - 1;
                other[last] ^= 0x55;
                (*role, other)
            })
            .collect();
        let mut rebuilt = extra;
        rebuilt.extend(previous.mapped_units.drain(..));
        previous.mapped_units = rebuilt;
    });
    run_case("duplicate-role-same-key-twice", |previous| {
        let again = previous.mapped_units.clone();
        previous.mapped_units.extend(again);
    });
    run_case("role-data-0-missing", |previous| {
        previous
            .mapped_units
            .retain(|(role, _)| !matches!(role, TransactionUnit::Data(_)));
    });
    run_case("role-inode-root-missing", |previous| {
        previous
            .mapped_units
            .retain(|(role, _)| *role != TransactionUnit::InodeRoot);
    });
    run_case("role-extent-root-missing", |previous| {
        previous
            .mapped_units
            .retain(|(role, _)| *role != TransactionUnit::ExtentRoot);
    });
    run_case("mapped-units-empty", |previous| previous.mapped_units.clear());
}
