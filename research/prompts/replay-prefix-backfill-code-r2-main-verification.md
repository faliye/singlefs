# replay-prefix-backfill-code-r2：判决（2026-11-19）

正文 `research/prompts/_replay-prefix-backfill-code-r2-body.md`，开工快照 `research/prompts/replay-prefix-backfill-code-r2-snapshot/`。这一题的第二轮；第一轮判决 `research/prompts/replay-prefix-backfill-code-r1-main-verification.md`，第三节第 3 到 6 条的改法经打补丁任务 20895、回读任务 20898 合进主工作区。这一轮另攻同一天另一路活落的交集长度组合证明。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方 | `research/prompts/replay-prefix-backfill-code-r2-opus-output.md` | 打中 3 条（README 义务数、`GPU_GC_PROOFS.md` 复跑命令与卡的说法、必红表三行改动没进快照），第 6 条三处小问题都成立，检查单「带 --item 的命令与项」不成立；列了 5 处还没有反例罩着的 ensures |
| 辩方 | `research/prompts/replay-prefix-backfill-code-r2-sonnet-output.md` | 第一轮 14 条打中 10 条站得住；一·4 写宽了、一·5 同样打中 G11、一·2 的清扫没做完（`verus-pins.tsv:47`、必红表第 365 行）；第三节第 3 到 5 条做到，第 6 条没做完（`GPU_GC_PROOFS.md:97`、`:104`–`:112`）；交集长度两处 `None` 支没有必红 |

主 agent 的核查（2026-11-19 现读）：

- 开工快照 `sha256sum -c research/prompts/replay-prefix-backfill-code-r2-snapshot/crates-sha256.txt`：十份都 OK，腿跑着的时候被判的文件没被改过。
- 两条腿交回时各自跑了 `scripts/agents/cite-check.py`：攻方「核了 49 处引文、3 处裸行号，对上 49 处」，辩方「核了 27 处引文、37 处裸行号，对上 25 处，没判 39 处」。主 agent 没复跑这条命令，判决引到的每一处原文由主 agent 在主工作区逐处现读：`crates/singlefs-replay-prefix/src/README.md:109`、`:111`，`src/GPU_GC_PROOFS.md:97`、`:104`–`:112`，`.claude/gate.d/verus-pins.tsv:47`–`:49`，`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:365`，`src/file_range_proofs.rs:169`–`:172`、`:219`–`:222`；`git diff HEAD -U0 -- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 的段头是 `-198`、`-324`、`-399,2`、`+402,91`。
- 判决引的读数都是主 agent 交过的任务：点名测试 36 条（任务 20675）、formal-verification 97 项（任务 20692）、71 行变异（任务 20696）。
- 两条腿都要主 agent 交一条不点名测试函数的 `cargo test` 核复跑命令会不会被拒，没交：同一形今天被范围声明钩子当场拒过，钩子自证（任务 21078）里有现成一格「task-server 提交 cargo test 点名目标、不点名测试函数：拒」。

## 二、判决

1. **说明文件的义务数没跟上（攻方 4-1）**：打中。`src/README.md:109` 写「文件区间模块的 5 项 Verus 义务」、`:111` 写「合计 9 项义务」，交集长度那一路在 `file_range_proofs.rs` 加了三条证明（`:345`、`:357`、`:369`）没回头改这两行。
2. **`GPU_GC_PROOFS.md` 的复跑命令（攻方 4-2、辩方第 2 条）**：打中两处。`:104`–`:112` 五条 `cargo test` 不点名测试函数、不带 `--exact`，照今天的写法交会被拒；`:97`「卡、内存与线程由服务分」与 `.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」那一条相反。`:112` 那条是真卡对拍，删掉的 `GPU_GC_SELFTEST.md` 里讲 `NOT_RUN_NO_GPU_ADAPTER` 怎么读的现状句没并过来。检查单「带 --item 的命令与项」因此不成立。
3. **G30 没有调用点的清扫没做完（辩方）**：打中。`.claude/gate.d/verus-pins.tsv:47` 写「十个产品核」、必红表第 365 行注释写「产品直接调用的容量、记账、派发与映射表核」，G30 `extended_gpu_chunk_input`（`gpu_gc_capacity_proofs.rs:183`）没有产品调用点；`verus-pins.tsv:48` 还写「211 verified」，crate 行（`:50`）是 214。
4. **必红表三行改动没进两份快照补丁（攻方 5-1）**：坐实。相对 HEAD 改了第 324、399、400 三行，只改条数那一段（门禁只报不判）；这一题的 `changes.patch` 与交集长度那一路的补丁里都没有，是别的会话早先落的。不改产品，这三行由这一轮判决点名收下。
5. **交集长度两处 `None` 支没有必红（辩方）**：打中。`file_range_proofs.rs:171` 与 `:221` 的「交集长度为零」两条 ensures 没有必红行罩着。攻方另列的没有反例罩着的 ensures（候选窗口引理两条、排序二分定理三条、覆盖写第一条引理两条、`witness_file_range_boundaries`）与辩方列的 `gpu_gc_scoring_proofs.rs:117`、`:214`–`:216`、`gpu_gc_proofs.rs:345`–`:346`，都是两条腿读码推的，没量；一并交实现员逐处补或写明为什么补不出。
6. **三处小问题**：都成立。必红表第 394–409 行挂在打分那段注释底下、没有自己的注释行（攻方另指出 394–401 也是）；`src/README.md` 文件表 `file_range_proofs.rs` 那一行没写精确交集长度；`verus-pins.tsv:49` 注释带日期。门禁 doc-text 的 comment-history 格的判法不读 `.tsv`，拦不到这一行：记一笔欠账（要拦的输入：门禁登记表注释里写日期与改动经过）。
7. **第一轮判决的复核（辩方）**：十条站得住。一·4「窗口常数 65535 不被钉住」写宽了：证明只要退 65534 个槽，实现员没加「窄一格」那一行是对的，两条腿都复核了那段推导。一·5「G04 一族是定义展开」同样打中 G11（三条 ensures 是 `record_contribution` 定义的展开），说明文件只标了「模型」。
8. **没打中的**：u32 界断言承重、落点对；覆盖写按段三条 ensures 合起来要 `overwrite_fits`；G17 的量词不是空真；75 行新必红抽查 30 行改坏的都是名字说的那条、落点对；九行变异与九条测试一一对应（读码推的；任务 20696 里这九行是照键复用的判定）。两条腿都没打中，记「没打中」。
9. **任务 20692 的读数**：93 行必红里现判 1 行、复用 92 行。复用按单元键，键就是这棵树的键，判定作数；正文引它时写「照键复用」，不写成这一趟现判。7 行必红的第五段条数与实测不同（`overwrite-exact-needs-fit`、G01 四行、`gpu-gc-G18-short-remainder-is-not-overrun`、`gpu-gc-G28-group-count-is-not-truncated`），门禁只报不判，并进改法第 4 条照实测改。

