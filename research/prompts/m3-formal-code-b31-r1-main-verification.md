# 第 31 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 R1 R2 R3 R4 B3 U10 U11 U12 U13 U14 U15 U16 V32 V33 V34 V35 V36 M35 -->

被判的文件（第 31 批三份补丁动到的全部 `crates/`，逐份见改动全文 `research/prompts/_m3-formal-code-b31-r1-diff.md`）：
- 31-1：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/common/mod.rs`、`crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs`，以及装根环表的 `crates/singlefs-harness/tests/accounting_tree_node_full.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/common_tree_split/mod.rs`、`crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs`、`crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs`、`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`、`crates/singlefs-harness/tests/unequal_devices.rs`、`crates/singlefs-harness/tests/write_accounting_per_structure.rs`、`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`、`crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs`；
- 31-3：`crates/singlefs-checker-tier/src/on_device_modes.rs`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs`、`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`、`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`；
- 31-2：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`；
- `crates/mutations.tsv` 三份追加与替换。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b31-r1-body.md`、`_m3-formal-code-b31-r1-diff.md`、`_m3-formal-code-b31-r1-background.md` | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b31-r1/tree/`（第 31 批合入后），清单同目录 310 行；kb 快照 `research/prompts/m3-formal-code-b31-r1-snapshot/kb-sha256.txt` | 腿交回后核：冻结副本 310 行全 OK；kb 快照里 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md` 对不上（E156 第 5 次执行员写实验页），两份腿报告引它 0 处，不碍判决 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b31-r1-sonnet-output.md`（sha256 0b6e9fe0…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh` 五段复跑 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b31-r1-opus-output.md`（sha256 fd851bdd…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 六段复跑，另补一趟 R4 |
| 引文 | 两份腿报告 | 主 agent 合跑 cite-check：92 / 92 |

## 二、逐格判决

### N1　core 断言

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 断言条件恰好是「没装根环表而记根」 | 一致：产品里建分配器的两处（mkfs 同进程、挂载重建）都装，抬 F 预演在装过表的拷贝上，回退、取号不另建 | 没打中：core 之外 23 + 4 处建法逐处对；harness 慢档 62 条只有 4 条红，撤掉 31-1 的对照副本上同样红、数逐字同 | **两腿同判，站住**（复跑：慢档 62 条、红的 4 条与报告同名） |
| 表与盘上的环不符而不被拦 | 一致（读码：core 只拦跳号） | 没打中：build_pool 前缀之后放开扫用户动作（覆盖写、抬 F、回退）64 个种子 6504 步，表与盘 0 处不同，两种断言 0 次误 panic | **站住**（复跑：9 件合计 6504 步，每件 `findings=0`，最高 txg 90–97、163–169 与报告同） |
| `expect` 消息与 `# Panics` | 一致 | 小打中：字段注释说没装表的分配器「只做不发布的事（分配、准入读数）」，而 `admission_reading_before_a_publish` 没装表就 panic | **坐实**（主 agent 现查 `crates/singlefs-core/src/allocator.rs:1296` 与 `crates/singlefs-core/src/admission.rs:1387`）。注释说宽了：没装表能走的只有 `admission_reading_before_a_publish_on_a_ring_of`。并进 32-2 改注释 |

