# 全工程查复用键（survey-pruning-r1）：盘上的库、标记与日志（路子 disk）

日期：2026-10-15（`TZ=Asia/Tokyo date +%F`）。批次：records/2026-10-13-复用键与全仓改名.md#2026-10-15-批3。
只读：没起 cargo、门禁、测试二进制；没打开任何 RocksDB（不经库代码，用自写的只读解析器 `tools/sst.py` 直接读 SST 与 WAL 文件，不碰 LOCK、不写库目录）；没写源码摘要缓存。
草稿与中间产物都在 `~/.local/share/singlefs/work/survey-pruning-r1-disk/`：`tools/`（解析脚本）、`fullroot/libs/<库名>/blocks.tsv`（每个库逐块的块键成分、结论种类、版本前 16 位、节点名、块里状态数；整解每个 SST 块得来，§4 校验过条数）、`libs/<库名>/summary.json`（各表行数、`09` 清单、`11` 流名；这一份是只解块前缀的快扫，块数比整解少，只取表名与清单用）、`cross-full.txt`、`trans-full.txt`、`entry-table.txt`、`hits.txt`（近两周日志里的复用计数行）。
标注：每条结论标「量的」（指到文件:行号、库文件或命令输出）或「推的」。私有主机名一律写「第二台」。
时间：报告只写日期；运行目录名里带时刻的，写成 `<前缀>-*-<进程号>` 的通配写法（例 `runs/entry-*-1990665/`），照样能用 shell 展开找到。

## 0 结论

1. 盘上能直接看到的阻碍剪枝，比三份审计报告多出三类有量的证据（量的，§3）：
   - **块键带名字**：同一块内容（输入指纹、计划哈希、块起点、结论种类、版本五样全等）只因节点名不同就各存各判。旧库 `library` 里 4100 块（268435542 个状态）只差设点文件路径；`layer0-enumeration-library` 与 `library-by-conclusion-kind` 之间 9679 块（632578246 个状态）只差点名写法（`position_addressed::X::full` 对 `position_addressed_X`）。
   - **崩溃点重放把整条流当一个点**：`fixed_script::full` 一个节点、一个计划哈希、25611 块，放量里状态数相同（1678377204）的那条流按点拆成小节点（是不是同一批状态是推的）；两边块键、输入指纹、计划哈希的交集都是 0（量的），所以「两库合一」单做不够，块的单元要先对齐。
   - **判法版本换一次，同一块内容整库重判**：`library-by-conclusion-kind` 里 35455 块内容在不止一把版本下各判过，多判 60563 次、3935242934 个状态；7fa0adbf→f0b9512b 一次 30172 块 1975517365 个状态，其中那次闭包里 29 份改动文件有 24 份只改注释（3 份 wgsl 只改注释也换了版本）。
2. 近两周每一趟复用掉成 0 的归因见 §2（20 行）。没有一趟是「只作废受影响的函数」：全是整库级的键（判法版本、整 crate、整表、整份脚本、手写常量）或库目录换了。
3. 有三处盘上的东西让事后归因做不到（判不了，§6）：新布局崩溃库不再写 `09` 表（版本→文件清单），旧布局的 `09` 只有文件名没有摘要；crates 变异与形式化的判法版本目录不存版本的输入；整道全绿标记只存文件个数不存清单。
4. 改名与换目录在两周里丢掉或孤立的库：`layer0-enumeration-library`（620M）、`library`（271M，旧布局拒开）、`given-states-library.polluted-by-gpu-fault-tests-2026-10-11`（104M）、`.git/singlefs-layer0-progress`（100M）、5 条崩溃枚举用例的全绿标记（用例改名），crates 变异判法版本目录在日志里至少删了 7 次（重型分片 2 次、轻型 5 次），研究侧 mutation-tables-compile 的库在 /tmp 已经没了（量的，§1）。

## 1 库目录清单（验收一：每个库目录一行）

「最近一次被读」的取法：RocksDB 每打开一次新建一份 `LOG`（旧的改名 `LOG.old.<微秒>`），取 `LOG` 的生日（`stat -c %W`）；文件型库取最新一条记录的写入日期与最近一趟读它的日志。
不用 atime：`library/` 下 194 份文件 atime 全是同一秒（`stat -c '%x' * | awk '{print $1}' | sort | uniq -c` 输出 `194 2026-10-14`），同一秒读完整个目录（量的），像是整片拷贝或扫描，不像门禁按键读（推的）。
版本数：从库里每个结论块（`0A`）或旧判定块（`01`）的键尾取版本、去重（量的，`fullroot/libs/<库名>/blocks.tsv`；L7、L8、L11 的子库没整解）。

