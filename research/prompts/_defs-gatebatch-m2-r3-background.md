# 里程碑二收尾：门禁批（层 0 续跑接入、崩溃枚举按用例复用）三方，第三轮正文（2026-09-27）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K1 K2 K3 K4 K5 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 D2 M1 M2 M3 G1 G2 G3 L1 L2 L3 -->

## 一、这一轮要判什么

第二轮判决 `research/prompts/defs-gatebatch-m2-r2-main-verification.md` 第二节给了改法 P1–P7、P4b、P9、P10、D2 与「标记字段」（被攻过零轮），第四节写死第三轮只攻三样：这几条改法的改后代码；实分一（层 0 双机分片）带进这几份文件的分片判法；前两轮站住的形态（Y1、K3 floor-raise、J1-c）撑不撑得住。第三轮之后停，这一轮新冒出来的零轮形态写进判决的交用户表。

改法由 G1 做完（报告 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md`，diff `research/prompts/defs-gatebatch-m2-r2-fixes.diff`，1630 行，sha256 `a4c1ffecf1f7cf1a…`），实分一与第二轮判决余项由 G2 接进来（报告 `research/prompts/defs-gatebatch-m2-g2-report.md`，diff `research/prompts/defs-gatebatch-m2-g2-changes.diff`，1498 行，sha256 `d2a94fafe7b200bf…`），两批 2026-09-27 00:32 UTC 一起进了提交 ecdf8465。被判的就是这两批 diff 落在今天文件里的样子。攻击面不重复前两轮（第一轮：54 号范围、判法进不进指纹、排除法漏认与减得太少、构建输入、闸的绕法与误拒、文字矛盾；第二轮：Y1–Y8 修补本身、改法的证红办法、第 36、37 行两条用例的判法字段、指纹变了之后的处置、第一轮判决自身）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| L1 | **改法撑不撑得住**：P1（`command_under_launcher` 认短选项合写）、P2（`systemd-run -E` / `--setenv=`）、P3（libtest 带值选项之后的 `--list`）、P4 / P4b（按每一处同名 `fn <名>(` 判 `#[ignore]`）、P5（判不出按没标算）、P6 / P7（拼名字认 `::core::` / `std::` 前缀与 `env!` / `option_env!`）、P9（runner / wrapper 首词之后指到现存文件的参数按内容进指纹）、P10（`--config` 按 TOML 读 runner）、D2（判法摘要：准入模块按 ast 从判法入口求模块级闭包）、标记字段（配置的线程数与实际起的线程数分记）；以及前两轮站住的 Y1、K3 floor-raise、J1-c | 照今天的代码，每一处改法有没有新的假绿（该拒没拒、该重跑没重跑）或假红（合法命令被拒、合法复用被作废）；第二轮判决给 P4、P5 写的代价「方向是多拒，接受」在今天的代码上是不是真的只多拒；D2 的闭包漏看什么（`getattr`、字符串拼出的函数名、被 import 的别的模块、分派表之外的入口），漏看的那一处改了判法而摘要不变就是假复用；Y1、K3 floor-raise、J1-c 三处第二轮判「站住」的，在 G1、G2 改过之后还站不站得住 |
| L2 | **分片判法与条款一致**：实分一带进来的 `shard=across-machines` 登记字段、`crash-case-shardable` 子命令（G2 第 3 条把它加进 `CRASH_CASE_JUDGING_SUBCOMMANDS`）、merge 那一趟的线程行、双机驱动按内容进两条层 0 流的指纹（G2 第 4 条）；第二轮判决报出的数 | 正推：代码做的是不是 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」「测试与崩溃检测优先多线程」两节与 `.claude/agents/crash-verifier.md` 说的；分片那一趟判绿要的每一样（两片都跑完、merge 线程判定、指纹两台一致）在代码里各落在哪一行、缺哪一样。核数：第二轮判决第一节 K4 那一行「判法闭包 365 / 3194 行（11%）」「这一批 21 个 hunk 0 个落在闭包里」在今天的 `admission.py` 上重算是多少，对不上的写出差多少、为什么 |
| L3 | **登记行的完整性**：`.claude/gate.d/stage-inputs.tsv` 今天的 7 条 `crash-case:` 行（第 36–42 行） | 逐格核：每条登记的判法字段（`count-line=`、`exhaustive=`、`threads=`、`shard=` 等）是什么；那条用例的源码打得出哪些行（grep 原样）；54 号快档与 `--full` 对每个字段各判什么（引 `admission.py` 的判法行号）；登记了而用例打不出的字段、用例打得出而没登记的判定、没有线程判定的用例，各列出来 |

**共用问句**：照今天的字面与代码，哪一步会做错、放过、或误拒；给具体的命令、改动或派发情形；能在临时拷贝上量的量出来。

**不归这一轮的**：54 号与 `research/scripts/layer0-shard-run.sh` 里读层 0 发现日志的那几段（2026-09-27 JST 11:4x 工具员加的，报告 `research/prompts/defs-gate54-findings-report.md`；工作区里还没提交），与写发现日志的 `crates/singlefs-harness/src/crash.rs` 一起归代码第二轮三方；这一轮的腿读得到它们，不攻、不列。

## 二、实现今天的样子（主 agent 的观测，2026-09-27 JST 12:3x）

- 自证现跑（原样末行，日志在主 agent 草稿目录，腿自己复跑为准）：
  - `SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest` → `✓ admission.py 自证通过：232 格都对（含弄坏开关 skip-unchanged、…）`，rc=0
  - `python3 .claude/hooks/lib_heavy_tests.py --selftest` → `✓ lib_heavy_tests 自检通过（查了 141 种：cargo 与包装过的命令行 55 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 33 种、直接执行的测试二进制 20 种、按名字判的脚本 6 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种）`，rc=0
  - `bash .claude/hooks/heavy-test-guard.sh --selftest` → `✓ 自检通过（查了 661 种，其中该拒 122 种）：…`，rc=0
  - `bash research/scripts/layer0-shard-run.sh --selftest` → `✓ layer0-shard-run.sh 自证通过：11 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）`，rc=0（11 格里第 ⑦ 格是发现日志那一件加的，不归这一轮）
- 门禁 72 号现跑 `✓ 改过的定义与共用约束 10 份都有去向（新写的判决文件 2 份，另有 8 份按 .claude/agent-def-review-exempt 豁免，基准 faf255e235300d129ede6d6f85af31d686a88519）`。
- 被判文件自 ecdf8465 之后的改动：`.claude/rules/implementation-workflow.md` 已提交的 2 行（别的会话的措辞改动）；工作区里 `.claude/gate.d/54-layer0-replay.sh` +150 行、`research/scripts/layer0-shard-run.sh` +60 行与 `research/scripts/layer0-shard-run-selftest.sh` +34 行（自证第 ⑦ 格）是发现日志那一件（不归这一轮），`.claude/gate.d/stage-inputs.tsv` 第 34 行 E142 那一行加了一个输入 `research/prompts/e142-r19-prereg.md`（与崩溃枚举无关）。`admission.py`、`lib_heavy_tests.py`、`heavy-test-guard.sh`、`layer0-shard-configuration-check.sh` 与 ecdf8465 逐字节相同（`git diff --stat ecdf8465 -- <这几份>` 为空）。
- `crates/` 这一轮不碰（A3a、模型跟上乙、乙-配置续三个实现员在改）；开工快照 `research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt`（14 个文件：第二轮那 11 个，加 `research/scripts/layer0-shard-run.sh`、`layer0-shard-run-selftest.sh`、`layer0-shard-configuration-check.sh`）。
- 真 `--full` 与带真 cargo 的双机驱动一次都没跑过（G1、G2 都只在假 cargo 与临时仓上证过，两份报告的「推翻条件」各写了一句）；这一轮同样不跑。

## 三、条款（材料员整段抄进附录）

- `.claude/rules/implementation-workflow.md` 全文；
- `.claude/agents/crash-verifier.md` 全文；
- 第二轮判决 `research/prompts/defs-gatebatch-m2-r2-main-verification.md` 全文；
- 第一轮判决 `research/prompts/defs-gatebatch-m2-r1-main-verification.md` 第二节（J1–J5 的判定，Y1、J1-c 的来历）；
- G1 报告 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md` 全文与 G2 报告 `research/prompts/defs-gatebatch-m2-g2-report.md` 全文；
- `records/2026-09-24-里程碑二收尾调度.md` 第三节「崩溃枚举的跑法」那一行（用户原话）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | L1 | 在临时拷贝上造改动、命令与日志攻改后的代码：`admission.py` 的子命令与 `--selftest`、54 号的场景自证（假 cargo）、hook 喂 JSON 看退出码；D2 闭包拿「改一处判法、看摘要变不变」量；被判的重型命令一条都不执行 |
| 云端正推（Sonnet） | L2 | 从条款正推：分片判法每一步在代码里的落点、缺哪一步；重算第二轮判决报的数 |
| 本地攻方 | L3 | 按事实表逐格核七条登记行：登记字段、用例打出的行（grep 原样）、判法行号；每行写来源（命令与输出行），每张表不超过 7 行 |

两条攻方腿不重叠：Opus 攻改法与站住形态的代码语义，本地只逐格核七条登记行的字面与用例打出的行。本地派攻方不派辩方：第二轮已有辩方腿复核第一轮判决，第三轮三格里没有要替之辩护的一方。

## 五、交付

- 腿的报告写 `research/prompts/defs-gatebatch-m2-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-gatebatch-m2-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行；引 kb 与条款写那份文件自己的行号，去文件里现查，不从背景材料里数。
- 不跑重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test`、双机驱动不带 `--selftest` 一律不跑；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知；交回之前后台不许留着跑的东西。
### 小节清单：`.claude/rules/implementation-workflow.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| （实现改动的流程：写代码 标题之下、第一个下级标题之前的正文：第 2-5 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 三步，缺一步就不算做完 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 改 agent 定义与共用约束，走同一条三步 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 代码轮派腿之前记一份开工快照 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 重型测试只在提交时跑 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 测试与崩溃检测优先多线程 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |

### 小节清单：`.claude/agents/crash-verifier.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 崩溃一致性验证员（crash-verifier） | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| （崩溃一致性验证员（crash-verifier） 标题之下、第一个下级标题之前的正文：第 11-16 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 输入（主 agent 必须给） | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 做什么 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 写范围 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 产出 | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |
| ## 没做什么（固定会有的） | 抄 | 正文第三节点名的条款来源，全文整段抄进附录 |

### 小节清单：`research/prompts/defs-gatebatch-m2-r2-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁批（层 0 续跑接入、崩溃枚举按用例复用）三方第二轮判决（2026-09-27） | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| （门禁批（层 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 一、各格判定 | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| ## 二、改法（被攻过零轮，第三轮攻） | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| ## 三、实现次序 | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| ## 四、第三轮 | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| ## 五、按路径点名被判的定义与文件（门禁 72 号） | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |
| ## 回看决策 | 抄 | 正文第三节点名的条款来源（第二轮判决），全文整段抄进附录 |

### 小节清单：`research/prompts/defs-gatebatch-m2-r1-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁批第一轮判决（defs-gatebatch-m2-r1，2026-09-26） | 不抄 | 正文只点名第二节；顶部标题本身不含判定内容 |
| （门禁批第一轮判决（defs-gatebatch-m2-r1，2026-09-26） 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 正文只点名第二节；这段是文档头 doc-lint 编号声明与空行，不属于第二节内容 |
| ## 一、这一轮 | 不抄 | 正文第三节只点名「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，其余各节不进材料 |
| ## 二、逐格判 | 抄 | 正文第三节明写「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，就是这一节 |
| ## 三、改法（被攻过零轮，第二轮攻） | 不抄 | 正文第三节只点名「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，其余各节不进材料 |
| ## 四、第二轮 | 不抄 | 正文第三节只点名「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，其余各节不进材料 |
| ## 五、按路径点名被判的定义与文件（门禁 72 号） | 不抄 | 正文第三节只点名「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，其余各节不进材料 |
| ## 回看决策 | 不抄 | 正文第三节只点名「第一轮判决第二节（J1–J5 的判定，Y1、J1-c 的来历）」，其余各节不进材料 |

### 小节清单：`research/prompts/defs-gatebatch-m2-r2-fixes-report.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 门禁批第二轮改法（G1）交回：P1–P7、P9、P10、D2、标记字段，加 singlefs-39 的 54 号替换 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| （门禁批第二轮改法（G1）交回：P1–P7、P9、P10、D2、标记字段，加 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 结论 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 改动一览（只这五份） | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## D2 与标记字段：改前判错 → 改后判对 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 自证与弄坏开关证红 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## P1–P7、P9、P10：攻方探针改前判错 → 改后判对 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## rerun.sh 复跑（仓的临时拷贝） | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 门禁与 lint（仓里现文件，2026-09-26 UTC） | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 与实分一交来的四份 diff 相邻的地方（`/tmp/claude-1000/impl-shard-1/deliver/`，这一次没套） | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 看到但没做（交主 agent 定） | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 推翻条件 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 没做什么 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |
| ## 草稿目录与清理 | 抄 | 正文第三节点名的条款来源（G1 报告），全文整段抄进附录 |

### 小节清单：`research/prompts/defs-gatebatch-m2-g2-report.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # G2 交回：门禁批第二轮判决的余项与实分一接入（tooling-writer） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| （G2 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 随（顶部一级标题与其下引言段合并成一个行区间，checklist-specs.py 自动处理） |
| ## 结论 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 改过的文件与 git diff --stat | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 各条改法与证红 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 1 条：实分一三份 diff 重打（admission.py、stage-inputs.tsv、lib_heavy_tests.py） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 2 条：三份新脚本（来源 deliver，照 G1 之后的 admission.py 核过） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 3 条：`crash-case-shardable` 进 `CRASH_CASE_JUDGING_SUBCOMMANDS` | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 4 条：双机驱动进两条层 0 流用例的指纹（选「按内容算进输入」） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 5 条：`strace -E VAR=VAL` / `--env=VAR=VAL` | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 6 条：`crash-case-command` 起用例时清掉调用方的 `SINGLEFS_LAYER0_SHARD` | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 7 条：stage-inputs.tsv | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 8 条：`.claude/rules/implementation-workflow.md`（第二种活，改的规则） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 第 9 条：54 号（最后套） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ### 途中收到的追加 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 出口各项（在仓里现跑，UTC 2026-09-26 23:3x–23:5x；末行原样） | 抄 | 随（标题里带冒号，@标题 取法会在冒号处切错，用 --extra research/prompts/defs-gatebatch-m2-g2-report.md:156-178 取同一段） |
| ## 新写的 `# gate-similar:` / `# hook-events:` 行 | 抄 | 随（标题里带冒号，@标题 取法会在冒号处切错，用 --extra research/prompts/defs-gatebatch-m2-g2-report.md:179-186 取同一段） |
| ## 第三轮要攻的改后字面（判决第四节口径） | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 看到但没做 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 没做什么 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 草稿目录与清理 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |
| ## 补记 | 抄 | 正文第三节点名的条款来源（G2 报告），全文整段抄进附录 |

### 小节清单：`records/2026-09-24-里程碑二收尾调度.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 里程碑二收尾调度（2026-09-24） | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| （里程碑二收尾调度（2026-09-24） 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| ## 零、收尾出口（用户 2026-09-24 JST 24 点后定的九步，次序照做） | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| ## 一、批次 | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| ## 二、交用户的 | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| ## 三、用户 2026-09-24 第二次定案（JST 13 点前后，两批问答） | 不抄 | 正文只点名这一节里的一行（第 190 行「崩溃枚举的跑法」），不是整节；用 --extra 按行区间取 |
| ## 历史版本 | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |
| ### 2026-09-24 | 不抄 | 正文第三节只点名「第三节『崩溃枚举的跑法』那一行，第 190 行」，其余各节不进材料 |


### 小节清单：`.claude/kb/decisions/02-RAID条带策略.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D2 RAID 条带策略 —— 半定（一项未定） | 不抄 | 正文第 13 行「D2（判法摘要：…）」是这一批改法自己的标签（与 `defs-gatebatch-m2-r2-main-verification.md` 表里「D2｜判法摘要替掉整份准入模块进指纹」同一个 D2），不是对这份决策编号 D2 的引用；`checklist-specs.py --cited` 只按字面 `D2（` 匹配，误认成决策文件，与这一轮无关 |
| （D2 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 同上，D2 在这一轮是改法标签，不是这份决策 |
| ### 已定项 | 不抄 | 同上；这份决策与门禁批第三轮判什么无关 |
| #### 已定项 1：「宽度可变」的粒度 | 不抄 | 同上 |
| #### 已定项 2：各盘不必等大 | 不抄 | 同上 |
| #### 已定项 3：设备集合变化走显式「几何再求值」事务 | 不抄 | 同上 |
| #### 已定项 4：几何再求值事务的三样形态 | 不抄 | 同上 |
| #### 已定项 5：根环不重放置 | 不抄 | 同上 |
| #### 已定项 6：每次写用几列 | 不抄 | 同上 |
| #### 已定项 7：根环归属逐区域显式存设备身份 | 不抄 | 同上 |
| #### 已定项 8：参与哪些设备 | 不抄 | 同上 |
| #### 已定项 9：第一个可运行目标跑 2 块盘 | 不抄 | 同上 |
| #### 已定项 10：一个单元整个落在一列上 | 不抄 | 同上 |
| #### 已定项 11：条带的生命周期——部分条带钉住、后台惰性重落 | 不抄 | 同上 |
| #### 已定项 12：成员表住码 3 打包记录，走容器索引，容器恒走镜像 | 不抄 | 同上 |
| #### 已定项 13：降级期间只读 | 不抄 | 同上 |
| #### 已定项 14：条带表按落点排，与分配记录树同一个坐标系 | 不抄 | 同上 |
| #### 已定项 15：「生命周期不同的对象」的通用判据 | 不抄 | 同上 |
| #### 已定项 16：不发出小于 `io_min` 的写 | 不抄 | 同上 |
| #### 已定项 17：不让两个生命周期不同的对象共享同一个物理映射单元 | 不抄 | 同上 |
| #### 已定项 18：系统配置里 w_max 与 g 写 4 / 4 | 不抄 | 同上 |
| #### 已定项 19：根槽与系统配置槽的槽距 | 不抄 | 同上 |
| #### 已定项 20：条带表与容器索引不 day-1 注册进树表 | 不抄 | 同上 |
| #### 已定项 22：全条带写，永不 read-modify-write | 不抄 | 同上 |
| ### 未定项 | 不抄 | 同上 |
| #### 未定项 21：parity 几格 | 不抄 | 同上 |
| ## 历史版本 | 不抄 | 同上 |
**出处 `.claude/rules/implementation-workflow.md:1-5`（整段抄，未转述）**

```markdown
# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/implementation-workflow.md:6-15`（整段抄，未转述）**

```markdown
## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

```

**出处 `.claude/rules/implementation-workflow.md:16-23`（整段抄，未转述）**

```markdown
## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

```

**出处 `.claude/rules/implementation-workflow.md:24-29`（整段抄，未转述）**

```markdown
## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），连同派腿的时刻（`date -u`）交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。

```

**出处 `.claude/rules/implementation-workflow.md:30-45`（整段抄，未转述）**

```markdown
## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段（57 号 herd7 就是）没有复用判定，每次照跑。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树；只在 `research/scripts/gate-staged.sh` 起的那一趟里比，直接跑 `gate.sh` 一律照跑），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

```

**出处 `.claude/rules/implementation-workflow.md:46-81`（整段抄，未转述）**

```markdown
## 重型测试只在提交时跑

**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：

- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
- 全部实验复跑：87 号。
- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv`、`strace -E` / `--env` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。

只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算（短选项合写的 `strace -fo <文件>`、`flock -xw 10` 同样剥）。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算，跟在 `--skip`、`--logfile` 这类带值的选项后面的 `--list` 是那个选项的值，照算。命令位置上执行的脚本（`bash <脚本>`、`./<脚本>.sh`、`source <脚本>`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。

| 场合 | 跑不跑 |
|---|---|
| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量（HEAD + 暂存区的 worktree 里 `--full`，连同登记的崩溃枚举用例）、QEMU、herd7、crates 变异整表由 `crash-verifier` 跑，主 agent 后台派、看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
| 其余任何时候 | 不跑 |
| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员在 HEAD + 暂存区的 worktree 里跑 54 号 `--full`（连同登记的崩溃枚举用例），另跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，57 号没有复用、照跑 |
| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |

由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

```

**出处 `.claude/rules/implementation-workflow.md:82-90`（整段抄，未转述）**

```markdown
## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

```

**出处 `.claude/agents/crash-verifier.md:1-16`（整段抄，未转述）**

```markdown
---
name: crash-verifier
description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
tools: Read, Bash
model: opus
effort: high
omitClaudeMd: true
---

# 崩溃一致性验证员（crash-verifier）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。

```

**出处 `.claude/agents/crash-verifier.md:17-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
- 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
- 54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
- 报告路径与草稿目录。

```

**出处 `.claude/agents/crash-verifier.md:25-34`（整段抄，未转述）**

```markdown
## 做什么

1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。55、57、59 在主工作区跑，判的是工作区那一份：每道开跑前取它自己的输入（55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，55 号按名字读的 `research/results/` 产物这里不查，由整轮门禁里的 87 号兜；57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层；要没有输出，别处的未跟踪文件不挡；57 号读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
   1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`，经内存包装照第 1b 步）：它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus Cargo.toml Cargo.lock research/scripts | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

```

**出处 `.claude/agents/crash-verifier.md:35-38`（整段抄，未转述）**

```markdown
## 写范围

- 报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。

```

**出处 `.claude/agents/crash-verifier.md:39-42`（整段抄，未转述）**

```markdown
## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「没做什么」。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），59 号那一份是变异分诊员要的输入。

```

**出处 `.claude/agents/crash-verifier.md:43-47`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:1-6`（整段抄，未转述）**

```markdown
# 门禁批（层 0 续跑接入、崩溃枚举按用例复用）三方第二轮判决（2026-09-27）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K1 K2 K3 K4 K5 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 D1 D2 D3 V2 V3 V4 V5 V6 V7 L2 L4 L6 L7 L12 L13 R2 R3 R5 R7 T2 T3 T4 M1 M2 M3 M4 M5 G1 G2 G3 -->

正文 `research/prompts/_defs-gatebatch-m2-r2-body.md`，背景 `_defs-gatebatch-m2-r2-background.md`，被判的 diff `_defs-gatebatch-m2-r2-diff.md`（改法 Y1–Y8，1707 行、6 个文件）。三条腿：辩方 `defs-gatebatch-m2-r2-defense-output.md`（K5 + K2）、攻方 `defs-gatebatch-m2-r2-opus-output.md`（K1、K3、K4，探针 `defs-gatebatch-m2-r2-opus-model/`）、本地攻方 `defs-gatebatch-m2-r2-local-attack-output-s1.md` / `-s2.md`（K3 事实表，两份干净样本；第一次调用因提示里的 Rust `::` 被损坏闸误红作废，`-output-void1.md`）。核查员 `defs-gatebatch-m2-r2-verifier-output.md`：核 66 处，✓ 59、✗ 4、分不清 3，判别力自证通过；攻方 Y2–Y7「打中，量过」在含 `.git` 的仓副本上复跑全部重现，6 条 SUMMARY 与 3 份日志逐字节相同。开工快照 `defs-gatebatch-m2-r2-snapshot/sha256sums.txt` 11 个文件，腿交齐之后主 agent `sha256sum -c` 全 OK。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:7-24`（整段抄，未转述）**

```markdown
## 一、各格判定

| 格 | 判定 | 依据 |
|---|---|---|
| K5 第一轮判决的打中 | **站得住**：J1-a、J1-b、J4-b、J2-a、J2-b、J3-a 六处打中是真的，Y1、Y2、Y5、Y6、Y7 对症；J1-c「两趟并发只会假红」与 J4-c 那一半接受误拒都站得住 | 辩方第一、二节；核查员 |
| K5 第一轮判决自己的引文 | **第一轮判决第 16 行「`admission.py:168`」引错了**：那一行是 `AdmissionCondition.fingerprint_text` 的文档字符串，够不着「判日志的代码不进指纹」；结论不变，撑它的是攻方与核查员表里的别的引文。第一轮判决是冻结证据，不改，这一处以这里为准 | 辩方第一节；核查员第二节（属实，三条独立证据） |
| K2 证红办法 | 新自证格在旧代码上红，只证新检查认得旧行为；攻方探针在合成小仓与假 cargo 上跑、闸只喂 JSON；54 号真 `--full` 一次没跑。**这一格不改改法，改的是说法**：Y1–Y8 与这一轮的改法都只在模型上证过，真 `--full` 的第一次是提交时崩溃验证员那一趟 | 辩方第四节 |
| K1-Y1 `--extra-file` | **没打中**：快档与 `--full` 都经 `write_crash_case_manifest` 同一个函数算，`crash-case-record` 不重算 | 攻方 K1 第一节 |
| K1-Y2 `#[ignore]` 属性解析 | **打中（量过）**：同名函数 cfg 二选一（V3）、子模块同名且标 ignore 的写在前面（V4）闸放行、自查退 0；宏生成的用例（V5）闸放行；`# [ignore]`（V7）误拒。V3–V5 不是新引入，V7 是 | 攻方 Y2；核查员复现 |
| K1-Y3 剥包装与 runner | **打中（量过）**：短选项合写（`strace -fo`、`flock -xw`、`systemd-run -qu`、`/usr/bin/time -ao`，L2、L4、L6、L7、L13）、`systemd-run -E` / `--setenv=` 设 runner（R2、R3）、`--config` 带引号的 `"runner"` 键（R7）；派发点名的三个形状拒对了。R5（`--config` 别名）cargo 1.98 不认、走不到（核查员独立复现）；L12（`env -S`）在这一批没改的 `lib_shell_words.py` 里，不归这一轮 | 攻方 Y3；核查员 |
| K1-Y4 `--list` 不算重型 | **打中（量过），这一批新引入**：`--skip --list`、`--logfile --list` 里 `--list` 是前一个选项的值，libtest 照样全跑，闸放行（T2–T4） | 攻方 Y4 |
| K1-Y5 拼出来的名字 | **打中（量过）**：`::core::include_str!(::core::concat!(…))`（M1）、`include_str!(env!(…))`（M2）认不出；M3（`build.rs` 只写 `mod scan;`）属文件头已声明 | 攻方 Y5、Y6 |
| K1-Y6 `mutations.tsv` 与 `src/bin` 不进 | **打中（量过），这一批新引入**：`concat!` 拼出这两类路径（M4、M5）照样被减掉 | 同上 |
| K1-Y7 runner 按内容进指纹 | **打中（量过）**：runner 前带解释器只哈希了解释器（G1、G2）；runner 脚本 `source` 的文件改了指纹不变（G3，判别子观测不到） | 攻方 Y7 |
| K3 floor-raise（`stage-inputs.tsv` 第 36 行） | **没打中**：只跑了部分切片的日志判红，靠 `threads=` 那一格的片数守恒；缺 `exhaustive=` 不弱于两条流（用例断言状态数等于闭式） | 攻方 K3；本地攻方两份样本 |
| K3 c561（第 37 行） | **问句属实（量过）**：门禁只要 1 passed 加一行 `C561_SIGMA_FULL`，状态数、判缺席数、线程数一概不看；标记里 `worker_threads=` 记的是配置值、不是起了几个线程，字面误导。用例打不出这几行归实审 B3b（在改），登记字段等它交回补 | 攻方 K3；实审 B3b 规格 `/tmp/claude-1000/impl-rev-b3b/spec.md` |
| K4 整份准入模块进指纹 | **代价是真的，今天为零，范围太宽**：common-dir 里崩溃枚举用例的全绿标记今天 0 格；判法闭包 365 / 3194 行（11%），这一批 21 个 hunk 0 个落在闭包里、却每一个都让四条全重跑；与用户原话「改了只跑改了的部分」相反。攻方 D2（判法摘要）在模型上对 4 种非判法改动不变、对 4 种判法改动都变 | 攻方 K4 |

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:25-41`（整段抄，未转述）**

```markdown
## 二、改法（被攻过零轮，第三轮攻）

| 改法 | 做什么 | 修哪几格 |
|---|---|---|
| P1 | `command_under_launcher` 认短选项合写 | L2、L4、L6、L7、L13 |
| P2 | `systemd-run -E` / `--setenv=` 带进里面那条命令 | R2、R3 |
| P3 | libtest 带值的选项跳过它们的值再找 `--list`（cargo 与直接执行两处） | T2–T4 |
| P4、P4b | 属性按每一处同名 `fn <名>(` 判，有一处没标就算没标；`#` 与 `[` 之间许空白。代价（推的）：同一目标里别的模块有同名、合法不标的快用例时误判红——方向是多拒，接受 | V3、V4、V7 |
| P5 | 闸判不出标没标（找不到函数、导入不了准入模块）时按没标算。代价：准入模块坏了时点名登记目标的 `cargo test` 全被拒——方向是多拒，接受 | V5 |
| P6、P7 | 拼名字的正则认 `::core::` / `std::` 前缀，`env!` / `option_env!` 与 `concat!` 同样对待；任何 `.rs` 里有这种 include 时 `mutations.tsv` 与 `src/bin` 两类一份都不减 | M1、M2、M4、M5 |
| P9 | runner / wrapper 的值里，首词之后指到现存文件的参数也按内容进指纹；G3 写进准入模块文件头「认不出的」 | G1、G2（G3 声明） |
| P10 | `--config` 的值按 TOML 读，定了任何 `target.<…>.runner` 就当可能带 `--ignored` | R7 |
| D2 | 判法摘要替掉整份准入模块进指纹：准入模块按 ast 从判法入口求模块级闭包（连同 `main` 里那几项子命令的分派），闭包里每个定义的源码按名字排好算一个摘要，以一行进清单；54 号起用例的命令与环境改由准入模块交出（一个子命令），54 号整份不再进指纹。限度写进文件头（`getattr`、字符串拼出的函数名看不见；Python 升级不进） | K4 |
| 标记字段 | 标记里线程那一格分记「配置的线程数」与「实际起的线程数」 | K3 c561 那一格的字面 |

攻方 P1–P7、P9、P10 在副本上改完之后，它自己的两个探针剩 L12、R5（不归这一轮 / 走不到）与 M3、G3（已声明 / 观测不到），副本三份自证全过；这是攻方自己模型上的数，被攻过零轮。P8 不做（M3 属已声明）。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:42-48`（整段抄，未转述）**

```markdown
## 三、实现次序

1. 门禁审核会话 singlefs-39 交来的 54 号那一处替换（`/tmp/claude-1000/gate-fix-54-handoff/`：复用判定只认退 1 为可跳过，其余判红；换用 `research/scripts/stage-run-or-skip.sh`）先套上；
2. 这一节的改法由一个通用 agent 做（规格另写），每条先在旧代码上造会红的输入；
3. 实分一（层 0 双机分片）交来的 `admission.py`、54 号、`stage-inputs.tsv`、`lib_heavy_tests.py` 四份 diff（`/tmp/claude-1000/impl-shard-1/deliver/`）在 2 之后按新代码重打；
4. 第三轮攻 2、3 的改后字面（第一、二轮攻过的面不重复），腿跑着时这几份文件不动。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:49-52`（整段抄，未转述）**

```markdown
## 四、第三轮

只攻：第二节改法的改后代码（P1–P7、P9、P10、D2、标记字段）、实分一带进这几份文件的分片判法（`shard=across-machines`、`crash-case-shardable`、merge 那一趟的线程行），以及前两轮站住的形态（Y1、K3 floor-raise、J1-c）撑不撑得住。第三轮之后停；第三轮新冒出来的零轮形态写进判决的交用户表。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:53-56`（整段抄，未转述）**

```markdown
## 五、按路径点名被判的定义与文件（门禁 72 号）

被判的：`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/admission.py`、`.claude/gate.d/stage-inputs.tsv`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/rules/implementation-workflow.md`；一并核过（开工快照里、没改）的 `.claude/gate.d/stage-owners.tsv`、`.claude/agents/crash-verifier.md`、`research/scripts/check-segment-registry.py`、`.claude/main-agent.md`、`.claude/agents/gate-triage.md`。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-main-verification.md:57-60`（整段抄，未转述）**

```markdown
## 回看决策

不涉及决策：被判的是门禁、研究脚本、hook 与规则，不改 `.claude/kb/decisions/` 里任何一条决策。

```

**出处 `research/prompts/defs-gatebatch-m2-r1-main-verification.md:11-27`（整段抄，未转述）**

```markdown
## 二、逐格判

| 格 | 判 | 依据 |
|---|---|---|
| J1-a 只改登记表或 54 号本身 | **打中（量过），这一批新带进来的（改 54 号本身那一形之前就有）**：范围那一问（`54-layer0-replay.sh:114`）只按 `crates/ Cargo.toml Cargo.lock` 判，只改 `stage-inputs.tsv` 或 54 号的提交快档退 77、一格标记都不核；54 号的自证全带 `SINGLEFS_GATE_FULL=1` 跑所以没抓到；先提交带 ignore 的用例、后提交登记行，新用例一次都不会被核 | 攻方 J1、核查员复跑 |
| J1-b 判法不进指纹 | **打中（量过）**：判日志的代码搬进了 `admission.py`，它不进每条用例的指纹（`admission.py:168`），修前修后指纹都是 `639097891a56ec19…`，旧判法写下的标记照样被复用 | 同上 |
| J1-c 两趟并发 | 低危、推的：后一趟判红会删掉前一趟刚写的绿标记，只会假红 | 攻方 |
| J2-a 排除法漏认 | 打中（量过），今天的仓里没有这种写法：`include!(concat!(…))` 编译期拼文件名、别的包的 `build.rs` 按目录读本包 `tests/`，被减掉的文件其实编进了用例 | 攻方 |
| J2-b 排除法减得太少 | **打中（量过，真仓拷贝）**：只改 `crates/mutations.tsv`、`tests/common_tree_split/mod.rs` 或 harness 的 `src/bin/e142_*.rs`，四条用例的指纹全变，第二条流约 2.3 天（推的）要重跑；与用户「改了只跑改了的部分」相反，而每处改法都要在 `mutations.tsv` 留一行，这种提交最常见 | 攻方 |
| J3-a 指纹漏的构建输入 | 打中（量过），今天走到的可能性低：runner、`RUSTC_WRAPPER` 指的脚本换了内容，配置里 `build.rustc` 指的编译器换了版本，指纹都不变 | 攻方 |
| J3 其余 | `CARGO_BUILD_JOBS` 不进指纹对；主工作区与 worktree 的指纹不同只会偏假红，造不成假复用 | 攻方 |
| J4-a 闸的绕法 | 打中（量过，只喂 JSON）：`cargo nextest`、`cargo mutants`、`--config` 别名与 runner、`CARGO_TARGET_*_RUNNER`、拷走测试二进制再执行、`find -exec`、`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`systemd-run --scope`、`prlimit`；包装这一类以前就有，层 0 同样放行 | 攻方 |
| J4-b 用例的 ignore 没人查 | **打中（量过）**：去掉 c561 那条的 `#[ignore]` 之后，不带 `--ignored` 的命令跑 2^18 个状态，闸放行、`crash-cases` 自查退 0 | 攻方、核查员复跑 |
| J4-c 误拒 | 打中、危害小：`-- --ignored --list`、`--include-ignored --exact <一条快用例>` 被拒 | 攻方 |
| J5 文字 | 几份文字之间没有互相矛盾，都改成了逐条复用；层 0 规模第三轮判决「乙不采纳」与用户 18:5x 定案之间没写覆盖——主 agent 已在那份判决第四节之后补了覆盖记录；正推说的「四行三项都齐」不对（核查员 ✗），见 Y8 | 正推、核查员 |
| 本地 | 两份样本一致：四条用例保留、减去的文件数与四个抽样文件的去留与 `admission.py crash-case-manifest` 现跑对得上 | 本地、核查员 |

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:1-4`（整段抄，未转述）**

```markdown
# 门禁批第二轮改法（G1）交回：P1–P7、P9、P10、D2、标记字段，加 singlefs-39 的 54 号替换

写于 2026-09-26 UTC（交回时刻见文末）。规格 `/tmp/claude-1000/gate-batch-m2-r2-fixes/spec.md`，判决 `research/prompts/defs-gatebatch-m2-r2-main-verification.md` 第二、三节。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:5-12`（整段抄，未转述）**

```markdown
## 结论

- 判决第二节 P1–P7、P9、P10、D2、标记字段全部落进仓里五份文件；singlefs-39 的 54 号替换先套上（命中 1 次、新 inode，与 `54-after.sh` 逐字节相同）。P8 没做（照规格）。G3 写进 `admission.py` 文件头「管不到的」一段。
- 改前在攻方两个探针上各格原样判错，改后同一探针在仓的临时拷贝上复跑，剩下的是否只有 L12、R5、M3、G3 见「rerun.sh 复跑」一节。
- 三份自证在仓里全过：`admission.py` 207 格（原 167 格）、`lib_heavy_tests.py` 132 种（原 109 种）、`heavy-test-guard.sh` 658 种（原 628 种）；新加的每一格都有弄坏开关，开关打开时对应那几格转红（原样行见「弄坏开关证红」一节）。
- 47 号退 1、doc-lint 退 1，红的都不在这一批的文件里（check-segment-registry.py 的 E142 段序列、kb 里的编号引用），见「门禁与 lint」一节；62、63、gate-lint、shell-lint 全绿。
- 推翻条件：真仓里出现这一批改法之后照样放行的写法（例：`strace -E CARGO_TARGET_…_RUNNER=…` 这一类，见「看到但没做」），或 54 号 `--full` 真跑一趟时 `crash-case-command` 交的命令与用例实际要的不一样（这一批只在假 cargo 上证过）。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:13-28`（整段抄，未转述）**

```markdown
## 改动一览（只这五份）

| 文件 | 改了什么 | 对应改法 |
|---|---|---|
| `.claude/gate.d/54-layer0-replay.sh` | 套 singlefs-39 那一处（`stage-run-or-skip.sh`）；清单改给 `--judging-digest`，不再给 54 号与整份准入模块；起用例改照 `admission.py crash-case-command` 交的 NUL 分隔词跑（`read_crash_case_command`、`run_crash_case <日志> <命令…>`），线程数、续跑变量、进度目录都由它交；判日志与写标记的核数、线程数、explicit/default 取它交的前三个；删掉自己算线程数的那一段；文件头相应改写 | 替换、D2、标记字段 |
| `research/scripts/admission.py` | P4/P4b（`attributes_before_each_definition`、`definitions_marked_ignored`、`IGNORE_ATTRIBUTE_FORM` 许 `# [`）；P6（`COMPILE_TIME_COMPUTED_INCLUDE`：include 族参数不以字符串字面量开头就按宽处理，宏名带 `::core::` / `std::` 也认）；P7（`files_crash_cases_do_not_read` 在有这种 include 时一份都不减）；P9（`program_contents` 把首词之后指到现存文件的参数按内容接上）；D2（`crash_case_judging_digest_text` 等、`--judging-digest`、`crash-case-command` 子命令、`crash_case_launch`、`crash_case_worker_threads`）；标记字段（`started_worker_threads=` 由判日志写，`configured_worker_threads=` 由 `crash-case-record` 按 `--machine-cores/--threads/--threads-origin` 写，旧的 `--threads-text` 与 `worker_threads=` 去掉）；自证加格与 9 个弄坏开关；文件头 | P4、P4b、P6、P7、P9、D2、标记字段、G3 声明 |
| `.claude/hooks/lib_heavy_tests.py` | P1（`launcher_option`：短选项合写逐字母查表）；P2（`LAUNCHER_ENVIRONMENT_OPTIONS`：`systemd-run -E` / `--setenv` 的 NAME=VALUE 写在交回命令最前面，交回的首词是赋值时不当包装剥）；P3（`libtest_lists_only` 跳过带值选项的值，cargo 与直接执行两处都换）；P5（`crash_case_function_runs_without_ignored` 判不出按没标）；P10（`configuration_value_names_runner` 按 TOML 读 `--config`）；样本工作区多登记四条（cfg 二选一、子模块同名、宏生成、`# [ignore]`）；自证加格；`LIB_HEAVY_TESTS_BREAK` 五个开关；文件头 | P1、P2、P3、P5、P10 |
| `.claude/hooks/heavy-test-guard.sh` | 自证加 10 格（走 JSON 判定）；文件头里 `--list`、runner、「没标 #[ignore]」、接受的误拒三处字面跟着改；判定逻辑没动（它已经把剥出来的命令重新切词，NAME=VALUE 前缀照认） | P1–P5、P10 的闸那一侧 |
| `.claude/rules/implementation-workflow.md` | 重型测试清单「崩溃枚举用例」一条与 `--list` / 剥包装那一段的字面跟着闸改 | 字面 |

逐文件改动行数（`diff 备份 现文件` 数出来的 `>` / `<`）：54 号 +72 −50、admission.py +516 −95、lib_heavy_tests.py +174 −24、heavy-test-guard.sh +33 −5、implementation-workflow.md +2 −2。diff 在 `/tmp/claude-1000/gate-batch-m2-r2-fixes/my-changes.diff`（1630 行，由 `backup/` 与现文件生成；54 号那一份含 singlefs-39 的替换，因为备份是替换之前的）。

改法怎么写的、与判决字面有出入的两处：
- P6 写成「include 族宏的参数不以字符串字面量开头」一条正则，而不是逐个列 `concat!` / `env!` / `option_env!`：include 的参数只能是字面量或宏，这样写把判决点名的三个宏连同别的宏（用户自己的 `macro_rules!`）一起罩住，方向只会少减、多重跑。真仓 `crates/` 里 include 只有一处 `include_str!("model.rs")`（字面量，`grep -rnE 'include(_str|_bytes)?\s*!' crates --include=*.rs` 现跑），四条真用例的清单改前改后都是 61 个文件、减去 92 个（改前模块与改后模块各跑一遍 `crash-case-manifest`）。
- D2 的判法摘要：入口是判决列的那几项再加 `crash_case_launch`、`started_worker_threads_text`、`configured_worker_threads_text`；分派表只取 `crash-case-command/judge/record/marker-check/marker-path` 五项（「子命令 → 函数」进摘要，函数也当入口）；`main` 与 `if __name__` 那一段按原文进、不往下顺（`main` 引自证）。真仓那一份现算：闭包 47 个定义，摘要原文 525 行，模块全文 3615 行。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:29-50`（整段抄，未转述）**

````markdown
## D2 与标记字段：改前判错 → 改后判对

在草稿目录的临时小仓（包 pkg、用例 own_case、登记 count-line=C561_SIGMA_FULL，与 c561 同形）上，改前拿备份的 admission.py 与套完 singlefs-39 替换、还没做 D2 的 54 号（`/tmp/claude-1000/gate-fix-54-handoff/54-after.sh`），照那时 54 号给的参数（两个 `--extra-file`）算指纹；改后拿这一批的两份、给 `--judging-digest`。脚本 `/tmp/claude-1000/gate-batch-m2-r2-fixes/demo_d2_marker.py`，原样输出（`demo-d2-marker.log`）：

```
# D2：每条崩溃枚举用例的指纹里，准入模块与 54 号怎么进
D2	只改准入模块判法之外的部分（文件尾加一行注释） ⇒ 应当不变	改前：指纹变（0fa0e7184dad… → b5d072eeca38…），错；改后：指纹不变（c13846a9e8db… → c13846a9e8db…），对
D2	只改 54 号（文件尾加一行注释） ⇒ 应当不变	改前：指纹变（0fa0e7184dad… → a81a2d507b82…），错；改后：指纹不变（c13846a9e8db… → c13846a9e8db…），对
D2	改准入模块的判法（judge_worker_threads 把「至少两片」改成「至少三片」） ⇒ 应当变	改前：指纹变（0fa0e7184dad… → fb1cb1a21a71…），对；改后：指纹变（c13846a9e8db… → 1bc7706718b4…），对
# 标记字段：c561 那种只登记 count-line= 的用例，判绿写标记之后线程那几格
标记	改前：写退 0；线程那几格：worker_threads=SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核
标记	改后：写退 0；线程那几格：configured_worker_threads=SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核 ｜ started_worker_threads=读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；日志里有 LAYER0_PARALLEL_FINISHED 的另原样记在 parallel_finished=）
```

54 号那一侧另在开发副本上把 54 号改回「自己进指纹」（`write_crash_case_manifest` 里补回 `--extra-file "<判它的 54 号：…>" "$layer0_stage_script_path"`），跑 `admission.py --selftest`，新加的那一格转红，另一格是自证里「期望指纹」按 `--judging-digest` 算、与改回去的 54 号对不上（`mut54-selftest.log` 里的 ✗ 行，每行截到前 230 个字符；那份副本判完就删了，日志留着）：

```
  ✗ 54 号 --full 设续跑的环境变量：进度目录 <common-dir>/singlefs-layer0-progress/<这条用例的指纹>、输入指纹是这条用例的；调用方环境里的 SINGLEFS_LAYER0_START_OVER=1 在不带 --start-over
  ✗ 54 号快档（不带 SINGLEFS_GATE_FULL=1）：只改 54 号（加一行注释；54 号不进指纹） ⇒ 照样核标记（不退 77），三条标记都作数，判绿：退 1，输出尾部：EAD + 暂存区的 worktree
  ✗ admission.py 自证没过：2 格判错（共 207 格）
```

````

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:51-107`（整段抄，未转述）**

````markdown
## 自证与弄坏开关证红

仓里现文件上跑（2026-09-26 UTC），末行原样：

```
$ python3 research/scripts/admission.py --selftest   → 退 0
  ✓ admission.py 自证通过：207 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）
$ python3 .claude/hooks/lib_heavy_tests.py --selftest   → 退 0
  ✓ lib_heavy_tests 自检通过（查了 132 种：cargo 与包装过的命令行 52 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 28 种、直接执行的测试二进制 20 种、按名字判的脚本 5 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种）
$ bash .claude/hooks/heavy-test-guard.sh --selftest   → 退 0
  ✓ 自检通过（查了 658 种，其中该拒 121 种
```

admission.py 的开关在自证里面转红（照它原有的写法，每个开关一格「弄坏开关 … 下 … 那一格红」），新加的 9 个开关对应的原样行（`final-admission.log`）：

```
  ✓ 弄坏开关 runner-program-only 下「runner = bash <脚本>，脚本换了内容」那一格红（指纹不变）
  ✓ 弄坏开关 first-definition-only 下「同名函数 cfg 二选一、一份不标」那一格红（crash-cases 退 0）
  ✓ 弄坏开关 first-definition-only 下「子模块里同名标 ignore 的写在前面、顶层那一份不标」那一格红（crash-cases 退 0）
  ✓ 弄坏开关 ignore-attribute-without-space 下「# [ignore]」那一格红（crash-cases 退 2）
  ✓ 弄坏开关 concat-include-only 下「::core::include_str!(::core::concat!(…))」那一格红（改 other_target.rs 指纹不变）
  ✓ 弄坏开关 computed-include-subtracts-non-test-files 下「include_str!(concat!(…)) 拼出 ../mutations.tsv」那一格红（指纹不变）
  ✓ 弄坏开关 single-worker-threads-field 下「配的与起的线程分两格记」那一格红（只剩一格 worker_threads=）
  ✓ 弄坏开关 judging-digest-without-dispatch 下「分派表两项对调」那一格红（摘要没变）
  ✓ 弄坏开关 whole-module-in-judging-digest 下「实验准入加一行注释」那一格红（摘要变了）
```

lib_heavy_tests.py 与 heavy-test-guard.sh 照它们原有的写法由外面设开关（与 HEAVY_TEST_GUARD_DISABLE_CHECK 同一种），每个开关各跑一遍、都退 1，转红的格原样（开发副本上跑，副本与仓里逐字节相同，`cmp` 过）：

```
LIB_HEAVY_TESTS_BREAK=launcher-whole-word-options lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：strace -fo 文件（短选项合写） 应当是 full-cargo，实际 None
  ✗ lib_heavy_tests 自检：flock -xw 10 锁文件（短选项合写） 应当是 full-cargo，实际 None
  ✗ lib_heavy_tests 自检：systemd-run --user --scope -qu 名字（短选项合写） 应当是 full-cargo，实际 None
  ✗ lib_heavy_tests 自检：/usr/bin/time -ao 文件（短选项合写） 应当是 full-cargo，实际 None
LIB_HEAVY_TESTS_BREAK=launcher-drops-setenv lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：systemd-run -E 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：systemd-run --setenv= 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：systemd-run -qE 合写设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
LIB_HEAVY_TESTS_BREAK=list-by-presence lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：点名崩溃枚举用例、-- --include-ignored --skip --list（--list 是 --skip 的值） 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：点名崩溃枚举用例、-- --include-ignored --exact the_full_case --logfile --list（--list 是日志文件名） 应当是 crash-case-
  ✗ lib_heavy_tests 自检：崩溃枚举用例的测试二进制 --include-ignored --skip --list（--list 是 --skip 的值） 应当是 crash-case-binary，实际 Non
  ✗ lib_heavy_tests 自检：层 0 测试二进制 --logfile --list（--list 是日志文件名） 应当是 layer0-binary，实际 None
LIB_HEAVY_TESTS_BREAK=undecided-ignore-allowed lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：点名用例是宏生成的（找不到字面的 fn <名>(）登记目标、不带 --ignored：判不出按没标算 应当是 crash-case-c
  ✗ lib_heavy_tests 自检：导入不了 admission.py：点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 按没标算 应当是 crash-case-carg
LIB_HEAVY_TESTS_BREAK=runner-configuration-by-regex-only lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：--config 里带引号的键 target.<三元组>."runner"、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：--config 里点号两边带空白的 target . <三元组> . runner 应当是 crash-case-cargo，实际 None
ADMISSION_BREAK=first-definition-only lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：点名同名函数 cfg 二选一、一份不标的登记目标、不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：点名子模块里同名标 ignore 写在前面、顶层那一份不标的登记目标、不带 --ignored 应当是 crash-case-cargo，实
ADMISSION_BREAK=ignore-attribute-without-space lib_heavy_tests.py --selftest → 退 1
  ✗ lib_heavy_tests 自检：点名用例函数标的是 # [ignore]（# 与 [ 之间有空格）的登记目标、不带 --ignored 应当是 None，实际 crash-case-ca
```

````

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:108-151`（整段抄，未转述）**

````markdown
## P1–P7、P9、P10：攻方探针改前判错 → 改后判对

改前：攻方两个探针（`probe_k1_hook.py`、`probe_k1_manifest.py`）原样拷到仓的一份拷贝（`before-repo/`，被判的四份与备份逐字节相同，`cmp` 过；只把 `probe_common.py` 的 SCRATCH_ROOT 指到我的草稿目录）上跑；改后：装进仓之后再拷一份（`rerun-repo/`，带 .git、不带 target），跑攻方的 `rerun.sh`。只列攻击格（ATTACK / OVER），逐格原样的判定与细节截到判定那一段：

| 格 | 改法 | 改前（仓里被判的那一份，before-k1-*.log） | 改后（仓的临时拷贝，rerun-out/k1-*.log） |
|---|---|---|---|
| L2 | P1 | BROKEN：退 0 | holds：退 2 |
| L4 | P1 | BROKEN：退 0 | holds：退 2 |
| L6 | P1 | BROKEN：退 0 | holds：退 2 |
| L7 | P1 | BROKEN：退 0 | holds：退 2 |
| L12 | 不归这一轮 | BROKEN：退 0 | BROKEN：退 0 |
| L13 | P1 | BROKEN：退 0 | holds：退 2 |
| R2 | P2 | BROKEN：退 0 | holds：退 2 |
| R3 | P2 | BROKEN：退 0 | holds：退 2 |
| R5 | 走不到 | BROKEN：退 0 | BROKEN：退 0 |
| R7 | P10 | BROKEN：退 0 | holds：退 2 |
| R8 |  | holds：退 2 | holds：退 2 |
| T2 | P3 | BROKEN：退 0 | holds：退 2 |
| T3 | P3 | BROKEN：退 0 | holds：退 2 |
| T4 | P3 | BROKEN：退 0 | holds：退 2 |
| V3（闸） | P4 | BROKEN：闸退 0 | holds：闸退 2 |
| V3（自查） | P4 | BROKEN：自查退 0：crash-case:own | holds：自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函 |
| V4（闸） | P4 | BROKEN：闸退 0 | holds：闸退 2 |
| V4（自查） | P4 | BROKEN：自查退 0：crash-case:own | holds：自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函 |
| V5（闸） | P5 | BROKEN：闸退 0 | holds：闸退 2 |
| V5（自查） | P5 | holds：自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函 | holds：自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函 |
| V7（闸） | P4b | BROKEN：闸退 2 | holds：闸退 0 |
| V7（自查） | P4b | BROKEN：自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函 | holds：自查退 0：crash-case:own |
| M1 | P6 | BROKEN：改前 730d1b23600cc67d… 改后 730d1b23600cc67d…（文件数 / 减去数 改前 4/3 改后 4/3） | holds：改前 72476b461264130f… 改后 e6f8fc846ca3035d…（文件数 / 减去数 改前 7/0 改后 7/0） |
| M2 | P6 | BROKEN：改前 07253dee10dc690a… 改后 07253dee10dc690a…（文件数 / 减去数 改前 4/3 改后 4/3） | holds：改前 98952592a29d09e1… 改后 e1f2d9b20bd8805d…（文件数 / 减去数 改前 7/0 改后 7/0） |
| M3 | 已声明（P8 不做） | BROKEN：改前 d5ad98ece8532053… 改后 d5ad98ece8532053…（文件数 / 减去数 改前 8/3 改后 8/3） | BROKEN：改前 d8dbd079ea31b394… 改后 d8dbd079ea31b394…（文件数 / 减去数 改前 10/1 改后 10/1） |
| M4 | P7 | BROKEN：改前 f4bcff450e202357… 改后 f4bcff450e202357…（文件数 / 减去数 改前 5/2 改后 5/2） | holds：改前 e8a3da52c6d8b3d3… 改后 a641f9cb37345a04…（文件数 / 减去数 改前 7/0 改后 7/0） |
| M5 | P7 | BROKEN：改前 43f35d7b9292b91e… 改后 43f35d7b9292b91e…（文件数 / 减去数 改前 5/2 改后 5/2） | holds：改前 42bcf6f1a3dff2cb… 改后 958f5127e71ffe6e…（文件数 / 减去数 改前 7/0 改后 7/0） |
| G1 | P9 | BROKEN：改前 6b386f2d5cce25b7… 改后 6b386f2d5cce25b7…（文件数 / 减去数 改前 4/3 改后 4/3） | holds：改前 361f5a644ec94a55… 改后 f0ed9b65caeddfc7…（文件数 / 减去数 改前 4/3 改后 4/3） |
| G2 | P9 | BROKEN：改前 f62727a21db0898f… 改后 f62727a21db0898f…（文件数 / 减去数 改前 4/3 改后 4/3） | holds：改前 d42e0c84514605d6… 改后 a96ddf5f7d35c1d3…（文件数 / 减去数 改前 4/3 改后 4/3） |
| G3 | 观测不到（写进文件头） | BROKEN：改前 4c4bfde2419e64ad… 改后 4c4bfde2419e64ad…（文件数 / 减去数 改前 4/3 改后 4/3） | BROKEN：改前 830c948ec01a32ac… 改后 830c948ec01a32ac…（文件数 / 减去数 改前 4/3 改后 4/3） |

改前两份汇总行（`before-k1-hook.log`、`before-k1-manifest.log` 末行，与攻方入库的 `outputs/k1-*.log` 数相同）：

```
SUMMARY K1 闸与属性解析: cells=44 attack_cells_broken=18 control_cells_broken=0
SUMMARY K1 排除规则与 runner 指纹: cells=12 attack_cells_broken=8 control_cells_broken=0
```

````

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:152-206`（整段抄，未转述）**

````markdown
## rerun.sh 复跑（仓的临时拷贝）

命令：在 `/tmp/claude-1000/gate-batch-m2-r2-fixes/rerun-repo` 里 `bash research/prompts/defs-gatebatch-m2-r2-opus-model/rerun.sh <rerun-out> <rerun-scratch>`；拷贝里改了两处：`probe_common.py` 的 SCRATCH_ROOT 指到我的草稿目录，`verify_semantics.sh` 的 `capped.sh 6` 改成 `capped.sh 4`（派发的线程上限）。驱动的原样输出：

```
k1-hook rc=1
k1-manifest rc=1
k3-judge rc=0
k4-cost rc=0
semantics rc=0
cp: cannot stat '/tmp/claude-1000/gate-batch-m2-r2-fixes/rerun-scratch/fixcopy/admission.py.patch': No such file or directory
cp: cannot stat '/tmp/claude-1000/gate-batch-m2-r2-fixes/rerun-scratch/fixcopy/lib_heavy_tests.py.patch': No such file or directory
fix-k1-hook rc=1
fix-k1-manifest rc=1
done: 24 个文件
rerun rc=0
```

各探针汇总行（`rerun-out/*.log`，前两行是这一批的代码，与规格要的对上：剩的正是 L12、R5 与 M3、G3）：

```
k1-hook: SUMMARY K1 闸与属性解析: cells=44 attack_cells_broken=2 control_cells_broken=0
k1-manifest: SUMMARY K1 排除规则与 runner 指纹: cells=12 attack_cells_broken=2 control_cells_broken=0
k3-judge: SUMMARY K3 两条登记行的判法: cells=12 attack_cells_broken=0 control_cells_broken=0
k4-cost: SUMMARY K4 窄的判法摘要: cells=8 attack_cells_broken=0 control_cells_broken=0
ATTACK	BROKEN	M3 gen 的 build.rs 只写 mod scan;，按目录读 pkg 的 tests/ 的那几行在 scan.rs 里 
ATTACK	BROKEN	G3 runner = <仓>/tools/r.sh，它 source 的 r-lib.sh 改了 ⇒ 改 tools/r-lib.sh 指纹应
ATTACK	BROKEN	L12 env -S '<整条命令>'（lib_shell_words 的前缀，不在这一批改动里） ⇒ 应当
ATTACK	BROKEN	R5 --config alias."xt"=…（TOML 带引号的键，同一个别名） ⇒ 应当拒	退 0；「
```

fix-* 那一半不作数：`make_fix_copy.py` 按改前的字面打补丁，在这一批的代码上一处都打不上（`fix-build.log`，两个 `.patch` 没生成，驱动里两行 `cp: cannot stat`），fix-k1-* 跑的是没打补丁的拷贝，也就是这一批的代码，数与上面相同。拷贝里 `admission.py --selftest` 退 0（`selftest-admission.log` 末行：  ✓ admission.py 自证通过：207 …），`verify_semantics.sh` 退 0。

heavy-test-guard.sh --selftest 在同样的开关下各跑一遍，都退 1，这一批加的闸那一侧的格转红（另一行「lib_heavy_tests.py 的自检 应当是 0，实际 1」是它顺带跑的 lib 自证）：

```
LIB_HEAVY_TESTS_BREAK=launcher-whole-word-options → 退 1
  ✗ 自检：实现员内存包装里 strace -fo 包一层跑崩溃枚举用例 应当是 2，实际 0
  ✗ 自检：实现员不经内存包装 strace -fo 包一层跑崩溃枚举用例 应当是 2，实际 0
  ✗ 自检：实现员内存包装里 flock -xw 10 包一层跑崩溃枚举用例 应当是 2，实际 0
LIB_HEAVY_TESTS_BREAK=launcher-drops-setenv → 退 1
  ✗ 自检：实现员内存包装里 systemd-run -E 设 runner、不带 --ignored 应当是 2，实际 0
LIB_HEAVY_TESTS_BREAK=list-by-presence → 退 1
  ✗ 自检：实现员内存包装里 -- --include-ignored --skip --list（--list 是 --skip 的值） 应当是 2，实际 0
  ✗ 自检：实现员内存包装里直接执行崩溃枚举用例的测试二进制 --include-ignored --skip --list 应当是 2，实际 0
LIB_HEAVY_TESTS_BREAK=undecided-ignore-allowed → 退 1
  ✗ 自检：实现员内存包装里跑用例是宏生成的登记目标（判不出，按没标算） 应当是 2，实际 0
LIB_HEAVY_TESTS_BREAK=runner-configuration-by-regex-only → 退 1
  ✗ 自检：实现员内存包装里 --config 带引号的 runner 键、不带 --ignored 应当是 2，实际 0
ADMISSION_BREAK=first-definition-only → 退 1
  ✗ 自检：实现员内存包装里跑同名函数 cfg 二选一、一份不标的登记目标（不带 --ignored） 应当是 2，实际 0
ADMISSION_BREAK=ignore-attribute-without-space → 退 1
  ✗ 自检：实现员内存包装里跑用例函数标 # [ignore] 的登记目标（算标了，不带 --ignored 放行） 应当是 0，实际 2
```

````

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:207-222`（整段抄，未转述）**

```markdown
## 门禁与 lint（仓里现文件，2026-09-26 UTC）

| 命令 | 退出码 | 末行 / 判红的那几行 |
|---|---|---|
| `bash research/scripts/stage-run-or-skip.sh --selftest`（套完 54 号替换之后） | 0 | `✓ stage-run-or-skip 自证通过：4 格（退 0 往下跑、退 1 退 77、其余判红）` |
| `bash .claude/gate.d/47-research-script-selftests.sh` | 1 | 只有一条没过：`✗ python3 research/scripts/check-segment-registry.py --selftest 没过（退出码 1）：`，原因 `✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与`；admission.py 与 stage-run-or-skip.sh 的自证在这一趟里过了（没被列出） |
| `bash .claude/gate.d/62-stage-owners.sh` | 0 | `✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| `bash .claude/gate.d/63-agent-write-scope.sh` | 0 | `✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸…` |
| `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` | 1 | `✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 516，跳过 0）`；点名的是 O1–O3 的 not-numbers、F23/F24/M32/M34/U11/U13 没登记位、`.claude/kb/experiments-history.md` 与 `.claude/kb/experiments/158-择根与修复四岔路.md` 的引用，这一批的五份文件里 `grep -nwE 'O1\|O2\|O3\|F23\|F24\|M32\|M34\|U11\|U13'` 零命中 |
| `GATE_LINT_DIR=.claude/gate.d bash .claude/singlefs-ai-sop/scripts/gate-lint.sh` | 0 | `✓ 门禁自检通过：84 个脚本（.sh 与 .py）、304 条拒绝都带了出路` |
| `GATE_LINT_DIR=.claude/hooks …gate-lint.sh` | 0 | `✓ 门禁自检通过：12 个脚本（.sh 与 .py）、18 条拒绝都带了出路` |
| `SHELL_LINT_DIR=.claude/gate.d …shell-lint.sh` | 0 | `✓ shell 纪律检查通过（共 76 个脚本）` |
| `SHELL_LINT_DIR=.claude/hooks …shell-lint.sh` | 0 | `✓ shell 纪律检查通过（共 9 个脚本）` |

47 号与 doc-lint 红的两处都在别的会话没提交的改动里：`git status --short` 现跑，`research/scripts/check-segment-registry.py`、`.claude/kb/layout/01-first-txn.md`、`.claude/kb/layout/02-second-txn.md` 都是 M、另有 7 份 `research/results/e142-first-txn-dry-run-2026-09-2*-r1*.out` 未跟踪；这一批没碰它们，照规格不修。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:223-230`（整段抄，未转述）**

```markdown
## 与实分一交来的四份 diff 相邻的地方（`/tmp/claude-1000/impl-shard-1/deliver/`，这一次没套）

重打时按新代码手工对，下面几处上下文已经变了，原 hunk 不会直接套上：
- `54-layer0-replay.sh`：文件头「断点续跑」那三行（它在后面加双机分片四行）我改写了；`run_crash_case` 整个改写成 `run_crash_case <日志> <命令…>`，前面多了 `read_crash_case_command`（它要在 `run_crash_case` 后面加 `run_crash_case_in_two_shards`）；循环里「开跑」那一句与 `if ! run_crash_case …` 那一段我改了（它把这一段换成分片 / 单机两支）。它在「--full 开跑」那一句之后加的分片判定那几行，上下文我没动。
- `admission.py`：文件头第三列说明（test= 那两行我改成四行，它改「认四种 → 五种」与 threads= 那一行）、子命令表（crash-case-manifest / crash-case-record 两条我改了，它在 crash-case-marker-path 之后加 crash-case-shardable）、弄坏开关那一段（同一行我续了 8 个开关，它续 threads-ignore-shards）、「崩溃枚举用例」一节开头的注释（我改了 ① 与 ② ③）、自证成功那一句（同一行两边都续）、`FAKE_CARGO_FOR_STAGE` 的 printf 行（我加了第 9 列线程数）、分派表 COMMANDS 与 main 的用法句（两边都加一项）。`parse_crash_case`、`judge_worker_threads` 两边都没冲突：它改的那几处我没动，但我的判法摘要自证有一格改 `parse_crash_case` 里 `if prefix not in count_lines]` 那一串，它要保证那一串在函数里仍恰好一处。
- `lib_heavy_tests.py`：它加的 KNOWN_SCRIPT_LOCATIONS 一项、classify 里 layer0-shard-run.sh 那一支、自证里三格与一个按名字判的格，所在的上下文我没改，按行号偏移应能套上（没试）。
- `stage-inputs.tsv`：我没改。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:231-238`（整段抄，未转述）**

