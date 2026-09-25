//! seed540096 查因：把 s_opus_s23 `s23_random_series_oracle` 里种子 540096 那一段历史原样重走（随机数的消耗次序逐次相同），
//! 每一步前后打出：各盘两个系统配置槽（自证过没有、槽世代号、实例代号、journal tail）、根环里读得出的根（实例、txg、区域、槽）、
//! 各盘 journal 环里自证过的记录（计数器、实例、txg、事务号、反向链、按计数器 − 1 那条算出来的期望链值）、每次挂载取到的号
//! 与按条款式子现算的号、录制流逐条（哪一步落了哪一步没落），以及每一步之后池级 checker 的红项原文。只在草稿拷贝里跑。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use common::{
    build_pool, crash_state_devices, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::mount::{mount_rollback, mount_writable, MountError, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, readable_roots_with_ring_slots,
    PoolReader,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

fn pool_through(tag: &str, last_txg: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let mut current = pool.output.clone();
    let mut seed = 1;
    while current.root.checkpoint_txg.0 < last_txg {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(seed),
            FIXED_WRITE_TIME_SECONDS + 60 * seed as u64,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        seed += 1;
    }
    pool.output = current;
    pool
}

fn image_through(last_txg: u64) -> MemoryPool {
    let pool = pool_through(&format!("seed540096-{last_txg}"), last_txg);
    let image = pool.memory_pool();
    drop(pool);
    image
}

fn roots_of(image: &MemoryPool) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<_> = readable_roots(
        image,
        &sc.immutable.region_devices,
        &sc.immutable.sizes,
        &sc.immutable.filesystem_identifier,
    )
    .iter()
    .map(|root| (root.instance, root.checkpoint_txg))
    .collect();
    roots.sort_by_key(|(i, t)| (*t, *i));
    roots
}

fn corrupt_the_oldest_root_slot(image: &mut MemoryPool) -> (u32, u64) {
    let (instance, txg) = roots_of(image)[0];
    let target = target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    image.flip_byte(
        device,
        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing),
        100,
    );
    (instance.0, txg.0)
}

fn slot_generation_newest(image: &MemoryPool, device: DeviceIdentity) -> Option<u64> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut best: Option<(u64, u64)> = None;
    let mut valid = 0;
    for offset in [0, spacing] {
        let Some(bytes) = PoolReader::read(image, device, DeviceOffsetInBytes(offset), slot_bytes) else {
            continue;
        };
        if let Ok(sc) = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
            valid += 1;
            let generation = sc.quantities.slot_generation;
            if best.map_or(true, |(g, _)| generation > g) {
                best = Some((generation, offset));
            }
        }
    }
    if valid == 2 { best.map(|(_, offset)| offset) } else { None }
}

fn degrade_the_newest_system_configuration_slots(image: &mut MemoryPool) -> Option<Vec<(u32, u64)>> {
    let devices = [DeviceIdentity(0), DeviceIdentity(1)];
    let offsets: Vec<Option<u64>> = devices.iter().map(|device| slot_generation_newest(image, *device)).collect();
    if offsets.iter().any(Option::is_none) {
        return None;
    }
    let mut flipped = Vec::new();
    for (device, offset) in devices.iter().zip(offsets) {
        let offset = offset.expect("判过");
        image.flip_byte(*device, DeviceOffsetInBytes(offset), 100);
        flipped.push((device.0, offset / 4096));
    }
    Some(flipped)
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

// ---------------- 读数 ----------------

/// 各盘两槽：(槽号, 自证过没有, 槽世代号, 实例代号, journal tail)。
fn system_configuration_slots(image: &MemoryPool) -> String {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut out = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut per = Vec::new();
        for slot in 0..2u64 {
            let bytes = PoolReader::read(image, device, DeviceOffsetInBytes(slot * spacing), slot_bytes);
            let text = match bytes.map(|b| singlefs_core::system_configuration::SystemConfiguration::parse_slot(&b)) {
                Some(Ok(sc)) => format!(
                    "槽{slot}:自证 世代{} 实例{} tail{} 见证{:?}",
                    sc.quantities.slot_generation,
                    sc.quantities.journal_instance.0,
                    sc.quantities.journal_tail,
                    sc.rollback_witness
                        .entries()
                        .iter()
                        .map(|e| (e.new_instance.0, e.rollback_target_instance.0, e.rollback_target_txg.0))
                        .collect::<Vec<_>>()
                ),
                Some(Err(_)) => format!("槽{slot}:坏"),
                None => format!("槽{slot}:读不出"),
            };
            per.push(text);
        }
        out.push(format!("盘{} [{}]", device.0, per.join(" | ")));
    }
    out.join("  ")
}

