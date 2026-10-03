#!/usr/bin/env python3
"""m3-ring-r3 辩方腿：C331 网格移植到冻结副本的补充改动（与 mkcand.py、mkmixed.py 配套；草稿目录里实际跑过的就是这几段）。
顺序：g3（冻结副本 + 第一轮 common-mod.rs.diff + 第二轮 common-mod-memory.rs.diff（第 1 个 hunk 的 use 行手改）+ 网格用例）
→ mkcand.py（造 g3-eio 甲-读错、g3-dev 甲-读错-每盘）→ mkmixed.py（混合读回开关）→ 本文件（网格里新增的枚举分支、swap 开关、g3-r2、g3-base）。
"""
import shutil, os
ROOT = '/tmp/claude-1000/m3-ring-r3-sonnet'

def sub(path, old, new):
    text = open(path).read()
    assert text.count(old) == 1, (path, old[:60], text.count(old))
    open(path, 'w').write(text.replace(old, new))

# 1 网格用例里报见证读判定的 match 要认新成员（冻结副本里甲已经合入，多了这个枚举成员）
for d in ('g3', 'g3-eio', 'g3-dev'):
    sub(f'{ROOT}/{d}/crates/singlefs-harness/tests/c331_candidates_grid.rs',
        '        WitnessedCounterComparison::Undecidable => "undecidable".to_string(),\n',
        '        WitnessedCounterComparison::Undecidable => "undecidable".to_string(),\n'
        '        WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => "some_slot_unread_or_unverified".to_string(),\n')

# 2 swap 开关：R3_MIXED_SLOT_READ_BACK=swap 时盘 0 的系统配置槽读回全 0、盘 1 的报读错
sub(f'{ROOT}/g3-dev/crates/singlefs-harness/tests/common/mod.rs',
    '                if range.device.0 == 0 {\n                    UnreadableRangeReadBack::DeviceError\n                } else {\n                    UnreadableRangeReadBack::Zeros\n                }',
    '                let swapped = std::env::var("R3_MIXED_SLOT_READ_BACK").is_ok_and(|v| v == "swap");\n'
    '                if (range.device.0 == 0) != swapped {\n                    UnreadableRangeReadBack::DeviceError\n                } else {\n                    UnreadableRangeReadBack::Zeros\n                }')

# 3 g3-r2：判不出 = 有槽设备报读错 且 没有一块盘两槽都自证过（零轮收严，辩方腿自己提的，只在这个网格上量过）
shutil.rmtree(f'{ROOT}/g3-r2', ignore_errors=True)
shutil.copytree(f'{ROOT}/g3-dev', f'{ROOT}/g3-r2', ignore=shutil.ignore_patterns('target'))
d = f'{ROOT}/g3-r2/crates/singlefs-core/src'
sub(d + '/mount.rs',
    'let some_system_configuration_slot_unread = !slots_and_errors.is_empty() && slots_and_errors.iter().all(|(_, errors)| *errors > 0);',
    'let some_system_configuration_slot_unread = slots_and_errors.iter().any(|(_, errors)| *errors > 0) && !slots_and_errors.iter().any(|(slots, errors)| slots.len() == 2 && *errors == 0);')
sub(d + '/transaction.rs',
    '''    let every_device_has_a_slot_with_a_device_read_error = !pool.devices.is_empty()
        && pool.devices.iter().all(|(identity, _)| {
            crate::recovery::system_configuration_slots_read_once_with_device_read_errors(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            )
            .1
                > 0
        });
''',
    '''    let per_device: Vec<(usize, u64)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let (slots, errors) = crate::recovery::system_configuration_slots_read_once_with_device_read_errors(
                &*pool.devices,
                *identity,
                spacing,
                &filesystem_identifier,
            );
            (slots.len(), errors)
        })
        .collect();
    let every_device_has_a_slot_with_a_device_read_error = per_device.iter().any(|(_, errors)| *errors > 0)
        && !per_device.iter().any(|(slots, errors)| *slots == 2 && *errors == 0);
''')

# 4 g3-base：把冻结副本里已经合入的甲拿掉，回到 2026-09-28 的「今天」（取号读盘只拒一份自证过的都没有的盘）
shutil.rmtree(f'{ROOT}/g3-base', ignore_errors=True)
shutil.copytree(f'{ROOT}/g3-dev', f'{ROOT}/g3-base', ignore=shutil.ignore_patterns('target'))
d = f'{ROOT}/g3-base/crates/singlefs-core/src'
sub(d + '/mount.rs',
    'let some_system_configuration_slot_unread = !slots_and_errors.is_empty() && slots_and_errors.iter().all(|(_, errors)| *errors > 0);',
    'let some_system_configuration_slot_unread = false;')
sub(d + '/transaction.rs',
    'if every_device_has_a_slot_with_a_device_read_error || slots_of_this_device.is_empty() {',
    'if slots_of_this_device.is_empty() {')
print('ok')
