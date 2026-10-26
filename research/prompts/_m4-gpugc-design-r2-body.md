# m4-gpugc-design-r2：里程碑四七·2 GPU 垃圾回收设计轮，第二轮正文（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 Q9 Q13 Q17 F7 -->

## 一、题面

被判的仍是第一轮正文 `research/prompts/_m4-gpugc-design-r1-body.md` 的七格 G1–G7，加第一轮判决新冒出的一格 G8（GC 挪不挪出用户改动的调用线程）。第一轮判决 `research/prompts/m4-gpugc-design-r1-main-verification.md`，岔路单 `research/prompts/m4-gpugc-design-r1-forks.md`。第一轮打中的都挂着，没写进 kb 正文。

第一轮判决第三节第 1 条要的代价数已经量了：E185（GC一批搬迁的挂钟拆分） 第 3 次重跑，跑前登记 `research/prompts/e185-r3-prereg.md`，三段产物都在 `research/results/`（第四节）。第二轮照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第二轮分工：攻第一轮的修补本身、产生结论的方法、从未被看过的地方；辩方腿复核第一轮判决。

用户的前提不变（原话在决策变更史 D26 节 2026-10-19 那一块）：「gpu垃圾回收需要加入。不需要讨论。」值不值按 D26（后台整理与放置回收） 已定项 13 第 7 条算省下的主机 CPU 时间。

## 二、实现今天的样子（主 agent 的观测，2026-10-24）

方案按 `crates/` 今天的实现来谈。主 agent 现读过的行：
- 用户改动的发布函数里、返回之前起一趟整理：`crates/singlefs-core/src/mounted_session.rs` 第 1927–1933 行（`compaction_starts_after_a_user_change` 判真时调 `run_compaction_on`）。
- 整理入口：同一文件第 1956 行 `run_compaction`、第 2588 行 `run_compaction_on`、第 2989 行 `advance_compaction_by_one_publish`。
- 区域扫描：`crates/singlefs-core/src/region_scan.rs` 第 204 行 `scan_region`。
- 一趟发布预算：`crates/singlefs-core/src/compaction_plan.rs` 第 808 行 `publish_budget_of_a_compaction_run`。
- 发布与逐条核计划：`crates/singlefs-core/src/transaction.rs` 第 9676 行 `publish_relocation_batch`、第 10138 行 `relocation_checked_against_the_version`。
- 产品代码里没有 GPU 依赖：`wgpu` 只在 `crates/singlefs-checker-tier/Cargo.toml`。
- 第一轮判决第一节核过的其余代码行照旧（腿引之前现查行号，行号可能挪了几行）。

## 三、第一轮判决的去向（摘要，原文以判决为准）

- G1：三处打中挂起（① 净收益符号缺 c_rt；② 发布那一半没有能放 GPU 又不碰共享事务层的部分；③ 今天的 GC 在用户改动的返回路径上跑，开 GPU 就进了前台路径）。
- G2：打中挂起（跨批攒被每批重扫、至多一条活意图、发布预算三条约束挡住；放宽 R 要改格式）。
- G3、G5、G7：没打中，只有脆弱点；按「一条腿只抽一次样不算一次观测」，这一轮另派一条同立场攻方腿再攻。
- G4：打中挂起（输入表准备那一类同输入对拍抓不到；CPU 只改位图与逐条核计划的关系没写清；N 的计数住内存）。
- G6：打中挂起（「前台有 fsync 在等」在今天的形态下观测不到）。
- 第一轮推的改法（零轮）：按设备类型筛卡、对拍从这一版起算、重挂时把不确定的条目直接交 CPU、挂载时用已知答案自检、GC 挪出调用线程。

## 四、E185 第 3 次重跑的读数（主 agent 的观测；数以产物为准）

| 段 | 产物 | 判定那一行 |
|---|---|---|
| 第一段 一次 GPU 提交的主机 CPU | `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`（8007 行） | `E7RESULT name=verdict segment=host_cpu_per_submission constant_or_anchor_mismatches=0 stop_clause_findings=0 cards_measured=2 card_failures=0 card_failure_items=none voided_forms_by_the_positive_controls=0` |
| 第二段 真卡单元级重算的不一致率 | `research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out`（317 行） | `E7RESULT name=verdict segment=recompute_disagreement constant_or_anchor_mismatches=0 stop_clause_findings=0 cards_measured=2 positive_control_failures=0 positive_control_failure_items=none two_cards_differ_on_the_same_input=false` |
| 第三段 按函数拆开的一批 CPU | `research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out`（1258 行） | `E7RESULT name=verdict segment=function_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none` |

同一天第一段另有一份无后缀的产物（390 行），是被别的计算进程占卡、等满上限停机的那一趟（末行之前 `E7RESULT name=stopped segment=host_cpu_per_submission reason=machine_not_back_to_normal_within_the_waiting_cap`），不是有效读数。

