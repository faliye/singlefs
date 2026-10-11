# 全池没有可用的系统配置槽 0 时槽 1 在哪：第一轮判决（2026-10-02）

<!-- doc-lint:not-numbers V1 V2 V3 G03 S1 S2 S4 S6 S8 O3 W1 W2 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 | `research/prompts/_m3-c651-r1-body.md`、`_m3-c651-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-c651-r1-snapshot/`：`crates-sha256.txt`（230 份，冻结副本 `/tmp/claude-1000/m3-c651-r1/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK；`kb-sha256.txt` 只 `.claude/kb/checks-owed.md` 对不上（别的会话追加了行）；腿引它的只有正推的 `checks-owed.md:520`（C651 那一行），现查仍在第 520 行、内容同开工时 |
| 云端正推（Sonnet） | `research/prompts/m3-c651-r1-sonnet-output.md`（248 行，sha256 b8d12c28…）；模型 `…-sonnet-model/`（`c651_probe*.rs`、`count_slot0_b.py`） | 探针数只当线索，判决靠攻方复跑坐实的那一份 |
| 云端攻方（Opus） | `research/prompts/m3-c651-r1-opus-output.md`（sha256 8cb81c7a…）；模型 `…-opus-model/`（`patch_copy.py`、原型 `c651proto/`、`run_batch.sh`、汇表脚本、`runs/`） | 主 agent 在 `/tmp/claude-1000/m3-c651-r1-main-check/` 照报告「复跑」一节在冻结副本的拷贝上编（3G 上限，27 秒）、跑九个候选：九件 rc 全 0，九份日志去掉 `elapsed` 行与 `runs/b2-*.log` 逐行相同，`summary.py` 汇表与 `runs/summary2.txt` 逐行相同 |
| 引文 | 两份腿报告 | `python3 scripts/cite-check.py <两份报告> --root /tmp/claude-1000/m3-c651-r1/tree --root . --background research/prompts/_m3-c651-r1-background.md`：「✓ 核了 74 处引文，对上 74 处，没判 0 处」 |

攻方的数是副本装置（内存稀疏盘、层 0 枚举域、192 段历史 × 9 个读法、每读法 3 240 态）上量的；照 `.claude/rules/three-way-inference.md`，进 kb 之前在入库装置上重做（第五节第 3 条）。

复跑坐实的汇表（`runs/summary2.txt`，C 组 = 新池单次崩溃撕了两盘槽 0，S 组 = 用过的盘换 io_min 重做 mkfs 之后同样的崩溃；ok 择对且挂上、checker_only 只 checker 判红、stale_mounted 择到旧池残留槽而挂上、refused 整池拒）：

```
summary label=const group=C states=12 ok=4 checker_only=0 stale_mounted=0 refused=8
summary label=const group=S states=240 ok=92 checker_only=4 stale_mounted=48 refused=96
summary label=refuse group=C states=12 ok=0 checker_only=0 stale_mounted=0 refused=12
summary label=search group=C states=12 ok=12 checker_only=0 stale_mounted=0 refused=0
summary label=search group=S states=240 ok=136 checker_only=0 stale_mounted=52 refused=52
summary label=search-mkfsclears group=S states=240 ok=230 checker_only=10 stale_mounted=0 refused=0
summary label=today group=C states=12 ok=4 checker_only=8 stale_mounted=0 refused=0
summary label=unique group=S states=240 ok=0 checker_only=120 stale_mounted=0 refused=120
```

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| V1-常量 | 冲突：槽距 8192、两盘槽 0 都坏时 checker 读 4096 判不出槽 1、I-8.5 红，core 择到（探针 S2） | 打中（量过）：io_min 8192 / 65536 的新池一次崩溃撕了两盘槽 0，常量读法读 4096 的空档整池拒，C 组 12 态拒 8；今天的组合 checker_only 8 | **出局，两腿同向、复跑坐实**：与 D22 已定项 16「槽 i 的设备内偏移 = i × 固定结构槽距」、I-8.5 相冲 |
| V1-拒 | 冲突：全池每盘槽 0 都坏在「每盘一槽系统配置坏」的容错承诺里（D22 第 178 行）；E170 批 1 有 14 态是这一形 | 打中（量过）：槽距 4096 的池同一种单次崩溃也整池拒，C 组 12 / 12 | **出局，两腿同向、复跑坐实**：两槽轮换要防的正是一次撕裂抹掉唯一见证者（D23 已定项 3） |
| V1-逐档 | 只有它与条款文字相容；连 ① ② 一起写的候选句在报告 1.5 | 打中（量过）：用过的盘换 io_min 重做 mkfs，mkfs 不清 [4096, 1 MiB)，旧池槽 1 世代号更大就被择中；S 组 240 态 136 择对、52 择旧槽而挂上（fsid 相同时读出旧池文件）、52 整池拒 | **读法站住，残留另立一账**：残留那一格打中的不只是逐档——同一组 S 态上常量 48 / 96、首个 48 / 48、探测 24–48 / 96–120、唯一 0 择对 120 拒，只有「逐档 + mkfs 清零」230 择对、0 择旧（复跑坐实）。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中不分辨臂」，它的病根在各读法共用的前提（mkfs 不清 [0, 根环基址)），不拿它判 V1；V1 取逐档，残留进第三节岔路 W |
| V2 | 规则没写，要新写；「同号取偏移小」全仓只有 `layout/01-first-txn.md:160` 的描述 | 平局只出现在本池槽 1 对旧池残留之间；mkfs 清零之后这一问不存在（推的） | **站住，写成条款候选句**：「几档都收得到时取世代号最大的，世代号相同取偏移小的」（照实现今天的行为，零轮）；岔路 W 取 W1 之后它只在介质损坏造出的残留上有对象 |
| V3 | 缺 io_min 8192 那一格：harness 档一条用例；checker 档坏盘输入没有这种坏法也没有 io_min 轴 | 建议 harness 两张坏镜像（io_min 8192 抹两盘槽 0；真走一遍旧池再重做 mkfs），加一条层 0 流 mkfs(io_min 8192) → 第一次可写挂载（33 态，2 态两盘槽 0 同撕） | **站住**：两张坏镜像进下一批实现；层 0 那条流进 checker 档，算多一档枚举（`.claude/main-agent.md`「禁止」第一条：要加就加） |

另冒出的、与候选无关的（攻方「另一笔账」，复跑坐实）：同 fsid 在用过的盘上重做 mkfs 的那几格，每个候选都有 checker 报 I-7.7 红（全池槽 0 都可用的态里每个候选 70 个）；根因没查（攻方推是旧池的根留在新 mkfs 按新槽距清的那一段之外，与 C484 同类）。登记一笔新欠账，不挡这一轮。

两条腿也都点出背景材料没写的两份读法：GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` 第 671 行起的逐档、O3 执行器的暂代读法（收到第一个就停、不比世代号，攻方「首个」那一臂）。定案之后三处（checker、GPU 判器、O3 执行器）一起改到同一句。

