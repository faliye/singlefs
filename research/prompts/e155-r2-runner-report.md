# E155 第二次跑第一段 —— 执行报告

## 单测
命令：
```
cd /home/fy5090/code/singlefs/research && cargo test --release -p e7-index-bench --bin e155-second-run-fsync-write-volume
```
结果：`test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

## 停机条款（跑产物之前应做；本报告如实记录顺序上的偏差）
- **S1**（甲 P=1/F1/seq/k=1 一格与 `crates` 对拍）：命令 `cargo test -p singlefs-harness --test second_transaction_supplement_one_write_accounting` → `test result: ok. 3 passed`（含 `first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table`）。**顺序偏差如实报**：这条命令是在产物已经生成、实验页已经写完之后才补跑确认的，不是产物之前；确认结果是绿的，不影响已交付的结论，但流程上没有照“先 S1、后产物”的顺序做。
- **S2**（`crates/` 有没有在跑的时候变）：登记里 3.1 记的六个文件 sha256，与我这次收尾时现取的完全相同（`090665c3…`/`50a19aa7…`/`64060917…`/`6bbe70ff…`/`894d534e…`/`65afb316…`），未变，不停。
- **S3**（装置与登记对不上）：第三节 3.2 四处按 R1–R4 改过，用 B10（门槛 205→181）、B15（2.0/1.971429）、B16（1.78012）、B17（7.952435）四条单测钉住，四条都绿。

## 变异
命令：
```
cd /home/fy5090/code/singlefs/research && bash scripts/mutate.sh e155-second-run-fsync-write-volume e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs mutations/e155_second_run_fsync_write_volume.tsv
```
三个数：基线全绿；27 条变异里 **26 条命中（红）**、**0 条无效**、**1 条判「等价」**（`R2_M19_甲按WAL臂算中间版结构恒零`：一个测试都没红，登记原文预期这条——甲每批就是一次发布，套用 WAL 臂的中间版公式在结构上恒等于 0，`0.0_f64 * 1.0_f64` 与 `0.0` 对模型输出没有任何差异，不是漏测）。收尾原样输出：`已还原，基线仍全绿`。

分类：
- 第一次的变异表（15 条，`research/mutations/e155_fsync_write_volume.tsv` 原文照抄）：15 条全部再抓到。
- 第二次新加（12 条，`R2_M1`–`R2_M19`，对应登记第九节 M1–M8、M14、M15、M18、M19）：11 条抓到，1 条（M19）等价。

## 产物
路径：`research/results/e155-second-run-fsync-write-volume-2026-09-19-stage1.out`（372 行）
完成标记：`E7RESULT name=done emitted=372`
分类计数（一条命令数出来）：
```
$ grep -oE 'name=[a-z0-9_]+' research/results/e155-second-run-fsync-write-volume-2026-09-19-stage1.out | sort | uniq -c
      1 name=b1_capacities
      1 name=config
      1 name=discriminability_self_proof
      1 name=done
      6 name=geometry_sensitivity_sample_row6
      1 name=positive_control
     72 name=q6_11_legacy_vs_fixed
    288 name=row6_grid
      1 name=scope
```
`row6_grid` = 6P×2族×2落点×3策略×4N = 288，`q6_11_legacy_vs_fixed` = 6P×2族×2落点×3策略 = 72，与登记网格逐项吻合，无缺行。

## replay.sh
`research/scripts/replay.sh` 新增一行：
```
E155R2|e155-second-run-fsync-write-volume||e155-second-run-fsync-write-volume-2026-09-19-stage1.out|exact
```
命令与结果：
```
$ cd research && bash scripts/replay.sh E155R2
E155R2 e155-second-run-fsync-write-volume 字节一致 e155-second-run-fsync-write-volume-2026-09-19-stage1.out
字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0
```
第一次的 `E155` 行未动，`bash scripts/replay.sh E155` 复核仍「字节一致」。

## 实验页
`.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`：
- 标题状态改为「部分已跑（第二次跑第一段；……）」。
- 加「第二次跑（2026-09-19）第一段做了什么」「产物读出来的几个数」「它答不了的」三段。
- kb-scribe 规格三件事原样照做：第 35 行整行改对；「## 历史版本」加 `### 2026-09-19` 一节（含 kb-scribe 的行与我这段的行）；加「### 影响的决策」表——6 行既有分项重新回看（日期 2026-09-19）+ 6 行新补（D16、D19 已定项 12、D16 已定项 1、D16 已定项 2、D25、以及把登记 5.6 写的「D16 已定项 4/7；D23 已定项 14/15 合一行」拆成 4 行——原文括注「每分项一行」，合并成一行是我的误读，已改回四行）。
- 索引行 `.claude/kb/experiments.md`、历史条目 `.claude/kb/experiments-history.md`（新增 `### 2026-09-19（其四）`）都已更新。
- `.claude/decision-links-pending` 里 `E155` 那一行按分工由主 agent 删，我没有碰它。

