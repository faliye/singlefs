# m3-close-b19-code-r2 判决（代码轮第二轮，2026-10-29）

正文 `research/prompts/_m3-close-b19-code-r2-body.md`；本轮改动 `research/prompts/_m3-close-b19-code-r2-diff.md`；第一轮判决 `research/prompts/m3-close-b19-code-r1-main-verification.md`。腿：辩方 `research/prompts/m3-close-b19-code-r2-sonnet-output.md`（e519dd57…，模型 `research/prompts/m3-close-b19-code-r2-sonnet-model/`）、攻方 `research/prompts/m3-close-b19-code-r2-opus-output.md`（93532bad…，没建模型）。

被判的产品代码：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/instance_table_sweep.rs`（本轮修补），连同第一轮的 `crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/root_record.rs`。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-code-r2 --out ~/.local/share/singlefs/work/m3-close-b19-code-r2-judge/tree`：「清单点名 16 份，倒推出 16 份，sha256 对上 16 份」。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-code-r2-sonnet-output.md research/prompts/m3-close-b19-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-code-r2-judge/tree --background research/prompts/_m3-close-b19-code-r2-background.md`：「核了 77 处引文、39 处裸行号，对上 59 处，没判 57 处」，判了的 0 处对不上。
- 复跑：辩方模型 `tools/rerun.sh`，本机内存排不上，经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 16G` 拿主工作区（与快照 16/16 相同）的拷贝当冻结树跑（日志 `~/.local/share/singlefs/work/m3-close-b19-code-r2-judge/rerun.log`），退 0，13 步全 `SAME`（`new-u1`、`old-u1`、`new-u2`、`new-u4`、`new-u5`、`new-u6`、`new-pinned-and-new-tests`、`old-pinned-and-t4`、`new-root-record-target`、`new-core-lib-root-record`、`new-t1-mutation-6144`、`new-t2-mutation-6248`、`new-t2-mutation-6155`），变异还原后 `mount.rs` 前缀回到 `ba5453bdb9b1c31d`。副本上的数不进 kb；入库形态是下一批的用例。
- 核不动的：攻方 U2 没造盘面（推的）；辩方 U2 的「预演停在非读类错上又有补写盘」那一段没造（推的）；T6 里 `root_record.rs` 的新旧字节对比没做；攻方 U4 另记的「扣下的段不放回」没量。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| U1 T4 改法一 | **站得住，维持**。两腿一致：逐盘核走直读盘那一路、不碰预演缓存与两本账；辩方量了三形（盘 1 两槽写 0 加续做缺空间：旧红新做成；加 C841 镜像：旧次序把 C841 拒成「预演读坏」、新次序报 C841；盘 1 整块写 0 加 C841：新次序报逐盘核那一拒）。更正第一轮修补员的一句理由：钉读序数的用例不是「没跑到新次序」，是对这两段次序不敏感（辩方量的，四个目标新旧两棵树都过）。补一条用例钉「两种拒同时成立时报逐盘核那一拒」（辩方探针升格），这一次序写进条文（D22 那一行与 C874） | 辩方 U1 复跑 `new-u1`、`old-u1` 对上；攻方 U1 |
| U2 改法二 | **打中拒因被盖，采**：辩方量了盘 1 两槽写 0 加树表每次读坏，拒的结局不变但拒因报成盘 1 槽 0、不是出故障的那份树表（`new-u2` 对上）；攻方推的同一机理（补写盘两槽读坏进两本账，`(None, None)` 那一支走不到）。拒因报错违背 C874「运行时看得出」。改：判两本账前剔掉补写盘那几条（攻方乙）；用例：辩方那一形断言拒因点名树表，不点名补写盘的系统配置槽；变异「不剔」必红。零轮，进第三轮 | 辩方 U2、攻方 U2；复跑 |
| U3 T2 断言放处 | **站得住，维持**（两腿一致；`raise_rollback_floor` 一并改是共用函数的必然后果，不越位）。`crates/mutations.tsv` 第 6155 行名字写的还是挪走前的断言，下一批改名（行本身仍红，`new-t2-mutation-6155` 对上）；断言文档补列第三个调用处（取号之前只进内存那一趟） | 辩方 U3、攻方 U3 |
| U4 C928 | **维持相符**（两腿都没打中丢 discard；辩方补量「重发先失败再成功」没丢）。可选补一条用户改动点的用例，先造两段不被那次改动占去的候选段，不挡这一批。攻方另记的「判孤儿头与读不出时扣下的段从待 discard 表取走不放回」与冻结无关，登一笔欠账 | 辩方 U4、攻方 U4 |
| U5 C929 | **采候选一（照今天不判）**：两腿一致；辩方量了候选二会误拒「取号成功后崩溃」的正常盘面（同一份单元区，盘 1 的槽在则收、槽写 0 则被补写闸四拒），候选一会错的那一形对所有有自证槽的盘同形、不是回卷独有；攻方：回卷后两槽与普通成员盘分不开，判别子观测不到。条款补一句（C929 写回），回卷用例补断言「下一次挂载补写盘表为空」 | 辩方 U5（`new-u5` 对上）、攻方 U5 |
| U6 | **维持**：辩方 T5 量 420 次挂载清扫第一次在第 366 行触发、之后行数 2、全程不超过一片（仅 12 槽环一个池，`new-u6` 对上）；T1 平局倒着破钉平局的用例红（`new-t1-mutation-6144` 对上）；攻方 T3 grep 核 `discard_at` 产品代码只一处。`root_record.rs` 那一块列核不动，交验证段的形式化轻型 | 辩方 U6、攻方 U6 |
| P1–P3 | **没打中，两次抽样都没打中，记「没打中」** | 两轮攻方 P1–P3 |
| P4 | **没打中，两次抽样**；攻方读完 I-9.16 ④、I-3.20：I-3.1 与 I-3.18 读分配记录差在根记录直接持有的那一棵，差出来那一形要一份坏根记录，那张镜像因读不出已红（整张盘不会判成好的）。主 agent 判：不算 Q9 的漏（Q9 的条件管「这一块没有别的不变量红时判红」，这一形整张已红），写进 C937 射程 | 攻方 P4 |

## 三、交用户的

无。

## 四、被攻过零轮的

U2 的改法（剔掉补写盘那几条）、U1 的「两种拒同时成立报逐盘核那一拒」用例与条文句——实现交回后进第三轮（最终轮），第三轮只攻前两轮站住的形态与这两处。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） | 备料 | 2026-10-29 不受影响：U1 只补逐盘核与续做预演的先后一句，随 C874 写回 |
