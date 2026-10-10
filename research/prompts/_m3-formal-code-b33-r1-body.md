# 第 32–33 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 B3 B4 F1 F2 R3 R4 B3 U11 U20 M35 M44 M45 M46 M47 G5 W7 W15 W18 -->

## 一、题面

第 32 批（`records/2026-10-01-里程碑三形式化接入.md#32`）与第 33 批（同一份记录 `#33`）合入后一轮代码三方，改动全文 `research/prompts/_m3-formal-code-b33-r1-diff.md`：

- 32-1：照第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第三节第 1 条——B3（`crates/singlefs-harness/src/model_comparison.rs` 胶水逐片单测 k = 0..134）、B4（`crates/singlefs-harness/src/model.rs` 模型回退结构单测，36 种片数组合）、F1（`crates/singlefs-checker/src/walk.rs` 树表非零而读不出的版本 I-5.4 报部分没判）、F2（`crates/singlefs-checker/src/image.rs` 与 `walk.rs` 判指针头部偏移 28 / 29 / 30 三段按第一版取值，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 同一判），坏镜像进 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，GPU 对拍加一条。
- 32-2：照第 31 批判决 `research/prompts/m3-formal-code-b31-r1-main-verification.md` 第三节第 1 条——`crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 812 统计用例每版跑池级 checker；R3 / B3 / R4 / d / U11 变异行；`crates/singlefs-core/src/allocator.rs` 第 1296 行一段注释与 `checker_known_bad_images.rs` 手拼根环表那段注释；`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 抬 F 慢档用例 I-5.5、I-4.7、I-4.6、I-4.4 四支豁免（调查 `/tmp/claude-1000/i55-reuse-investigation/report.md`；I-4.x 三支照删除线判定）。
- 33-a：C551 连续覆盖写那条用例（`repeated_overwrites_go_past_the_812_records_…`）末尾判全部不变量；`crates/singlefs-checker-tier/src/crash_injection.rs` 用例 `mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount` 期望改成 `Some(1 + 3)`（调查 `/tmp/claude-1000/red-one-investigation/report.md`：第 28 批 G5 让每个二次崩溃各记一次）。
- 33-b：`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs` 那条回退下界用例换造法（调查 `/tmp/claude-1000/red-two-investigation/report.md` 候选 A；根环线认），变异表旧第 38 行改名换替换文、照第 1377 行加一行。
- 33-c：`walk.rs` 罩上「树表读得出、0 条、根记录直接持有的分配记录树读不出」那一版（F1 同形），坏镜像一张。
- E156 装置 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`：第 5 次（W7 换 PC-钉门槛）与第 6 次（门槛照用户定乙并进挂载时读回全零的槽、U20）两次执行员的改动，登记 `research/prompts/e156-r5-prereg.md`、`research/prompts/e156-r6-prereg.md`。

条款：两份判决的第三节；`.claude/kb/invariants.md` I-2.4、I-5.4、I-5.5、I-4.4、I-4.6、I-4.7 几行；`.claude/kb/checks-owed.md` C551；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 1」（含「根槽这一次读坏」那一行、第 37 行算 F_生效 读根环、第 71 行点名的用例）；E156 第 5、6 次登记与末尾主 agent 认定；用户 2026-10-05 定乙原话 `records/2026-09-30-里程碑三开工第一轮.md` 第 628 行。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b33-r1/tree/`（33-c 合入后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单同目录 `crates-sha256.txt`，314 行）。树里另有删除线第 21、22 批与根环线第 10g 批，不在这一轮被判。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录，每份副本用自己的编译目录（几份副本共用一个编译目录、又保留了旧修改时间时，cargo 不重编、静默跑旧二进制，这一轮已踩过一次）。
- 合入验证全绿：32-1 `/tmp/claude-1000/merge-and-verify/2026-10-05-3691827/`、32-2 `…/2026-10-05-4013826/`、33-a 与 33-b `…/2026-10-06-4171628/`、33-c `…/2026-10-06-121758/`（harness 169 个测试二进制、fmt、clippy；32-1 另补 `--all-targets --all-features` clippy 退 0）；抬 F 慢档那条在主工作区单跑 ok。
- checker 档主 agent 跑过的（带 `SINGLEFS_HEAVY_TESTS=user-request`，独立编译目录）：33-a 的 ① 两条 ok、33-b 那份 11 条全 ok；四行变异（第 721 行、改名后的「步 4：回退候选集的 F 用目标根自己带的 F 而不是 F_生效」、照第 1377 行新加那行、「第33批 G5 二次崩溃那一截跳过挂载透明却不记没判」）经 `checker-tier-crates-mutation-replay.sh --item` 全部抓到、基线 4 行 ok。没跑的：GPU 对拍新加那条与 F2 的 GPU 变异、E156 装置内联单测 19 条与 M35 / M44–M47、32-2 的 U11 那一行、其余 checker 档。
- 已知、已记的：33-c 留一处更窄的同形——树表指针全零的根、根记录那一项非零而那棵树读不出，今天整版不进评估（`.claude/kb/invariants.md` I-9.28 / I-9.5 把这种根算成没有树表），33-c 报告 `/tmp/claude-1000/m3-formal-33-c/report.md`「要你定的」。