| 号 | 路径 | 形态 | 大小 | 版本数 | 最近一次被读 | 还有没有人读 |
|---|---|---|---|---|---|---|
| L1 | `~/.local/share/singlefs/crash-amplification/library` | RocksDB 旧布局（`01` 判定块 + `09` 版本清单，没有 `00` 布局行） | 271M | 7（`01` 表；`09` 存了 7 份文件清单，只有文件名） | 2026-10-08 | 没有：新布局拒开旧库（`verdict_store.rs:66` 文件头「打开时拒绝、不迁移」）；推的 |
| L2 | `…/library-by-conclusion-kind` | RocksDB 布局 2（`00`=2） | 1232M | GPU 7、CPU 1、观察者 4 | 2026-10-14 | 崩溃放量入口（`crash-amplification.sh:7`）在用 |
| L3 | `…/layer0-enumeration-library` | 布局 2 | 620M | GPU 1（f0b9512b）、CPU 124、观察者 93 | 2026-10-13 | 没有：改名后代码只认 `crash-state-enumeration-library`（`crash_library.rs:132`） |
| L4 | `…/crash-state-enumeration-library` | 布局 2 | 250M | GPU 6、CPU 50、观察者 45 | 2026-10-15 | 崩溃点重放 --full 与快档、库内单测在用 |
| L5 | `…/given-states-library` | 布局 2 | 2122M | GPU 8、CPU 70 | 2026-10-15 | 给定状态那一路在用 |
| L6 | `…/given-states-library.polluted-by-gpu-fault-tests-2026-10-11` | 布局 2 | 104M | GPU 1、CPU 13 | 2026-10-12 | 没有：手工改名挪开（目录名自述） |
| L7 | `…/given-states-library-of-harness-scenario-images-judged-afresh` | 73 份子库，一个用例一份（以用例函数名命名） | 1803M | 每份子库每趟新建（抽 4 份：`LOG.old` 0 份、目录生日 = 最近一趟） | 2026-10-15 | 每趟新建，不跨趟读（量的；`common_given_states_of_harness_scenario_images/mod.rs:39`–40） |
| L8 | `…/amplification-unit-tests-library` | 顶层一份 + 61 份按用例标签的子库（共 76 份 RocksDB） | 917M | 没逐份解 | 2026-10-15 | 放量流水线的单测在用（`crash_library.rs:136`） |
| L9 | `…/judge-tables-pairing-library` | 布局 2，只有 `11`/`12`/`13`（录下的清单、GPU 判表、给定状态表），没有判定 | 158M | 0（不存判定） | 2026-10-15 | 在用 |
| L10 | `…/facts-pairing-library` | 布局 2，只有 `07`/`11` | 3M | 0 | 2026-10-14 | 在用 |
| L11 | `…/mutation-replay/`（5 份镜像库：同名的 L4、L5、L7、L8、L9） | 24 份 RocksDB | 980M | 没逐份解 | 2026-10-14 | crates 变异复跑起 cargo 时把家目录换到这里（`crates-mutation-rows.py:268`） |
| L12 | `…/runs/`（213 份逐趟的 RocksDB：`library-for-peer` 102、`fetched/crash-amplification-library` 102、`library-for-peer-placement` 9） | 每趟一份传给第二台的副本、一份取回的副本 | 89G（三类各 42076900K、42140844K、8115628K，`du -sc`） | — | 2026-10-14 | 取回导入之后没人再读（推的：没找到读它们的入口） |
| L13 | `…/crash-point-replay-quick/` | 9 个逐趟目录，只有日志（51 份） | 564K | — | 2026-10-13 | 轻型快档的日志，不是库 |
| L14 | `…/crash-point-replay-fast-tier/` | 一个空的逐趟目录 | 12K | — | — | 空 |
| L15 | `~/.local/share/singlefs/crash-amplification-smoke/` | 2 份 RocksDB | 2M | 没解 | 2026-09-29 | 没有 |
| L16 | `.git/singlefs-crash-case-green.<用例>.<指纹>` | 一条用例一个文本标记，带全部输入文件的逐文件摘要清单 | 23 份、12 条用例 | 每条 1–3 份 | 2026-10-15 写了 1 份 | 在用；14 条登记用例里 7 条没有标记，5 条标记的用例名已不在登记表（量的，§3 X17） |
| L17 | `.git/singlefs-stage-green.<阶段或格>.<指纹>` | 一格一个文本标记，只存 `input_hash` 与 `input_file_count`，不存清单 | 25 份（形式化 6 格 19、LKMM 3、QEMU 2） | — | 2026-10-13 写了最近那批 | 在用；crates 变异、广谱变异、研究复跑、崩溃点重放整道都没有一份 |
| L18 | `.git/singlefs-mutation-rows/<判法版本>/` | 一条一个文件（`mutation.<行键>`、`baseline.<行键>`）+ `snapshots/`（整仓逐文件原文 sha256）+ `coverage/` + `blobs/`（源码原文 zlib） | 37ab6ce3 8.9M（496 条）、3a63d840 12M（891 条）、d0ef8d8c 15M（815 条） | 3（只留 3 个，`crates-mutation-rows.py:84`、`:712`） | 2026-10-14 | 在用；10-12 至 10-14 日志里至少 7 把不同的版本（23404d01、37ab6ce3、f8ab0022、19937413、604b3601、3a63d840、d0ef8d8c），7 次「删了更早的判法版本目录 1 个」（§2 R13、R14） |
| L19 | `.git/singlefs-mutation-row-timings/` | 2185 份，按行原文摘要，只记用时 | 9M | — | 2026-10-14 | 排先后用，不是判定 |
| L20 | `.git/singlefs-formal-verdicts/{proof,vacuity,must-red-worlds,pinned-red-examples-replay}` | `keys/<键>.json` + `items/<项名摘要>.json` | 36K、416K、1.9M、8.9M | 判法版本：proof 3 把、vacuity 1、must-red 1、pinned 1；另有 942 条 10-09 的旧键形没人读 | 2026-10-14 | 在用 |
| L21 | `.git/singlefs-layer0-progress/` | 旧进度文件（按输入指纹分目录、文件名带流名与哈希） | 100M | — | 2026-09-27 | 没有（改名后的代码不读这个目录名，推的） |
| L22 | `.git/singlefs-layer0-logs/`、`.git/singlefs-crash-point-replay-logs/` | 日志 | 19M、24K | — | 2026-10-13、2026-10-15 | 日志 |
| L23 | `.git/singlefs-source-digest-cache.tsv` | 原文 sha256 → 整份文件的词法摘要（1776 行） | 352K | — | 2026-10-15 | 全绿标记算整文件摘要用 |
| L24 | `~/.local/share/singlefs/broad-mutation/` | `conclusions.tsv`（跨版本）+ `versions/<这一版指纹>/`（2 份）+ `tree/`（按提交导出的树）+ `target*` | 17G（库本身 < 30M） | 2（19c45e02、e1ba2574），结论表只有 2 行 | 2026-10-01 | 近两周只有 10-01 那一趟 |
| L25 | `${TMPDIR:-/tmp}/singlefs-mutation-tables-compile-verdicts` | — | 不存在 | — | — | 丢了（`ls` 回 No such file） |
| L26 | `.git/singlefs-replay-verdicts` | — | 不存在 | — | — | 从没落盘 |
| L27 | `/tmp/singlefs-c790-slices-observer-*-<进程号>`（50 份） | 单测按进程号各建的 RocksDB | 7.4M（连同 14 份镜像文件，64 项合计） | — | 2026-10-14 | 单测残留 |
| L28 | `~/.local/share/singlefs/cross-run-cache/` | 空目录（目录项 12288 字节，原来放过很多项） | 0 | — | 2026-10-14 改过 | 编译目录与开跑时刻文件用，不是判定库 |
| L29 | `/tmp/claude-1000/` | 找过 `CURRENT`、`singlefs-*verdicts`、全绿标记、`conclusions.tsv`：0 份 | — | — | — | 没有库 |
| L30 | `~/.local/state/singlefs/` | `detached-runs/`（116 趟的命令与日志）、`reminders/` | 8M | — | 2026-10-15 | 日志，不是库 |

## 2 近两周每一趟复用掉成 0 的归因（验收二）

范围：2026-10-01 起、留在盘上的日志（`find … -newermt 2026-09-30`，排除仓副本与 RocksDB 自己的 WAL，剩 19697 份；按「复用 0」「现判 N」「resumed_slices=」「states_reused=」「tables_reused=」「沿用」「判法版本 变了」「复用抓到」这类写法 grep 出 13812 行，全文在 `hits.txt`）。
重型各道逐趟列；轻型（--quick、库内单测）只在它和重型共用一份库时列，标「轻型」。
「两版清单 diff」的取法：崩溃枚举用例拿相邻两份全绿标记的逐文件摘要清单比（`tools/markerdiff.py`）；崩溃库拿同一块内容在库里出现过的版本按写入序号排先后（`tools/trans.py`），判法版本的输入在新布局里没存（§6），改用两棵暂存树的 `git diff` 量闭包文件、再按去注释比（`tools/strip_cmp.py`）；形式化拿 `keys/*.json` 的 components 逐样比；广谱变异拿两版的 `global-key.txt` 与 `manifest.tsv` 比。

