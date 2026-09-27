# m2-layer0-scale-r1 云端正推（Sonnet）报告

负责格：L1、L2、L4、L5、L7。方案按冻结副本 `/tmp/claude-1000/l0scale-r1-frozen/` 谈，行号全部在该副本或工作区仓库里现查（命令与输出见各节）。禁读本轮 Opus / 本地攻方的提示与产出，未读。

时区：本机 UTC，人在 JST（UTC+9）；下面「现查」的时刻是本次执行时的 UTC。

## 判据表（正文第一节）与本报告的关系

正文（`research/prompts/_m2-layer0-scale-r1-background.md`）第一节表格给出的是**问题**，不是**结论**——L1–L7 每一格本身没有断言"应该是什么"。我这条腿的任务是：从条款 + `crates/` 现状推出"应该是什么"，再与正文里**能坐实为断言的那几句话**（如"5 段 30 写的段占 96%"、"crates/ 里没有任何约简"）逐句比对。下面每一节先给推导，再给判定。

## L1：甲的前提在什么条件下成立

### 推导

甲拆成两条判据，对应 `crates/singlefs-harness/src/segments.rs:20-29` 的 `StepKind` 六种：`ZeroFill`、`UnitWrite`、`JournalRecord`、`RootRecordFua`、`SystemConfigurationSlot`、`Barrier`。落点由 `FixedGeometry::classify`（`segments.rs:58-87`）按偏移判：系统配置槽区间之外、根环区间之外、journal 环区间之外的一切普通写都归 `UnitWrite`（`segments.rs:81-83`）——"COW 新写的单元、分配记录、映射条目"字面上都落在这一类。

1. **"块不被任何『恢复可能选中的版本』引用"**：`.claude/kb/decisions/16-发布语义.md:27` 已定项 1 的回收谓词——"可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)"——就是分配器只会把新的 COW 写落在"不被回退候选集里任何根引用"的块上的**唯一保证来源**；`.claude/kb/invariants.md:54` 的 I-7.4 把它重述成不变量原文："回退候选集里每一个根……所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头"。**"恢复可能选中的版本"这个集合，按 D16（发布语义）已定项 1 的回退候选集取**（`.claude/kb/decisions/23-journal的角色与格式.md:351` 已定项 14 的"候选集"一行原文引用同一个集合："根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义）已定项 1 的回退候选集……）"）。

   ⇒ **甲对 UnitWrite 的前提，语义上就是 I-7.4 这条不变量本身**：甲成立当且仅当 I-7.4 在这条被测历史上成立，不是一条独立于 I-7.4 的新假设。

2. **"也不被 journal 重放读到"**：由持久顺序保证。`.claude/kb/decisions/16-发布语义.md:174` 已定项 7 钉死顺序"COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽"；`segments.rs:94-127` 的 `split_into_segments` 用屏障切段（屏障关掉它之前那一段，`segments.rs:104-109`）——同一次发布里 `UnitWrite` 与它自己的 `JournalRecord` 之间隔着一道屏障，必然落在不同段，且 `JournalRecord` 段严格晚于 `UnitWrite` 段。D13（验证路线）已定项 4（`.claude/kb/decisions/13-验证路线.md:69`）定义崩溃状态为"前若干段全部持久，当前段任意子集持久，之后各段全部没持久"；`crash.rs:1474-1509` 的 `Layer0StatePlan` 就是这么实现的。⇒ 当 `UnitWrite` 段是"当前段"时，它自己那次发布的 `JournalRecord` 段属于"之后"、必然整段未持久，重放读不到——**这半句在同一次发布内部恒成立，不需要额外证明**；真正要看的只剩**跨发布的块复用**（本次 UnitWrite 落的物理块，是不是被更早、仍在候选集内的某次发布的 journal 记录/根指着），这归到第 1 条。

### 有没有可达历史让前提不成立

条款里明写的边角，全部指向"分配器/回收谓词的正确性今天不是全证明过的"：

- `.claude/kb/decisions/23-journal的角色与格式.md:351` 已定项 14"挂着时回退的已知边角三样"第①条原文："『R_old 仍分配的落点在 cur 的账里是同槽同分配代』这道核不在定案里，也补不上……它放过的读不对的回退只在 F 回落时走得到"。
- `.claude/kb/invariants.md:54` I-7.4 表格行原文："checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条……欠 C556（checker 与层 0 不读系统配置里的 F）"——I-7.4 没被判红不等于它真的成立，只等于 checker 没查出来。
- 同一行原文："它们引用的块在离开根环之前同样不许重新分配、不许抹头……留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出"——这条不变量本身就是在堵一个**历史上真实出现过**的"COW 写落在了后来被证明仍可能被选中的块上"的反例类型。

⇒ **结论**：甲对 UnitWrite 的前提不是独立公理，是"I-7.4 + 写序保证的段隔离"的推论，在正确实现下应当成立，但**不能只靠论证认定**——它依赖的正是可能有 bug 的分配 / 回收逻辑本身，且该逻辑的判定（checker）已知有欠账（C556）、历史上出过反例（H6、C314）。这直接给出 L2 的必要性：约简必须有独立的运行时现判，不能只凭"前提论证过了"。

### 判红的现状落在哪一步（与 L2 共用这条现查）

`crates/` 今天没有任何"现判"。现查：
```
$ grep -n 'reduc\|约简' crates/singlefs-harness/src/segments.rs crates/singlefs-harness/src/crash.rs
（零命中）
```
与正文附录二之三的观测一致（附录二之三另指出同一条 grep 在 `crates/singlefs-harness/src/` 全目录下命中 8 行，全部落在 `src/bin/e158_root_choice_repair.rs` 的字段名 `skipped_by_the_reduction_rule`，与本轮无关——正文第二节写"零命中"不准确，附录已自己指出，我复核同意附录）。