### N2　`build_pool` 与 11 处装根环表

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 装表参数与 mkfs 同一张 | 一致：逐处对上 mkfs 与装表用的是同一个变量 | 没打中 | 站住 |
| 两处手拼的表 | 读数不变；`checker_known_bad_images` 注释「照可写挂载读到的样子」不精确 | 读数不变；那张表比盘上少几条根、门槛偏高，靠「那个分配器上没有可回收的已释放记录」才不出事，注释没写这个前提 | **两腿同判**：读数不变，注释不精确且漏写前提（读码）。并进 32-2 改注释（32-1 合入之后，32-1 也动这份文件） |
| 装表之后读数会变而没变 | 量到 `row_publish_admission` 回收事件 10 → 65，用例只钉 `> 812` 下界照绿；txg ≤ 25 的会话一次回收都没有，第一次回收在 txg 26 | 同：这一形会话第一次回收在 txg 26 | 站住；`row_publish_admission` 不是缺陷，不拿它当「不回收」的证据。checker 档 30 份引 `build_pool` 一族的文件哪几份写过 txg 25，两腿都没逐份数，归提交时 checker 档快档 |
| 新用例罩不罩得住别的回收错法 | 部分：门槛 ±1、提前（a b c）红；只提前回收一槽再被复用（d）新用例与 812 用例都绿，已有 `root_ring_turn_in_one_mount` 红、每 txg 跑 checker 的探针在 txg 4 红 | 打中：门槛 ±1、释放代少一、根环公式少一槽（R1 R2 R3 B3）新用例红；带单元的发布在取落点之前记根（R4）新用例、812 用例、随机历史那条都绿，只在发布中途的崩溃态上红（38 个态） | **坐实为第 31 批新用例的射程，不是 harness 档的盲区**。主 agent 在攻方副本上补跑 R4 下的 `crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs`：基线 3 条全过，R4 下 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it` 红在第 108 行（「txg 26 盖掉 txg 2 之后 mkfs 那片树表按谓词回收，同一次挂载里就回到空闲」：R4 把刚回收的槽当场取走，那一槽不空）；正推的 d 也被这条抓到。攻方的推翻条件（「已有一条用例在 R4 下红」）在 harness 档就满足了。改法：R4 与 d 各加一行变异点名这条已有用例（锁住，不新写用例），并进 32-2；攻方提的「每次发布取第一道屏障前的崩溃态跑 checker」那条新用例不采，R4 已有用例罩着 |

### N3　31-3 的装置与用例

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| `on_device_modes` 拼的表与 core 的 | 一致（复跑 `{:?}` 逐字同） | 没打中：8 种几何下逐字同，接着 77 次覆盖写每步表同 | **两腿同判，站住**（复跑：正推 `genesis_txg=0 roots=[1, 2]` 那条 ok；攻方原型 8 格三项都 `true`、`steps_with_different_ring_tables=0` 8 行，与腿留存输出逐行同） |
| E170、旧 E171、E156 内联单测「输出不变」 | 站得住（推的）：记到 txg ≤ 15 | 同：在它们实际的会话长度上成立 | 站住到提交时 checker 档与实验复跑；旧 E171 `replay.sh` 第 270、272–274 行与 s4 表第 95 行会变（31-3 报告第 4 节，两腿读码同） |
| 812 用例新判法与 D16 谓词、D3 已定项 7 | 同一个谓词；模拟与 core 的表 87 个 txg 相同；看不见 d | 不同来源、在被测会话上逐条同（201 + 6504 步）；打中：判法从实现取释放代与 `target_for_publish` 两样输入，R3、B3 下这条照绿 | **坐实**（复跑：R3、B3 两趟这条 rc=0，同档 31-1 新用例与随机历史那条红）。新判法在谓词上对，但不独立。改法：这条用例每一版都跑池级 checker、要求 0 违例——攻方原型甲量过 R3 红 I-3.9、B3 红 I-7.17、基线 201 版 0 违例，正推探针量过 d 在 txg 4 红；也就是 C551 推翻条件那一句「同一条用例判全部不变量」。零轮，并进 32-2 |

### N4　31-2 的 E171 新装置

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 修订 6 各件对登记 | 一致；S1 比的是两边 `{:?}` 文本的 sha256、登记写「文本相同」，U13 登记写「末行判决」、实际是倒数第二行 | — | 站住；两处偏离只在写法，结局等价（文本相同 ⇔ 摘要相同不计碰撞；用例按倒数第二行写对了），记进 E171 登记的主 agent 认定 |
| 注入入口真跑恒空 | 一致（grep 全部构造点） | 没打中 | 站住 |
| U13、U14 期望数 144 | 前提站得住（推的） | 没攻 | 站住到提交时 checker 档 |
| p6 之差恰好第 29 批 4 行 | 一致（独立复跑） | 没打中（现打） | **站住**（复跑：`q5-plus-p6.rs` 2c933e21…，与冻结副本那份 cc736772… `diff` 只差 4 行，两份补丁去掉行号头后逐字节同） |

### N5　变异行

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 锚点、点名、基线 | 16 行静态全过；harness / core 5 行逐行证红 | 16 行静态全过；第 2736 行副本上证红 | **站住**（复跑：静态 19 行 `problems=0`；2734、2735、2736、2737、570 五行都抓到） |
| 互相掩护 | 三组同改法不同测试，门禁判法允许 | 推的：V32（第 2743 行，点名 U11）在带断言的 core 上红在 core 断言，U11 那句 I-3.1 判据表里没有一行证它红；E156 M35（冻结副本第 803 行）同形 | V32 / U11 坐实为证红缺口（读码：点名 U11 的只有那一行）。改法：照第 2735 行的改法（记根时环转了也不回收，不碰断言）加一行点名 U11，零轮，并进 32-2（checker 档，证红归提交时整表）。E156 M35 已由 E156 第 5 次换掉（新 M35 改 core、点名 U14，主工作区 `crates/mutations.tsv`），不再处置 |

### 附带：与第 31 批无关的 4 条红

攻方在冻结副本上跑 harness 慢档，4 条红（`reuse_after_raising_the_floor` 1 条、`rollback_by_a_forward_publish` 3 条），撤掉 31-1 的对照副本上同样红、数逐字同；`crates/mutations.tsv` 有 12 行点名它们，整表复跑时基线就红。主 agent 复跑冻结副本同 4 条红；在今天的主工作区上的结局与归属见第四节末。

## 三、去向

1. **32-2（第 32 批实现员，32-1 合入之后派）**：
   - `overwrite_in_one_instance.rs` 812 用例：说明照装根环表之后改写（第 605–612 行），每一版跑池级 checker、要求 0 违例；证红：R3、B3 两种坏实现各一行变异点名它；
   - R4、d 两种坏实现各一行变异，点名已有的 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it`；
   - 一行变异照第 2735 行的改法点名 E171 新装置 U11（证红归提交时整表）；
   - 注释两处：`crates/singlefs-core/src/allocator.rs:1296`（没装表只能走 `admission_reading_before_a_publish_on_a_ring_of`）、`crates/singlefs-harness/tests/checker_known_bad_images.rs` 手拼根环表那一段（写明「只记最新那一条，够这次发布用；不等于可写挂载读到的环」与「这个分配器上没有可回收的已释放记录」那个前提）。
