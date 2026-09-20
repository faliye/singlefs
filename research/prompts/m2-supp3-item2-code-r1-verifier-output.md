# m2-supp3-item2-code-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

挑 Opus 报告里的引用「`crates/singlefs-harness/src/model.rs:1240` 整行 `                bounds.allocation_records > Self::allocation_record_node_capacity()`」，
在草稿目录（`/tmp/claude-1000/m2-supp3-item2-code-r1-verifier/selftest/`）里把行号从 1240 改成 1241 再核：

```
$ awk 'NR==1241' crates/singlefs-harness/src/model.rs
            }
```

第 1241 行实际是 `            }`，与引用原文不符 ⇒ **判定：✗**。方法能分辨，往下按同一方法核。

## 输入与快照核对

- 云端攻方报告 `m2-supp3-item2-code-r1-opus-output.md` sha256：现算 `d6292f4978503994ec0c205187337dda9e8e48f1b2876d30583598016b8ed158`，与主 agent 给的一致。
- 云端正推报告 `m2-supp3-item2-code-r1-sonnet-output.md` sha256：现算 `1534127532144bb905ec3060dd26535c81c3e0245556a8fb5a94823b2537e876`，与主 agent 给的一致。
- 开工快照 `sha256sum -c m2-supp3-item2-code-r1-start-snapshot.sha256`（对主树核，现在这一刻）：**19 行里 2 行 FAILED**——`.claude/kb/decisions/28-挂载期承诺量.md`（主 agent 已知，给了快照副本）与 **`.claude/kb/decisions/04-校验和位置.md`（新发现，主 agent 未提及）**。
  - D28：主 agent 给的快照副本 `m2-supp3-item2-code-r1-snapshot-copies/.claude/kb/decisions/28-挂载期承诺量.md` sha256 `14b347b8…6c44` 与快照记录一致；`git show HEAD:.claude/kb/decisions/28-挂载期承诺量.md | sha256sum` 同样得 `14b347b8…6c44`——HEAD 提交本身就是快照那一刻的原样，工作区的改动是未提交的。
  - D04：现查 `git show HEAD:.claude/kb/decisions/04-校验和位置.md | sha256sum` = `c7e754ab45…d696`，与快照记录的 `c7e754ab45…d696` 一致；工作区的改动（体例瘦身，`git status --short` 显示 `M`）同样是未提交的。**两个腿报告（Opus 于 10:11 UTC、Sonnet 两次核对）当时看到的 D04 都是 `OK`**，说明这处漂移发生在两条云端腿都交付之后、在我这次核查开工之后（应是另一个并发会话仍在做的 kb 腐化回扫）。这不是腿报告的错，但意味着**下面涉及 D04 的引用不能对现在的工作区核，改用 `git show HEAD:` 取快照原样核**（HEAD 与快照哈希一致，可以当快照副本用）。
- Opus 攻方模型目录 `SHA256SUMS`（18 个文件）：`sha256sum -c` 全部 `OK`。

## 一、云端攻方（Opus）报告核对

引用/产物/复跑 → 结果 → 命令依据。逐条列在下表，命中的原文只贴关键片段（完整比对命令见下）。

