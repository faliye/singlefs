# 里程碑二收尾：门禁批第三轮判决（2026-09-27）

<!-- doc-lint:not-numbers J1 K3 K4 Y1 P1 P2 P3 P4 P5 P6 P7 P9 P10 D2 L1 L2 L3 F1 F2 F3 F4 F5 F6 F9 F10 FD2 FD3 FM G1 G2 G3 -->

正文 `research/prompts/_defs-gatebatch-m2-r3-body.md`；腿：云端攻方 `research/prompts/defs-gatebatch-m2-r3-opus-output.md`（sha256 b3fb28ef…）、云端正推 `research/prompts/defs-gatebatch-m2-r3-sonnet-output.md`（sha256 41aaa135…）、本地攻方两份样本 `research/prompts/defs-gatebatch-m2-r3-local-attack-output-s1.md`、`-s2.md`；核查员 `research/prompts/defs-gatebatch-m2-r3-verifier-output.md`（sha256 f95fb489…）：核了 101 处，✓ 99、✗ 1（正推报告把 `admission.py:1632-1633` 写成 `:1629-1630`，结论不受影响）、核不动 1（正推报告里两条 `--selftest` 没复跑）；攻方 `rerun.sh` 与正推 K4 那一节的复跑逐字吻合。开工快照 `research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt`（14 个文件）腿开工（2026-09-27）与交齐时两次 `sha256sum -c` 全 OK。

这是门禁批第三轮，按 `.claude/rules/three-way-inference.md`「第三轮之后停」不开第四轮。

## 一、各格判定

| 格 | 判定 | 依据 |
|---|---|---|
| P3（libtest 带值选项之后的 `--list`） | **打中（量过），假绿**：第二个 `--` 之后的 `--list` 是过滤词，登记的用例照跑，闸放行 | 攻方第一节 T5–T7，核查员复跑吻合 |
| P10（`--config` 按 TOML 读） | **打中（量过），假绿**：`--config` 写在 `test` 之后 cargo 照认，闸不收它的值 | 攻方 R9–R11 |
| P2（`systemd-run` 设给里面那条命令的环境） | **打中（量过），假绿**：`-p Environment=` / `--property=Environment=` 闸放行 | 攻方 S1、S2 |
| P1（剥包装认短选项合写） | 短选项合写**没打中**；同一类第二个实例**打中**：要值的长选项值另起一个词（`systemd-run --expand-environment no`、`strace --output <文件>`）闸把值当命令 | 攻方 S3、S4 |
| P5（判不出按没标算） | **打中（量过），第二轮判的「方向是多拒」只对一半**：准入模块导入得了、判的那一刻抛异常时闸退 0 放行、不记检出 | 攻方第二节 C、D |
| P4 / P4b（按每一处同名 `fn` 判 `#[ignore]`） | 这一改动本身只多拒（成立）；**`include!` 进来的同名不标定义看不见（量过，假绿，P4 之前就有）** | 攻方 I1、I2 |
| P6 / P7（算出来的 include 按宽处理） | **打中（量过），假复用**：`use core::include_str as grab;` 改名后认不出 | 攻方 N1–N3 |
| P9（runner 参数按内容进指纹） | **打中（量过），假复用**：cargo 起 runner 的当前目录是包目录，`sh ../../tools/r.sh` 进不了指纹；G1 报告自己写的推翻条件成立 | 攻方 P9a、P9b |
| D2（判法摘要） | 今天**没打中**（运行时用到的 51 个定义全在 52 个的闭包里）；**以后会打中**的四种写法（元组解包、`if` 块里的 `def`、分派表值写成 lambda、判法挪进 import 的模块）不在文件头「看不见的」那一句里；「54 号只剩流程的次序」**字面不成立**（判绿取决于 54 号转过去的线程参数） | 攻方第四节 ①②③ |
| 标记字段 | **打中（量过），危害小**：`crash-injection-fast-tier` 读 `SINGLEFS_CRASH_INJECTION_THREADS`，标记写 `SINGLEFS_LAYER0_THREADS`；调用方的值漏进 `--full`；`OMP_NUM_THREADS=1` 让交的核数变 1、64 片 1 线程判绿 | 攻方第五节 ①②③ |
| Y1、K3 floor-raise、J1-c | **站得住（推的，读代码，没跑探针）**；J1-c 那句只点名两趟 `--full`，没点名单独跑的双机驱动 | 攻方第六节 |
| 分片判法四条链路 | **与条款一致**（`shard=`、`crash-case-shardable` 进判法摘要、merge 线程行、驱动按内容进指纹） | 正推第一至四节 |
| 条款没跟上分片 | **属实**：`.claude/rules/implementation-workflow.md` 与 `.claude/agents/crash-verifier.md` 零处提分片 / 双机；第二台那一片的 cargo test 没有内存上限（`run-with-memory-cap.sh` 只包本机进程树），`layer0-shard-configuration-check.sh` 七个配置键里没有内存键（读脚本正文，没有第二台可实测） | 正推第五节 |
| 第二轮判决 K4 的两个数 | **对不上，原因查清**：同一份探针在正确重建的第二轮基线上是 37 / 365 / 11% / 1 个 hunk 落在闭包里（第二轮判决写 0 个，是探针给删除行记新行号的已知不精确），在今天的文件上是 42 / 427 / 10% / 3；今天真正出货的 D2 算法算是 52 个定义 / 595 行 / 15% | 正推第六节，核查员复跑四组逐字吻合 |
| 七条崩溃枚举登记行 | 本地两份样本一致的：`crash-injection-fast-tier` 打的 `CRASH_INJECTION_FINISHED` 行带 `worker_threads=`，而 `admission.py` 的线程判定只认 `LAYER0_PARALLEL_FINISHED` 行，这条用例没有线程判定；floor-raise 与 crash-injection 两条没登记 `exhaustive=`；快档对七条都只看 `test result:`（七条全标 `#[ignore]`）。两处不稳定（Q1 Row D、Q4 Row D，两份样本填法不同）不采 | 本地 s1、s2；核查员表三 |

