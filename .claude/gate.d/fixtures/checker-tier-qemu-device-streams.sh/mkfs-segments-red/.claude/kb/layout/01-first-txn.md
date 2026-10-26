## 八、根槽写路径的段序列登记表（样本里的合成副本：只留门禁 checker-tier-qemu-device-streams 读的那几处登记）

产物的 `name=segments` 四行（`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`）；四样登记值都照 2026-10-02 录样本那一趟虚机输出（PROVENANCE.txt）。

| 根槽写路径 | 录制流的段序列 | 出处 | 崩溃点重放枚举 |
|---|---|---|---|
| mkfs 种根 | `2+12+1+1+1+4`，27 次操作、4117 个崩溃状态，种类 `[zero_fill×2,barrier×2]\|[zero_fill×8,unit_write×4,barrier×2]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier×2]` | 真仓现行那一行的四样（钉值在 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs` 的 `recorded_stream_matches_the_registered_mkfs_segment_sequence`；PROVENANCE.txt） | 不在 |
| 第一次可写挂载取号 | `2`，2 次操作、4 个崩溃状态，种类 `[system_configuration_slot×2]` | 产物 `name=segments path=instance_acquisition`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |
| 空发布（暖机，第一版 2 次） | `2+1+2+2+1+2`，20 次操作、15 个崩溃状态，种类 `[journal_record×2,barrier×4]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 产物 `name=segments path=warm_up`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |
| 普通发布（新池新建文件） | `24+2+1+2`，35 次操作、16777223 个崩溃状态，种类 `[unit_write×24,barrier×2]\|[journal_record×2,barrier×2]\|[root_record_fua]\|[system_configuration_slot×2,barrier×2]` | 产物 `name=segments path=transaction`（照 2026-10-02 录样本那一趟虚机输出，PROVENANCE.txt） | 在 |

⚠️ 新池新建文件那条流是整条流 `2+2+1+2+2+1+2+24+2+1+2`、16777240 个状态（种类 `[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]`；57 次操作）。