2. **E171 登记**：主 agent 认定追加 S1 比摘要、U13 判决行是倒数第二行两处写法偏离。
3. **下一轮代码轮**（第 32 批合入后）：攻 32-1、32-2。
4. **4 条无关的红**：照第四节末的结论发给归属那条线。

## 四、复跑核对

正推：模型 `tools/rerun.sh` 的 static、prepare、tests、mutants、rows 五段在会话草稿 `b31-sonnet-rerun/` 跑，各段退 0：变异 a–e 的结局与 `left/right` 数（第 22、23、2 次覆盖写）与报告同；`compared_txgs=87`；`genesis_txg=0 roots=[1, 2]`；p6 之差 4 行；静态 `CHECKED 19 rows, problems=0`；五行变异都 `抓到(红)`。

攻方：模型 `tools/rerun.sh` 的 copy、build、mutants、sweep、ignored 五段在会话草稿 `b31-opus-rerun/` 跑，退 0；on-device 那一段编好后内存包装拒了 `auto`（键是草稿路径、峰值表里没有），主 agent 改用 8G 直接跑那个二进制，输出与腿留存的 `outputs/on-device/run.out` 只差内存包装的一行提示。8 趟坏实现逐件退出码与报告的表同（R4 四件全过、崩溃态 `violated_states=38`，`I-2.1`、`I-4.8`、`I-7.4` 各 38、`I-3.10` 37；B3 `violated_states=42` 全是 I-7.17；基线 `violated_states=0`、`findings=0`）；放开扫 9 件 6504 步全 `findings=0`；慢档 62 条、4 条红。补跑：R4 下 `root_ring_turn_in_one_mount` 一条红在第 108 行，基线 3 条全过，撤回后 `singlefs-core` 与冻结副本 `diff` 无差。

核不动：checker 档（U10–U16、checker 档 11 行变异、E170 / 旧 E171 / E156 复跑、QEMU），两腿都没跑，归提交时。

无关的 4 条红在今天的主工作区（根环线第 10g 批合后）上照样红：`cargo test -p singlefs-harness --test reuse_after_raising_the_floor -- --ignored` 12 过 1 红，`--test rollback_by_a_forward_publish -- --ignored` 19 过 3 红，红的就是那 4 条。两个文件暂存区里有别的线的未提交改动，归属主 agent 判不出，已把复现命令与对照结论发给根环线与删除线认领。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | 812 用例每一版跑池级 checker | 攻方，正推同提 | 攻方原型与正推探针量过 | 32-2 |
| 2 | R4、d、R3、B3 与 U11 的变异行 | 攻方、正推、主 agent | R4 / d 主 agent 与正推量过点名那条红；R3 / B3 随改法 1；U11 推的 | 32-2 |
| 3 | 两处注释 | 攻方、正推 | 读码 | 32-2 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-05 不受影响：812 用例的新判法与可再分配谓词同一个，改法只加 checker 判全部不变量，不碰谓词 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-10-05 不受影响：「条目留到该落点被重新分配时覆盖」是 812 用例新判法的依据，两腿同判一致 |