`enumerate_layer0_versions`（`crash.rs:1851-1866`）与 `enumerate_layer0`（`crash.rs:1870-1885`）——分别是两条流 `--full` 唯一调用的入口——把 `expand` 参数硬编码成 `&|_segment_index, _segment| true`：**每一段都全展开，没有约简**。`expand` 这个钩子本身已经存在（`Layer0StatePlan::new`，`crash.rs:1484-1509`，第 1497 行 `if expand(segment_index, segment) { next_ordinal += (1u64 << segment.len()) - 1; }`），但今天**唯一**用它做"跳过展开"的地方是"快档"用例：`second_transaction_step_zero_layer0.rs:457`：`let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;`——判据是**段长度**，不是 L1 要求的"块引用 / journal 读取"语义，而且这条 `expand` 从未接到 `enumerate_layer0_versions`/`enumerate_layer0`（--full 的入口），只接到 `enumerate_layer0_selecting_versions`（`crash.rs:1670-1688`）供快档用例单独调用。

⇒ **约简的钩子已经存在，约简的判据不存在**；54 号门禁 `--full`（全量，`.claude/gate.d/54-layer0-replay.sh:340-396`）走的是 `enumerate_layer0_versions`/`enumerate_layer0`，今天这条路径**没有、也接不到**任何约简。

### 判定

| 子问题 | 判定 | 依据 |
|---|---|---|
| 甲对 UnitWrite 的前提是不是独立假设 | **规则没说**（背景材料未断言，这是我推导补上的） | I-7.4（`invariants.md:54`）= 甲的前提，二者等价 |
| "journal 重放读到"半句是否需要单独证明 | **一致**（与背景材料没有冲突，只是背景材料没展开） | D16 已定项 7 持久顺序 + `segments.rs` 的屏障切段 |
| 正文第二节"crates/ 里没有任何约简……零命中" | **冲突**，附录已自证冲突，我复核该冲突成立且额外补充 8 处命中的准确出处 | 附录二之三；`e158_root_choice_repair.rs:1210` 等 8 行 |
| `crates/` 里有没有可复用的约简钩子 | **规则没说**（背景材料没提到 `expand` 参数） | `crash.rs:1489,1497`，`second_transaction_step_zero_layer0.rs:457` |

### 推翻条件

若在 `crates/singlefs-core` 里找到一处 COW 分配路径**不经过** D16 已定项 1 的回收谓词（例如绕过 `allocator.rs` 直接复用一个仍在候选集内的块的代码路径），则"甲的前提=I-7.4"这条等价关系不成立，甲需要一条独立于 I-7.4 的额外证明。**复核不了**：我没有跑 `crates/singlefs-core/src/allocator.rs` 的单测或读它的全部调用点来确认这条路径不存在，只读了 harness 侧（`crash.rs`）的证据链；这半句需要读 `allocator.rs` 源码或请攻方腿造历史核实（分工表已把这件事派给 Opus）。

## L2：甲的前提怎么判红

### 改法（对冻结副本 `crates/` 的改动）

**文件**：`crates/singlefs-harness/src/crash.rs`。

**步骤 1**：新增函数，紧邻 `reuse_is_not_proven_illegal_by_the_reclaim_predicate`（`crash.rs:786-826`）之后：

```
fn segment_lands_only_on_blocks_invisible_to_recovery(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> bool
```

逻辑：
1. 若 `segment` 内任一写的 `kind` ∈ `{RootRecordFua, SystemConfigurationSlot, JournalRecord}`，直接返回 `false`（这一段必须全展开——对应 L1 后半句"只有原地覆盖的写要逐个子集枚举"）。
2. 否则（段内只有 `UnitWrite` / `ZeroFill`），对段内每一写取 `(device, offset, length)`，只用**这一段之前**的写（该状态模型下已确定全持久的历史，对应 `writes[..segment[0]]`）重建候选集下界——**复用** `reuse_is_not_proven_illegal_by_the_reclaim_predicate` 里已经写好的同一套推导（`root_in_each_ring_slot`、`highest_rollback_floor`、`oldest_root_in_the_ring`，`crash.rs:794-825`），只是把判据方向倒过来问："这个写要落的区间，此前有没有被候选集内仍有效的某次发布占用过"。段内任一写命中就返回 `false`；全部不命中返回 `true`。

**步骤 2**：接线——把 `enumerate_layer0_versions`（`crash.rs:1858-1865`）与 `enumerate_layer0`（`crash.rs:1877-1884`）里硬编码的 `&|_segment_index, _segment| true` 换成：
```
&|_segment_index, segment| !segment_lands_only_on_blocks_invisible_to_recovery(writes, segment)
```
不需要改 `Layer0StatePlan::new`（`crash.rs:1489,1497`）——它逐段调 `expand` 的机制已经存在，这一步只是换调用方给的闭包。

### 判它用的『引用集合』从哪来、会不会与被测实现共用同一段代码

不会共用。`reuse_is_not_proven_illegal_by_the_reclaim_predicate`（`crash.rs:786-826`）与我提议的新函数都只读 `RetainedWrite`（录制下来的写日志），**不调用** `crates/singlefs-core/src/allocator.rs` 或 `crates/singlefs-core/src/recovery.rs` 里的任何生产代码——现查：
```
$ grep -n 'singlefs_core::' crates/singlefs-harness/src/crash.rs | grep -i 'alloc\|recover'
（在 786-826 行区间内零命中；`recover(...)` 调用只出现在 evaluate_state_for_versions 等评状态函数里，不在 reuse_is_not_proven_illegal_by_the_reclaim_predicate 内）
```
这与 `check_records_against`（`crash.rs:676-740`，"记录核对器"，D13（验证路线）已定项 7）是同一种独立复算方式，满足 `.claude/rules/fs-design.md:17` 一节的"运行时与 checker 必须用不同的算法"的精神：这条禁令字面管的是"运行时决策路径 vs checker"，但约简判据要防的是同一类失效——**约简若调用真实分配器的判断逻辑，分配器自己的 bug 会被约简"认可"而不去展开子集**，L3 要抓的错就会被约简本身吃掉。

