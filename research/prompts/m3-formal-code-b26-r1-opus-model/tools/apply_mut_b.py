#!/usr/bin/env python3
"""opus MUT-B（第三版，报告里用的那一版）：盘 0 每次写系统配置槽之后，把另一槽清成全 0、与新槽同在一段里。
用法：apply_mut_b.py <仓副本根>。原文恰好命中一次才改。只改副本。"""
import sys
path = sys.argv[1] + '/crates/singlefs-core/src/transaction.rs'
s = open(path, encoding='utf-8').read()
old = '''            &slot_bytes,
            WriteDurability::Plain,
        )?;
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);
        Ok(())'''
new = '''            &slot_bytes,
            WriteDurability::Plain,
        )?;
        // opus MUT-B：盘 0 轮换写完新槽之后把另一槽（上一份）清成全 0，与新槽同在一段里：新槽撕坏、清零落了的崩溃态上这块盘一份自证过的都不剩。
        let other_slot = DeviceOffsetInBytes(((slot_generation + 1) % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing);
        if index == 0 {
            self.devices[index].1.write_at(other_slot, &vec![0u8; slot_bytes.len()], WriteDurability::Plain)?;
        }
        self.writes_by_structure_kind
            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);
        Ok(())'''
assert s.count(old) == 1, '锚点不是恰好一处'
open(path, 'w', encoding='utf-8').write(s.replace(old, new))
print('applied MUT-B')