/// 取号式子现算：max(两盘四槽里自证过的实例代号, 根环里全部根的实例代号) + 1，连同两路各自的最大值。
fn acquisition_by_the_clause(image: &MemoryPool) -> String {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut sc_max = 0u32;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot in 0..2u64 {
            if let Some(bytes) = PoolReader::read(image, device, DeviceOffsetInBytes(slot * spacing), slot_bytes) {
                if let Ok(sc) = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
                    sc_max = sc_max.max(sc.quantities.journal_instance.0);
                }
            }
        }
    }
    let root_max = roots_of(image).iter().map(|(i, _)| i.0).max().unwrap_or(0);
    let journal_max = journal_records(image)
        .iter()
        .map(|r| r.2)
        .max()
        .unwrap_or(0);
    format!(
        "式子: 系统配置最大实例={sc_max} 根环最大实例={root_max} ⇒ 取号={}；（式子不读的）journal 环里最大实例={journal_max}",
        sc_max.max(root_max) + 1
    )
}

fn ring_roots(image: &MemoryPool) -> String {
    let sc = choose_system_configuration(image).expect("系统配置");
    let mut roots: Vec<_> = readable_roots_with_ring_slots(
        image,
        &sc.immutable.region_devices,
        &sc.immutable.sizes,
        &sc.immutable.filesystem_identifier,
    )
    .iter()
    .map(|(slot, root)| (root.checkpoint_txg.0, root.instance.0, slot.region, slot.slot))
    .collect();
    roots.sort();
    roots
        .iter()
        .map(|(t, i, r, s)| format!("({i},{t})@区{r}槽{s}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// (盘, 计数器, 实例, txg, 事务号, 反向链, 本条头算出的链值)。
fn journal_records(image: &MemoryPool) -> Vec<(u32, u64, u32, u64, u64, u32, u32)> {
    let fsid_low = u64::from_le_bytes(parameters().filesystem_identifier[..8].try_into().expect("8"));
    let ring_bytes = parameters().geometry.journal_ring_bytes;
    let mut out = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for counter in 1..=400u64 {
            let offset = singlefs_core::journal::record_offset(counter, ring_bytes);
            let Some(bytes) = PoolReader::read(image, device, offset, 4096) else { continue };
            let Ok(view) = singlefs_checker::check_journal_record(&bytes, fsid_low) else { continue };
            out.push((
                device.0,
                view.counter,
                view.instance,
                view.checkpoint_txg,
                view.transaction,
                view.back_chain,
                singlefs_checker::back_chain_of_record_header(&bytes),
            ));
        }
    }
    out
}

fn journal_dump(image: &MemoryPool) -> String {
    let records = journal_records(image);
    let mut lines = Vec::new();
    for device in [0u32, 1] {
        let on: Vec<_> = records.iter().filter(|r| r.0 == device).collect();
        let text = on
            .iter()
            .map(|r| {
                let previous = on.iter().find(|p| p.1 + 1 == r.1);
                let verdict = if r.1 == 1 {
                    if r.5 == 0 { "链ok(首条)".to_string() } else { "链✗(首条应0)".to_string() }
                } else {
                    match previous {
                        Some(p) if p.2 == r.2 => {
                            if r.5 == p.6 { "链ok".to_string() } else { format!("链✗(应{:#010x})", p.6) }
                        }
                        Some(_) => {
                            if r.5 == 0 { "链ok(跨实例首条)".to_string() } else { "链✗(跨实例应0)".to_string() }
                        }
                        None => "前一条不在".to_string(),
                    }
                };
                format!("c{}:实例{} txg{} tx{} 链{:#010x} {verdict}", r.1, r.2, r.3, r.4, r.5)
            })
            .collect::<Vec<_>>()
            .join(" ; ");
        lines.push(format!("    盘{device} journal: {text}"));
    }
    lines.join("\n")
}

fn checker_reds(image: &MemoryPool) -> String {
    let reds: Vec<String> = singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(message) => Some(format!("{invariant}: {message}")),
            _ => None,
        })
        .collect();
    if reds.is_empty() { "全绿".to_string() } else { reds.join("\n      ") }
}