### 拿什么变异证明这道现判会红

在 `crash.rs` 的 `#[cfg(test)] mod tests` 里新增用例（不经 `Script`/`prepare`，手摆 `RetainedWrite`，风格与既有的 `state_slices_cover_every_state_exactly_once_in_ordinal_order`（`crash.rs:1986` 附近）一致）：构造一条更早发布的根仍在候选集内、引用物理块 P，再构造一段只含一次 `UnitWrite` 落在同一个 P——断言 `segment_lands_only_on_blocks_invisible_to_recovery` 返回 `false`；反例把根的 `checkpoint_txg` 改到 `F_生效` 以下（候选集之外）——断言返回 `true`。写进 `crates/mutations.tsv`（与既有第 54 行"步 6：I-7.4 不判候选根引用的单元被复用或抹头"、第 23 行"layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape"同一族），跑 `-p singlefs-harness --lib`，不必碰任何 `layer0` 测试目标。

### 判定

| 子问题 | 判定 | 依据 |
|---|---|---|
| 判法今天落在 crash.rs 哪一步 | **规则没说**（背景材料只问，未断言现状） | `crash.rs:1858-1865,1877-1884` 硬编码 `true`，`crash.rs:1489,1497` 的 `expand` 钩子空转 |
| 判法会不会与被测实现共用同一段代码 | **一致的写法方向**：背景材料点名 `.claude/rules/fs-design.md`"审计与被审计不许同一段代码"，我复用 `reuse_is_not_proven_illegal_by_the_reclaim_predicate` 的既有独立实现，方向一致 | `crash.rs:786-826` 不调用 `singlefs-core` |

### 推翻条件

若在真实录制流的某个前缀上（不需要跑完整 layer0，只需跑到能产出 `writes`/`segments` 的那一步）找到一段"全是 UnitWrite"却被这道函数误判为可约简（返回 `true`），而它实际落在候选集内某个块——证明这道判据本身有漏洞。**复核不了**：这需要真实的 `Script::` 前缀数据（`writes`/`segments`），我没有生成过（生成它就要跑到 `prepare()`，与"不跑 layer0 测试目标"这条约束的边界很近，留给崩溃验证员或主 agent 在提交时用单元测试核）。

## L4：乙——按流拆的输入清单

### 今天的现状（现查）

```
$ grep -n '^54-layer0-replay' .claude/gate.d/stage-inputs.tsv
20:54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	command=cargo command=rustc	# ……
```
一整棵 `crates/` 树是单一登记路径，两条主流（以及 L6 的 7 份）共用同一格全绿标记，按 `research/scripts/admission.py` 的 `input_manifest`（`admission.py:302-316`）与 `listed_input_files`（`admission.py:268-276`）算出**单一** `layer0_input_hash`。

依赖闭包（现查）：workspace 只有 4 个 crate（`Cargo.toml:5-9`：`singlefs-format`、`singlefs-core`、`singlefs-harness`、`singlefs-checker`），`singlefs-harness` 依赖另外三个（`crates/singlefs-harness/Cargo.toml:8-11`）。54 号跑两条流用 `cargo test --release -p singlefs-harness --test "$1"`（`.claude/gate.d/54-layer0-replay.sh:148`）——**每个 `tests/*.rs` 文件是 Cargo 里独立的测试二进制**，只链接这 4 个 crate 的库代码，不链接彼此的源文件。

9 份带 `layer0` 的用例全部 `mod common;`（现查逐一 grep）：
```
$ for f in first_transaction_step_seven_layer0.rs second_transaction_step_zero_layer0.rs \
  second_transaction_parallel_line_one_layer0.rs second_transaction_parallel_line_three_spill_over_layer0.rs \
  second_transaction_position_addressed_trees_layer0.rs second_transaction_step_three_acquisition_barrier_layer0.rs \
  second_transaction_step_three_formatted_pool_layer0.rs second_transaction_supplement_two_rollback_witness_layer0.rs \
  second_transaction_supplement_two_tree_split_layer0.rs; do grep -n '^mod ' "crates/singlefs-harness/tests/$f"; done
```
全部 9 份都有 `mod common;`；其中 `second_transaction_position_addressed_trees_layer0.rs:25-26` 与 `second_transaction_supplement_two_tree_split_layer0.rs:21-22` 额外 `mod common_tree_split;`。`tests/common/mod.rs`、`tests/common_tree_split/mod.rs` 是**跨全部 9 份用例共享的源码**——改了它必须让全部 9 份的绿标记一起失效，不能只挂在某一份下面。

### 按流拆的输入清单（提案，改 `.claude/gate.d/stage-inputs.tsv`）

沿用表头已有的"同一个实验另有一种调用方式写 `<键>/<方式>`"记法（`stage-inputs.tsv` 文件头注释第 9–10 行），新增两行、保留原有整行不动（原行仍可给"55 号快档"这类别的用途用，改不改由主 agent 定）：

```
54-layer0-replay.sh/first-transaction	crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs crates/singlefs-harness/tests/common/ crates/singlefs-harness/src/ crates/singlefs-core/src/ crates/singlefs-format/src/ crates/singlefs-checker/src/ Cargo.toml Cargo.lock crates/singlefs-harness/Cargo.toml crates/singlefs-core/Cargo.toml crates/singlefs-format/Cargo.toml crates/singlefs-checker/Cargo.toml	command=cargo command=rustc	# 第一条流自己的编译依赖闭包
54-layer0-replay.sh/second-transaction	crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs crates/singlefs-harness/tests/common/ crates/singlefs-harness/src/ crates/singlefs-core/src/ crates/singlefs-format/src/ crates/singlefs-checker/src/ Cargo.toml Cargo.lock crates/singlefs-harness/Cargo.toml crates/singlefs-core/Cargo.toml crates/singlefs-format/Cargo.toml crates/singlefs-checker/Cargo.toml	command=cargo command=rustc	# 第二条流自己的编译依赖闭包
```

