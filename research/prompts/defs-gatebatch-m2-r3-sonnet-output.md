# 门禁批第三轮·正推腿（L2）报告

分工：这条腿只判 L2（分片判法与条款一致），不判 L1、L3。立场：正推——从
`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」「测试与崩溃检测优先多线程」两节
与 `.claude/agents/crash-verifier.md` 出发，推「应该是什么」，再逐格与今天的代码比对；一样写一样，
不一样把两边整行并排抄出来。

被判文件：`research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt` 里的 14 个文件；用
`sha256sum <14 个路径>` 与该文件核对，逐字节相同（下面「附：开工核对」一节贴命令与输出）。

## 一、shard=across-machines 登记字段

**正推（从条款到代码，逐步核落点）**：条款没有专门讲「跨机分片登记字段该长什么样」——两节规则都在讲
单机多线程；分片是这一批 admission.py 与 `.claude/gate.d/stage-inputs.tsv` 自己长出来的一层，条款管不到
它的字面，只管它要不要经过「重型测试」「多线程判红」这两道闸。逐步核：

| 步骤 | 落在哪一行 |
|---|---|
| 常量：分片只认 `across-machines` 这一个值 | `research/scripts/admission.py:191`：`CRASH_CASE_SHARD_ACROSS_MACHINES = "across-machines"` |
| 登记表第三列解析 `shard=` 词 | `research/scripts/admission.py:1168-1173`（`parse_crash_case` 里 `if kind == "shard":` 分支，值不是 `across-machines` 或多于一条都 `raise RegistrationError`） |
| `CrashCase` 对象携带这一位 | `research/scripts/admission.py:1138`（`__init__` 参数 `is_shardable_across_machines`）、`1140`（`self.is_shardable_across_machines = is_shardable_across_machines`） |
| 子命令查这一位 | `research/scripts/admission.py:2247-2261`（`command_crash_case_shardable`：登记了退 0、没登记退 1） |
| 登记表今天实际打了这一行的用例 | `.claude/gate.d/stage-inputs.tsv` 第 36、37、40 行：`crash-case:layer0-first-stream`、`crash-case:layer0-second-stream`、`crash-case:layer0-parallel-line-one-stream` 都带 `shard=across-machines`（命令见下） |

**与背景材料 L2 问题格字面的一处不一致（整行并排抄）**：

- 背景材料正文（`research/prompts/_defs-gatebatch-m2-r3-background.md:14`）写：
  「双机驱动按内容进两条层 0 流的指纹（G2 第 4 条）」
- `.claude/gate.d/stage-inputs.tsv` 今天实际登记 `shard=across-machines` 的是**三条**（36、37、40 行），
  不是两条：多出的一条是 `crash-case:layer0-parallel-line-one-stream`（第 40 行）。

这不是代码错了，是「两条层 0 流」这个说法（沿用 G2 报告标题「第 4 条：双机驱动进两条层 0 流用例的指纹」，
`research/prompts/defs-gatebatch-m2-g2-report.md:60` 的原话）在 G2 交付之后被别的会话（实分片实现员）
扩到了第三条用例，字面没跟着改。推翻条件：谁把第 40 行的 `shard=across-machines` 摘掉，或指出它是这一轮
之前就有、G2 报告写「两条」时已经算上它的，这一条不一致就不成立——我核过：`git log -p -- .claude/gate.d/stage-inputs.tsv`
第 40 行的 `shard=across-machines` 在这一轮开工快照里已经在，不是这一轮新加的，但仍然是「三条」而不是「两条」。

## 二、`crash-case-shardable` 子命令进判法摘要

正推链路：

| 步骤 | 落在哪一行 |
|---|---|
| 分派表把子命令指到函数 | `research/scripts/admission.py:3921`：`"crash-case-shardable": command_crash_case_shardable,`（分派表 `COMMANDS`） |
| 判法摘要要连这个子命令一起进闭包 | `research/scripts/admission.py:215-216`：`CRASH_CASE_JUDGING_SUBCOMMANDS = ("crash-case-command", "crash-case-judge", "crash-case-record", "crash-case-marker-check", "crash-case-marker-path", "crash-case-shardable")` |
| 求闭包时把它算进去 | `research/scripts/admission.py:1901-1902`：`subcommands = tuple(name for name in CRASH_CASE_JUDGING_SUBCOMMANDS if not (name == "crash-case-shardable" and break_is_set("shardable-outside-judging-digest")))` |
| 弄坏开关能把它摘掉、证红 | `research/scripts/admission.py:3478-3481`（`with BreakSwitch("shardable-outside-judging-digest"):` 那一格），自证跑法：见下 |

复跑（自己跑过，原样）：

```
$ SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest | grep -i shardable
  ✓ 判法摘要：单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了 ⇒ 摘要变
  ✓ 弄坏开关 shardable-outside-judging-digest 下「crash-case-shardable 把没登记答成登记了」那一格红（摘要没变）
  ✓ admission.py 自证通过：232 格都对（…）

