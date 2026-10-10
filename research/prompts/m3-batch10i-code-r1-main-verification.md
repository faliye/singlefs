# 根环实现第 10i 批：代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 S1 R1 -->

正文 `research/prompts/_m3-batch10i-code-r1-body.md`；改动全文 `research/prompts/_m3-batch10i-code-r1-diff.md`（第 10i 批一份补丁）；冻结副本 `/tmp/claude-1000/m3-batch10i-code-r1/tree/`；两份腿报告：正推 `research/prompts/m3-batch10i-code-r1-sonnet-output.md`（184 行，sha256 2cb0de1a…a214，模型 `-sonnet-model/`），攻方 `research/prompts/m3-batch10i-code-r1-opus-output.md`（218 行，sha256 8b07d69a…5c5c，模型 `-opus-model/`）。

## 一、核查

| 核了什么 | 结果 |
|---|---|
| 开工快照 | 冻结副本对 `research/prompts/m3-batch10i-code-r1-snapshot/crates-sha256.txt` 313 份逐份相同。腿跑着时主工作区被删除与并发那条线第 26 批合入改了快照里 5 份：`crates/mutations.tsv`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`（与那条线发来的合后 sha256 `/tmp/claude-1000/m3-batch26/after.sha256` 逐份相同）；`kb-sha256.txt` 里 `.claude/kb/checks-owed.md` 一份对不上。两腿引的行全在冻结副本里（cite-check 以冻结副本为根），除 `crates/mutations.tsv` 外没有一处落在这 6 份；`crates/mutations.tsv` 引的是冻结副本的行号（1314、2646、2864–2927），冻结副本那一份与快照相同；`checks-owed.md` 两腿没按行号引 |
| 引文 | `cite-check.py`（`--root` 冻结副本、`--background research/prompts/_m3-batch10i-code-r1-background.md`）两份合跑：「✓ 核了 48 处引文，对上 48 处，没判 0 处（逐处列在上面）」 |
| 攻方模型复跑 | 主 agent 把它的模型目录拷进自己的草稿目录（拷完各文件 sha256 与报告开头那张表逐个相同），照 `rerun.sh` setup 之后跑 16 个阶段（`scan1-prod-u8-s6`、`-s9`、`-s12`、`-s15`，`scan1-refuse-u8-s6`、`scan1-ledger-u8-s6`、`scan1-exact-u8-s6`、`scan1-asread-u8-s6`，`live-prod-ab384`、`live-exact-ab384-A`、`live-asread-ab384`、`live-prod-mat-384-30-8`、`live-asread-mat-384-30-8`，`k2-prod-s6`、`k2-prod-3`、`k2-refuse-3`）；与腿 `out/` 里同名日志的 `OPUSI1-RUN` / `OPUSI1-SUM` 行排序后比：16 份全同。第四节交用户的数从主 agent 这 16 份日志数出（没复跑的格写明是攻方的数）。K1-1 那一行 checker 原文在主 agent 的 `scan1-prod-u8-s6` 日志上经 `agg.py --by=sp --show-checker=1` 原样打出：「OPUSI1-CHECKER ab-384-3-0 prod s1-A@0/3(d0)-trig0-c-Z step=9 F=17 txg=24 classes=["I-7.4"] broken>=F=[] broken_abandoned=[] v=["I-7.4: 被抛弃的根（实例 2、txg 9）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：实例表单元 在盘 0 槽 1536 的那一份与位置条目里的校验和对不上；实例表单元两份都读不到对得上的"]」；同一份日志 `--by=arm,pattern,red,classes` 打出 `arm=prod pattern=c red=true classes=I-7.4, latent-abandoned-free n=8`（共 54 段） |
| 主 agent 读码 | 冻结副本 `crates/singlefs-core/src/mount.rs` 第 3336–3345 行：抬 F 那一遍「读坏重读仍坏就拒」的那一组把两遍里只有一遍自证过的槽拿掉；第 3561–3563 行交回的 `roots` 只有这一遍读出的根；第 3577–3581 行影子账重算只从 `roots` 取被抛弃根；`crates/singlefs-core/src/allocator.rs` 第 1038–1040 行根环表对被抛弃根记着 `referenced_placements`。两腿说的机理与代码一致 |
| 正推腿的变异复跑 | 没重做：K3-d 那 6 条（P1–P4、A2、A3）是「没红」，判决只拿它当「缺口在」的线索、交下一批补用例并证红，不引它的数进 kb |

腿的数都在冻结副本的拷贝或攻方原型上，是副本上的数，不进 kb；判决只拿来分辨候选。

## 二、逐格判决

判据照正文第五节。正推腿与攻方腿各一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 不记成站住。

### K1　新类撑不撑得住

| 格 | 两腿 | 判决 |
|---|---|---|
| 被抛弃的根掉出影子账 | 正推 K1-b 推出这一形（冲突，没造）；攻方 K1-1 造出，I-7.4 红 | **打中，安全性**。坐实：主 agent 复跑与读码（第一节）。违反 `.claude/kb/decisions/23-journal的角色与格式.md:390`「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。冻结副本 `mount.rs:3556` 的注释自己写着改前那一拒挡的就是这一形（C554 乙 Q6）。S1 定案的字面（照「挂载那一刻读坏」钉住）推不出这一形可以放过：「挂载那一刻读坏」那一类内容不知道，新类被抛弃那一半内容知道、落点记在根环表里，没用上。改法交用户（第四节 T1），必须改 |
| 被抛弃根读坏时卡住准入上限 | 攻方 K1-2 | **打中，代价比必要的大**。`.claude/kb/decisions/16-发布语义.md:36`「「有效根」「非空」在环里全部的根上取」，进程已按最新根的实例表判出那条根被抛弃；今天照「能住的最大 txg」并进上限，F 卡在被抛弃根自己的 txg，副本上 54 段里 42 段 ENOSPC。与 T1 同一题 |
| 有效根读坏时卡住准入上限 | 攻方 K1-3 量出代价 | **代价量出来，不是打中**：照字面不越权；副本上 384 槽盘约 35%–56% 的段仍停摆，照「挂载时读出的那条根」算（asread）0 停摆 0 红。与 T1 同一题 |
| 回收门槛越过盘上有效根 | 正推 K1-a 一致；攻方没打中（五个起点约 1176 段） | 一次抽样没打中，不记站住 |
| 「能住的 txg」集合 | 正推 K1-c 一致（没有测试钉它，见 K3） | 一致 |
| 并进读出那条根的 txg 之后 F 越过它 | 正推 K1-d 推出一形：R 是洞里的旧根、读得出的非空根不足 4 条；攻方没造带洞的环与第 0 代根副本落在 S1 槽的年轻环 | **推的，没量**。若 T1 定成照记着的那条根算，这一形随之换成「照 R 本身算上限」，不再按窗口取最大；交下一轮攻方造带洞的环核 |
| 什么时候解开 | 正推 K1-f：写掉那一槽摆得出，重读读出根没有占位转换，钉只活在一次抬 F 里；攻方读码同 | 两腿一致：新类不留跨抬 F 的钉，I-3.1 那条用户认下的代价对新类不延长；D16 第 37 行写回时把新类的解开写明（写掉那一槽换类；钉只活在一次抬 F 里） |

### K2　回退与 F_生效 那两处的读法

| 格 | 判决 |
|---|---|
| 回退与抬 F 对同一槽处置不同 | 攻方 K2-1 打中**前后不一致**：同一次挂载里抬 F 照钉住推过某一 S1 槽，随后管理员回退因同一槽拒（副本上 `ab-384-3-5` 覆盖写 6 次再回退 132 段、`mat-384-30-8` 12 次 79 段），原样「rollback=Err(txg51:CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{ring_slot:RootRingSlot{region:0,slot:2},first_reading:Unreadable,reread:Unreadable}))」。回退拒本身第 10i 批之前就有（`k2-refuse-3` 与 `k2-prod-3` 逐类同数）；「本可以安全回退」是推的，没有一条「回退时不拒」的臂量过。拒的理由与 D16 第 37 行字面对得上（第二节读法：用户只定了抬 F 那一半）。交用户（第四节 T2） |
| F_生效 | 攻方 K2-2 读码：抬 F 那一串的 F_生效 读坏当没有根、不拒；拒的只在回退那一份。一致 |

### K3　补强的用例

| 格 | 判决 |
|---|---|
| rollback 三条每一步核方向 | 正推 K3-a 一致（txg 32、40、51、52 四个点各套一遍，三条都在那一步红） |
| R1 16 条 | 正推 K3-b：每条红在自己要罩的那件事上，但第 10i 批那 14 行变异（`crates/mutations.tsv` 第 2913–2926 行）让整个二进制 31 条全红，分不出归属。**缺口**：下一批换成分得开的变异（正推腿用的 2864、2868 两种旧样子各罩一类） |
| 丢盘整读新起点两条 | 正推 K3-c 一致，罩的是整环重读发生 |
| S1 两条与新类里没人钉的几处 | 正推 K3-d **打中缺口**：S1 两条只造有效那一半、只验「不拒」；钉最旧 txg、并最大 txg、仍坏映射记钉、能住的 txg 集合、被抛弃那一半算进还被引用的落点、换下时清隔离，6 处改坏都 31 条全绿，变异表没有行。下一批随 T1 的改法一起补：被抛弃那一半至少一条用例造攻方 K1-1 那一形，红 I-7.4 改前、改后绿 |
| 读序号 16 | 正推 K3-e：数对，脆（15、17 下变异 2910 杀不掉瞬时那条）。下一批在用例里断言「那一次读确是抬 F 第一次算上限」，不只靠常量 |

### K4　数能不能核

正推腿逐个核了第 10i 批报告里的数（narrow-384-20-2 起点、`(5, 3)` 对 `(4, 2)`、R1 31 条 14 秒、txg 40 那一次证红）：都推得出、钉得住；16 那一个是现量的（K3-e）。一致。

## 三、改法与去向

| 改法 | 出处 | 被攻轮数 | 去向 |
|---|---|---|---|
| 新类抬 F 那一遍读坏重读仍坏时怎么算（三个候选） | 攻方 K1-1、K1-2、K1-3 的 ledger、exact、asread 三臂；refuse 臂即第 10i 批之前 | 零轮，只在攻方原型上量过 | 交用户（T1） |
| 回退判候选与算 F_生效 时新类读坏怎么办 | 攻方 K2-1 | 零轮，「不拒」那一臂没量 | 交用户（T2） |
| 新类 6 处判别补用例与变异；R1 14 行换成分得开的变异；读序号那条加断言 | 正推 K3-b、K3-d、K3-e | — | 下一批（随 T1 的改法） |
| 带洞的环、第 0 代根副本落在 S1 槽的年轻环、4 GiB 盘上造 K1-d 与 K1-1 | 正推 K1-d；攻方「这条腿自己的限度」 | — | 下一轮代码轮攻方的攻击面 |
| D16 已定项 1 第 37 行补新类（含解开、T1 与 T2 的定案） | 第 10 批第三轮判决第六节；K1 解开那一格 | — | 下一份 kb 写回 |

## 四、交用户的岔路

岔路单 `research/prompts/m3-batch10i-code-r1-forks.md`（T1、T2 两行）。

## 五、旁及与去向

| 线索 | 依据 | 去向 |
|---|---|---|
| 正常卸载那一串不判上限，S1-A a、b 两形卸载之后可发槽 4 个、checker 没红 | 攻方 K1-1 末段 | T1 改法一并罩（卸载那一串同一份影子账）；下一批用例覆盖 |
| `on_device_modes.rs` 的 `raise_the_rollback_floor_to_its_ceiling` 用空的新类表 | 正推 K1-g | 不打中（那个模式的分配器来自 mkfs 同一个进程，不会有新类）；不做 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-06 不受影响：第 37 行今天还没有新类，T1、T2 定了再写回；第 36 行「有效根」只在有效根上取，与攻方 K1-2 对得上 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-06 不受影响：第 390 行被第 10i 批的实现违反（K1-1），改实现、不改条文 |

## 被判的 `crates/` 文件

第 10i 批补丁改到的每一份：`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`、`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`。

## 六、用户定案

用户 2026-10-06 弹窗定（岔路单 `research/prompts/m3-batch10i-code-r1-forks.md`）：

| 岔路 | 定的 | 去向 |
|---|---|---|
| T1 重建分配器两遍里只有一遍自证过的槽，抬 F 那一遍读坏、重读仍坏时怎么算 | 甲：照挂载时读出、记在根环表里的那条根算（有效根照它本身进上限与候选集，被抛弃根照它记着的落点进影子账、不进上限），根环表多记整条根记录 | 下一批实现（攻方原型 asread 臂的做法，被攻过零轮），连同第三节新类 6 处判别的用例与变异；下一轮代码三方 |
| T2 挂着时管理员回退判候选、算 F_生效 时这一类槽读坏重读仍坏 | 照今天拒，下一轮先量 | 回退那一处不改；下一轮代码轮攻方在 T1 甲的实现上加一条「回退不拒」的臂，量回退结局与 checker 红，量完再交用户 |