fn dump(label: &str, image: &MemoryPool) {
    println!("  [{label}]");
    println!("    系统配置: {}", system_configuration_slots(image));
    println!("    根环: {}", ring_roots(image));
    println!("    {}", acquisition_by_the_clause(image));
    println!("{}", journal_dump(image));
    println!("    checker 红项: {}", checker_reds(image));
}

fn describe_operation(index: usize, op: &RetainedOperation, k: usize) -> String {
    let o = &op.operation;
    let offset = o.offset.0;
    let region = match o.kind {
        RecordedOperationKind::Barrier => "屏障".to_string(),
        _ if offset < 2 * 4096 => format!("系统配置槽{}", offset / 4096),
        _ if offset >= 1024 * 1024 && offset < 16 * 1024 * 1024 => {
            let relative = offset - 1024 * 1024;
            format!("根环区{}槽{}", relative / (3 * 1024 * 1024), (relative % (3 * 1024 * 1024)) / 4096)
        }
        _ if offset >= 16 * 1024 * 1024 && offset < 16 * 1024 * 1024 + 768 * 1024 * 1024 => {
            let counter = (offset - 16 * 1024 * 1024) / 4096 + 1;
            let instance = op
                .contents
                .as_ref()
                .map(|bytes| u32::from_le_bytes(bytes[16..20].try_into().expect("4")))
                .unwrap_or(0);
            format!("journal c{counter}(实例{instance})")
        }
        _ => format!("单元区 off={offset}"),
    };
    let landed = if index < k { "落" } else { "未落" };
    format!("#{index} 盘{} {:?} {region} [{landed}]", o.device.0, o.kind)
}

fn print_operations(ops: &[RetainedOperation], k: usize) {
    for (index, op) in ops.iter().enumerate() {
        println!("      {}", describe_operation(index, op, k));
    }
}