$ ADMISSION_BREAK=shardable-outside-judging-digest SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest | grep -i "shardable\|自证"
  ✓ 判法摘要：自证：run_selftest 里加一行注释 ⇒ 摘要不变
  ✗ 判法摘要：单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了 ⇒ 摘要变：摘要没变
  ✓ 弄坏开关 shardable-outside-judging-digest 下「crash-case-shardable 把没登记答成登记了」那一格红（摘要没变）
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

一致：`crash-case-shardable` 真的进了判法摘要的闭包，弄坏开关能证红，与背景材料 G2 报告第 3 条
（`research/prompts/defs-gatebatch-m2-g2-report.md:47`「改法：admission.py 第 208–209 行把它加进元组」）
说的是同一件事，只是今天的行号是 215-216（G1 在它前面插入了行，行号整体往后挪了 7 行，属正常漂移，不算不一致）。

## 三、merge 那一趟的线程行

正推链路：单机的门禁多线程判法在 `judge_worker_threads`；分片跑完 merge 时日志里的 `LAYER0_PARALLEL_FINISHED`
带 `shards=` 字段，走另一支：

| 步骤 | 落在哪一行 |
|---|---|
| 单机判法入口，先看是不是分片 merge | `research/scripts/admission.py:1592-1615`（`judge_worker_threads`），第 1615 行 `if "shards" in fields and not break_is_set("threads-ignore-shards"): return judge_threads_of_each_shard(prefix, finished[0], fields)` |
| 逐片各自判（核数、显式设没设都取那一片自己账本里记的，不取 merge 这台的） | `research/scripts/admission.py:1636-1665`（`judge_threads_of_each_shard`），docstring 第 1637-1639 行写明「机器核数与「显式设成 1」都取那一片账本里记的，不取 merge 这台的」 |
| 逐片字段名表 | `research/scripts/admission.py:1629-1630`：`SHARD_FIELD_NAMES = ("worker_threads", "configured_worker_threads", "worker_threads_sources", "available_parallelism", "resumed_slices", "freshly_run_slices")` |
| 驱动脚本真正跑两片、等两片都退出、核对分片日志行 | `research/scripts/layer0-shard-run.sh:218-231`（起两片，本机 `SINGLEFS_LAYER0_SHARD=0/2`、第二台 ssh 起 `SINGLEFS_LAYER0_SHARD=1/2`）；`232-233`（`wait "$local_shard_process"`、`wait "$peer_shard_process"`，两片都等到退出才往下走）；`236-240`（退出码与 `LAYER0_SHARD mode=run shard=…/2` 行都要在，缺一样进 `shard_problems`） |
| merge 那一趟本身 | `research/scripts/layer0-shard-run.sh:276-289`（`SINGLEFS_LAYER0_SHARD=merge/2` 起 cargo，第 285 行核 `LAYER0_SHARD mode=merge shards=2 ` 那一行在不在，退非 0 或缺行都判红），merge 那一趟的日志整份交给 54 号走 `crash-case-judge`（`judge_crash_case_log` → `judge_worker_threads` → `judge_threads_of_each_shard`） |

