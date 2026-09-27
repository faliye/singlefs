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