## 门禁（`stage-owners.tsv` 登记给 experiment-runner 的阶段）
| 阶段 | 末行 | 退出码 |
|---|---|---|
| 27-format-constants.sh | ✓ 格式常量同步（19 个已登记，19 个在源码里被钉住） | 0 |
| 33-mutation-tables.sh | ✓ 142 个实验二进制都有成形的变异表，1502 条变异的原文各命中源码一次 | 0 |
| 34-experiment-index-sync.sh | ✓ 实验索引行与正文标题一致（索引 154 行、正文 154 份） | 0 |
| 40-results-cited.sh | ✓ research/results 下的实验产物全部被 experiments.md 点名（311 个文件） | 0 |
| 52-segment-registry.sh | ✓ 比对了 5 处登记……（与本轮无关的既有登记，逐字一致） | 0 |
| 65-relay-timing.sh | ✓ 没有读子进程输出的循环一边给行打时间戳一边转打 | 0 |
| 69-evidence-in-repo.sh | ✗ 三处引了 `/tmp` 依据（`research/prompts/m2-closeout-recheck-{c,d,e}.md`）——**不是这一轮改的文件**，未修，照写 | 1 |
| 75-decision-experiment-links.sh | ✗ 4 处「指不到」（见下） | 0（脚本本身退出码 0，但报了红） |
| 80-absolute-assertions.sh | ✓ 142 个实验各自至少有一条绝对值断言 | 0 |
| 85-repro-command.sh | ✓ 点了产物的实验都写了复跑命令（146 个） | 0 |
| 86-experiment-orphans.sh | ✓ research 里的实验号在 kb 里都有正文（144 个） | 0 |
| 88-quoted-result-lines.sh | ✓ kb 正文里整行抄的产物行都在产物里逐字找得到（594 行，对照 311 份产物） | 0 |

**75 号红的根因不在我的改动里**：`D16（发布语义） 已定项 9`、`D16（发布语义） 已定项 2`、`D19（块指针的结构与宽度预算） 已定项 12`、`D23（journal 的角色与格式） 已定项 15` 这四个分项在各自的决策文件里**没有独立的 `### 已定项 N` 小节**（内容就是已定项索引表那一行的正文，D19 已定项 12 与 D23 已定项 15 的条款原文自己写着「正文里没有已定项 N 的单独小节，条款全文就是已定项索引表第 N 行」），而 75 号脚本的解析器目前只认标题锚点，认不到只活在索引表里的分项。这四行是**登记第五节 5.6 表格原文点名要加的行**（q3 已经抄过 D19 已定项 12、D23 已定项 15 这两句原文），不是我编出来的引用；修法要么给这几个分项补标题小节（决策文件，不归我）要么给 75 号解析器加索引表兜底（`.claude/gate.d/`，不归我），两处都不在我的写范围，交主 agent 定。

**69 号红的三行不是本轮改的文件**（`m2-closeout-recheck-c/d/e.md`），照 `agent-common.md`「红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写」处理，未动。

15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归我，交 `gate-triage`。

## 登记修订
`research/prompts/e155-r2-prereg.md` 第十二节「修订」这一段本轮**没有写**——第一段做的每一处都在登记原有的 R1–R7 定义与判据范围内，没有需要收严或补臂的发现；4 处停机条款全部现跑通过，未触发任何需要收窄结论的失败条款（跑到的几条：A3 已知会触发 F1，按登记原样记「条款字面漏列」；F5（K9′ 释放代读法）、F10（间隔按批 / 按 fsync 计）这一段因为没建 K9′-b、N 按批计在 checkpoint 侧的可计算开关，答不了；F9（不共享取随机）只在第 7 行判定格上适用，第一段不涉及；这些都已在实验页「它答不了的」与下面岔路表里列明，不构成登记第十二节意义上的「修订」）。

## 岔路表（登记里岔路单第 6、7 行；`research/prompts/m2-s1-forks.md:7-15`）

