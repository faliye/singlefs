# m3-formal-r6 攻方腿（Opus）报告（第三轮、最终轮，2026-10-02）

<!-- doc-lint:not-numbers O1 O2 O3 Y2 Y7 T1 T5 T6 U1 U2 U3 U4 U5 U6 U7 F1 F2 F3 F4 F5 F6 G1 G2 P Q R0 R1 R1b R2 M2 M3 V1 V2 V3 W1 W2 X1 X2 X3 -->

攻击面：正文第四节分工表「云端攻方」一行，X1–X3，重点 X1。被判对象读冻结副本 `/tmp/claude-1000/m3-formal-r6/tree/`（清单 231 份 `sha256sum -c` 全 OK）；量都在它的拷贝 `/tmp/claude-1000/m3-formal-r6-attack/tree/` 上，下文「量过」都指这份拷贝上的原型 r6probe，是副本上的数，不是入库装置上的数。不重复前两轮攻方报告（`m3-formal-r4-opus-output.md`、`m3-formal-r5-opus-output.md`、`m3-c651-r1-opus-output.md`）各节的角度：前几轮没量过的是「回退做成算不算边界」「崩溃时刻归属」「回退目标可不可命名」「W1 在 mkfs 中途崩溃」这几格。

## 复跑

```
bash research/prompts/m3-formal-r6-opus-model/rerun.sh /tmp/claude-1000/m3-formal-r6/tree /tmp/claude-1000/<草稿目录> [件名…]
```

它把冻结副本 rsync 到草稿目录、拷 `.cargo`、打两份副本补丁（`c651_patch_copy.py` 与 m3-c651-r1 攻方的 `patch_copy.py` 逐字节相同：C651 候选开关 `C651_CANDIDATE`、W1 开关 `C651_MKFS_CLEARS`；`r6_patch_copy.py`：变异开关 `R6_MUTATION=rollback_not_durable`、把 r6probe 加进工作区），经内存包装（3G）编 release，再逐件跑（每件 5 线程，内存包装 2G，单次调用不超过 20 万态）。件名：`sizes lo-rbm lo-rbm-mut tgt-fft tgt-rbm w1-off-A w1-on-A`。`w1-on-A` 全量 82 970 态，我只跑完 22 070 态（见二、与「要主 agent 跑的命令」）。

模型目录 `research/prompts/m3-formal-r6-opus-model/` 每个文件的 sha256（`sha256sum` 原样）：

```
be4d9adc5c9499a649d91b100dcb0a10fa53389c78133d4f8ebd7047ef127f2e  c651_patch_copy.py
e01177088600060ef2f17390baf633daa620e5e5fc077ebc3b4c3ebac8b2e089  r6_patch_copy.py
53e30784906b84e1d53717d59de27036975e9a076f3d9ff8229198490f1d41b1  main.rs
a3c38046e6189508464ded3e823f2676d14b2de1e7fec066027c8f51be014087  Cargo.toml
04c0c4f280227c5da5aaa5b5a18d56bc10b58afad623795cb4897b1cae27b137  rerun.sh
69efbb85da7f6b3df7d074b3957dc034664eca204f2694862d67cf158360520b  runs/lo-rbm-clean-0.log
08a99c1f352f8836ed84357bec1e62c14c5a0d0e3fd21a202861fb728265b799  runs/lo-rbm-clean-1.log
e838330f9664dbb11f1fed0d8381872e935bf42218bba47f5fc7d45c04180eab  runs/lo-rbm-mut-0.log
70df28ba9fcf9f6f5376d19aed8b4f4a6d2e99e9af178ea1955a0bd8ffb25d4d  runs/lo-rbm-mut-1.log
0d4745c450b810bb61b5c3902002ce38a0a1d49f7f0f5d9ac18be167572a7f1a  runs/sizes-rbm.log
4d2f3ac96c647742927aa8aa09d33607d612de3dd8f20ff5ac9dc74b92de724b  runs/sizes-rbm-mut.log
d3e22b053a078a2b7b86763f4334ca4d9940b522e14f065413d2e82ee95be0f2  runs/tgt-fft.log
055ee0de07aed06bebf9bfb5756b7af39a52c81c710a5471f66b8ad3ac096e8d  runs/tgt-rbm-0.log
5d9c1fdd18c0b7ecc8ad4c1835c419e9417800ce8b6ad2624db6cb7e8d82c288  runs/tgt-rbm-1.log
9c961a7f34b726e18058b6eb408b03c4c48608c383b3e857b6a78a955a12d485  runs/tgt-rbm-2.log
346de352b8e18edd033341f7b4a8f77f53e7ce587705a066960d365dc8731438  runs/tgt-rbm-3.log
1888c10dd1b6f0e5654a6a723c2c535da7b0305ff1f38cd04128491226662a3b  runs/tgt-rbm-sum.txt
a8ce0ad2d7fc1c12e29b2f898829a917e5ecbac483b3b73d05946232d1ee2de0  runs/w1-off-A-0.log
7481030ad0d0644744594525ed2a3329f44b70969c1ca92c803791c3596cd601  runs/w1-off-A-1.log
4f27771300d053354532970af88efd41e0dd2094b84ca5696e280fb6bd3ddeaf  runs/w1-off-A-sum.txt
2f458fc3c9bbc789a377794d928cbbb5bf4ef8f59acb4d129b80e7ce0839c170  runs/w1-on-A-est.log
c2a2cb97a876f599bc45f823a33b7bced361a9afc9bf29bed7220fdd4206e937  runs/w1-on-A-1.log
ce941ab09fc4e28f01b50cd196aec104ef0a509dad7674c08db3e03c6ca6d0c1  runs/w1-on-A-2.log
bce9bdbcbaa18155482a65db4629a20c1f2a812a875669b36b2c25170c26c6e2  runs/w1-on-A-tail.log
9ea50488acccb7cd6b3005a0c4b718078103861187eba026a6927832c5e63bf6  runs/w1-on-A-partial-sum.txt
```