#[test]
fn seed540096_trace() {
    let seed: u64 = std::env::var("TRACE_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(540_096);
    let zero_share: u64 = 4;
    let max_steps: u64 = 8;
    let degrade_share: u64 = 30;
    let mut rng = Lcg(seed);
    let last_txg = 5 + rng.below(6);
    let mut image = image_through(last_txg);
    println!("TRACE seed={seed} last_txg={last_txg}");
    dump("起点", &image);
    let steps = 1 + rng.below(max_steps);
    for step in 0..steps {
        let action = rng.below(100);
        if action < 55 {
            let mut roots = roots_of(&image);
            let allow_zero = rng.below(zero_share.max(1)) == 0;
            if !allow_zero {
                roots.retain(|(i, _)| i.0 != 0);
            }
            if roots.is_empty() {
                println!("STEP {step}: 回退（没有候选，跳过）");
                continue;
            }
            let (instance, txg) = roots[rng.below(roots.len() as u64) as usize];
            let crash = rng.below(2) == 0;
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            match mount_rollback(
                &parameters(),
                &mut devices,
                RollbackTarget { instance, checkpoint_txg: txg },
                ShadowLedger::On,
            ) {
                Ok(mounted) => {
                    let ops = stream.retained_operations();
                    let k = if crash { rng.below(ops.len() as u64 + 1) as usize } else { ops.len() };
                    println!(
                        "STEP {step}: 回退挂载到 ({},{}) 取号={} 录制流 {} 条、落 {k} 条{}",
                        instance.0,
                        txg.0,
                        mounted.output.instance.0,
                        ops.len(),
                        if crash { "（崩）" } else { "（整次写完）" }
                    );
                    print_operations(&ops, k);
                    image.apply(&ops[..k]);
                }
                Err(MountError::RollbackTargetNotACandidate { .. }) => {
                    println!("STEP {step}: 回退到 ({},{}) 被拒：不在候选集", instance.0, txg.0);
                }
                Err(other) => println!("STEP {step}: 回退被拒 {other:?}"),
            }
        } else if action < 85 {
            let crash = rng.below(3) == 0;
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            match mount_writable(&parameters(), &mut devices) {
                Ok(mounted) => {
                    let ops = stream.retained_operations();
                    let k = if crash { rng.below(ops.len() as u64 + 1) as usize } else { ops.len() };
                    println!(
                        "STEP {step}: 可写挂载 所选根=({},{}) 取号={} 录制流 {} 条、落 {k} 条{}",
                        mounted.output.chosen_root.instance.0,
                        mounted.output.chosen_root.checkpoint_txg.0,
                        mounted.output.instance.0,
                        ops.len(),
                        if crash { "（崩）" } else { "（整次写完）" }
                    );
                    print_operations(&ops, k);
                    image.apply(&ops[..k]);
                }
                Err(e) => println!("STEP {step}: 可写挂载被拒 {e:?}"),
            }
        } else if action < 100 - degrade_share {
            let (i, t) = corrupt_the_oldest_root_slot(&mut image);
            println!("STEP {step}: 最旧根 ({i},{t}) 的根环槽翻坏一个字节");
        } else {
            match degrade_the_newest_system_configuration_slots(&mut image) {
                Some(flipped) => println!("STEP {step}: 两盘最新系统配置槽各翻坏一个字节（盘, 槽）={flipped:?}"),
                None => println!("STEP {step}: 两盘最新系统配置槽（不是两槽都自证过，没翻）"),
            }
        }
        dump(&format!("STEP {step} 之后"), &image);
    }
    // 最后一次整次写完的可写挂载（s23 里的 writable_mount_choosing）。
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => {
            let ops = stream.retained_operations();
            println!(
                "FINAL: 可写挂载 所选根=({},{}) 取号={} 录制流 {} 条、整次写完",
                mounted.output.chosen_root.instance.0,
                mounted.output.chosen_root.checkpoint_txg.0,
                mounted.output.instance.0,
                ops.len()
            );
            print_operations(&ops, ops.len());
            image.apply(&ops);
        }
        Err(e) => println!("FINAL: 可写挂载被拒 {e:?}"),
    }
    dump("FINAL 之后", &image);
}

// ---------------- 最小历史：不回退、一次崩掉的可写挂载、两盘最新系统配置槽坏、再一次可写挂载 ----------------

#[derive(Clone, Copy, Debug)]
enum Degrade {
    /// 两块盘各自最新的那个系统配置槽都翻坏（s23 的 badsc）。
    BothDevices,
    /// 对照：只翻盘 0 的最新槽。
    OnlyDevice0,
    /// 对照：不翻。
    None,
}

fn degrade(image: &mut MemoryPool, mode: Degrade) -> String {
    match mode {
        Degrade::BothDevices => format!("{:?}", degrade_the_newest_system_configuration_slots(image)),
        Degrade::OnlyDevice0 => match slot_generation_newest(image, DeviceIdentity(0)) {
            Some(offset) => {
                image.flip_byte(DeviceIdentity(0), DeviceOffsetInBytes(offset), 100);
                format!("[(0, {})]", offset / 4096)
            }
            None => "没翻".to_string(),
        },
        Degrade::None => "不翻".to_string(),
    }
}

fn red_names(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(_) => Some(invariant),
            _ => None,
        })
        .collect()
}

