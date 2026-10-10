import json
F='crates/singlefs-core/src/mount.rs'
HT="-p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table"
LIB="-p singlefs-core --lib -- rollback_recognized_among_the_packed_units_named_by_a_publish_tests"
rows=[]
def r(name,old,new,cmd,test=''): rows.append([name,F,old,new,cmd,test])
r("X1 宽护那一截的跨度一律记 1","            .filter(|record| record.is_released)\n            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))\n            .collect()\n    };","            .filter(|record| record.is_released)\n            .map(|record| (record.device, record.slot, 1u64))\n            .collect()\n    };",HT)
r("X2 宽护那一截取现行账里仍分配的、不取已释放的","            .filter(|record| record.is_released)\n            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))\n            .collect()\n    };","            .filter(|record| !record.is_released)\n            .map(|record| (record.device, record.slot, u64::from(record.span_slots)))\n            .collect()\n    };",HT)
r("X4 读盘一层只看第一条记录点名的打包单元",".iter()\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n",".iter()\n        .take(1)\n        .flat_map(|record| record.named.iter())\n        .filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n",HT)
r("X5 读盘一层只看第一个打包单元",".filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n",".filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .take(1)\n        .map(\n",HT)
r("X6 读盘一层不筛类标签 3",".filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)\n        .map(\n",".map(\n",HT)
r("X7 认回退比的是 T_old 不是 rollback_txg","Some(lineage) if lineage.rollback_txg == checkpoint_txg_of_the_publish => {","Some(lineage) if lineage.rollback_target_txg == checkpoint_txg_of_the_publish => {",LIB)
r("X18 读得出而解不开的打包单元当读得出的非谱系片（旧写法的 ok() 吞掉）","                Err(_unparsable) => PackedUnitNamedByAPublishAsRead::ReadButUnparsable,","                Err(_unparsable) => PackedUnitNamedByAPublishAsRead::Parsed {\n                    record_type: 0,\n                    records: Vec::new(),\n                },",HT)
r("X19 谱系片里只看第一条记录","                        Some(_lineage_of_another_publish) => {}","                        Some(_lineage_of_another_publish) => break,",LIB)
r("X20 读盘一层把打包记录类型记成 0","                        record_type: packed.identity.record_type,","                        record_type: 0,",HT)
r("X21 读盘一层把记录丢空","                        records: packed.records,","                        records: Vec::new(),",HT)
r("X22 R_old 候选不看 txg 等于 T_old 之后再取第一条","            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)\n            .map(|rollback_target| {","            .filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)\n            .take(1)\n            .map(|rollback_target| {",HT)
r("X23 读不出的根槽那一支：R_old 不在环里当 R_old 账为空账（accounts 空时交空）","        Some(_) | None => all_released_in_the_current_account(),","        Some(_) => Vec::new(),\n        None => all_released_in_the_current_account(),",HT)
json.dump(rows,open('mut/extra-core.json','w'),ensure_ascii=False)
print(len(rows))