一致：条款「测试与崩溃检测优先多线程」里「机器多于 1 核却只用了 1 个线程，判红」这一句在单机分支
（`judge_worker_threads` 第 1616-1622 行 `judged_on_one_thread` 那一段）原样落地；分片分支
（`judge_threads_of_each_shard`）把同一条判法逐片各做一遍，字面上是条款没写过的（下面第五节详细说）。

## 四、双机驱动按内容进层 0 流用例的指纹

正推链路：

| 步骤 | 落在哪一行 |
|---|---|
| 两份要按内容进指纹的文件 | `research/scripts/admission.py:194`：`SHARD_DRIVER_FILES = ("research/scripts/layer0-shard-run.sh", "research/scripts/layer0-shard-configuration-check.sh")` |
| 求指纹时接进去 | `research/scripts/admission.py:1554`：`manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + shard_driver_lines(root, case) + [registration])` |
| 按内容取这两份文件、没登记的用例交空表、文件不在时记「找不到」 | `research/scripts/admission.py:1558-1571`（`shard_driver_lines`） |
| `stage-inputs.tsv` 里 54 号那一行也把这两份列进它自己的路径 | `.claude/gate.d/stage-inputs.tsv:28`（路径列表尾部有 `research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh`） |

自证（真跑，原样，取自上面已经跑过的那一趟 232 格自证输出）：`admission.py --selftest` 的
`shard-driver-outside-manifest` 那一格（`research/scripts/admission.py:3845-3854`，`run_layer0_stage_shard_cells`
开头）就是拿弄坏开关摘掉这两份文件、看指纹变不变；232 格全过已经把它算在内，没有单独跑弄坏开关是因为
这一格已经在上面「232 格都对」那句列出的弄坏开关清单里（`shard-driver-outside-manifest`）。

## 五、缺哪一步——条款字面没跟上分片这一层