**改法（对 `.claude/gate.d/54-layer0-replay.sh`）**：`write_layer0_input_manifest`（`54-layer0-replay.sh:775-782`）改成对每条流各调一次 `admission.py manifest`（键分别取 `54-layer0-replay.sh/first-transaction`、`.../second-transaction`），算出两个 `layer0_input_hash`；全绿标记按两个哈希分别命名（如 `singlefs-layer0-full-green.<第一条流哈希>.first-transaction`），快档判绿逻辑（`54-layer0-replay.sh:271-324`）相应拆成两次独立判断，各自的 `run_quick_tier_of_stream`（`54-layer0-replay.sh:248-268`）调用不变。

### 会不会让一次真改动被跳过

不会，只要清单包含全部 4 个 crate 的 `src/` 与 `tests/common(_tree_split)/`——共享部分被两条流的清单**同时**登记（重叠登记是安全的：任何落在共享代码里的改动，两条流都会失效，符合 `.claude/rules/implementation-workflow.md:30`"判据是那一道读的全部输入自上次跑绿以来变没变"）。真正被排除、因而"省下重跑"的，仅限于：改了另一条流自己的 `.rs` 测试文件、或改了 L6 那 7 份 `layer0` 用例中与本流无关的那几份、或改了 `tests/` 下的非 `layer0` 用例——**这三类改动今天会让两条流的整棵 `crates/` 哈希一起变、被迫都重跑一次全量**，是纯粹的浪费，不是漏判风险；拆分之后消掉的正是这份浪费。

### 乙之下一次只改 checker、只改一条流用到的代码时要重跑哪几条流

改 `crates/singlefs-checker/src/` 下任何文件——两条流的提案清单都包含它（checker 是两条流共用的第三方 oracle，D13（验证路线）已定项 4"三个互相独立的 oracle"之一；`crash.rs:1321` 的 `check_pool_image(&image)` 调用点在 `evaluate_state_for_versions` 里，两条流都经这个函数评状态），**两条都要重跑**。只改 `first_transaction_step_seven_layer0.rs` 自己——只有第一条流的清单变，**只重跑第一条流**，`second-transaction` 那一格的标记原样沿用。

### 判定

| 子问题 | 判定 | 依据 |
|---|---|---|
| 按流拆还是按段拆 | **规则没说**（背景材料把两种都列成候选，未定） | L4 行原文"按流复用……还是按段复用" |
| 拆细会不会让真改动被跳过 | **规则没说**（背景材料只问，我给出安全条件：共享代码必须重叠登记） | `stage-inputs.tsv:20`、`Cargo.toml:5-9`、`54-layer0-replay.sh:148` |

### 推翻条件

若两条流之间存在我没找到的第三种共享源（例如 `build.rs`、workspace 的 `[patch]` 段引入的隐藏依赖），使得改一条流自己的测试文件也能改变另一条流的实际行为，"拆分安全"这条结论不成立。现查：
```
$ ls crates/singlefs-harness/build.rs   → 不存在
$ grep -n '\[patch' Cargo.toml         → 零命中
```
补查全部 4 个 crate：`find crates -maxdepth 2 -iname 'build.rs'` 零命中，`grep -rn '\[patch' crates/*/Cargo.toml` 零命中——没有找到这类隐藏依赖，本条不构成推翻。


## L5：代价——闭式与算法

### 闭式算法

沿用今天的状态模型公式（两份同名 `closed_form_state_count`：`segments.rs:146-158`、`crash.rs:522-529`，均为 `1 + Σ(2^|段| − 1)`）。甲把 `expand(i)`（`Layer0StatePlan::new`，`crash.rs:1489,1497`）从"恒真"换成 L2 的现判，而 `expand` 是**整段一个布尔**（不是逐写细分）：段内只要有一个 `RootRecordFua`/`SystemConfigurationSlot`/`JournalRecord` 写，或有一个 `UnitWrite`/`ZeroFill` 没通过 L2 现判，整段退回 `2^|段|-1`；段内全是 `UnitWrite`/`ZeroFill` 且全部通过现判，这一段在 `Layer0StatePlan` 里贡献**恰好 0**（`crash.rs:1497-1499`：`expand` 返回 `false` 时 `next_ordinal` 不推进，这一段不产生任何"当前段=i"的状态点，不是"折叠成 1 个状态"）：

```
CF_甲(segments) = 1 + Σ_i [ expand(i) ? (2^|segment_i| − 1) : 0 ]
```

`expand(i)` 由 `segment_lands_only_on_blocks_invisible_to_recovery`（L2 提议的函数）取反给出。

### 今天第二条流具体剩多少个状态——冲突，答不出确数

背景材料正文第 9 行给的"背景数"（2026-09-25 JST 23:5x 实测，`SINGLEFS_HEAVY_TESTS=user-request`）：
```
$ grep -n '5575802973\|423 次写' research/prompts/_m2-layer0-scale-r1-background.md
9:……第二条流（固定脚本到 E）423 次写、54 段、闭式 5575802973 个状态，5 段 30 写的段占 96%……
```

冻结副本自己的测试源码：
```
$ grep -n '2_104_413\|54 段、闭式' /tmp/claude-1000/l0scale-r1-frozen/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
581:/// 全量：固定脚本到 E 为止 54 段、闭式 2 104 413 个状态（八个 18 写段各 262143、七个 10 写段各 1023、两个 4 写段各 15，其余 2 写段各 3、1 写段各 1）。
588:    assert_eq!(closed_form, 2_104_413, "闭式：1 + Σ(2^|段| − 1)，五十四段");
```
按 `:581` 的分解反算总写数：`8×18 + 7×10 + 2×4 + 20×2 + 17×1 = 144+70+8+40+17 = 279`（20 个 2 写段 + 17 个 1 写段凑齐 `8+7+2+20+17=54` 段）。**两处都自称"固定脚本到 E"、同为 54 段**，但写数（279 对 423）与闭式（2,104,413 对 5,575,802,973，相差约 2650 倍）都对不上；`:581` 的分解里没有任何"30 写段"：
```
$ grep -n '30 写\|30写' /tmp/claude-1000/l0scale-r1-frozen/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
（零命中）
```
**这是本轮判定表的一格实打实的冲突**，两边并排如上，我不判谁对——按定义，判定谁对需要真跑一次 `cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored`，而派发提示的硬约束是"不跑名字带 `layer0` 的测试目标"，我不能跑。

