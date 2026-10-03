# defs-m3-formal-gate-r3 辩方腿（Sonnet）报告：L3、L4

日期 2026-10-01。批次 `records/2026-10-01-里程碑三形式化接入.md#4`（4-5 定义三方第三轮，最终轮）。被判文件读主仓现行版本；引文件写那份文件自己的行号，行号现查。
探针只读：不起 Verus、不起真包装、不跑真阶段；放在 `research/prompts/defs-m3-formal-gate-r3-sonnet-model/`（`SHA256SUMS`、`rerun.sh`）。本轮新冒出的改法都标「零轮」。

## 一、结论摘要

- L3 崩溃验证员（`.claude/agents/crash-verifier.md` 第 3 步，「一格」为单位）：全是没跑成、混合、全是真红三种格都走得通（探针把真仓的空真格登记与核喂给桩包装，三种各跑一遍，第二节原样）。
- L3 门禁分诊（`.claude/agents/gate-triage.md` 第 3 步，「一道」为单位）：「全是没跑成」这一支照字面走不通。每一道经 `lib/stage-cells.sh` 汇总的阶段（研究构建与单测、模型差分与场景、形式化验证三道都是）在有红格时都打一行汇总 ✗：「N 格里 K 格红」，这一行不写「内存包装退 25x」，于是「一道里每一条 ✗ 都写着内存包装退 25x」永远不成立，字面落到「同一道里另有一条不是这样写的 ✗（真红）」那一支，记红、按点名的文件与行归属，汇总行不点名文件，结局是「分不清」。方向是安全的（仍记红、25x 那几条照抄），但「并发 / 环境」的归属拿不到；改前按行读不受这一行影响。**零轮。**
- L3 `gate.sh` 汇总：这一道退 1 时只打一行 ✗ 阶段名与一句「细节在上方」，对得上定义里「绿·红·77」三栏里记红；没有一个字与「没跑成」「并发」冲突。
- L3 判决采的 G11 只补了 E165 一侧的外部杀进程（退 137），Verus 一侧同形没补：Verus 退 137 仍写「Verus 没出判定……编不过或起不来」并给格自己的出路「看上面 Verus 的报错」，与 E165 改前同一种指错方向。同样打中。**零轮。**
- L4 已定格：E1、E2、E3 的改法都落到位且探针对得上；第一轮五格与第二轮 E4 没有被这一轮的改动推翻。
- L4 记账不够：J2 并进的那一行没有写进去；QEMU 两处与第 1 步停下的频率没有落点；E3-e（排队等待）判决一个字没记；3-12 留下的两处旧字面（`stage-inputs.tsv` 第 31 行、阶段文件第 83 行）判决也没记。
- L4 报出的数：54 个 hunk、两份报告的 sha256、六份探针输出与主 agent 草稿里的复跑逐字节相同，今天核得动；「106 处引文 1 处对不上」今天核不动（被引文件被 3-12 改了，28 处里 27 处落在 3-12 改过的文件），「22 份 18 份 OK」与 e2-verus-toy 的键序核不动。

