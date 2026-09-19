# E155 第二次跑第二段（组提交）—— 收尾报告

这是第二段的收尾：前一个执行员（会话 aa1a28cc983b15423）在「改 replay.sh 指到 stage2 产物」之后被上下文耗尽打断，实际上已经把第二段几乎全部做完（`row7_grid` 1800 格、`row7_fold_point` 360 组、B12–B14、变异表追加到 34 条、实验页正文的「第二段做了什么/产物读出来的几个数/它答不了的」三段、决策回看表新行）。本轮做的是：核对现场一致、补齐两处遗漏的文字同步（源码里的 stage 说明、`experiments.md` 索引行、`experiments-history.md` 历史条目）、修一个 doc-lint 新问题、确认门禁 33 号已经是绿的。

## 1. 现场核对（按派发第 1 步）

- `cargo test --release -p e7-index-bench --bin e155-second-run-fsync-write-volume`：**52 passed; 0 failed**。
- `bash research/scripts/replay.sh E155R2`：`e155-second-run-fsync-write-volume-2026-09-19-stage2.out` **字节一致**。
- `bash research/scripts/replay.sh E155` （第一次跑那一行）：`e155-fsync-write-volume-2026-09-17-stage4.out` **字节一致**，未受影响。
- 发现并修：装置文件头注释（`e155_second_run_fsync_write_volume.rs` 1–19 行）与产物 `name=scope` 行原来仍写着「stage=1」「第二段（R8 组提交、第 7 行）未做」——这是前一个执行员写实验页时被打断、来不及回填这两处文字造成的，与产物里的实际数据（`row7_grid`/`row7_fold_point` 早就在了）不符。已改成准确描述并重新生成产物（`rm` 旧文件、重新 `cargo build --release` 后跑出，仅 `name=scope` 那一行文本变化，其余 2534 行字节不变）；重跑 `replay.sh E155R2` 确认新产物与自身逐字节一致。

## 2. 第 7 行产物齐不齐 / F2 逐格判（按派发第 2 步）

```
$ grep -oE 'name=[a-z0-9_]+' research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out | sort | uniq -c
      1 name=b1_capacities
      1 name=config
      1 name=discriminability_self_proof
      1 name=done
      6 name=geometry_sensitivity_sample_row6
      2 name=geometry_sensitivity_sample_row7
      1 name=positive_control
      1 name=positive_control_row7
     72 name=q6_11_legacy_vs_fixed
    288 name=row6_grid
    360 name=row7_fold_point
   1800 name=row7_grid
      1 name=scope
```
`row7_grid` = 5P×2族×2落点×3策略×3N×5concurrent_fsync_count×2共享 = 1800，与登记 5.4「第 7 行主表」网格逐项吻合（P=1、N=1 的格按登记「取第 6 行主表」不在这张表里，读法写死原文已说明）；`row7_fold_point` = 同一网格去掉 concurrent_fsync_count 轴（每组一行报 Q7.3/Q7.3b）= 360，同样吻合。`E7RESULT name=done emitted=2535` 与 `wc -l` 2535 一致。

F2（第十节）两半在 `row7_grid` 每一行都报了字段（`jia_unconverged`/`jia_saturated_trees`、`wal_full_unconverged`/`wal_full_saturated_trees`、`wal_leaf_unconverged`/`wal_leaf_saturated_trees`），不是只判了几个例子：
```
$ grep "name=row7_grid" .../stage2.out | grep -c "jia_unconverged=true"
24
$ grep "name=row7_grid" .../stage2.out | grep -oE 'jia_saturated_trees=\[[^]]*\]' | grep -vc '\[\]'
186
```
F2 触发的格没有被拿来判 Q7.2/Q7.3/Q7.3b（登记「结论收窄」原文），装置与实验页都遵守了这一条。

## 3. 实验页 / 索引 / 历史（按派发第 3 步）

