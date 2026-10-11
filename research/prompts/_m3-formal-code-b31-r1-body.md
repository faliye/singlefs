# 第 31 批代码轮第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 U10 U11 U12 U13 U14 U15 U16 V32 V33 V34 V35 V36 A1 A2 -->

## 一、题面

第 31 批（`records/2026-10-01-里程碑三形式化接入.md#31`）照用户 2026-10-05 的定案（弹窗选项原文「再加 core 断言」：E171 装置与 harness `build_pool` 都装根环表，core 在分配器没装根环表而记了一条根时断言）与 E171 跑前登记修订 6 改，三份补丁，改动全文 `research/prompts/_m3-formal-code-b31-r1-diff.md`：

- 31-1：`crates/singlefs-core/src/allocator.rs` 的 `record_root_written_by_this_process` 在 `root_ring` 为 None 时断言；`crates/singlefs-harness/tests/common/mod.rs` 的 `build_pool` 装根环表（还 C551）；新用例 `crates/singlefs-harness/tests/root_ring_turn_in_the_make_filesystem_session.rs`；直接建分配器而会撞断言的 harness 档 9 处、checker 档测试 2 处改成装根环表（逐处判定在 `/tmp/claude-1000/m3-formal-31-1/report.md` 第 5、6 节）。
- 31-3：E170 装置、旧 E171 装置、E156 内联单测、`crates/singlefs-checker-tier/src/on_device_modes.rs`（改名改签名，在内存里照 core 拼 mkfs 那张根环表、补记 txg 1–3）与两个调用方装根环表；`crates/singlefs-harness/tests/overwrite_in_one_instance.rs` 那条 812 记录用例改判法（「已分配 = 没被回收的记录跨度之和」，依据 D3（空间分配） 已定项 7「条目留到该落点被重新分配时覆盖」）。
- 31-2：E171 新装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 照登记修订 6：装根环表与 S1 启动核对、Q5 判决路四个调用点的注入入口（A1、A2）、内联用例 U10–U16 与变异 V32–V36。

条款：用户定案（上面原文）；`.claude/kb/checks-owed.md` C551；`.claude/kb/decisions/03-空间分配.md`「#### 已定项 7」；`.claude/kb/decisions/16-发布语义.md`「#### 已定项 1」（环里最旧有效根与可再分配谓词）；`.claude/kb/invariants.md` I-3.1、I-3.11、I-5.4 三行；`research/prompts/e171-r1-prereg.md` 修订 6 与「主 agent 认定：修订 6 交回」；调查报告 `/tmp/claude-1000/e171-s1-i31-investigation/report.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本 `/tmp/claude-1000/m3-formal-code-b31-r1/tree/`（主仓第 31 批合入后的 `crates/`、`Cargo.toml`、`Cargo.lock`，sha256 清单同目录 `crates-sha256.txt`，310 行）。代码引文读冻结副本，写 `crates/…` 相对路径与那棵树里的行号。要编就拷到自己的草稿目录。
- 合入验证 `/tmp/claude-1000/merge-and-verify/2026-10-05-2822486/`：harness 档 165 个测试二进制全绿；fmt、`--all-targets --all-features` clippy、单开 gpu / verdict-store 两种 clippy、musl release `new_pool_file_creation_on_device` 都过。checker 档一条没跑：checker 档经 `build_pool` 一族搭的流（31-1 报告第 9 节：29 份测试文件与 E161 装置）在 mkfs 同进程里发布超过 23 次的，落点 / 写流 / 崩溃态数可能变（读码，没逐份数）；E171 新装置的 U10–U16、各装置的变异行、QEMU 那道门禁都只编过。
- 已知、已记的：旧 E171 装置比对器那几份复跑（`scripts/experiments/replay.sh` 第 270、272–274 行）与 s4 坏实现表第 95 行结局会变（31-3 报告第 2 条）；E156 W7 那一格阳性对照今天在第一次记根时 panic，归 E156 第 5 次重跑；`overwrite_in_one_instance.rs` 第 605–612 行那条 812 用例的说明过时，归下一批。

方案按 `crates/` 今天的实现来谈。

## 三、格

### N1　core 断言
断言的条件是不是恰好「没装根环表而记了一条根」：有没有一条合法路径（挂载重建、回退、抬 F、取号回卷、多实例、checker 档与装置里的搭建）在装了根环表之前就记根，或者装的根环表与盘上的环不符（跳号断言之外）而不被拦；`expect` 消息与 `# Panics` 是不是写清了依赖哪条不变量。

### N2　`build_pool` 与 11 处装根环表
装上的根环表是不是 mkfs 刚写下的那一张（`root_ring_occupancy_after_make_filesystem` 的参数对不对）；两处手拼的（`accounting_tree_node_full.rs`、`checker_known_bad_images.rs`）与盘上的环对不对得上；装了之后哪些用例的读数会变而没变、或该变的没被发现（取样点只到 txg 23 以内的那些罩不罩得住回收）。C551 的新用例罩不罩得住「记根时不回收」以外的回收错法。

### N3　31-3 的装置与用例
`on_device_modes` 在内存里拼的根环表与 core `root_ring_occupancy_after_make_filesystem` 给的是不是逐项相同（第 0 代根的 txg、F、占用）；E170、旧 E171、E156 内联单测装了之后输出不变的推论站不站得住；`overwrite_in_one_instance` 那条用例的新判法（用例自己按 txg 模拟根环现算「已回收」）与 D3 已定项 7、D16 已定项 1 是不是同一个谓词，模拟写错会不会让用例与实现一起错。

### N4　31-2 的 E171 新装置
修订 6 的每一件（6.2 装根环表与 S1 启动核对、A1 / A2 注入、6.5 U10–U16 / V32–V36）做的是不是登记说的；注入入口在跑法里一律交空，有没有一条路让真跑时注入非空；U13、U14 的期望数（「c1 落了」的态数）前提站不站得住；`p6-on-dbef78ed.patch` 打出来的那份与仓里那份之差是不是恰好第 29 批那 4 行（`/tmp/claude-1000/m3-formal-31-2/report.md` 第 6.3 节核法）。

### N5　变异行
三份补丁新加与替换的变异行，锚点各恰好命中一次、点名的测试跑得到（`#[ignore]` 的带 `--include-ignored`），有没有一行在合入后基线就红、或与已有行原文相同而互相掩护。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | N1–N5 | 从用户定案、条款与登记正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | N1–N5，重点 N1、N3 | 找反例：一段合法历史让断言误 panic 或让新判法误红，或一个坏实现让它放过；在拷出的副本上造、量 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法、新用例算零轮。
- 重型测试不跑：checker 档的测试、装置内联用例、QEMU、实验复跑与 `crates/mutations.tsv` 整表都不跑，涉及它们的结论写「推的」；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b31-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b31-r1-<腿名>-model/`，带一份 `tools/rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