## 二、判定一览

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻它 |
|---|---|---|---|
| E3 G11 崩溃验证员一格里每一条 ✗ 都是没跑成才记没跑成 | 站得住 | 三、1：三种格各跑一遍，字面判与实际行数对得上（15 / 0、8 / 8、0 / 15） | 一格里出现不属这三类的 ✗（本轮没找到） |
| E3 G11 门禁分诊一道里每一条 ✗ 都是内存包装退 25x 才不按文件判 | 字面够不着（意图读站得住） | 三、2：汇总行 `lib/stage-cells.sh:274` 每个有红格的道都有 | 分诊员实际走一遍时把汇总行当不算（本轮没派人走，不算观测） |
| E3 G11 Verus 253 按超时判红（`gate-triage.md`） | 站得住；条款写得比事实宽一点 | 三、3：E165 的 253 仍写「没跑成：内存包装……退 253」，与「这一道的 253 是 Verus 超时」字面相撞，但没有谁给 E165 设限时 | 门禁里出现 E165 一侧的退 253 |
| E3 G11 E165 一侧外部杀进程改照包装文件头 | 站得住；Verus 一侧同形没改，同样打中 | 三、4：探针退 137 一行 | Verus 一侧退 137 时 ✗ 行已写「照包装文件头」 |
| E3 G11 Verus 一侧原样带出包装的 ✗ 与 → | 站得住 | 三、1：探针里「没跑成」那一条行尾带「包装自己打的：「✗ 桩：…」「→ 桩：…」」 | 包装 stderr 里的 ✗ 行没被带出 |
| E3 G11 gate.sh 汇总里这一道退 1 的说法 | 对得上 | 三、5 | 汇总里出现与定义矛盾的类名 |
| E1 J2 不归这一道，并进共用库盲区那一批 | 记账够不着：并进没写 | 四、1：该行不含 J2 的写法 | §43 第 1 行补上这四种写法 |
| E2 G10 / E1 G9 本体 | 本腿不判（L1、L2 归攻方）；只核了顶层摘要确在代码里这一件 | 四、2 | 见攻方报告 |
| E4-b 第 1 步停下的频率「等几次提交的实况再判」 | 记账够不着：没有落点 | 四、3：第三个数据点 | 在 `.claude/kb/checks-owed.md` 或 records 里有一行带计数方法 |
| 第一轮 QEMU 两处记账 | 记账够不着：没有落点 | 四、3 | 同上 |
| E3-e 排队等待（辩方列了，判决未记） | 同样打中判决（遗漏） | 四、4：必红世界 94 处，每处各等满 3600 秒 | 在占满的 slice 上量一格的挂钟 |
| E3-d 判决只收 E165 | 见第 4 行 | 三、4 | 同上 |
| 第二轮报出的数 | 三项核得动，两项核不动 | 四、5 | 见各条 |

## 三、L3：三种格上照改后的字面走一遍

探针（`research/prompts/defs-m3-formal-gate-r3-sonnet-model/`，原样输出在 `rerun-output.txt`，`rerun.sh` 挂钟 89 秒，复跑一次与入库逐字节相同）：`l3-cell-modes.py` 把真仓的空真格登记与核喂给桩包装（`verus-proof-check.py` 的 `check_vacuity`，15 处 requires），三种模式：全部退 252、奇数次退 252 偶数次「验得过」、全部「验得过」；`l3-synthetic-stage.sh` 造一道三格的合成阶段、source 真的 `lib/stage-cells.sh`，看整道的汇总行。没起真 Verus、没起真包装、没跑真阶段。

### 1. 崩溃验证员（定义以「一格」为单位）：三种格都走得通

定义的字面：「一格里每一条」（`.claude/agents/crash-verifier.md:32`）「✗」都写着没跑成才记没跑成；「同一格里另有一条不是这样写的」（`.claude/agents/crash-verifier.md:32`）「✗」（真红）的，那一格记红，两类连同各自的 → 都原样抄。判法一侧的分类键是 `NOT_RUN_MARK`：「NOT_RUN_MARK = "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 "」（`research/scripts/verus-proof-check.py:137`），混合时另打一行：「这一格 {len(real)} 条真红、{len(not_run)} 条没跑成：按红记，不是没跑成」（`research/scripts/verus-proof-check.py:175`）。

`rerun-output.txt` 原样（`## ` 开头的行是探针按定义字面算的）：

```
## mode=notrun 退 1；✗ 行 15 条：写着「没跑成：内存包装…退 <码>」15 条，不是这样写的 0 条
## 字面判：每一条 ✗ 都是没跑成 = True；另有真红 = False
## mode=mixed 退 1；✗ 行 16 条：写着「没跑成：内存包装…退 <码>」8 条，不是这样写的 8 条
## 字面判：每一条 ✗ 都是没跑成 = False；另有真红 = True
## mode=red 退 1；✗ 行 15 条：写着「没跑成：内存包装…退 <码>」0 条，不是这样写的 15 条
## 字面判：每一条 ✗ 都是没跑成 = False；另有真红 = True
```