**同一份源码内部另有一处不一致，进一步说明不能靠读注释手数**：
```
$ grep -n '18 写的段' /tmp/claude-1000/l0scale-r1-frozen/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
4://! 平时 `cargo test` 跳过两个 18 写的段；全量那条标 ignored，54 号门禁在 release 下跑它。再加四组靶向的阳性对照。
453:/// 平时跑的那一份：三个 18 写的段与三个 10 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
581:/// 全量：固定脚本到 E 为止 54 段、闭式 2 104 413 个状态（八个 18 写段各 262143、七个 10 写段各 1023、两个 4 写段各 15，其余 2 写段各 3、1 写段各 1）。
```
同一份文件里"18 写段"的个数在三处注释里分别是**两个**（第 4 行）、**三个**（第 453 行）、**八个**（第 581 行）——同一个属性、同一份"固定脚本到 E"，三个数互不相同。这说明脚本在多轮迭代里变长过，注释没有跟着同步更新，**手数注释得不到可信数字**，唯一权威的是运行时打印的 `LAYER0B` 行（`54-layer0-replay.sh:382` 读它）。

### 套算法算一次示意值（不当结论，只示范算法怎么用）

**假设**（都需要主 agent 或攻方腿用真实分段数据核实，不是我验证过的事实）：`:581` 的 8 个 18 写段与 7 个 10 写段全部是纯 `UnitWrite`（不掺 `RootRecordFua`/`SystemConfigurationSlot`/`JournalRecord`），且 L2 现判对它们全部判"可约简"（`expand=false`）。代入算法：

```
CF_甲 = 2_104_413 − [8×(2^18−1) + 7×(2^10−1)]
      = 2_104_413 − [8×262143 + 7×1023]
      = 2_104_413 − [2_097_144 + 7_161]
      = 2_104_413 − 2_104_305
      = 108
```

这恰好等于**今天"快档"用例已经手工写死的数字**（`second_transaction_step_zero_layer0.rs:478`：`assert_eq!(tally.states, 108, ...)`）——快档的 `expand = segment.len() < 10` 这条启发式，在**这一份具体的段序列**上，数值效果与甲的语义判据一致（都排除了那 15 个大段，`10 < 10` 为假，长度恰为 10 的段同样被排除）。**这不代表启发式与甲判据在一般情况下等价**（L3 的职责：启发式只看长度，甲判据看语义，二者可能在别的段序列上分道扬镳——例如一个长度 12 的段如果全是 `JournalRecord`，启发式会把它误判为"不展开"而甲判据会正确判"必须展开"），但它说明：**若 8/7 段的组成假设成立，甲对这份 54 段序列的数值收益是巨大的**（2,104,413 → 108，降到约 5×10⁻⁵ 倍）。

**这个示意值不能套到背景材料给的 423 写/5,575,802,973 那组数字上**——那组数字本身在冻结副本里找不到出处（上一节的冲突），无法拆出"哪几段是 30 写、是不是纯 UnitWrite"。

### 本机 32 线程要跑多久（推的）

背景材料给的速率是"约每秒 13880 个（16 线程，与别的重活抢 CPU）"，据此推 32 线程"约 2.3 天"（背景材料自己标注"推的，没量过"）。若线性外推到线程数（这个外推本身没有被验证过，状态求值涉及内存分配与镜像构造，未必线性），32 线程速率 ≈ 13880 × 2 = 27760 个/秒：

- 若 `CF_甲 ≈ 108`（上面示意值成立）：`108 / 27760 ≈ 0.004` 秒——远低于任何有意义的挂钟成本，断点续跑（L7）在这个量级上没有必要。
- 若沿用背景材料的 5,575,802,973、且甲能把"5 段 30 写"（占 96%）整段约简掉：剩余 ≈ `5,575,802,973 × 0.04 ≈ 223,032,119` 个，`223,032,119 / 27760 ≈ 8036` 秒 ≈ **2.2 小时**。

两个估计相差 5 个数量级，**都只能算"推的"**——因为它们建在互相冲突、都没有被我验证过的两组输入数字上。这正是 L7"断点续跑要不要做"这件事本身的关键前提：若真实数字接近第一档（约简后秒级），续跑机制的收益很小；若接近第二档甚至背景材料给出的"未约简 32 线程 2.3 天"，续跑机制就是刚需（用户 2026-09-26 JST 08:4x 的原话已经把这个当刚需在提：详见 L7 一节）。**哪个数字对，需要主 agent 在 54 号 `--full` 上实测一次才能定，我不能跑这一步。**

### 乙之下重跑范围（与 L4 共用同一份推导，见上一节"乙之下一次只改 checker……"）

### 判定

| 子问题 | 判定 | 依据 |
|---|---|---|
| 闭式公式本身 | **一致**（甲的语义与今天的公式结构兼容，只是把"段"替换成"通过现判的段"） | `segments.rs:146-158`、`crash.rs:1497-1499` |
| 背景材料给出的具体数字（423 次写/54 段/5575802973） | **冲突**（与冻结副本自身的测试断言 279 次写/54 段/2104413 对不上，且冻结副本内部三处注释互相矛盾） | 见上方并排引文 |
| 32 线程挂钟估计 | **规则没说**（背景材料自己标"推的"，我给出的两档估计同样是推的） | 无实测支撑 |

