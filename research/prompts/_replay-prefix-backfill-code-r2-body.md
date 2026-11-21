# replay-prefix-backfill-code-r2：重放前缀第一轮改法合入之后的复核（2026-11-18）

代码轮第二轮。第一轮判决 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md` 第三节第 3 到 6 条的改法已合进主工作区（打补丁任务 20895、回读任务 20898）；第 1、2、7 条（产品改调 `extended_gpu_chunk_input`、exec 核参数换新类型、`live_slot_kernels.rs:379` 的 `expect`）照判决并进产品 GPU 包那一题，这一轮不攻。另有一路活在同一天落了交集长度组合证明（`src/file_range_proofs.rs`、`src/README.md`、必红表，补丁原样在快照的 `changes-intersection-length.patch`，那一路只做过源码与文本核对、没走三方），一并攻。开工快照 `research/prompts/replay-prefix-backfill-code-r2-snapshot/`：`crates-sha256.txt` 十份（`src/GPU_GC_SELFTEST.md` 照判决删掉了、不在清单里）、`changes.patch` 是这一题第一轮改法合进来的那一段、`mutations-append.tsv` 是这一题追加的 9 行变异。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测，2026-11-18 现读）

- `crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:47`：`lower_bound_of_sorted_records` 体内加了 `assert(middle + 1 <= u32::MAX);`，配必红行 `gpu-score-binary-midpoint-needs-u32-bound`。
- 窗口常数「窄一格」那一行没加：实现员推 W = 65534 照样成立是对的证明（报告 `~/.local/share/singlefs/work/replay-prefix-backfill-fixes/result.md`「九条各做到哪」第 2 条）。
- `src/partial_write_proofs.rs:120` `lemma_overwrite_changes_exactly_the_requested_bytes`：`requires overwrite_fits(...)`（`:121`）留着，ensures 加了按段的三条 subrange 结论，配改坏定义的必红行（第 410、411 行，落点按实测改过）。
- `src/gpu_gc_proofs.rs:285` 新加 spec fn `allocation_stays_within_the_limit`，G17 的 ensures 用它做量词触发项。
- 两份测试 `tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs`、`tests/gpu_gc_capacity_matches_independent_reference.rs` 的 `EDGES` 加了 `u64::MAX - 2`、`u64::MAX - 1`；G27 参照改成折半的定义不等式（u128），G29 参照按产品 `region_record_kernels.rs` 文件头「缓冲布局」逐字段数；九条 `#[test]` 各挂一行变异。
- 必红表 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 非空行 480 行；这一题新加 75 行，交集长度那一路加 8 行、改 7 行。
- 说明文件：README 表补了两份证明文件；`GPU_GC_PROOFS.md` 整份重写；`GPU_GC_SELFTEST.md` 删掉。
- 交集长度那一路：`src/file_range_proofs.rs:157` spec fn `range_intersection_length`，`bytes_in_unit`（`:166`）与 `checked_bytes_in_indexed_unit`（`:212`）的 ensures 关联到它，三条新证明在 `:345`、`:357`、`:369`。
- `.claude/gate.d/verus-pins.tsv:50` 登记重放前缀 214 条义务。

## 这一轮攻什么

1. 第一轮第三节第 3 条：u32 界的断言是不是真承重（删掉 requires 里那条界，落点是不是这一处）；窗口常数没加「窄一格」那一行，那段推导站不站得住；覆盖写第一条引理新加的按段 ensures 是不是离了 `overwrite_fits` 就推不出；G17 的量词有没有触发上、是不是空真。
2. 第 4 条：新加的 75 行必红，每行改坏的是不是它名字说的那条 ensures、落点对不对；第一轮两条腿列的没有反例罩着的 ensures 还剩哪些。
3. 第 5 条：两份测试的新参照与被测还有没有共用的式子或字面值；九行变异是不是红在它点名的那条测试、那条断言上。
4. 第 6 条：说明文件的数（十份源文件、九个模块、测试数、义务数）与树对不对得上；`GPU_GC_SELFTEST.md` 里还成立的现状句并进 `GPU_GC_PROOFS.md` 没有。
5. 交集长度那一路：`range_intersection_length` 的定义与两条分割引理的前提；产品 `crates/singlefs-core/src/write_request_split.rs` 消费返回长度的那几处（`:385`、`:393`、`:397`）与新 ensures 对不对得上；必红表新加的 8 行与改过的 7 行。
6. 实现员与回扫报的三处小问题（只读核出来、没改）：必红表第 402–409 行没有自己的注释行、挂在上一段注释底下；README 文件表 `file_range_proofs.rs` 那一行没写精确交集长度；`.claude/gate.d/verus-pins.tsv` 第 49 行注释带日期。