| 格 | 定义要它记 | 下一步 | 走得通吗 |
|---|---|---|---|
| 全是没跑成（15 条都带包装结局） | 「没跑成」，抄 →，命令与退出码写进报告 | 照每条 → 里按码给的出路（252：`--status` 看谁占着、空出来再重跑），不绕开包装 | 走得通；阶段退 1、不写全绿标记 |
| 混合（8 条没跑成、7 条真红，外加按红记那一行） | 红，两类 ✗ 与各自 → 原样抄 | 先修真红那几条，没跑成的空出来再重跑（按红记那一行的 →） | 走得通；改前这一格会被记成没跑成（辩方 E3-c） |
| 全是真红（15 条都是「照样验得过」） | 红，抄 ✗ 与 → | 格自己的出路（改前置条件） | 走得通 |

一处字面小毛病，不挡路：定义写「表里那一格记」没跑成（第 32 行），而它自己的产出表是一道一行：「一张表：阶段 / 退出码」（`.claude/agents/crash-verifier.md:43`），表里没有「格」这一栏，「没跑成」只能落在「原样的 ✗ 与 →」那一栏里。推翻它：定义的产出表加一栏格名。

### 2. 门禁分诊（定义以「一道」为单位）：「全是没跑成」那一支字面走不通

定义的字面：「一道里每一条 ✗ 都写着内存包装退 25x 的」（`.claude/agents/gate-triage.md:29`）不按点名的文件判，按码归「环境 / 并发」；「同一道里另有一条不是这样写的 ✗」（`.claude/agents/gate-triage.md:29`）（真红）的，那一道记红、照真红点名的文件与行归属。

共用库给每一道有红格的阶段另打一行汇总 ✗，它写的是格数不是包装结局：「✗ %s：%s 格里 %s 格红（%s）」（`.claude/gate.d/lib/stage-cells.sh:274`）。三道名在分诊定义里的阶段都经这个库汇总：形式化验证（`.claude/gate.d/checker-tier-formal-verification.sh:142`）、研究构建与单测（`.claude/gate.d/checker-tier-research-build-and-replay.sh:336`）、模型差分与场景（`.claude/gate.d/harness-model-differential-and-scenarios.sh:256`）。合成阶段里三种格的汇总原样（`rerun-output.txt`，只留 ✗ 行与格名行）：

```
=== vac-notrun
  vac-notrun：红（退出 1）
  ✗ synthetic-verus.sh：1 格里 1 格红（vac-notrun）
=== vac-mixed
  vac-mixed：红（退出 1）
  ✗ synthetic-verus.sh：1 格里 1 格红（vac-mixed）
=== vac-red
  vac-red：红（退出 1）
  ✗ synthetic-verus.sh：1 格里 1 格红（vac-red）
```

三种格上这一行一字不差，所以「全是没跑成」的道里也有一条不写「内存包装退 25x」的 ✗。按字面：

| 道 | 字面落在哪一支 | 归属 | 与意图差在哪 |
|---|---|---|---|
| 全是没跑成 | 「每一条」不成立，落到「另有一条不是这样写的 ✗」那一支 | 记红；汇总行不点名文件与行，归「分不清」 | 意图是「并发 / 环境」，拿不到 |
| 混合 | 「另有一条」支，对（碰巧由汇总行与真红两条共同满足） | 记红，照真红点名归属，25x 那几条连 → 抄 | 对 |
| 全是真红 | 同上 | 记红，照点名归属 | 对 |

后果有界：仍记红（不会当通过）、25x 那几条的 ✗ 与 → 仍按「原样抄」交出去，丢的只有「并发 / 环境」那个归属字。按意图读（汇总行是共用库打的汇总，不是真红）走得通，但定义没写这句。没派分诊员实际走一遍（不派子 agent 核子 agent），所以「按字面会落到分不清」是读代码推的，不是观测。
同类先例：改前的字面是按 ✗ 行逐条读（转述 3-12 报告「改前」原文），不受汇总行影响，代价是 E3-c 盖住真红；这一轮的改法用「一道里每一条」换掉它，换进了汇总行这个新口子。被留下的形态（按行读）没有这个口子，所以这一处是采的改法自己打中了自己，不是同样打中所有替代方案。
推翻它：分诊定义或这一道的输出里有一句把汇总行排除在外（例：「不算共用库打的『N 格里 K 格红』那一行与它下面的 → 行」）。**零轮**（这是本轮读出的新毛病，没被攻过）。

