//! m3-ring-r2 攻方原型 G4：连崩写行（K1）留下的实例 j 没有一条根进过环、实例表里是 (j, 0, 0) 行；
//! 它写行那次写下的单元（单元头写序里的实例代号 = j）在下一次挂载之后的几次发布里还剩几个、什么时候被盖完。
//! 判「未发布」：单元头读得出、实例代号 = j，且不在环里任何一条读得出的根（有效与被抛弃都算）引用的槽里。
//!
//! 用法：orphan_units_probe <S> <k1|k1x2> <K> <扫描槽数>
use std::collections::{BTreeMap, BTreeSet};

use ring_window_proto::{build_prefix, content, crash_a_mount_before_its_row_root, devices_of, parameters, pool_of, WRITE_TIME};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{allocation_records_under_root, choose_system_configuration, instance_table_of_root, readable_roots_with_ring_slots, PoolReader};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_core::unit::{parse_data_unit, parse_index_node, parse_packed_unit};
use singlefs_harness::memory_pool::MemoryPool;

const SLOT: u64 = 16384;

fn referenced(image: &MemoryPool) -> BTreeSet<u64> {
    let c = choose_system_configuration(image).expect("系统配置");
    let im = &c.immutable;
    let mut out = BTreeSet::new();
    for (_, root) in readable_roots_with_ring_slots(image, &im.region_devices, &im.sizes, &im.filesystem_identifier) {
        if let Ok(records) = allocation_records_under_root(image, &root) {
            for r in records.iter().filter(|r| r.device == DeviceIdentity(0) && !r.is_released) {
                out.extend(r.slot.0..r.slot.0 + u64::from(r.span_slots));
            }
        }
        for (locations, span) in [(&root.instance_table.locations, 2u64), (&root.tree_table.locations, 1u64)] {
            for l in locations.iter().filter(|l| l.device == DeviceIdentity(0) && l.slot.0 != 0) {
                out.extend(l.slot.0..l.slot.0 + span);
            }
        }
    }
    out
}

fn orphans(image: &MemoryPool, start: u64, count: u64) -> BTreeMap<u32, Vec<u64>> {
    let refs = referenced(image);
    let mut by_instance: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
    for slot in start..start + count {
        let Some(one) = image.read(DeviceIdentity(0), DeviceOffsetInBytes(slot * SLOT), 16384) else { continue };
        if one.iter().all(|b| *b == 0) {
            continue;
        }
        let two = image.read(DeviceIdentity(0), DeviceOffsetInBytes(slot * SLOT), 32768).unwrap_or_default();
        let instance = parse_index_node(&one).map(|h| h.instance.0).ok()
            .or_else(|| parse_packed_unit(&two).map(|h| h.write_order.instance.0).ok())
            .or_else(|| parse_data_unit(&two).map(|h| h.write_order.instance.0).ok());
        if let Some(instance) = instance {
            if !refs.contains(&slot) {
                by_instance.entry(instance).or_default().push(slot);
            }
        }
    }
    by_instance
}

fn rows(image: &MemoryPool) -> Vec<(u32, u64)> {
    let c = choose_system_configuration(image).expect("系统配置");
    let im = &c.immutable;
    let newest = readable_roots_with_ring_slots(image, &im.region_devices, &im.sizes, &im.filesystem_identifier)
        .into_iter().map(|(_, r)| r).max_by_key(|r| (r.checkpoint_txg, r.instance)).expect("根");
    instance_table_of_root(image, &newest).map(|t| t.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0)).collect()).unwrap_or_default()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let history = a[2].as_str();
    let k: u64 = a[3].parse().expect("K");
    let scan: u64 = a[4].parse().expect("扫描槽数");
    let parameters = parameters(s);
    let hole = 3 * s + 4;
    std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
    let prefix = build_prefix(s, hole);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let base = if history == "k1x2" { crash_a_mount_before_its_row_root(s, &prefix.image).0 } else { prefix.image.clone() };
    let start: u64 = std::env::var("SCAN_START").ok().and_then(|v| v.parse().ok()).unwrap_or(50_000);
    println!("PROBE s={s} history={history} step=crashed rows={:?} orphans={:?}", rows(&base), orphans(&base, start, scan).iter().map(|(i, v)| (*i, v.len())).collect::<Vec<_>>());
    let mut devices = devices_of(&base);
    let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("带文件");
    let image = pool_of(&devices);
    println!("PROBE s={s} history={history} step=mounted instance={} rows={:?} orphans={:?}", instance.0, rows(&image), orphans(&image, start, scan).iter().map(|(i, v)| (*i, v.len(), v.iter().take(6).copied().collect::<Vec<_>>())).collect::<Vec<_>>());
    for step in 0..k {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(700 + step), write_time_seconds: WRITE_TIME + 700 + step }, instance).expect("覆盖写");
        let image = pool_of(&devices);
        println!("PROBE s={s} history={history} step={step} orphans={:?}", orphans(&image, start, scan).iter().map(|(i, v)| (*i, v.len(), v.iter().take(6).copied().collect::<Vec<_>>())).collect::<Vec<_>>());
    }
}