```markdown
## 看到但没做（交主 agent 定）

- `strace -E VAR=VAL` / `--env=VAR=VAL` 同样给里面那条命令设环境变量（`strace -h` 现跑第 15 行 `  -E VAR=VAL, --env=VAR=VAL`），与 R2、R3 同一类；判决的 P2 只点名 systemd-run，我只做了 systemd-run（`LAUNCHER_ENVIRONMENT_OPTIONS` 一张表，加一行就罩上，没加）。闸对 `strace -E CARGO_TARGET_…_RUNNER=… cargo test -p … --test <登记目标>` 现在放行（推的，没喂闸）。
- `.claude/gate.d/stage-inputs.tsv` 第 16 行 `#     「崩溃枚举用例」一节），再加判它的 54 号、准入模块 admission.py、工具链、构建环境与这一行本身。` 与第 26 行注释里「（它也进每条用例的指纹）」，D2 之后是过时的话（进的是判法摘要，54 号不进）；规格不许我改这份，留给补 c561 那一行字段的那一次一并改。
- 实分一重打时：`crash-case-shardable` 定「这条用例怎么跑」，要加进 `CRASH_CASE_JUDGING_SUBCOMMANDS`；双机驱动 `research/scripts/layer0-shard-run.sh` 自己起 cargo、不经 `crash-case-command`，D2 之下它不在任何一条用例的指纹里（与 54 号剩下的那一份同形），它改了旧标记照样作数。
- `crash-case-command` 起用例时没清调用方的 `SINGLEFS_LAYER0_SHARD`（攻方 K3 记过，假红方向）；没做，不在这一批改法里。
- `.claude/rules/implementation-workflow.md` 第 89 行「这一趟跑了至少两片…判红」那一句对 c561 不成立（K3 c561）；判决只要标记字段的字面，规则那一句没动。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:239-246`（整段抄，未转述）**

```markdown
## 推翻条件

