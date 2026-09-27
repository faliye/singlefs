# 门禁批（层 0 续跑接入、崩溃枚举按用例复用）三方第二轮判决（2026-09-27）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K1 K2 K3 K4 K5 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 D1 D2 D3 V2 V3 V4 V5 V6 V7 L2 L4 L6 L7 L12 L13 R2 R3 R5 R7 T2 T3 T4 M1 M2 M3 M4 M5 G1 G2 G3 -->

正文 `research/prompts/_defs-gatebatch-m2-r2-body.md`，背景 `_defs-gatebatch-m2-r2-background.md`，被判的 diff `_defs-gatebatch-m2-r2-diff.md`（改法 Y1–Y8，1707 行、6 个文件）。三条腿：辩方 `defs-gatebatch-m2-r2-defense-output.md`（K5 + K2）、攻方 `defs-gatebatch-m2-r2-opus-output.md`（K1、K3、K4，探针 `defs-gatebatch-m2-r2-opus-model/`）、本地攻方 `defs-gatebatch-m2-r2-local-attack-output-s1.md` / `-s2.md`（K3 事实表，两份干净样本；第一次调用因提示里的 Rust `::` 被损坏闸误红作废，`-output-void1.md`）。核查员 `defs-gatebatch-m2-r2-verifier-output.md`：核 66 处，✓ 59、✗ 4、分不清 3，判别力自证通过；攻方 Y2–Y7「打中，量过」在含 `.git` 的仓副本上复跑全部重现，6 条 SUMMARY 与 3 份日志逐字节相同。开工快照 `defs-gatebatch-m2-r2-snapshot/sha256sums.txt` 11 个文件，腿交齐之后主 agent `sha256sum -c` 全 OK。

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

## 三、实现次序

1. 门禁审核会话 singlefs-39 交来的 54 号那一处替换（`/tmp/claude-1000/gate-fix-54-handoff/`：复用判定只认退 1 为可跳过，其余判红；换用 `research/scripts/stage-run-or-skip.sh`）先套上；
2. 这一节的改法由一个通用 agent 做（规格另写），每条先在旧代码上造会红的输入；
3. 实分一（层 0 双机分片）交来的 `admission.py`、54 号、`stage-inputs.tsv`、`lib_heavy_tests.py` 四份 diff（`/tmp/claude-1000/impl-shard-1/deliver/`）在 2 之后按新代码重打；
4. 第三轮攻 2、3 的改后字面（第一、二轮攻过的面不重复），腿跑着时这几份文件不动。

## 四、第三轮

只攻：第二节改法的改后代码（P1–P7、P9、P10、D2、标记字段）、实分一带进这几份文件的分片判法（`shard=across-machines`、`crash-case-shardable`、merge 那一趟的线程行），以及前两轮站住的形态（Y1、K3 floor-raise、J1-c）撑不撑得住。第三轮之后停；第三轮新冒出来的零轮形态写进判决的交用户表。

## 五、按路径点名被判的定义与文件（门禁 72 号）

被判的：`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/admission.py`、`.claude/gate.d/stage-inputs.tsv`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/rules/implementation-workflow.md`；一并核过（开工快照里、没改）的 `.claude/gate.d/stage-owners.tsv`、`.claude/agents/crash-verifier.md`、`research/scripts/check-segment-registry.py`、`.claude/main-agent.md`、`.claude/agents/gate-triage.md`。

## 回看决策

不涉及决策：被判的是门禁、研究脚本、hook 与规则，不改 `.claude/kb/decisions/` 里任何一条决策。