### 推翻条件

跑一次 `cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture` 并读 `LAYER0B` 行的 `states=`、`closed_form=` 字段，与两组数字任一相符，即可判定哪一组是当前源码的真实值、哪一组是过时或来自别的版本。**这一步必须由主 agent 或崩溃验证员做**（受 `SINGLEFS_HEAVY_TESTS` 约束、且我这条腿明确被禁止跑 layer0 目标），复核不了。


## L7：断点续跑

### 今天的现状（现查）

`enumerate_layer0_in_state_slices`（`crash.rs:1728-1847`）已经把 `[0, state_count)` 切成区间片、工作线程按原子计数器 `next_slice_index`（`crash.rs:1791`附近）领片跑，跑完一片打一行 `LAYER0_PROGRESS`（`crash.rs:1801-1812`），但**没有任何东西把这一行以外的信息落到磁盘**——中途 kill 掉，下一次只能从 `state_count=0` 重新起跑。`.claude/gate.d/54-layer0-replay.sh` 的 `--full` 路径（`54-layer0-replay.sh:327-431`）同样没有断点：`trap`（`:337`）只在退出时清理临时目录与判红时删全绿标记，不写任何可续跑的中间状态。

切片长度今天**依赖线程数**：`state_slices`（`crash.rs:1556-1578`）在 `Layer0SliceLength::ScaledToWorkerThreads` 模式下，`states_per_slice` 由 `LAYER0_MINIMUM_SLICE_COUNT`（`crash.rs:1375`，64）、`LAYER0_SLICES_PER_WORKER_THREAD`（`crash.rs:1377`，16）与 `worker_threads` 共同决定（`crash.rs:1561-1565`：`slice_count = max(64, threads*16)`，`states_per_slice = ceil(state_count/slice_count)`）——**用 16 线程跑出来的切片边界，与用 32 线程跑出来的不是同一组区间**。这正是 L7 派发提示点名的风险之一（"切片按状态序号区间定、不随线程数变"）：今天不满足这条。

`Layer0Tally::absorb_following_slice`（`crash.rs:955-1015`）是纯加法合并（逐字段 `+=`，"第一处违例"只在 `self.first_violation.is_none()` 时才收下面片的值，`crash.rs`对应行），且合并顺序由 `waiting_for_earlier_slices`（`BTreeMap<usize, FinishedSlice>`，`crash.rs:1794`）严格按片号从小到大喂给 `absorb_following_slice`（`crash.rs:1814-1828`），与线程数、完成次序无关——这是**续跑在算法层面可行**的关键前提：只要能拿到"每一片自己的 `Layer0Tally`"，不管这些片是这一次跑出来的还是上一次留下来的，按片号顺序喂给同一个 `absorb_following_slice` 循环，结果必然与一口气跑完逐字节相同（这条不变量今天已经有测试钉着：`second_transaction_step_zero_layer0.rs:524-579` 的 `one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread`，只是它验的是"同一次跑、不同切法"，不是"跨两次进程的续跑"）。

`--full` 路径调用 `enumerate_layer0_versions`（走 `enumerate_layer0_selecting_versions` → `enumerate_layer0_in_state_slices`，`observe_state=None`，`CountOnly` 保留档，`crash.rs:1858-1865`），**不需要观察者**——续跑只需要持久化每片的 `Layer0Tally`，不需要持久化逐状态的镜像或恢复报告，数据量可控。

### 改法（对冻结副本 `crates/` 与 `.claude/gate.d/54-layer0-replay.sh` 的改动）

**步骤 1——切片边界与线程数解耦**：`crates/singlefs-harness/src/crash.rs`，`state_slices`（`crash.rs:1556-1578`）新增一种 `Layer0SliceLength` 变体（`crash.rs:1407-1412` 处扩枚举），例如 `ResumableFixedCount(NonZeroU64)`：`states_per_slice` 取一个只依赖 `state_count` 的固定值（不读 `worker_threads`），`Layer0Parallelism::from_environment`（`crash.rs:1428-1469`）在续跑模式下改用这个变体而不是 `ScaledToWorkerThreads`。这样切片边界只由 `state_count` 决定，16 线程与 32 线程续跑同一批状态时算出同一组区间——工作线程数仍然可以变（`next_slice_index.fetch_add`，`crash.rs`工作线程循环那一段，是与线程数无关的任务队列，不需要改）。

**步骤 2——每片完成时追加进度行**：在 `enumerate_layer0_in_state_slices` 的合并循环里（`merged.absorb_following_slice(in_order.tally)` 那一行，`crash.rs:1827`），紧接着这一行之后，若调用方传入了一个进度文件句柄，把 `in_order.slice_index`、`in_order.tally` 序列化成一行 JSON（或与 `LAYER0_PROGRESS` 同风格的 `key=value` 行）追加写入并 `fsync`——**只序列化 `Layer0Tally`（纯计数 + 字符串），不序列化 `observed_states`**（`--full` 路径本来就是 `None`，见上一节）。这是新增的一个可选参数（`Option<&mut File>`），不改变现有调用方（`observe_state`那几个测试用例）的行为。

