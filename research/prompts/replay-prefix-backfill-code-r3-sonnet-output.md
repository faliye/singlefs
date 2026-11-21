# 重放前缀代码轮第三轮：正推腿（Sonnet）报告

日期 2026-11-19。立场：正推。方案按 `crates/` 今天的实现来谈：下面每个「应该是什么」都从 `crates/` 的源码与已定条款推出，再与被判对象逐格比；一样写一样，不一样把两边整行并排抄出来。引文都是现读现抄，行号取被引文件自己的。

被判对象：第二轮判决第三节第 1 到 5 条的改法合进主工作区之后的样子（`src/README.md`、`src/GPU_GC_PROOFS.md`、必红表 `verus-must-red-worlds.tsv`、`.claude/gate.d/verus-pins.tsv` 三行注释）。开工快照 `research/prompts/replay-prefix-backfill-code-r3-snapshot/`。下面是交回前现跑的原样输出，八份文件与开工快照逐份相同：

```
$ sha256sum -c research/prompts/replay-prefix-backfill-code-r3-snapshot/crates-sha256.txt
crates/singlefs-replay-prefix/src/README.md: OK
crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md: OK
crates/singlefs-replay-prefix/verus-must-red-worlds.tsv: OK
crates/singlefs-replay-prefix/src/file_range_proofs.rs: OK
crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs: OK
crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs: OK
crates/singlefs-replay-prefix/src/partial_write_proofs.rs: OK
$ sha256sum -c research/prompts/replay-prefix-backfill-code-r3-snapshot/governance-sha256.txt
.claude/gate.d/verus-pins.tsv: OK
```

## 零、复跑命令与模型文件