**这是这一格的核心发现**：`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」一节
（`.claude/rules/implementation-workflow.md:82-90`）与 `.claude/agents/crash-verifier.md` 全文都**没有一个字提到「分片」「双机」「第二台」**：

```
$ grep -n "分片\|shard\|双机\|第二台" .claude/agents/crash-verifier.md
（零命中）
```

这不是代码缺了一步，是**条款没跟上代码**——分片是 G2 这一批才加的能力，两份条款文件都还停在
「单机、单进程、`std::thread::scope` 内部并行」那个世界模型里。具体来说：

1. 「测试与崩溃检测优先多线程」一节的判红条件写的是「机器多于 1 核却只用了 1 个线程」（单数「机器」，
   `.claude/rules/implementation-workflow.md:89`），而分片场景下这句话的真正含义是「**两台**机器各自
   多于 1 核却各自只用了 1 个线程」——代码（`judge_threads_of_each_shard`）已经把判法逐片扩展好了，
   但规则正文的字面没有跟着提一句「分片时逐片各判」。规则没说，不算冲突，但读者只看这条规则会以为
   「多线程」只是进程内部的事，看不出还有一层跨机分片的判法。
2. `crash-verifier.md`「输入（主 agent 必须给）」一节（`.claude/agents/crash-verifier.md:17-24`）列的五项输入（改动范围、提交还是
   用户要求、三道的内存上限、54 号在哪棵 worktree 跑、报告路径）里**没有一项覆盖「第二台机器」**：
   分片开不开是 `research/scripts/layer0-shard-configuration-check.sh` 在跑的那一刻自动判的
   （本地配置文件 `layer0-shard.env` 在不在、第二台 ssh 连不连得上），不需要主 agent 显式告诉
   crash-verifier「这次要不要分片」——这一点是自动的，不算缺；**但内存上限这一项缺了跨机的一半**：
   `.claude/agents/crash-verifier.md:21` 只要求 54、57 号的内存上限「写明是量过的...还是推的」，而 54 号 `--full`
   分片开着时会经 `run_crash_case_in_two_shards` 把一片交给 `research/scripts/layer0-shard-run.sh`，
   驱动脚本里第二台那一片是整条 `ssh ... cargo test ...`（`research/scripts/layer0-shard-run.sh:134`
   的 `run_on_peer`，`226` 行实际起用的那一条），我核过驱动脚本与配置判法全文，一次 `memory\|ulimit\|cap\b\|prlimit`
   都没出现：
   ```
   $ grep -n "memory\|ulimit\|cap\b\|prlimit" research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh
   （零命中）
   ```
   共用约束「不做」一节的内存包装规则（`bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/54-layer0-replay.sh --full`）
   包住的是**本机**这一次调用产生的子进程树；第二台是 ssh 起的一个全新进程，不在本机这棵进程树里，
   `run-with-memory-cap.sh` 的上限**够不着它**。crash-verifier.md 给的「54 号内存上限」这个输入因此只
   管到本机那一片（加 merge），第二台那一片的内存没有任何上限、也没人要求主 agent 给一个「第二台内存
   上限」。这是条款没跟上代码的一处实缺：不是判法错了，是「跑重型测试要包内存」这条规矩在双机分片下
   只包了一半，两份被判条款都没提这件事。

**推翻条件**：这一条会被下面任何一样推翻——① `layer0-shard.env.example` 或 `PEER_*` 配置键里其实有
内存相关的键（我读过 `research/scripts/layer0-shard-configuration-check.sh:24-25` 的
`LAYER0_SHARD_CONFIGURATION_KEYS`，只有 `PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY
QUIESCE_STOP_COMMAND QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_START_COMMAND QUIESCE_STARTED_CHECK_COMMAND`
七个键，没有内存键，我核过原文，没有）；② 第二台上还有另一层不在这两份脚本里的资源限制（例如第二台
自己的 systemd 单元、cgroup 配额）——这一点我没有第二台机器可核，写「复核不了：没有第二台环境」。

## 六、重算 K4 那一行「判法闭包 365 / 3194 行（11%）」「这一批 21 个 hunk 0 个落在闭包里」

第二轮判决 K4 那一行的原文（`research/prompts/defs-gatebatch-m2-r2-main-verification.md`，「一、各格判定」
一节，我在文件里现查的行号是第 23 行）：

> K4 整份准入模块进指纹 | **代价是真的，今天为零，范围太宽**：common-dir 里崩溃枚举用例的全绿标记今天 0 格；判法闭包 365 / 3194 行（11%），这一批 21 个 hunk 0 个落在闭包里、却每一个都让四条全重跑；与用户原话「改了只跑改了的部分」相反。攻方 D2（判法摘要）在模型上对 4 种非判法改动不变、对 4 种判法改动都变 | 攻方 K4

这两个数字**不是从 admission.py 自己算出来的**，是 r2 云端攻方（Opus）自己写的探针
`research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py` 算出来的（我读了它的正文，
它自己的说明第 1-7 行写明是「我的模型，不在被判代码里」）。这份探针脚本我原样没有改一个字，
分别在①今天的 `research/scripts/admission.py`、②今天的文件用 `patch -R` 依次退掉 G2、G1 两份
admission.py diff 重建出的「r2 判决那一刻的 admission.py」上重跑了一遍（复跑脚本落在模型目录
`research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh`，用法 `bash rerun.sh <仓根>`）。

### 6.1 重建基线核对

`patch -R -p1 --fuzz=0` 把 G2、G1 两份 admission.py 的 diff（`research/prompts/defs-gatebatch-m2-g2-changes.diff`、
`research/prompts/defs-gatebatch-m2-r2-fixes.diff`，sha256 与背景材料第 9 行给的 `d2a94fafe7b200bf…`、
`a4c1ffecf1f7cf1a…` 一致，见附「开工核对」）依次退掉，两次都干净应用（只有因为别的文件早先改动带来的
行号偏移 offset，没有冲突）。重建出的文件**行数是 3202 行，不是 r2 判决说的 3194 行，差 8 行**——
但探针脚本自带的对照句证实这份重建文件与 r2 攻方当时实际读的文件在**判法闭包这件事上逐字节相同**：

```
$ K1_ADMISSION=<重建文件> python3 research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py
MEASURE admission.py 全文 3202 行；闭包 37 个定义、365 行（11%）；自证从第 1951 行起到文件尾共 1252 行
MEASURE 改法之前的备份 /tmp/claude-1000/gate-batch-m2-r1-fixes/backup/research/scripts/admission.py：判法摘要 0f18d61a0a890957…，今天 0f18d61a0a890957…，相同；闭包名字差：多 [] 少 []
MEASURE 附录二 admission.py 的 21 个 hunk：落在判法闭包里 1 个，闭包外 20 个
```

闭包「37 个定义、365 行（11%）」与 r2 判决的数字**完全对上**；`/tmp/claude-1000/gate-batch-m2-r1-fixes/backup/...`
这份 r2 时期留下的备份文件今天仍在磁盘上，我读过（`ls -la` 见附），探针拿它与我重建出的文件比对判法摘要，
结果「相同」——说明我用 `patch -R` 重建出的文件与 r2 判决时真正被判的文件，在「判法摘要覆盖到的那部分」
逐字节一致。3202 与 3194 的 8 行差异**没能查清原因**（两种可能：r2 攻方探针当时读到的 admission.py 与
我用两份归档 diff 反推出来的不是同一份逐字节文件，比如中间还套了一次没有归档进这两份 diff 的小改；或者
探针脚本用的行数统计口径与我这里不同），标「未核实」，不影响下面的结论——因为判法闭包内容（37 个定义、
365 行、判法摘要哈希）三样都对上了，8 行的落差落在闭包**之外**的部分。

**但「这一批 21 个 hunk 0 个落在闭包里」这一句，在重建出的基线上重算，是「1 个」，不是「0 个」**：

```
HUNK 新文件第 152–162 行：闭包内（ATTRIBUTE_START、COUNT_LINE_PREFIX_FORM、CRASH_CASE_CONDITION_FORM、CRASH_CASE_MARKER_PREFIX、CRASH_CASE_TEST_FORM、IGNORE_ATTRIBUTE_FORM、LAYER0_PARALLEL_FINISHED_PREFIX、MARKER_DIFFERENCES_LISTED_AT_MOST、PASSED_ONE_TEST_FORM）
```

这与「判法摘要相同」（上面那一行「相同」）**不矛盾**：探针脚本判一个 hunk 「落在闭包里」用的是**行号重叠**
（把 diff 里 `-` 行与 `+` 行按新文件坐标记一遍，看落点跟哪个模块级定义的 `lineno..end_lineno` 有交集），
不是「这个 hunk 真的改了闭包函数的源码」。判法摘要哈希前后相同，说明这 21 个 hunk 里**没有一个真的改动
了闭包函数的源码**；但探针给「-」行记行号的写法（`research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py`
里 `if line.startswith("-"): hunks[-1].add(new_line); continue`）用的是**当前的 `new_line` 指针**——
对纯删除行，这个指针指向的是「删除点在新文件里大致的位置」，不是删除内容本身在新文件里还存在的位置。
这一处 hunk（对应 r2 diff 里改文件头一段说明文字的那处改动）恰好落在 `CRASH_CASE_MARKER_PREFIX` 等几个
常量定义紧邻的行号上，触发了「行号重叠」但没有改动闭包内容。**这是探针脚本自己一个已知的不精确处，
不是 D2 或分片改法的缺陷**：r2 判决引这个数字时同样是拿这份探针的输出转述的，那一处「0 个」本身在
r2 判决时也不是「逐个人工核对过」，是这份探针原样吐出来的——重算一遍才发现它自己的口径有这个误差。

### 6.2 用同一份探针原样跑在今天的 admission.py 上——两个数都对不上，差多少、为什么

```
$ K1_ADMISSION=research/scripts/admission.py python3 research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py
MEASURE admission.py 全文 3939 行；闭包 42 个定义、427 行（10%）；自证从第 2289 行起到文件尾共 1651 行
MEASURE 附录二 admission.py 的 21 个 hunk：落在判法闭包里 3 个，闭包外 18 个
```

**差多少**：

| 量 | r2 判决 | 今天（同一份探针原样跑） | 差 |
|---|---|---|---|
| 闭包定义个数 | 37 | 42 | +5 |
| 闭包行数 | 365 | 427 | +62 |
| 全文行数 | 3194（重建基线实测 3202） | 3939 | +745（对 3202 说） |
| 百分比 | 11% | 10% | −1 个百分点（分子分母都涨，分母涨得更快） |
| 21 个 hunk 落在闭包里 | 0 | 3 | +3 |

**为什么**：

1. **百分比与定义数的变化是真实的、有意义的**：admission.py 从 r2 判决时的 3202 行长到今天的
   3939 行（+737，与 G1 39 个 hunk、G2 35 个 hunk 共 74 个 hunk 相符，多数是 D2 本身的实现代码、
   P1/P4/P5/P9/P10 等改法与 G2 的分片四条），而闭包只多了 62 行、5 个定义——多出来的定义正是分片
   相关的 `CRASH_CASE_SHARD_ACROSS_MACHINES`、`SHARD_FIELD_NAMES`、`judge_threads_of_each_shard`、
   `started_worker_threads_text`、`configured_worker_threads_text`（探针自己打的「闭包名字差」那一行
   列全了）。这与第三节「merge 那一趟的线程行」的发现是同一件事：判法闭包确实按需要多收了分片判法，
   涨幅比整份文件小得多，方向与 D2 的目标一致。
2. **「21 个 hunk 落在闭包里 3 个」这个数不能直接拿来跟 r2 的「0 个」比**：这 21 个 hunk 来自
   `research/prompts/_defs-gatebatch-m2-r2-diff.md`（r2 判的、已经冻结的证据），它记的行号是**那时候**
   新文件的坐标；今天的文件比那时候多了 745 行（这次探针跑用的是 `research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py`
   里硬编码的 `DIFF = .../_defs-gatebatch-m2-r2-diff.md`，探针脚本按这份冻结 diff 自带的旧行号原样定位，
   套到今天变长的文件上）。用重建出的 r2 基线核过：那份基线上「1 个落在闭包里」（6.1 节），今天的文件上
   变成「3 个」——多出的 2 个纯粹是行号漂移巧合撞上了别的常量定义（我核过其中一处，第 152–162 行今天
   撞上的是 `REGISTRATION_TABLE`/`REPLAY_SCRIPT`/`RESULTS_DIRECTORY` 那几个常量，而这个 hunk 在 r2 diff
   里改的其实是文件头一段说明文字，跟这几个常量毫无关系）。这不是「今天的判法漏看了什么」，是
   「拿一份写死绝对行号的旧探针，套在一份后来又长了 745 行的文件上」这件事本身不成立——**推翻条件**：
   谁要是能证明这 3 个「落在闭包里」的 hunk 真的改动了闭包函数的源码（不只是行号偶然重叠），这一条
   解释就不成立；我核过其中输出打印的那一个（152–162 行），确认是常量重排导致的行号巧合，不是真改动
   （见 6.1 节判法摘要哈希「相同」那条独立证据）。

### 6.3 今天真正出货的 D2 闭包算法（不是探针，是 `crash_case_judging_digest_text` 本身）

上面两节用的都是 r2 攻方**自己写的、没有进被判代码的**探针（`probe_k4_cost.py`），它的入口列表
（`ENTRY_POINTS`，13 个，硬编码在探针脚本里）是 r2 时手选的一份**旧**清单，跟今天**真正出货**的
`crash_case_judging_digest_text`（`research/scripts/admission.py:1890-1926`，从 `CRASH_CASE_JUDGING_ENTRIES`
12 个入口 + 分派表 `CRASH_CASE_JUDGING_SUBCOMMANDS` 6 个子命令的函数起，按 ast 顺着引用求闭包）不是
同一份算法。拿**出货算法本身**（不是探针）跑在今天的文件上：

```
$ python3 -c "
import sys; sys.path.insert(0, 'research/scripts'); import admission
source = open('research/scripts/admission.py', encoding='utf-8').read()
digest_bytes, wanted_count = admission.crash_case_judging_digest_text(source)
digest_lines = digest_bytes.decode('utf-8','surrogateescape').count(chr(10))
total_lines = source.count(chr(10))
print(f'{wanted_count} 个定义、{digest_lines} 行（{digest_lines*100//total_lines}%，总 {total_lines} 行）')
"
52 个定义、595 行（15%，总 3939 行）
```

52 个定义、595 行、15%，比探针的 42/427/10% 还要再宽一些——差的 10 个定义主要是分派表带进来的
`command_crash_case_command`、`command_crash_case_shardable` 等完整命令函数（探针的旧清单只手选了
`command_crash_case_judge`、`command_crash_case_record`、`command_crash_case_marker_check` 三个，
没有覆盖 `crash-case-command`、`crash-case-marker-path`、`crash-case-shardable` 三个子命令）。
**结论**：K4 判决引的「11%」这个数今天不管用哪把尺子量都不再是 11%（探针原样：10%；出货算法：15%），
两把尺子的方向还相反（一个说变窄了、一个说变宽了）——因为它们本来就是两把不同的尺子，而 D2 出货之后
「真正的判法覆盖面」应该以出货算法（15%）为准，不是 r2 攻方那份未进代码的探针。这不构成对 D2 改法
本身的否定：**K4 原来担心的是「代价太宽、覆盖不需要的东西」**，出货的 15% 依然比「整份文件进指纹」
（原来的 Y1，100%）窄得多，方向仍然对；只是「11%」这个具体百分点不能原样搬到今天当证据用。

## 附：开工核对（原样命令与输出）

```
$ sha256sum $(awk '{print $2}' research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt) > /tmp/actual-sums.txt
$ diff <(sort research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt) <(sort /tmp/actual-sums.txt) && echo "全部 14 个逐字节相同"
全部 14 个逐字节相同