| 行 | 已够判 / 还差什么 | 这个实验剩下的量能不能让它翻面 | 指到产物里算它的命令 |
|---|---|---|---|
| 第 6 行 | **够判**（登记原文够判条件「第 1 行同一批取样点上三臂的 fsync 行与摊销行齐；树高 1、一个文件、覆盖写那一格甲仍与 crates 实测逐字节相同」两条都满足：`row6_grid` 288 格齐、B2/S1 逐字节对上）。还差：第 6 行判定格上的 8.2 全量反向几何扫描（这次只跑了 1 个代表格 P=10⁵、F1、seq、Lbalanced、N=16）；Q6.4 的规则 R 标签矩阵（产物有原始比值、没有逐格套规则 R 打标签）；K9′-b 与「N 按批计」两个反向取样点没有可计算的开关（F5、F10 答不了）。 | 已够判部分不会翻面（`s` 在主几何上跨过 0% 与 10% 两道门槛，判别力自证过；三臂包含关系 V2 全网格核过无违反）；还差的三项都是「补测」性质，不改已有格上的判定方向，但 K9′-b / N 按批计两个方向具体会不会把某些格的 `s` 从「省一成以上」压到「省不到一成」没有数，交主 agent 判要不要续。 | `grep -c "name=row6_grid" .../stage1.out` → 288；`grep "name=discriminability_self_proof" .../stage1.out`；`grep "name=geometry_sensitivity_sample_row6" .../stage1.out` |
| 第 7 行 | **开着，一步没跑**。第二段（R8 组提交、k 轴、共享/不共享）完全未做：B12–B14、Q7.1–Q7.6、共享/不共享与甲组提交两条阳性对照、变异 M9–M13/M16/M17 一个字节都没算。 | 这个实验这一段答不了——第 7 行要不要续跑、什么时候跑，交主 agent 按岔路单排期。 | （未跑，无命令） |

## 没做什么
- 没跑第二段（R8 组提交、第 7 行）：B12–B14、Q7.1–Q7.6、共享/不共享阳性对照、变异 M9–M13/M16/M17。
- 第 6 行判定格上的 8.2 几何敏感性只跑了 1 个代表格，不是登记要求的「每道判定在它自己的判定格上」全量扫描。
- Q6.4（规则 R 标签矩阵）、Q6.6（K9′ 比 K9 多写多少，逐格增量）、Q6.7（刷写/FUA 逐格）没有摘抄进实验页正文，产物里有原始数但没有二次加工成标签/增量表。
- Q6.11 只报了「哪些格 differs」的粗读（P=1/100 不变、P=10⁴ 起变），没有给出改前改后字节差的完整分布；这是主 agent 点名要跑的附带量，够判后按定义不必深挖，但如实报「只做了粗读」。
- K9′-b（中间版不进积压当主）与「N 按批计」两个反向取样点这一版没有实现成可计算的开关，F5、F10 答不了。
- 没判这个实验的结论能不能推翻或确立里程碑「第二个事务」增补 1 或三方 `m2-s1-r1` 的判决（那是推论，要走三方）；没跑门禁 15 号、87 号（不归我）；没提交。
- 没有修改 `.claude/gate.d/75-decision-experiment-links.sh` 或任何决策文件（不在写范围），75 号剩下的 4 处「指不到」交主 agent。
- `.claude/decision-links-pending` 里 E155 那一行没有碰（按分工归主 agent）。

## 关键文件路径
- 装置：`/home/fy5090/code/singlefs/research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs`
- Cargo 登记：`/home/fy5090/code/singlefs/research/e7-index-bench/Cargo.toml`（`e155-second-run-fsync-write-volume`）
- 变异表：`/home/fy5090/code/singlefs/research/mutations/e155_second_run_fsync_write_volume.tsv`
- 产物：`/home/fy5090/code/singlefs/research/results/e155-second-run-fsync-write-volume-2026-09-19-stage1.out`
- replay 登记：`/home/fy5090/code/singlefs/research/scripts/replay.sh`（新增 `E155R2` 行）
- 跑前登记（未改，只确认过）：`/home/fy5090/code/singlefs/research/prompts/e155-r2-prereg.md`
- 实验页：`/home/fy5090/code/singlefs/.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`
- 索引：`/home/fy5090/code/singlefs/.claude/kb/experiments.md`
- 历史：`/home/fy5090/code/singlefs/.claude/kb/experiments-history.md`（`### 2026-09-19（其四）`）

---

# 追加：F2（第十节）判定补做