| 号 | 哪道、哪一趟 | 复用数（原样） | 键的哪一项变了（量的 / 推的） | 依据 |
|---|---|---|---|---|
| R1 | 崩溃放量，`runs/entry-*-2361385/unlink-after-the-multi-record-publish.log`（10-08，旧库 L1） | `pending_states=1448083638 states_reused=0`，`judge_version=93e1e4e309c26611`；同一趟 multi-record-publish 是 `c7c1e096a5fc69cf`、复用 16777259 | 判法版本 c7c1e096→93e1e4e3；L1 里同一块内容在这两把下各判一次的有 20907 块、1369571440 个状态（量的）。两把版本的 `09` 清单逐行相同（`diff` 0 行，量的）——清单只记文件名，变的是哪份文件的摘要盘上判不了 | `libs/library/09-*.txt`；`tools/trans.py library` |
| R2 | 崩溃放量，`runs/entry-*-600562/fixed-script-through-rollback.log`（10-11） | `pending_states=67502198 states_reused=0` | 库目录换了：L2 `library-by-conclusion-kind` 的目录生日就在这一趟（同一天、比入口目录名晚两分钟，`stat -c %W`，量的）；旧库 L1 是旧布局、新代码拒开（`verdict_store.rs:66`） | `stat`；`entry-table.txt` |
| R3 | 崩溃放量，`runs/entry-*-1990665/`（10-11） | rollback `pending_states=67502198 states_reused=0`，`judge_version=gpu_judge:7fa0adbf10708a36` | 判法版本 cedd6ed8→7fa0adbf。暂存树 663bb8b6→b5a35b44（树与这一趟的对应取自路子三的 `entry-table.tsv`，推的）之间闭包文件改了 11 份，去注释比 11 份都改了代码：`crash.rs`、`crash_amplification.rs`、`crash_facts.rs`、`crash_given_states.rs`、`crash_identity.rs`、`crash_judge_conclusions.rs`、`crash_judge_dispatch.rs`、`crash_library.rs`（新增）、`crash_verify_gpu.rs`、`verdict_store.rs`、`gpu_memory_ledger.rs`（量的）；其中流水线与存储 5 份（`crash_amplification.rs`、`crash_judge_dispatch.rs`、`crash_library.rs`、`verdict_store.rs`、`gpu_memory_ledger.rs`）不判对错 | `tools/strip_cmp.py 663bb8b6 b5a35b44 …` 输出 |
| R4 | 崩溃放量，`runs/entry-*-2782527/`（10-12） | rollback `pending_states=67502198 states_reused=0`，`gpu_judge:f0b9512b88d5bbfe` | 判法版本 7fa0adbf→f0b9512b。b5a35b44→f1985f9f 闭包文件改了 29 份：24 份去注释后逐字相同（其中 3 份 wgsl：`crash_judge_checker_common.wgsl`、`crash_judge_checker_scan.wgsl`、`crash_judge_recovery.wgsl`，wgsl 按原文进摘要，只改注释也换版本），5 份改了代码：`crash_amplification.rs`（流水线）、`singlefs-core/src/block_device.rs`、`recovery.rs`、`singlefs-harness/src/lib.rs`、`memory_pool.rs`（量的）。L2 里 7fa0adbf→f0b9512b 同一块内容重判 30172 块、1975517365 个状态（量的） | `tools/strip_cmp.py`；`tools/trans.py library-by-conclusion-kind` |
| R5 | 崩溃放量，`runs/entry-*-3243321/`（10-12） | position-addressed-trees `268435480 / 0`、tree-split-streams `1048588 / 0`、floor-raise `8223 / 0`、sigma `65548 / 0` | 不是键的问题：这四项在 L2 里此前没有一块全量判定（`blocks.tsv` 里这四类节点的 GPU 版本只有 f0b9512b 与三把快档版本，量的）；以前没判过 | `grep position_addressed fullroot/libs/library-by-conclusion-kind/blocks.tsv` |
| R6 | 崩溃放量，`runs/entry-*-3168619/`（10-13） | rollback `pending_states=67502198 states_reused=0`，`gpu_judge:dca6156d6e736c21` | 判法版本 f0b9512b→dca6156d。4fc0bcda→9da32924 闭包文件改了 8 份、都改了代码：`crash.rs`、`crash_amplification.rs`、`crash_facts.rs`、`crash_judge_common.wgsl`、`crash_judge_tables.rs`、`crash_library.rs`、`crash_verify_gpu.rs`、`verdict_store.rs`（量的；其中流水线与存储 3 份）。L2 里 f0b9512b→dca6156d 重判 19192 块、1255932011 个状态（量的） | 同上 |
| R7 | 崩溃点重放 --full，`.git/singlefs-layer0-logs/*-3532157/`（10-12，9 条用例） | 汇总行「这一趟判绿 9 条、复用 0 条」；每条 `resumed_slices=0`（`f-layer0-full.l.log` 第 60–191 行，例 position-addressed 两片 `states=268435480 … freshly_run_slices=4098 … elapsed_seconds=967.2`、`states=268435468 … elapsed_seconds=2181.0`） | 两样：库目录换了（这一趟写 `library=…/layer0-enumeration-library`，30 行，量的），节点名与放量不同。同一判法版本 f0b9512b 下，L2 早已判过的块里有 9679 块、632578246 个状态与 L3 这一趟判的五样全等，只差节点名（`…position_addressed_trees.rs::position_addressed::ExtentInlineToLowerSegment::full` 对 `…::position_addressed_ExtentInlineToLowerSegment`，量的） | `tools/cross.py`；名字对照的 python 输出 |
| R8 | 同一趟的 crash-injection-fast-tier（给定状态库 L5） | `tables_judged_here=116 tables_reused=0 … states_reused=0` | 库被手工换过：L6（`…polluted-by-gpu-fault-tests-2026-10-11`）生日比 L5 早约两个半小时（量的）；L5 与 L6 之间同一块内容、种类、版本全等的有 2833 块、15072 个状态（L6 的全部 2833 块）（量的） | `stat -c %W`；`tools/cross.py` |
| R9 | 崩溃点重放 --full 三条双机用例，`runs/layer0-layer0-*-*-2504991`、`*-2515435`、`*-504464`（10-12 至 10-13） | second-stream 两片 `resumed_slices=0 freshly_run_slices=12806`，`elapsed_seconds=9286.2` 与 `7670.6`；multi-record 两片 `3131.3`、`2332.3`；first-stream `46.0`、`38.8` | 前几趟这三条在分片里 panic、没判出东西（路子三量过），这一趟没有可复用的旧判定（量的：L3 里这三类节点只有 f0b9512b 一把）。放量在 L2 里判过同样状态数的流，但块键交集为 0：崩溃点重放把整条流当一个节点（`fixed_script::full`：1 个输入指纹、1 个计划哈希、25611 块），放量按点拆（`common_crash_points/mod.rs::unmount.index_node.1` 这类），输入指纹交集 0、计划哈希交集 0（量的） | python 输出「fps 1 fp∩amp 0 plans 1 plan∩amp 0 blocks 25611 blk∩amp 0」 |
| R10 | 崩溃点重放 --full，`.git/singlefs-layer0-logs/*-1997314/`（10-12，同一天随后那一趟） | 12 条用例的全绿标记全换了指纹（库里全复用：例 `resumed_slices=2 freshly_run_slices=0`） | 标记层：floor-raise 两份标记（76e959ce→a7f8d71a）189 个文件里变了 3 个：`crates/mutations.tsv`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/verdict_store.rs`；c561-sigma-full 一样是这 3 个（量的）。三份都不是用例调到的判定函数：变异表、不相干的实验装置、存储 | `tools/markerdiff.py` |
| R11 | 崩溃点重放 --full，`.git/singlefs-crash-point-replay-logs/*-3463989/log.floor-raise-pushed-by-the-session`（10-15，全量 full-2026-10-15） | `CRASH_POINT_REPLAY_PARALLEL_START states=8223 slices=2 … resumed_slices=0 freshly_run_slices=2 judge=gpu library=/home/fy5090/.local/share/singlefs/crash-amplification/crash-state-enumeration-library item=floor_raise_pushed_by_the_session::full imported_blocks=0` | 库层：块内容没变——（输入指纹 406e11089ff57faa…、计划 c0857d8dfe22c7c7…、起点 0）这一块在 L3 与 L2 里都有 GPU f0b9512b 的判定，在 L4 里这一趟换成 GPU 4e9cfb6b 现判（量的，三库 `blocks.tsv`）。变的是两样：判法版本 f0b9512b→4e9cfb6b，库目录 L3→L4（改名）。标记层：a7f8d71a→5a861ba6 文件 189→426，摘要变了 148、新增 249（其中 `crates/singlefs-harness/tests` 240 份）、去掉 12（量的） | `tools/markerdiff.py`；`grep floor_raise fullroot/libs/*/blocks.tsv` |
| R12 | 同一趟另三条双机用例与 c561-sigma-full | `GPU_QUIESCE step=exit code=78`（没判到） | 不是键的问题：清场回读没过 | `log.c561-sigma-full` |
| R13 | crates 变异整表，`commit-2026-10-11/checker-tier-crates-mutation-replay.log`（10-12，两片） | 第 15 行「复用抓到 0 条、复用基线 ok 0 行，现跑变异 1759 条、现跑基线 248 批」；第二台片 `.git/singlefs-mutation-shard-logs/mutation-2446965-*/peer.log` 第 1 行「复用抓到 0 条、复用基线 ok 0 行 …判法版本 37ab6ce31a0dbf86…」 | 判法版本 23404d01→37ab6ce3（`mutation-318844-*/local.log` 第 1 行与 `mutation-2446965-*/local.log` 第 1 行，两趟都写「删了更早的判法版本目录 1 个」，量的）。判法版本的输入盘上没存（§6）；git log 里 10-12 这天没有一个提交碰判法版本的那几份输入（`git log --since=2026-10-09 -- <六份>` 只有 10-10、10-11、10-14 的提交，量的），是工作区里没提交的改动（推的）。同一趟跨片复用 1693 条是本机片刚判的（路子三已量），跨趟复用 0 | `hits.txt` |
| R14 | crates 变异 --quick（轻型，10-13 至 10-14，与 R13 共用 `.git/singlefs-mutation-rows`） | 每趟「复用抓到 0 条、复用基线 ok 0 行」：判法版本 f8ab0022（`detached-runs/owed-c456-quick-2/log` 第 9 行）、19937413（`mutation-light-2026-10-13.log` 第 3 行）、604b3601（`mutation-light-2rows-16G-2026-10-13.log` 第 3 行）、3a63d840（`detached-runs/owed-c456-quick-3/log` 第 17 行）、d0ef8d8c（`two-stage-validate/v4.log` 第 6 行） | 每趟一把新判法版本、一律 0；其中 4 趟「删了更早的判法版本目录 1 个」。另量到行键随限时变：3a63d840 里限时 1800 与 3600 两种都有，与 37ab6ce3 内容相同（`row_content_hash` 相等）的 257 行里，限时相同的 94 行行键相同，限时 1800→3600 的 163 行行键全变；3a63d840 与 d0ef8d8c 之间 `table_line`（表里的行号）变了而限时没变的 123 行行键不变（量的） | `.git/singlefs-mutation-rows/*/mutation.*` 逐条比 |
| R15 | 形式化，`commit-2026-10-11/checker-tier-formal-verification.log`（10-13） | 第 377 行「  pinned-red-examples-replay：现判 360、复用 0（现判的原因：装置 变了 360）」 | 装置那一样（e165 装置 bin 的 cargo 依赖闭包逐文件摘要，`e165-pin-red-examples.py:1171`–1182、`:1308`）变了；d83fe646→1722e788 crates 下只改了 `crates/singlefs-harness/src/read_tally.rs`（+112）与 `crates/singlefs-harness/tests/presumed_clause_checks.rs`（量的 `git diff --stat`）；库里 `装置` 一样在 10-09 到 10-13 有 6 个取值，每换一次 360 条全现判（量的） | `.git/singlefs-formal-verdicts/pinned-red-examples-replay/keys/*.json` |
| R16 | 形式化 10-12（日志已被覆盖，只剩库） | vacuity 33、must-red-worlds 153、pinned 360 条在 10-12 全部新记一份 | 键的形状换了：10-09 那批键的 components 里有「判法」一样，10-12 起「判法」挪到另一列 `judge_version`（量的，按记录日期分组的 components 名单）；同时「核 crate」（整个 crate 目录的摘要）与「verus-pins.tsv」（整张表）也变了（量的）。10-09 的 942 条旧形键没人再读（量的：对不上新键公式） | 按条 components 逐样比的 python 输出 |
| R17 | 形式化 proof（10-13→10-14） | proof 一项 10-14 新记一份 | components 五样逐字相同，只有判法版本 6dacce53→3eecb666 变了（量的）；判法版本 = 整份 `verus-proof-check.py` 原文摘要 + 整份 `lib_verdict_library.py` 原文摘要（`verus-proof-check.py:793`）；1b6f9865 改的是 `--threads` 缺省取法与自证格（`git show 1b6f9865 -- scripts/gate/verus-proof-check.py`，量的），与判 Verus 证明无关 | 同上 |
| R18 | 研究侧 mutation-tables-compile，`detached-runs/owed-c798-full-cell/log`、`-3/log`（10-13） | 「现判 164 张、复用 0 张」两趟 | `owed-c798-full-cell` 那一趟：130 张「现判（无记录）」、34 张「现判（mutate.sh、判法版本 变了）」；`-3` 那一趟：164 张全是「现判（mutate.sh、判法版本 变了）」（`grep -o '现判（[^）]*）'` 再 `sort`、`uniq -c` 数的，量的）。判法版本 = 整份阶段脚本 + `lib_verdict_library.py`；`mutate.sh` 整份进每张表的键。库在 `${TMPDIR:-/tmp}` 下（`checker-tier-research-build-and-replay.sh:458`），现在不在了（L25） | `hits.txt` |
| R19 | LKMM（10-12 暂存树 d83fe646 那一趟、10-13，`commit-2026-10-11/summary.txt`） | `checker-tier-lkmm rc=0 秒=1`（两次重跑），标记 1849ded8→b5ae6404→d22421f9 | 暂存树 74022d17→d83fe646：只动了 `crates/mutations.tsv` 与 `crates/singlefs-checker-tier/` 下 2 份；d83fe646→1722e788：`.claude/gate.d/stage-inputs.tsv` 与 `crates/singlefs-harness/` 下 2 份；`litmus/`、`lkmm.sh`、`fetch-deps.sh`、`lib.sh` 两次都 0 份（量的 `git diff --name-only`）。标记只存个数（371）不存清单，盘上比不出来，用暂存树补 | `fingerprints.txt` 的 tree 列 |
| R20 | 广谱变异，`detached-runs/broad-mutation-local/log`（10-01） | `BROAD_MUTATION_PLAN fingerprint=19c45e021b5638f6 commit=fcc4d3404c80 list=5091 selected=2397 threads=31 memory_cap=24G`；结论表只有 2 行 | 两版 19c45e02→e1ba2574 的 `global-key.txt` 只差一行：`< rules prune-r3-e1-e6` / `> rules prune-r3-e1-e6-fresh-build`；`manifest.tsv` 218 个文件逐行相同（量的 `diff`）。即只因手写的判法版本常量改了一个字串，19c45e02 那一版 304 行结果一行也没进跨版本结论表（`wc -l versions/*/results.tsv` 304 与 3，量的）。现在驱动里的常量又是 `prune-r3-e1-e6-fresh-build-mac`（`broad-mutation.py:469`），下一趟这 2 行也不沿用（推的，按文件头） | `diff versions/*/global-key.txt` |

快档（轻型）旁证，不在射程，只说明它与重型共用库时的后果：崩溃放量快档 10-10 至 10-14 七趟各一把新的 GPU 判法版本（cc324f51、7e1d5ad8、0024a93a、cedd6ed8、3b773890、c17fb1e8、4183272c），跨趟复用都是 0，库内复用只来自同一趟里共前缀的点（例 `fixed-script-through-unmount pending_states=145 states_reused=116`，`entry-table.txt`，量的）。这几把版本的块写进了重型也用的 L2、L4、L5（L2 里 3b773890、c17fb1e8、4183272c 各 115 块，量的）。

## 3 阻碍剪枝清单（交付表）

判据照派发提示：一条判定沿不沿用，只许看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本。①按提交 / HEAD / 上一次的树算改动；②整份原文或原文字节进键；③文件级、目录级、包级、整表级指纹或闭包当键；④手写版本常量当判法版本；⑤键里带路径、名字、流名、类名、种子串、行号、限时这类不是函数内容的东西；⑥没有判定库、只有整道或整格一个标记，或库每趟新建；⑦键过窄会漏判；⑧库放在会丢的位置、改名丢库、两库重复判。
本路只看盘上的东西：「位置」先写盘上看得到它的那份文件或库，再写写出它的那一行代码（代码的键算法归别的路子，这里只引到看得见那一样的那一行）。每行末尾标量的 / 推的。

| 编号 | 位置（文件:行号） | 管哪种判定 | 键由什么组成（逐项，盘上看到的） | 违反 | 重做方向（单元怎么切、键改成什么、落点） |
|---|---|---|---|---|---|
| X1 | 盘：L1–L5 各库 `0A`/`01` 键里的节点名（`fullroot/libs/*/blocks.tsv` 第 7 列）；代码：`verdict_store.rs:22`（块键布局）、`crash_identity.rs:71`（`identity_text`） | 崩溃态结论块（GPU、CPU、观察者三种） | 输入指纹 32 字节 ‖ 节点名（`<设点文件仓内路径>::<点名>::<复用键十六进制>`）‖ 计划哈希 ‖ 块起点 ‖ 种类 ‖ 版本 | ⑤ ⑧ | 块键只留内容（起步镜像、写表、枚举计划、块起点），节点名与路径只进已有的路径索引表 `06`；量的：L1 里 4100 块 / 268435542 状态只差设点文件路径，L3 与 L2 之间 9679 块 / 632578246 状态只差点名写法 |
| X2 | 盘：L5、L6 节点名例 `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs::crash-injection-fast-tier::seed=7463871032432355113/first-crash/power-losses=[]`；代码：`crash_amplification.rs:12126`–12129（节点名 = 设点文件 :: 项名 :: 表名）、`:12095`（输入串带 `item=`、`seeds_and_scale=`） | 给定状态表的结论块 | 同 X1，节点名里另带项名与带种子的表名；输入串带种子与规模串 | ⑤ | 表的内容（写、持久集合、基础镜像、容量旋钮）已在输入串里，去掉项名、种子与规模串、设点文件；落点 `crash_amplification.rs` 给定状态块键那一段；量的（键里的字样），种子数变了前面的表会换键是推的 |
| X3 | 盘：L2 GPU 7 把、L4 6 把、L5 8 把版本，版本间重判见 §2 R3/R4/R6；代码：判法版本取判器闭包摘要（`verdict_store.rs:16`–24 文件头「复用只认同一判法版本」） | GPU 结论（一把对全部点） | 判器闭包里每份文件的摘要（.rs 取词法、wgsl 取原文）、停止表、`Cargo.lock` 与各 `Cargo.toml` 原文、工具链（组成照路子二报告，盘上只看得到结果） | ③ ② | 单元切到判器真调到的函数：每个 fn 项去注释摘要，判法版本 = 这一块判定路径上那些函数摘要的集合；流水线、存储、派活不进；落点 `judging_closure.rs`、`crash_identity.rs`；量的：L2 里 35455 块内容在 >1 把版本下各判，多判 3935242934 个状态 |
| X4 | 盘：§2 R4 那三份 wgsl 只改注释、版本照换；代码：同 X3 | GPU 结论 | wgsl 内核按原文字节进摘要 | ② | wgsl 也按词法去注释，最好按内核函数切单元；落点 `judging_closure.rs`、`build.rs` |
| X5 | 盘：L4 CPU 50 把版本对 749 块内容，698 块在 >1 把下判过、多判 3057 次；L3 CPU 124 把、观察者 93 把 | CPU 结论、观察者结论 | 设点文件（测试文件）连同它 `mod` 进来的公共测试模块的闭包摘要 + 判器闭包 | ③ ⑤ | 版本只取被判这一块真用到的观察者函数与判器函数的摘要，不取整份设点文件；落点 `judging_closure.rs`（入口文件那一项）、`crash_state_enumeration.rs` 给观察者代码文件那两处 |
| X6 | 盘：L5 里 1431 块内容 GPU 与 CPU 各判一份，L2、L3、L4 各 24、26、24 块 | GPU 与 CPU 两种结论 | 结论种类进键，两种互不借 | ⑧（要设计判断） | 两种判器判同一块、结论按位比过一致的，记一行对拍证据后互借；判不判得等价交三方；量的块数 |
| X7 | 盘：L1→L2（布局换）、L3→L4（改名）、L6（手工挪开）；代码：`crash_library.rs:131`–136（类→目录名）、`crash-amplification.sh:7`、`common_given_states_of_harness_scenario_images/mod.rs:39`–40 | 全部崩溃结论 | 库目录名随类名与术语定 | ⑧ | 一份库、目录名不带类名与术语、不随改名动；换布局或改名时迁移而不是另起；量的：L3 620M、L1 271M、L6 104M 孤立，R2、R7、R8、R11 四趟复用掉 0 都有这一条 |
| X8 | 盘：L2 与 L3 之间 9679 块全等各判一份；L5 与 L6 之间 2833 块；R9 那条流 25611 块与放量交集 0 | 崩溃点重放 × 崩溃放量 | 两边块的单元不同：崩溃点重放整条流一个节点一份计划哈希，放量按点 | ⑧ ③ | 两边统一按「点 + 段」切块（同一份枚举计划切法），块键只留内容，两类共一份库；落点 `crash_state_enumeration.rs` 的点划分、`crash_amplification.rs` 的 `plan_crash_points`；量的交集数 |
| X9 | 盘：R9 `fixed_script::full` 一个计划哈希管 25611 块；代码：计划哈希按整个点算（路子二报告 `crash_amplification.rs:624`–733，本路没现读） | 结论块 | 计划哈希 = 这个点所有段的序、态数、展开方式、下游发布、判器另读的输入 | ③ | 计划哈希按段拆，块键带段哈希与段内序号；点里别的段变了不连累；量的（一个哈希管 25611 块），影响面推的 |
| X10 | 盘：L1–L5 新布局都没有 `09` 表，L1 的 `09` 只有文件名；代码：`verdict_store.rs:20` | 判法版本的出处 | — | 不在 ①–⑧（归因缺口） | 每把版本落一份「函数单元 → 摘要」清单进库，换版本时能逐函数 diff；落点 `verdict_store.rs`、`crash_identity.rs` |
| X11 | 盘：L7 每份子库每趟新建（抽 4 份 `LOG.old` 0 份）；代码：`common_given_states_of_harness_scenario_images/mod.rs:39`–40 | 镜像场景用例的结论 | 一个用例一份库，以用例函数名命名 | ⑥ ⑤ | 并进 L4/L5 的同一份库，靠块键内容复用；要单独核判到位的用例只读不清；量的 |
| X12 | 盘：L8 顶层 + 61 份按标签的子库；代码：`crash_library.rs:121`–123、`:136` | 放量流水线单测的结论 | 一个用例标签一份库 | ⑤ ⑧ | 同 X11；落点 `crash_library.rs` 的 `amplification_unit_tests_library_directory` |
| X13 | 盘：L11 五份镜像库 980M；代码：`crates-mutation-rows.py:263`–268 | crates 变异复跑时被测代码跑出的崩溃结论 | 另一个家目录，库与长期库分开 | ⑧（推的） | 变异后的判器摘要本来就不同，按函数单元键存进同一份库不会串；判不判得要看 X3 改完之后版本是否覆盖到被变异的函数（推的） |
| X14 | 盘：L12 213 份逐趟 RocksDB、89G；代码：双机驱动 `crash-amplification-two-hosts.sh`（行号待查） | 双机分片传送 | 每趟整份复制库给第二台、再整份取回 | ⑧ | 只传第二台要判的块与它要读的表，取回只导入新判的块，副本跑完删；量的大小，「没人再读」推的 |
| X15 | 代码：`crash-point-replay.sh:16`–17、`:632`；盘：`crash-point-replay-full/` 目录不存在（从没跑过） | 崩溃点重放入口 `--full` | 每次换一份新的空库、先删全绿标记、带 `--start-over` | ⑥ | 全量也照块键读库（全量只是「不跳过任何一块」，不是「不读库」）；量的（文件头原话与目录不在） |
| X16 | 盘：L16 标记里 `input_file` 清单（例 floor-raise 5a861ba6：426 行，含 `crates/mutations.tsv`、`src/bin/*.rs`、240 份 `crates/singlefs-harness/tests/*`、`Cargo.lock`、`Cargo.toml`、`<判法摘要：admission.py …>`、`<登记行：crash-case:…>`）；代码：`admission.py:336` | 崩溃枚举用例整条（编译、录入段、问库） | 整个 `crates/` 下逐文件的词法摘要（`.git/singlefs-source-digest-cache.tsv` 那一列）+ Cargo 清单原文 + admission.py 判法摘要 + 工具链 + 构建环境 + 登记行原文 | ③ ② ⑤ | 用例级键取这条测试从入口函数起真调到的函数单元摘要集合（非 Rust 输入按解析出的依赖版本）；登记行只取会改判法的字段；落点 `admission.py` 用例清单那段；量的：R10 一趟 12 条全失标记只因 3 份无关文件，R11 失标记时变了 148、多了 249 |
| X17 | 盘：L16 有标记的 12 条里 5 条（`layer0-*`）已不在登记表，登记表 14 条里 7 条没有标记 | 崩溃枚举用例整条 | 标记文件名与键带用例名 | ⑤ ⑧ | 键不带用例名，标记按内容键存；用例改名不丢；量的（`comm` 两份名单） |
| X18 | 盘：L17 标记只有 `stage=`、`input_hash=`、`input_file_count=`、`finished_utc=`、`judged_root=`、`heavy_prefix=` 六行（例 `.git/singlefs-stage-green.checker-tier-lkmm.sh.1849ded8…` 第 2–7 行）；代码：`admission.py:339`、`:1590`（整道阶段登记表整份进比对）、`stage-inputs.tsv:38`（LKMM：`litmus/ … crates/`） | LKMM 整道（8 份 litmus 一个标记） | 登记的几个目录逐文件摘要 + herd7 版本 + 整份 `stage-inputs.tsv` | ③ ⑥ | 一份 litmus 一格：键取这份 litmus、它绑到的那几个函数单元（代码绑定表点名的 fn）、herd7 版本；不收整个 `crates/` 与整张登记表；落点 `.claude/gate.d/checker-tier-lkmm.sh`、`stage-inputs.tsv:38`；量的：R19 两次重跑 litmus 侧 0 处改动 |
| X19 | 盘：L17 QEMU 两份标记（10-01 `input_file_count=697`、10-12 `1675`）；代码：`stage-inputs.tsv:37`（`crates/ Cargo.toml Cargo.lock scripts/ research/results/ .claude/kb/layout/01-first-txn.md`） | QEMU 六次虚机跑整道一个标记 | 登记的整目录（含整个 `scripts/`、`research/results/`）+ 整份登记表 | ③ ⑥ | 一次虚机跑一格，键取那一档装置真调到的函数单元、QEMU 构建戳（`~/.local/share/singlefs/qemu/singlefs-build-stamp.txt` 已有源版本、补丁 sha256、configure 参数）与内核镜像；落点 `checker-tier-qemu-device-streams.sh`、`stage-inputs.tsv:37`；量的（个数），两份之间变了什么盘上判不了 |
| X20 | 盘：L17 形式化四格（verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds）在暂存树 74022d17→d83fe646 之间各重写一份标记，而登记的输入里只变了 `scripts/hosts/run-with-memory-cap.sh`（量的 `git diff --name-only`）；代码：`stage-inputs.tsv:39`–42 | 形式化按格外层标记 | 登记行逐文件原文摘要：`crates/singlefs-replay-prefix/` 整目录、`verus-pins.tsv` 整表、判法脚本、内存包装、`capped.sh`、`stage-cells.sh`、`lib_verdict_library.py`、verus 版本 | ③ ② | 外层格标记去掉，直接靠按条库（X21–X24）；内存包装、`capped.sh`、`stage-cells.sh`、存储共用库不进任何键；量的 |
| X21 | 盘：`.git/singlefs-formal-verdicts/{proof,vacuity,must-red-worlds}/keys/*.json` 的 components：「核 crate」「verus-pins.tsv」「工具链」「限时」，vacuity 另加「这一处」、must-red 另加「这一行」；代码：`verus-proof-check.py:807`、`:808`、`:810`、`:811` | Verus 证明、空真、必红世界按条 | 核 crate 整个目录摘要（`digest_tree`）+ 钉表整份 + 工具链 + 限时 + 这一项自己那一段 | ③ ⑤ | 一项的键取这一项证的那几个函数（`ensures`/`requires` 所在 fn 与它调到的 spec fn）各自去注释摘要 + verus 版本；限时不进键（超时不记绿，本来就不复用）；钉表只取这一项引的那一行；量的：R16 一次「核 crate」变了 33+153+1 项全现判 |
| X22 | 盘：vacuity 项名 `vacuity:crates/singlefs-replay-prefix/src/lib.rs:1197` 这类（`items/*.json` 的 `item`）；代码：`verus-proof-check.py:1015`（项名带核的相对路径与 requires 行号）、`:1068`（「这一处」= `digest_text(f"{item}\t{label}")`） | 空真按条 | 「这一处」摘要里带路径与行号 | ⑤ | 「这一处」改取那条 requires 所在函数的去注释摘要与它在函数里的序；上面插一行不该让下面每一处换键；落点 `verus-proof-check.py` 收空真点那一段；量的（项名），插行连带换键是推的 |
| X23 | 代码：`verus-proof-check.py:1202`（必红世界「这一行」= name、file、original、replacement、expected） | 必红世界按条 | 「这一行」带被改文件的路径 | ⑤ | 换成被改那个函数的去注释摘要 + 替换文；落点同文件 |
| X24 | 盘：proof 的 10-13 与 10-14 两份键 components 逐字相同、`judge_version` 6dacce53→3eecb666（量的）；代码：`verus-proof-check.py:793`（判法版本 = 整份脚本原文 + 整份 `lib_verdict_library.py` 原文） | 形式化四种按条 | 判法版本 = 两份脚本整份原文摘要 | ② ③ | 判法版本只取「怎么判绿」的那几个函数（结果行解析、判绿条件）去注释摘要；线程、调度、自证、存储不进；落点 `verus-proof-check.py`、`e165-pin-red-examples.py` 的 `judge_version`；量的：R17 |
| X25 | 盘：`pinned-red-examples-replay/keys/*.json` 的「装置」6 个取值、每换一次 360 条全现判；代码：`e165-pin-red-examples.py:1171`–1182、`:1308` | E165 钉住红例按条 | 装置 = e165 装置 bin 的 cargo 依赖闭包逐文件摘要 + `Cargo.toml` + `Cargo.lock` + 工具链，360 行共一样 | ③ ② | 每行的键取这一行红例真走到的函数（装置入口到被钉变异所在函数的调用路径）去注释摘要；落点 `device_inputs_digest`；量的：R15 harness 新加一个 E165 不读的模块就 360 条全现判 |
| X26 | 盘：942 条 10-09 旧形键没人读（R16）；代码：`lib_verdict_library.py:14`–17（键 = sha256（输入指纹 NUL 判法版本）） | 形式化全部 | 键的形状一变（「判法」从 components 挪到另一列）旧键全部对不上 | ⑧ | 键形状换代时对旧记录按新公式重算一次键、而不是丢；或键只由函数单元组成、形状不再随判法脚本改；量的 |
| X27 | 盘：L18 三个版本目录，10-12 至 10-14 至少 7 把不同的版本、7 次删版本目录；每把版本下 `复用抓到 0`（R13、R14）；代码：`crates-mutation-rows.py:59`–61（判法版本 = 登记行 + 阶段脚本 + 本模块 + `mutation-prune.py` + `lib_libtest_result_lines.py` + `lib_verdict_library.py` + 内存包装 + 构建环境）、`:84`、`:712`（只留 3 个） | crates 变异每一行（抓到、基线 ok） | 版本目录 = 上述各份脚本的摘要 | ② ③ ⑧ | 判法版本只取「抓没抓到」的那几个函数（libtest 结果行解析、切 fn 规则）去注释摘要；内存包装、阶段脚本调度、存储共用库、构建环境不进；版本目录不按个数删；落点 `crates-mutation-rows.py` |
| X28 | 盘：L18 `mutation.*` 记录里 `memory_max=4G`、`timeout_seconds=1800` 或 `3600`、`toolchain` 进行键：限时 1800→3600 的 163 行行键全换（R14）；代码：`crates-mutation-rows.py:640`–641 | crates 变异每一行 | 行键 = 这一行六段原文（含被改文件路径、变异名）+ 内存上限 + 限时 + 工具链 | ⑤ | 行键只取被改函数（去注释）摘要 + 替换文 + 工具链；路径、名字、上限、限时不进；落点 `row_key` |
| X29 | 盘：L18 `snapshots/*.json.gz` 只有 `files: {路径: 原文 sha256}`（379–438 份文件），`blobs/` 存源码原文 | crates 变异复用判据的「源码与上次一字未动」 | 整仓逐文件原文 sha256 | ②（作定位用） | 快照改存「函数单元 → 去注释摘要」，复用判据直接比被变异函数与执行过的函数的单元摘要，不经整文件原文；落点同上；量的（快照内容） |
| X30 | 代码：`checker-tier-research-build-and-replay.sh:46`–47（「相对上次整轮全绿那棵暂存树没变就退 77（scripts/gate/stage-must-run.sh）」）；盘：`.git/singlefs-stage-green.checker-tier-research-build-and-replay.sh*` 一份也没有（L17） | 研究侧 mutation-tables-compile 与 experiment-replay 整格 | 登记的路径相对上次整轮全绿的暂存树变没变 | ① ③ ⑥ | 去掉按暂存树比，整格不设外层，直接按表、按行的库；落点 `stage-must-run.sh` 与阶段脚本；量的（原话与盘上没有标记） |
| X31 | 盘：L25 不存在；R18 两趟 164 张全现判；代码：`checker-tier-research-build-and-replay.sh:458`（库在 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}`）、`:48`–49（每张表的键：表、源文件、include 的文件、`scripts/experiments/` 的 `Cargo.toml`、`Cargo.lock`、`lib.rs`、`mutate.sh` 各自 sha256 + 工具链；判法版本 = 阶段脚本 + 判定库两份 sha256） | 研究侧变异表逐表只编译 | 见左 | ② ③ ⑧ | 库挪到 `~/.local/share/singlefs/` 或 git common-dir；一条替换文一个单元，键取被替换的那个函数去注释摘要 + 替换文 + 解析出的依赖版本（不收整份 `Cargo.lock`、`lib.rs`、`mutate.sh`）；判法版本只取「编不编得过」的判法函数；量的 |
| X32 | 盘：L26 不存在；代码：`replay.sh:37`（`${REPLAY_VERDICT_LIBRARY:-<git common-dir>/singlefs-replay-verdicts}`） | 实验复跑按行 | 设计上按行，盘上一次都没落过 | ⑥ | 先让它落盘，再按函数单元收装置键；落点 `replay.sh`；量的（目录不在），以后会不会落推的 |
| X33 | 盘：L24 `versions/*/global-key.txt`：`rules prune-r3-e1-e6-fresh-build`、`file Cargo.lock …`、`file Cargo.toml …`、各 crate `Cargo.toml`、`.cargo/config.toml`、`.cargo/mutants.toml`、`tool cargo-nextest/cargo-mutants/LLVM`、`exclusions <摘要>`、`limits cpu=3.0x+30.0 wall=10.0x+300.0`；代码：`broad-mutation.py:469`、`:2869` | 广谱变异全部结论 | 手写规则常量 + Cargo 清单与锁原文 + 构建配置原文 + 工具 + 排除表整表摘要 + 限时 | ④ ② ③ ⑤ | 判法版本取剪枝规则函数（`mutation-prune.py` 里判沿用的那几个 fn）去注释摘要；依赖取被测包解析出的依赖版本；排除表只取影响这一条的那一行；限时不进；量的：R20 只差 rules 一行就丢 304 行 |
| X34 | 盘：`conclusions.tsv` 表头 `key outcome caught_by test_fingerprint fingerprint dependency global_key version commit sequence finished_at`，键例 `["crates/singlefs-core/src/pointer.rs", "<impl LocationEntry>::write_to", "14.0-78.1", "()"]`，`commit=fcc4d340…`；`tree.source` 记 `"commit": "fcc4d3404c80…"`；结果行变异名带 `address.rs:72:9`；代码：`broad-mutation.py:467`、`:1459`、`:2976` | 广谱变异逐条结论 | 键 =（文件路径、限定名、函数内相对起止、替换文）；每行另记提交号；导出的树按提交 | ⑤ ①（提交号进不进沿用判据归代码路子核） | 键去掉文件路径（函数挪文件不换键），只留函数单元摘要 + 函数内位置 + 替换文；提交号只作记录；导出按内容不按提交；量的（盘上字样） |
| X35 | 盘：L21 `.git/singlefs-layer0-progress/`（100M，09-27） | 旧进度文件 | 目录按输入指纹、文件名带流名与哈希 | ⑧ ⑤ | 删；进度只是加速，按块键问库即可；量的 |
| X36 | 盘：`commit-2026-10-11/f-amplification.log` 第 18 行 `CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=0 reused=67502198 …`，同一行末 `seconds=143`；代码：`crash-amplification.sh:2`（`# admission: always 每次调都在这一刻的树上跑全量或点名的分项`） | 崩溃放量每个分项整项 | 没有分项级的键，每趟每项照跑录入段（跑流、排计划、写判表） | ⑥ | 分项外层也按函数单元键：录入段的产物（录下的流 `10` 表、判表 `12` 表）本来按摘要存，录入段先按「录入用到的函数单元 + 流程输入」查这几张表，查到就跳过；量的：一项现判 0 个状态仍花 143 秒 |
| X37 | 代码：`stage-inputs.tsv:45`（crates 变异整道：`crates/ Cargo.toml Cargo.lock litmus/ .cargo/` + 脚本若干）、`:46`（广谱变异整道：同类加 `admission.py`、排除表）；盘：两道都没有一份整道标记（L17） | crates 变异、广谱变异整道 | 整目录逐文件摘要 + 整份登记表 | ③ ⑥ | 整道不设外层标记，只靠按行 / 按条的库；落点 `stage-inputs.tsv`、`admission.py` 的 `stage_fingerprint`（`:2797`）；量的（登记行与盘上没有标记） |
| X38 | 代码：`stage-inputs.tsv:36`（崩溃点重放整道：`crates/ Cargo.toml Cargo.lock` + 双机脚本若干）、`:53` 起各 `crash-case:` 行（每条都是 `crates/ Cargo.toml Cargo.lock`） | 崩溃点重放整道与逐条用例 | 整个 `crates/` | ③ | 同 X16：逐条用例按测试入口的函数单元闭包；落点 `stage-inputs.tsv` 各 `crash-case:` 行与 `admission.py` 用例清单那段；量的（登记行原文） |