## 三、改法

都是说明文字、登记表注释与必红行，不动证明与产品代码；改完照代码轮收法再攻一轮。

1. `src/README.md:109`、`:111` 的义务数照 `file_range_proofs.rs` 现有被验函数改；文件表 `file_range_proofs.rs` 那一行补精确交集长度。量过（主 agent 现读）。
2. `src/GPU_GC_PROOFS.md:104`–`:112` 五条复跑命令改成点名测试全名加 `--exact`、范围写测试数；`:97` 改成任务不拿卡、真卡那一条交 GPU 服务代跑（接口待定）；`NOT_RUN_NO_GPU_ADAPTER` 怎么读的那句现状说明并进来。量过（主 agent 现读）。
3. `.claude/gate.d/verus-pins.tsv:47`、`:48`、`:49` 与必红表第 365 行注释照现状改：G30 没有调用点、义务数 214、去掉日期。登记表与必红表同一次改。
4. 必红表：`file_range_proofs.rs:171`、`:221` 的 `None` 支各补一行必红；第二节第 5 条两条腿列的其余 ensures 逐处补或写明补不出的理由；7 行条数照任务 20692 的实测改；第 394–409 行补自己的注释行。**零轮**：新必红行由 formal-verification 带 `--item` 证红，不改产品代码，不做 A/B。
5. G11 的说明文件补「定义展开」那一标。
6. 欠账：门禁 doc-text 的 comment-history 格不读 `.tsv`，登 `.claude/kb/checks-owed.md`（三语），交下一批写回。

## 交用户的

无。改法都是说明与登记的对齐，条款现成。

## 这一轮被判过的文件

- crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md：判了第一轮第 6 条改法合进来的样子，打中复跑命令不点名测试函数（`:104`–`:112`）、卡的说法与规则相反（`:97`）、G11 少标「定义展开」。
- crates/singlefs-replay-prefix/src/README.md：判了份数与文件表，打中文件区间模块义务数没跟上（`:109`、`:111`）、文件表缺精确交集长度。
- crates/singlefs-replay-prefix/src/file_range_proofs.rs：判了交集长度那一路的定义、两条分割引理的前提、两个产品入口的新 ensures 与 core 消费处对齐，没打中；打中 `:171`、`:221` 的 `None` 支没有必红。
- crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs：判了 G30 头注释改成没有调用点，成立；清扫没做完落在 `verus-pins.tsv` 与必红表，不在这一份。
- crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs：判了 G17 的量词与新 spec fn `allocation_stays_within_the_limit`（`:285`），没打中；`:345`–`:346` 没有必红是辩方读码推的，并进改法第 4 条。
- crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs：判了 `:47` u32 界断言承重与窗口常数推导，没打中；`:117`、`:214`–`:216` 没有必红是推的，并进改法第 4 条。
- crates/singlefs-replay-prefix/src/partial_write_proofs.rs：判了覆盖写第一条引理按段的 ensures 要 `overwrite_fits`，没打中。
- crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs：判了 `u64::MAX - 2`、`u64::MAX - 1` 两个边界值与参照独立，没打中；对应的变异行靠这两个边界值才红。
- crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs：判了 G27 折半定义不等式与 G29 逐字段数的参照与被测不共用，没打中。
- crates/singlefs-replay-prefix/verus-must-red-worlds.tsv：判了新加 75 行（抽 30 行）与交集长度 8 行，没打中落点；打中第 365 行注释、第 324、399、400 三行改动没进快照（收下）、7 行条数与实测不同、第 394–409 行注释挂错段、两处 `None` 支没有行。
- crates/mutations.tsv：判了这一题追加的 9 行（快照 `mutations-append.tsv`），一一对应九条测试，没打中。

## 回看决策

不涉及决策：改法只对齐说明文件、登记表注释与必红行，不动证明、产品代码与任何决策分项。