主 agent 中途追加要求：登记第 1097–1099 行 F2 失败条款这一段没判，要求装置在 `row6_grid` 与 `q6_11_legacy_vs_fixed` 每一行报出 F2 两半（第一半：撞 1000 次迭代未收敛；第二半：某棵树第 0 层节点数 ≥ 64 且脏节点占比 ≥ 50%）各自触没触发。以下是这一段的补充交付。

## 装置改动
- 新增 `layer_zero_is_mostly_dirty`（F2 第二半的判据）、`convergence_and_saturation_status`（F2 两半合判，输出 `(bool, Vec<触发的树名>)`）、`jia_convergence_and_saturation_status`、`checkpoint_convergence_and_saturation_status` 四个函数。
- `WriteAheadLogCheckpointOutcome` 加 `extent_layers`/`extent_dirty`/`inode_layers`/`inode_dirty`/`allocation_dirty`/`mapping_layers`/`mapping_dirty`/`fixed_point_iterations` 八个字段，供 F2 判定读取（`solve_write_ahead_log_checkpoint` 用最终那一遍不动点核心的值填）。
- `row6_grid` 每行新增 10 个字段：甲与四条 WAL checkpoint 核心（wal_full 的 K9/K9′、乙-M 的 K9/K9′）各自的 `*_unconverged`（bool）与 `*_saturated_trees`（`Vec<&str>`）。
- `q6_11_legacy_vs_fixed` 每行新增 4 个字段：`fixed_unconverged`/`fixed_saturated_trees`/`legacy_unconverged`/`legacy_saturated_trees`（fixed 与 legacy 分开判）。
- 命名纪律：最初用 `f2_half1`/`f2_half2`/`f2_status` 这类名字，`naming-lint.sh` 判「f2」是单字母加数字，逐一改成上面这几个拼全的名字（`F2` 这个标签本身只留在注释与断言消息里，不进标识符）。

## 单测
新增 5 条：`f2_status_tests` 模块 3 条（`layer_zero_is_mostly_dirty` 的边界值：63 节点不触发、64 节点占比恰 50% 触发、`convergence_and_saturation_status` 两半独立判定）；`jia_anchor_tests` 里 2 条钉死主 agent 指出的两个真实锚点：
- `fixed_point_iteration_ceiling_triggers_at_file_count_1e8_one_data_unit_per_file_random_balanced`：P=10⁸、F1、rand、Lbalanced 撞满 1000 次迭代（F2 第一半），第二半不触发。
- `layer_zero_saturation_triggers_on_allocation_tree_at_file_count_1e6_one_data_unit_per_file_random_far`：P=10⁶、F1、rand、Lfar 分配记录树第 0 层饱和（F2 第二半，触发树含 `allocation`）；同一个 P 换 Lbalanced 不触发，两者对照证明判据不是恒真恒假。
命令：`cargo test --release -p e7-index-bench --bin e155-second-run-fsync-write-volume` → `test result: ok. 47 passed; 0 failed`。

## 变异
重新跑了一遍完整变异表（27 条，命令与文件都不变），结果不变：26 抓、1 等价（`R2_M19`）、0 无效——新加的 F2 代码没有影响任何一条既有变异的命中。

## 产物与 replay
重新生成 `research/results/e155-second-run-fsync-write-volume-2026-09-19-stage1.out`（同名覆盖，372 行不变，字段变多）；`bash scripts/replay.sh E155R2` 与 `bash scripts/replay.sh E155` 均报「字节一致」。