合并提示（给主 agent 并表用）：X1、X2、X8、X9、X17、X35 是「块与标记的单元与名字」一族；X3、X4、X5、X10、X24、X27、X31、X33 是「判法版本取整份脚本或整个闭包」一族；X16、X18、X19、X20、X21、X25、X29、X37、X38 是「输入按文件、目录、包、整表」一族；X7、X11、X12、X13、X14、X15、X26、X31、X32、X35 是「库放哪、丢没丢、几份」一族；X22、X23、X28、X34 是「键里带路径、行号、限时」一族；X30、X34 带 ①。

## 4 跑过的命令与原样输出（校验）

解析器自己会不会错：拿 RocksDB 自己记在 `LOG` 里的 `table_file_creation` 事件（每份 SST 的 `num_entries`）对我解出来的条数，这是一条不经我的解析器的独立路子。
只解块前缀的快扫在大块上漏条，这条校验抓到了；整解之后逐条对上。§0–§3 里的块数、状态数都取整解的那一份（`fullroot/`）。

```
$ grep -h "\"file_number\": 29279," LOG LOG.old.* | grep table_file_creation | grep -o '"num_entries": [0-9]*'   （crash-state-enumeration-library，另两份同法）
29279 rocksdb: "num_entries": 1952681 "num_deletions": 0
29285 rocksdb: "num_entries": 9 "num_deletions": 0
29280 rocksdb: "num_entries": 11013 "num_deletions": 0
快扫：029279.sst parsed entries 1952663 deletions 3 / 029285.sst parsed entries 9 deletions 0 / 029280.sst parsed entries 11007 deletions 0
整解：029280.sst full-decode entries 11013 deletions 0 / 029279.sst full-decode entries 1952681 deletions 0
```

