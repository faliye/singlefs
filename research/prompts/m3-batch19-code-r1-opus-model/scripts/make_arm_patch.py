#!/usr/bin/env python3
"""给副本的 core 打上攻方原型的臂开关与扣住集钩子（只在副本上，不进仓）。用法：make_arm_patch.py <副本根>"""
import sys, pathlib
root = pathlib.Path(sys.argv[1]) / 'crates/singlefs-core/src'
def sub(path, old, new):
    p = root / path
    t = p.read_text(encoding='utf-8')
    assert t.count(old) == 1, (path, old[:60], t.count(old))
    p.write_text(t.replace(old, new), encoding='utf-8')
# allocator: 扣住集的读出
sub('allocator.rs', '    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。\n    pub fn release_reclaim_holds(&mut self) {',
    '    /// 攻方原型：每块盘上扣住位置着的槽（盘号, 槽号）。\n    pub fn b19_held_slots(&self) -> Vec<(u32, u64)> {\n        self.devices.iter().flat_map(|m| m.b19_held_slots().into_iter().map(move |s| (m.device.0, s))).collect()\n    }\n\n    /// 抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。\n    pub fn release_reclaim_holds(&mut self) {')
sub('allocator.rs', '    fn index(&self, slot: SlotNumber) -> usize {',
    '    /// 攻方原型：扣住位置着的槽号。\n    pub fn b19_held_slots(&self) -> Vec<u64> {\n        let start = self.unit_area_start.slot().0;\n        self.held_until_floor_takes_effect.iter().enumerate().filter(|(_, h)| **h).map(|(i, _)| start + i as u64).collect()\n    }\n\n    fn index(&self, slot: SlotNumber) -> usize {')
# mount: 臂开关
sub('mount.rs', 'fn establish_instance<Device: BlockDevice>(',
    '''/// 攻方原型的臂：0 合法（冻结副本原样）；1 甲（暖机之前放开，预演不动，= mutations.tsv 删除批第 19 批 甲）；2 N4（暖机之后不放开，= 删除批第 19 批 N4）；
/// 3 甲同步（暖机之前放开，预演在拷贝上同一处也放开：预演对真发那一判不分叉）；4 预演之前放开（取号之前就放开，预演与真发都看放开之后的）；
/// 5 只在暖机第一次之后放开（暖机多于一次时早于落满，预演不动）；6 同 5、预演在拷贝上同一处也放开。初值读环境变量 B19_ARM，原型可改。
/// 每个线程各自一个臂（原型在几个线程上同时跑不同的臂）；没设过的线程读环境变量 B19_ARM。
pub fn b19_set_arm(arm: u8) {
    B19_ARM.with(|a| a.set(arm));
}
pub fn b19_arm() -> u8 {
    let a = B19_ARM.with(|a| a.get());
    if a != 255 { return a; }
    let v = std::env::var("B19_ARM").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    B19_ARM.with(|a| a.set(v));
    v
}
thread_local! {
    static B19_ARM: std::cell::Cell<u8> = const { std::cell::Cell::new(255) };
    /// 攻方原型：最近一次 establish_instance 进门时分配器上扣住位置着的槽。
    pub static B19_HELD_AT_ESTABLISH: std::cell::RefCell<Vec<(u32, u64)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// 攻方原型：最近一次 establish_instance 推的暖机次数。
    pub static B19_WARM_UPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn establish_instance<Device: BlockDevice>(''')
sub('mount.rs', '''    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来''', '''    B19_HELD_AT_ESTABLISH.with(|h| *h.borrow_mut() = allocator.b19_held_slots());
    if b19_arm() == 4 {
        allocator.release_reclaim_holds();
    }
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来''')
sub('mount.rs', '''    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
    let mut current = row_publish;''', '''    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
    if b19_arm() == 3 {
        copy.release_reclaim_holds();
    }
    let mut current = row_publish;''')
sub('mount.rs', '''    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();''', '''    if matches!(b19_arm(), 1 | 3) {
        allocator.release_reclaim_holds();
    }
    B19_WARM_UPS.with(|w| w.set(warm_up_publishes_planned.len()));
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();''')
sub('mount.rs', '''        warm_up_publishes.push(next.clone());
        current = next;
    }''', '''        warm_up_publishes.push(next.clone());
        current = next;
        if matches!(b19_arm(), 5 | 6) && warm_up_publishes.len() == 1 {
            allocator.release_reclaim_holds();
        }
    }''')
sub('mount.rs', '''        taken_by_every_publish.push(placements_taken_by(&next));
        current = next;
    }
    Ok(taken_by_every_publish)''', '''        taken_by_every_publish.push(placements_taken_by(&next));
        current = next;
        if b19_arm() == 6 && warm_up_position == 0 {
            copy.release_reclaim_holds();
        }
    }
    Ok(taken_by_every_publish)''')
sub('mount.rs', '''    // 扣住的那一截不交给任何会话。
    allocator.release_reclaim_holds();''', '''    // 扣住的那一截不交给任何会话。
    if b19_arm() != 2 {
        allocator.release_reclaim_holds();
    }''')
print('ok')
# 攻方腿自己的错实现，按环境变量 B19_MUT 选（不设 = 冻结副本原样）：
# q-unlink-mono：会话层 unlink 的时间取给的与这个线程上一次会话 unlink 用过的时间里较大的那个（只对 unlink 只进不退）；
# c2-third：unlink 一批的单元数取 max(产品批大小, ⌈剩下单元数 ÷ 3⌉)（批数封顶 3 的捷径）。
sub('unlink.rs', '''#[must_use]
pub fn next_unlink_step(data_units_remaining: u64, batch_size: UnlinkBatchSize) -> UnlinkStep {
    let data_units_per_batch = batch_size.data_units_per_batch();''', '''/// 攻方原型：错实现开关（环境变量 B19_MUT，读一次）。
pub fn b19_mut_is(name: &str) -> bool {
    static MUT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MUT.get_or_init(|| std::env::var("B19_MUT").unwrap_or_default()) == name
}

#[must_use]
pub fn next_unlink_step(data_units_remaining: u64, batch_size: UnlinkBatchSize) -> UnlinkStep {
    let data_units_per_batch = match batch_size {
        UnlinkBatchSize::OneExtentLowerLeaf if b19_mut_is("c2-third") => {
            batch_size.data_units_per_batch().max(data_units_remaining.div_ceil(3))
        }
        _ => batch_size.data_units_per_batch(),
    };''')
sub('mounted_session.rs', '''        UserChange::Unlink {
            inode,
            write_time_seconds,
        } => publish_unlink_intent(
            &mut writer,
            allocator,
            previous,
            inode,
            write_time_seconds,
            instance,
        ),''', '''        UserChange::Unlink {
            inode,
            write_time_seconds,
        } => publish_unlink_intent(
            &mut writer,
            allocator,
            previous,
            inode,
            if crate::unlink::b19_mut_is("q-unlink-mono") {
                thread_local! { static LAST_UNLINK_TIME: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
                let clamped = LAST_UNLINK_TIME.with(|last| last.get().max(write_time_seconds));
                LAST_UNLINK_TIME.with(|last| last.set(clamped));
                clamped
            } else {
                write_time_seconds
            },
            instance,
        ),''')
print('mutants ok')