## F2 判定的读数（这就是「回对第一次实验页第 5 行」要用的数）
- **第一半（未收敛）**：`row6_grid` 里只有 `p=100000000 family=F1 placement=rand policy=Lbalanced` 这一格（全部 4 个 N）触发，且甲与四条 WAL checkpoint 核心**一起**未收敛（不是只有甲）。
- **第二半（层 0 饱和）**：`jia_saturated_trees` 非空的有 24 行（`p∈{10⁴(仅 F8A/Lbalanced)、10⁶、10⁸} × placement=rand`，触发树都是 `allocation`）；WAL checkpoint 核心触发得更多——`wal_full_released_and_backlogged_saturated_trees` 非空的有 65 行，印证登记原文「K9′ 在大 N 上让分配记录树变大，这一条在这一次更可能触发」。
- ⚠️ **F2 第二半不只在 L远出现**：`p=10000 family=F8A placement=rand policy=Lbalanced` 也触发（`jia_saturated_trees=["allocation"]`）。登记第十节 F2 原文「只在 L远 出现时第 6、7 行那一格按区间交出」这半句在这批数据里不成立——已在实验页写明，交主 agent 决定要不要收窄这句登记原文的适用范围。
- **回对第一次实验页第 5 行的数**（F1、Lbalanced，改后的甲，`q6_11_legacy_vs_fixed` 的 `fixed_bytes` 列）：

  | P | `placement=seq` | `placement=rand` |
  |---|---|---|
  | 1 | 344 576 | 344 576 |
  | 100 | 344 576 | 344 576 |
  | 10⁴ | 740 874 | 1 190 698 |
  | 10⁶ | 1 270 070 | 35 332 318 |
  | 10⁸ | 1 831 514 | 3 330 926 706（**F2 第一半，未收敛，数值不可信，只标量级**） |
  | 145 | 377 344 | 377 344 |

  `seq` 落点 5.3 倍（P 到 10⁸）与第一次的 ρ_a 结论一致；`rand` 落点在 P=10⁴→10⁶ 涨 29.7 倍（100 倍 P），P=10⁸ 那格因未收敛只能定性说「这个位置策略下一次持久化要重写树的一大半」。**这不是我能替用户下的判断，只把改后的数摆出来**：第一次实验页「主几何 Lbalanced 下不会失控」这句话，对 `seq` 落点仍然成立，对 `rand` 落点在 R1 修好之后已经看不出是「不失控」了——要不要改写第一次实验页第 5 行的结论、要不要新开一条问题记录，交主 agent 定。

## 实验页改动（本次追加）
`.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`：标题单测数改 42→47；「做了什么」加 F2 建设说明；「产物读出来的几个数」新增两条（F2 两半判定摘要、回对第一次第 5 行的 P 轴对照与结论收窄提示）。`.claude/kb/experiments.md` 索引行的结果摘要改写，把「主几何下不失控」限定到 `seq` 落点，加 `rand` 落点的新发现。`.claude/kb/experiments-history.md` 的 `### 2026-09-19（其四）` 条目原地扩写（同一条历史记录，不是新开一条，因为这是同一次交付的追加修订）。

## 岔路表第 6 行——重写（覆盖此前那版）

| 行 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 | 指到产物里算它的命令 |
|---|---|---|---|
| 第 6 行 | **够判**，且这一次已经把 F2 两半在全部 288+72 行上判过（不是只判了主 agent 举的那几个例子）。还差：第 6 行判定格上的 8.2 全量反向几何扫描（只跑了 1 个代表格）；Q6.4 规则 R 标签矩阵（产物有原始比值、未逐格打标签，且给 P 轴打标签时 F2 未收敛的那 4 行需要先按「结论收窄」处理，不能直接套规则 R）；K9′-b、N 按批计两个反向点没有可计算开关（F5、F10 答不了）。 | **F2 的发现本身可能翻面**：第一次实验页第 5 行「主几何 Lbalanced 下写量不会失控」这句话，现在只对 `seq` 落点站得住，`rand` 落点在 R1 修好之后大 P 时接近线性甚至撞不收敛——这是不是要算「翻面」、要不要因此重开第 5 行的结论，是这一段答不了、要交主 agent 或走三方论证的问题，不是我能自己判的。除此之外，第 6 行原有的够判结论（`s` 跨过 0/10% 门槛、V2 包含关系）不受 F2 影响——F2 触发的都是 `rand` 大池格，`s` 的判别力自证用的是 P=1 沿 N 轴，不在 F2 触发范围内。 | `grep "jia_unconverged=true" .../stage1.out`；`grep -v 'jia_saturated_trees=\[\]' .../stage1.out \| grep row6_grid`；`grep "name=q6_11_legacy_vs_fixed" .../stage1.out \| grep "family=F1 placement=rand"` |
| 第 7 行 | 不变，仍是开着、一步没跑（这一段仍不跑第 7 行）。 | 不变。 | （未跑，无命令） |

## 没做什么（追加）
- F2 判定只做到「检测 + 报告 + 结论收窄」，没有去消除它（登记本身允许 F2 触发，不要求修好）——第一半（P=10⁸ 那 4 行）目前没有再加大迭代上限或换算法去强行收敛，因为这不是登记要求的方向。
- 没有把 F2 触发的具体行数摘成一张完整的「触发格清单」贴进实验页（只贴了汇总计数与两个锚点作单测），完整清单在产物里，交主 agent 要的话可以再摘。
- 没有判断「rand 落点这个新发现」是否推翻或需要修订第一次实验页第 5 行的结论、或牵动决策——这是推论，要走三方或交主 agent，不是我能自己下的判断。