主 agent 现算的第一段透视（`host_cpu_per_submission_point` 行，线程钟，按候选 × 卡 × 等法 × 门槛种类数判定；命令：对产物逐行取 `candidate index wait clock threshold_kind judgement` 计数）：
- 候选 A（只放扫描里的校验和）、B（A 加选段打分）、D（只放释放前核）：两张卡、三种等法、两种门槛下，没有一格判 `not_losing`；卡 1 上全判 `losing`，卡 0 上 `losing` 与 `unstable` 各占一部分（B 在卡 0 上另有 `not_losing_lower_bound` 4–14 格）。
- 候选 C（甲、乙、丙全放）：是唯一出现成批 `not_losing` 的候选。卡 0 上每种等法 14–38 格 `not_losing`、0–3 格 `losing`；卡 1 上 6–24 格 `not_losing`、12–19 格 `losing`。
- 拟合：`fixed_and_per_byte_fit` 18 行，`maximum_relative_residual` 有超过 20% 的（例 `index=0 form=shared wait=blocking gap_milliseconds=0 … maximum_relative_residual=0.217393 … fit_not_linear=true`）；`merging_shape` 432 行，类别 `one_submission_per_batch` 217、`one_submission_per_intent` 78、`merge_across_intents` 1、`fit_not_linear` 1、`unstable` 135。

第二段：每张卡 2097120 个单元次、`first_round_kernel_disagreements=0`、`upper_bound_at_95_percent_when_zero=1.428497e-6`；`determinism_replay … all_identical=true` 四行；`both_cards_same_input cards=0,1 units=16384 identical=16384 cards_differ=false`。

第三段：八行 `function_split_summary` 的 `region_scan_share_median` 在 0.001666–0.031326；读这一版、克隆两项的份额见同一份产物同几行。

腿要引的数一律在产物里现取整行，不从这一节抄。

## 五、格

### G1　GPU 上算哪几段（攻第一轮的修补与新读数）
- 第四节的透视是不是说候选 A、B、D 出局、只剩 C？反过来找：在哪种读法（门槛种类、等法、哪张卡、`unstable` 怎么算）下这个结论翻面。
- 候选 C 放上 GPU 的那一半里有乙（区域扫描里的单元头解析与映射反查）。第三段量的区域扫描份额只有 0.17%–3.1%：C 赢的那一部分是从哪一段省下的，算一遍（引产物行）。
- 卡 1 上 C 也赔得多：「全部显卡」那条用户定案（D26 已定项 13 第 4 条）下，一张赔钱的卡要不要用，与 G3 一起答。

### G2　攒批的形态
- 第一段的拟合有超过 20% 残差的点：照登记 J6 那几点写 `fit_not_linear`。拿拟合结论判「乙-1 / 乙-2 / 丙」站不站得住，算法本身靠不靠得住。
- `merging_shape` 绝大多数是 `one_submission_per_batch` 与 `one_submission_per_intent`，跨意图只有 1 格：第一轮挂着的「跨批攒被三条约束挡住」要不要因此改判。

### G3　多卡分活（第一轮没打中，再攻）
攻正推的推荐（PCI 总线号、已验厂商独显、按窗口轮转），加第一段两张卡判定差得多这一条新读数。

### G4　两份核比对与 N
- 第二段零次不一致、上界 1.43×10⁻⁶：N 取几、不确定计数住内存还是盘上，第一轮挂着的几处要不要因这个数改判。
- 输入表准备那一类（第一轮 g4 模型）第二段量不到：说清第二段的数罩到哪、罩不到哪。

### G5　控制接口（第一轮没打中，再攻）
### G6　三个默认值（第一轮打中挂着，攻它的改法）
### G7　GPU 代码住哪、用什么写（第一轮没打中，再攻）

### G8　GC 挪不挪出用户改动的调用线程（第一轮判决新冒出的前提）
- 第一轮判 G1-③、G5、G6 都挂在这一件上。攻这一前提本身：挪出之后，`mounted_session.rs` 第 1927–1933 行那种「改动返回之前跑一批」的保证（例如空间准入、抬 F 的次序）有哪些会丢；`run_compaction` 已有的显式入口（第 1956 行）够不够当后台入口。
- 不挪出、只在调用线程上用 GPU 的那一条路，第一段的数（阻塞等与睡眠等）给出的前台挂钟增量是多少。

## 六、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | G1–G8 | 复核第一轮判决：每一处「打中，挂起」够不够得着、是不是同样打中所有替代方案；每一处「没打中」是不是漏判；第一轮推的改法（零轮）里哪一条站得住、哪一条被第四节的新读数推翻。替被判出局的候选辩护：A、B、D 在第四节的透视下有没有哪一种读法能活 |
| 云端攻方（Opus） | G1–G8 | 第二轮攻法：攻第一轮的修补本身（第三节末那五条改法）、产生结论的方法（第四节透视的口径、拟合、`unstable` 的读法、只量两张卡）、从未被看过的地方。G3、G5、G7 必须各给反例或写明构造不出。不许重复第一轮攻方已攻过的角度（第一轮报告 `research/prompts/m4-gpugc-design-r1-opus-output.md`）。能用算术或小模型核的，在自己的草稿目录里核，报告写命令与输出 |

## 七、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写 `crates/` 下的路径与行号（主工作区，开工快照在 `research/prompts/m4-gpugc-design-r2-snapshot/sha256sums.txt`）。
- 引产物整行抄，带文件名。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；要编译的经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，用自己的编译目录，跑完删。不用第二台。
- 报告写进 `research/prompts/m4-gpugc-design-r2-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 交岔路的地方写成表：候选（各自的定义）、翻面观测、够判条件；不写倾向。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