- P1–P3、P5、P10：真仓里有这一批自证与探针之外、闸照样放行的写法（例：`strace -E` 那一类），或看门狗在进程那一层认不出这几种（看门狗与闸共用 `command_under_launcher` / `classify`，没喂看门狗，推的）。
- P4、P4b：闸与自查之外另有一道按编译结果判 ignore 的检查，或 rustc 不认 `# [ignore]`（攻方 `outputs/semantics.log` 量过它认）。
- P6、P7、P9：include 的参数以字面量开头却读了别的文件（改不了的：`include_str!("…")` 字面量读的就是那一份，已由「按整词点名」那一条罩着）；runner 参数里的相对路径其实按测试二进制的工作目录（包目录）解，而不是 `.cargo` 那一层（我按攻方模型取的后者，推的，没量 cargo 怎么交参数给 runner 进程的当前目录）。
- D2：闭包之外的某个定义其实决定日志判不判绿（`getattr` / 字符串拼函数名这一类，文件头已声明），或 54 号 `--full` 真跑时 `crash-case-command` 交的命令与原来 54 号自己拼的那一条不同（逐词对过：`env -u SINGLEFS_LAYER0_START_OVER`（或 `=1`）、进度目录、输入指纹，再加 `SINGLEFS_LAYER0_THREADS=<n>`，然后 `cargo test --release -p … --test … -- --include-ignored --exact … --nocapture`；多出来的只有显式传线程数这一项，原来是 54 号 export 进环境）。
- 标记字段：c561 那种没登记 threads= 的用例，`started_worker_threads=` 记「读不到」；若有人按旧格读 `worker_threads=`（我在仓里 `.claude`、`research/scripts`、`records` 下 grep 过，只有 54 号与 admission.py 用它），会读不到。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:247-255`（整段抄，未转述）**

```markdown
## 没做什么