## 三、交用户的岔路（零轮）

| # | 问题 | 候选 |
|---|---|---|
| W | 用过的盘上重做 mkfs 留下的旧池系统配置槽怎么处置 | **W1 mkfs 清零**：mkfs 在写元数据之前把每块盘的 [0, 根环基址) 整段写 0（与 D22 已定项 8 第 3 条已有的根环、journal 环整段清零同一种动作，每盘多一次调用、录制流每盘多一步）；**W2 不清**：残留照留，读法照逐档，用过的盘上崩溃之后可能择到旧池槽（fsid 不同整池拒、相同挂上旧池） |

代价（`crates/singlefs-core/src/make_filesystem.rs` 第 525–554 行现数：今天每盘清根环三个区域与 journal 环，`write_zeroes_at` 两处调用；`crates/` 里引写零一步的 `.rs` 31 份，`grep -rln 'ZeroRange\|write_zeroes\|整段清零\|WholeRangeZero' crates/ --include=*.rs | wc -l`）：

| 候选 | 改什么 | 罩住 | 罩不住 |
|---|---|---|---|
| W1 | core 加一处 `write_zeroes_at`（推：3–10 行）；D22 已定项 8 第 3 条与 `layout/01-first-txn.md` 写序表加一步；录制流 mkfs 那一段每盘多一步，层 0 与崩溃放量里钉 mkfs 段步数、态数的值要重钉（推：凡枚举含 mkfs 那一段的用例都动，范围没数） | S 组 240 态 230 择对、0 择旧（复跑坐实）；剩 10 态 checker 报 I-7.7，与读法无关 | mkfs 之后介质自己造出的残留；I-7.7 那一笔 |
| W2 | 0 行 | 新池上的 C 组（逐档 12 / 12 择对） | S 组 52 态择旧槽挂上、52 态整池拒（复跑坐实）；fsid 每次 mkfs 随机时「挂上旧池」那一半要两次 mkfs 撞同一个 fsid（推的，没量概率） |