整解 23 片并行、每片退出码各写一份：`派出 23 件，收到 23 份`、`23 0`。整解与快扫的 `0A`/`01` 行数：library 128952 对 128910、library-by-conclusion-kind 118081 对 118036、layer0-enumeration-library 57236 对 57211、crash-state-enumeration-library 3845 对 3835、given-states-library 19558 对 17681、polluted 2833 对 2833。

形式化判定库的键公式（`lib_verdict_library.py:14`–17）在盘上逐条重算，看键里有没有 components 与判法版本之外的东西：

```
{('must-red-worlds', 'oldform'): 155, ('must-red-worlds', True, True): 153, ('pinned-red-examples-replay', 'oldform'): 753, ('pinned-red-examples-replay', True, True): 1080, ('proof', True, True): 4, ('proof', 'oldform'): 1, ('vacuity', 'oldform'): 33, ('vacuity', True, True): 33}
```
新形 1270 条全部对上（输入指纹 = components 排序拼接的 sha256，键 = sha256（输入指纹 NUL 判法版本）），项名不进键（量的）；旧形 942 条没有 `input_fingerprint`、对不上。

crates 变异行键与限时（同内容的两条记录逐对比：行键同否、限时同否、内存同否、表里行号同否）：

```
37ab6ce3 3a63d840 (row_key same, timeout same, mem same, table_line same): {(True, True, True, True): 94, (False, False, True, True): 163} []
37ab6ce3 d0ef8d8c (row_key same, timeout same, mem same, table_line same): {(False, False, True, False): 115, (False, False, True, True): 19} []
3a63d840 d0ef8d8c (row_key same, timeout same, mem same, table_line same): {(False, False, True, False): 62, (True, True, True, False): 123, (False, False, True, True): 19, (True, True, True, True): 18} []
```
行键变了的每一对限时都变了，限时没变的行键都没变（反例 0 对，量的）。