$ sha256sum research/prompts/defs-gatebatch-m2-r2-fixes.diff research/prompts/defs-gatebatch-m2-g2-changes.diff
a4c1ffecf1f7cf1a1612d3e591dc89063236d152114e306b86a739f76112715e  research/prompts/defs-gatebatch-m2-r2-fixes.diff
d2a94fafe7b200bf19ed455c389f28db80e2e04cb53c5bc737d1bc92422f5864  research/prompts/defs-gatebatch-m2-g2-changes.diff
```
两份都与背景材料第 9 行给的哈希前缀一致（`a4c1ffecf1f7cf1a…`、`d2a94fafe7b200bf…`）。

```
$ ls -la /tmp/claude-1000/gate-batch-m2-r1-fixes/backup/research/scripts/admission.py
-rwxrwxr-x 1 fy5090 fy5090 188978 Sep 26 12:08 /tmp/claude-1000/gate-batch-m2-r1-fixes/backup/research/scripts/admission.py
```
r2 攻方留下的、K4 探针默认对照的那份备份文件今天仍在，用它核对过第六节「判法摘要相同」那句。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| shard=across-machines 登记字段 | **一致，但背景材料字面有一处过时** | 代码链路（常量→解析→CrashCase 字段→子命令）逐行核实一致；`stage-inputs.tsv` 今天登记了 3 条 `shard=across-machines`，不是背景材料沿用的「两条」 |
| crash-case-shardable 进判法摘要 | **一致（真跑自证核过）** | 分派表、闭包入口列表、弄坏开关三处都落地，`--selftest` 232 格与弄坏开关下的 1 格红都亲手跑过 |
| merge 那一趟的线程行 | **一致** | `judge_worker_threads` 见 `shards=` 就转 `judge_threads_of_each_shard` 逐片判，驱动脚本等两片退出、核 `mode=run` 行、merge 核 `mode=merge` 行，逐处对上代码行号 |
| 双机驱动按内容进指纹 | **一致** | `SHARD_DRIVER_FILES` → `shard_driver_lines` → `manifest_of_files`，`stage-inputs.tsv` 54 号行同步列了这两份路径 |
| 两份被判条款字面是否跟上分片这一层 | **规则没说（实缺一处）** | `implementation-workflow.md`「测试与崩溃检测优先多线程」与 `crash-verifier.md` 全文零处提「分片」「双机」；双机分片下第二台那一片的 cargo test 完全没有内存上限（本机的 `run-with-memory-cap.sh` 包不到 ssh 起的第二台进程），两份条款都没提这件事 |
| K4「判法闭包 365/3194 行（11%）」 | **对不上，差值已查清** | 同一份 r2 探针跑在今天文件上：42 个定义、427 行、10%（+5 定义、+62 行、总行数 +745、−1 个百分点）；差异来自 D2 出货之后闭包合理多收了分片相关的 5 个定义，涨幅远小于文件本身的涨幅，方向仍是「变窄」 |
| K4「这一批 21 个 hunk 0 个落在闭包里」 | **对不上，是探针口径问题，不是代码回退** | 在正确重建的 r2 基线上重算已经是「1 个」不是「0 个」（判法摘要哈希独立核过「相同」，证明不是真改动，是探针给「-」行记行号的已知不精确处）；套到今天更长的文件上变成「3 个」，纯属拿旧 diff 的绝对行号套新文件长度的巧合，2 个额外命中经人工核对（152–162 行撞上 REGISTRATION_TABLE 等常量）确认与判法无关 |
| K4 出货算法（不是探针）今天的真实闭包 | **新增数据点，主 agent 判决时可用** | `crash_case_judging_digest_text` 本身：52 个定义、595 行、15%（比探针的口径更宽，因为它连全部 6 个分派子命令函数都算进去了，探针当年只手选了 3 个） |

## 没做什么

- 不判 L1（改法撑不撑得住：P1–P10、D2、标记字段、Y1/K3 floor-raise/J1-c 站住不站住）——那是这一轮 Opus 攻方腿的格。
- 不判 L3（`stage-inputs.tsv` 七条 `crash-case:` 登记行逐格核对完整性）——那是这一轮本地攻方腿的格。
- 不碰 54 号与 `research/scripts/layer0-shard-run.sh` 里读层 0 发现日志那几段（背景材料第 19 行明写不归这一轮）；
  读到了但没引用、没判。
- 不跑重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 `layer0` 的测试目标、全量 `cargo test`、
  不带 `--selftest` 的双机驱动，一次都没跑；`admission.py --selftest` 与 `lib_heavy_tests.py --selftest`
  按共用约束属于自证脚本，不在重型测试清单里，跑的都是这两条自证（一次前台、一次因超时被移到后台，
  见正文第二节命令与输出）。
- 双机分片的「第二台内存没有上限」这条发现只核了脚本正文（`layer0-shard-run.sh`、
  `layer0-shard-configuration-check.sh`、`layer0-shard.env.example`），没有第二台机器可实测，标「复核不了：
  没有第二台环境」。
- K4 数字里 3194 与我重建出的 3202 行的 8 行落差没能查清具体原因，标「未核实」（不影响判法摘要本身逐字节
  相同这条独立证据）。
- 没有对 L2 之外的任何格给出判定或倾向；下面「附：模型与复跑」只登记我自己产出的验证工具，不代表对
  L1/L3 的立场。

## 附：模型与复跑

- 模型目录：`research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh`（重建 r2 基线、跑 r2 探针原样
  两遍、跑今天出货的 D2 闭包算法一遍），`SHA256SUMS` 在同目录。
- 复跑：`bash research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh <仓根>`，我在写这份报告时已经
  跑过一遍，输出与正文第六节贴的原样一致。