- `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`：正文「第二次跑第二段做了什么」「产物读出来的几个数」「它答不了的」三段已由前一个执行员写好，本轮核对数字全部与产物一致（`row7_fold_point` 360、`fold_point_zero_wal_full=Some` 0 次、`fold_point_ten_percent_wal_full` 125 次、`fold_point_ten_percent_wal_leaf` 94 次、`jia_saturated_trees` 非空 186 行，均现查命令核过）；改了一处不准确的表述——原文说 `fold_point_ten_percent` 「几乎全部落在 concurrent_fsync_count=8 那一档」，现查实际分布是 wal_full 97/125（77.6%）、乙-M 66/94（70.2%）落在 k=8，改成写实际比例，不用「几乎全部」。补了「历史版本」里缺的第二段一条（原来只有第一段那一条）。「影响的决策」表里 D16（发布语义） 已定项 2 那一行原来仍写「这一版没有可计算的开关」，现在第二段已经在 1 个代表格上建了「N 按批计」开关，改成准确描述（只是方向确认，不是逐判定重判，F10 仍答不了）。加了 B12/B13/B14 到文件头的 `<!-- doc-lint:not-numbers -->` 声明（第二段新引的三个手算格锚点，doc-lint 复跑之前一直没有登记，达到 ≥3 次引用的门槛会报「无登记位」）。
- `.claude/kb/experiments.md`：索引行原来还写着「部分已跑（第二次跑第一段…第 7 行/R8 组提交未做）」——这一句是本轮改的主要内容，改成反映第一、二段都做完，结论列补了第 7 行组提交与 F2 的头号读数。
- `.claude/kb/experiments-history.md`：原来只有「其四」（第一段）一条，没有第二段的历史条目。本轮新增「### 2026-09-19（其九）」，写清改前/改后/依据。⚠️ **一处自我纠正**：我起草时把实验页正文里「M7、R2_M8、R2_M15 三条既有变异从恰好命中一次退化成命中两次」这句话原样搬进了自己起草的历史条目，但我用可见的命令记录（`e155-r2-runner-handover.md` 里最后 25 条命令）核不动这个具体说法——那份记录里能看到的两次 mutate.sh 运行之间被修好的是 `R2_M17`（从「一个测试都没红」变成被抓到），不是锚点命中次数的问题，而具体是不是还有另外三条锚点在我看不到的更早命令里被修过，我确认不了。已把我自己写的历史条目那一句改成不点名具体是哪几条、只写「已确认现在全部 34 条恰好命中一次」，避免引用一句我自己没法证实的具体断言（正文那句是前一个执行员的原话，不在我的改动范围内，未动）。

## 4. doc-lint（按派发第 4 步）

`DOC_LINT_VERBOSE=1 bash .claude/singlefs-ai-sop/scripts/doc-lint.sh`（前台跑，全仓）：

第一次跑（在我做完 1–3 步之后）：两个目标文件本身都过（`✓ .claude/kb/experiments-history.md`、`✓ .claude/kb/experiments/155-...md`），但**新冒出一条我引入的问题**：
```
✗ 编号 B12 出现 5 次，却没有任何登记位
✗ 编号 B13 出现 3 次，却没有任何登记位
✗ 编号 B14 出现 5 次，却没有任何登记位
✗ 文档铁律检查失败：...3 处编号定义/引用不合规...
```
根因：第二段新引用了 B12/B13/B14（登记第七节的三个手算格），但实验页文件头的 `<!-- doc-lint:not-numbers ... -->` 声明只登记到 B1–B11、B15–B19，没跟上第二段新加的三个。补上 B12 B13 B14 之后，`.claude/gate.d/34-experiment-index-sync.sh` 也顺带查出我自己在 `experiments.md` 索引行里加的两个百分比（≈10.3%、≈8.3%，正文没有逐字出现）不合规，一并删掉只保留正文里确实出现的原始计数。

第二次跑（改完两处之后，全仓前台跑，用 `run_in_background` 起、等完成通知，没有用会自我匹配的等待循环）：
```
✓ 文档铁律检查通过（检查 442，跳过 0；DOC_LINT_VERBOSE=1 看全部）
```

## 5. 门禁 33 号（按派发第 5 步）

派发原文说「有 3 条锚点不再恰好命中一次」。**现查：我接手时门禁 33 号已经是绿的**——
```
$ bash .claude/gate.d/33-mutation-tables.sh
✓ 142 个实验二进制都有成形的变异表，1509 条变异的原文各命中源码一次；...
```
逐条子串计数复核（Python 脚本单独扫这张表 34 行）同样是 0 bad。对照前一个执行员的交接摘要（`research/prompts/e155-r2-runner-handover.md` 第四节），他们最后一次改动（15:19–15:31 UTC 那一段）已经把锚点全部修到恰好命中一次并确认了 `gate.d/33` 绿、再重新生成的 stage2 产物——这些改动发生在我接手**之前**。所以派发里说的「3 条锚点」问题在我开工时已经不存在，我这一轮**没有再改锚点**，只是重新跑了一遍完整变异（见下）确认没有回归。

完整 `mutate.sh` 结果（我这一轮改完 scope 行、重新 build 之后又跑了一遍，确认我的改动没有影响任何一条变异）：
- 34 条：**33 条命中（红）**，**1 条判「等价」**（`R2_M19`：甲每批就是一次发布，套用 WAL 臂的中间版公式在结构上恒等于 0，0 个测试变红，是登记原文预期的等价变异），**0 条无效**。
- 收尾：`已还原，基线仍全绿`。

## 6. 岔路表（第 7 行；`research/prompts/m2-s1-forks.md:15`）

