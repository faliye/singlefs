## 条 1
文件：`.claude/kb/decisions/23-journal的角色与格式.md`
旧串：
```
- **切换重新读盘择根带出的三件**：读盘选出的根比内存里记的新时，不重发在飞 checkpoint，以读盘选出的根为准接着走；切换写的实例表行与 W 按读盘选出的根写；回退那次发布里，管理员选的 R_old 必须在「以读盘选出的根为最新根」算出的回退候选集里，不在就拒绝、盘上逐字节不变。挂载内实例切换的实现随里程碑「第二个事务」收口表第 16、40 行在后面的里程碑（C458（实例切换取内存里的根，不重新读盘））。
```
新串：
```
- **切换重新读盘择根带出的三件**：读盘选出的根比内存里记的新时，不重发在飞 checkpoint，以读盘选出的根为准接着走；切换写的实例表行与 W 按读盘选出的根写；回退那次发布里，管理员选的 R_old 必须在「以读盘选出的根为最新根」算出的回退候选集里，不在就拒绝、盘上逐字节不变。挂载内实例切换的实现随里程碑「第二个事务」收口表第 16、40 行在后面的里程碑（C458（实例切换取内存里的根，不重新读盘））。
- ⚠️ **零单元发布这一支不冻结**：发布末尾系统配置轮换之后那道屏障报错时，带单元 / 带记录的发布与轮换报错同一处置（冻结、原样重发）；零单元发布（`crates/singlefs-core/src/transaction.rs` 的 `publish_without_units`，末尾走 `persist_the_root_then_rotate_the_system_configuration` 那一路）这一支不冻结、整个挂载返回错误（失败经 `PublishError::BlockDevice`），下一次发布落在下一次挂载、换了实例代号；这是替条款没写的一支做的选择（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y1 那一格；`research/prompts/m2-impl-c577-barrier-implementer-report.md` 第 106–111 行「停下交主 agent 的设计问题」第 2 条），钉它的用例随代码三方第三轮那一批写。
```
依据：判决 research/prompts/m2-closeout-code-r2-main-verification.md 第一节 Y1 那一格；research/prompts/m2-impl-c577-barrier-implementer-report.md 第 106-111 行第 2 条

## 条 2
文件：`.claude/kb/invariants.md`
旧串：
```
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`（:46）登记这个编号，`IMPLEMENTED_INVARIANTS`（:54）由此变 47 条；判定 `judge_system_configuration_values_the_reader_accepts`（image.rs:454）在 `walk::check_pool_image`（walk.rs:5573 起）里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回（walk.rs:5578）；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`：盘 1 槽 0 的加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、红在盘 1 偏移 0 那一槽，其余不变量全报不适用） |
```
新串：
```
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`（:46）登记这个编号，`IMPLEMENTED_INVARIANTS`（:54）由此变 47 条；判定 `judge_system_configuration_values_the_reader_accepts`（image.rs:454）在 `walk::check_pool_image`（walk.rs:5573 起）里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回（walk.rs:5578）；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`：盘 1 槽 0 的加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、红在盘 1 偏移 0 那一槽，其余不变量全报不适用） ；今天 checker 只判前五项（格式版本、加密类型、槽距、根槽宽、环长），core 只判后三项（单元区起始槽号、journal 环起点、根环起点），两边补齐随代码三方第三轮那一批（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y4-a 那一格）。|
```
依据：判决 research/prompts/m2-closeout-code-r2-main-verification.md 第一节 Y4-a 那一格；D22（单元原子性怎么合成） 已定项 16 第 1 条 2026-09-27 已写的 core 那一半

## 条 3
文件：`.claude/kb/decisions-history/2026-09.md`
旧串：
```
## 历史版本

### 2026-09-27（其二十一）：D18（块里携带什么信息） 已定项 11、D22（单元原子性怎么合成） 已定项 16：挂着之后的入口再核本盘设备号与落后支；core 读根环起点与 journal 环起点字段
```
新串：
```
## 历史版本

### 2026-09-27（其二十二）：D23（journal 的角色与格式） 已定项 14、invariants.md I-7.13（系统配置池级字段在读者收的范围里）：零单元发布不冻结的选择；checker 与 core 判同一张表的现状写清

> 快查·改前：D23（journal 的角色与格式） 已定项 14 射程没提发布末尾那道屏障报错时零单元发布这一支怎么处置；`invariants.md` I-7.13（系统配置池级字段在读者收的范围里） 判据没写与 core 收同一张表，状态列没写今天两边各判哪几项。
>
> 快查·改后：D23（journal 的角色与格式） 已定项 14 射程补一句：零单元发布这一支不冻结、失败让整个挂载返回错误，是替没写的条款做的选择；`invariants.md` I-7.13（系统配置池级字段在读者收的范围里） 判据补与 core 收同一张表的八项字段，状态列写明今天 checker 判前五项、core 判后三项，两边补齐随代码三方第三轮那一批。

- 改前：见快查·改前。
- 改后：见快查·改后。
- 依据：代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y1、Y4-a 两格；C577（系统配置没见证到的最新根，乙罩不到） 实现员报告 `research/prompts/m2-impl-c577-barrier-implementer-report.md` 第 106–111 行「停下交主 agent 的设计问题」第 2 条。

### 2026-09-27（其二十一）：D18（块里携带什么信息） 已定项 11、D22（单元原子性怎么合成） 已定项 16：挂着之后的入口再核本盘设备号与落后支；core 读根环起点与 journal 环起点字段
```
依据：用户定案出处见 records/2026-09-24-里程碑二收尾调度.md「用户定第二轮改法：四入口再核两样、core 读 325/371（2026-09-27）」那一行