**步骤 3——进度文件的位置与命名**：放 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}` 下（与 `.claude/gate.d/59-crates-mutation-replay.sh:32-33,97` 的编译产物缓存同一约定：跨轮复用的东西放 `GATE_CROSS_RUN_TMPDIR`，不放会被每轮清空的 `TMPDIR`），文件名按输入哈希（L4 拆分之后是 `second-transaction` 那个哈希）与切片方案参数（固定的 `states_per_slice` 值）一起编码，例如 `singlefs-layer0-progress.<输入哈希>.<states_per_slice>`——**输入变了或切片方案变了，文件名就不同，天然不会被误用**（不需要在文件内容里再判一次，命名本身就是判据，这与全绿标记按输入哈希分格的既有做法同构，`54-layer0-replay.sh:29,332`）。

**步骤 4——末行半截怎么认**：写入方每行末尾写一个该行自己的 CRC32C（`crates/singlefs-core::checksum::crc32_castagnoli`，与 D23 已定项 11 记录反向链同一个校验和函数，不新引入依赖），格式 `slice=<n> states=<...> ... crc=<十六进制>`；读入方逐行校验，第一行**CRC 对不上或整行不完整（没有换行结尾）就在那一行截断**，只承认它之前的完整行——对应"进程被杀、最后一行写了一半"的场景（append 到普通文件不是原子的，最后一次 `write` 可能只落了一部分字节）。

**步骤 5——恢复时的跳过逻辑**：`enumerate_layer0_in_state_slices` 新增入参 `resume_from: Option<&[FinishedSlice的tally]>`（按片号索引的数组，缺的片留空）；`next_slice_index`（`crash.rs`）的工作线程领片循环改成：先把 `resume_from` 里已有的片直接喂给合并循环（不再派给工作线程），再让工作线程从 `next_slice_index` 起领**没有**记录在 `resume_from` 里的片。

**步骤 6——54 号脚本改法**：`.claude/gate.d/54-layer0-replay.sh` 的 `--full` 路径（`54-layer0-replay.sh:340-396`）在跑 `second_transaction_step_zero_layer0` 之前，先看进度文件存不存在（按步骤 3 的命名，同一个输入哈希+切片方案）：存在就把它的路径通过环境变量（例如 `SINGLEFS_LAYER0_RESUME_FILE`）传给 `cargo test`，测试内部读取步骤 5 的 `resume_from`；成功句里加一行"续跑：从第 N/M 片接着跑"或"全新起跑"（对应派发提示"54 号怎么报『这一次是续跑的』"）。新增 `--full --fresh`（或环境变量 `SINGLEFS_LAYER0_FORCE_FRESH=1`）忽略已有进度文件、强制从头跑（对应"强制从头跑的开关"）。

### 逐条回答派发提示的子问题

- **续跑之后的计数行与一口气跑完的逐字节相同吗**：算法上应当相同，理由是 `absorb_following_slice` 的合并只依赖"哪一片、什么顺序、每片自己的 tally"，与"这片是刚跑出来的还是从文件读回来的"无关（上一节已论证）。**拿什么用例钉住**：新增一个测试，跑一遍 `enumerate_layer0_in_state_slices`（不开续跑）拿到 `tally_a`；把跑到一半时收集到的每片 `FinishedSlice` 落成一份"假的进度文件"，再用它作为 `resume_from` 跑第二遍（只让工作线程跑剩下没记录的片）拿到 `tally_b`；断言 `tally_a == tally_b`（`Layer0Tally` 需要能整体比较——现查它今天有没有 `derive(PartialEq)`：`crash.rs:891` 附近的 `#[derive(...)]`，测试文件 `second_transaction_step_zero_layer0.rs:575-578` 已经在用 `assert_eq!(one_state_slices_tally, one_slice_tally)`，说明 `Layer0Tally` 已经可比较，这条测试写法不需要新加派生）。这条**没有实现**，是本轮判决之后的落地工作，不是我现在就能核实的结论。
- **进度文件写到一半被杀（末行半截）怎么认**：按步骤 4，逐行校验 CRC，第一处不完整就截断、只认它之前的行。
- **输入变了、切片方案变了、线程数变了各怎么办**：输入变了或切片方案（`states_per_slice`）变了——文件名（步骤 3）就不同，旧文件找不到，自动等价于"没有进度文件"，从头跑；线程数变了——因为步骤 1 把切片边界与线程数解耦，线程数变化不影响文件名也不影响切片边界，旧进度文件继续有效，只是工作线程数变多或变少。
- **续跑会不会让一片被数两次或漏数**：不会数两次——`resume_from` 里已有的片不再派给工作线程（步骤 5），工作线程只领 `resume_from` 没有的片；不会漏数——`assert_eq!(merged_slice_count, slices.len(), ...)`（既有断言，`crash.rs:931-935`附近）今天已经钉住"合并的片数必须等于总片数"，续跑模式下这条断言不需要改，只需要保证 `resume_from` 的片 + 工作线程新跑的片二者并集恰好覆盖 `0..slices.len()`（`resume_from` 与"没有被领过的片"互斥，因为领片逻辑会跳过 `resume_from` 里有的下标）。
- **强制从头跑的开关与 54 号怎么报『这一次是续跑的』**：见步骤 6（`--full --fresh` / `SINGLEFS_LAYER0_FORCE_FRESH=1`；成功句加"续跑：从第 N/M 片接着跑"一行）。

### 判定

| 子问题 | 判定 | 依据 |
|---|---|---|
| 今天有没有断点续跑 | **一致**（背景材料"今天……没有续跑"与现查相符） | `54-layer0-replay.sh:337`（trap 不写中间状态）、`crash.rs:1728-1847`（无落盘） |
| 切片是否随线程数变 | **一致**（背景材料把这个列为要解决的风险，我现查确认今天确实随线程数变） | `crash.rs:1556-1565`，`LAYER0_SLICES_PER_WORKER_THREAD`（`:1377`） |
| 续跑机制是否有算法基础 | **规则没说**（背景材料没有论证，我补充：`absorb_following_slice` 的结合律 + 既有的顺序合并逻辑已经是续跑的算法基础） | `crash.rs:955-1015`，`crash.rs:1794,1814-1828` |

### 推翻条件