复跑命令（整条交任务服务；在仓根起；`-run` 收尾的运行目录由外面这层脚本先删后建，只认工作根下 `-run` 收尾的目录，别的一律不删、退 2）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/replay-prefix-backfill-code-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r3-forward/measure-run /home/fy5090/code/singlefs/research/prompts/replay-prefix-backfill-code-r3-sonnet-model
```

模型目录 `research/prompts/replay-prefix-backfill-code-r3-sonnet-model/`，每个文件的 sha256（交回前现算）：

```
$ cd research/prompts/replay-prefix-backfill-code-r3-sonnet-model && sha256sum tools/* out/*
0adbb0419886c2baeccddc41f41b6dba967a038923e030d90641862f9a176a24  tools/check_commands.py
e7081aaef91f23315460ee43fc20a32eab42df5895a5c5061aa570fa9967c87b  tools/check_hits.py
87cf5d4ed0ed6a2cd3b666dcd49a8f07291d51f0a65321accf042db8d1d76dfa  tools/measure_rows.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
69237cb035a1af250bc9893bb5c5506408d50cf1b0cb79e3949e12302bbdd747  tools/rerun.sh
2e53d9a9181f6bee7d2eb26d25f78d3ed31075a506c02225c842c4b6b98fd71e  out/check-commands.txt
81194b996151e9647d4bf98e1aa3e1b823d700bab34ee131042d2c139df47411  out/check-hits.txt
```

- `tools/measure_rows.py`：把必红表指定行的原文换成替换文，在 crate 拷贝里直接起 Verus（参数照表头文字：`--crate-type=lib`、`--edition`、`--crate-name`、`--num-threads`、`--output-json`、`--rlimit`、`src/lib.rs`），数证明错、取落点，与表里第五、六段并排打出。判法照表头自己写，与门禁 formal-verification 不共用代码。
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只改了 `rerun_steps` 与行名单：先照开工快照核七份文件的 sha256，再起一趟基线（不改任何东西，要 0 errors）与 20 行各一趟（新加 13 行加条数改过的 7 行），整 crate 验证。
- `tools/check_hits.py`、`tools/check_commands.py`：读码核（13 行原文在目标文件里命中几次；说明文件里复跑命令点名的测试全名是不是 `#[test]`、范围数对不对），输出在 `out/`。

探针的内存与时间：没量。这一腿没有起过 Verus（子 agent 不跑项目代码）；`measure_rows.py` 与 `rerun.sh` 只用假 Verus（只吐 JSON 结局的脚本）接过线：20 行原文都恰好命中一次、基线与逐行的读数能打出。真 Verus 一趟的挂钟与内存待主 agent 跑完读 `seconds=` 字段与任务服务的内存行；参照量（推的，不是这条探针的）：主 agent 跑的 formal-verification 那一趟里被强制现判的必红行是 3.4 秒。

自证：`bash <模型目录>/tools/rerun.sh --selftest` 待主 agent 跑（命令在「要主 agent 跑的命令」一节）；`bash -n`、`python3 -m py_compile` 与 `measure_rows.py --selftest`（纯文本解析与假 Verus）本腿跑过，过。

## 读法

- 「一致」「冲突」「规则没说」三种判定；每条结论后写「什么现象会推翻它」。
- Verus 报几条证明错不是读码能定的数：Verus 默认每个函数先找第一处、再多问至多 2 次，同一处假前提还可能让后面的断言空真（推的）。这一点的依据是 Verus 自己的帮助串（二进制里的原文）：

```
$ grep -a -o 'multiple-errors[^\x00]\{0,170\}' <Verus 安装目录>/rust_verify | head -1
multiple-errorsIf 0, look for at most one error per function; if > 0, always find first error in function and make extra queries to find more errors (default: 2)INTEGERno-vstdDo not loa
```

所以对条数这一格，读码只给「几处独立的红点」与「每函数至多 3 条」的上下界；确切的数留给 `measure_rows.py` 的实量。落点（哪个函数报错）读码可推，推不出的单列。

## 一、第二轮判决第三节第 1 到 5 条：改法做到没有

判决原文在 `research/prompts/replay-prefix-backfill-code-r2-main-verification.md` 第三节，逐条对照。

### 第 1 条：README 义务数与文件表，做到（一致）

判决要求：「1. `src/README.md:109`、`:111` 的义务数照 `file_range_proofs.rs` 现有被验函数改；文件表 `file_range_proofs.rs` 那一行补精确交集长度。量过（主 agent 现读）。」（`research/prompts/replay-prefix-backfill-code-r2-main-verification.md:35`）

今天的两行与文件表那一行：

- 「文件区间模块共 19 项 Verus 义务（按门禁 formal-verification 的 verus-proof 格成项：10 个 exec fn、9 个 proof fn，没有带 decreases 的 spec fn 与带值的 const）：9 个产品入口（文件表那一行列的）、8 条引理、2 个无前提见证（`witness_file_range_boundaries`、`witness_intersection_length_partitions`）。必须被拒绝的变异是必红表里第二段为 `crates/singlefs-replay-prefix/src/file_range_proofs.rs` 的各行。普通 Rust 测试另外枚举小区间的实际字节集、逐批释放的单元集，并检查全宽端点。」（`crates/singlefs-replay-prefix/src/README.md:109`）
- 「文件区间模块的 19 项义务里，管 u64 边界与声明长度的 4 项：」（`crates/singlefs-replay-prefix/src/README.md:111`）
- 「精确交集长度（`range_intersection_length`，单元分界与请求分界两侧长度相加等于整体）」（`crates/singlefs-replay-prefix/src/README.md:18`）

正推：「成项」的规则在门禁 formal-verification 的 verus-proof 格，「按全路径：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，带值的 const 与 assume_specification 各成一项」（`.claude/gate.d/formal-verification.sh:18`）。照它数 `file_range_proofs.rs`：

```
$ grep -cE '^pub fn ' crates/singlefs-replay-prefix/src/file_range_proofs.rs
10
$ grep -cE '^pub proof fn ' crates/singlefs-replay-prefix/src/file_range_proofs.rs
9
$ grep -cE '^pub proof fn lemma_' crates/singlefs-replay-prefix/src/file_range_proofs.rs
8
$ grep -nE '^pub (proof )?fn witness_' crates/singlefs-replay-prefix/src/file_range_proofs.rs
277:pub fn witness_file_range_boundaries() {
369:pub proof fn witness_intersection_length_partitions()
$ grep -nE '^pub (open )?spec fn|decreases|^pub const|^const |assume_specification' crates/singlefs-replay-prefix/src/file_range_proofs.rs | cut -c1-70
7:pub open spec fn covering_units(size: u64, capacity: u64) -> int {
87:pub open spec fn payload_length_for_size(start: u64, size: u64, cap
93:pub open spec fn payload_byte_is_in_file(start: u64, size: u64, cap
152:pub open spec fn in_both_ranges(byte: int, write_start: u64, write
157:pub open spec fn range_intersection_length(
207:pub open spec fn byte_in_write_and_indexed_unit(byte: int, write_s
```

10 个 exec fn（9 个产品入口加 `witness_file_range_boundaries`）、9 个 proof fn（8 条 `lemma_` 加 `witness_intersection_length_partitions`）、没有带 decreases 的 spec fn、没有带值的 const，合计 19，与 :109 的拆法（9 个产品入口、8 条引理、2 个无前提见证）逐项相同；:111 说的 4 项（`checked_write_end`、`payload_length_from_start`、两条引理）是这 19 项的子集。独立的第二条路：判决第二节第 9 条引的那一趟 formal-verification 的 log（主 agent 交的，本腿只读 `task-server/task log`）里列出的 `file_range_proofs` 项也是 19 个：

```
$ grep -o 'proof:singlefs_replay_prefix::file_range_proofs::[A-Za-z_]*' <判决第二节第 9 条引的那一趟 formal-verification 的 task log> | sort -u | wc -l
19
$ grep -n "成项 198 个" <同一份 log>
1665:  ✓ Verus 逐项验 crates/singlefs-replay-prefix：成项 198 个（proof fn 116、exec fn 50、带 decreases 的 spec fn 16、derive(Clone) 生成的 clone 13、带值的 const 3），各项 verified 之和 214、0 errors（登记 214）；不成项的 spec fn 157 个（没有证明义务，随用到它们的项进键）
```

判定：一致。推翻它的现象：门禁 verus-proof 格的成项规则改了，或 `file_range_proofs.rs` 又加减了 exec fn 与 proof fn 而说明文件没回头改。

同一份 README 里同类的数（本轮攻 1 点名的几处），逐处判：

- :117「另有 7 个必红变异」、:126「本轮新增 6 个必红变异」、:148「11 个必红变异」：表里对应行段数得出，一致（下面输出的前三段依次是 291–297 共 7 行、298–303 共 6 行、304–314 共 11 行，第四段 279–285 是 :138 点名的那 7 行）。

```
$ awk -F'\t' '!/^#/ && NR>=291 && NR<=297 {print NR": "$1}' crates/singlefs-replay-prefix/verus-must-red-worlds.tsv
291: write-end-overflow-not-saturated
292: write-end-max-is-representable
293: payload-past-eof-is-empty
294: payload-length-is-relative-to-start
295: payload-length-is-capped
296: payload-growth-requires-size-order
297: payload-prefix-requires-truncation
$ awk ... NR>=298 && NR<=303
298: unit-count-empty-is-zero
299: unit-count-does-not-drop-the-last-unit
300: unit-count-does-not-add-an-extra-unit
301: sequential-write-empty-keeps-one-unit
302: rounded-size-refuses-overflow
303: unit-count-monotonicity-needs-ordered-sizes
$ awk ... NR>=304 && NR<=314
304: partial-old-unit-must-read-uncovered-bytes
305: partial-touching-endpoints-are-not-written
306: partial-equal-range-covers-whole-unit
307: partial-new-gap-does-not-read-residue
308: partial-new-partial-unit-reads-residue
309: partial-rewrite-at-old-end-includes-partial-tail
310: partial-rewrite-does-not-skip-partial-tail
311: partial-after-count-keeps-old-suffix
312: partial-whole-coverage-needs-nonempty-unit
313: partial-byte-index-is-not-the-next-unit
314: partial-byte-index-needs-positive-capacity
$ awk ... NR>=279 && NR<=285
279: unlink-plan-equality-is-final
280: unlink-plan-batch-cuts-one-more
281: unlink-plan-continues-from-one-less
282: unlink-final-batch-keeps-one
283: unlink-size-adds-instead-of-multiplying
284: deletion-batch-carries-the-cut-leaf
285: lower-node-exists-one-past-the-end
```

- :119 与 :109 并排，冲突（词义）：
  - 「文件区间模块共 19 项 Verus 义务（按门禁 formal-verification 的 verus-proof 格成项：10 个 exec fn、9 个 proof fn，没有带 decreases 的 spec fn 与带值的 const）：9 个产品入口（文件表那一行列的）、8 条引理、2 个无前提见证（`witness_file_range_boundaries`、`witness_intersection_length_partitions`）。必须被拒绝的变异是必红表里第二段为 `crates/singlefs-replay-prefix/src/file_range_proofs.rs` 的各行。普通 Rust 测试另外枚举小区间的实际字节集、逐批释放的单元集，并检查全宽端点。」（`crates/singlefs-replay-prefix/src/README.md:109`）
  - 「同日继续补齐单元取整：新增三个产品函数与一个单调引理，加上取整边界的 nonlinear_arith 子证明，共 5 条义务。」（`crates/singlefs-replay-prefix/src/README.md:119`）
  - 照 :109 的成项规则，:119 说的「三个产品函数与一个单调引理」是 4 项（`units_covering_size`、`units_for_sequential_write`、`rounded_size_fits`、`lemma_covering_units_are_monotone`，都在上面 19 项里）；「共 5 条义务」是 verified 数（4 项加取整边界的 nonlinear_arith 子证明）。`.claude/gate.d/verus-pins.tsv` 第 40 行的注释也是这个口径。同一份 README 里「义务」一词在 :109 是成项数、在 :119 是 verified 数。:148 同理：「本模块新增 6 个被验函数（含一条 nonlinear_arith 子证明，共 7 条义务）与 11 个必红变异。证明依赖输入区间有效：构造后的计划字段须保持一致，单元端点的乘加不溢出；不宣称已经证明公开字段被任意改写后的计划、区间地址转换、五类来源对应的真实 I/O 或整个发布事务。核测试枚举实际覆盖字节并确认五类都出现，core 包装测试单独核对公开枚举的读旧数据/读残留/补零标记。」（`crates/singlefs-replay-prefix/src/README.md:148`） 6 个是成项数，7 条是 verified 数。
  - 这两句还带历史式说法（「同日继续补齐」「本轮新增」），而 「代码与脚本的注释、说明只写现状，不写历史」（`.claude/agent-common.md:46`） 的射程字面含 `crates/` 下注释「与这几处的 `README.md`」。规则没说这份 README 算不算射程内的存量，记「规则没说」。
- :138 冲突（数过时）：「必红世界表另登记 7 个变异」（`crates/singlefs-replay-prefix/src/README.md:138`）；表里第二段是 `unlink_proofs.rs` 的行是 12 行（本节末尾计数输出的第一条），句子里点名的 7 个是 279–285 那 7 行（上面输出的第四段），286–290 的 5 行是后来加的、没进这个数。
- :165 冲突（清单不全）：「必红表里这一块有各函数的契约反例」（`crates/singlefs-replay-prefix/src/README.md:165`） 之后按类列出：契约反例 5 加 3、删前提 6、改定义 2、改另两条引理结论 2、组合性质另 3，共 21 行；表里第一段以 `overwrite-` 起头、第二段是 `partial_write_proofs.rs` 的行是 23 行（本节末尾计数输出的第三条），多出的 2 行是这一轮新加的 510、511（第一条引理的 `ensures` 本身），清单没列。

```
$ awk -F'\t' '$2 ~ /unlink_proofs.rs/ {n++} END{print n}' crates/singlefs-replay-prefix/verus-must-red-worlds.tsv
12
$ awk -F'\t' '$2 ~ /file_range_proofs.rs/ {n++} END{print n}'
42
$ awk 表里第一段以 overwrite- 起头且第二段是 partial_write_proofs.rs 的行
23
```

这几处都不在第三节第 1 条的字面里（它只点 :109、:111 与文件表），所以第 1 条的判定仍是「做到」；它们是同类遗留，交主 agent 判要不要收。推翻「:119 与 :109 词义冲突」的现象：主 agent 认定「义务」一词在这份 README 里本来就两种口径并存、各段自己写明。

### 第 2 条：复跑命令、卡的说法、NOT_RUN 的读法，复跑命令做到，两处说法有出入

判决要求：「2. `src/GPU_GC_PROOFS.md:104`–`:112` 五条复跑命令改成点名测试全名加 `--exact`、范围写测试数；`:97` 改成任务不拿卡、真卡那一条交 GPU 服务代跑（接口待定）；`NOT_RUN_NO_GPU_ADAPTER` 怎么读的那句现状说明并进来。量过（主 agent 现读）。」（`research/prompts/replay-prefix-backfill-code-r2-main-verification.md:36`）

**命令点名**：说明文件里的 4 条 `cargo test`，范围声明与命令形状：

- 「范围=4 个测试」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:104`）；「范围=5 个测试」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:106`）；「范围=82 个测试」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:108`）；「范围=2 个测试」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:193`）。
- 读码核（`tools/check_commands.py`，输出原样）：4 个测试目标里点名的全名都是 `#[test]` 函数、没有重名、数 = 声明的范围数（4、5、82、2）；82 条恰是 `singlefs-background-gpu` 四个主机模块里全部的 `#[test]`（7 + 36 + 23 + 16），没有漏列：

```
SKIP(not cargo test): bash .claude/gate.d/formal-verification.sh --item must-red:gpu-score-binary-skips-entry --
SKIP(not cargo test): bash .claude/gate.d/formal-verification.sh --item must-red:gpu-gc-G01 --item must-red:gpu-
package=singlefs-replay-prefix target=gpu_gc_arithmetic_matches_wide_integer_reference names=4 declared_scope=4 found_as_test_fn=4 missing=[] duplicates=[]
  test_fns_in_file=4 unlisted=[]
package=singlefs-replay-prefix target=gpu_gc_capacity_matches_independent_reference names=5 declared_scope=5 found_as_test_fn=5 missing=[] duplicates=[]
  test_fns_in_file=5 unlisted=[]
package=singlefs-background-gpu target=--lib names=82 declared_scope=82 found_as_test_fn=82 missing=[] duplicates=[]
  module two_kernel_reconciliation: listed=7 test_fns_in_file=7 unlisted=[]
  module memory_limit_and_chunks: listed=36 test_fns_in_file=36 unlisted=[]
  module live_slot_kernels: listed=23 test_fns_in_file=23 unlisted=[]
  module region_record_kernels: listed=16 test_fns_in_file=16 unlisted=[]
package=singlefs-background-gpu target=gpu_gc_formal_contracts_through_the_host names=2 declared_scope=2 found_as_test_fn=2 missing=[] duplicates=[]
  test_fns_in_file=2 unlisted=[]
```

- 命令形状照 「点名测试函数写 `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <目标> 里的 <N> 条测试; 范围=<N> 个测试; 为什么=…'」（`.claude/agent-common.md:65`）：四段齐、种类 target、点名全名加 `--exact`、范围数 = 名字数，一致。服务与钩子是不是真收，要主 agent 原样交一次才知道（命令在「要主 agent 跑的命令」一节）。必红两条命令点名的 16 个 `must-red:<名>` 在表里各命中一次（本腿 awk 核过，16 个都是 1）。

**:97 卡的说法**：「统一任务服务只调配 CPU，内存、线程与机器由它定，任务不拿卡」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:97`）。规则：「统一任务服务只调配 CPU：经 `task-server/task submit -- <命令>` 交的任务只跑录入、往判定库写待判行、读结论，不领卡、不分卡、不清场」（`.claude/rules/verification.md:50`）。一致。

**真卡那一节**（:198、:203）：「接口定下之前这两组没有可交的复跑命令」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:198`）；12 个全名在两个测试目标里各 6 个 `#[test]`（本腿数过：两个文件各 6 个 `#[test]`，名字与文里逐个相同）；`tests-on-a-real-gpu` 特性在 `crates/singlefs-background-gpu-tests/Cargo.toml` 里是这两个目标的 `required-features`。没有会被拒的可照抄命令。一致。

**NOT_RUN 那几句**（:203）逐句对：

- 「没拿到卡时每条打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`」：「format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:751`），一致。
- 「拿到卡的打 `GPUS_HANDED_OVER_BY_THE_TASK_SERVER test=<用例名> gpus=…`」：「"GPUS_HANDED_OVER_BY_THE_TASK_SERVER test={test_name} gpus={}"」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:758`），一致。
- 「cargo 报的通过数不说明上过卡」：对 「那一行是 `ok`（带不带 `GPU_DEVICE_LOST` 都照这一条往下判），输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它」（`.claude/rules/mutation-sampling.md:63`）（那一行是 ok、输出里又有 NOT_RUN 点名它，记「本次未跑」），一致。

出入两处（规则与实现两边并排）：

1. 说明文件写的是「没有 `NOT_RUN_NO_GPU_ADAPTER` 与 `GPU_DEVICE_LOST` 行，才是在卡上判过」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:203`），即要求日志里没有 `GPU_DEVICE_LOST` 行才算上过卡，而这两个测试包里没有打这一行的地方（零命中）。这一行的出处是 「`GPU_DEVICE_LOST test=<当前线程名> card=<卡名> reason=<一句>`」（`.claude/rules/mutation-sampling.md:61`），那是变异判法读的行，`crates/` 里只有 `singlefs-verification` 两份源码打它：

```
$ grep -rln 'GPU_DEVICE_LOST' crates --include=*.rs
crates/singlefs-verification/src/judging_units_entries.rs
crates/singlefs-verification/src/gpu_memory_ledger.rs
$ grep -rc 'GPU_DEVICE_LOST' crates/singlefs-background-gpu crates/singlefs-background-gpu-tests --include=*.rs | grep -v ':0$'
（无输出；grep 退出码 1）
```

   判：:203 的这半句对这两个测试目标是真话但空转（没有打印点）；判不出它是照变异判法顺手搬来的，还是后台 GPU 包以后要加。记「规则没说」。推翻它的现象：这两个包里加了打这一行的代码，或实跑日志里出现这一行。
2. :198 写「没有可交的复跑命令」，门禁里已有一格会起这两个目标：「product-gpu-real-card-tests 起产品 GPU 包的测试包（singlefs-background-gpu-tests）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试」（`.claude/gate.d/crash-point-replay.sh:12`）。窄读（没有点名测试函数的复跑命令）成立，宽读（没有任何跑法）不成立；这一格归提交时那一趟，说明文件没提。判：冲突（宽读）。推翻它的现象：主 agent 认定这一格不算「复跑命令」。

另记一处不判：「GPUS_HANDED_OVER_BY_THE_TASK_SERVER」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:758`） 的行名说卡由任务服务交，而 :97 与 `.claude/rules/verification.md` 里同一条说任务不拿卡；两句同在一份文件里。这两个包别的会话在拆，本轮只判今天的树，不判哪边是终态。

### 第 3 条：verus-pins 三行注释与必红表第 365 行，做到（一致）

判决要求：「3. `.claude/gate.d/verus-pins.tsv:47`、`:48`、`:49` 与必红表第 365 行注释照现状改：G30 没有调用点、义务数 214、去掉日期。登记表与必红表同一次改。」（`research/prompts/replay-prefix-backfill-code-r2-main-verification.md:37`）

今天的四行与必红表那一行：

- 「# GPU GC G21–G30：十个核（G30 没有产品调用点）与一个边界见证；模块现量 11 条义务、0 errors。」（`.claude/gate.d/verus-pins.tsv:47`）
- 「# 固定长度覆盖写组合：分段等价、包含写折入请求与构造见证共三条；整 crate 的义务数见 crate 行。」（`.claude/gate.d/verus-pins.tsv:48`）
- 「# 交集长度组合：单元分割、请求分割与无前提见证共三条，整 crate 实测 214 verified、0 errors。」（`.claude/gate.d/verus-pins.tsv:49`）
- 「crate	crates/singlefs-replay-prefix	214」（`.claude/gate.d/verus-pins.tsv:50`）
- 「# G21–G30：容量、记账、派发与映射表核；G21–G29 产品直接调用，G30 产品没有调用点。」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:365`）

正推：

- 「十个核与一个边界见证」「模块现量 11 条义务」：`gpu_gc_capacity_proofs.rs` 的 `pub fn` 是 11 个，其中 `extended_gpu_chunk_input`（G30）一个、`witness_gpu_capacity_and_accounting_boundaries` 一个，前 9 个是 G21–G29：

```
$ grep -c '^pub fn ' crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs
11
$ grep -nE '^pub fn (extended_gpu_chunk_input|witness_)' crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs | cut -c1-110
183:pub fn extended_gpu_chunk_input(previous: u64, additional: u64, entries: u64, capacity: &GpuChunkCapacity)
198:pub fn witness_gpu_capacity_and_accounting_boundaries() {
$ grep -c '^pub proof fn \|^pub fn ' crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs
11
```

- 「G30 没有产品调用点」：「G30：向当前批追加输入」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:181`）；全仓 `crates/` 下 `extended_gpu_chunk_input` 只有核自己的定义与一处测试引用（下面第一个输出）；产品的后台 GPU 包 `singlefs-background-gpu/src` 里 G21–G29 的 9 个名字各出现在 1 份源文件、G30 的名字 0 份（下面第二个输出）。

```
$ grep -rn 'extended_gpu_chunk_input' crates --include=*.rs（不含 gpu_gc_capacity_proofs.rs 自己）
crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs:85:                                extended_gpu_chunk_input(input, additional, entries, &capacity),
```

```
$ for n in <G21 到 G30 的十个函数名>; do printf '%s ' $n; grep -rlw $n crates/singlefs-background-gpu/src | wc -l; done
gpu_chunk_fits 1
largest_gpu_piece_input 1
tightened_gpu_memory_limit 1
checked_gpu_memory_hold 1
checked_gpu_memory_release 1
gpu_bytes_after_submission_wait 1
gpu_memory_cap_after_allocation_failure 1
gpu_workgroups_for_entries 1
checked_gpu_mapping_table_bytes 1
extended_gpu_chunk_input 0
```

- 「214」：与 :50 的登记数相同，与主 agent 那一趟 log 的 verified 之和相同（上面「第 1 条」里第二个输出的 `各项 verified 之和 214、0 errors（登记 214）`）。:48 不再写数、指向 crate 行，:49 去掉了日期。必红表 365 行与 「`extended_gpu_chunk_input` 产品此刻没有调用点」（`crates/singlefs-replay-prefix/src/README.md:16`）、「//! GPU GC 的切批准入、最大分片、显存账本和派发尺寸；G21–G29 由产品调用同一份 exec 核，G30 产品此刻没有调用点」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:1`） 同口径。

判定：一致。不在判决字面里、但同在 `verus-pins.tsv` 的存量：「# 2026-10-29 部分写计划核：三个产品函数与两个区间语义引理，143 + 5 = 148（整 crate --no-cheating --rlimit 10 实测）。」（`.claude/gate.d/verus-pins.tsv:41`）（第 40 至 46 行都带日期或改动经过，判决第 6 条的欠账是这一类，还没做，本轮不判）。推翻本条的现象：`gpu_gc_capacity_proofs.rs` 以外出现 `extended_gpu_chunk_input` 的调用，或 crate 行的数与整 crate 的 verified 之和又对不上。

### 第 4 条：必红表，做到（一致）

判决要求：「4. 必红表：`file_range_proofs.rs:171`、`:221` 的 `None` 支各补一行必红；第二节第 5 条两条腿列的其余 ensures 逐处补或写明补不出的理由；7 行条数照…的实测改；第 394–409 行补自己的注释行。**零轮**：新必红行由 formal-verification 带 `--item` 证红，不改产品代码，不做 A/B。」（`research/prompts/replay-prefix-backfill-code-r2-main-verification.md:38`）

- **两处 `None` 支各一行**：「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`） 与 「&&& range_intersection_length(write_start as int, write_end as int, unit_index * capacity, unit_index * capacity + capacity) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:221`） 各有一行：表第 414、415 行 `write-intersection-empty-has-zero-length`、`indexed-intersection-empty-has-zero-length`，原文都恰好命中一次（`out/check-hits.txt`）。逐行推演见下一节。
- **两条腿列的其余 ensures**：判决第二节第 5 条点名的与今天的行一一对上：候选窗口引理两条对 503、504；排序二分定理三条（`gpu_gc_scoring_proofs.rs` 第 214 至 216 行）对 505、506、507；`gpu_gc_scoring_proofs.rs` 第 117 行 `first <= past` 对 503；覆盖写第一条引理两条对 510、511；`witness_file_range_boundaries` 对 502；`gpu_gc_proofs.rs` 第 345、346 行对 508、509。没有留「补不出」的。
- **7 行条数**：表里今天的第五段与那一趟 log 里的实测并排（log 是判决第二节第 9 条引的那一趟，本腿读 `task-server/task log`；表头说条数只报不判）：

```
$ awk -F'\t' (条数改过的 7 行，现在的第五段与第六段)
324	indexed-intersection-does-not-drop-written-bytes	proof_errors=2	landing=checked_bytes_in_indexed_unit
424	overwrite-exact-needs-fit	proof_errors=2	landing=lemma_overwrite_changes_exactly_the_requested_bytes
431	gpu-gc-G01-confirmation-never-asks-cpu	proof_errors=2	landing=reconcile_gpu_entry
432	gpu-gc-G01-cpu-judgment-restarts-the-count	proof_errors=2	landing=reconcile_gpu_entry
433	gpu-gc-G01-second-disagreement-asks-cpu	proof_errors=2	landing=reconcile_gpu_entry
434	gpu-gc-G01-threshold-constant	proof_errors=2	landing=reconcile_gpu_entry
466	gpu-gc-G18-short-remainder-is-not-overrun	proof_errors=2	landing=next_gpu_record_slice_end
492	gpu-gc-G28-group-count-is-not-truncated	proof_errors=3	landing=gpu_workgroups_for_entries
$ grep -n '条数与登记' <判决第二节第 9 条引的那一趟 formal-verification 的 task log>（每行截到「条数只报」）
2595: indexed-intersection-does-not-drop-written-bytes | 落点 checked_bytes_in_indexed_unit；证明错 2 条（条数与登记的 proof_errors=1 不同
2615: overwrite-exact-needs-fit | 落点 lemma_overwrite_changes_exactly_the_requested_bytes；证明错 2 条（条数与登记的 proof_errors=1 不同
2620: gpu-gc-G01-confirmation-never-asks-cpu | 落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同
2621: gpu-gc-G01-cpu-judgment-restarts-the-count | 落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同
2622: gpu-gc-G01-second-disagreement-asks-cpu | 落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同
2623: gpu-gc-G01-threshold-constant | 落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同
2661: gpu-gc-G18-short-remainder-is-not-overrun | 落点 next_gpu_record_slice_end；证明错 2 条（条数与登记的 proof_errors=1 不同
2684: gpu-gc-G28-group-count-is-not-truncated | 落点 gpu_workgroups_for_entries；证明错 3 条（条数与登记的 proof_errors=1 不同
```

  7 行（`overwrite-exact-needs-fit`、G01 四行、G18 一行、G28 一行）的今天的值 2、2、2、2、2、2、3 与 log 的实测 2、2、2、2、2、2、3 逐行相同；`indexed-intersection-does-not-drop-written-bytes`（表第 324 行）是 log 里另一处登记 1 实测 2 的行，今天的表也已是 2。落点没动。一致。
- **注释行**：第 394 至 409 行原先挂在打分那段注释底下，今天各有自己的注释行：「# 覆盖写的组合性质：两条组合引理与见证的契约反例、后缀与后写偏移错位、两条删前提、见证字节。」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:395`）、「# 精确交集长度：两条分割引理与见证的契约反例、两条删分界前提、两个入口 Some 支的长度加一、None 支的长度改成非零、函数体漏掉一字节的交集、见证值。」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:405`）。核了两段注释说的类别与下面的行：395 段 396–403 共 8 行 = 契约反例 3 + 后缀与后写偏移错位 2 + 两条删前提 2 + 见证字节 1；405 段 406–416 共 11 行 = 契约反例 3 + 两条删分界前提 2 + Some 支长度加一 2 + 见证值 1 + None 支非零 2 + 漏一字节 1，都与注释一致。:501 那一段 「# 别的行没有单独罩着的 ensures 条与见证：候选窗口引理两条、排序二分定理三条、覆盖写第一条引理两条（各改函数体或 ensures 本身一处）；协议与打分见证的两个具体值各改错一个；文件区间见证注入不可能断言。」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:501`） 之下 502–511 共 10 行，与注释列的类别一致。

判定：一致。推翻本条的现象：主 agent 复跑 `measure_rows.py` 时 20 行里有落点不同的，或表里某行的原文不再恰好命中一次。

### 第 5 条：G11 的「定义自检」，做到（一致，宽读），另有一格同类嫌疑

判决要求：「G11 的说明文件补…那一标」（`research/prompts/replay-prefix-backfill-code-r2-main-verification.md:39`）

今天：「模型，定义自检（三条 ensures 是 `record_contribution` 定义的展开）」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:42`）；图例 「“定义自检”表示 `ensures` 是同一文件里 spec 函数定义的展开，只钉住那份定义，对产品代码没有约束」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:28`）；:67 加了一句：「G11 的三条 `ensures` 同样是 `record_contribution`（连同它用的 `overlap_slots`）定义的展开」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:67`）。

正推：G11 的引理在 「pub proof fn lemma_released_and_foreign_records_do_not_score(record: ScoringRecord, device: int, start: int, end: int)」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:191`），`record_contribution` 与 `overlap_slots` 都定义在同一个文件（第 157、126 行起），定义是：

- 「if record.released || record.device != device { 0 }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:158`）
- 「else { overlap_slots(record.start, record.end, start, end) }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:159`）

三条 `ensures`：第一条（已释放或异盘则贡献零）是第 158 行那一支原样展开；第三条（同盘活记录贡献恰为 `overlap_slots`）是第 159 行那一支原样展开；第二条（贡献为正当且仅当严格相交）是 `overlap_slots` 的定义（`begin < end` 才非零）加上 `requires` 里的 `record.start < record.end, start < end` 做算术得到的推论，不是逐字展开。图例后半句「只钉住那份定义，对产品代码没有约束」三条都成立：三条只出现 `record_contribution` 与 `overlap_slots`，没有产品函数。改坏定义会红的例子是必红表第 362 行：

```
362	gpu-gc-audit-score-capped-at-two	proof_errors=2	landing=lemma_released_and_foreign_records_do_not_score,lemma_search_prefix_matches_the_walk
```

判定：一致（宽读：「展开」对第二条是「定义加算术」）。推翻它的现象：主 agent 把「展开」限定成逐字展开，那第二条要改写成「定义的推论」。

同类嫌疑一格（规则没说，交主 agent）：G10 「模型，只证重数 2」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:41`） 没标定义自检。它的引理体只有 「reveal_with_fuel(record_slice_sum, 3);」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:187`），`ensures` 是递归定义 `record_slice_sum` 展开两步再合并，按图例字面也是「spec 函数定义的展开」。第二轮判决只点了 G11，没有点 G10。

## 二、新加 13 行必红：逐行正推「改坏这一处，Verus 在哪个函数报错、报几条」

### 方法与前提

对每一行：读被改的那一处（表第三、四段，原文都恰好命中一次，`out/check-hits.txt`）；推它让哪一条 `ensures`、断言或调用前提不成立；再推谁会看到改后的契约（调用方）；最后与表第五、六段对。三条前提（都是读码推的，不是量的）：

1. 一条 `ensures`、一处 `assert`、一处调用前提各是一次检查；同一条 `ensures` 里用 `&&&` 并起来的几个合取算一次（没有开 `--expand-errors`）。
2. 一个函数里 Verus 先找第一处错、再多问至多 2 次（上面「读法」的帮助串），所以一个函数至多报 3 条。
3. 调用处照被调函数的 `ensures`（改后的）往下推；被调函数自己那一处红了，调用处仍当它的 `ensures` 成立。

Verus 一侧的调用关系（只算 `verus!` 里的，`#[cfg(test)]` 里的普通 Rust 调用不进证明）：

```
$ grep -nE '(bytes_in_unit|checked_bytes_in_indexed_unit)\(' crates/singlefs-replay-prefix/src/file_range_proofs.rs | cut -c1-120
166:pub fn bytes_in_unit(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64)
212:pub fn checked_bytes_in_indexed_unit(write_start: u64, write_end: u64, unit_index: u64, capacity: u64)
240:            let intersection = bytes_in_unit(write_start, write_end, start, end);
282:    let _indexed_bytes = checked_bytes_in_indexed_unit(3, 7, 1, 4);
310:    let _adjacent = bytes_in_unit(0, 10, 10, 20);
430:                        checked_bytes_in_indexed_unit(0, u64::MAX, unit_index, capacity),
444:            checked_bytes_in_indexed_unit(u64::MAX - 1, u64::MAX, u64::MAX - 1, 1),
459:                        let actual = checked_bytes_in_indexed_unit(
593:                        match bytes_in_unit(write_start, write_end, unit_start, unit_end) {
621:            bytes_in_unit(0, u64::MAX, 0, u64::MAX),
625:            bytes_in_unit(u64::MAX - 1, u64::MAX, 1, u64::MAX),
$ grep -nE '(lemma_candidate_window_keeps_every_contribution|theorem_sorted_search_equals_full_record_walk)\(' crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs | cut -c1-120
107:pub proof fn lemma_candidate_window_keeps_every_contribution(
170:        lemma_candidate_window_keeps_every_contribution(records, device, start, end, first, past, visited - 1);
206:pub proof fn theorem_sorted_search_equals_full_record_walk(
223:        lemma_candidate_window_keeps_every_contribution(sorted, device, start, end, first, past, past);
264:    let window = theorem_sorted_search_equals_full_record_walk(original, sorted, 7, 10, 20);
269:    theorem_sorted_search_equals_full_record_walk(empty, empty, 7, 0, 1);
271:    let crossing_window = theorem_sorted_search_equals_full_record_walk(longest_crossing, longest_crossing, 7, 65535
$ grep -nE '(lemma_overwrite_changes_exactly_the_requested_bytes|witness_gpu_gc_protocol_and_score_premises|witness_file_range_boundaries)\(' crates/singlefs-replay-prefix/src/*.rs | cut -c1-140
crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:342:pub proof fn witness_gpu_gc_protocol_and_score_premises()
crates/singlefs-replay-prefix/src/file_range_proofs.rs:277:pub fn witness_file_range_boundaries() {
crates/singlefs-replay-prefix/src/file_range_proofs.rs:614:        witness_file_range_boundaries();
crates/singlefs-replay-prefix/src/partial_write_proofs.rs:120:pub proof fn lemma_overwrite_changes_exactly_the_requested_bytes(old: Seq<u8>,
crates/singlefs-replay-prefix/src/partial_write_proofs.rs:206:    lemma_overwrite_changes_exactly_the_requested_bytes(old, 1, seq![8u8, 9]);
crates/singlefs-replay-prefix/src/partial_write_proofs.rs:207:    lemma_overwrite_changes_exactly_the_requested_bytes(old, 4, Seq::<u8>::emp
```

即：`bytes_in_unit` 在证明里的调用方是 `checked_bytes_in_indexed_unit`（240 行）与 `witness_file_range_boundaries`（310 行）；`checked_bytes_in_indexed_unit` 的调用方只有 `witness_file_range_boundaries`（282 行）；`lemma_candidate_window_keeps_every_contribution` 的调用方是 `lemma_search_prefix_matches_the_walk`（170 行）与 `theorem_sorted_search_equals_full_record_walk`（223 行）；该定理的调用方只有打分见证；`witness_gpu_gc_protocol_and_score_premises` 没有调用方。

表里 13 行今天的第五、六段（原样）：

```
$ awk -F'\t' '行号 名 第五段 第六段' (新加 13 行，行号是 crates/singlefs-replay-prefix/verus-must-red-worlds.tsv 自己的)
414	write-intersection-empty-has-zero-length	proof_errors=2	landing=bytes_in_unit,checked_bytes_in_indexed_unit
415	indexed-intersection-empty-has-zero-length	proof_errors=1	landing=checked_bytes_in_indexed_unit
416	write-intersection-one-byte-is-not-dropped	proof_errors=1	landing=bytes_in_unit
502	file-range-witness-is-checked	proof_errors=1	landing=witness_file_range_boundaries
503	gpu-score-candidate-window-first-not-after-past	proof_errors=1	landing=lemma_candidate_window_keeps_every_contribution
504	gpu-score-candidate-window-holds-only-the-device	proof_errors=1	landing=lemma_candidate_window_keeps_every_contribution
505	gpu-score-theorem-first-is-the-candidate-lower-bound	proof_errors=2	landing=theorem_sorted_search_equals_full_record_walk
506	gpu-score-theorem-past-is-the-end-lower-bound	proof_errors=2	landing=theorem_sorted_search_equals_full_record_walk
507	gpu-score-theorem-window-may-be-empty	proof_errors=1	landing=theorem_sorted_search_equals_full_record_walk
508	gpu-gc-witness-record-slice-sum-value	proof_errors=1	landing=witness_gpu_gc_protocol_and_score_premises
509	gpu-gc-witness-first-disagreement-count-value	proof_errors=1	landing=witness_gpu_gc_protocol_and_score_premises
510	overwrite-exact-keeps-the-length	proof_errors=1	landing=lemma_overwrite_changes_exactly_the_requested_bytes
511	overwrite-exact-empty-request-keeps-the-file	proof_errors=1	landing=lemma_overwrite_changes_exactly_the_requested_bytes
```

交叉印证推法：表里已有的 `write-intersection-exact-length`（第 411 行，`bytes_in_unit` 的 Some 支长度加一，登记只落 `bytes_in_unit`）。按上面三条前提，调用方 `checked_bytes_in_indexed_unit` 会拿到改后的 Some 支 `length == R + 1`，再加上同一条 `ensures` 里的 `in_unit + length <= unit_end - unit_start`、`in_write + length <= write_end - write_start` 与两侧起点相等，线性算术上自相矛盾（长度至多是两区间的交，比 R 大 1 不可能），调用方的假设空真、不红，所以只落一个函数。下面第 414 行的 None 支没有这种自相矛盾，所以调用方会红。这是同一套推法对已有一行的预言，与表一致。

### 第 414 行 `write-intersection-empty-has-zero-length`（A）

- 改坏：「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`） 的 `== 0` 改 `== 1`。
- 推：`bytes_in_unit` 的 None 分支路径条件是 `!(from < until)`，按 「if intersection_start < intersection_end { intersection_end - intersection_start } else { 0 }」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:162`） 的 else 支 `range_intersection_length` 为 0，`0 == 1` 不成立，None 返回处红 1 条。调用方 `checked_bytes_in_indexed_unit` 在 None 支要证 :221 的 `== 0`，手里是改后的 `R(…, start, end) == 1` 加 「Some((start, end)) => start == unit_index * capacity && end == start + capacity,」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:199`），同一个项，红 1 条。见证的两处调用只用 Some 支，不红。
- 推得：落点 `bytes_in_unit`、`checked_bytes_in_indexed_unit`，共 2 条。表：`proof_errors=2`，`landing=bytes_in_unit,checked_bytes_in_indexed_unit`。**落点与条数都一致。**

### 第 415 行 `indexed-intersection-empty-has-zero-length`（B）

- 改坏：「&&& range_intersection_length(write_start as int, write_end as int, unit_index * capacity, unit_index * capacity + capacity) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:221`） 的 `== 0` 改 `== 1`。
- 推：`checked_bytes_in_indexed_unit` 走 Some(None) 时，被调的 `bytes_in_unit`（没改）给 `R == 0`，目标 `R == 1`，红 1 条，在它自己的 `ensures`。别的函数不用这一支（见证 282 行走 Some(Some)）。
- 推得：落点 `checked_bytes_in_indexed_unit`，1 条。表：`proof_errors=1`，`landing=checked_bytes_in_indexed_unit`。**一致。**

### 第 416 行 `write-intersection-one-byte-is-not-dropped`（C）

- 改坏：「if from < until {」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:187`） 改成 `if from < until && until - from > 1 {`。
- 推：交集恰一个字节时走 None 支，而 None 支的 `ensures` 要 `R == 0` 且逐字节无交；这两个合取同在 :170 起的那一条 `match` 臂里，算一次检查，红 1 条。`until - from` 在 `from < until` 之后才算，不下溢。两个调用方用的是 `ensures`、没改，不红。
- 推得：落点 `bytes_in_unit`，1 条。表：`proof_errors=1`，`landing=bytes_in_unit`。**一致**（1 条依赖前提 1：若 Verus 把 `&&&` 拆开报，会是 2，实量定）。

### 第 502 行 `file-range-witness-is-checked`（D）

- 改坏：「pub fn witness_file_range_boundaries() {」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:277`） 之后插入 `assert(false);`。
- 推：这条 `assert(false)` 必红 1 条；之后的断言本来为真，不再报。证明里没有调用方（上面输出里 614 行是 `#[cfg(test)]` 里的普通 Rust 调用）。
- 推得：落点 `witness_file_range_boundaries`，1 条。表：`proof_errors=1`，`landing=witness_file_range_boundaries`。**一致。**

### 第 503 行 `gpu-score-candidate-window-first-not-after-past`（E）

- 改坏：「if past < first {」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:121`） 改成 `if false {`；它保护的是 「first <= past,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:117`）。
- 推：这条 `ensures` 靠 :121–:124 那两个断言（在 `past` 这个位置把两个 `is_lower_bound` 的量词各实例化一次，再用 `E(start) <= end`）推出；`if false` 之后这个位置上没有相应的项，量词不会自己实例化，`first <= past` 不成立，红 1 条。:118、:119 两条的证明按 `position` 与 `first`、`past` 各自比较（:128–:137），不用 `first <= past`，仍成立。调用方（170、223 行）只看 `ensures`，没改。
- 推得：落点 `lemma_candidate_window_keeps_every_contribution`，1 条。表：`proof_errors=1`，同落点。**一致**（「量词不会自己实例化」是求解器行为，推的，实量定）。

### 第 504 行 `gpu-score-candidate-window-holds-only-the-device`（F）

- 改坏：「first <= position < past ==> records[position].device == device,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:118`） 的 `< past` 改 `<= past`。
- 推：`position == past`（且 `past` 小于表长）时，`records[past]` 只知道「不在 `end` 之前」与「不在 `E(start)` 之前」，设备号可以更大，等于不成立，红 1 条；别的两条不受影响；调用方假设更强的 `ensures`，不新红。
- 推得：落点 `lemma_candidate_window_keeps_every_contribution`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 第 505 行 `gpu-score-theorem-first-is-the-candidate-lower-bound`（G）

- 改坏：「let first = lower_bound_of_sorted_records(sorted, device, earliest_candidate_start(start), 0, sorted.len() as int);」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:220`） 里 `earliest_candidate_start(start)` 改成 `start`。
- 推：`first` 变成 `start` 处的下界，不再是 `E(start)` 处的。红点三处，都在定理自己里：(1) 返回处 「is_lower_bound(sorted, device, earliest_candidate_start(start), window.0),」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:214`） 不成立；(2) 223 行调用候选窗口引理，要 `is_lower_bound(…, E(start), first)`（引理的 :113）不成立；(3) 「lemma_search_prefix_matches_the_walk(sorted, device, start, end, first, past, sorted.len() as int);」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:225`） 要同一条（引理的 :160）也不成立。`window.0 <= window.1` 仍可由 223 行调用后的 `ensures` 得到，不另红。别的函数不受影响（改的是定理体）。
- 推得：落点只有 `theorem_sorted_search_equals_full_record_walk`；独立红点 3 处，每函数至多报 3，哪两处先被找到由求解器定，条数 2 或 3。表：`proof_errors=2`，同落点。**落点一致；条数推不出**（2 在 2 到 3 之内）。

### 第 506 行 `gpu-score-theorem-past-is-the-end-lower-bound`（H）

- 改坏：「let past = lower_bound_of_sorted_records(sorted, device, end, 0, sorted.len() as int);」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:221`） 里 `end` 改成 `end - 1`。
- 推：红点三处，同样都在定理里：(1) 「is_lower_bound(sorted, device, end, window.1),」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:215`） 不成立；(2) 223 行调用要 `is_lower_bound(…, end, past)`（引理 :114）不成立；(3) 225 行调用要同一条（:161）不成立。
- 推得：落点只有 `theorem_sorted_search_equals_full_record_walk`，条数 2 或 3。表：`proof_errors=2`，同落点。**落点一致；条数推不出。**

### 第 507 行 `gpu-score-theorem-window-may-be-empty`（I）

- 改坏：「window.0 <= window.1,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:216`） 改成 `window.0 < window.1`。
- 推：窗口可以为空（空记录表时 `first == past == 0`，见证 269 行 `theorem_sorted_search_equals_full_record_walk(empty, empty, 7, 0, 1)` 就是这个形态），严格小于不成立，返回处红 1 条；`first <= past` 本身仍由 222–224 行的分情况证出。三个调用处（264、269、271 行）拿到更强的 `ensures`，不新红。
- 推得：落点 `theorem_sorted_search_equals_full_record_walk`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 第 508 行 `gpu-gc-witness-record-slice-sum-value`（J）

- 改坏：「record_slice_sum(seq![ScoringRecord { device: 7, released: false, start: 9, end: 12 }], 7, 10, 20, 0, 1) == 2,」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:345`） 的 `== 2` 改 `== 3`。
- 推：一条记录、同盘未释放、`[9,12)` 对段 `[10,20)` 的交是 `[10,12)`，长度 2（「if begin < end { end - begin } else { 0 }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:129`）），`record_slice_sum` 的递归定义（:165–:167）展开一步得 2，`== 3` 不成立，这一条 `ensures` 红 1 条；体内的引理调用证的是真命题，不会推出 3。没有调用方（上面输出）。
- 推得：落点 `witness_gpu_gc_protocol_and_score_premises`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 第 509 行 `gpu-gc-witness-first-disagreement-count-value`（K）

- 改坏：「count_of(update_count(Map::<int, nat>::empty(), 7, false), 7) == 1,」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:346`） 的 `== 1` 改 `== 2`。
- 推：空表、键 7、不一致，「if agree || previous >= 1 { 0 } else { 1 }」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:20`） 给 `next_uncertain_count(0, false) == 1`，`update_count` 走 `counts.insert(7, 1)`，`count_of` 是 1，`== 2` 不成立，红 1 条。
- 推得：落点 `witness_gpu_gc_protocol_and_score_premises`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 第 510 行 `overwrite-exact-keeps-the-length`（L）

- 改坏：「overwritten_bytes(old, start, data).len() == old.len(),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:123`） 改成 `== old.len() + 1`。
- 推：「Seq::new(old.len(), |byte: int|」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:114`） 定义里长度就是 `old.len()`，`+ 1` 不成立，红 1 条，在引理自己的 `ensures`；别的 5 条 `ensures` 不依赖它。调用方（206、207 行的见证）拿到与 `Seq::new` 长度公理自相矛盾的假设，空真，不新红。
- 推得：落点 `lemma_overwrite_changes_exactly_the_requested_bytes`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 第 511 行 `overwrite-exact-empty-request-keeps-the-file`（M）

- 改坏：「data.len() == 0 ==> overwritten_bytes(old, start, data) == old,」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:131`） 的 `== 0` 改 `<= 1`。
- 推：`data.len() == 1` 时写一个字节，结果与 `old` 一般不同，`== old` 不成立，红 1 条；体内 「if data.len() == 0 {」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:138`） 那一支不变。调用方 206 行（`data.len() == 2`）前提为假、207 行（空）仍成立，不新红。
- 推得：落点 `lemma_overwrite_changes_exactly_the_requested_bytes`，1 条。表：`proof_errors=1`，同落点。**一致。**

### 13 行对照小结

| 行 | 被改处 | 读码推的落点 | 读码推的条数 | 表第五段 | 表第六段 | 对照 |
| --- | --- | --- | --- | --- | --- | --- |
| 414 | `bytes_in_unit` 的 None 支 R 为零 | bytes_in_unit、checked_bytes_in_indexed_unit | 2 | 2 | 同 | 一致 |
| 415 | `checked_bytes_in_indexed_unit` 的 None 支 R 为零 | checked_bytes_in_indexed_unit | 1 | 1 | 同 | 一致 |
| 416 | `bytes_in_unit` 函数体丢掉一字节交集 | bytes_in_unit | 1（依赖前提 1） | 1 | 同 | 一致 |
| 502 | 见证注入 `assert(false)` | witness_file_range_boundaries | 1 | 1 | 同 | 一致 |
| 503 | 候选窗口引理 `first <= past` 的分情况去掉 | lemma_candidate_window_keeps_every_contribution | 1 | 1 | 同 | 一致 |
| 504 | 候选窗口引理 窗内只有目标设备 | lemma_candidate_window_keeps_every_contribution | 1 | 1 | 同 | 一致 |
| 505 | 定理 `first` 的下界参数 | theorem_sorted_search_equals_full_record_walk | 2 或 3 | 2 | 同 | 落点一致，条数推不出 |
| 506 | 定理 `past` 的下界参数 | theorem_sorted_search_equals_full_record_walk | 2 或 3 | 2 | 同 | 落点一致，条数推不出 |
| 507 | 定理 窗口可以为空 | theorem_sorted_search_equals_full_record_walk | 1 | 1 | 同 | 一致 |
| 508 | 见证 `record_slice_sum` 的值 | witness_gpu_gc_protocol_and_score_premises | 1 | 1 | 同 | 一致 |
| 509 | 见证 首次分歧计数的值 | witness_gpu_gc_protocol_and_score_premises | 1 | 1 | 同 | 一致 |
| 510 | 第一条引理 长度不变 | lemma_overwrite_changes_exactly_the_requested_bytes | 1 | 1 | 同 | 一致 |
| 511 | 第一条引理 空请求不改文件 | lemma_overwrite_changes_exactly_the_requested_bytes | 1 | 1 | 同 | 一致 |

落点 13 行读码都推到与第六段相同的集合；条数 11 行推到与第五段相同，2 行（505、506）推不出。

**改坏的是不是名字说的那条**：13 行都是。414、415 改的就是名字说的两条「空交集长度为零」；416 改函数体让一字节交集落进 None 支，名字说的「一字节不被丢」正是被它打穿的 None 支两个合取；502、508、509 改的是见证自己的断言；503 去掉的是 `first <= past` 的推理、504、507 改的是名字点名的那一条 `ensures`；505、506 改的是名字点名的那个下界参数，红的是定理自己的 `ensures` 与它调用的引理前提；510、511 改的是第一条引理里名字点名的那一条。

**冲突**：没有读码上的冲突。**规则没说**：505、506 的条数（求解器行为）与 503、416 依赖的两条求解器假设；交 `measure_rows.py` 实量。推翻这一节的现象：`measure_rows.py` 的 `ROW` 行里有 `landing=DIFFER`，或某行 `measured_errors` 与上表「读码推的条数」的单值不同（505、506 以外）。

## 三、检查单八格

每格回「成立」「不成立」或「推不出」，带证据。命令输出（原样）：

```
$ grep '^diff --git' research/prompts/replay-prefix-backfill-code-r3-snapshot/changes.patch
diff --git a/crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md b/crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md
diff --git a/crates/singlefs-replay-prefix/src/README.md b/crates/singlefs-replay-prefix/src/README.md
diff --git a/crates/singlefs-replay-prefix/verus-must-red-worlds.tsv b/crates/singlefs-replay-prefix/verus-must-red-worlds.tsv
$ grep -nE '^\+.*(兼容|legacy|迁移|旧版|旧库|旧镜像|旧布局|版本号)' research/prompts/replay-prefix-backfill-code-r3-snapshot/changes.patch | cut -c1-160
（以上无输出则是没有命中）
$ grep -nE '^\+.*(崩溃点重放|崩溃枚举|crash-case|CRASH_POINT)' research/prompts/replay-prefix-backfill-code-r3-snapshot/changes.patch | cut -c1-200
（以上无输出则是没有命中）
$ grep -nE '/home/|~/|/tmp|\.local/share|HOME|localhost|ssh |--threads|--memory|--peer|--local-only|SINGLEFS_THREAD_CAP|capped\.sh|nice |timeout ' crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md crates/singlefs-replay-prefix/src/README.md
（以上无输出则是没有命中）
$ awk -F'\t' '!/^#/ && NF>0 {n++; c[$1]++; if (NF!=6) bad++} END{print "rows="n, "distinct="length(c), "not6fields="bad+0}' crates/singlefs-replay-prefix/verus-must-red-worlds.tsv
rows=458 distinct=458 not6fields=0
```

1. **走不到的理由二次查实：成立（本轮没有这一类）。** 补丁 `changes.patch` 只动三个文件（上面第一条输出），没有 `.rs`，也没有产品代码的「走不到」声明可查；七份被判文件今天与开工快照 sha256 逐份相同（开头那两条 `sha256sum -c`）。推翻它的现象：补丁里出现 `.rs`。
2. **类型与名字：成立。** 补丁没有新类型与新函数。13 行必红的变异名每一个都说得出它改坏的是哪一条（上一节「改坏的是不是名字说的那条」逐行对过：414、415、416、502 到 511 各对应名字里点名的 `ensures`、见证值或下界参数）。名字里的 `first`、`past` 是被改引理与定理自己的参数名，不是序数代称。推翻它的现象：实量时某行的红点落在名字没点名的条款上。
3. **不兼容旧代码与旧格式：成立。** 补丁新增行里没有版本号、迁移、旧库旧镜像旧布局的字样（上面第二条输出无命中），没有认旧版本的分支。推翻它的现象：补丁里出现只为旧数据留的放宽。
4. **不新建判定库：成立。** 必红表是门禁读的登记表，不是判定存放处；判定走 「判定库：只有 prune-server 那一份（verdict-library 族，种类 singlefs-formal-unit-verdicts）」（`.claude/gate.d/formal-verification.sh:44`）。补丁里没有新的 TSV、sqlite、RocksDB 目录。推翻它的现象：补丁或脚本里出现往登记表以外的地方写判定。
5. **所有任务多机可用：成立。** 说明文件与 README 里没有写死本机路径、主机名、`~`、`$HOME`，也没有线程、内存、挑台参数与包装前缀（上面第四条输出无命中）；复跑命令都是仓内相对路径加 `task-server/task submit --`。本腿的复跑命令里写的绝对路径都在仓里或工作根（共享根）下，Verus 安装目录由 `measure_rows.py` 经 `lib_work_root.py` 的 `shared_data_path("formal-tools")` 取，不经 `~`。推翻它的现象：派到第二台读不到 Verus 安装或运行目录。
6. **测试优先多线程：成立（本轮没有这一类）。** 没有新测试，补丁没有 `.rs`；复跑命令不写线程参数（同上第四条输出）。推翻它的现象：补丁里出现新测试。
7. **带 `--item` 的命令与项：静态成立，实交推不出（待主 agent 跑）。** 新加 13 行能不能被列出：门禁头写明 「必红世界每行一项 must-red:<变异名>」（`.claude/gate.d/formal-verification.sh:52`）；表 458 行、第一段 458 个互不相同、每行恰 6 段（上面第五条输出），13 个名字各出现一次且原文恰好命中一次（`out/check-hits.txt`）。说明文件里 `--item must-red:<名>` 点名的 16 个名字在表里各命中一次；4 条 `cargo test` 命令形状与点名全名都对（`out/check-commands.txt`）。主 agent 已交过的 21 项那一趟「证明与 20 行必红」读数是背景材料里的，本腿没有 task log 可核。真的能不能列、能不能原样交，要 `--list-items` 与六条命令实交：下一节第 3、4 条。推翻它的现象：`--list-items` 里缺这 13 个名字，或命令被钩子与服务拒。
8. **新崩溃流多罩了什么：成立（本轮没有这一类）。** 补丁新增行里没有崩溃点重放流与崩溃枚举用例的字样（上面第三条输出无命中）。推翻它的现象：补丁里出现 `crash-case:` 登记行或崩溃枚举用例。

## 四、要主 agent 跑的命令

都在仓根 `/home/fy5090/code/singlefs` 起，各自单交。

1. 13 行与 7 行的实量（整 crate Verus，基线加 20 行各一趟）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/replay-prefix-backfill-code-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r3-forward/measure-run /home/fy5090/code/singlefs/research/prompts/replay-prefix-backfill-code-r3-sonnet-model
```

   看：`TASK_SERVER_RESULT` 行 `exit=0`；`task-server/task log <号>` 里有一行 `BASELINE exit=0 verified=214 errors=0`、20 行 `ROW name=…`、末行 `SUMMARY rows=20 landing_same=20 counts_same=<n>`；每行 `landing=same`（有 `DIFFER` 即退 1）；`counts=differ` 的行只报、不判，要与上面第二节「读码推的条数」对（505、506 在 2 到 3 内，别的行与表第五段相同）；`ROW` 行后缩进的 `ERR` 行是每条证明错与落到的函数；开头那 7 行 `sha256sum -c` 都是 `OK`（不是 `OK` 说明跑的不是开工快照的样子）。没有给范围声明：这是模型复跑，不是门禁阶段；若钩子要声明，照它的拒绝说明补。

2. 模型自带的接线自证（只拷、不起 Verus）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/replay-prefix-backfill-code-r3-sonnet-model/tools/rerun.sh --selftest
```

   看：`exit=0`，日志里 `✓ 自证通过（查了 6 格）`。

3. 新加 13 行能否被逐项点名列出（只读登记表，不起 Verus）：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 的 --list-items 只读列项; 范围=只列项不判; 为什么=核新加 13 行必红能被逐项点名列出' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items
```

   看：`exit=0`；日志里 `must-red:write-intersection-empty-has-zero-length`、`must-red:indexed-intersection-empty-has-zero-length`、`must-red:write-intersection-one-byte-is-not-dropped`、`must-red:file-range-witness-is-checked`、`must-red:gpu-score-candidate-window-first-not-after-past`、`must-red:gpu-score-candidate-window-holds-only-the-device`、`must-red:gpu-score-theorem-first-is-the-candidate-lower-bound`、`must-red:gpu-score-theorem-past-is-the-end-lower-bound`、`must-red:gpu-score-theorem-window-may-be-empty`、`must-red:gpu-gc-witness-record-slice-sum-value`、`must-red:gpu-gc-witness-first-disagreement-count-value`、`must-red:overwrite-exact-keeps-the-length`、`must-red:overwrite-exact-empty-request-keeps-the-file` 各出现一行（13 个）。

4. 说明文件「复跑」一节的命令原样交一次（看钩子与服务当场收不收）。前五条是文件第 100、102、104、106、193 行，原样：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 4 项; 为什么=复判打分算法的二分跳项、窗口过窄、释放误计与重数四行反例' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item must-red:gpu-score-binary-skips-entry --item must-red:gpu-score-window-too-narrow --item must-red:gpu-score-search-counts-released --item must-red:gpu-score-permutation-needs-multiplicity .

SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 formal-verification.sh 带 --item; 范围=点名的 4 项; 为什么=复判 G01 到 G04 四行反例，看仲裁与计数模型的契约还抓得住' task-server/task submit -- bash .claude/gate.d/formal-verification.sh --item must-red:gpu-gc-G01 --item must-red:gpu-gc-G02 --item must-red:gpu-gc-G03 --item must-red:gpu-gc-G04 .

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_arithmetic_matches_wide_integer_reference 里的 4 条测试; 范围=4 个测试; 为什么=用 u128 参照复核 G01 与 G14 到 G20 的产品核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_arithmetic_matches_wide_integer_reference -- gpu_budget_arithmetic_matches_a_u128_reference_at_full_width gpu_slices_scans_and_independent_totals_match_wide_arithmetic gpu_readback_rejects_unwritten_words_and_preserves_both_halves gpu_reconciliation_matches_the_two_round_history_rule --exact

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-replay-prefix 的测试目标 gpu_gc_capacity_matches_independent_reference 里的 5 条测试; 范围=5 个测试; 为什么=用独立参照复核 G21 到 G30 的容量与记账核' task-server/task submit -- cargo test -p singlefs-replay-prefix --test gpu_gc_capacity_matches_independent_reference -- chunk_admission_and_extension_match_full_width_arithmetic largest_piece_matches_exhaustive_search_and_is_maximal holds_releases_and_waits_preserve_the_exact_ledger memory_caps_never_expand_and_allocation_failure_makes_progress mapping_buffers_and_dispatch_cover_their_exact_full_width_sizes --exact

SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的测试目标 gpu_gc_formal_contracts_through_the_host 里的 2 条测试; 范围=2 个测试; 为什么=复核真实主机仲裁适配层的有限历史枚举' task-server/task submit -- cargo test -p singlefs-background-gpu --test gpu_gc_formal_contracts_through_the_host -- interleaved_entries_failures_empty_batches_and_remounts_match_the_history_model permuting_distinct_entries_preserves_keyed_results_and_cpu_calls --exact
```

   第六条是文件第 108 行起到第 191 行止（以 `--exact` 单独一行收尾，各行反斜杠续行）的 82 条测试那一条，原样整条交。看：每条 submit 当场不拒；`TASK_SERVER_RESULT` 行 `exit=0`；`cargo test` 的 `test result: ok. N passed` 里 N 依次是 4（第 104 行）、5（第 106 行）、82（第 108 行）、2（第 193 行）；两条 `formal-verification.sh --item must-red:…` 的成功句里点名的行数是 4 与 4。

等哪几条结果才能往下做：第 1 条出结果之前，第二节里 505、506 两行的条数与 416、503 的「依赖求解器假设」两处都只是读码推的。第 3、4 条出结果之前，检查单第 7 格的「实交」只是静态判断。

## 五、判定一览

| 格 | 判定 | 一句话 |
| --- | --- | --- |
| 判决第三节第 1 条（README 义务数与文件表） | 一致（做到） | 19 = 10 个 exec fn + 9 个 proof fn，与门禁成项规则和主 agent 那一趟 log 的 19 项都对上；文件表补了精确交集长度 |
| README 同类遗留（:119、:148、:138、:165） | 冲突，交主 agent | 「义务」一词两种口径并存；unlink 必红数过时（12 行不是 7）；覆盖写清单少 510、511 两行；:119、:126 带历史式说法 |
| 第 2 条：4 条 cargo 命令的点名与范围 | 一致 | 4、5、82、2 个全名都是 `#[test]`，数 = 范围数，82 = 四个模块的全部；实交待主 agent |
| 第 2 条：:97 卡的说法 | 一致 | 与 `.claude/rules/verification.md` 同一条相同 |
| 第 2 条：真卡节与 NOT_RUN 读法 | 一致加两处出入 | NOT_RUN 与交卡两行对得上；`GPU_DEVICE_LOST` 在这两个包里没有打印点；「没有可交的复跑命令」宽读与门禁里 product-gpu-real-card-tests 格冲突 |
| 第 3 条：verus-pins 三行与必红 365 行 | 一致 | 10 个核加 1 个见证 = 11；G30 全仓只有测试引用；214 与登记数、log 的 verified 之和相同；日期去掉 |
| 第 4 条：必红表补行、条数、注释 | 一致 | 判决点名的 ensures 都有行罩着；7 行条数与 log 实测逐行相同；注释行与下面的行类别对得上 |
| 第 5 条：G11 定义自检 | 一致（宽读） | 三条里第二条是定义加算术的推论；G10 同类未标，交主 agent |
| 新加 13 行：落点 | 一致 | 13 行读码推的落点集合都与第六段相同 |
| 新加 13 行：条数 | 11 行一致，2 行推不出 | 505、506 有 3 处独立红点，报 2 还是 3 由求解器定；实量待 `measure_rows.py` |
| 新加 13 行：改坏的是不是名字说的那条 | 成立 | 13 行逐行对过 |
| 检查单：走不到的理由二次查实 | 成立 | 补丁没有 `.rs` |
| 检查单：类型与名字 | 成立 | 无新类型与函数；13 个变异名各说得出它改坏的条款 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 新增行无旧版本字样 |
| 检查单：不新建判定库 | 成立 | 必红表是登记表，判定仍进 prune-server 那一份 |
| 检查单：所有任务多机可用 | 成立 | 无本机路径、主机名、线程参数 |
| 检查单：测试优先多线程 | 成立 | 无新测试 |
| 检查单：带 `--item` 的命令与项 | 静态成立，实交推不出 | 13 个名字唯一、命令形状对；`--list-items` 与六条命令待主 agent 跑 |
| 检查单：新崩溃流多罩了什么 | 成立 | 无崩溃流与崩溃枚举用例 |

## 六、没做什么

- 不判别的腿的格（攻方、辩方腿的结论），不替主 agent 采纳或出判决。没往统一任务服务交任务；判决要引的数（13 行与 7 行的实量、`--list-items`、六条复跑命令的实交）没复跑，命令在第四节。没有起过 Verus：第二节的条数、落点都是读码推的，只有「7 行条数与 log 实测相同」一项对着主 agent 那一趟 log（本腿读 `task-server/task log`）核过。
- 没读禁读清单里的文件（同一轮另一条腿的产出与模型目录）。
- 没判：README :126「3 个 Rust 回归测试」的数（没数）；后台 GPU 的两个包别的会话在拆，只判了今天的树；判决第 6 条的欠账登记没做、也不在这一轮；`verus-pins.tsv` 第 40 至 46 行的带日期注释没动，不在判决字面里。
- 清理：用完删了草稿目录里的接线自证副本（80K）、`rerun.sh` 接线试跑的运行目录（1.7M）与读主 agent 那趟 task log 时拷下的文本（484K，其中的数已落进上面各输出块）。草稿目录现在只剩命令输出、各段草稿与小工具，共 268K，没有仓副本与编译目录。

## 七、引文核对

交回前跑 `python3 scripts/agents/cite-check.py` 的末行（报告里每处「引文」（`路径:行号`）都在被引文件那一行里逐字找到；没判的 7 处是代码块里 `grep -n` 输出带的 `路径:行号:` 前缀，不是引文）：

```
$ python3 scripts/agents/cite-check.py research/prompts/replay-prefix-backfill-code-r3-sonnet-output.md
  ✓ 核了 77 处引文、7 处裸行号，对上 77 处，没判 7 处（其中裸行号没有引文可比 7 处；逐处列在上面）
```