- 重型测试一条都没跑：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的目标、全量 `cargo test` 都没起；54 号只在 `admission.py --selftest` 的假 cargo 场景里跑。真 `--full` 第一次跑 `crash-case-command` 交的命令，是提交时崩溃验证员那一趟（判决 K2：这一轮改法同样只在模型上证过）。
- `rerun.sh` 里 `verify_semantics.sh` 在小 crate 上编过、跑过（经 `capped.sh 4` 与 `run-with-memory-cap.sh`），这是攻方探针本身的一步，不是这一批的验证链。
- 没改 `.claude/gate.d/stage-inputs.tsv`、别的 gate.d 阶段、`crates/`、`.claude/kb/`；没套实分一的四份 diff；P8 没做；47 号与 doc-lint 的红不是这一批的，没修。
- 没做 git 写操作，没提交、没暂存。
- 看门狗（`research/scripts/agent-watch.py`）没喂这几种命令：它与闸共用同一个 `classify` / `command_under_launcher`，认不认得出是推的。
- 登记给我这一类的门禁阶段：派发没点名 agent 类型，`stage-owners.tsv` 里按「general-purpose」没有登记行，没跑额外阶段。

```

**出处 `research/prompts/defs-gatebatch-m2-r2-fixes-report.md:256-263`（整段抄，未转述）**

```markdown
## 草稿目录与清理

