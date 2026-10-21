# m3-close-c409-r1 判决：崩溃点重放的常规层、大负载层怎么立（主 agent，2026-10-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 K1 K2 K3 K4 G1 G2 G3 G4 -->

正文 `research/prompts/_m3-close-c409-r1-body.md`；正推腿 `research/prompts/m3-close-c409-r1-sonnet-output.md`（sha256 58d20909…）；攻方腿 `research/prompts/m3-close-c409-r1-opus-output.md`（sha256 fb416ab1…），模型 `research/prompts/m3-close-c409-r1-opus-model/`；岔路单 `research/prompts/m3-close-c409-r1-forks.md`。这是第一轮，打中的先挂起，不写进决策正文。

## 一、核查

- `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-c409-r1-background.md`：「✓ 核了 92 处引文、93 处裸行号，对上 92 处，没判 93 处（其中裸行号没有引文可比 93 处）」。
- 开工快照 `research/prompts/m3-close-c409-r1-snapshot/kb-sha256.txt`：`sha256sum -c` 两份对不上（`.claude/kb/milestone/03-third-txn.md`、`.claude/kb/checks-owed.md`），腿跑着时被别的写回定点改过。腿引这两份的行（03 第 109、145、150、177、193 行，欠账表 C409 那一行）由 cite-check 对着今天的文件逐字对上；没倒推快照原样。
- 裸行号里判决要用的几处，主 agent 现读：
  - `crates/singlefs-core/src/make_filesystem.rs` 的 `make_filesystem`：`assert_eq!(devices.len(), 2, "第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）")`。正文第二节那句「设备数取自交进来的设备身份个数」是错的（正文是这一轮的冻结证据，不回改，记在这里）。
  - `.claude/gate.d/checker-tier-crash-point-replay.sh` 文件头：「快档逐条带 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1 起用例（只算块键、查库，一块都不判）」；退出码 0「绿；快档有用例的块没有判定、带特性那一格有本次未跑时也退 0」；「快档不写真的判定库、不按全量的规模跑」。
  - `crates/singlefs-checker-tier/src/crash_identity.rs` 文件头：「块键 = 这一步带屏障、FUA、盘、偏移、段号的写表与它起步的镜像（扇区终值）的摘要，加段展开、判器另读的输入与段内序号」。
  - `.claude/rules/test-script-discipline.md`「入口脚本：不带参数就是剪枝跑全部项」：「只跑一部分项、一部分状态或一部分种子的（抽样）既不是剪枝跑，也不是全量。」
  - `.claude/kb/decisions/13-验证路线.md` 已定项 9 层表三行与已定项 11「⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**」逐字对上。
  - `.claude/kb/checks-owed.md` C409 行「常规层（上千写请求、按崩溃点所在子阶段分桶抽样、每次提交）」；`.claude/gate.d/stage-inputs.tsv` 第 69 行 `crash-case:crash-injection-fast-tier`。
  - `.claude/kb/milestone/03-third-txn.md`「待优化」第 13 行「要拦的输入」一格：「新立一个验证装置、门禁阶段或测试入口，而仓里已有判同一件事、同一批对象的装置」；`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」第 2 条：「有管同一件事的，把新判据追加进它」。
- 攻方模型复跑：主 agent 把模型目录拷到 `~/.local/share/singlefs/work/m3-close-c409-r1-main-checks/model`，经 16G 内存包装起 `rerun.sh`（结果见第六节）。正推腿的 `segment_sums.py`、`chain_states.py` 只读已入库常量算闭式，没复跑。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| K3-甲 写请求数当规模轴 | 采（模型量的，复跑见第六节）：态数由最长段定。正推腿 1.3 从另一条路（已入库的段形常量）推出同一结论：上千写的态数由每次发布改写的单元数定，两条路不共用代码 | 岔路 G1 |
| K3-乙 固定种子的常规层按字面就是冒烟层 | 采（读 kb，现查对上）：层表常规层一行没有一个与冒烟层不同的负载分句。打中不分辨臂，病根在层表本身 | 岔路 G1；层表缺口挂起，第二轮攻 |
| K3-丙 随机历史已作为 `crash-injection-fast-tier` 登在冒烟层 | 采（现查 `stage-inputs.tsv` 第 69 行） | G1 候选乙要写明它和这一条的差只在「整段全量展开」 |
| K3-丁 分层倒挂 | 采：负载更大的常规层被要求比冒烟层更勤地跑更全的域；正推腿 1.7 独立读到同一处（「每次提交」在实现里是查库） | 岔路 G2 |
| K3-戊 长原地段 | 部分采：固定脚本出长原地段时只能按段登记、不是精确全量（`crash-point-replay-segments.tsv` 第 2 行，正推腿 1.4 同读）；随机历史一侧没打中只抽了一次样 | G1 候选甲要报最长段 |
| K3-己 N≥5 轮在今天的键下是空的 | 采：同种子确定（模型量的）加块键只由内容定（现查）；正推腿第三节独立指出「轮」两读、D17 已定项 2 现值 1 | 岔路 G4 |
| K3-庚 多设备档位只有一档 | 采（现查断言） | 岔路 G4；正文那句错记在第一节 |
| K3-辛 按种类串去重 | 采：D13 已定项 10 射程里按种类串认节点本身挂着没定 | 不进候选 |
| K4-甲、乙、丙 三种并法 | 采：当 crash-case 行时提交那一趟一块不判、退 0；当快档用例时整条测试函数重判；当入口新项时提交不起 | 岔路 G3 候选甲写明「常规层登记行提交时照 G2 的档真判」 |
| K4-丁 块级复用罩不住动写路径的提交 | 推的，没量：按块键定义推得通，作废面多大要在副本上改一处写路径字节数出来 | G2 够判条件里的一项 |
| K4-戊 另立撞不撞「待优化」第 13 行 | 采攻方的字面读法：第 13 行拦的是「同一件事、同一批对象」，新负载不是同一批对象；撞的是 sop-first 第 2 条（只要求同一件事）。C409 补法、用户弹窗记录写「立阶段」，与第 13 行的改法没被调和 | 岔路 G3 |
| Q4 按 T1 截断 | 采：按挂钟截断判到哪一块随次序与机器变，是抽样；T1 写进 kb 又撞已定项 11 那处没判的「说反话」 | G2 候选丁只留「静态 ignore 登记、预算约束开几个节点」那一读，不按挂钟截断 |
| Q5 照 C409 补法写关账判据 | 采：C409「怎么拦」写分桶抽样与「抽样数设成 0」，与已定项 9 用户定案「我们不是抽样 我们是剪枝 加 初期ignore」相反（两条腿独立读到） | 第三节 |
| 正推 K2：四条候选三样代价都缺数 | 采：今天的冷吞吐没量（只有 2026-09-30 的旧数），库每态字节没读，键作废频度只有上界（碰 `crates` 的提交 35/196） | 第四节 |

## 三、C409 关账判据（零轮，第二轮攻）

一层算还清，三样同时满足（正推腿第六节的三合一，判别力那一条换成攻方改法 3）：

1. 负载登记：这一层至少一行 `crash-case:` 与合表一项（带层标记），登记行写这条负载的全量态数与最长段；大负载层的多设备档逐档一行，取不到的写「等实现」。
2. 门禁一格：判这一层，成功句报开着 / ignore / 等实现各几个节点、每个 ignore 节点的名字；这一层提交那一趟现判加复用为 0 块时退 77、不记通过；开着的节点有红退 1；ignore 数比上一次多退 1。
3. 判别力：门禁样本里造「这一层的节点全标 ignore」与「没有登记行」，那一格必须红或退 77；弄坏开关让引擎跳过这一层负载的一段，计数行的态数与登记的闭式对不上判红。

C409 行「怎么拦」里的分桶抽样与「抽样数设成 0」随第二轮判过之后一起改，这一轮不动欠账表。

## 四、下一步

- 代价问题单 `research/prompts/m3-close-c409-cost-questions.md`（主 agent 写，下一批交实验设计员）：G1 的计数探针、G2 的冷吞吐与键作废面、G3 的固定开销。冷吞吐那一项要起崩溃放量一个小项，是重型，起之前弹窗问用户。
- 第二轮（攻：第一轮的修补本身 / 产生结论的方法 / 没被看过的地方）在代价数出来之后开，攻 G1–G4 的候选与第三节判据。

## 五、交用户

这一轮不交用户定：四条岔路都还缺代价数（`.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 备料 | 2026-10-18 不受影响：层表缺口（K3-乙、丁、己、庚）挂起，第一轮不改正文 |
| D13（验证路线） 已定项 11 | 备料 | 2026-10-18 不受影响：T1 不写进 kb，G2 候选丁只用静态 ignore 登记 |
| D17（实现分层与第三方管道） 已定项 2 | 备料 | 2026-10-18 不受影响：G4 把「轮 = 等价类数」列成一读，不改现值 |
| D2（RAID 条带策略） 已定项 9 | 不影响 | 2026-10-18 不受影响：第一版两块盘，多设备档照实登「本次未跑」 |

## 六、攻方模型复跑

主 agent 在副本 `~/.local/share/singlefs/work/m3-close-c409-r1-main-checks/model` 上起 `rerun.sh`（16G 内存包装，`--frozen` 用腿留的冻结副本），末两行原样：

```
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-opus/frozen model=/home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-main-checks/model work=/home/fy5090/.local/share/singlefs/work/m3-close-c409-r1-main-checks/run
name=rerun_compare result=identical
```

两份产物（`load_profile.out`、`kind_strings.out`）与腿入库的 `out/` 逐字节相同；K3-甲、K3-己（确定性）、K3-辛 引的数照此采。模型只数闭式、没判态，这些数不进 kb。
