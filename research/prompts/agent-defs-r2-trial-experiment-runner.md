# E67 重新试跑报告

工作目录：`/tmp/claude-1000/agents/trials2/experiment-runner/repo/`（仓副本，带 `.git`）。
跑前登记：`research/prompts/e67-r2-prereg.md`。真仓 `/home/fy5090/code/singlefs` 一个文件都没碰。

（注：系统工具拒绝了我把这份报告落成 `.md` 文件的尝试，`/tmp/claude-1000/agents/trials2/experiment-runner/report.md` 这个路径没有被写出，报告全文只在这条 SubagentHandback 消息里。）

## 0. 开跑前 ps

`ps aux | grep -E 'cargo|qemu-system|gate\.sh'` 看到本机（宿主，不是这个副本）有一条
`cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0`
在跑（pid 231695，05:52 起）。按派发的例外条款（定义第 1 步允许这次照常编译，加 `nice -n 19`），
继续往下做，全部编译/跑测试都加了 `nice -n 19`。

## 1. 单测

`grep -c '#\[test\]' research/e7-index-bench/src/bin/e67_device_subset.rs` → **9**。

`cd research/e7-index-bench && nice -n 19 cargo test --release --bin e67-device-subset`：

```
running 9 tests
test tests::negative_control_width_two ... ok
test tests::skew_actually_skews ... ok
test tests::absolute_object_pinned_intact_matches_pair_ratio ... ok
test tests::absolute_object_pinned_is_22_over_28 ... ok
test tests::absolute_round_robin_intact_is_four_over_28 ... ok
test tests::golden_pinned_by_emptiest ... ok
test tests::absolute_lost_fraction_is_three_over_28 ... ok
test tests::positive_control_full_width_every_arm ... ok
test tests::conservation_lost_fraction_equal_across_arms ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

9 单测，全过，与 kb 页登记的「9 单测」一致。

## 2. 变异表三个数

变异表 `research/mutations/e67_device_subset.tsv`（4 条，不加不减，与登记第九节一致）。

**跑法有一处与派发提示给的定义文本不符**（细节见文末「试跑观察」）：定义第 3 步写「在仓根下」跑
`mutate.sh`，但在这个副本今天的源码状态下，在仓根下跑会直接报错，跑不出任何数：

```
$ cd 仓根 && nice -n 19 bash research/scripts/mutate.sh e67-device-subset \
    research/e7-index-bench/src/bin/e67_device_subset.rs research/mutations/e67_device_subset.tsv
mutate: 基线就是红的，先修好再来
```

改成 `cd research` 之后按相对 `research/` 的路径跑，才跑得动：

```
$ cd research && nice -n 19 bash scripts/mutate.sh e67-device-subset \
    e7-index-bench/src/bin/e67_device_subset.rs mutations/e67_device_subset.tsv