### 3. 253 那一句：写得比事实宽（低）

分诊定义里那一句把整道写成：「的 253 是 Verus 超时」（`.claude/agents/gate-triage.md:29`），✗ 行写超过 N 秒没出结局、按超时判红。Verus 一侧成立（探针退 253 落「超时红」，第 4 点的表）。E165 一侧不成立：它的包装结局表里 253 还列着：「253: "超过限时"」（`research/scripts/e165-pin-red-examples.py:82`），探针退 253 打的仍是「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 253（超过限时，上限 1G）」（`rerun-output.txt` 最后一段）。两条规则对同一道里的 E165 253 各说一句：崩溃验证员记没跑成，分诊按这一句记超时红。今天走不到：E165 一侧没有谁给它设限时（`e165-pin-red-examples.py` 里 `TIME_LIMIT` 只出现在 → 的出路句里，第 408 行；包装把限时变量传进命令前清掉）。推翻它：门禁里真出现 E165 一侧退 253。

### 4. Verus 一侧外部杀进程：E3-d 只改了一半（同样打中）

判决收 E3-d 的写法是：「E165 一侧外部杀进程的出路改成照包装文件头」（`research/prompts/defs-m3-formal-gate-r2-main-verification.md:22`）。辩方 E3-d 原话（转述）是两份判法的退 137 都不在 250–254 里，Verus 写「没出判定」、E165 写「修实现回归」。3-12 把 E165 一侧改成「装置被信号停了」并照包装文件头给出路：「raise Red(f"装置被信号停了」（`research/scripts/e165-pin-red-examples.py:412`）。Verus 一侧没动：

```
退   0 | 类 没出判定   | 出路 格自己的出路 | 没带 | Verus 没出判定（退 0，编不过或起不来）：…
退 137 | 类 没出判定   | 出路 格自己的出路 | 没带 | Verus 没出判定（退 137，编不过或起不来）：…
退 253 | 类 超时红    | 出路 格自己的出路 | 没带 | 超过 600 秒没出结局，按红记（C5：超时不当通过）
```

（`rerun-output.txt` 的 `l3-verus-exit-codes` 段，每行截到前 60 字。）判法里那句：「Verus 没出判定（退 {outcome['exit']}，编不过或起不来）」（`research/scripts/verus-proof-check.py:415`），出路是格自己的（「照 Verus 报错的位置修证明或实现」一类）。包装文件头明写命令本身被杀时退 137、外面 kill -9 同样退 137（`research/scripts/run-with-memory-cap.sh:23`，文件头「退出码」一段）。所以别的会话或 OOM 之外的原因杀掉一次 Verus，崩溃验证员读到的是红在证明或实现上的 ✗ 加「修证明」的 →，与改前 E165 那一条同一种指错方向。同族先例：E165 一侧这件事判决已采，说明这个形状算数；Verus 一侧没被它罩住，是同样打中别的臂。
低：触发要外面杀进程；后果是误导与推迟，不是放过（仍退 1）。改法（推的，零轮）：Verus 一侧退 128 + 信号号（或负数）的，照 E165 一侧那样单列。推翻它：Verus 一侧退 137 时 ✗ 行已指向包装文件头。

### 5. `gate.sh` 汇总里这一道退 1 的说法：对得上

阶段退 1 在 `gate.sh` 里走「失败」：`*) bad "${STAGES[$i]}"`（`.claude/singlefs-ai-sop/scripts/gate.sh:778`）加一句：「红色阶段的细节与出路在上方对应段落里」（`.claude/singlefs-ai-sop/scripts/gate.sh:781`）。汇总里只有阶段名，没有「没跑成」「并发」「环境」任何一个类名，所以与分诊定义的产出表「一张表：阶段 / 绿·红·77」（`.claude/agents/gate-triage.md:38`）不冲突：一道全是没跑成的，在表里记「红」、归属写「环境（上限）」或「并发」，细节抄上方段落。退 77 是另一回事（本次未跑，不是没跑成），两种说法没有混用。

## 四、L4：已定格、记账、数

### 1. J2 的记账：并进的那一行没有写进去

判决：「J2 不归这一道」（`research/prompts/defs-m3-formal-gate-r2-main-verification.md:20`）并进共用库盲区那一批，指到 `records/2026-09-16-subagent拆分提案.md` 第四十三节第 1 行。那一行现在写的是：「认不出 `time -p`、`env -S`」（`records/2026-09-16-subagent拆分提案.md:998`）一类，J2 的四种写法（`env -C` 与 `--chdir`、脚本目录放变量、`script -qc`、`xargs`）一种都不在：

```
script -qc 命中: 0
env -C 命中: 0
--- 四十三节第 1 行里 xargs/chdir 命中: 0
```

（对 `records/2026-09-16-subagent拆分提案.md` 全文数的；第三项只数第 998 行。）这四种写法只记在 `lib_heavy_tests.py` 文件头：「罩不住的：改名拷贝（两份没有签名行）」（`.claude/hooks/lib_heavy_tests.py:74`），那是代码注释，不是欠账表。判决的「并进」是打算，今天还没落。攻方 r2 另量到 `script -qc` 连 checker 档 `cargo test` 也放（转述其 e1b 输出），所以那一行写的时候该写成重型闸共同的盲区，不只广谱变异。推翻它：该行补上这四种写法。

### 2. G9、G10 本体

L1、L2 归攻方，本腿没复核它们的代码行为。能核的只有 3-12 报告（`/tmp` 下，不当依据）里的文字与仓里一致这一件：`tool_versions` 里确有安装目录顶层摘要：「install_top_level_sha256」（`research/scripts/verus-proof-check.py:813`）。

### 3. 第 1 步停下的频率、QEMU 两处：没有落点

判决：「E4-b 记账」（`research/prompts/defs-m3-formal-gate-r2-main-verification.md:23`）写成「等几次提交的实况再判」（同一行）；第一轮两处同形写成「QEMU 两处记账」（`research/prompts/defs-m3-formal-gate-r2-main-verification.md:19`）。仓里能找到的记载只有进度记录里一句话：「QEMU 两处同形、第 1 步停下的频率记账」（`records/2026-10-01-里程碑三形式化接入.md:252`），`grep -rn 'QEMU 两处\|停下的频率' records .claude/kb` 只命中这一行；`.claude/kb/checks-owed.md` 里没有对应行（该文件里 QEMU 的几行都是别的题）。没有落点就没有人去数「几次提交」：等的那个数没有计数方法，也没有谁在哪一步记它。

第三个数据点（辩方 r2 量过 0 → 2 个未跟踪）。今天：

```
$ git ls-files --others --exclude-standard -- crates research/results | wc -l
0
$ git diff --name-only -- <这一道登记的路径与阶段脚本>
.claude/gate.d/checker-tier-formal-verification.sh
research/scripts/e165-pin-red-examples.py
research/scripts/verus-proof-check.py
```

未跟踪 0 个，但这一道的 `git diff --quiet` 今天会退 1：3-12 自己改的三份文件还没暂存。照第 1 步字面今天派崩溃验证员，这一道会停在第 1 步；停的原因不是别的会话，是「改动没暂存」，与第 1 步的本意一致（先暂存再派）。频率仍不能下结论：三次观测（0、2、0 个未跟踪）原因各不同。

### 4. E3-e 排队等待：判决一个字没记（同样打中判决的遗漏）

辩方 r2 列了「E3-e 排队等待推的」（`research/prompts/defs-m3-formal-gate-r2-main-verification.md:22`，在辩方一栏），判决一栏既没采也没记账。现查它推的那条链：包装的默认等待：「DEFAULT_WAIT_SECONDS=3600」（`research/scripts/run-with-memory-cap.sh:124`），文件头：「等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下」（`research/scripts/run-with-memory-cap.sh:54`）；判法一侧并行度：「jobs = options["jobs"] or min(len(edits), max(1, options["threads"] // 4))」（`research/scripts/verus-proof-check.py:607`），线程数默认取：「"threads": int(os.environ.get("SINGLEFS_THREAD_CAP")」（`research/scripts/verus-proof-check.py:835`）。必红世界表 94 处（`grep -vc '^#' crates/singlefs-replay-prefix/verus-mutations.tsv` 输出 94）：

```
SINGLEFS_THREAD_CAP='' 线程 32 jobs 8 轮 12
SINGLEFS_THREAD_CAP='4' 线程 4 jobs 1 轮 94
```

slice 一直被别的会话占满时，每一轮各等满 3600 秒再退 252：崩溃验证员命令里不带 `capped.sh`，取 32 线程、12 轮，最坏约 12 小时（再加空真 2 轮、证明 1 轮）；若主 agent 给的是线程上限 4，94 轮。这是推的，没量：要占满 slice 一小时以上。定义（「长活可以等」）不禁，但「没跑成」在最坏时要很久才出现，值得记一笔，改法不在本轮范围。推翻它：在占满的 slice 上量到必红世界一格的挂钟短于这个数。

### 5. 3-12 自己列了、判决没记的旧字面

3-12 报告（`/tmp` 下）「没做什么」点了几处没改：登记表第 31 行注释现在少说了顶层摘要与 `VERUS_Z3_PATH`，原句还在：「Verus 的 version、commit 与可执行文件的 sha256 经 environment=」（`.claude/gate.d/stage-inputs.tsv:31`）；阶段文件里红例重放那一格的出路还写：「按回归修实现」（`.claude/gate.d/checker-tier-formal-verification.sh:83`），它与同一格 ✗ 行自己的 → 并排出现在汇总里（装置被信号停了时 ✗ 行的 → 说不修实现，汇总的 → 里没有这个例外，只写「没跑成：内存包装」那一种）；E165 一侧 252 两种成因没分；分诊定义里「装置被信号停了」归哪一类没写。判决写于 3-12 之前，没有这几笔；records 里 3-12 那一条（第 253 行）也没列。都是文字残留，不影响判红，但没有落点就会丢。

### 6. 第二轮报出的数，今天能不能核

| 数 | 今天怎么核 | 结果 |
|---|---|---|
| 54 个 hunk | `grep -c '^@@' research/prompts/_defs-m3-formal-gate-r2-diff.md` | 54，对上 |
| 两份报告 sha256 前缀 c7287385、66a9ad84 | `sha256sum` 两份 | 对上 |
| 复跑六份输出与入库逐字节相同（e1、e1b、e1-G9、e1b-G9、e2-lkmm-hook、e3-mixed） | `cmp` 主 agent 草稿目录里那六份与 `defs-m3-formal-gate-r2-opus-model/` 里入库的 | 六份都相同；只核了「草稿里那份与入库那份」，没重跑攻方探针 |
| e2-reuse 只差输入文件数（566 → 576） | `diff` 同上两份 | 只差第 1–3 次与 worktree 四行，文件数 576 对 566 对上；指纹值也不同（文件多了） |
| e2-verus-toy 只差 Verus JSON 的键序 | 没核 | 核不动（输出行里 JSON 被截断，没逐键比） |
| 「核了 106 处引文，1 处对不上」 | 今天用同一条命令重跑 | 106 处里 28 处对不上：27 处在 3-12 改过的四份文件（`lib_heavy_tests.py` 11、`verus-proof-check.py` 13、`e165-pin-red-examples.py` 2、`heavy-test-guard.sh` 1），另 1 处是判决说的 `gate.sh:666` 对 668。今天不能复核当时的 106 / 1，但差额说得清：落在 3-12 改过的文件里，是行号漂移或那一句被改写，不是引文当时失实（没有逐处分开两种） |
| 「22 份 18 份 OK」 | `sha256sum -c` 第二轮快照 | 今天 8 份 OK、14 份 FAILED（3-12 与别的会话改了），核不动 |
| 辩方 r2「15 条 ✗：8 没跑成、7 真红」 | 第二节三种格的探针 | 对上：混合格 15 条问题 8 没跑成 7 真红，改后另多一行按红记，所以 ✗ 共 16 行 |

### 7. 已定格撑不撑得住

- 第一轮 D1–D5、第二轮 E4：本轮的改动没有推翻它们。`main-agent.md` 第 66 行已补复用上限：「判绿那一次过了复用上限」（`.claude/main-agent.md:66`），与代码对得上：「hours_text = os.environ.get("SINGLEFS_REUSE_HOURS", "") or "24"」（`research/scripts/admission.py:1157`）。
- E1、E2：本腿不判，见攻方报告。
- E3：第二、三节。判法一侧与崩溃验证员一侧站得住；分诊一侧与 Verus 137 是新的零轮毛病。

## 五、同类先例

| 问题 | 先例或「找不到」 |
|---|---|
| 一道里的汇总 ✗ 行与「每一条 ✗」类字面冲突 | 找到同族：`lib/stage-cells.sh` 的汇总行每个经它的阶段都有（`.claude/gate.d/checker-tier-crates-mutation-replay.sh:274`、`.claude/gate.d/checker-tier-lkmm.sh:98`、`.claude/gate.d/checker-tier-qemu-device-streams.sh:420` 同样 source 它），分诊定义里别的「每一条」句式没有；找不到别处先例处理过「汇总行不算」 |
| 一格里没跑成与真红并存，判法打一行「按红记」 | 找不到：判决写的那几道的先例一次只起一条命令（辩方 r2 已写同一结论） |
| 外部杀进程（137）的出路指向包装文件头 | 找到：E165 一侧（本轮 3-12，`research/scripts/e165-pin-red-examples.py:412`）；Verus 一侧找不到 |
| 包装 25x 判红、不判 77 | 找到：研究构建与单测的「没判出结果」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:190`）、模型差分与场景的同一句（`.claude/gate.d/harness-model-differential-and-scenarios.sh:200`） |

## 六、没做什么

- 没跑重型测试、没起 Verus、没编装置、没跑真阶段（`checker-tier-formal-verification.sh` 一次也没起）、没跑 `--selftest`、没跑 `gate.sh`；探针里的包装与阶段都是桩与合成阶段。
- 没派分诊员或崩溃验证员实际走一遍：第三节「分诊字面走不通」是按定义字面与阶段输出对读推的，不是观测；同样「Verus 退 137 的 → 指错方向」是读判法代码加桩探针，没造真 OOM。
- 没复核 L1、L2（G9 的只读名单与误拒、G10 的哈希范围与会变的东西），那两格归攻方；没重跑攻方第二轮的探针；没读禁读清单里的 `defs-m3-formal-gate-r3-opus-output.md` 与 `defs-m3-formal-gate-r3-opus-model/`。
- 没改 `.claude/agents/`、`records/`、`.claude/kb/` 里任何一行：第三节分诊字面的收严、§43 第 1 行补写、QEMU 两处与 E3-e 的落点、两处旧字面都只是指出，补不补、记在哪由主 agent 定（都是被攻过零轮的形态，写进交用户表时标「零轮」）。
- 没量：E3-e 的最坏挂钟（要占满 slice 一小时以上）、第 1 步停下的频率（三次观测各不同因）、Verus 一侧真被外面 kill 时崩溃验证员的记法。
- 探针没经 `run-with-memory-cap.sh` 跑（Python 桩与合成阶段，没有编出来的代码）；经 `nice -n 19 bash research/scripts/capped.sh 4` 起。起跑前 `ps` 里没有 `qemu-system`、`vm-bench`、`e152`、`fio`（数出来 0 个）。
- 草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r3-sonnet/` 里只有桩包装、假 Verus 目录、计数文件与输出文本，没有编译目录、工作树与仓副本（合成阶段的小拷贝已删，`rerun.sh` 每次重建）；探针与输出都在模型目录，所以不另入库。

## 七、交回前核对

```
$ sha256sum -c research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt | grep -c ': OK$'
9
$ wc -l < research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt
9
$ (cd research/prompts/defs-m3-formal-gate-r3-sonnet-model && sha256sum -c SHA256SUMS | grep -c ': OK$')
6
```

模型目录 `research/prompts/defs-m3-formal-gate-r3-sonnet-model/`：`l3-cell-modes.py`、`l3-synthetic-stage.sh`、`l3-verus-exit-codes.py`、`l3-e165-exit-codes.py`（从第二轮辩方模型拷来，未改）、`rerun.sh`、`rerun-output.txt`、`SHA256SUMS`。

```
$ python3 research/scripts/cite-check.py research/prompts/defs-m3-formal-gate-r3-sonnet-output.md --root . | tail -1
  ✓ 核了 32 处引文，对上 32 处，没判 0 处（逐处列在上面）
```