- 删了（交回之前，`du -sh` 记的大小）：`/tmp/claude-1000/gate-batch-m2-r2-fixes/dev`（172M，开发用的仓拷贝）、`before-repo`（172M，改前探针用的仓拷贝）、`rerun-repo`（360M，rerun.sh 用的仓拷贝，含 .git）、`rerun-scratch`（205M，含 `semantics/crate/target` 编译目录）、`probe-scratch-before`（3.9M，探针小仓）；`mut54`（开发副本的一份拷贝）判完当场删了。
- 留着：`backup/`（五份改前原件，`cp -p`）、`my-changes.diff`、各份日志（`before-k1-*.log`、`rerun-out/`、`final-*.log`、`lib-break-*.log`、`guard-*break-*.log`、`lint-*.log`、`demo-d2-marker.log`、`mut54-selftest.log`）、`demo_d2_marker.py`、`54-before-d2.sh`（套完 singlefs-39 替换、D2 之前的 54 号，演示用）。
- 攻方的草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r2-opus/` 不是我建的，没动（我的探针与复跑都把 SCRATCH_ROOT 指到了自己的草稿目录）。

交回时刻：2026-09-26T17:06Z（JST 2026-09-27T02:06）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:1-4`（整段抄，未转述）**

```markdown
# G2 交回：门禁批第二轮判决的余项与实分一接入（tooling-writer）

写于 2026-09-27 JST 08:4x（UTC 2026-09-26 23:4x）。规格 `/tmp/claude-1000/gate-batch-m2-g2/spec.md`；途中收到主 agent 四条追加（见「途中收到的追加」一节）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:5-12`（整段抄，未转述）**