第一轮已攻过而没打中的角度这一轮不重复：三份新证明文件的空真与四处 `as` 转换、覆盖写另三条引理的前提、改名一批的结构。

## 验证读数（主 agent 经任务服务跑出的）

- 重放前缀三个目标点名的 36 条测试（任务 20675）：「27 passed」「4 passed」「5 passed」，0 failed。
- formal-verification 97 项（任务 20692）：「formal-verification.sh：3 格里 3 格绿」；必红世界「93 行（--item 点名的 93 行）每行改坏之后 Verus 都报错，证明错落点与第六段 landing= 相同」；空真 3 处都在插的那一行报 assertion failed。
- 这一题 9 行变异在 71 行变异任务 20696 里：「抓到 71 条、没红 0 条、无效 0 条、锚点失效 0 行」，基线不绿 0 行。

### 检查单：走不到的理由二次查实

实现员报告原文（`~/.local/share/singlefs/work/replay-prefix-backfill-fixes/result.md`「走不到的理由」一节）：新加的都是 Verus 的 `assert(...)` 证明义务，普通构建擦掉，没有新加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员；逐处列了 `gpu_gc_scoring_proofs.rs:47`、`partial_write_proofs.rs:134`、`:135`、`:136`、`:139` 五处。腿在 `changes.patch` 与 `changes-intersection-length.patch` 的非测试段里自己搜一遍这几样，有一处不是证明义务就是打中。

### 检查单：类型与名字

新加的名字：`allocation_stays_within_the_limit`、`range_intersection_length`、`lemma_adjacent_unit_intersection_lengths_add_up`、`lemma_split_write_intersection_lengths_add_up`、`witness_intersection_length_partitions`。exec 核参数仍是裸整数，那一条照判决并进产品 GPU 包那一题，这一轮不判。腿判新加的名字说不说得出它是什么、测试里新加的常量与辅助函数有没有缩写。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：改的是证明、必红表、测试与说明文件，没有认旧版本的分支或旧布局号。腿在两份补丁里搜。

### 检查单：不新建判定库

本轮改动没有这一类：必红表是门禁 formal-verification 读的登记表，判定写进 prune-server 那一份库（任务 20692 日志「判定在 prune-server 那一份（verdict-library 族 singlefs-formal-unit-verdicts）」）。腿搜新开的目录、TSV、sqlite、RocksDB。

### 检查单：所有任务多机可用

两份测试只读编进二进制的常量与现造的输入，不读仓根之外的路径（实现员报告「拒算的测试」一节：「无」）。腿核新测试与必红行读不读本机独有状态。

### 检查单：测试优先多线程

两份测试是小用例，每条几毫秒（任务 20675「finished in 0.00s」「0.01s」），不走崩溃枚举。腿判有没有该切片并行而没并行的。

### 检查单：带 --item 的命令与项

实现员报告「要主 agent 跑的命令」里 formal-verification 点 97 项、变异点 9 行；主 agent 都交了（任务 20692、20696）。腿核新必红行与新变异行能不能被各自的 `--list-items` 列出、命令有没有漏项。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例（两份补丁里没有 `crash-case:` 登记行，也没有调 `enumerate_crash_states` 一族的新测试）。