闭包文件去注释比（`tools/strip_cmp.py`，`//`、`/* */` 去掉、空白并一个，字符串原样保留）：

```
663bb8b6->b5a35b44: 只改注释或空白 0 份；改了代码 11 份：…
b5a35b44->f1985f9f: 只改注释或空白 24 份；改了代码 5 份：crates/singlefs-checker-tier/src/crash_amplification.rs crates/singlefs-core/src/block_device.rs crates/singlefs-core/src/recovery.rs crates/singlefs-harness/src/lib.rs crates/singlefs-harness/src/memory_pool.rs
4fc0bcda->9da32924: 只改注释或空白 0 份；改了代码 8 份：…
```
（省略号处的文件名在 §2 R3、R6 里逐个列了；闭包文件清单取路子三的 `closure-files-68.txt`，那是 10-14 的清单，套到 10-11 至 10-13 的树上是推的。）

全绿标记两版逐文件比（`tools/markerdiff.py`）：

```
crash-case:floor-raise-pushed-by-the-session 2026-10-12→2026-10-12 文件 189→189，摘要变了 3，新增 0，去掉 0
   变: crates/mutations.tsv
   变: crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
   变: crates/singlefs-checker-tier/src/verdict_store.rs
```
10-12 → 10-15 那一对（a7f8d71a→5a861ba6）：`前 189 后 426 新增 249 去掉 12 摘要变了 148 不变 29`；新增按目录 `{'crates/singlefs-harness/tests': 240, 'crates/singlefs-checker-tier/tests': 5, 'crates/singlefs-checker-tier/src': 2, '<键的写法：2>': 1, 'crates/singlefs-harness/src': 1}`。