/// 起点（last_txg 7，实例 1）→ 可写挂载 M1 在录制流第 k 条之后崩 → 按 mode 翻系统配置槽 → 可写挂载 M2 整次写完 → checker。
/// 交回 (M1 取的号, 录制流条数, 翻之后按式子算的号, M2 取的号或拒的理由, M2 之前的红项, M2 之后的红项)。
fn minimal_history(base: &MemoryPool, k: Option<usize>, mode: Degrade, verbose: bool) -> (u32, usize, String, String, Vec<&'static str>, Vec<&'static str>) {
    let mut image = base.clone();
    if verbose {
        dump("起点", &image);
    }
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices).expect("M1 可写挂载");
    let ops = stream.retained_operations();
    let k = k.unwrap_or(ops.len()).min(ops.len());
    if verbose {
        println!(
            "M1: 可写挂载 所选根=({},{}) 取号={} 录制流 {} 条、落 {k} 条",
            mounted.output.chosen_root.instance.0,
            mounted.output.chosen_root.checkpoint_txg.0,
            mounted.output.instance.0,
            ops.len()
        );
        print_operations(&ops, k);
    }
    image.apply(&ops[..k]);
    if verbose {
        dump("M1 崩之后", &image);
    }
    let flipped = degrade(&mut image, mode);
    let by_clause = acquisition_by_the_clause(&image);
    let before = red_names(&image);
    if verbose {
        println!("翻系统配置槽（{mode:?}）: {flipped}");
        dump("翻之后、M2 之前", &image);
    }
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let second = match mount_writable(&parameters(), &mut devices) {
        Ok(m2) => {
            let ops2 = stream.retained_operations();
            if verbose {
                println!(
                    "M2: 可写挂载 所选根=({},{}) 取号={} 录制流 {} 条、整次写完",
                    m2.output.chosen_root.instance.0,
                    m2.output.chosen_root.checkpoint_txg.0,
                    m2.output.instance.0,
                    ops2.len()
                );
                print_operations(&ops2, ops2.len());
            }
            image.apply(&ops2);
            format!("{}", m2.output.instance.0)
        }
        Err(error) => format!("拒:{}", format!("{error:?}").chars().take(80).collect::<String>()),
    };
    if verbose {
        dump("M2 之后", &image);
    }
    let after = red_names(&image);
    (mounted.output.instance.0, ops.len(), by_clause, second, before, after)
}

#[test]
fn seed540096_minimal_sweep() {
    let base = image_through(7);
    let (_, length, _, _, _, _) = minimal_history(&base, None, Degrade::None, false);
    let mut first_red: Option<usize> = None;
    for mode in [Degrade::BothDevices, Degrade::OnlyDevice0, Degrade::None] {
        let mut red_ks = Vec::new();
        for k in 0..=length {
            let (m1, _, by_clause, m2, before, after) = minimal_history(&base, Some(k), mode, false);
            println!(
                "MIN mode={mode:?} k={k}/{length} M1取号={m1} 翻之后{by_clause} M2取号={m2} M2之前红={before:?} M2之后红={after:?}"
            );
            if after.contains(&"I-8.6") {
                red_ks.push(k);
                if first_red.is_none() {
                    if let Degrade::BothDevices = mode {
                        first_red = Some(k);
                    }
                }
            }
        }
        println!("MIN summary mode={mode:?} 录制流 {length} 条 M2之后 I-8.6 红的 k = {red_ks:?}");
    }
    if let Some(k) = first_red {
        println!("==== 最小历史逐步读数（BothDevices, k={k}）====");
        minimal_history(&base, Some(k), Degrade::BothDevices, true);
    }
}

/// 单点逐步读数：MIN_K（M1 录制流落几条）、MIN_MODE（both / dev0 / none）由环境变量给，打出每一步的读数与 checker 红项原文。
#[test]
fn seed540096_minimal_one() {
    let k: usize = std::env::var("MIN_K").ok().and_then(|v| v.parse().ok()).unwrap_or(23);
    let mode = match std::env::var("MIN_MODE").as_deref() {
        Ok("dev0") => Degrade::OnlyDevice0,
        Ok("none") => Degrade::None,
        _ => Degrade::BothDevices,
    };
    let base = image_through(7);
    println!("==== 单点逐步读数（{mode:?}, k={k}）====");
    let (m1, length, by_clause, m2, before, after) = minimal_history(&base, Some(k), mode, true);
    println!("ONE mode={mode:?} k={k}/{length} M1取号={m1} 翻之后{by_clause} M2取号={m2} M2之前红={before:?} M2之后红={after:?}");
}