枚举：每个崩溃状态取自 `crates/singlefs-checker-tier/src/crash.rs` 的 `judge_layer0_state_range` 加 `full_expansion`（层 0 同一枚举域：每段都展开、原地覆写取第三态；每个状态都跑恢复与池级 checker），状态数用 `layer0_state_count_with_torn_in_place_overwrites` 先算。历史都是本原型手写的（`main.rs` 的 `spec` 与 `w1_histories`），不取自任何层 0 用例；测试目标与二进制名不带 layer0。盘宽：rbm / fft 取 `UnitAreaOf384Slots`（环 6 MiB）；W1 那一组取 m3-c651-r1 攻方同一档小盘（(1408 + 384) × 16384 字节，环 6 MiB）。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| G2 / U2 ①「回退之后又崩溃」 | 打中（量过，副本） | 条款没写回退做成算不算边界、抬不抬 lo。rbm（覆盖写 → 回退到次新 → 重挂）24 618 态：照「发布调用返回抬 lo、回退不是边界」读（R1），恢复读回回退内容的 20 522 态全假红；照「lo 只由 fsync 抬」读（R0）不假红，但下一格放过坏实现。O1 全四条 24 618 态全绿 | 一、1 |
| G1 + U1：回退返回早于持久的坏实现 | 打中（量过，副本） | 回退在单元屏障之后就交回 Ok 的坏实现：24 677 态里 9 态恢复读回回退之前的 201 字节；O1 全四条、R0、R1、R1b 在三种崩溃时刻归属下全绿；R2（回退抬 lo）也只在「崩溃时刻按切开这一段的屏障归属」（se）下判红，崩溃注入今天的归属（第一个被扣下的写，fw）下照样绿 | 一、2 |
| G2「第 m 次已确认发布」命不了名 | 打中（量过，副本） | 恢复采纳了没交回的那次发布（第一个文件在飞）之后，挂载、改内容、回退到那一版的根或挂载写的根：fft 13 态里 7 态挂得上，18 次回退做成，读回的内容不是任何一次已确认发布的内容 | 一、3 |
| G2 回退目标只能按内容命名 | 量过（副本），条款缺一句 | 回退到挂载写行 / 暖机写的根：rbm 41 011 次做成，那条根不是任何发布调用写的，「根 → 第几次发布」表查不到，只能按内容对上 | 一、3 |
| G2「回退被拒」 | 推的 | 在任何写之前拒：RefFS 当空操作，定义得出。落盘途中失败（`RollbackError::Publish`）：没有「实现报的回退根」，m 只能取调用方自己交的目标；冻结之后原样重发成功就是回退做成，白名单的操作结局里没有「原样重发」 | 一、4 |
| G2「回退目标是被抛弃时间线的根」 | 没打中（推的） | 实现拒（`OnAbandonedTimeline`），O1 逐步对拍要求拒；m 对得上（那次发布当时确认过），坏实现接受它时 RefFS 放行而 O1 判红。今天的历史步「崩溃恢复抛弃根」已不再造出被抛弃的根 | 一、5 |
| U1 + U2 ② 抛弃根的恢复 | 推的 | 「每次恢复之后先判读出的状态在允许集合里」在抛弃了已确认发布的恢复上必红；射程句只点名说谎的设备与丢整块盘 | 一、5 |
| U6「O1 判官在同一份镜像上复算同样判拒」 | 推的（判别子观测不到） | O1 的崩溃判官只收版本目录、最新已持久根与读回，看不到镜像；tgt 两段里挂载被拒的 25 态，O1 复算不出那条理由 | 一、6 |
| W1 在 mkfs 中途崩溃 | 打中不分辨臂（量过，部分） | mkfs 第一道屏障之前崩溃：W1 那一笔整段清零撕成第三态时，旧池槽 0 没了、槽 1 还在，逐档择到旧池槽 1、读出旧池最后一版，checker 报 I-2.1 / I-3.1 / I-4.8 / I-7.4 红（22 070 态里 7 348 态）；不清（今天）在同一段里同样挂上旧池。屏障之后：W1 0 态择旧，今天 12 态择旧 | 二 |
| V1 ① ② / V2 | 没打中（量过，部分） | 屏障之后系统配置写到一半的态，逐档都择到本池槽 1 或整池拒，没有平局 | 二 |
| X2 白名单 | 推的 | 要补三样：崩溃时刻归属（哪几步在崩溃之前已交回）、「根 → 版本」表收非发布调用写的根与恢复读出的起点、原样重发的结局 | 三 |
| X3 数的取法 | 推的 | 第二轮 U3 的 RefFS-Q 替身取 {窗前, 窗后}，等于默认了 R1b / R2 与 fw；入库重做时先定回退是不是边界与崩溃时刻归属，否则同一窗的态数不是同一个量 | 四 |

## 一、X1：G2、G1 与 U1 / U2 的条款句