```markdown
## 结论

- 规格第 1–9 条都做完，第 9 条（54 号）最后套。途中追加的四件（admission 自证临时仓链 `.claude/singlefs-ai-sop` 与 `.claude/scripts`、仓根模板改放草稿目录、新脚本改 source 垫片并写准入声明、驱动自证默认改假 cargo）也做完。
- 出口各项的原样末行都绿，只有两处红不在这一批：47 号红在 `research/scripts/check-segment-registry.py --selftest`；doc-lint 红在 O1–O3、F23 等编号与两份 kb 文件（这一批改过的文件里 grep 零命中）。
- 仓根 `layer0-shard.env.example` 不在我的写范围，写闸拒了。成品在 `/tmp/claude-1000/gate-batch-m2-g2/layer0-shard.env.example`，sha256 `72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4`，与 deliver 那份逐字节相同（没改）；由主 agent 拷进仓根。
- 第 4 条选「把驱动脚本按内容算进输入」，理由见第 4 条那一段。
- 推翻条件：真 `--full` 在分片开着时，驱动脚本在 HEAD + 暂存区的临时树里算出的指纹与 54 号给的对不上（这一批只在假 cargo 与临时仓上证过）；或者别的会话再改 `stage-must-run.sh` 这类 helper 的 source 写法，admission 自证的临时仓又缺文件。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:13-32`（整段抄，未转述）**

````markdown
## 改过的文件与 git diff --stat

逐文件相对开工时备份（`backup/`）的增删行数，由 `diff backup/<名> <仓里的文件>` 数 `>` / `<` 得出：admission.py +339 −23，stage-inputs.tsv +10 −7，lib_heavy_tests.py +34 −7，implementation-workflow.md +1 −1，54 号 +46 −4。54 号的数里含 singlefs-39 在 23:22Z 做的第 63 行替换（source 垫片，+1 −1），那一行不是我改的。新建三份：layer0-shard-run.sh 280 行，layer0-shard-run-selftest.sh 207 行，layer0-shard-configuration-check.sh 98 行，权限 775。八份都与测过的 dev 副本逐字节相同（`cmp` 现核）。合在一起的 diff：`/tmp/claude-1000/gate-batch-m2-g2/g2-changes.diff`（1498 行）。

`git diff --stat -- <这几份>` 原样。它比的是 HEAD，所以混着 G1 与别的会话没提交的改动；admission.py 与三份新脚本没进 git，不在 stat 里：

```
 .claude/gate.d/54-layer0-replay.sh       | 617 +++++++++++++------------
 .claude/gate.d/stage-inputs.tsv          |  45 +-
 .claude/hooks/lib_heavy_tests.py         | 745 +++++++++++++++++++++++++++++--
 .claude/rules/implementation-workflow.md |  16 +-
 4 files changed, 1074 insertions(+), 349 deletions(-)
?? research/scripts/admission.py
?? research/scripts/layer0-shard-configuration-check.sh
?? research/scripts/layer0-shard-run-selftest.sh
?? research/scripts/layer0-shard-run.sh
```

仓里的改动都是定点替换：开发在草稿里的仓副本上做，再由 `apply_hunks.py` 把每个 hunk 交给 `research/scripts/replace-once.py` 套进仓里。每个 hunk 在仓里恰好命中一次，套完回读与 dev 相同。日志在 `logs/repo-apply*-*.log`。三份新脚本用 Write 新建，之后的改动同样走 replace-once。第一轮 hunk 数：admission 35、lib 12、54 号 4、tsv 3、规则 1。第二轮：admission 1、驱动 1、自证 8、配置判法 1。

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:33-155`（整段抄，未转述）**

````markdown
## 各条改法与证红

### 第 1 条：实分一三份 diff 重打（admission.py、stage-inputs.tsv、lib_heavy_tests.py）
- `patch --dry-run` 在今天的文件上：admission.py 19 个 hunk 有 5 个打不上（文件头第三列、弄坏开关表、自证成功句、线程格之后的 merge 格、main 的用法句），全在草稿副本里照 diff 的意思按新代码重写。lib 的 4 个全套上。tsv 那一份按第 7 条合并着改。
- 按 G1 之后的接口改的地方：merge 格接在 G1 新加的 `started_worker_threads=` 两格之后；54 号分片格里假驱动的参数照旧；`CrashCase` 多一个成员 `is_shardable_across_machines`。
- 原有自证：`admission.py --selftest` 实分一的格全在，现 232 格全过（原样末行见「出口各项」）；`lib_heavy_tests.py --selftest` 141 种全过，其中实分一的 4 格也在。

### 第 2 条：三份新脚本（来源 deliver，照 G1 之后的 admission.py 核过）
- deliver 版调 `crash-case-manifest` 用的是 `--extra-file <54 号> --extra-file <admission.py>`。G1 之后 54 号给的是 `--judging-digest --toolchain --build-environment`，照原样调，两边指纹永远对不上。驱动本机、第二台、跑完三处都改成 G1 的参数，自证里算指纹那一处也跟着改。
- deliver 版调 `crash-case-record` 用 `--threads-text`，G1 已经删掉这个选项，改成 `--machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin"`。
- 追加指示之后：三份都改成 `source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"`（垫片）。驱动的 `--selftest` 分支挪到 preflight 那一行之后，同时删掉 `run-condition: check …configuration-check.sh`：preflight 的 check 读不到参数，留着它的话，没有配置时 `--selftest` 会被拒 78。配置由驱动开跑后第一步的配置判法拒，退 1，原因与出路照旧，自证 ⑤ 相应改成期望退 1。
- 三份相对 deliver 的差：`driver-vs-deliver.diff`（70 行）、`selftest-vs-deliver.diff`（175 行）、`check-vs-deliver.diff`（14 行）。

### 第 3 条：`crash-case-shardable` 进 `CRASH_CASE_JUDGING_SUBCOMMANDS`
- 改法：admission.py 第 208–209 行把它加进元组。判法摘要（第 1894–1895 行）从分派表取这几项；弄坏开关 `shardable-outside-judging-digest` 把它摘掉。
- 该红的输入：把 `command_crash_case_shardable` 里「没登记」那一支的 `return 1` 改成 `return 0`，看判法摘要变不变（`probe_g2.py`）。
- 改前（G1 代码加实分一 hunk）→ 改后，原样：
```
改前	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要不变
改后	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要变
```
- 自证：`JUDGING_DIGEST_VARIANTS` 加一格（第 3416 行 `SHARDABLE_ANSWER_UNREGISTERED`）。开关格在第 3471 行。在草稿副本上用 `ADMISSION_BREAK=shardable-outside-judging-digest` 跑：
```
  ✗ 判法摘要：单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了 ⇒ 摘要变：摘要没变
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 4 条：双机驱动进两条层 0 流用例的指纹（选「按内容算进输入」）
- 为什么不走 `crash-case-command`：驱动决定判绿的不只是起 cargo 那一条命令。两片的退出码与 `LAYER0_SHARD mode=run` 行、账本拷回与数份、merge 那一趟的环境、merge 行在不在，都是它判的。改成经 `crash-case-command` 起，进摘要的只有命令，这些判法照样在指纹之外；`crash-case-command` 还得另收第二台的进度目录与线程数。按内容算进去，整份驱动都在指纹里。驱动 eval 配置判法打出来的赋值，所以配置判法 `layer0-shard-configuration-check.sh` 也一起进。
- 改法：admission.py 第 187 行 `SHARD_DRIVER_FILES`；第 1551 行 `shard_driver_lines`（登记了 `shard=across-machines` 的，这两份按内容排在登记行之前，不在的记「找不到 <路径>」）；第 1547 行接进清单。stage-inputs.tsv 第 28 行 54 号那一行的路径加上这两份：只改驱动时，快档不会在「范围那一问」退 77，照样核标记。代价：分片关着、照单机跑时驱动也在指纹里，改它这两条会多跑一趟（文件头「管不到的」写明，接受）。
- 该红的输入：临时 git 仓里放一条 `shard=across-machines` 的用例，先放进驱动脚本，再改它，看指纹。原样：
```
改前	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹不变；改驱动脚本 ⇒ 指纹不变（7ee48c0c812a → 7ee48c0c812a）
改后	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹变；改驱动脚本 ⇒ 指纹变（03e9308a66e2 → 3f3cc1939518）
```
- 自证：`run_layer0_stage_shard_cells` 开头两格（第 3824 行起）。`ADMISSION_BREAK=shard-driver-outside-manifest` 下：
```
  ✗ 输入清单：放进双机分片的驱动脚本与配置判法 ⇒ 登记了 shard=across-machines 的 stream-a 指纹变、没登记的 stream-b 不变：stream-a 6fd93cbbda19bb0d → 6fd93cbbda19bb0d，stream-b 96ad622302455cf8 → 96ad622302455cf8
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 5 条：`strace -E VAR=VAL` / `--env=VAR=VAL`
- `strace -h` 现跑，第 15 行原样 `  -E VAR=VAL, --env=VAR=VAL`，第 17 行 `  -E VAR, --env=VAR`（只清变量）。
- 改法：lib_heavy_tests.py 第 606 行 `LAUNCHER_ENVIRONMENT_OPTIONS` 加 `"strace": {"-E", "--env"}`；第 597 行 strace 带值的选项加 `--env`；第 651 行接弄坏开关 `strace-drops-env`。`-E VAR` 不带 `=` 的不带进里面那条命令（沿用原有的 `"=" in value` 判法）。文件头第 15、40、54 行跟着改。
- 改前（仓里原来那份 lib）→ 改后，`probe_strace.py` 在 lib 自己的样本工作区里判，原样：
```
改前	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改前	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改后	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
```
  改前 `--env <值>` 分开写那一种碰巧判成重型：`--env` 不在表里，被当成不带值的选项，赋值那个词被当成要起的命令剥过去。这是巧合，不是判对了。
- 自证：第 967 行起加 5 格（最后一格 `-E VAR` 只清变量，判不重型）。`LIB_HEAVY_TESTS_BREAK=strace-drops-env` 下，lib 自检与 heavy-test-guard.sh 自检都红：
```
  ✗ lib_heavy_tests 自检：strace -E 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env= 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env 值另起一个词、设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace -fE 合写设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ 自检：lib_heavy_tests.py 的自检 应当是 0，实际 1
```
  heavy-test-guard.sh 本身没改。

### 第 6 条：`crash-case-command` 起用例时清掉调用方的 `SINGLEFS_LAYER0_SHARD`
- 改法：admission.py 第 1856–1857 行，`env -u SINGLEFS_LAYER0_SHARD` 放在第一个 `NAME=VALUE` 之前。第一版写在 `SINGLEFS_LAYER0_START_OVER=1` 之后，54 号 `--start-over` 那一格当场红了：env 把后面的 `-u` 当成要起的命令。
- 该红的输入：调用方设着 `SINGLEFS_LAYER0_SHARD=0/2`，照交出的命令起一趟，把 cargo 往后换成打那个变量的 sh。原样：
```
改前	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「0/2」（命令前段：env -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「unset」（命令前段：env -u SINGLEFS_LAYER0_SHARD -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	带 --start-over 时起的用例看到「unset/1」、退 0
```
  （命令前段里的临时路径我换成了 `…`；整行原文在 `logs/probe-before.log` 与 `logs/probe-after.log`。）
- 自证：第 3804、3813 行两个函数。`ADMISSION_BREAK=keep-caller-shard-switch` 下：
```
  ✗ crash-case-command：调用方环境里设着 SINGLEFS_LAYER0_SHARD=0/2 ⇒ 起的用例看不到它：用例看到的是「0/2」
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 7 条：stage-inputs.tsv
- 第 16–20 行（原第 16–18 行）：「再加判它的 54 号、准入模块 admission.py」改成现状：进指纹的是准入模块里崩溃枚举用例的判法摘要；登记了 `shard=across-machines` 的另按内容加驱动与配置判法；54 号不进。第三列的说明加上 `shard=across-machines`。
- 第 28 行（54 号那一行）：路径加 `research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh`（第 4 条）；注释「（它也进每条用例的指纹）」改成「进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份」，另加一句驱动与配置判法。
- 第 36、37 行：实分一的 `shard=across-machines` 与注释。
- 第 39 行 c561：第三列与注释逐字照实审 B3b 报告第 116 行起那一段（`exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL`）。我核过用例确实打这两行：`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs` 第 656 行打计数行，第 675 行打 `LAYER0_PARALLEL_FINISHED` 同形行。
- 第 40 行新加 `crash-case:crash-injection-fast-tier`：原文是实审 B3c-1 报告第 85 行整行抽出来追加的（`awk 'NR==85'`），追加之后与抽出来的那行 `cmp` 相同，4 列。用例在 `second_transaction_supplement_three_crash_injection.rs` 第 131 行标 `#[ignore]`，第 132 行是用例函数。
- 该红的输入：拿改前 / 改后的表判合成日志（`probe_g2.py`）。原样：
```
改前	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改前	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 登记表里读不出这一条（.claude/gate.d/stage-inputs.tsv 里崩溃枚举用例 crash-case:crash-injection-fast-tier 要恰好一行，实际 0 行）
改后	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改后	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 判红
```
- 自证：admission.py 第 3783 行 `run_real_crash_case_row_cells`，拿真仓登记表的这两行判 5 格。登记表是数据，没配弄坏开关；红的样子就是上面「改前」那几行，谁把这两行改回去，这 5 格就红。

### 第 8 条：`.claude/rules/implementation-workflow.md`（第二种活，改的规则）
- 第 58 行「崩溃枚举用例」一条末尾加一句：「双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。」这与 lib 第 706 行的判法一致：`return None if "--selftest" in arguments else heavy_test("crash-case-cargo", …)`。
- 同一行 runner 那一句「`systemd-run -E` / `--setenv`」后面加「、`strace -E` / `--env`」，跟第 5 条的闸对齐（规格没点名这一处，但规则要与闸逐类相同）。
- 改过的定义、共用约束与规则只有这一份，交主 agent 开定义三方（门禁 72 号）。

### 第 9 条：54 号（最后套）
- 动手前读了一遍今天的样子，与备份相同（singlefs-39 的准入声明与 preflight 调用已在）。实分一 4 个 hunk 有 2 个打不上（文件头、循环那一段），照 G1 的 `read_crash_case_command` / `run_crash_case <日志> <命令…>` 重写。
- 改后字面：第 25–29 行文件头的双机分片一段；第 237–244 行 `run_crash_case_in_two_shards`；第 350–358 行判开不开（配置判法判得过才开，打一行开或关）；第 394–418 行逐条用例：先 `crash-case-shardable` 定走哪一路，分片的 `case_threads_note` 写明第二台取它自己的核数，退非 0 时分片那一支有自己的 ✗ 与出路。merge 那一趟的日志照单机的判法判、写同一格（线程逐片判在 admission.py 第 1608、1629 行）。
- gate-lint 在第 413 行报过一次红：出路句里写了「✗」字，被当成下一处拒绝。改成「驱动脚本输出里判红的那一句」，重跑转绿（见「出口各项」）。
- 自证：admission.py 里 54 号那几格与实分一的两格分片格（第 3824 行起）全过，含「双机分片：关」「开：stream-a 交给驱动 --merged-log、另两条单机跑」「驱动退非 0 判红、不写标记」。