| # | 引用 | 核的结果 |
|---|---|---|
| 1 | `model.rs:1240` 整行（分配记录墙判定） | ✓ `awk 'NR==1240'` 逐字相同 |
| 2 | `model.rs:763` 整行（上界只加不减） | ✓ 逐字相同 |
| 3 | `model.rs:1260` 整行（任一次超了放行） | ✓ 逐字相同 |
| 4 | `model.rs:1228` 整行（「只会更宽」注释） | ✓ 逐字相同 |
| 5 | `transaction.rs:1326` 整行（`records_after_this_publish >`） | ✓ 逐字相同 |
| 6 | `model.rs:1155` 整行（mkfs 那一版判据） | ✓ 逐字相同 |
| 7 | `model_comparison.rs:184` 整行 | ✓ 逐字相同 |
| 8 | `model_comparison.rs:183` 整行（注释） | ✓ 逐字相同 |
| 9 | `mount.rs:1206` 整行 `reason: "txg 低于生效的回退下界 F"` | ✓ 逐字相同 |
| 10 | `mount.rs:1216` 整行 `reason: "实例表里那个实例的行 T 更小……"` | ✓ 逐字相同 |
| 11 | `transaction.rs:921` 整行（文档注释） | ✓ 逐字相同 |
| 12 | `transaction.rs:1595` 整行（`NoSpaceFor` 发出处） | ✓ 逐字相同 |
| 13 | `allocator.rs:716` 整行（用户数据拒绝原因丢弃） | ✓ 逐字相同 |
| 14 | `allocator.rs:763`（「提交内生块同一句」） | ✓ 紧邻 716 下一行，句式对应，逐字相同 |
| 15 | `model_comparison.rs:75` 整行（按落点找记录） | ✓ 逐字相同 |
| 16 | `model.rs:1565` 整行（逐角色比对） | ✓ 逐字相同 |
| 17 | `transaction.rs` 928 行附近（`AllocationRecordsExceedOneNode { records, capacity }` 带 `records` 字段） | ✓ 现查确有 `records: usize` 字段 |
| 18 | `.claude/kb/milestone/02-second-txn.md:356` 第 39 行数据 | ✓ 与引文起首逐字相同（引用用「…」标了截断，未隐藏截断） |
| 19 | `mount.rs:40-43`（`RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 文档注释「现行版本里没有重写过的实例表单元」） | ✓ 第 40 行含此子串，逐字相同 |
| 20 | `model.rs` 引 D28 处：`grep -n 'D28'` 命中第 227、1230 行 | ✓ 复算 grep 结果与报告一致 |
| 21 | `mount.rs:1203`（B2/B2s 变异靶行 `if target.checkpoint_txg < effective_floor {`） | ✓ 逐字相同，且与 `mutants.tsv` 里 B2/B2s 行的 `old` 字段一致 |

产物核对（逐字比对报告表格与模型目录里的文件）：

| 产物 | 报告里引的数 | 核的结果 |
|---|---|---|
| `wall-interval-reach.txt` | 一节「区间在三段里开没开」表格 7 行（模型上界最高/实现条数最高/上界>812 步数/那些步实现条数） | ✓ 7 行全部逐字段对上（`checker on/off × broad/reuse/rollback × 30/200`） |
| `wall-long-runs.txt` | W1 长历史表格「82/17/0」「228/27/0」「基线 80/216、0/0、0/0」 | ✓ 复跑得到同样的 82、17、0（见下「复跑」节），产物文件本身未见与表格数字不符处（人工比对） |
| `B2s-windows.txt` | 20 窗新发现序列「9、2、9、6、7、8、4、5、7、11、9、4、9、7、3、5、6、8、8、2，最少 2」；语义签名「1–8」 | ✓ 逐个数字相同；语义列 min=1 max=8 与「1–8」相符 |
| `geometry-table.txt` | 表 4（M5）B1/B2/B2s/B3/B5/B6/N2 共 7×9=63 格 | ✓ 63 格全部核对，逐字/逐数相同，包括「N2 在 G9 不红（0/96）」 |
| `reason-fix-runs.txt` | 「R1+改法 回退 12、快档 4 段判红；基线+改法 三段 0、[0,960)×2 组 0」 | ✓ 文件里 `R1fix-std` 三段新发现为 12（回退）/0（复用）/4（快档），`basefix2-std` 三段全 0，`basefix2 big rollback/broad` 均新发现 0；`R1fix big` 两组新发现 152/54，报告未提及但也未与之矛盾 |
| `matrix.log`/`matrix2.log` | R1/W1/N2/A9/A10 各自 9 行（std/noharness/modelonly × 三段） | ✓ 抽查全部命中，与复跑结果（下节）逐字一致 |
| `mutants.tsv`（模型目录自带的攻方变异表） | W1、R1、B2、B2s、A9、A10 等的 old/new 定义 | ✓ 与报告描述的改法逐条对应 |

**复跑**（草稿目录 `/tmp/claude-1000/m2-supp3-item2-code-r1-verifier/`，仓副本从主工作区 `rsync --exclude target --exclude .git` 拷出，`nice -n 19` 跑，未用主工作区）：

- `sha256sum -c SHA256SUMS`（18 个文件）：全 `OK`。
- `attack-patch.py` 补丁只改了 `history.rs`（加两处 `ATTACK_NO_CHECKER` 清空、一处 `ATTACK_NO_HARNESS` 短路、一段 `ATTACK_PROBE` 探针打印）与 `model.rs`（加一个 `attack_current_bound` 访问器）——`diff` 现查确认，与报告「只加三个环境变量开关与一行探针」相符。
- `run-mutant.sh base`：三组套件三段共 9 行输出，`新发现` 全为 0；`run-mutant.sh W1`、`R1`、`N2` 三次重跑，9 行输出逐字对应 `matrix.log` 里 W1/R1/N2 各自的 9 行（全部相同，含 `跑完`、`已知红收尾`、`根环转过一圈`、`最高 txg` 各字段）。
- W1 长历史（`ATTACK_NO_CHECKER=1 ATTACK_PROBE=1 SEEDS=96 OPERATIONS=200 WEIGHTS=broad SHRINK=none`，`wall-summary.py`）：`publish wall refusals 82; …count+16<=812…: 17; model disagreements {}`，与报告「82 / 17 / 0」逐字相同；示例首行 `seed 3 step 189 model_bound 1516 impl_records 796` 与报告举的例子（种子 3 第 189 步、上界 1516、实现 796）相同。
- base 同配置长历史：`publish wall refusals 80; …: 0; model disagreements {}`，与报告表格「基线 80」相同。

## 二、云端正推（Sonnet）报告核对

大多数引用逐字核对无误（下方给样例与计数），**发现两组共 5 处行号引用错误**，集中在两个位置：

| # | 引用 | 报告里说的 | 核的结果 |
|---|---|---|---|
| A1 | `.claude/kb/decisions/16-发布语义.md:375` | 「回退候选集 \| 按实例表判仍然有效 ∧ txg ≥ F_生效」 | **✗** 第 375 行实际是「生效 \| 每块幸存盘上都有带新 F 的持久根才生效……」；引文内容实际在**第 376 行** |
| A2 | `.claude/kb/decisions/16-发布语义.md:376` | 「抬 F 的上限 \| min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根」 | **✗** 第 376 行实际是「回退候选集 \| ……」（即 A1 的内容）；引文实际在**第 374 行**，且引用还漏了该行后半句「一次处置的目标 = min(……)」（未加省略号标注截断） |
| B1 | `B2.log` 第 217 行「种子 6……MountRollback 该拒：[\"回退到树表 0 条的根（第一版不支持）\"]；实现 拒了：MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）」 | 报告称该内容在第 217 行 | **✗** 第 217 行实际是「已知红第 1 条（增补 2 收口表第 43 行）：1 段；前几个种子 [(80, Operation(21))]」；引文内容实际横跨**第 218–219 行**（218 行「……第一个种子 6……在 Operation(10)（Some(CloseAndMountRollback)）之后」，219 行「模型对不上（拒绝的理由）：模型答 MountRollback 该拒……」） |
| B2 | `B2.log` 第 219 行「Operation(10)（Some(CloseAndMountRollback)）」 | 报告称该内容在第 219 行 | **✗** 该子串实际在**第 218 行**；第 219 行是 B1 里那句「模型对不上（拒绝的理由）……」 |
| B3 | `B2.log` 「第 8–9 行区块内」「第一个种子 7（同签名的种子 [7, 36]），在 Operation(16)」 | 报告称该内容在第 8–9 行区块 | **✗** 第 8–9 行实际是两条 `test … ok` 通过行，与「种子 7」无关；引文内容实际在**第 76–77 行**（该文件里第 295–296 行还有一份完全相同的重复） |

其余核对（样例，逐字/逐数相同）：

| # | 引用 | 结果 |
|---|---|---|
| C1 | `model.rs:42-45`（`ModelRootKey`） | ✓ |
| C2 | `.claude/kb/decisions/22-单元原子性怎么合成.md:488`（择新在…高者赢） | ✓ |
| C3 | `model.rs:651-657`（`is_abandoned_by`） | ✓ |
| C4 | `.claude/kb/decisions/23-journal的角色与格式.md:1209`（候选集=…） | ✓ |
| C5 | `.claude/kb/decisions/23-journal的角色与格式.md:695`（mkfs 写 0…） | ✓ |
| C6 | `model.rs:212`（`data_unit_payload_capacity_in_bytes`） | ✓（grep 命中同一行） |
| C7 | `model.rs:1001`（取号） | ✓ |
| C8 | `model.rs` 枚举成员注释行 217/227/235/237/239/241/243（7 行） | ✓ 全部逐字相同 |
| C9 | `model.rs` `answer_mount_rollback` 内 949/962/966/968/970/971/973/974/976（9 行边界） | ✓ |
| C10 | `model.rs` 1120/1126/1127/1136/1140/1143（6 行边界） | ✓ |
| C11 | `model.rs` 1002/1020（写行区间边界） | ✓ |
| C12 | `model.rs` `rollback_floor_ceiling` 665/701（边界） | ✓ |
| C13 | `history.rs:593`（`fn judge_by_model`）、`history.rs:1081`（`fn only_allocated_statistic_above_walked`） | ✓ 两处 `awk` 命中与报告一致 |
| C14 | `model_comparison.rs:263,285`（测试函数边界） | ✓ |
| C15 | `crates/mutations.tsv:154`（N2 行）、`:155`（import 行） | ✓ 两行内容逐字相同 |
| C16 | `.claude/gate.d/74-model-differential.sh` green/red 实测输出 | ✓ 与报告逐字相同（含 `脚本里 steps=…` 判断行） |
| C17 | `research/prompts/m2-supp3-item2-implementer-report.md:160,112` | ✓ 两行内容逐字相同 |
| C18 | `research/prompts/m2-supp3-item2-implementer/B1.log:216-217` | ✓ 内容与行号都对（与 B2.log 的错法不同） |
| C19 | `check.log:492,526`；`gate33-final.log`/`gate53-final.log`/`gate59-new-rows.log` 末行 | ✓ 全部逐字相同 |
| C20 | `.claude/kb/decisions/28-挂载期承诺量.md`（快照副本）第 16、21、82、84、88、102 行 | ✓ 全部逐字相同（对副本核，副本哈希与快照一致） |
| C21 | `.claude/kb/decisions/04-校验和位置.md:290`「附近」（已定项 5 全文） | 软性：sonnet 自己标了「附近」不是精确行号，且注明精确范围取自背景材料「285-306」出处标注——现查 `git show HEAD:` 该文件第 285 行确为「已定项 5」标题、287 行为「单元恒 32768 字节」正文，285-306 的范围本身准确；不计入 ✗（未被断言为精确单行） |

## 三、本地攻方核对

**核对表**（`m2-supp3-item2-code-r1-local-attack-translation-audit.md`）12 行「原文文件:行」逐条现查，**全部 ✓**：

| 项 | 引的文件:行 | 结果 |
|---|---|---|
| Q1 | `04-校验和位置.md:287`（对 `git show HEAD:` 核，因工作区已漂移） | ✓ |
| Q2 | `18-块里携带什么信息.md:944` | ✓ |
| Q3 | `18-块里携带什么信息.md:919` | ✓ |
| Q4 | `08-核心索引结构.md:512` | ✓ |
| Q5 | `03-空间分配.md:422` | ✓ |
| Q6 | `18-块里携带什么信息.md:711-713` | ✓ |
| Q7 | `16-发布语义.md:374`（抬 F 的上限） | ✓——**注意**：这正是 sonnet 报告 A2 条错误引用的那句话，本地攻方提示自己标对了行号（374），可反证 sonnet 的 376 确实是行号错误，不是我这边算错 |
| Q8 | `16-发布语义.md:372` | ✓ |
| Q9 | `16-发布语义.md:375`（生效） | ✓——sonnet 报告 A1 条把「回退候选集」（实际 376 行）错标成 375（这里的「生效」才是真正的 375 行） |
| Q10 | `16-发布语义.md:381` | ✓ |
| Q11 | `16-发布语义.md:383` | ✓ |
| Q12 | `m2-supp3-item2-implementer-report.md:76` | ✓ |

**运行记录**（`-local-attack-runlog.md`）核对：

```
$ wc -w s1 s2 → 1105 638（与运行记录逐字相同）
$ python3 research/scripts/oov-check.py s1 → 绿 … 生词=7 拼接=0（生词: overturned）exit=0
$ python3 research/scripts/oov-check.py s2 → 绿 … 生词=7 拼接=0（生词: overturned）exit=0
$ python3 research/scripts/corruption-check.py s1 / s2 → 均绿，exit=0
```

与运行记录表格（退出码 0、词数 1105/638、oov-check 判定「绿」「生词=7 拼接=0」、拼接 0、归类「干净」）逐项相同。两份干净样本达到停止条件，与规则「一条腿只抽一次样不算一次观测——两次都没打中才记『没打中』」的抽样纪律一致（这里是「构造攻击」类，两次判绿即停，规则本身没有要求更多次）。

## 计数

- Opus 报告：核了 21 处「文件:行 + 原文」引用 + 7 项产物文件核对（合计 63+20+7+9×5 ≈ 200+ 个数字/字段级核对点）+ 18 个模型目录文件哈希 + 5 次复跑（base、W1、R1、N2 的 3×3 矩阵，W1 与 base 的长历史墙测试）。**引用 21/21 ✓，产物 7/7 ✓，复跑 5/5 与产物数字逐字一致，SHA256SUMS 18/18 OK。核不动：0。✗：0。**
- Sonnet 报告：核了约 40 处「文件:行 + 原文」引用及产物/grep 复现。**35 ✓，5 ✗**（集中在 `decisions/16-发布语义.md` 两处行号互换/错位、`B2.log` 三处行号错位），另 1 处软性未计入（自称「附近」）。核不动：0。
- 本地攻方：核对表 12 行引用 **12/12 ✓**；运行记录里可机核的 4 项声明（词数×2、oov-check×2、corruption-check×2 实际共 6 条命令）**全部复现一致**。✗：0，核不动：0。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（M2–M5 的「四句」论证、M1 的一致性判断、本地攻方的算术是否真的按规则算对），只核引用、产物与复跑。
- Opus 攻方给出的「改法」（`wall-fix-patch.py`、`reason-fix-patch.py`、给环里有效根记表的第三条改法）**没有复跑**：报告自己标注「只在我的模型上量过、被攻过零轮」，按核查范围这类自称未验证的探索性结果不需要复核数字，只需确认它确实被标成了未验证——已确认（正文与末节两处都写了「被攻过零轮」）。
- Opus 报告里另外约 15 条变异（B1、B3、B5、B6、A1–A4、A8、N1、N3–N5、W2）没有逐条复跑，只核对了 `matrix.log`/`matrix2.log`/`geometry-table.txt` 里对应行与报告表格文字是否逐字一致（结果一致，见上表），没有重新生成这些行——完整复跑 20 条变异 × 3 组套件 × 3 段、九个几何、B2s 20 窗，预计数小时，本轮按「核了几处代表性的且核心『打中』结论」的取舍做了取舍：优先复跑了 M2（W1，含长历史）、M3（R1）、M4/M5（N2）三个「打中」判定各自的核心变异。
- 没有跑 `.claude/gate.d/89-stage-selftest.sh` 全量（sonnet 报告自己也未跑，说明已经写清）；没有编译入库装置本身、没有改动主工作区任何文件。
- 没有对 Opus 报告「四句」栏目里「系统当时看不看得到」这类语义判断做真假核实——那是判断，不是可核的引用或产物。
- 没有核实 `research/prompts/m2-supp3-item2-implementer/` 目录下除 B1.log、B2.log、check.log、gate33/53/59 log、mutants-summary.txt 之外的其余日志文件（`B3.log`、`B5.log`、`B6.log` 等）逐行内容，只核了报告实际引用到的那几处。
- D04 现在（此刻）与快照不一致这件事，已记入「快照核对」一节；由于两条云端腿完成时该文件仍是 `OK`，这不算它们的责任，但意味着**任何后续会话再引用 D04 的当前内容都要重新核**（与 D28 同类，均是并发会话未提交的编辑）。