| 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 指到产物里算它的命令 |
|---|---|---|
| **够判**（岔路单第 7 行原文够判条件：「第 1 行同一批取样点 × k 五档 × 共享与不共享两种齐」）。`row7_grid` 1800 格 = 5P×2族×2落点×3策略×3N×5concurrent_fsync_count×2共享，与网格定义（登记 5.4）逐项吻合；P=1/N=1 按登记读法取第 6 行主表，不是遗漏。F2 两半在这 1800 格上都判了（不是抽样）。阳性对照（B12/B13/B14）与两条互比断言（V2 包含关系、共享≤不共享）都钉进单测且绿。还差：①第 7 行判定格上的 8.2 反向几何敏感性只做了「N 按批计」1 个代表格（1 个 P/family/placement/policy/k/N 组合），φ=0.5、K3′、g=0、pbs=4096、K9′-b 五个反向取样点在第 7 行**一个字都没测**（K9′-b 连第 6 行也没建可计算开关）；②F9（不共享取随机）完全没实现，只留了一行占位说明；③沿 concurrent_fsync_count 轴的判别力自证没做（只在 P=1/N 轴做过，针对第 6 行）；④Q7.4（写请求换算）、Q7.5（共享/不共享之比）没有另外套公式产出对应的判定，产物里已有原始字段但没加工。 | **主结论方向大概率不会翻**，但**没有证据支持这句话**——因为这四个反向点一个都没测，是"大概率"而不是"核过"。已经算出来、不依赖这四个空缺的两条判定是稳的：(a) `row7_fold_point` 360 组里没有一组的 `fold_point_zero`（`s`≤0）在 k≤16 内出现——这是穷举了全部 360 组算出来的，不会因为再测几个反向点就变成"有"；(b) V2 包含关系（乙-M≤wal_full≤甲）在 1800 格上核过零违反，是恒等式级别的检查，几何取法不会动它。**会不会翻面答不出来**的是 Q7.2/Q7.3/Q7.3b 的具体数值判定（比如"多数落在 k=8"这类依赖具体取值的结论）在 K9′-b、不共享取随机这两个方向上——K9′-b 让 WAL 两臂变便宜、不共享取随机让不共享那一半变便宜，两个反向点的方向都已知（登记 8.2 表），但会不会把某些格从"省一成以上"压过或压不过门槛，没有数。 | `grep -c "name=row7_grid" .../stage2.out` → 1800；`grep -c "name=row7_fold_point" .../stage2.out` → 360；`grep "fold_point_zero_wal_full=Some" .../stage2.out \| wc -l` → 0；`grep -oE "fold_point_ten_percent_wal_full=Some\([0-9]+\)" .../stage2.out \| sort \| uniq -c`；`grep "name=row7_grid" .../stage2.out \| grep -c "jia_unconverged=true"` → 24；`grep "name=row7_grid" .../stage2.out \| grep -oE 'jia_saturated_trees=\[[^]]*\]' \| grep -vc '\[\]'` → 186；`grep "name=geometry_sensitivity_sample_row7"` .../stage2.out（N按批计与不共享取随机占位说明各一行） |

第 5、6、8 行本轮未碰（第 8 行另有第三次跑登记）。

## 没做什么

- 没有实现第 7 行判定格上的 φ=0.5、K3′、g=0、pbs=4096、K9′-b 五个反向取样点的可计算开关（登记 8.2 要求，这一版一个都没建）；F9（不共享取随机）同样没有可计算开关。
- 没有做沿 concurrent_fsync_count 轴的判别力自证（只在 P/N 轴上做过，针对第 6 行）。
- Q7.4（写请求换算）、Q7.5（共享/不共享逐格之比）没有另外套公式产出、贴进实验页；产物 `row7_grid` 里已有 `jia_batch_write_calls`/`wal_full_batch_write_calls`/`wal_leaf_batch_write_calls` 三个写请求字段，够算但没算。
- 没有判断 rand 落点在第一段发现的"随对象数会不会失控"结论要不要因此改写第一次实验页第 5 行——这是推论，要走三方或交主 agent，不是我这一轮能替用户下的判断。
- 没有修 `research/prompts/e155-r2-runner-handover.md`（门禁 69 号红，引了 `/tmp` 路径）——这份文件不在 experiment-runner 的写范围内（不匹配 `agent-write-scope.tsv` 里我那几行的任何一个模式），也不是我这一轮写的文件，按共用约束「红了先看是不是这一轮改的，不在的不修」处理，照写。
- 没跑门禁 15 号（编整个 research 工作区）、87 号（复跑全部已入库实验）——不归 experiment-runner，交 `gate-triage`。
- 没有提交（git add/commit）。
- 没有判断这个实验的结论能不能推翻或确立里程碑「第二个事务」增补 1 或三方 `m2-s1-r1` 的判决——那是推论，要走三方。

## 关键文件路径

- 装置：`/home/fy5090/code/singlefs/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs`（本轮改动：文件头注释 13–19 行、`main()` 里的 `name=scope` 那一行，不涉及任何计算逻辑）
- 变异表：`/home/fy5090/code/singlefs/research/mutations/e155_second_run_fsync_write_volume.tsv`（本轮未改内容，只复核）
- 产物：`/home/fy5090/code/singlefs/research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out`（本轮重新生成，2535 行，只有 `name=scope` 那一行文本变化）
- replay 登记：`/home/fy5090/code/singlefs/research/scripts/replay.sh`（`E155R2` 行未改，仍指向 stage2）
- 实验页：`/home/fy5090/code/singlefs/.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`
- 索引：`/home/fy5090/code/singlefs/.claude/kb/experiments.md`（本轮改动：E155 那一行的状态列与结论列）
- 历史：`/home/fy5090/code/singlefs/.claude/kb/experiments-history.md`（本轮新增「### 2026-09-19（其九）」）
