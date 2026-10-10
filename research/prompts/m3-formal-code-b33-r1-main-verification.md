# 第 32–33 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 B3 B4 F1 F2 R3 R4 U11 M35 M36 M42 M44 M45 M46 M47 G5 RV Z1 -->

被判的文件（第 32、33 批五份补丁与 E156 装置两次执行员改动动到的全部 `crates/`，逐份见改动全文 `research/prompts/_m3-formal-code-b33-r1-diff.md`）：
- 32-1：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`；
- 32-2：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`、`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs`；
- 33-a：`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`；
- 33-b：`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`；
- 33-c：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`；
- E156 第 5、6 次：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`；
- `crates/mutations.tsv` 各份追加、替换与按名删。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b33-r1-body.md`、`_m3-formal-code-b33-r1-diff.md`、`_m3-formal-code-b33-r1-background.md`（第一版 1.5 MB 挪到会话草稿 `b33-materials-v1/`，裁成 140 KB，裁法记在 `records/2026-10-01-里程碑三形式化接入.md`） | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b33-r1/tree/`（33-c 合入后），清单同目录 314 行；kb 快照 `research/prompts/m3-formal-code-b33-r1-snapshot/kb-sha256.txt` | 腿交回后核：冻结副本 314 行全 OK；kb 快照里 `.claude/kb/invariants.md`、`.claude/kb/checks-owed.md` 对不上（删除线第 22、23 批写回；I-9.29 插在 `invariants.md` 第 316 行，两腿引的不变量行都在第 315 行及之前），cite-check 在今天的文件上全对上，不碍判决 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b33-r1-sonnet-output.md`（sha256 c5063295…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b33-r1-opus-output.md`（sha256 2ca4d79a…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh x fix p3 p2 c551` 复跑（第四节） |
| 引文 | 两份腿报告 | 主 agent 合跑 cite-check：128 / 128 |

## 二、逐格判决

### P1　32-1 的 B3、B4

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| B3 逐片单测 | 一致：135 与实现扇出同钉（`crates/singlefs-harness/src/model_comparison.rs:1427`、`:1429`），8 个非等价胶水变体全红、`.min(134)` 是等价变体 | 没打中（读码） | 站住 |
| B4 回退结构单测 | 一致（取样点之内）；只在目标装 60–118 片时才错的变体存活（取样点 {1, 2, 3, 4, 5, 135}） | 没打中（读码，推的）：只按片数触发的错罩不到 | 站住；射程限度坐实为取样点不敏感（`.claude/rules/mutation-sampling.md` 第三类），补一个落在 60–118 之间的取样点（例 100），零轮，进第 34 批 |

### P2　F1、33-c 与 F2

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| F1、33-c 在合法历史上多报 | 一致：162 个非慢档二进制全过，探针只在刻意弄坏树表的用例上出；6 条慢档一次调用没跑完 | 没打中（一次抽样）：树表 0 条一族 12 段历史、层 0 全量 210 147 个崩溃态，部分没判 0 次 | **两次独立抽样都没打中**（复跑：攻方 p2 三段总数逐字同，例 `B33P total states=37413 … i54_violated=0 i54_partially_f1=0 i54_partially_33c=0`）。正推那 6 条慢档的复跑见第四节 |
| 由记录施加出来的版本与下一次挂载先施加的那一版 | 没提 | 打中（副本量过）：I-5.4 按版本那一层只走 `candidate_indexes`（`crates/singlefs-checker/src/walk.rs:5082`、调用 `:10487`），这两类版本整版不进；同一种「两条记录罩住同一个槽」放在最新根上红 I-5.4，放在这两类版本上 0 违例，后一类在可写挂载时被 core 拒成 `AllocationRecordOutsideThePoolGeometry` | **坐实**（主 agent 复跑：`B33X label=a3-after-mount-record-version-overlapping-records violated=[] i54=Holds`、`B33X label=b3-before-mount-next-version-overlapping-records violated=[] i54=Holds`、`B33X mount label=b3-… outcome=err error=Recovery(AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" })`、对照 `c3 … violated=["I-5.4"]` 都与报告同）。射程主 agent 判：在 I-5.4 里。依据是 `.claude/kb/invariants.md:198` 那一行自己写的动机「留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽」，b3 就是这件事本身；同一份清单第 199 行 I-5.5 写明「含由记录施加出来的那几版」，checker 也已认出这两类版本、给 I-3.10 与 I-5.5 用。改法 RV（攻方：两类版本进同一个按版本循环），攻方副本上量过 a1、a2 改报部分没判、a3、b3 只红 I-5.4、坏镜像 104 条与另 7 份 harness 89 条全过；零轮，进第 34 批，GPU 走读同一处同改；I-5.4 的射程句归写回批 |
| 树表指针全零的根 | 规则没说（读码坐实，套件 0 用例走到） | 打中，比正文已记的更宽（副本量过）：最新那条根（txg 5）树表指针整条写零、重封，checker 0 违例、I-5.4 成立，core 可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })` | **坐实**（复跑：`B33X label=y4-newest-txg5-tree-table-pointer-all-zero violated=[] i54=Holds`、`B33X mount label=y4-… outcome=err error=Recovery(UnitUnreadable { slot: SlotNumber(0) })` 同）。checker 把根记录里全零的树表指针读成「没有树表」（`.claude/kb/invariants.md:315` I-9.28 那一行「树表指针全零的根没有树表，不算没判到」），core 的根记录读者把它当指向槽 0 的指针。第一版没有一处写者写得出这种根（攻方 grep，推的）。两边口径不同，而 I-9.28 那一句是 kb 已写的条款，改哪一边是条款上的选择，交用户（第五节第 1 条） |
| F2 与 core 是不是同一组取值、同一判法 | 一致：同组取值同判法；checker 比 core 多判根记录里的四条指针（MAC 有先例） | 取值相同、判的指针不同（副本量过）：core 的根记录读者四条指针只判 extent 偏移（`crates/singlefs-core/src/root_record.rs:146`），算法类型写 1 的根 checker 红 I-2.4、core 照样择它、挂载做成 | **两腿同判：checker 一侧对，core 根记录读者比条款松**（复跑：`B33X label=f1-tree-table-pointer violated=["I-2.4", "I-8.10"] i54=Holds`、`B33X mount label=f1-… outcome=ok chosen_txg=4` 同）。`.claude/kb/invariants.md:133` I-2.4 写「读者遇到非 0 一律判该结构损坏」；core 自己记着这一笔（`crates/singlefs-core/src/pointer.rs:239`「只判 extent 偏移（C581）：今天只剩根记录（`root_record.rs`）走这一判」），journal 新根段里同样四条指针已判全（`crates/singlefs-core/src/journal.rs:324`）。改法：根记录读者四条指针换成与 journal 新根段同一判法，零轮，进第 34 批，还 C581 那一笔 |
| GPU `head_ok` 与 CPU | 一致（读码） | 没打中（读码，推的） | 站住到提交时 GPU 对拍 |

### P3　32-2

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 812 用例每版跑 checker | 一致：R3 红 I-3.9（txg 4）、B3 红 I-7.17（txg 21）、d 红，R4 不红 | 一致：门槛 ±1、R3、B3 两条 812 用例都红，d 只红每版跑 checker 的那一条，R4 照判决预期罩不住 | **两腿同判，站住**（攻方 p3 十一趟复跑红绿与报告同） |
| R4、d 点名的已有用例 | 一致：基线 3 过，变异下红在第 108 行 | 一致 | 站住 |
| U11 那一行 | 一致（推的）：同一处变异在 harness 档只红 I-3.1 | 一致：812 用例里红在 txg 26 的 I-3.1 | 站住；主 agent 另跑 E171 装置 U11 那条 `the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1`（checker 档，带前缀）`ok`；变异证红归提交时整表 |
| 四支豁免 | I-4.x 三支一致；I-5.5 那支认的是 checker 的缺省理由（`crates/singlefs-checker/src/image.rs:358`），分不开「没有对象」与「一次都没判」 | 没打中（读码） | I-4.x 三支站住；**I-5.5 那支坐实偏弱**（主 agent 现查 `image.rs:350`–`360`：没评估过、也没写专门理由的不变量一律落到这一句）。改法：checker 给 I-5.5 写一句专门的不适用理由，用例只认那一句，零轮，进第 34 批（删除线同一文件不动这一支，已约定） |
| 抬 F 慢档用例今天的结局 | 冲突：冻结副本上红在 I-9.18 | — | **坐实，不是这两批带来的**：主 agent 在主工作区现跑红在「I-9.18 在 E 之后的镜像上要成立」、left `NotApplicable("走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判")`；删除线第 22 批新加 I-9.17–I-9.24 没跟改这条用例，删除线认、进它第 24 批（同时给那几条写专门理由）。点名这条用例的 11 行变异在改好之前基线红，记删除线账上 |
| 第 31 批没点名的回收错法 | — | 补一格：Z1-d（复用时不删被新记录罩住的已回收记录，变异表第 105 行）两条 812 用例都绿，由已有的再挂用例抓 | 站住，不改（已有用例罩着；攻方 p3 复跑 `row105` 红在 `an_overwrite_that_goes_out_after_a_remount_goes_out_in_the_same_mount_as_well`） |

### P4　33-a、33-b

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| C551 推翻条件 | 一致：满足 | 一致（副本量过）：812 两条用例末尾与每一版上 I-3.1 都成立，不在不适用里 | **两腿同判，站住**；C551 那一行写成已还清，归写回批 |
| ① `Some(1 + 3)` | 一致（推的） | 没打中（读码）：与 G5 同一个理由键 | 站住（主 agent 已在主工作区跑这两条 `ok`） |
| ② 新造法 | 一致：harness 档探针基线绿，四行变异下红在下界那一判 | 没打中：下界只能从盘 0 的系统配置来，别的拒因是 `VersionWithoutFile`、分得开 | 站住（主 agent 已跑那份 11 条全 ok、四行变异全抓到） |
| 旧第 38 行改名 | 两份 E171 `bad-implementations-117.tsv` 第 33 行按旧名找，找不到走补位、不静默失效 | 同 | 站住；E171 以后从今天的树取快照重跑时这一行走补位，记 E171 欠账（正在跑的修订 6 读冻结树里的旧表，不受影响） |

### P5　E156 装置

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 门槛读法与 core 是不是同一个谓词 | 一致（这一族上）；重读解开那条路这一族不出现 | 没打中（读码，推的） | 站住；主 agent 跑装置内联单测（checker 档，带前缀）`19 passed` |
| 装置与 core 会不会一起错 | 不一起错：core 去钉住、装置钉住表不动，K10 红 34 点等于推出来的 L6；共用 `target_for_publish` 等三个函数 | — | 站住 |
| 两个对照分不分得出 | 各分得出；分不出「钉最旧还是钉最大 txg」「重读解开」「读坏的槽」 | — | 站住；分不出的三样记进 E156 实验页「它答不了的」，归写回批 |
| M36、M42 | 无互相掩护（读码） | 不互相掩护；M36 在前洞格上可能少一块判别力（推的） | 站住；M36 那一句记进实验页，没量 |

### P6　变异行

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 锚点、点名、基线 | 锚点恰好一次、点名的测试跑得到；同改法点名不同测试有四组（门禁允许）；harness 档基线绿，抬 F 慢档那 11 行基线红 | 静态核过；按名挑的 25 行点名的测试函数都在 | 站住；那 11 行基线红归删除线（P3 末行） |

## 三、去向

1. **第 34 批实现员**（零轮）：
   - RV：`crates/singlefs-checker/src/walk.rs` 的 `judge_allocation_records_disjoint` 多收由记录施加出来的那几版与下一次挂载先施加的那一版，进同一个按版本循环；GPU 走读同一处同改；坏镜像 a1、a2、a3、b3 四形进 `checker_known_bad_images.rs`；配变异。
   - C581 余下那一笔：`crates/singlefs-core/src/root_record.rs` 根记录里四条指针换成与 journal 新根段同一判法（`read_judging_the_encryption_compression_and_extent_offset_fields_from`），配用例与变异。
   - I-5.5 专门的不适用理由：`walk.rs` 给 I-5.5 写一句，`reuse_after_raising_the_floor.rs` 那一支只认它（删除线第 24 批不动这一支，谁先交回谁先合）。
   - B4 补一个落在 60–118 片之间的取样点。
2. **用户已定甲**（第五节第 1 条）：池级 checker 对自证过的根，树表指针全零不再当「没有树表」，照 core 跟过去判损坏；删除态扫描等同一读法的几处一并改；进第 34 批。
3. **写回批**：I-5.4 射程句写明含由记录施加出来的版本与下一次挂载先施加的那一版；C551 已还清；C581 随第 34 批合入改状态；E156 实验页「它答不了的」补 P5 那三样与 M36 一句。
4. **转删除线**：抬 F 慢档用例 I-9.17–I-9.24 不适用（它已认、进第 24 批）。
5. **E171 欠账**：旧第 38 行改名后，E171 从今天的树重跑时第 33 行走补位。

## 四、复跑核对

攻方：模型 `tools/rerun.sh x fix p3 p2 c551` 在会话草稿 `b33-opus-rerun/` 跑，退 0。输出按段合成几份（`x.log`、`fix.log`、`c551.log`），与腿留存的分文件按行比：`B33X mount` 行逐字同；`B33X label=` 行除三行 clean 基线被打在 `test …` 同一行上之外逐字同；p3 十一趟每一趟红的用例与报告同；p2 三段 `B33P total` 行逐字同；fix 段坏镜像用例复跑 105 条全过（腿那一趟 104 条，复跑多带了探针那几条）。

正推：模型 `tools/rerun.sh setup static base p1 p2 p3 p4 reuse sweep sweep-slow` 在会话草稿 `b33-sonnet-rerun/` 跑，退 0：`base-reuse` 那一段抬 F 慢档用例红（与 P3 同）；`sweep` 只跑到 `SWEEP done=40 total=163 nonzero_rc=1`、`sweep-slow` 只跑到 `SWEEP-IGNORED done=4 total=66`（3 条 0、1 条 124 超时），探针行 `B33F1` / `B33C` 一行都没出——复跑没跑全，P2 合法历史多报那一格照两腿各自那一趟（正推 162 个非慢档二进制、攻方 210 147 个崩溃态）判，不靠这一趟。

checker 档主 agent 跑过的（带 `SINGLEFS_HEAVY_TESTS=user-request`，独立编译目录，各核 `Compiling singlefs-checker-tier` 1 行）：E156 装置内联单测 `test result: ok. 19 passed`；E171 `the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1 ... ok`。抬 F 慢档用例在主工作区现跑 568 秒，红在 I-9.18（P3）。

核不动：GPU 对拍与 GPU 变异、E156 的 M35 / M44–M47 与 U11 那一行的整表证红、层 0，归提交时。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | 树表指针全零的根：checker 照 I-9.28 当「没有树表」、0 违例；core 当指向槽 0、可写挂载拒成「槽 0 读不出」。候选：甲 checker 判「自证过的根的树表指针非全零」、I-9.28 那一句改写；乙 core 的根记录读者把全零树表指针当自证不过、这条根不进候选；丙 照今天，两边各判各的 | 攻方打中，正推读码同 | 攻方副本量过 y1–y4 | **用户 2026-10-05 弹窗定甲**（选项原文「甲 checker 判它损坏」）：池级 checker 不再把根记录里全零的树表指针读成「没有树表」，照 core 一样跟过去、判出损坏；I-9.28 那一句改写归写回批；实现进第 34 批 |
| 2 | RV（I-5.4 罩两类由记录造出来的版本） | 攻方 | 攻方副本量过 | 第 34 批 |
| 3 | 根记录四条指针判全 | 两腿 | 攻方副本量过 core 一侧照择 | 第 34 批 |
| 4 | I-5.5 专门理由 | 正推 | 读码坐实 | 第 34 批 |
| 5 | B4 补取样点 | 正推 | 正推副本量过 60–118 片变体存活 | 第 34 批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-05 不受影响：② 新造法下回退判序照第 37 行「算 F_生效 读根环也走已知槽重读仍坏就拒」不变，RV 与根记录判全不碰回收门槛与回退候选 |
| D19（块指针的结构与宽度预算） 已定项 3 | 备料 | 2026-10-05 不受影响：根记录四条指针判全是把已定的头部判法铺到最后一处读者，字段表不变 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-10-05 不受影响：RV 只扩 I-5.4 判的版本集合，不改分配记录条目的编码 |