基线：全绿
⏭  [M1_戊不固定每次重选] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
✅ [M2_命中只看一块盘] 红：absolute_object_pinned_is_22_over_28,absolute_object_pinned_intact_matches_pair_ratio,absolute_lost_fraction_is_three_over_28,golden_pinned_by_emptiest,absolute_round_robin_intact_is_four_over_28
✅ [M3_窗口宽度写死] 红：skew_actually_skews,absolute_lost_fraction_is_three_over_28,absolute_object_pinned_intact_matches_pair_ratio,absolute_round_robin_intact_is_four_over_28,absolute_object_pinned_is_22_over_28,conservation_lost_fraction_equal_across_arms
✅ [M4_丢数据按w计] 红：absolute_lost_fraction_is_three_over_28
已还原，基线仍全绿
```

**抓到 3（M2、M3、M4）、无效 1（M1）、没红 0**。与登记第四节「跑之前已经存在的数」
（2026-09-16 复跑读数：抓到 3、无效 1、没红 0）**一致，不需要单列**。

## 3. 产物与留存产物

`nice -n 19 bash research/scripts/replay.sh E67`（在仓副本根跑，脚本自己 `cd` 到 `research/`）：

```
实验 二进制                判定     说明
-------------------------------------------------------------------------
E67   e67-device-subset        字节一致 e67-device-subset-2026-08-31.out
-------------------------------------------------------------------------
结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）：
-------------------------------------------------------------------------
字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0
本轮输出：/tmp/singlefs-replay-1900016
```

退出码 0，**逐字节一致**。全绿且用的是自动分配的临时目录，脚本自己清理了 `$OUT_DIR`。

## 4. 完成标记

`replay.sh` 本轮的完成标记是它自己的收尾行「字节一致 1 ／ … ／结论断言不中 0」加退出码 0；
`mutate.sh` 本轮的完成标记是收尾那行「已还原，基线仍全绿」。两个都拿到了。
本轮没有产生新的 `research/results/` 文件——产物与留存产物逐字节一致，
按登记第十节「不一致才记『对不上』」的反面，一致就不改留存产物、不新写文件。

## 5. kb 写回

- `.claude/kb/experiments/67-一条条带参与哪几块盘.md`：在「口径与复跑」一节末尾补了一条
  「2026-09-17 复核重跑」，写了 replay 结果、单测数、变异三个数，并记录了 mutate.sh 正确跑法
  （`cd research && bash scripts/mutate.sh …`）与在仓根下会报错的原因。**没有改索引行**
  （`.claude/kb/experiments.md` 里 E67 那一行一个字没动，git diff 已核实：那份文件另有一行
  E152 的改动，不是我碰的，是这个共享副本里其他轮次留下的未提交改动）。
- `.claude/kb/experiments-history.md`：加了一条 `### 2026-09-17（其二）：E67（一条条带参与哪几块盘）
  复核重跑——产物与变异三个数不变`，改前/改后/依据三段齐全，指到跑前登记与 kb 页那一节。

## 6. 跑前登记的「修订」

**没有修订。** 登记第十二节「修订」保持空白，我没有改动 `research/prompts/e67-r2-prereg.md`
这份文件——单测跑完到跑变异表/产物之前，我没有发现任何需要补的判据或臂；
唯一的发现（mutate.sh 在仓根下跑不动）是在**产物已经跑出来之后**才碰到的（先跑的
`replay.sh`，后跑的 `mutate.sh`），按定义「产物跑过之后不改登记，交主 agent」，
只写进报告和 kb 页，不回填登记。

## 7. 门禁：登记给我的阶段

`awk` 按 `.claude/agent-common.md` 给的公式，从 `.claude/gate.d/stage-owners.tsv` 取出登记给
`experiment-runner` 的阶段：`15-research-build.sh 27-format-constants.sh 33-mutation-tables.sh
34-experiment-index-sync.sh 40-results-cited.sh 52-segment-registry.sh 80-absolute-assertions.sh
85-repro-command.sh 86-experiment-orphans.sh 87-replay.sh 88-quoted-result-lines.sh`。

按派发提示的例外，**15 号与 87 号这次不跑**；其余 9 个全跑了，逐个 `nice -n 19 bash .claude/gate.d/<文件>`：

| 阶段 | 退出码 | 原样末行 |
|---|---|---|
| 27-format-constants.sh | 0 | `✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）` |
| 33-mutation-tables.sh | 0 | `✓ 138 个实验二进制都有成形的变异表，1397 条变异的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）` |
| 34-experiment-index-sync.sh | 0 | `✓ 实验索引行与正文标题一致（索引 151 行、正文 151 份）` |
| 40-results-cited.sh | 0 | `✓ 已跑的实验都点了名或写明了产物未留存`（前一行 `✓ research/results 下的实验产物全部被 experiments.md 点名（293 个文件）`）|
| 52-segment-registry.sh | 0 | `✓ 比对了 5 处登记（表格 4 行 + 整条流提示 1 处，段序列与每段步骤种类多重集都与 e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out 的 name=segments 行逐字一致），跳过 0 条标预想的表格行；4 条标「装置钉住」的表格行不与产物比…` |
| 80-absolute-assertions.sh | 0 | `✓ 138 个实验各自至少有一条绝对值断言` |
| 85-repro-command.sh | 0 | `✓ 点了产物的实验都写了复跑命令（143 个）` |
| 86-experiment-orphans.sh | 0 | `✓ research 里的实验号在 kb 里都有正文（141 个）` |
| 88-quoted-result-lines.sh | 0 | `✓ kb 正文里整行抄的产物行都在产物里逐字找得到（553 行，对照 293 份产物）` |

9 个全部退出码 0（原样输出留在 `/tmp/claude-1000/agents/trials2/experiment-runner/draft/gate_*.log`）。
这些阶段是在我写完 kb 两处改动**之后**跑的，反映的是包含我这次改动的状态。

## 8. 写工具

- 新建/改动 `.claude/kb/experiments/67-一条条带参与哪几块盘.md`：`Read` 后用 `Edit`（旧串恰好命中一次）。
- 新建/改动 `.claude/kb/experiments-history.md`：`Read` 后用 `Edit`。
- `/tmp/claude-1000/agents/trials2/experiment-runner/draft/gate_*.log`：`Bash` 重定向写入（草稿目录，非仓内）。
- 尝试用 `Write` 建 `/tmp/claude-1000/agents/trials2/experiment-runner/report.md` 被系统拒绝（见上，报告改为全文放进这条消息）。
- 没有新建 bin 源文件、没有改动变异表（登记第九节明写「不加不减」）、没有新写 `research/results/` 产物文件、
  没有改 `research/scripts/replay.sh`（E67 那一行本来就对，不需要改）。

## 没做什么

- 没跑 15-research-build.sh、87-replay.sh（派发提示的例外，明写这次不跑）。
- 没判 E67 的结论（三条盘绑组更好那条）能不能推翻或确立 D2（RAID 条带策略），那是推论，要走三方。
- 没跑门禁全量（`gate.sh`），只跑了登记给我、且这次没被排除的 9 个阶段。
- 没提交、没有任何 git 写操作。
- 没有触碰真仓 `/home/fy5090/code/singlefs`。
- 没有去改 `research/scripts/mutate.sh` 本身（不在我的写范围内），也没有去改别的实验页
  里同样过时的 `mutate.sh` 仓根调用写法（E102、E103、E140、E146 等），只在 E67 自己的
  kb 页与 experiments-history.md 里如实记录、并留了一句给主 agent 判要不要统一处理。
- 没有对 E152、E142 等其他实验在这个共享副本里的既有改动做任何处理——那些是其他轮次
  留下的未提交改动，不是我这一轮的，`git status` / `git diff` 已核实并区分清楚。

## 试跑观察（定义哪里不清楚、做不下去、与实际对不上）

1. **`.claude/agents/experiment-runner.md` 第 3 步「在仓根下 … 跑完整张表」这句话，在
   这个仓 2026-09-14 加了仓根 `Cargo.toml`（里程碑「第一个事务」步 0 的脚手架）之后已经
   跑不通了。** 仓根 `Cargo.toml` 自己也声明了 `[workspace]`（成员是 `crates/` 下四个
   crate，注释里显式写「research/ 是实验 workspace，自己有 Cargo.toml，这里显式排除」）。
   而 `mutate.sh` 判断「能不能跑」看的是**当前工作目录**的 `Cargo.toml` 有没有
   `[workspace]`（`if [[ ! -f Cargo.toml ]] || ! grep -q '^\[workspace\]' Cargo.toml; then
   … exit 2; fi`），仓根这份同样满足，检查过得去；但接下来它把**当前工作目录整个**
   `rsync` 进副本再编译——在仓根跑，拷进去的就是仓根那个不含 `e7-index-bench` 的
   workspace，`cargo test --bin e67-device-subset` 报 `error: no bin target named
   e67-device-subset in default-run packages`，于是基线检查判「基线就是红的」，退出码
   2，一条变异都不会跑。**必须先 `cd research`**，把源文件与变异表路径都改成相对
   `research/`，才能跑通。
   ⚠️ **这不是只影响 E67**：我核实了仓里一份较新（2026-09-16 才更新）的实验页
   `.claude/kb/experiments/146-livelist条目按映射key定身份之后的宽度与代价.md`，它记的
   `mutate.sh` 命令仍是仓根路径的写法（`bash research/scripts/mutate.sh
   e146-livelist-entry-width research/e7-index-bench/src/bin/e146_livelist_entry_width.rs
   research/mutations/e146_livelist_entry_width.tsv`），我在这个副本上原样跑了一次做核实
   （不改任何文件，跑完退出码 2，未新建/未改任何文件），同样报「基线就是红的」——说明
   这条问题不是 E67 特有，是仓根加了 `[workspace]` 之后对所有按这个写法记录复跑命令的
   实验页都成立的一个环境变化，需要主 agent 决定要不要统一改 `mutate.sh` 本身（比如
   让它自己 `cd` 到 `research/`）或者统一改所有实验页的复跑命令写法。我没有在此基础上
   去改任何一份不属于 E67 的文件，只在 E67 自己的页面和 experiments-history.md 里如实
   记了这一条，并点名了受影响的至少四份实验页（E102、E103、E140、E146）供主 agent 定夺
   （这四个编号是从历史命令写法里认出来的，没有逐一重跑核实，只有 E146 那一份我实测过）。
2. **定义第 6 步的「阶段归属表登记给你的阶段」这句本身没问题**，`.claude/agent-common.md`
   给的那条 `awk` 命令直接可用，取出来的 11 个阶段与我自己核对 `.claude/gate.d/
   stage-owners.tsv` 里 `experiment-runner` 那几行完全一致，没有出入。
3. **「产物」在这次「重跑复核」场景下具体指什么，登记与定义都没有直接点破**，我是按
   `replay.sh` 跑出的临时输出（与留存产物比对用的那份）理解为「产物」的——它比对完就
   被脚本自己清理掉了，不落进 `research/results/`。这与「跑一次新实验」时「产物进
   `research/results/`」的正常含义不同（那种场景下产物就是要留存的新文件）。这次因为
   逐字节一致，没有新文件要留，判定过程本身没有歧义，但如果以后哪次重跑真的对不上，
   「文末失败条款该不该在这一步就新写一份产物文件」这件事，登记与定义都没有写清楚，
   建议以后重跑登记模板里补一句。
4. **主 agent 报告要求里的「报告路径」这个输入，与系统层面「subagent 不许用 Write 建报告
   文件」这条硬规则冲突**：我按输入尝试 `Write` 到
   `/tmp/claude-1000/agents/trials2/experiment-runner/report.md`，被工具当场拒绝
   （`Subagents should return findings as text, not write report files`）。这次改成把全文
   放进 SubagentHandback 消息里交付，`report.md` 这个路径这一轮没有任何文件产生——如果
   主 agent 确实要一份落盘的报告存档，这一条需要另想办法（比如主 agent 自己把这条消息
   存盘），定义里「报告路径与草稿目录」这个输入项对着这条硬规则是矛盾的。
5. 其余步骤（第 1、2、4、5、7 步）跟着做下来都对得上，没有卡住的地方。