广谱变异两版全局键：

```
$ diff 19c45e02*/global-key.txt e1ba2574*/global-key.txt
1c1
< rules prune-r3-e1-e6
---
> rules prune-r3-e1-e6-fresh-build
```
`diff <(cut -f1 19c45e02*/manifest.tsv) <(cut -f1 e1ba2574*/manifest.tsv)` 没有输出，两份各 218 行。

块重复与重判（`fullroot` 下 `tools/cross.py`，全文在 `cross-full.txt`）原样摘行：

```
  library-by-conclusion-kind: 块内容 57518，其中在 >1 把版本下判过 35455 块，多判 60563 次、3935242934 个状态
  library: 4100 块内容有 >1 个名字，多存 4100 份、268435542 个状态 例 crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs::sequential_write.journal_record.1::08dde936b19d55e7f4f6898e4eb7e5ad56ce609d4050cdfb81d58c0417027b91 | crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs::sequential_write.journal_record.1::08dde936b19d55e7f4f6898e4eb7e5ad56ce609d4050cdfb81d58c0417027b91
   ('layer0-enumeration-library', 'library-by-conclusion-kind') 9679 块 632578246 状态
   ('given-states-library', 'given-states-library.polluted-by-gpu-fault-tests-2026-10-11') 2833 块 15072 状态
  given-states-library: 1431 块内容 GPU 与 CPU 各判一份
```
R9 的交集：`fixed_script::full fps 1 ∩ 0 plans 1 ∩ 0 blocks 25611 ∩ 0 states 1678377203`、`multi_record_publish::full fps 1 ∩ 0 plans 1 ∩ 0 blocks 20737 ∩ 0 states 1358954633`。