## 二、改法（全部被攻过零轮；第三轮之后不再为它们开一轮，交用户表里标「零轮」）

打中的都是门禁工具的假绿 / 假复用，是工具的缺陷，照 `.claude/singlefs-ai-sop/rules/sop-first.md`「改了 `scripts/` 就造一个该被拦的输入」修：每条先在今天的代码上造会红的输入、改完转绿、自证加一格。

| 改法 | 做什么 | 修哪几格 | 来源 |
|---|---|---|---|
| F1 | 剥包装时要值的长选项（`systemd-run --expand-environment`、`strace --output` 等）的值跳过；各包装程序的长选项照各自 `--help` 列全 | S3、S4 | 攻方，副本量过（补全推的） |
| F2 | `systemd-run -p` / `--property` 的值以 `Environment=` 开头时当环境设给里面那条命令；`EnvironmentFile=` 按内容读 | S1、S2 | 攻方，副本量过（`EnvironmentFile=` 推的） |
| F3 | `libtest_lists_only` 见到 `--` 就停、交「不是只列」；代价是多拒 `-- --ignored -- --list`（一条都不跑），与已接受的误拒同类，接受 | T5–T7 | 攻方，副本量过 |
| F5 | 调准入模块判标没标时接住异常，按「判不出」算（拒） | P5 的 C、D | 攻方，副本量过 |
| F10 | 子命令之后、`--` 之前的 `--config` / `--config=` 的值也收进来 | R9–R11 | 攻方，副本量过 |
| F4 | 判 `#[ignore]` 时顺着测试目标里 `include!` / `#[path]` 的字面路径把那几份一起读；`include!` 参数不是字面量的整条按判不出 | I1、I2 | 攻方，推的 |
| F6 | 任何 `.rs` 里有 `use` 引进 include 族宏（改名或不改名）就按宽处理，不减文件；代价是那种写法出现时多重跑 | N1–N3 | 攻方，推的；主 agent 取宽的那一种 |
| F9 | runner / wrapper 参数里的相对路径另从每个登记用例的包目录解一遍，解得到的都进指纹 | P9a、P9b | 攻方，推的 |
| FD2 | 判法摘要：模块级语句不是 def / class / 名字赋值的整条原文进摘要；分派表的值不是名字时拒算；判法入口闭包里出现 import 进来的非标准库名字时拒算；文件头「看不见的」那一句补上仍看不见的写法 | D2 ② | 攻方，推的 |
| FD3 | 54 号不再转线程参数：`crash-case-judge` 自己按 `crash_case_worker_threads` 现取核数与线程数；改完 54 号文件头「只剩流程的次序」那一句才成立 | D2 ③ | 攻方，推的 |
| FM | 本机核数取 `os.cpu_count()` 与 CPU 亲和取小，不认 `OMP_NUM_THREADS`；登记行加一格点名用例读的线程变量（`crash-injection-fast-tier` 登记 `SINGLEFS_CRASH_INJECTION_THREADS`），`crash-case-command` 清掉调用方的这个变量并设成配的线程数、标记照实写这个变量；`crash-injection-fast-tier` 的 `CRASH_INJECTION_FINISHED worker_threads=` 进线程判定 | 标记字段 ①②③；L3 线程那一格 | 攻方推的，加本地腿那一格 |
| R1 | 条款跟上分片：`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」与「门禁管哪一半」、`.claude/agents/crash-verifier.md` 写进双机分片（哪几条用例登记了 `shard=across-machines`、驱动怎么起、判绿要哪几样）；第二台那一片包内存上限（配置加内存键、驱动在第二台上经 `run-with-memory-cap.sh` 起）；J1-c 那句点名单独跑的驱动 | 正推第五节；攻方第六节 J1-c | 主 agent |

## 三、实现次序

1. 派工具实现员（G3）一次做完第二节全部改法，规格另写；它动的是 `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`（自证）、`research/scripts/admission.py`、`.claude/gate.d/54-layer0-replay.sh`（FD3；读发现日志那几段不动）、`.claude/gate.d/stage-inputs.tsv`（FM 的登记格）、`research/scripts/layer0-shard-run.sh` 与 `layer0-shard-configuration-check.sh`（R1 的内存键）、`.claude/rules/implementation-workflow.md`、`.claude/agents/crash-verifier.md`。
2. 改法全被攻过零轮，不再开第四轮；G3 交回后主 agent 逐条核自证格（每格改前红、改后绿），`crash-verifier.md` 改动过门禁 72 号要去向：交用户选「逐份豁免」或另开一轮定义三方，弹窗时带上这一条。
3. 改了准入模块的判法（FD2、FD3、FM）会让崩溃枚举用例的指纹变，已有全绿标记全部不再作数：下一趟 `--full`（提交时）要全跑。

## 四、交用户的（零轮）

| # | 事 | 为什么交用户 |
|---|---|---|
| 1 | 第二节 F1–F10、FD2、FD3、FM、R1 全部被攻过零轮，落地后不再攻 | `.claude/rules/three-way-inference.md`「第三轮之后停」：零轮形态写进交用户表 |
| 2 | F3、F5、F6 三处选了多拒 / 多重跑的方向（F3 多拒一条都不跑的写法；F5 判的时候抛异常就拒；F6 有 `use` 引 include 宏就不减） | 取舍是「宁可多拒多跑，不放过」，与第二轮 P4、P5 同向 |
| 3 | `crash-verifier.md` 改动过 72 号的去向 | 72 号要么三方判决点名、要么用户逐份豁免 |

## 五、按路径点名被判的定义与文件（门禁 72 号）

被判的：`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/admission.py`、`.claude/gate.d/stage-inputs.tsv`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/rules/implementation-workflow.md`、`.claude/agents/crash-verifier.md`、`research/scripts/layer0-shard-run.sh`、`research/scripts/layer0-shard-run-selftest.sh`、`research/scripts/layer0-shard-configuration-check.sh`；一并核过（开工快照里、没改）的 `.claude/gate.d/stage-owners.tsv`、`.claude/main-agent.md`、`.claude/agents/gate-triage.md`、`research/scripts/check-segment-registry.py`。

## 回看决策

不涉及决策：被判的是门禁、研究脚本、hook、规则与一份 agent 定义，不改 `.claude/kb/decisions/` 里任何一条决策。