被攻的字面（照第二轮判决原文）：G2「回退算 RefFS 的一个操作，目标「第 m 次已确认发布的内容」由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对」（`research/prompts/m3-formal-r5-main-verification.md` 用户定案一节）；U2 ①「边界只取历史里的显式发布点（发布调用与 fsync）」；U1「fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在」；Q 的 `lo` 是「最后一次已返回的 fsync 覆盖到第几个操作」（`research/prompts/m3-formal-r4-forks.md` 第 11 行记号一段）。实现里回退是一次普通发布：「新根持久之后才交回 `Ok`（与 fsync 同一个返回条件）」（`crates/singlefs-core/src/mount.rs:5231`），条款上是 `.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「**管理员回退是挂着时的一次向前发布**」。

### 1　回退做成算不算边界、抬不抬 lo（量过，副本）

八条与 G2 合起来没有一句说回退做成之后 lo 动不动、j 能不能落在回退末尾。照字面能读出四种（`main.rs` 的 `Reading`）：

| 读法 | lo 由谁抬 | j 可落在哪 |
|---|---|---|
| R0 | 只有 fsync（这几段历史没有 fsync ⇒ lo 停在历史开头） | 发布调用末尾 |
| R1 | 发布调用返回、fsync | 发布调用末尾（回退不是） |
| R1b | 同 R1 | 发布调用末尾与回退末尾 |
| R2 | 发布调用、回退做成、fsync | 同 R1b |

历史 rbm：第一个文件（3000 字节，`v@0`）→ 覆盖写（201 字节，`v@1`，基线到这里）→ 回退 `RingRoot { index_from_newest: 1 }`（目标是第一个文件那一版）→ 关掉会话、可写挂载。枚举回退与重挂两窗，24 618 态（`runs/sizes-rbm.log`）。每态并排判：O1 判官全四条（`crash_recovery_disagreement`，入参照崩溃注入第一截：模型每步攒的版本目录；③ 的下界取基线里最新的根与这一窗已持久根槽的较大者）、四种读法下的 RefFS-Q（允许集合 {S_lo} ∪ {S_p : lo < p ≤ h，p 是边界}，回退之后的内容 = 目标那一版的），h 是崩溃时在飞的那一步，三种归属（见 2）。产物原样（干净构建）：

- 「rec=v@0 rebuilt=false h_fw=3 h_lp=3 h_se=3 o1=green R0.fw=green R1.fw=red R1b.fw=green R2.fw=green R0.lp=green R1.lp=red R1b.lp=green R2.lp=green R0.se=green R1.se=red R1b.se=green R2.se=green count=8201」（`research/prompts/m3-formal-r6-opus-model/runs/lo-rbm-clean-0.log:12`）
- 「rec=v@0 rebuilt=false h_fw=3 h_lp=3 h_se=3 o1=green R0.fw=green R1.fw=red R1b.fw=green R2.fw=green R0.lp=green R1.lp=red R1b.lp=green R2.lp=green R0.se=green R1.se=red R1b.se=green R2.se=green count=12306」（`research/prompts/m3-formal-r6-opus-model/runs/lo-rbm-clean-1.log:10`）
- 「rec=v@1 rebuilt=false h_fw=2 h_lp=2 h_se=2 o1=green R0.fw=green R1.fw=green R1b.fw=green R2.fw=green R0.lp=green R1.lp=green R1b.lp=green R2.lp=green R0.se=green R1.se=green R1b.se=green R2.se=green count=4096」（`research/prompts/m3-formal-r6-opus-model/runs/lo-rbm-clean-0.log:14`）

两份日志的 `rec=v@0` 行相加 20 522 态（8 + 1 + 8 201 + 3 + 12 306 + 3），读回的都是回退做成之后该有的 3000 字节；R1 在三种归属下全判红，R0、R1b、R2 全绿，O1 全绿。读回 201 字节的 4 096 态是崩在回退单元屏障之前，四种读法都绿。

四句：
- 分不分辨臂：分辨 R0 / R1 / R1b / R2 这四种读法（R1 在 20 522 格假红，另三种不假红）；不分辨 P 与 Q。
- 判别子看不看得到：看得到——回退交没交回是调用方的事实（实现交回 `RolledBack`），写条款的问题。
- 满足的分句：X1「有没有一段历史让它把合法状态判坏」，被打的是 U2 ① 的边界名单（发布调用与 fsync）与 G2 的「算 RefFS 的一个操作」合在一起没给回退定边界身份。
- 跑前条款给的改法在这几格上中不中：判决没给这一格的改法。我提的（零轮，只在我的模型上量过）：「回退做成与发布调用返回、fsync 返回一样，是边界，并把 lo 抬到它的末尾」（R2），在这 20 522 格上不假红（量过）。R0 不假红是因为回退的内容与第一个文件那次发布的内容逐字节相同、而 R0 的 lo 从不抬（内容巧合），它的代价在下一节。

### 2　回退返回早于持久的坏实现：只有 R2 加「按切段屏障归属」判得红（量过，副本）

坏实现（`r6_patch_copy.py`，只在副本里）：回退那次发布照常写单元、发单元之后那道屏障，之后的屏障都不发、根槽写不带 FUA，就交回 Ok。D23 已定项 15 让「记录持久、根槽没持久」由记录重建，所以要让记录也不持久才走得到「回退做成而崩溃之后读回回退之前」。录制流里回退的记录、根槽、系统配置轮换并进一段，由重挂那一步的第一道屏障切开（`runs/sizes-rbm-mut.log` 第 1 段 5 个写）。

三种崩溃时刻归属（`main.rs` 的 `attributions`）：fw = 崩溃注入今天的写法，段里第一个被扣下的写落在哪一步——「.step_containing(stream_indexes[segments[segment_index][first_withheld]])」（`crates/singlefs-checker-tier/src/crash_injection.rs:2064`）；lp = 已持久的写里最晚那一个落在哪一步；se = 切开当前段的那道屏障（或那次 FUA 写）是哪一步发的。守约的实现一步交回之前自己发完切段的屏障，所以 se 不会把守约实现的态归到下一步（干净构建上 se 与 fw 的判定逐行相同，见上节三行）。

产物原样（坏构建）：「rec=v@1 rebuilt=false h_fw=2 h_lp=2 h_se=3 o1=green R0.fw=green R1.fw=green R1b.fw=green R2.fw=green R0.lp=green R1.lp=green R1b.lp=green R2.lp=green R0.se=green R1.se=green R1b.se=green R2.se=red count=9」（`research/prompts/m3-formal-r6-opus-model/runs/lo-rbm-mut-0.log:15`）。这 9 态读回 201 字节（回退之前），而切开那一段的屏障是重挂发的：回退已经交回 Ok。只有 R2.se 判红；O1 全四条绿（③ 只认已持久的根槽，回退的根槽没持久）；R0、R1、R1b 在三种归属下全绿，R2 在 fw、lp 下也绿。坏构建其余 24 668 态的判定与干净构建同形（R1 在读回回退内容的 20 573 态假红）。

四句：
- 分不分辨臂：分辨「回退抬不抬 lo」与「崩溃时刻怎么归属」两件事的四乘三种组合，只有 R2 × se 抓得到。
- 判别子看不看得到：只看镜像看不到「回退交回了没有」——同一份镜像，崩在回退交回之前合法、之后不合法；看得到的是录制流里那道屏障由谁发。fw 把这 9 态归给回退那一步，等于替坏实现选了「还没交回」那个时刻。
- 满足的分句：X1「放过坏状态」。G1 的合取（RefFS ∧ O1 全四条）在 R0、R1、R1b 下与 fw 下都放过；U1 的「fsync 返回成功之后……必须在」写对了承诺，但调用点按 fw 给 hi 时，这句承诺在「返回早于持久」这一类上没有判别力。
- 改法：判决没给。我提的（零轮，只在我的模型上量过）：(a) 同上节 R2；(b) 调用点交给 RefFS 的崩溃时刻按「切开当前段的那道屏障由哪一步发」定（se），不按「第一个被扣下的写」定。两样都要：只做 (a) 在这 9 格上不中（R2.fw、R2.lp 绿，量过）；只做 (b) 不中（R0.se、R1.se、R1b.se 绿，量过）。同一机理推到发布调用（推的，没造坏实现）：R0 下 lo 永远不抬，任何「发布返回早于持久」都放过。

### 3　「第 m 次已确认发布」命不了名的回退目标（量过，副本）

G2 的目标是「第 m 次已确认发布的内容」，m「由调用方从实现报的回退根翻出来」——第二轮攻方给的翻法是在调用方攒的「第几次发布写出的哪条根」表里按 (txg, 实例) 查。回退候选集是「根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件」（`crates/singlefs-core/src/mount.rs` 的 `rollback_candidate` 文档注释），候选集里的根不都是发布调用写的，内容也不都是某次已确认发布的。

量法（`tgt` 模式）：每个崩溃状态上恢复 → 可写挂载 → 一次改内容的发布 c3（已确认）→ 对根环里每一条可读根各试一次回退（用户选哪条放开扫，只固定前缀与崩溃）。

- fft：mkfs → 可写挂载 → 第一个文件（201 字节，`v@2`）在飞。基线切在那次发布自己的单元屏障之后（缩历史长度：单元段 2^20 态全读回「没有文件」，不产生被采纳的版本），只枚举记录、根槽、系统配置三段，13 态（`runs/tgt-fft.log:8`）。产物原样：「rec=v@2 target_origin=continuation_mount -> ok:v@2:unnameable count=14」（`research/prompts/m3-formal-r6-opus-model/runs/tgt-fft.log:14`）、「rec=v@2 target_origin=original_run -> ok:v@2:unnameable count=4」（`research/prompts/m3-formal-r6-opus-model/runs/tgt-fft.log:16`）。12 态恢复采纳了在飞的第一个文件，其中 5 态可写挂载被拒（「rec=v@2 mount_refused:NewerStateStillUnreadableAfterOneReread count=5」，`research/prompts/m3-formal-r6-opus-model/runs/tgt-fft.log:12`），7 态挂上；这 7 态上回退到重挂写行 / 暖机写的根（14 次）与那次第一个文件自己的根（4 次，根槽已持久的那几态）都做成，读回 201 字节。那次发布从没交回，历史里已确认的发布只有 c3：「第 m 次已确认发布」里没有一个 m 的内容是 201 字节。
- rbm（同第 1 节的历史，24 618 态全跑）：回退到重挂写的根做成 41 011 次（`runs/tgt-rbm-sum.txt`），读回的都是某次已确认发布的内容，但那条根不是任何发布调用写的，「第几次发布写出的哪条根」表里查不到，只能按内容对上。（`original_run` 那几行把起点第一个文件的根、覆盖写的根与回退、挂载写的根并在一起计，名字里的 `by_content_only` 对前两种不准，这几行不拿来下结论。）

四句：
- 分不分辨臂：分辨 G2 的两种写法——「m 指第几次已确认发布」在 fft 18 格命不了名（RefFS 只能判红或弃权），「目标是 RefFS 历史里一个已持久的版本：某次已确认发布之后的内容，或某次恢复读出、RefFS 以它重新起头的那一份」在这 18 格上命得了名（推的：按 U2 ② 那份起头内容就是 201 字节）。
- 判别子看不看得到：RefFS 看不到根；调用方看得到实现报的根，但表要收挂载写行 / 暖机、抬 F、回退、卸载写的根，并把每条映到写它时的现行版本。
- 满足的分句：题面 X1「G2 的「第 m 次已确认发布」在……这几格上定义得出吗」，新的一格是「回退目标是恢复采纳的、没交回的那次发布」。
- 改法（零轮，只在我的模型上量过可达性，改法本身推的）：G2 改成「回退之后的内容 = 调用方交入的版本号 m 那一版的内容；m 指 RefFS 历史里的一个已持久版本——一次已确认发布之后、或一次恢复读出（U2 ② 重新起头）的那一份；调用方按实现报的回退根在『根 → 版本』表里查，表收这一池里写过根的每一个入口（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载），挂载写的根映到挂载时的现行版本」。

### 4　「回退被拒」（推的）

- 在任何写之前拒（`RollbackError` 除 `Publish` 落盘途中失败之外的每个成员）：RefFS 当空操作，m 用不上，定义得出。
- 落盘途中失败：「这次发布冻结在分配器上等原样重发（`transaction::resend_the_frozen_publish`），重发成功就是回退做成。」（`crates/singlefs-core/src/mount.rs:4812`）。两个缺口：① 这时没有「实现报的回退根」（`RolledBack` 只在 Ok 时交回），G2 字面的 m 来源不存在——m 要从调用方自己交给实现的目标翻；② 原样重发成功之后回退才算做成，而白名单的操作结局（第二轮攻方 U4 一节给、判决取的那份）只有「成、在任何写之前拒、写了之后报错」三样，U1 的「之后的操作接在下一次恢复读出的状态上判」默认报错之后先有一次恢复；D23 已定项 14 的失败处置是同一次挂载里「重发成功才建下一次发布」，中间没有恢复。照现字面，重发成功之后再崩、丢了那次回退，RefFS 按「报错那次两可」放过（推的；harness 的历史步里没有原样重发，`fault_injection` 停在报错那一步，今天走不到）。普通发布同一个缺口（D23 已定项 14「这一版的失败处置」）。
- 改法（零轮，推的）：操作结局加第四样「报错之后原样重发成功」，与「成」同样抬 lo；G2 的 m 写成「从调用方交给实现的回退目标翻」，不写「实现报的」。

### 5　「回退目标是被抛弃时间线的根」与抛弃根的恢复（没打中，推的）

- 实现拒：「OnAbandonedTimeline,」（`crates/singlefs-core/src/mount.rs:741`）；O1 逐步对拍要求拒：「required_refusals.insert(ModelRefusalReason::RollbackTargetOnAbandonedTimeline);」（`crates/singlefs-harness/src/model.rs:1387`）。m 定义得出（那条根是当时已确认的发布写的）；坏实现接受它时 RefFS 照 G2「不判目标选得对不对」放行，O1 逐步对拍判红，之后崩溃注入第一截的 ②（择到的根要是模型提交过的某一版）也红。这一格 G1 兜住，没打中。
- 今天的历史步造不出被抛弃的根：「这一步不再造出被抛弃的根」（`crates/singlefs-harness/src/history.rs:395`，那一行后半是乙罩不到的那一形另由 `tests/common/mod.rs` 的辅助函数造）。抛弃要恢复读不出一条已持久的根，是读故障；U1 的射程句只点名「说谎的设备、丢整块盘 C519」。U2 ②「每次恢复之后 RefFS 先判读出的状态在允许集合里」在抛弃了已确认发布的恢复上必红（推的）。改法（零轮，推的）：射程句写成「恢复时有读故障的（含抛弃根的那一形），那一次恢复不判、以读出的状态重新起头，之后照判」，或整段历史弃权，二选一要写明。

### 6　U6「O1 判官在同一份镜像上复算同样判拒」（推的，判别子观测不到）

O1 的崩溃判官只收三样：「committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,」「newest_persisted_root: Option<ModelRootKey>,」「read_back: &ObservedReadBack,」（`crates/singlefs-harness/src/model.rs:2247-2249`），没有镜像；它的拒绝模型要调用方先告诉它「最新那条根读不出」（`answer_mount_writable_with_the_newest_root_unreadable`，`crates/singlefs-harness/src/model.rs:1276`）。tgt 两段里挂载被拒共 25 态（fft 5、rbm 20），理由都是 `NewerStateStillUnreadableAfterOneReread`（第二轮攻方量到这一类全是第三态撕了系统配置槽；我这两段没逐态核是不是第三态、拒得合不合法）。O1 要「复算」，入参只能从录制流的第三态那一位来（与 ③ 的下界同源，记 C191 那一类账），或改成 checker 在镜像上复算。分辨的是「拒绝合法」四条件里第二条的执行者，不分辨臂；改法（零轮，推的）：第二条写成「拒的理由那一判在同一份镜像上由 checker（或拿录制流第三态位的 O1）复算为真」。

## 二、X1：W1 在 mkfs 中途崩溃（量过一段历史，W1 那一臂只跑完 22 070 / 82 970 态）

被攻的定案：「mkfs 写元数据之前把每块盘的 [0, 根环基址) 整段写 0」（`research/prompts/m3-c651-r1-main-verification.md` 用户定案一节），连同 V1 逐档。今天的 mkfs 自己写明「// mkfs 中途崩溃第一版没有条款（层 0 从 mkfs 之后的池起枚举，`.claude/kb/layout/01-first-txn.md` 八」（`crates/singlefs-core/src/make_filesystem.rs:541`）。W1 那一笔写零照 m3-c651-r1 攻方的补丁放在每块盘清根环之前（`c651_patch_copy.py` 末段，与 `patch_copy.py` 逐字节相同），和根环、journal 环的写零同在 mkfs 第一道屏障之前。

量法（`w1` 模式）：盘上先有一个旧池（槽距 65536、用户动作 u = 5：取号、暖机、第一个文件、覆盖三次，旧槽 0 世代 8、槽 1 世代 7），只有盘 0 是旧盘（`reuse=disk0`），fsid 与新池相同（m3-c651-r1 量出最危险的那一格）；换 io_min 512（槽距 4096）重做 mkfs，**mkfs 本身录成写流**、进层 0 枚举域。写零覆盖非零旧字节时算原地覆写、取第三态（「StepKind::ZeroFill」在 `is_tearable_in_place_overwrite` 的「不是单元写」那一支，`crates/singlefs-checker-tier/src/crash.rs:1843`），撕裂镜像是「新旧不同的那一截前一半新、后一半旧」（同文件 `torn_image_of_in_place_overwrite` 的文档注释）。两块盘都跑 `C651_CANDIDATE=search`（实现与 checker 都逐档）。段形 W1 开 `[14, 1, 1, 1, 4]`、82 970 态；W1 关 `[12, 1, 1, 1, 4]`、13 850 态。日志里 `new@65536gN` 是「fsid 与新池相同、槽距 65536」——那是旧池的槽；`new@4096g1` 才是本池的槽。

结果（W1 关 13 850 态全跑，`runs/w1-off-A-sum.txt`；W1 开跑了序号 0..22 000 与 82 900..82 970，`runs/w1-on-A-partial-sum.txt`）：

| 段 | W1 关（今天） | W1 开 |
|---|---|---|
| 第 0 段（mkfs 第一道屏障之前） | 13 823 态全择旧池槽 0、读出旧池最后一版 | 22 043 态：7 348 择旧池槽 0 读旧池（盘 0 的 W1 写零没落）；7 347 全池没有系统配置、整池拒（落了）；7 348 **逐档择到旧池槽 1**、读旧池最后一版（W1 写零撕成第三态） |
| 第 1–3 段（三次根槽 FUA） | 3 态择旧池槽 0、读旧池 | 3 态整池拒 |
| 第 4 段（系统配置轮换）与之后 | 24 态：12 择旧池（其中 4 态经逐档择旧槽 1）、12 新池 | 24 态：0 择旧池；新池 22（其中 6 态逐档择到本池槽 1）、整池拒 2 |

产物原样：
- 「name=sig w1=true h=W:x=65536:y=512:u=5:reuse=disk0:fsid=same count=4000 seg=0 torn_zero=true all_slot0_unusable=true core=new@65536g7d0 outcome=FileRead(i1,t6):old_last checker_slots=x/new@65536g7|x/x checker_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"]」（`research/prompts/m3-formal-r6-opus-model/runs/w1-on-A-2.log:6`）
- 「name=sig w1=false h=W:x=65536:y=512:u=5:reuse=disk0:fsid=same count=2 seg=4 torn_zero=false all_slot0_unusable=true core=new@65536g7d0 outcome=FileRead(i1,t6):old_last checker_slots=x/new@65536g7|x/x checker_red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"]」（`research/prompts/m3-formal-r6-opus-model/runs/w1-off-A-1.log:22`）
- 「name=sig w1=true h=W:x=65536:y=512:u=5:reuse=disk0:fsid=same count=2 seg=4 torn_zero=false all_slot0_unusable=true core=new@4096g1d0 outcome=NoFile(i0,t0) checker_slots=x/new@4096g1|x/new@4096g1 checker_red=["I-7.7"]」（`research/prompts/m3-formal-r6-opus-model/runs/w1-on-A-tail.log:23`）

读法：
- W1 站住的那一半：新池三条根已持久之后（第 1 段起），W1 让旧池一态都择不到；今天这一段有 15 态择旧池（第 1–3 段 3 态、第 4 段 12 态），其中 4 态正是逐档择到旧池槽 1（新池槽 0 那一写撕了、旧槽 0 被盖掉、旧槽 1 还在）。第 4 段全池槽 0 都不可用的态，W1 下逐档都择到本池槽 1，没有平局（V2 那一句没有对象）。
- 打不中 W1 的那一半：mkfs 第一道屏障之前崩溃，两臂都挂上旧池；W1 只是多出一条路——它那一笔写零撕成第三态时，旧池槽 0 被抹、槽 1 留着（撕裂镜像前一半是零），全池槽 0 都不可用，逐档择到旧池槽 1（世代 7，比被抹的槽 0 旧一代）。实现与 checker 择的是同一槽（`checker_slots=x/new@65536g7`），C651 的验收「两边结论不同判红」在这一格不红。
- checker 报的 I-2.1 / I-3.1 / I-4.8 / I-7.4 四条红在两臂、在什么都没落的态上都有：盘 1 是新盘，旧池缺了半边，是这一组历史的造法带来的，不是 W1 或逐档的结局。

四句：
- 分不分辨臂：第 0 段不分辨 W1 / W2（两臂都择旧池），病根是两臂共用的前提「mkfs 中途崩溃没有条款，第一道屏障之前旧池原样可挂」，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中不分辨臂」不拿它判 W1。第 1–4 段分辨（W1 0 态择旧、W2 15 态），W1 站住。
- 判别子看不看得到：第 0 段里读者只看得到旧池的槽；新池一样东西都还没写，读者分不出「mkfs 做到一半」与「旧池」。看不到。
- 满足的分句：题面 X1「W1 写零在 mkfs 中途崩溃时留下什么，读者照 V1 逐档还择得对吗」——留下的是「旧池槽 0 没了、槽 1 还在」，逐档择到它；「对不对」没有条款可判。
- 改法（零轮，推的）：W1 的射程句写明「W1 只在 mkfs 过了第一道屏障之后成立；之前崩溃，旧池可能被择中（含经逐档择到旧槽 1），mkfs 中途崩溃第一版没有条款」。若要第 0 段也不择旧池，要在写零之前先让旧池不可择（例如先单独把两块盘的旧槽 0、旧槽 1 所在的整段清掉并加一道屏障），这一样我没量。

没跑的：W1 开那一臂其余 60 900 态（序号 22 000..82 900，全在第 0 段，按已跑的 22 043 态的签名是周期重复的，推的）；`reuse=both`（两块盘都是旧盘——「一块盘清了、另一块没清」那一形只在这里有，W1 开 279 974 态）、旧槽距小于新槽距（x = 512）、fsid 不同的几格。命令见「要主 agent 跑的命令」。

## 三、X2：清单完整性（推的，入参是代码事实）

RefFS 的输入白名单今天四样：操作结局、回退目标 m、每次恢复读出的内容、显式发布点。实际要用而没列的：

1. **崩溃时刻归属**（哪几步在崩溃之前已交回）。同一份镜像，崩在一步交回之前合法、之后不合法（一、2 那 9 态）；lo、hi 都由它定。崩溃注入今天按「段里第一个被扣下的写」归（`crates/singlefs-checker-tier/src/crash_injection.rs:2064`），它对守约的实现与按切段屏障归属给出相同判定（一、1 三行），对返回早于持久的实现偏向放过。要写进白名单并写明规则（候选：切开当前段的那道屏障由哪一步发）。
2. **「根 → 版本」表**（G2 的 m 从哪来）：收每一个写根的入口（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载），映到写它时的现行版本；版本含恢复读出、重新起头的那一份（一、3）。今天 G2 只说「从实现报的回退根翻出来」，表的范围与「报错时没有回退根」都没写（一、4）。
3. **操作结局的第四样「报错之后原样重发成功」**（一、4）。
4. 逐字段表（第二轮 U5）这一轮没碰；根记录与系统配置的字段我没逐个过，留给辩方那一侧（这一条是「没做」，不是「没打中」）。

C651 那一题「全池槽 0 都不可用」还影响哪一步（推的，读码）：tail 与 F 那一份都住系统配置槽，逐档择到槽 1 时取的是那一槽的世代（二、表里旧池槽 1 世代 7、被抹的槽 0 世代 8），tail 与 F 可能比被抹的那一槽旧一代：tail 旧只让扫描多扫几条，F 旧由根上带的 F 兜（F 生效值取两处的大者，D16 已定项 1「生效」）；挂载之后下一次轮换写世代 = 本盘自证过的最大世代 + 1，落回槽 0、盖掉撕坏的那一槽（`write_system_configuration_slot` 的世代号算法）。二、量到的逐档择槽 1 的态全部读出了该读的那一版（W1 开第 4 段 6 态新池、第 0 段 7 348 态旧池最后一版），没量到 tail / F 读旧造成的误读。

## 四、X3：报出的数能不能核（推的）

- 第二轮 U3 的 6 态（rb、ow 两窗各 3 态）与 U6 的 3 态：RefFS-Q 替身取 {窗前, 窗后}（第二轮攻方报告 U3-1 一节）。对回退那一窗，这等于默认回退末尾是边界（R1b / R2）、崩溃时刻按 fw 归属。那 6 态读回的是错内容、RefFS 绿是因为错内容恰在 {窗前, 窗后} 里，换读法不改它们的判定（推的）；但同一窗里读回回退内容的态在 R1 下全假红（一、1 量过的机理），所以入库重做时「RefFS-Q 在这一窗判了几态红」要先定回退是不是边界与崩溃时刻归属，否则两次跑的不是同一个量。
- 第二轮 U5 的 16 390 / 24 598 / 12 309 / 57 446：挂载透明比的是 R1、R2 两份只读恢复，不经 RefFS 的 (lo, hi)，崩溃时刻归属不影响它们（推的）；要对上的是盘宽（384 / 256 槽）与 fms 那段关准入造的前缀——入库装置的崩溃注入盘宽是 4 GiB，um 那一格要先把卸载加进历史（第二轮判决第五节第 3 条已写）。
- C651 的 C 组 12、S 组 240：层 0 从 mkfs 之后起枚举，mkfs 那一段不进；W1 落地后 S 组的盘面要重造（mkfs 多一笔整段写零），这两组数要在 W1 开的装置上重做才是同一个量；二、表明第 0 段的结局不随 W1 变「择对」，所以 S 组若把 mkfs 中途崩溃并进来，「230 择对」那一格的分母会变。

## 改法一览（各修哪一格）

全部是我提的，「只在我的模型上量过、被攻过零轮」；「量过」= 上文贴了副本上原样输出的那几格，「推的」= 按代码与条文推、没实现没跑。

| 改法 | 修的格 | 在打中的格上还中不中 | 量过 / 推的 |
|---|---|---|---|
| R2：回退做成是边界、抬 lo（与发布调用返回、fsync 返回同） | 一、1 | 不中：rbm 20 522 格不假红 | 量过 |
| R2 + 崩溃时刻按切段屏障归属（se） | 一、2 | 不中：9 格判红；只做 R2 或只做 se 都还中 | 量过 |
| 发布调用返回也抬 lo（R0 → R1 的那一半） | 一、2 的推广 | 条款面；R0 下任何「发布返回早于持久」都放过 | 推的 |
| G2 的 m 指「已持久版本」（已确认发布之后或恢复读出起头的那一份），「根 → 版本」表收每个写根的入口 | 一、3 | fft 18 次回退命得了名 | 可达性量过、改法推的 |
| m 从调用方交给实现的目标翻；结局加「报错之后原样重发成功」并抬 lo | 一、4 | 条款面；今天走不到 | 推的 |
| 射程句写明读故障下的恢复（抛弃根）那一次不判、或整段弃权 | 一、5 | 条款面 | 推的 |
| U6 第二条的复算者写成 checker（或拿录制流第三态位的 O1） | 一、6 | 条款面；挂载被拒 25 态 | 推的 |
| W1 射程句：只在 mkfs 过了第一道屏障之后成立，之前崩溃可能择中旧池 | 二 | 条款面；第 0 段两臂都择旧池 | 推的（结局量过） |

## 没打中的形状

- G2「回退目标是被抛弃时间线的根」：实现与 O1 都拒，G1 兜住（一、5，推的；今天的历史步造不出被抛弃的根，没量）。
- 回退在任何写之前被拒：RefFS 当空操作，定义得出（推的）。
- rbm 干净构建上 O1 全四条在 24 618 态全绿；R0、R1b、R2 在三种崩溃时刻归属下 24 618 态全绿（量过）。也就是说，八条照 R2 写、调用点照今天的 fw 归属，在这段历史上不把合法状态判坏。
- W1 开、mkfs 过了第一道屏障之后（第 1–4 段 27 态）：没有一态择到旧池，逐档全择本池槽 1 或整池拒，没有世代号平局（量过）。
- tgt rbm 里回退到挂载写的根 41 011 次全部做成、内容都等于某次已确认发布的内容：只是命名问题，不是误读（量过）。
- 取样范围：rbm 一段历史（回退与重挂两窗全量，干净 24 618 态、坏构建 24 677 态）；fft 一段（缩成第一个文件那次发布的单元屏障之后，13 态）；W1 一组（旧槽距 65536 → 新 512、u = 5、只有盘 0 是旧盘、fsid 相同，W1 关 13 850 态全量、W1 开 22 070 / 82 970 态）。没取：并行提交、fsync 独立一步（今天入口没有）、两次以上崩溃、`reuse=both` 等其余 W1 格。
- 一次抽样的「没打中」不算没打中（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。

## 这条腿自己的限度

- RefFS-Q 是替身：每个位置做完之后的内容取自那时整池持久的镜像上的只读恢复（实现的输出），不是照条文另写的 RefFS；四种读法与三种归属是我按条款字面拆出来的，条款作者的本意可能就是 R2（D23 已定项 14 说回退是「一次普通发布」），那样一、1 是「条款没写全」而不是「条款写错」。
- 坏实现只造了一种（回退返回早于持久）；「发布调用返回早于持久」那一半是按同一机理推的，没造。
- se 归属是我在原型里按录制流的屏障位置算的（`segment_terminator_position`），没有独立路径核过它与崩溃注入里任何现成量的关系；fw 是照 `crash_injection.rs:2064` 的读法重写的一份，不是调它。
- tgt 的 `nameable` 标签对 `original_run` 那几行不准（一、3 括注），结论只用了 `continuation_mount` 与 fft 两处。
- W1 那组历史只有盘 0 是旧盘，checker 的四条红来自造法（盘 1 新盘）；「一块盘清了、另一块没清」只在 `reuse=both` 里有，没跑。W1 开只跑了 22 070 / 82 970 态。
- 估时与跑批：机器负载 45–124（别的会话在跑 random_histories、c331、bf_r2_sweep 等），同样 1 万态一批的挂钟在 30 秒到超过 230 秒之间。lo 与 tgt 先跑一小段（tgt rbm 500 态 9.9 秒）再按每批 4 300–12 309 态切；W1 开先跑 1 000 态（18.7 秒）再按每批 9 000–12 000 态切。被前台时限（`timeout 225–235`，只为不超过工具的前台上限，不是给批设的限时）截断的几批（lo rbm 坏构建整段一遍、tgt rbm 两遍、W1 关整段一遍、W1 开三遍）退出码 124，输出不算、日志已删，同一序号段重跑过的才入库；W1 开 22 000..32 000 两次都被截断，没再跑。
- 状态数：入库日志合计 lo 49 295、tgt 24 631、W1 35 920 态；连同被截断与估时的几遍（按发起的序号段上界算），全部原型跑的全量约 2.6 × 10⁵ 态，单次调用最多 4 万态（上限 20 万），跑过的每段历史全量不超过 8.3 万态，在 10⁶ / 10⁷ 之内。
- 包装的结局：第一次编译 252（内存排不上，`RUN_WITH_MEMORY_CAP_WAIT_SECONDS` 设 150 后重编）、一次 250（上限 1G 编 core 撞顶，改 3G 重编）、两次 251（上限超过峰值表 2 倍没给理由，补 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 重跑）；这几次都没有产出进结论。开跑前 `ps` 看到别的会话的 cargo / 测试在跑（e168、random_histories、bf-proto、c331），没有 qemu / fio / vm-bench / e152。
- 线程：每件 5 线程、串行，没有并行起两件。

## 要主 agent 跑的命令

工作目录：仓根。环境：不用另设（`rerun.sh` 自己设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）。编译 3G、跑每件 2G，5 线程。不是重型测试（原型是拷贝里新加的 crate，只调 `singlefs-checker-tier` 的库函数，不跑它的测试目标）。

1. W1 开那一臂补全（序号 0..82 970，9 份日志，空闲时每份约 1–3 分钟，负载高时一份可到十几分钟）：
   `bash research/prompts/m3-formal-r6-opus-model/rerun.sh /tmp/claude-1000/m3-formal-r6/tree /tmp/claude-1000/m3-formal-r6-main-w1 w1-on-A`
   判：`grep -h 'name=sig' /tmp/claude-1000/m3-formal-r6-main-w1/runs/w1-on-A-full-*.log` 按 `seg=` 与 `core=` 汇总；看第 0 段 `core=new@65536g7d0`（逐档择旧池槽 1）的态数，与第 1–4 段有没有 `core=new@65536`（W1 下择旧池，本报告预期 0）。
2. 「一块盘清了、另一块没清」那一形（两块盘都是旧盘，W1 开 279 974 态，要按每次 ≤ 20 万态切两次）：先在草稿副本里照 `rerun.sh` 编好，再
   `cd /tmp/claude-1000/m3-formal-r6-main-w1 && env C651_CANDIDATE=search C651_MKFS_CLEARS=1 nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh 2G ./target/release/r6probe w1 'W:x=65536:y=512:u=5:reuse=both:fsid=same' 5 140000 0 > runs/w1-on-D-0.log 2>&1`
   与同一条把 `140000 0` 换成 `279974 140000`、日志名换成 `w1-on-D-1.log` 的一条；W1 关对照：同上去掉 `C651_MKFS_CLEARS=1`、`31142 0`、`w1-off-D.log`。判：第 0 段里一块盘逐档择旧池槽 1、另一块盘择旧池槽 0 的态（`checker_slots` 里两盘一个 `x/new@65536g7`、一个 `new@65536g8/…`），实现择哪一份、恢复读出什么。
3. 本报告入库日志的复跑（核数用）：`rerun.sh` 不给件名全跑；比 `runs/` 下各份的 `^rec=`、`name=sig`、`states_judged` 行（去掉 `elapsed`）。

## 没做什么

- 不判辩方那几格；没读禁读清单里的文件（这一轮辩方腿的提示与产出、m3-formal-code-r1 一轮的全部腿产出；`ps` 里看得见 `/tmp/claude-1000/m3-formal-code-r1-attack/` 的进程，没读它的任何文件）。
- 没跑 checker 档的测试、Verus、门禁、变异；原型经 `cargo build -p r6probe` 编、直接执行编出的二进制（都经内存包装）。没改仓里的 `crates/`，没写 kb，不替主 agent 采纳。
- 副本上的数不进 kb；要引，照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节在入库装置上重做。
- 没量：发布调用返回早于持久的坏实现；原样重发；读故障下的抛弃根恢复；U5 逐字段表的其余字段；P / Q 的分辨；W1 的 `reuse=both` 与其余几何。
- 草稿：交回前删掉 `/tmp/claude-1000/m3-formal-r6-attack/tree`（仓副本）与 `/tmp/claude-1000/m3-formal-r6-attack/target`（编译目录）；草稿目录里留日志（估时与被截断那几遍之外的都已拷进模型目录 `runs/`）。复跑用 `rerun.sh` 从冻结副本重建。