若 `evaluate_state_slice`（`crash.rs:1617-...`，工作线程跑一片的函数）在某处使用了跨片的可变共享状态（例如全局计数器影响 `first_violation` 的判定顺序之外的东西），使得"哪些片是刚跑的、哪些是读回来的"会影响结果，则"续跑逐字节相同"这条结论不成立。现查：`evaluate_state_slice` 的入参只有 `base`（只读）、`writes`（只读）、`plan`（只读）与这一片自己的 `slice: Range<u64>`（`crash.rs:1617-1625`附近）。补查全仓 `static`/`thread_local!`：`grep -rn '^\s*static \|thread_local!' crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-harness/src/crash.rs` 命中 3 处——`singlefs-core/src/checksum.rs:11`、`singlefs-checker/src/lib.rs:60` 是只读初始化一次的 `OnceLock` 查表（CRC 表），`singlefs-core/src/block_device.rs:559` 是 `#[cfg(test)] mod tests` 内部一个与 layer0 无关的临时文件名计数器（`block_device.rs:562-567`，只给该模块自己的单测生成不重名的镜像路径，`MemoryPool`/`crash.rs` 走的是纯内存结构，不经这条路径）——三处都不影响 layer0 状态求值的确定性，没有找到会破坏续跑一致性的跨片可变状态。**复核不了的只剩**：`recover()`（`crates/singlefs-core/src/recovery.rs`）与 `check_pool_image()`（`crates/singlefs-checker/src/walk.rs`）内部我没有逐行读完，不能排除它们各自私有的、非 `static` 的某种缓存字段（例如结构体内部的 `Cell`/`RefCell`）产生跨调用副作用；这两份文件的全文没有作为这一格的输入材料给到我，留给主 agent 或 Opus 那条腿（分工表里 L7 也派给了 Opus 做续跑原型）核实。


## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| L1（甲的前提是否独立假设） | 规则没说 → 我补：等价于 I-7.4 | 甲对 UnitWrite 的前提就是 I-7.4 本身，不是新假设，但 I-7.4 有已知判定欠账（C556）与历史反例（H6） |
| L1（"journal 重放读到"半句） | 一致 | 由 D16 已定项 7 的写序 + 屏障切段保证，同一次发布内恒成立 |
| L1/正文第二节"零命中" | 冲突（附录已自证，我复核成立） | 8 处命中全在 `e158_root_choice_repair.rs`，与本轮无关，但字面"零命中"不准确 |
| L2（现判落在哪一步） | 规则没说（背景材料只问未答） | `crates/` 今天没有现判，`expand` 钩子存在但硬编码为 `true`（`--full`）或按段长度（快档） |
| L2（会不会共用被测代码） | 一致（写法方向对齐） | 提议函数复用 `reuse_is_not_proven_illegal_by_the_reclaim_predicate` 的独立复算风格，不调用 `singlefs-core` |
| L4（按流还是按段拆） | 规则没说（两种都是候选，未定） | 我给出按流拆的具体清单与安全条件（共享代码重叠登记） |
| L4（拆细会不会漏判） | 规则没说 → 我给出安全条件 | 4 个 crate 的 `src/` 与 `tests/common(_tree_split)/` 必须同时进两条流的清单 |
| L5（闭式公式） | 一致 | `CF_甲 = 1 + Σ[expand(i) ? (2^\|段\|-1) : 0]`，与今天的公式结构兼容 |
| L5（具体数字：423写/5575802973 对 279写/2104413） | **冲突**，答不出确数 | 背景材料的实测数与冻结副本自身的测试断言互相矛盾，且冻结副本内部三处注释（2/3/8 个 18 写段）互相矛盾 |
| L5（32 线程挂钟） | 规则没说，双方都标"推的" | 两档估计相差 5 个数量级（0.004 秒 对 2.2 小时），需要实测 |
| L7（今天有没有续跑） | 一致 | 现查确认 `54-layer0-replay.sh`、`crash.rs` 都没有落盘续跑状态 |
| L7（切片是否随线程数变） | 一致 | `state_slices` 的 `states_per_slice` 由 `worker_threads` 参与计算，今天确实随线程数变 |
| L7（续跑算法基础） | 规则没说 → 我补：`absorb_following_slice` 的结合律已经是基础 | 既有的顺序合并逻辑天然支持"部分片来自文件、部分片新跑" |

## 没做什么

- 不判 L3、L6（分给 Opus）与本地攻方的 L5 事实表逐格算数——按分工只负责闭式算法本身，不逐段套真实数据（因为真实数据本身就是本节的冲突所在）。
- 没有跑任何名字带 `layer0` 的测试目标，也没有跑 54 号门禁——派发提示的硬约束；因此 L5 的"背景材料 423 写/5575802973 对冻结副本 279 写/2104413"这条冲突**没有裁决谁对**，只并排列出。
- 没有读完 `crates/singlefs-core/src/recovery.rs` 与 `crates/singlefs-checker/src/walk.rs` 的全文（背景材料没有把它们列进"实现今天的样子"给出的观测范围），L7 的"跨片可变状态"推翻条件因此留了一角复核不了。
- 没有验证 L1"甲的前提"在 `crates/singlefs-core/src/allocator.rs` 里的实现是否真的严格遵守 D16 已定项 1 的回收谓词（这需要读完整个分配器实现或跑变异，分工表把"造历史攻前提"派给了 Opus）。
- L2、L7 提议的所有代码改动（新函数、新 `expand` 接线、切片方案解耦、进度文件）都**没有写进 `crates/`**——按定义我只能对冻结副本谈改法，不改动它，也不改主工作区（实三正在改）；这些改法是否被采纳、何时落地由主 agent 决定。
- L4 的提案清单（`stage-inputs.tsv` 两行新登记）没有写进 `.claude/gate.d/stage-inputs.tsv`——那是项目配置文件，不在我的写范围（报告文件、模型目录、草稿目录）之内。
- 没有生成 L5"套算法算一次示意值"一节里 108 这个数所依赖的真实分段数据（哪 15 个段确实是纯 `UnitWrite`）——这是一个建立在注释推断上的示意，不是我验证过的事实，已在正文标注。
- 模型目录 `research/prompts/m2-layer0-scale-r1-sonnet-model/` 本轮空着：这一格的任务是读代码给改法与算法，没有产出需要单独存档的模型/脚本；已建立空目录（写范围内）。