## 5 推翻条件（什么现象会推翻这些定位）

| 结论 | 会推翻它的观测 | 这一轮看过没有 |
|---|---|---|
| X1：块键带节点名，同内容各存一份 | 那 4100 / 9679 块的键在名字之外还有别的字节不同（我的解析把名字长度读错、把版本切错） | 看过：每块都解出合法的种类（gpu / cpu / observer:<名>）与 32 字节版本，若名字长度读错，种类字节会落在随机值上；整解条数与 RocksDB 自记的 `num_entries` 相等（§4） |
| X3：近两周崩溃放量复用掉 0 都是判法版本换了 | 同一把 GPU 版本下出现 `states_reused=0`、而 pending 等于全量的一趟 | 看过 `entry-table.txt`：同版本下没有（R2 是新库、R5 是以前没判过，另记） |
| X4：wgsl 只改注释也换版本 | 那 3 份 wgsl 在 b5a35b44→f1985f9f 之间改了注释以外的字符（我的去注释器漏了 wgsl 的某种注释或字符串写法） | 没逐行看那 3 份 diff；去注释器按 Rust 写法写的，wgsl 注释写法与它相同（推的） |
| X8 / R9：两边块键交集 0 | L2 里找到一块与 `fixed_script::full` 输入指纹相同的块 | 看过：交集 0 |
| R10：用例标记失效只因 3 份文件 | 两份标记之外另有进键的东西（键的写法、工具链）也变了 | 看过：`<工具链：…>`、`<构建环境：…>`、`<键的写法：…>` 那几行两份相同（只有 3 行 `input_file` 不同） |
| R14 / X28：行键随限时变、不随表里行号变 | 同内容、同限时、行键不同的一对；或限时不同、行键相同的一对 | 看过：两种都是 0 对（§4） |
| R17 / X24：proof 重判只因判法版本 | 两份键的 components 有一样不同 | 看过：五样逐字相同 |
| R20 / X33：广谱变异两版只差手写常量 | 两版 `global-key.txt` 或 `manifest.tsv` 另有一行不同 | 看过：各只差那一行 / 0 行 |
| L12：`runs/` 下的逐趟库没人再读 | 找到一个入口在导入之后还打开 `runs/*/fetched/…` 或 `library-for-peer` | 没找：只看了 213 份里最新一份 LOG 的生日（2026-10-14），读没读要查双机驱动的代码，归代码路子 |

没在副本里造推翻现象：派发写明只读，不起 cargo，造不了「改一份闭包文件看版本变不变」这类现象。能在盘上做的反推（找同版本下的 0、找同限时下的不同键、找交集）都做了，结果在上表。

## 6 没查到 / 判不了

- 崩溃库判法版本的输入：新布局（L2–L5）一条 `09` 都没写（各库表计数里没有 `09`，量的），旧布局 L1 的 `09` 只记文件名、不记摘要（`libs/library/09-*.txt`，7 份里 3 对逐行相同）。所以 R1 那次换版本是哪份文件变了，盘上判不了；R3、R4、R6 改用暂存树的 `git diff`，而「这一趟用的是哪棵树」取自路子三的 `entry-table.tsv`，我没另外核（推的）。
- crates 变异判法版本目录（L18）不存版本的输入；R13 那次 23404d01→37ab6ce3 是哪份脚本变了判不了。
- 整道全绿标记（L17）只存 `input_file_count`，QEMU 10-01 → 10-12 两份之间变了什么判不了；LKMM 用 `fingerprints.txt` 的树补了（R19），QEMU 那两份没有对应的树。
- 第二台上的库（日志写作「第二台的目录/mutation-kept/records/singlefs-mutation-rows/…」与双机驱动在第二台建的崩溃库）没看：不登第二台。
- L7、L8、L11 的子库没整解；L8 顶层与 61 份子库的版本数没量。
- 文件型库（L18、L20、L24）「最近一次被读」没有痕迹可查（atime 被整片刷过，§1），只给了最近一次写与最近一趟读它的日志。
- 形式化 10-12 两趟与 QEMU 各趟的门禁日志已被 `>` 覆盖，只剩库与标记。
- 广谱变异结论表里 `commit` 一列进不进沿用判据：盘上只看得到它被记下（X34），用不用归代码路子核。
- `/tmp/claude-1000/-home-fy5090-code-singlefs/8ff4a20a-*/scratchpad/verdict`、`verdict-defs` 两个目录：名字像，没打开（是别的会话的草稿，不是门禁的库，推的）。
- 研究侧 experiment-replay（L26）从没落盘，它的按行键在盘上无从核。

## 7 射程外（撞见了另列）

- 轻型与重型共用一份库：崩溃放量快档每趟一把新 GPU 版本，写进 L2、L4、L5（§2 末段）；crates 变异 --quick 与 two-stage-validate 那几趟占掉 L18「只留 3 个」的名额，现在 3 个目录里 3a63d840、d0ef8d8c 是轻型那几趟建的（R14），重型 37ab6ce3 再来两把轻型就会被删（推的，按 `crates-mutation-rows.py:84` 的删法）。按判据这是 ⑧，落点同 X27，但触发它的是轻型。
- 形式化轻型 `--quick`（`merge-2026-10-14/formal-quick-after-writer.log`）与重型共用 `.git/singlefs-formal-verdicts`，proof 那一项 10-14 的新键大概是它写的（推的）。
- 只管排先后、不管判定的几份：`.git/singlefs-mutation-row-timings/`（按行原文摘要）、`~/.local/share/singlefs/memory-peaks.tsv`（按命令记内存峰值）、`~/.local/share/singlefs/cross-run-cache/`（编译目录与开跑时刻）、`~/.local/share/singlefs/qemu/singlefs-build-stamp.txt`、`~/.local/share/singlefs/formal-tools/`（4.1G，构建产物）。
- `/tmp/singlefs-c790-slices-observer-*-<进程号>` 50 份单测按进程号建的库没人收（7.4M）。
- preflight 的 `inputs-changed`：没查。
- 盘的占用：`runs/` 89G（L12）、`broad-mutation/` 17G（大头是 `target`）、`cross-run-cache` 现在是空的。

## 8 没做什么

- 没改仓里任何文件；没起 cargo、门禁、测试二进制，没跑重型与轻型测试；没打开任何 RocksDB（只用只读解析器读文件），没写源码摘要缓存。
- 没读代码的键算法（归别的路子）；表里引的代码行号只为指明盘上看到的那一样是哪一行写出来的，都现取过（`grep -n` / `awk 'NR==…'`），X9 的 `crash_amplification.rs:624`–733 与 X14 的双机驱动行号没现读，已标。
- 没登第二台。
- 没在副本里造推翻现象（只读）。
- 草稿目录里只有日志摘录、解析脚本与解析结果（`libs/`、`full/`、`fullroot/`、`tools/`），没有仓副本与编译目录。
- 没写「要主 agent 跑的命令」：这一轮要的都是读盘，读完了。
