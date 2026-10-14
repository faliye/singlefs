## 八、根槽写路径的段序列登记表（样本里的合成副本：只留门禁 checker-tier-qemu-device-streams 读的「mkfs 种根」一行）

| 根槽写路径 | 录制流的段序列 | 出处 | 崩溃点重放枚举 |
|---|---|---|---|
| mkfs 种根 | `12+1+1+1+4`，23 次操作、4114 个崩溃状态，种类 `[zero_fill×8,unit_write×4,barrier×2]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[system_configuration_slot×4,barrier×2]` | 2026-10-02 录样本那一趟虚机输出的 path=mkfs 行（PROVENANCE.txt） | 不在 |