### 途中收到的追加
1. 07:5x 之后，在做第 1–7 条时收到：admission 自证在 singlefs-39 补声明之后有 replay.sh 两格、54 号若干格红，要在两处临时仓链真仓的 `.claude/singlefs-ai-sop` 并写进 .gitignore。做法：第 2706 行 `LINKED_INTO_SELFTEST_REPOSITORIES`、第 2709 行 `link_sop_copy`，在 replay（`run_replay_cells`）与 54 号（`run_layer0_stage_cells`）两处调。
   - 改前原样（开工时在仓里跑）：`  ✗ admission.py 自证没过：17 格判错（共 207 格）`；草稿副本上 replay.sh 也换成带 preflight 那一版之后：`  ✗ admission.py 自证没过：21 格判错（共 207 格）`。点名的是 54 号若干格与「replay.sh：老产物没有产物头…」两格，原因原样 `…/research/scripts/../../.claude/singlefs-ai-sop/scripts/preflight.sh: No such file or directory`。
   - 改后见「出口各项」232 格全过。
2. 在新建三份脚本时收到：仓根模板超写范围，改放草稿目录（见「结论」）。
3. 在跑出口检查时收到：singlefs-39 的垫片 `.claude/scripts/preflight.sh` 出来了，要新脚本改 source 它、补声明、`--selftest` 挪到 preflight 之后，admission 自证临时仓再链 `.claude/scripts`。都做了（第 2 条与第 1 项）。这期间仓里 admission 自证又因 helper 改 source 垫片红了 21 格，原样 `…/research/scripts/../../.claude/scripts/preflight.sh: No such file or directory`，加链之后转绿。
4. 看门狗报：驱动自证里真起了 `cargo test … --exact the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts`，被判成崩溃枚举用例。
   - ① 主工作区的 stage-inputs.tsv 没有这一行：它是自证在临时仓副本的登记表里追加的（`crash-case:sharded-selftest`）。真仓登记只点名标了 `#[ignore]` 的用例；新加的 fast-tier 那条第 131 行标了。
   - ② 自证改成默认用假 cargo：临时仓里另写一条标 `#[ignore]` 的替身用例，三趟都不带 `--include-ignored`，不算重型。只有带 `SINGLEFS_HEAVY_TESTS` 时才用真 cargo 跑那条小流用例，不带时成功行写明真 cargo 那一趟本次未跑（自证第 48–89 行）。
   - 那一次真 cargo 跑是 UTC 23:19–23:26（7 分钟，临时目录 1.2G，自己删了），10 格全过：`  ✓ layer0-shard-run.sh 自证通过：10 格都对（第二台是本机上的另一个目录，没碰真的第二台）`。它经 `capped.sh 4` 与 `run-with-memory-cap.sh 8G` 起。
   - 假 cargo 版的证红：草稿副本里把驱动比工具链与数账本两处换成 `true`，原样 `  ✗ layer0-shard-run.sh 自证没过：2 格判错（共 10 格）`（③ ④ 两格红），换回之后 10 格全过。

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:35-39`（整段抄，未转述）**

```markdown
### 第 1 条：实分一三份 diff 重打（admission.py、stage-inputs.tsv、lib_heavy_tests.py）
- `patch --dry-run` 在今天的文件上：admission.py 19 个 hunk 有 5 个打不上（文件头第三列、弄坏开关表、自证成功句、线程格之后的 merge 格、main 的用法句），全在草稿副本里照 diff 的意思按新代码重写。lib 的 4 个全套上。tsv 那一份按第 7 条合并着改。
- 按 G1 之后的接口改的地方：merge 格接在 G1 新加的 `started_worker_threads=` 两格之后；54 号分片格里假驱动的参数照旧；`CrashCase` 多一个成员 `is_shardable_across_machines`。
- 原有自证：`admission.py --selftest` 实分一的格全在，现 232 格全过（原样末行见「出口各项」）；`lib_heavy_tests.py --selftest` 141 种全过，其中实分一的 4 格也在。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:40-45`（整段抄，未转述）**

```markdown
### 第 2 条：三份新脚本（来源 deliver，照 G1 之后的 admission.py 核过）
- deliver 版调 `crash-case-manifest` 用的是 `--extra-file <54 号> --extra-file <admission.py>`。G1 之后 54 号给的是 `--judging-digest --toolchain --build-environment`，照原样调，两边指纹永远对不上。驱动本机、第二台、跑完三处都改成 G1 的参数，自证里算指纹那一处也跟着改。
- deliver 版调 `crash-case-record` 用 `--threads-text`，G1 已经删掉这个选项，改成 `--machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin"`。
- 追加指示之后：三份都改成 `source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"`（垫片）。驱动的 `--selftest` 分支挪到 preflight 那一行之后，同时删掉 `run-condition: check …configuration-check.sh`：preflight 的 check 读不到参数，留着它的话，没有配置时 `--selftest` 会被拒 78。配置由驱动开跑后第一步的配置判法拒，退 1，原因与出路照旧，自证 ⑤ 相应改成期望退 1。
- 三份相对 deliver 的差：`driver-vs-deliver.diff`（70 行）、`selftest-vs-deliver.diff`（175 行）、`check-vs-deliver.diff`（14 行）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:46-59`（整段抄，未转述）**

````markdown
### 第 3 条：`crash-case-shardable` 进 `CRASH_CASE_JUDGING_SUBCOMMANDS`
- 改法：admission.py 第 208–209 行把它加进元组。判法摘要（第 1894–1895 行）从分派表取这几项；弄坏开关 `shardable-outside-judging-digest` 把它摘掉。
- 该红的输入：把 `command_crash_case_shardable` 里「没登记」那一支的 `return 1` 改成 `return 0`，看判法摘要变不变（`probe_g2.py`）。
- 改前（G1 代码加实分一 hunk）→ 改后，原样：
```
改前	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要不变
改后	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要变
```
- 自证：`JUDGING_DIGEST_VARIANTS` 加一格（第 3416 行 `SHARDABLE_ANSWER_UNREGISTERED`）。开关格在第 3471 行。在草稿副本上用 `ADMISSION_BREAK=shardable-outside-judging-digest` 跑：
```
  ✗ 判法摘要：单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了 ⇒ 摘要变：摘要没变
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:60-73`（整段抄，未转述）**

````markdown
### 第 4 条：双机驱动进两条层 0 流用例的指纹（选「按内容算进输入」）
- 为什么不走 `crash-case-command`：驱动决定判绿的不只是起 cargo 那一条命令。两片的退出码与 `LAYER0_SHARD mode=run` 行、账本拷回与数份、merge 那一趟的环境、merge 行在不在，都是它判的。改成经 `crash-case-command` 起，进摘要的只有命令，这些判法照样在指纹之外；`crash-case-command` 还得另收第二台的进度目录与线程数。按内容算进去，整份驱动都在指纹里。驱动 eval 配置判法打出来的赋值，所以配置判法 `layer0-shard-configuration-check.sh` 也一起进。
- 改法：admission.py 第 187 行 `SHARD_DRIVER_FILES`；第 1551 行 `shard_driver_lines`（登记了 `shard=across-machines` 的，这两份按内容排在登记行之前，不在的记「找不到 <路径>」）；第 1547 行接进清单。stage-inputs.tsv 第 28 行 54 号那一行的路径加上这两份：只改驱动时，快档不会在「范围那一问」退 77，照样核标记。代价：分片关着、照单机跑时驱动也在指纹里，改它这两条会多跑一趟（文件头「管不到的」写明，接受）。
- 该红的输入：临时 git 仓里放一条 `shard=across-machines` 的用例，先放进驱动脚本，再改它，看指纹。原样：
```
改前	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹不变；改驱动脚本 ⇒ 指纹不变（7ee48c0c812a → 7ee48c0c812a）
改后	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹变；改驱动脚本 ⇒ 指纹变（03e9308a66e2 → 3f3cc1939518）
```
- 自证：`run_layer0_stage_shard_cells` 开头两格（第 3824 行起）。`ADMISSION_BREAK=shard-driver-outside-manifest` 下：
```
  ✗ 输入清单：放进双机分片的驱动脚本与配置判法 ⇒ 登记了 shard=across-machines 的 stream-a 指纹变、没登记的 stream-b 不变：stream-a 6fd93cbbda19bb0d → 6fd93cbbda19bb0d，stream-b 96ad622302455cf8 → 96ad622302455cf8
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:74-98`（整段抄，未转述）**

````markdown
### 第 5 条：`strace -E VAR=VAL` / `--env=VAR=VAL`
- `strace -h` 现跑，第 15 行原样 `  -E VAR=VAL, --env=VAR=VAL`，第 17 行 `  -E VAR, --env=VAR`（只清变量）。
- 改法：lib_heavy_tests.py 第 606 行 `LAUNCHER_ENVIRONMENT_OPTIONS` 加 `"strace": {"-E", "--env"}`；第 597 行 strace 带值的选项加 `--env`；第 651 行接弄坏开关 `strace-drops-env`。`-E VAR` 不带 `=` 的不带进里面那条命令（沿用原有的 `"=" in value` 判法）。文件头第 15、40、54 行跟着改。
- 改前（仓里原来那份 lib）→ 改后，`probe_strace.py` 在 lib 自己的样本工作区里判，原样：
```
改前	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改前	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改后	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
```
  改前 `--env <值>` 分开写那一种碰巧判成重型：`--env` 不在表里，被当成不带值的选项，赋值那个词被当成要起的命令剥过去。这是巧合，不是判对了。
- 自证：第 967 行起加 5 格（最后一格 `-E VAR` 只清变量，判不重型）。`LIB_HEAVY_TESTS_BREAK=strace-drops-env` 下，lib 自检与 heavy-test-guard.sh 自检都红：
```
  ✗ lib_heavy_tests 自检：strace -E 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env= 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env 值另起一个词、设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace -fE 合写设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ 自检：lib_heavy_tests.py 的自检 应当是 0，实际 1
```
  heavy-test-guard.sh 本身没改。

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:99-113`（整段抄，未转述）**

````markdown
### 第 6 条：`crash-case-command` 起用例时清掉调用方的 `SINGLEFS_LAYER0_SHARD`
- 改法：admission.py 第 1856–1857 行，`env -u SINGLEFS_LAYER0_SHARD` 放在第一个 `NAME=VALUE` 之前。第一版写在 `SINGLEFS_LAYER0_START_OVER=1` 之后，54 号 `--start-over` 那一格当场红了：env 把后面的 `-u` 当成要起的命令。
- 该红的输入：调用方设着 `SINGLEFS_LAYER0_SHARD=0/2`，照交出的命令起一趟，把 cargo 往后换成打那个变量的 sh。原样：
```
改前	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「0/2」（命令前段：env -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「unset」（命令前段：env -u SINGLEFS_LAYER0_SHARD -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	带 --start-over 时起的用例看到「unset/1」、退 0
```
  （命令前段里的临时路径我换成了 `…`；整行原文在 `logs/probe-before.log` 与 `logs/probe-after.log`。）
- 自证：第 3804、3813 行两个函数。`ADMISSION_BREAK=keep-caller-shard-switch` 下：
```
  ✗ crash-case-command：调用方环境里设着 SINGLEFS_LAYER0_SHARD=0/2 ⇒ 起的用例看不到它：用例看到的是「0/2」
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:114-132`（整段抄，未转述）**

````markdown
### 第 7 条：stage-inputs.tsv
- 第 16–20 行（原第 16–18 行）：「再加判它的 54 号、准入模块 admission.py」改成现状：进指纹的是准入模块里崩溃枚举用例的判法摘要；登记了 `shard=across-machines` 的另按内容加驱动与配置判法；54 号不进。第三列的说明加上 `shard=across-machines`。
- 第 28 行（54 号那一行）：路径加 `research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh`（第 4 条）；注释「（它也进每条用例的指纹）」改成「进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份」，另加一句驱动与配置判法。
- 第 36、37 行：实分一的 `shard=across-machines` 与注释。
- 第 39 行 c561：第三列与注释逐字照实审 B3b 报告第 116 行起那一段（`exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL`）。我核过用例确实打这两行：`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs` 第 656 行打计数行，第 675 行打 `LAYER0_PARALLEL_FINISHED` 同形行。
- 第 40 行新加 `crash-case:crash-injection-fast-tier`：原文是实审 B3c-1 报告第 85 行整行抽出来追加的（`awk 'NR==85'`），追加之后与抽出来的那行 `cmp` 相同，4 列。用例在 `second_transaction_supplement_three_crash_injection.rs` 第 131 行标 `#[ignore]`，第 132 行是用例函数。
- 该红的输入：拿改前 / 改后的表判合成日志（`probe_g2.py`）。原样：
```
改前	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改前	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 登记表里读不出这一条（.claude/gate.d/stage-inputs.tsv 里崩溃枚举用例 crash-case:crash-injection-fast-tier 要恰好一行，实际 0 行）
改后	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改后	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 判红
```
- 自证：admission.py 第 3783 行 `run_real_crash_case_row_cells`，拿真仓登记表的这两行判 5 格。登记表是数据，没配弄坏开关；红的样子就是上面「改前」那几行，谁把这两行改回去，这 5 格就红。