方案按 `crates/` 今天的实现来谈。

## 三、格

### P1　32-1 的 B3、B4
B3 那条逐片单测钉的是不是第 30 批判决要的「第 k 片都对」（135 这个上界从哪来、换一个上界会不会漏）；B4 的 36 种组合罩不罩得住回退结构的错法（只取回某几片、丢某几片、角色或分配代错一版），再换一个取样点还红不红。

### P2　F1、33-c 与 F2
F1 与 33-c 两支「部分没判」在合法历史上会不会多报（写行、暖机、回退、抬 F 之后、多实例），在坏盘上还有没有同形的版本整版不进按版本那一层（含二、末节那一处指针全零的根）；F2 判的三段与 core 读路径拒的是不是同一组取值、同一判法，GPU 一侧 `head_ok` 那一判与 CPU 是不是逐格同（GPU 跑不了的读码，写「推的」）。

### P3　32-2
812 统计用例每版跑 checker 之后罩不罩得住第 31 批判决点名的几种回收错法；R4、d 两行点名的已有用例是不是真在它们下面红（不是基线就红）；U11 那一行在带 core 断言的 core 上红在 I-3.1 那一句还是又红在别处；四支豁免各只认一种理由，有没有一支把本该判红的不适用一起放过。

### P4　33-a、33-b
C551 那条用例判全部不变量之后，C551 的推翻条件算不算满足；① 的新期望 `Some(1 + 3)` 与第 28 批 G5 那一格的条款是不是同一个口径；② 的新造法测到的还是不是「回退下界取系统配置里的 F」那一判（不是被别的拒因先拦下），旧第 38 行改名之后有没有别处按旧名找它（E171 坏实现表 `bad-implementations-117.tsv` 已知按名引用）。

### P5　E156 装置
第 6 次门槛读法（全零槽并进 min、三种解开时点）与 core `RootRingOccupant::AllZeroAtMount`（`crates/singlefs-core/src/allocator.rs` 第 989–996 行）是不是同一个谓词，装置自己算的会不会与 core 一起错；PC-钉门槛与 PC-全零钉两个对照分不分得出它们要分的东西；改调用点不改函数体那一处会不会让 M36、M42 互相掩护。

### P6　变异行
这一轮新加、替换、删除的变异行：锚点各恰好命中一次、点名的测试跑得到、合入后基线绿；有没有两行原文相同而点名的测试一起错。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | P1–P6 | 从两份判决、条款、登记与用户定案正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | P1–P6，重点 P2、P3、P5 | 找反例：一段合法历史让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试、GPU 对拍、装置内联用例与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b33-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b33-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
