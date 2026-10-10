#!/usr/bin/env python3
"""给仓副本（不是入库代码）加攻方开关：checker 的 walk.rs 与 core 的 inode_tree.rs 各几处定点替换，每处旧串必须恰好命中一次。
用法：patch_copy.py <仓副本根>。开关见替换进去的注释（OPUS_R3S2_*）。"""
import sys
from pathlib import Path

root = Path(sys.argv[1])


def patch(rel, pairs, tail=''):
    p = root / rel
    s = p.read_text()
    for old, new in pairs:
        n = s.count(old)
        if n != 1:
            sys.exit(f'{rel}: 旧串命中 {n} 次：{old[:60]!r}')
        s = s.replace(old, new)
    tmp = p.with_suffix(p.suffix + '.opus-r3s2-tmp')
    tmp.write_text(s + tail)
    tmp.replace(p)


patch('crates/singlefs-checker/src/walk.rs', [
    ('''                .judge("I-9.4", left.largest_inode < right.container, || {''',
     '''                .judge("I-9.4", left.largest_inode < right.container || opus_r3s2_mode("OPUS_R3S2_DROP_I94_THIRD"), || {'''),
    ('''    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {
        let Some(record) = records''',
     '''    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
        .filter(|tombstone| !opus_r3s2_mode("OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL") || tombstone.covers_the_whole_object())
    {
        let Some(record) = records'''),
    ('''            judgements.judge(invariant, tombstone.covers_the_whole_object(), || {''',
     '''            let allowed_shape = tombstone.covers_the_whole_object()
                || (opus_r3s2_mode("OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL")
                    && tombstone.range_start < tombstone.range_end
                    && tombstone.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END);
            judgements.judge(invariant, allowed_shape, || {'''),
    ('''    let mut objects_on_the_lineage: BTreeSet<(u64, u64)> = BTreeSet::new();
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
    {''',
     '''    let mut objects_on_the_lineage: BTreeSet<(u64, u64)> = BTreeSet::new();
    for tombstone in containers
        .iter()
        .flat_map(|container| container.records.iter())
        .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
        .filter(|tombstone| !opus_r3s2_mode("OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL") || tombstone.covers_the_whole_object())
    {'''),
], tail='''

/// 攻方副本（m3-leaf-death-r3 opus-s2）加的开关：环境变量设成 1 时打开那一种改法（只在草稿目录的仓副本里有，不是入库代码）。
/// `OPUS_R3S2_DROP_I94_THIRD`：删 I-9.4 第三句（N1）；`OPUS_R3S2_RANGE_TOMBSTONES_MINIMAL`：区间墓碑的最小改法——I-9.19 与 I-9.20 不重号只判整对象墓碑，
/// I-9.20 的形状一句放宽成「整对象或 [起, 2⁶⁴ − 1]、起 < 止」。
fn opus_r3s2_mode(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| value == "1")
}
''')

patch('crates/singlefs-core/src/inode_tree.rs', [
    ('''    pub fn separator_key(&self) -> u64 {
        self.identity.container
    }''',
     '''    pub fn separator_key(&self) -> u64 {
        // 攻方副本（m3-leaf-death-r3 opus-s2）：OPUS_R3S2_N1_SEPARATOR=1 时分隔 key = 这一版的最小 key（N1 新形）。
        if std::env::var_os("OPUS_R3S2_N1_SEPARATOR").is_some_and(|value| value == "1") {
            return self.records.first().map_or(self.identity.container, |record| record.inode);
        }
        self.identity.container
    }'''),
    ('''                                container: inode.0,
                                container_birth: checkpoint_txg,''',
     '''                                // 攻方副本：OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET=k 时右半容器号给成新记录号 − k（C116 那一族的写错）。
                                container: inode.0.saturating_sub(
                                    std::env::var("OPUS_R3S2_RIGHT_HALF_NUMBER_OFFSET").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0),
                                ),
                                container_birth: checkpoint_txg,'''),
])
print('patched')