## 四、站住、要写成条款的（零轮）

1. V1：「全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096），只收自证过、且自述的固定结构槽距等于所在偏移的槽」，写进 D22 已定项 16，① ② 两步（本盘槽 0 可用取它的槽距；否则取池里按设备次序第一块可用槽 0 的槽距）一起写进同一句，今天它们只在布局表里有描述。
2. V2：「几档都收得到时取世代号最大的，世代号相同取偏移小的」。
3. V3：harness 两张坏镜像、层 0 一条 io_min 8192 的流；checker 的 ③ 改成逐档，GPU 判器与 O3 执行器同改；C651 的验收（两边结论不同判红）由坏镜像那两条判。

## 五、下一步

1. 岔路 W 弹窗交用户（附第三节代价）。
2. 用户定案之后与第四节三条一起写回（D22 已定项 16、布局表系统配置槽一段、C651 改状态、新欠账 I-7.7 那一笔）；实现（checker ③、GPU 判器、两张坏镜像、W1 时的 mkfs）进下一批代码；O3 严格跑法在写回之后解冻重跑。
3. 攻方副本上的数在入库装置上重做：坏镜像两条用例与层 0 那条流落地时各自出数，判决引的 C 组 12、S 组 240 那几格对一遍。
4. 这一轮是第一轮；第四节与 W1 都是零轮，下一轮（与代码轮同批）攻它们。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 16 | 不影响 | 2026-10-02 不受影响：这一轮出条款候选，写回等用户定 W |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-10-02 不受影响：W1 要改它第 3 条，等用户定 |
| D23（journal 的角色与格式） 已定项 3 | 不影响 | 2026-10-02 不受影响：只被引作 V1-拒出局的依据 |
| D2（RAID 条带策略） 已定项 19 | 不影响 | 2026-10-02 不受影响：只被引作槽距的定义 |

## 用户定案（2026-10-02）

岔路 W 弹窗交用户，用户定 **W1 mkfs 清零**：mkfs 写元数据之前把每块盘的 [0, 根环基址) 整段写 0，与 D22 已定项 8 第 3 条已有的根环、journal 环整段清零同一种动作。连同第四节三条（V1 逐档、V2 世代号最大同号取偏移小、V3 两张坏镜像与一条 io_min 8192 的层 0 流）一起写回与实现；W1 被攻过零轮，与代码轮同批攻。

用户 2026-10-02 再定（m3-formal-r6 辩方指出，主 agent 现查坐实）：V1-常量就是用户 2026-09-14 收尾弹窗定的「甲」（`records/2026-09-13-总审核.md`「收尾弹窗（同日）」一段：「槽 0 无效时按最小槽距 4096 读槽 1（甲，维持现状）」；`crates/singlefs-checker/src/image.rs` 第 564 行注释同句），这一轮判决第二节把它判出局时没发现它是用户定案。弹窗交用户，用户定**改成逐档**，推翻 09-14 的甲；依据是第一节复跑坐实的 C 组数（io_min 8192 / 65536 的新池一次撕裂两盘槽 0，常量读法 12 态拒 8），写回时进 D22 变更史。