````

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:133-137`（整段抄，未转述）**

```markdown
### 第 8 条：`.claude/rules/implementation-workflow.md`（第二种活，改的规则）
- 第 58 行「崩溃枚举用例」一条末尾加一句：「双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。」这与 lib 第 706 行的判法一致：`return None if "--selftest" in arguments else heavy_test("crash-case-cargo", …)`。
- 同一行 runner 那一句「`systemd-run -E` / `--setenv`」后面加「、`strace -E` / `--env`」，跟第 5 条的闸对齐（规格没点名这一处，但规则要与闸逐类相同）。
- 改过的定义、共用约束与规则只有这一份，交主 agent 开定义三方（门禁 72 号）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:138-143`（整段抄，未转述）**

```markdown
### 第 9 条：54 号（最后套）
- 动手前读了一遍今天的样子，与备份相同（singlefs-39 的准入声明与 preflight 调用已在）。实分一 4 个 hunk 有 2 个打不上（文件头、循环那一段），照 G1 的 `read_crash_case_command` / `run_crash_case <日志> <命令…>` 重写。
- 改后字面：第 25–29 行文件头的双机分片一段；第 237–244 行 `run_crash_case_in_two_shards`；第 350–358 行判开不开（配置判法判得过才开，打一行开或关）；第 394–418 行逐条用例：先 `crash-case-shardable` 定走哪一路，分片的 `case_threads_note` 写明第二台取它自己的核数，退非 0 时分片那一支有自己的 ✗ 与出路。merge 那一趟的日志照单机的判法判、写同一格（线程逐片判在 admission.py 第 1608、1629 行）。
- gate-lint 在第 413 行报过一次红：出路句里写了「✗」字，被当成下一处拒绝。改成「驱动脚本输出里判红的那一句」，重跑转绿（见「出口各项」）。
- 自证：admission.py 里 54 号那几格与实分一的两格分片格（第 3824 行起）全过，含「双机分片：关」「开：stream-a 交给驱动 --merged-log、另两条单机跑」「驱动退非 0 判红、不写标记」。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:144-155`（整段抄，未转述）**

```markdown
### 途中收到的追加
1. 07:5x 之后，在做第 1–7 条时收到：admission 自证在 singlefs-39 补声明之后有 replay.sh 两格、54 号若干格红，要在两处临时仓链真仓的 `.claude/singlefs-ai-sop` 并写进 .gitignore。做法：第 2706 行 `LINKED_INTO_SELFTEST_REPOSITORIES`、第 2709 行 `link_sop_copy`，在 replay（`run_replay_cells`）与 54 号（`run_layer0_stage_cells`）两处调。
   - 改前原样（开工时在仓里跑）：`  ✗ admission.py 自证没过：17 格判错（共 207 格）`；草稿副本上 replay.sh 也换成带 preflight 那一版之后：`  ✗ admission.py 自证没过：21 格判错（共 207 格）`。点名的是 54 号若干格与「replay.sh：老产物没有产物头…」两格，原因原样 `…/research/scripts/../../.claude/singlefs-ai-sop/scripts/preflight.sh: No such file or directory`。
   - 改后见「出口各项」232 格全过。
2. 在新建三份脚本时收到：仓根模板超写范围，改放草稿目录（见「结论」）。
3. 在跑出口检查时收到：singlefs-39 的垫片 `.claude/scripts/preflight.sh` 出来了，要新脚本改 source 它、补声明、`--selftest` 挪到 preflight 之后，admission 自证临时仓再链 `.claude/scripts`。都做了（第 2 条与第 1 项）。这期间仓里 admission 自证又因 helper 改 source 垫片红了 21 格，原样 `…/research/scripts/../../.claude/scripts/preflight.sh: No such file or directory`，加链之后转绿。
4. 看门狗报：驱动自证里真起了 `cargo test … --exact the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts`，被判成崩溃枚举用例。
   - ① 主工作区的 stage-inputs.tsv 没有这一行：它是自证在临时仓副本的登记表里追加的（`crash-case:sharded-selftest`）。真仓登记只点名标了 `#[ignore]` 的用例；新加的 fast-tier 那条第 131 行标了。
   - ② 自证改成默认用假 cargo：临时仓里另写一条标 `#[ignore]` 的替身用例，三趟都不带 `--include-ignored`，不算重型。只有带 `SINGLEFS_HEAVY_TESTS` 时才用真 cargo 跑那条小流用例，不带时成功行写明真 cargo 那一趟本次未跑（自证第 48–89 行）。
   - 那一次真 cargo 跑是 UTC 23:19–23:26（7 分钟，临时目录 1.2G，自己删了），10 格全过：`  ✓ layer0-shard-run.sh 自证通过：10 格都对（第二台是本机上的另一个目录，没碰真的第二台）`。它经 `capped.sh 4` 与 `run-with-memory-cap.sh 8G` 起。
   - 假 cargo 版的证红：草稿副本里把驱动比工具链与数账本两处换成 `true`，原样 `  ✗ layer0-shard-run.sh 自证没过：2 格判错（共 10 格）`（③ ④ 两格红），换回之后 10 格全过。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:187-194`（整段抄，未转述）**

```markdown
## 第三轮要攻的改后字面（判决第四节口径）
- admission.py：第 182–189 行（`shard=` 写法、`SHARD_DRIVER_FILES`、`LAYER0_SHARD_VARIABLE`）；第 208–209 行（判法入口加 `crash-case-shardable`）；第 1163 行起 `parse_crash_case` 的 `shard=` 分支；第 1547–1563 行（驱动按内容进清单）；第 1608 行与第 1625–1657 行（merge 那一行逐片判线程）；第 1856–1857 行（清分片开关）；第 1894–1895 行（摘要里的子命令表）；第 2240 行 `command_crash_case_shardable`；自证第 2706–2718 行、第 3365 行、第 3416 行、第 3471 行、第 3783–3822 行、第 3824 行起。
- 54 号：第 25–29 行、第 237–244 行、第 350–358 行、第 394–418 行。
- stage-inputs.tsv：第 16–20 行、第 28 行、第 36–37 行、第 39–40 行。
- lib_heavy_tests.py：第 95 行、第 597 行、第 606 行、第 651 行、第 706–708 行，自证第 967 行起。
- 规则 implementation-workflow.md：第 58 行。
- 三份新脚本全文，重点是驱动第 33–39 行（preflight 在 `--selftest` 之前）、第 149–163 行（两台指纹）、第 263–279 行（判与写标记），以及自证的假 cargo（第 48–89 行）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:195-200`（整段抄，未转述）**

```markdown
## 看到但没做
1. 54 号快档的 `run_layer0_test_binary` 起两条流的快用例时，没清调用方的 `SINGLEFS_LAYER0_SHARD`。第 6 条只点名 `crash-case-command`，这一处照旧（假红方向）。
2. 分片 merge 那一趟，54 号交给 `crash-case-record` 的 `--machine-cores/--threads` 是本机那一片的，标记里 `configured_worker_threads=` 只记本机。第二台那一片配的数只在 `parallel_finished=` 行的 `shard_configured_worker_threads=` 里（推的：标记字面对分片的描述不全，没改）。
3. strace 的别的长选项（`--output=`、`--user=` 等）不在 `LAUNCHER_OPTIONS_WITH_VALUE` 里：带值另起一个词的写法会被当成命令，剥错。只补了 `--env`。
4. 仓根 `layer0-shard.env.example` 由主 agent 拷；实分一 deliver 里的 kb 第六节 diff 没碰（归书记员）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:201-206`（整段抄，未转述）**

```markdown
## 没做什么
- 没跑重型测试：54、55、57、59、87 号本身、`gate.sh`、全量 `cargo test`、`check.sh`、名字带 layer0 的目标、驱动的非 `--selftest` 形态都没跑。驱动自证在追加 ④ 之前用真 cargo 跑过一次小流用例（`crash_enumeration_sharded_across_processes`，不带 layer0、不带 `--include-ignored`，经内存包装与 capped 4）。看门狗把它记成崩溃枚举用例：临时仓登记表里那一行让它看起来像登记用例。之后改成默认假 cargo。
- 没走定义三方，没提交，没做 git 写操作。
- heavy-test-guard.sh 没改；47 号没改（新脚本的自证没挂进 runner 表）。
- 47 号与 doc-lint 的红不在这一批，没修。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:207-210`（整段抄，未转述）**

```markdown
## 草稿目录与清理
- 删了：`/tmp/claude-1000/gate-batch-m2-g2/dev`（215M，开发用的仓副本）、`sim`（516K，套 hunk 的演练副本）、`try`（1.1M，patch 试打）。自证、探针建的临时仓都由它们自己删掉了（`ls -d /tmp/admission-selftest-* /tmp/g2-probe-*` 零个）。
- 留着：`backup/`（五份改前原件）、`g2-changes.diff`、`*-vs-deliver.diff`、`hunks-*.diff`、`round2-*.diff`、`apply_hunks.py`、`probe_g2.py`、`probe_strace.py`、`logs/`、`progress.md`、`layer0-shard.env.example`（待拷）、`final-sha256.txt`（八份仓内文件交回时的 sha256）。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:211-213`（整段抄，未转述）**

```markdown
## 补记
仓根 `layer0-shard.env.example` 已由主 agent（README 那件）放进仓根，与草稿那份逐字节相同（`cmp` 现核相同，sha256 `72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4`），不用再拷。

```

**出处 `records/2026-09-24-里程碑二收尾调度.md:190-190`（整段抄，未转述）**

```markdown
| 崩溃枚举的跑法（用户 2026-09-26 JST 18:5x） | 用户原话：「这次跑可以 下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」起因：实五按主 agent 规格加了一条在会话推的抬 F 那一串上做小枚举的用例（`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`，20,493 个状态，debug 下单跑约 12 分钟、约 16 核），它是这条新写路径唯一的崩溃证据，层 0 两条流在 4 GiB 盘上碰不到准入被拒（推的）。**定**：① 这一次照跑；实五证红做完后给它加 `#[ignore]`，写明提交时由崩溃验证员按输入哈希跑（release）。② 门禁批（实六之后）把它接进提交时崩溃验证员那一组，在 `.claude/gate.d/stage-inputs.tsv` 登记它读的输入，输入没变复用上一次全绿判定。③ 以后崩溃枚举一律按用例 / 按流各自登记输入、各自复用，改了哪块只重跑读它的那几条——就是层 0 规模第二轮挂起的乙；第二轮打中「按流列的清单漏新加的共用文件」，门禁批照当时的修法做：清单用排除法写（整个 `crates/` 减去别的流自己的用例文件）、指纹补第三轮 U4 那几样；被攻过零轮，改的是门禁与崩溃验证员定义，照「改 agent 定义与共用约束，走同一条三步」走一轮三方 |
```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:156-178`（整段抄，未转述）**

```markdown
## 出口各项（在仓里现跑，UTC 2026-09-26 23:3x–23:5x；末行原样）

| 命令 | 退出码 | 末行 / 判红的行 |
|---|---|---|
| `python3 research/scripts/admission.py --selftest` | 0 | `  ✓ admission.py 自证通过：232 格都对（含弄坏开关 skip-unchanged、…、single-worker-threads-field、threads-ignore-shards、shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）`（中间一段开关名是我略的；全文在 `logs/final-admission.log`；改完 54 号出路句之后又跑了一次，同样 232 格全过） |
| `python3 .claude/hooks/lib_heavy_tests.py --selftest` | 0 | `  ✓ lib_heavy_tests 自检通过（查了 141 种：cargo 与包装过的命令行 55 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 33 种、直接执行的测试二进制 20 种、按名字判的脚本 6 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种）` |
| `bash .claude/hooks/heavy-test-guard.sh --selftest` | 0 | `  ✓ 自检通过（查了 658 种，其中该拒 121 种）：…` |
| `bash research/scripts/layer0-shard-run.sh --selftest` | 0 | `  ✓ layer0-shard-run.sh 自证通过：10 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）`（31 秒；直接跑 `layer0-shard-run-selftest.sh` 被重型闸要求经内存包装，经 `run-with-memory-cap.sh 8G` 跑同样 10 格全过） |
| 47 号 | 1 | 只有一处红：`  ✗ python3 research/scripts/check-segment-registry.py --selftest 没过（退出码 1）：`（段序列登记表与 E142 产物对不上 2 处）。不在这一批里，没修。admission.py 与驱动自证在这一趟里没被列出 |
| 62 号 | 0 | `  ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| 63 号 | 0 | `  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，…` |
| 73 号 | 0 | `    没扫的目录 0 个（不在）：（没有）`；它的汇总行 `  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录（research/scripts .claude/hooks .claude/scripts），拒绝都带出路、shell 纪律守住；…` |
| `doc-lint.sh .` | 1 | `  ✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 514，跳过 0）`：点名 O1–O3、F23、F24、M32、M34、U11、U13 与 `.claude/kb/experiments-history.md`、`.claude/kb/experiments/158-择根与修复四岔路.md`。在这一批改过的八份里 `grep -nwE` 这几个编号零命中，两份 kb 是别的会话的 M |
| rules-lint（项目本地） | 0 | `  ✓ 规则只写怎么做（扫了 29 份文件 1958 行；没扫 0 个；记录小节 0、论证小节 0、…命中 0；…）` |
| `GATE_LINT_DIR=.claude/gate.d` gate-lint | 0 | `  ✓ 门禁自检通过：84 个脚本（.sh 与 .py）、305 条拒绝都带了出路`（改 54 号第 414 行出路句之前红 1 处，见第 9 条） |
| `SHELL_LINT_DIR=.claude/gate.d` shell-lint | 0 | `  ✓ shell 纪律检查通过（共 76 个脚本）` |
| gate-lint / shell-lint 对 research/scripts 与 .claude/hooks | 0 | research gate-lint `  ✓ 门禁自检通过：73 个脚本（.sh 与 .py）、237 条拒绝都带了出路`；research shell-lint `  ✓ shell 纪律检查通过（共 39 个脚本）`；hooks gate-lint `  ✓ 门禁自检通过：12 个脚本（.sh 与 .py）、18 条拒绝都带了出路` |
| `preflight-lint.py` | 0 | `  ✓ 准入与运行条件：判了 137 个脚本（登记目录 49、脚本 3、钩子 9、门禁阶段 76），都在开头写明了条件并先判；…` |
| `gate-overlap.py` | 0 | `  ✓ 相对 97f5904b44cd：新加的门禁与钩子 2 个（.claude/gate.d/84-verdict-false-named.sh、.claude/hooks/handback-guard.sh）都写明了比过谁，改过的 92 份脚本对照已有的 133 份没有整段相同` |
| 阶段归属表登记给 tooling-writer 的阶段 | — | awk 列出 0 个，没有要额外跑的 |

`admission.py` 在 `.claude/preflight-exclude` 第 17 行登记成「还没改完：别的会话（singlefs-99 的 G2）正在改它，改完之后补声明」。它的准入声明我没写（规格说归 singlefs-39），现在交回，这一行要由那边收尾。

```

**出处 `research/prompts/defs-gatebatch-m2-g2-report.md:179-186`（整段抄，未转述）**

```markdown
## 新写的 `# gate-similar:` / `# hook-events:` 行
没有：这一批没新建门禁阶段或钩子（54 号是改，lib_heavy_tests.py 是库）。新建的三份是研究脚本，准入声明原样：
- layer0-shard-run.sh 第 33–34 行：`# admission: always 每次调都跑这一刻这棵树上的一条崩溃枚举用例；复用判定在门禁 54 号的全绿标记里，不在这里`／`# run-condition: command cargo rustc python3 rsync ssh nproc git`
- layer0-shard-run-selftest.sh 第 17–18 行：`# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑`／`# run-condition: command cargo rustc python3 rsync git nproc`
- layer0-shard-configuration-check.sh 第 17–18 行：`# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判`／`# run-condition: command git bash`

三份的自证没挂进 47 号的 runner 表：47 号不在规格的改动清单里，交主 agent 定。

```
