# 门禁批第三轮（defs-gatebatch-m2-r3）云端攻方（Opus）：L1 改法与站住形态

写于 2026-09-27 UTC 03:4x–04:1x（JST 12:4x–13:1x）。攻击面：正文第一节 L1。被判文件取开工快照（`research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt` 14 个，交回前 `sha256sum -c` 现核 14 个 OK）；54 号与驱动里读发现日志的那几段不攻、不列。所有「打中」都在今天仓里被判的那一份上量（闸只喂 JSON、准入模块在草稿目录的小仓上跑），改法那一栏的「量过」是 hooks 副本上的数。重型测试一条没跑。

## 复跑

```
cd /home/fy5090/code/singlefs
bash research/prompts/defs-gatebatch-m2-r3-opus-model/rerun.sh [<输出目录>] [<草稿目录>]
```

默认输出进模型目录的 `outputs/`，草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r3-opus`。最后一趟原样（`outputs/rerun-summary.txt`）：

```
l1-hook rc=1 SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=10 control_cells_broken=0
l1-p5 rc=1 SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=2 control_cells_broken=0
l1-attrs rc=1 SUMMARY L1 P4：include! 进来的同名定义: cells=10 attack_cells_broken=4 control_cells_broken=0
l1-manifest rc=1 SUMMARY L1 P6 P7 P9：清单与指纹: cells=10 attack_cells_broken=5 control_cells_broken=0
d2-trace rc=1 SUMMARY D2 判法摘要的闭包: cells=6 attack_cells_broken=5 control_cells_broken=0
marker-fields rc=1 SUMMARY 标记字段与线程来源: cells=3 attack_cells_broken=3 control_cells_broken=0
semantics rc=0 9 行 SEM
systemd-run rc=0 2 行 SEM
fix-l1-hook rc=2 SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=0 control_cells_broken=1
fix-l1-p5 rc=0 SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=0 control_cells_broken=0
fix-guard-selftest rc=0   ✓ 自检通过（查了 661 种，其中该拒 122 种�
rerun rc=0
```

模型目录 `research/prompts/defs-gatebatch-m2-r3-opus-model/` 每个文件的 sha256（`SHA256SUMS` 原样）：

```
6c833e0158c4688a3e5aa7729c081196aad94c29f22f6852815ae7529bd59b6d  ./fix-lib-heavy-tests.diff
2f39b7217f97aada57de829eaec99784ee818dea73070d26cbdf9663743bd2bd  ./make_fix_copy.sh
8d5bb7367442f68dc4a5b66984784f086a0cce9d9b65f5432049f1a82a3f8b2e  ./outputs/d2-trace.log
dd23832dd0ef528e248d2f981cd28a212536a22e1d61a6c30514d249b40aab58  ./outputs/fix-guard-selftest.log
56cd646caffdb355290208ffe6198bfb8e05c21796f6f97b5be753032eaeef0e  ./outputs/fix-l1-hook.log
ba13f9b3c7a547a2a79f7a028c63ba3ee710b8d0fa8f5f9d626a7ffb4d2cc57c  ./outputs/fix-l1-p5.log
0298c14b9c52dba4a07eba8f5285b3baaa842857bb085dd79f831a61f3368edf  ./outputs/l1-attrs.log
8002a83f7f1792fa14c5f9a342aae04c68717517fdc3b20764c06d7f51ea0334  ./outputs/l1-hook.log
22aa309fe642380be9da24d371d7576bbb89c1d9d5d2fd33e5abb562a1bdb274  ./outputs/l1-manifest.log
d519d6ce45a95ac6c7061638101a52ef621bdb3aef7d1002259a6d71d0ec2020  ./outputs/l1-p5.log
a0ce31f758e1a0794e53e5133b8332f7b83334cd215a869720dee2aafead10aa  ./outputs/marker-fields.log
2f1daffa03350bd9598ef56629824d53cd9d62da8e85066ccec9cfc6a9d98616  ./outputs/rerun-summary.txt
780ae69d1047b234227a19563fdea621dacc443ea4841829efe1e40ead7ef714  ./outputs/semantics.log
4b8f4e762673b72d2de681ab0707e9b093f3587707a3aefa9fbfb5c5033cdcba  ./outputs/systemd-run.log
15c420a5c6257a293abd456205ed61fe71b9c5e66a52b0f36287d0e48a23971c  ./probe_common.py
8746a5e3079963cc7dc60e1394ae60fffde67228d5d2d1c4e85d2ac128168601  ./probe_d2_trace.py
62df1ab72650d0984936ffb741b1aa0704ee97a6c2713a22c7bb3d0cc4844e4a  ./probe_l1_attrs.py
f4ba99fc64664f65ae648cce602f63e1e2c393a06ec8794cc3577178e3a98815  ./probe_l1_hook.py
db4d920991a90888e41a1510a7cd5f2230217f37465a2ff700c73e3a5675b1c0  ./probe_l1_manifest.py
c99e39f5529b632e186556ad83e69ad636a452b7af6068507ef885890b61327c  ./probe_l1_p5.py
7b9a7846ef441513f33b23a993f7ffc06fb3ede354f09b4b301fe35fadd5bbf5  ./probe_marker_fields.py
980b9b73ab377ff50818b9632f9d50bb3201abad2fadcef8c1209450c269d395  ./rerun.sh
246e838b0a02c24e3839a7e4a4a5a93fd0cb10341daf1789decdb05c3ca14ff7  ./verify_semantics.sh
ba3b4771d8b6eeff19d19ce06ae03ab2e4e649578e4555c397b3cd17c0480cce  ./verify_systemd_run.sh
```

格的写法：ATTACK 是该拒 / 该变的写法，BROKEN = 打中；CONTROL 是对照（今天应当成立，0 格坏）。语义（libtest、cargo、rustc、systemd-run 真这样跑）在 `verify_semantics.sh`（草稿目录里自己写的小 crate，经 `run-with-memory-cap.sh 2G`）与 `verify_systemd_run.sh` 上量，行首 `SEM`。

## 各格判定一览

| 格 | 判定 | 打中的写法（格名见 outputs/*.log） | 新旧 |
|---|---|---|---|
| P3 `--list` | **打中（量过），假绿** | 第二个 `--` 之后的 `--list` 是 libtest 的过滤词，登记的全量用例照跑（SEM P3a），闸放行（T5、T6、T7） | P3 之后仍在：P3 只跳过带值选项的值，没认 `--` |
| P10 `--config` 按 TOML 读 | **打中（量过），假绿** | `--config` 写在 `test` 之后（cargo 的全局选项，照认，SEM P10a/P10b），闸不读它的值（R9、R10、R11） | P10 之前就有；P10 只改了「读」，没改「从哪里收」 |
| P2 设给里面那条命令的环境 | **打中（量过），假绿** | `systemd-run --wait --pipe -p Environment=<runner>` / `--property=Environment=`（SEM S1 真设进去），闸放行（S1、S2） | P2 只认 `-E` / `--setenv` |
| P1 剥包装 | **打中（量过），假绿，同一类第二个实例** | 要值的长选项、值另起一个词：`systemd-run --expand-environment no <命令>`（SEM S3 真这样起），闸把 `no` 当命令（S3）；`strace --output <文件>`（S4）是 G2「看到但没做」第 3 条记过的同一类，今天仍放行 | 短选项合写（P1 本身）没打中 |
| P5「判不出按没标算」与第二轮判的代价「方向是多拒」 | **打中（量过），代价的方向说反了一半** | 准入模块**导入不了**时闸拒（对照 B）；**导入得了、判的那一刻抛异常**（函数改了名、闸调的名字没跟上）时闸退 0 放行、检出一条都不记（C、D） | P5 新写的那一支只管 `module is None` |
| P4 / P4b 按每一处 `fn <名>(` 判 | **打中（量过），假绿，P4 之前就有**；P4 这一改动本身只多拒（成立） | debug 下标 ignore、release 下 `include!` 进来一份不标的同名用例（SEM P4 真跑），闸放行、`crash-cases` 自查退 0（I1、I2） | 实现只读测试目标自己的源文件 |
| P6 / P7 算出来的 include 按宽处理 | **打中（量过），假复用** | `use core::include_str as grab;` 改名后 `grab!(concat!(…))`（SEM P6 编得过、读得到），`mutations.tsv`、`src/bin/`、别的测试目标照减，改它们指纹不变（N1、N2、N3） | 正则按宏名认 |
| P9 runner 参数按内容进指纹 | **打中（量过），假复用；G1 报告自己写的推翻条件成立** | cargo 起 runner 的当前目录是包目录（SEM P9、P9b）；能跑的写法 `sh ../../tools/r.sh`，准入模块从 .cargo 那一层 / 仓根解、指不到，改脚本指纹不变（P9a、P9b） | P9 新写的解法 |
| D2 判法摘要 | **今天没打中（量过）**；**以后打中（量过，今天仓里没有这种写法）**；**54 号那句字面不成立（量过）** | ① 七趟判法子命令运行时用到的 51 个模块级定义全在 52 个的静态闭包里；② 元组解包赋值、`if` 块里的 `def`、分派表的值写成 lambda、判法挪进 import 的别的模块，四种先改写法、再改判法，第二步摘要不变；③ 同一份「64 片 1 线程」的日志，54 号转过来的 `--threads` / `--threads-origin` 一换，判红变判绿 | ② 四种都不在文件头「看不见的」里 |
| 标记字段 | **打中（量过），危害小**（只误导字面、不改覆盖） | `crash-injection-fast-tier` 读 `SINGLEFS_CRASH_INJECTION_THREADS`，标记写 `configured_worker_threads=SINGLEFS_LAYER0_THREADS=…`；调用方的 `SINGLEFS_CRASH_INJECTION_THREADS=1` 漏进 `--full`；`OMP_NUM_THREADS=1` 让交的核数变 1、64 片 1 线程判绿（①②③） | ③ 取 nproc 认 OMP 是代码注释写明的取舍 |
| Y1（快档与 `--full` 同一个函数算） | **站得住（推的，读代码）** | 驱动三处与 54 号同参数，`--merged-log` 比 54 号给的指纹 | — |
| K3 floor-raise（第 38 行） | **站得住（推的，读代码）** | `crash-case-command` 清分片开关；片数守恒在 `shards=` 分支之前判 | — |
| J1-c（两趟并发只会假红） | **站得住（推的，读代码）**；定义里挡并发那一句没点名单独跑的驱动 | 驱动与 54 号都只在判红时删标记；共用进度目录的争用落在 crates 的账本核对里 | — |

攻方自己的改法 F1、F2、F3、F5、F10 只在我的模型上量过、被攻过零轮（见「改法一览」一节）；P4、P6、P9、D2、标记字段的改法都是推的。

## 一、闸那一侧：P3、P10、P2、P1（`probe_l1_hook.py`，`outputs/l1-hook.log`）

全部以子 agent（implementation-writer）身份喂仓里的 `heavy-test-guard.sh`，命令点名登记的 c561 目标（`record_checker_judges_absence_by_the_persisted_set`，用例函数标了 `#[ignore]`），外面套内存包装。17 格原样判定（第 4 列截到退出码）：

```
CONTROL	holds	T1 -- --ignored --list（只列，前两轮对照） ⇒ 应当放行	退 0
CONTROL	holds	T2 -- --include-ignored --skip --list（第二轮 T2，P3 修的那一格） ⇒ 应当拒	退 2
CONTROL	holds	T8 -- --include-ignored <全量用例>（不带 --list） ⇒ 应当拒	退 2
ATTACK	BROKEN	T5 -- --include-ignored <全量用例> -- --list（第二个 -- 之后 --list 是过滤词，libtest 照跑全量） ⇒ 应当拒	退 0
ATTACK	BROKEN	T6 -- --include-ignored --exact <全量用例> -- --list ⇒ 应当拒	退 0
ATTACK	BROKEN	T7 直接执行测试二进制 --include-ignored <全量用例> -- --list ⇒ 应当拒	退 0
CONTROL	holds	T9 -- --ignored -- --list（只剩过滤词 --list，一条都不跑：放行是对的） ⇒ 应当放行	退 0
CONTROL	holds	R6 cargo --config <runner> test …（第二轮对照） ⇒ 应当拒	退 2
ATTACK	BROKEN	R9 cargo test --config <runner> -p … --test <登记目标>（--config 写在 test 之后） ⇒ 应当拒	退 0
ATTACK	BROKEN	R10 cargo test -p … --test <登记目标> --config=<runner> ⇒ 应当拒	退 0
ATTACK	BROKEN	R11 cargo test --config 'target.x86_64-unknown-linux-gnu = { runner = … }'（行内表写在 test 之后） ⇒ 应当拒	退 0
CONTROL	holds	R2 systemd-run -E 设 runner（第二轮 R2，P2 修的那一格） ⇒ 应当拒	退 2
ATTACK	BROKEN	S1 systemd-run --wait --pipe -p Environment=<runner>（service 单元的 Environment= 属性） ⇒ 应当拒	退 0
ATTACK	BROKEN	S2 systemd-run --wait --pipe --property=Environment=<runner> ⇒ 应当拒	退 0
CONTROL	holds	L5 systemd-run --user --scope -q -u k1（前两轮对照） ⇒ 应当拒	退 2
ATTACK	BROKEN	S3 systemd-run --user --scope --expand-environment no <命令>（值另起一个词） ⇒ 应当拒	退 0
ATTACK	BROKEN	S4 strace --output <文件> <命令>（G2「看到但没做」3 记过的同一类，今天仍放行） ⇒ 应当拒	退 0
```

语义（`outputs/semantics.log`、`outputs/systemd-run.log` 原样；小 crate 里 `own_case` 有一条标 ignore 的 `the_case` 与一条快用例 `quick`，`the_case` 跑到就落一个文件）：

```
SEM	P3c-control	not-run	rc=0；test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
SEM	P3a	ran	rc=0；test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
SEM	P3b	not-run	rc=0；test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
SEM	P10a	ran	rc=0；ADD_IGNORED_RUNNER cwd=<小仓>/crates/pkg；test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
SEM	P10b	ran	rc=0；test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
SEM	S1	seen-by-the-unit	rc=0
SEM	S3	expand-environment-two-word-form-ran	rc=0
```

P3a 是 `cargo test -p pkg --test own_case -- --include-ignored the_case -- --list`：libtest 在第一个 `--`（libtest 自己那一层）处停止认选项，之后的 `--list` 成了第二个过滤词，`the_case` 照跑。P3b（`-- --ignored -- --list`）只剩过滤词 `--list`，一条都不跑，所以 T9 放行是对的。P10a 是 `cargo test -q --config "target.x86_64-unknown-linux-gnu.runner=…" -p pkg --test own_case`（不带 `--ignored`），runner 替测试二进制加了 `--include-ignored`，2 passed、0 ignored。

为什么放行（引文原样）：
- P3：闸找 `--list` 时只跳过带值选项的值，见到 `--` 不停：「if argument == LIST_ONLY_TEST_ARGUMENT:」（`.claude/hooks/lib_heavy_tests.py:114`）、「position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1」（`.claude/hooks/lib_heavy_tests.py:116`）；cargo 那一支只切第一个 `--`：「libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []」（`.claude/hooks/lib_heavy_tests.py:470`），直接执行那一支同样：「if libtest_lists_only(arguments):」（`.claude/hooks/lib_heavy_tests.py:679`）。
- P10：`--config` 的值只在子命令之前收：「if argument == "--config" and index + 1 < len(arguments):」（`.claude/hooks/lib_heavy_tests.py:320`）；子命令之后的 `--config` 在 cargo test 的带值选项表里，只当一个值跳过：「"--message-format", "--config", "-Z", "--lockfile-path",」（`.claude/hooks/lib_heavy_tests.py:288`）。
- P2：设给里面那条命令的环境只认两张选项：「LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}, "strace": {"-E", "--env"}}」（`.claude/hooks/lib_heavy_tests.py:606`）。
- P1：长选项按整词查表，不在表里的当不带值：「return (option, next_word, 2) if option in options_with_value else (option, "", 1)」（`.claude/hooks/lib_heavy_tests.py:621`）；`--expand-environment` 不在 systemd-run 那一格，`--output` 不在 strace 那一格：「"strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X"},」（`.claude/hooks/lib_heavy_tests.py:597`）。S4 这一类 G2 已记：「strace 的别的长选项（`--output=`、`--user=` 等）不在 `LAUNCHER_OPTIONS_WITH_VALUE` 里」（`research/prompts/defs-gatebatch-m2-g2-report.md:198`）。

条款：「libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算」（`.claude/rules/implementation-workflow.md:60`）——T5–T7 里的 `--list` 不是当选项出现，照条款算重型；「`--config` 或环境变量里定了测试二进制的 runner」（`.claude/rules/implementation-workflow.md:58`）——R9–R11 是 `--config` 定的 runner，S1、S2 是设给里面那条命令的环境变量定的 runner。

打中之后的四句：

| 打中 | 分不分辨臂 | 闸当时看不看得到 | 满足条款字面的哪一分句 | 第二轮给的改法在这几格上 |
|---|---|---|---|---|
| T5–T7 | 分辨「跳过带值选项的值」（P3 今天的写法）与「照 libtest 的 getopts 解析，`--` 之后不再认选项」 | 看得到：`--` 就在命令文本里 | 第 60 行「`--list` 当选项出现…不算」的反面：不是当选项出现 | P3 照中（它就是被打中的那一处） |
| R9–R11 | 分辨「只收子命令之前的 `--config`」与「全局选项哪里都收」 | 看得到 | 第 58 行「`--config`…里定了测试二进制的 runner」 | P10 照中：它改的是怎么读，收的仍只有子命令之前的 |
| S1、S2 | 分辨「按选项名列举设环境的写法」与「systemd-run 起 service 时一切属性都可能设环境」 | 看得到（`Environment=` 前缀是字面） | 第 58 行「环境变量里定了测试二进制的 runner」（`-E` / `--setenv` 只是点名的两种） | P2 照中 |
| S3、S4 | 不分辨 P1（短选项合写）的对错，打的是长选项表的完整性 | 看得到，但要知道每个包装程序每个长选项要不要值 | 第 60 行「`systemd-run`、`strace`…包在外面的剥掉照算」 | P1 照中（P1 不管长选项）；S4 G2 已记，没修 |

什么现象推翻：cargo 1.98 在 `test` 之后不认 `--config`（SEM P10a/P10b 量过它认）；libtest 在第二个 `--` 之后仍把 `--list` 当选项（SEM P3a 量过它当过滤词）；闸之外另有一道在进程那一层拦住这几种（看门狗与闸共用 `classify`，推的：同样认不出）。

## 二、P5 与 P4：第二轮判给它们的代价「方向是多拒」

### P5（`probe_l1_p5.py`，`outputs/l1-p5.log`）

镜像仓：`.claude/hooks` 拷真仓那三份，`.claude/scripts`、`.claude/singlefs-ai-sop` 链到真仓，`research/scripts/admission.py` 是真仓那一份或它的变体；仓里登记一条崩溃枚举用例，用例函数**没标** `#[ignore]`（不带 `--ignored` 的 `cargo test` 真跑全量，闸该拒）。原样：

```
CONTROL	holds	A 准入模块原样；用例函数没标 ignore、不带 --ignored ⇒ 应当拒	闸退 2，检出记了 1 条；stderr：✗ 重型测试被拒：cargo test --test own_case 不带 --ignored，而登记的用例函数 the_case 没标 #[ignore]，照样跑到全量（own_case）（崩溃枚举用例）：implementation-writer 不跑「崩溃枚举用例」 ｜ → 规矩：重型测试（
CONTROL	holds	B 导入不了（文件尾加一行语法错）；用例函数没标 ignore、不带 --ignored ⇒ 应当拒	闸退 2，检出记了 1 条；stderr：✗ 重型测试被拒：cargo test --test own_case 不带 --ignored，而登记的用例函数 the_case 没标 #[ignore]，照样跑到全量（own_case）（崩溃枚举用例）：implementation-writer 不跑「崩溃枚举用例」 ｜ → 规矩：重型测试（
ATTACK	BROKEN	C 导入得了、判的那一刻抛 NameError（definitions_marked_ignored 改名，调它的一处没跟上）；用例函数没标 ignore、不带 --ignored ⇒ 应当拒	闸退 0，检出记了 0 条；stderr：! heavy-test-guard.sh 没判成（NameError("name 'definitions_marked_ignored' is not defined")），这条命令照常执行
ATTACK	BROKEN	D 导入得了、闸调的 test_function_is_marked_ignored 改了名（AttributeError）；用例函数没标 ignore、不带 --ignored ⇒ 应当拒	闸退 0，检出记了 0 条；stderr：! heavy-test-guard.sh 没判成（AttributeError("module 'admission_for_heavy_tests' has no attribute 'test_function_is_marked_ignored'")），这条命令照常执行
CONTROL	holds	C 那种坏法下带 --include-ignored ⇒ 应当拒（不经准入模块）	闸退 2；stderr：✗ 重型测试被拒：cargo test --test own_case 带 --include-ignored，跑到登记的崩溃枚举用例（own_case）（崩溃枚举用例）：implementation-writer 不跑「崩溃枚举用例」 ｜
```

为什么：P5 只管导入不了那一支：「if module is None or package_directory is None:」（`.claude/hooks/lib_heavy_tests.py:217`）；导入得了之后直接调：「marked = module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function)」（`.claude/hooks/lib_heavy_tests.py:219`），抛出来的异常一路到闸的入口，入口放行：「except Exception as error:」（`.claude/hooks/heavy-test-guard.sh:1152`）、「没判成（{error!r}），这条命令照常执行」（`.claude/hooks/heavy-test-guard.sh:1153`），这一支不写检出记录。

第二轮判决写的代价：「代价：准入模块坏了时点名登记目标的 `cargo test` 全被拒——方向是多拒，接受」（`research/prompts/defs-gatebatch-m2-r2-main-verification.md:33`）；闸的文件头同样只写了导入不了那一半：「admission.py 导入不了时点名登记目标的 cargo test 一律拒」（`.claude/hooks/heavy-test-guard.sh:88`）。坏法分两种，只有导入不了那一种多拒；导入得了、判的时候抛异常那一种是放行，而且看门狗读的检出记录里没有它。今天仓里那一份没坏（A 退 2）；走到这一支要准入模块在别的会话改到一半（G1、G2 两批都在主工作区里改过 `admission.py`），或 `lib_heavy_tests.py` 与 `admission.py` 不同步地改了名、改了签名。

四句：分辨「导入不了」与「判的时候抛」两种坏法；闸看得到（它当场接住了异常）；满足的是条款「或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）」（`.claude/rules/implementation-workflow.md:58`）里「判不出」那一句：抛异常就是判不出；P5 照中，F5 修它（见「改法一览」，副本上量过 C、D 转拒）。推翻：真仓 `admission.py` 的判法函数在任何时刻都不会抛（它改动频繁，推的：不成立）；或看门狗在进程那一层把这种命令另判成重型（没量）。

### P4 / P4b（`probe_l1_attrs.py`，`outputs/l1-attrs.log`）

小仓：`tests/own_case.rs` 里 debug 下有一份标 ignore 的 `the_case`，release 下 `include!` 进来 `tests/common/release_case.rs` 里一份不标的同名用例。原样：

```
CONTROL	holds	V3 同名 cfg 二选一、release 那一份不标（第二轮 V3，P4 修的那一格）：闸（cargo test --release，不带 --ignored） ⇒ 应当拒	闸退 2
CONTROL	holds	V3 同名 cfg 二选一、release 那一份不标（第二轮 V3，P4 修的那一格）：crash-cases 自查 ⇒ 应当退 2	自查退 2：.claude/gate.d/stage-inputs.tsv 第 1 行：崩溃枚举用例 crash-case:own 的用例函数 the_case 有 1 处定义没标 #[ignore]（crates/pkg/tests/own_case.rs；共 2 处，cfg 二选一的、子
ATTACK	BROKEN	I1 debug 下标 ignore；release 下 include!("common/release_case.rs") 进来一份不标的同名用例：闸（cargo test --release，不带 --ignored） ⇒ 应当拒	闸退 0
ATTACK	BROKEN	I1 debug 下标 ignore；release 下 include!("common/release_case.rs") 进来一份不标的同名用例：crash-cases 自查 ⇒ 应当退 2	自查退 0：crash-case:own	pkg	own_case	the_case
ATTACK	BROKEN	I2 同 I1，include! 的参数换成 concat!("common/", "release_case.rs")：闸（cargo test --release，不带 --ignored） ⇒ 应当拒	闸退 0
ATTACK	BROKEN	I2 同 I1，include! 的参数换成 concat!("common/", "release_case.rs")：crash-cases 自查 ⇒ 应当退 2	自查退 0：crash-case:own	pkg	own_case	the_case
CONTROL	holds	I3 只有 include! 进来的一份（测试目标里没有字面的 fn the_case(）：判不出，按没标算（P5）：闸（cargo test --release，不带 --ignored） ⇒ 应当拒	闸退 2
```

语义：`SEM	P4	ran	rc=0；test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`（`cargo test --release -p pkg --test inc_case`，不带 `--ignored`，include 进来那一份跑了）。

为什么：单文件的测试目标只读它自己：「single_file = os.path.join(package_directory, "tests", target + ".rs")」（`research/scripts/admission.py:1232`）、「return [single_file]」（`research/scripts/admission.py:1235`）。而准入模块文件头写的是「同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；」（`research/scripts/admission.py:26`）。

「方向是多拒」对 P4 这一改动本身成立：P4 把「第一处标了」换成「每一处都标了」，放行的集合只缩不扩；P4b 认 `# [ignore]` 是放宽，rustc 认它（第二轮 semantics.log 量过），不造假绿。I1、I2 是 P4 之前就有的形状，P4 没修到：漏在「每一处」只数了测试目标自己的源文件。四句：不分辨 P4 与第一处判法（两种都漏）；准入模块看得到（`include!("…")` 的字面参数就在目标文件里，`concat!` 那种按宽处理即可）；满足第 58 行「登记的用例函数有一处定义没标 `#[ignore]`」；P4、P4b 在 I1、I2 上照中。今天仓里 `crates/` 下的 include 族只有一处（`grep -rnE 'include(_str|_bytes)?\s*!' crates --include=*.rs` 现跑一行：`crates/singlefs-harness/src/model_comparison.rs:943` 的 `include_str!("model.rs")`，不在测试目标里），走到的可能性低。推翻：rustc 不许在 `#[cfg]` 下 `include!` 一个带 `#[test]` 的文件（SEM P4 量过它许）。

## 三、清单与指纹：P6 / P7、P9（`probe_l1_manifest.py`，`outputs/l1-manifest.log`）

小 git 仓（包 pkg、用例 `own_case::the_case`，登记 `crates/ Cargo.toml`），跑仓里被判的 `admission.py crash-case-manifest … --build-environment`，改一份用例其实读得到的文件，看指纹。原样：

```
CONTROL	holds	M4r include_str!(concat!(…拆开写的 mutations.tsv…))（P7 修的那一格） ⇒ 改 crates/mutations.tsv 指纹应当变	改前 40c03309ca566943… 改后 8c501f1fe3cff334…（文件数 / 减去数 改前 7/0 改后 7/0）
ATTACK	BROKEN	N1 use core::include_str as grab; grab!(concat!(…拆开写的 mutations.tsv…)) ⇒ 改 crates/mutations.tsv 指纹应当变	改前 14aacffa15134d61… 改后 14aacffa15134d61…（文件数 / 减去数 改前 4/3 改后 4/3）
CONTROL	holds	M5r include_str!(concat!(…拆开写的 src/bin/tool.rs…)) ⇒ 改 crates/pkg/src/bin/tool.rs 指纹应当变	改前 8ce3383145999490… 改后 444ff5d1bc51d80d…（文件数 / 减去数 改前 7/0 改后 7/0）
ATTACK	BROKEN	N2 改名的 grab!(concat!(…拆开写的 src/bin/tool.rs…)) ⇒ 改 crates/pkg/src/bin/tool.rs 指纹应当变	改前 374813ee55c6809b… 改后 374813ee55c6809b…（文件数 / 减去数 改前 4/3 改后 4/3）
CONTROL	holds	M1r include_str!(concat!(…拆开写的 tests/other.rs…))（这个包的测试文件一份都不减） ⇒ 改 crates/pkg/tests/other.rs 指纹应当变	改前 60afa3078cc0df8f… 改后 58a079689239e10b…（文件数 / 减去数 改前 7/0 改后 7/0）
ATTACK	BROKEN	N3 改名的 grab!(concat!(…拆开写的 tests/other.rs…)) ⇒ 改 crates/pkg/tests/other.rs 指纹应当变	改前 aa91cc3e71e953a3… 改后 aa91cc3e71e953a3…（文件数 / 减去数 改前 4/3 改后 4/3）
CONTROL	holds	G1a runner = "sh <仓的绝对路径>/tools/r.sh" ⇒ 改 tools/r.sh 指纹应当变	改前 57debf681d8996b8… 改后 70443b26ab78effc…
CONTROL	holds	G1b runner = "sh tools/r.sh"（从 .cargo 那一层解；照 SEM-P9，cargo 在 crates/pkg 起它，这种写法跑不起来） ⇒ 改 tools/r.sh 指纹应当变	改前 15dcf03e3471a194… 改后 d839169e7e714b6e…（文件数 / 减去数 改前 4/3 改后 4/3）
ATTACK	BROKEN	P9a runner = "sh ../../tools/r.sh"（从包目录起，cargo 真能跑的写法） ⇒ 改 tools/r.sh 指纹应当变	改前 9d74e0c3104db913… 改后 9d74e0c3104db913…（文件数 / 减去数 改前 4/3 改后 4/3）
ATTACK	BROKEN	P9b 环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="sh ../../tools/r.sh" ⇒ 改 tools/r.sh 指纹应当变	改前 b2988232c88cd188… 改后 b2988232c88cd188…（文件数 / 减去数 改前 4/3 改后 4/3）
```

语义原样：

```
SEM	P6	RENAMED_INCLUDE_READ=mutation-table-content	rc=0
SEM	P9	RUNNER_SCRIPT_RAN cwd=<小仓>/crates/pkg	rc=0
SEM	P9b	RUNNER_SCRIPT_RAN cwd=<小仓>/crates/pkg	rc=0（环境变量写法 CARGO_TARGET_…_RUNNER="sh ../../tools/r.sh"）
```

P6 是 `use core::include_str as grab; const TABLE: &str = grab!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/mutations.tsv"));` 编得过、读到那份文件。P9 是小工作区根上 `.cargo/config.toml` 写 `runner = "sh ../../tools/r.sh"`，`cargo test -p pkg` 起的 runner 当前目录是 `crates/pkg`，脚本在仓根的 `tools/` 下被找到、跑了。

为什么：
- P6 / P7：认的是宏名：「COMPILE_TIME_COMPUTED_INCLUDE = re.compile(r'」（`research/scripts/admission.py:228`，整行是 `\binclude(?:_str|_bytes)?\s*!\s*[(\[{]\s*(?!b?r#*"|b?")\S` 那一条正则）；`use … as` 改了名，这一条碰不到，P7「一份都不减」那一支也就不开。N1–N3 把文件名拆开写，是为了绕开「按整词点名就留」那一条（M4 / M5 同形，第二轮量过）；拆开写本身 P7 已按宽处理（对照 M4r、M5r、M1r 都变），打中的只是改名那一步。文件头「认得出形状…按宽处理」那一句写的是「include_bytes! 的参数不以字符串字面量开头（套 concat!、env!、option_env! 或别的宏，宏名前带不带 ::core:: / std:: 都算）的，这个包的测试文件」（`research/scripts/admission.py:128`），改名不在「认不出的」清单里。
- P9：参数的相对路径从 base_directory 解：「candidate = argument if os.path.isabs(argument) else os.path.join(base_directory, argument)」（`research/scripts/admission.py:581`）；配置文件的 base_directory 是 .cargo 那一层，环境变量的是仓根：「program_contents(environment[variable].split(), root, environment)」（`research/scripts/admission.py:553`）。cargo 起 runner 时进程的当前目录是包目录（SEM P9、P9b），参数由 runner 自己按当前目录解。于是：能跑的写法（P9a、P9b）指纹里没有那份脚本；指纹里有那份脚本的写法（G1b）跑不起来。G1 报告自己写过这一条推翻条件：「runner 参数里的相对路径其实按测试二进制的工作目录（包目录）解」（`research/prompts/defs-gatebatch-m2-r2-fixes-report.md:243`），今天量到成立。

四句：
| 打中 | 分不分辨臂 | 准入模块看不看得到 | 满足条款哪一分句 | 第二轮改法在这几格上 |
|---|---|---|---|---|
| N1–N3 | 分辨「按宏名认」与「按展开之后读了什么认」；P6 与 P7 一起中 | 看得到 `use core::include_str as grab`（字面），要多认一层改名 | 「那一道读的全部输入自上次跑绿以来变没变」（`.claude/rules/implementation-workflow.md:32` 那一句）：读的文件变了而复用 | P6、P7 照中 |
| P9a、P9b | 分辨「从 .cargo 那一层 / 仓根解」与「从包目录解」 | 看得到：包目录由登记行的包名定 | 同上一格那一句；另打中准入模块文件头自己的写法「相对路径）接着按内容进（`runner = "bash tools/r.sh"` 这一类，定行为的是参数里的脚本）」（`research/scripts/admission.py:46`）——文件头举的例子正是 cargo 下跑不起来的那一种 | P9 照中 |

走到的可能性：仓里今天没有 `.cargo/config*`（`build_environment_lines` 读的那几份），也没有设 runner 的环境变量；两处都属第一轮 J3-a「今天走到的可能性低」那一类。推翻：cargo 起 runner 时把当前目录设在 .cargo 那一层（SEM P9 量过是包目录）。

## 四、D2 判法摘要（`probe_d2_trace.py`，`outputs/d2-trace.log`）

「改一处判法、看摘要变不变」分三件量，原样：

```
INFO	静态闭包 52 个定义（摘要里 ## 节 52 个），模块级定义共 232 个
INFO	七趟子命令的退出码 [0, 0, 0, 0, 0, 0, 0]（command、judge 单机、judge merge、record、marker-check、marker-path、shardable；0 是走到了判绿 / 作数那一支）
INFO	运行时执行到的函数 29 个、读到的模块级名字 45 个；合起来 51 个，其中闭包外的：无
ATTACK	holds	①今天：判法子命令运行时用到的模块级定义（main 之下）都在静态闭包里	闭包外的 无；…
ATTACK	BROKEN	②以后：元组解包赋值：PASSED_ONE_TEST_FORM 挪进 `A, B = …`，再把判绿的正则放宽成「test result: 」开头都算（FAILED 也判绿） ⇒ 第二步摘要应当变	原样 b62d8a74dd45b508… 第一步 a8392fded6b41eb9… 第二步 a8392fded6b41eb9…
ATTACK	BROKEN	②以后：if 块里的 def：fields_of_line 挪进 `if True:` 块，再让它把 exhaustive=false 读成 true ⇒ 第二步摘要应当变	原样 b62d8a74dd45b508… 第一步 c8d013dca586ba25… 第二步 c8d013dca586ba25…
ATTACK	BROKEN	②以后：分派表的值不是名字：crash-case-judge 那一项写成 lambda，再改 command_crash_case_judge 让它一律退 0 ⇒ 第二步摘要应当变	原样 b62d8a74dd45b508… 第一步 8f57aa3faacc9bcd… 第二步 8f57aa3faacc9bcd…
ATTACK	BROKEN	②以后：判法挪进 import 的别的模块：`from judging_helpers import fields_of_line`（再改那份模块，准入模块原文一个字不变） ⇒ 第二步摘要应当变	原样 b62d8a74dd45b508… 第一步 c8d013dca586ba25… 第二步 c8d013dca586ba25…
ATTACK	BROKEN	③同一份「64 片只起 1 个线程」的日志：判不判绿只取决于 54 号转过来的 --threads / --threads-origin（54 号不进指纹）	配的是 32（default） 退 1（本机 32 核、线程数没显式设成 1，这一趟跑了 64 片却只起了 1 个工作线程（多半是线程数没传进去）：LAYER0_PARALLEL_FINISHED states=100 ）；转成显式 1（explicit） 退 0（LAYER0：1 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片））
```

① 今天没打中：在小 git 仓上真跑判法那六个子命令（judge 单机日志与 merge 日志各一趟，merge 那一趟走 `judge_threads_of_each_shard`），`sys.settrace` 记下执行到的 29 个函数与它们读的 45 个模块级名字，合起来 51 个，全在静态闭包的 52 个里（`main` 是原文进摘要、不往下顺的那一格，它读的 `COMMANDS` 只有判法那六项进摘要，按设计另算）。`getattr`、字符串拼函数名、`exec` 今天在这几条路上都没有。

② 以后打中，四种都不在文件头「看不见的」那一句里：「靠 ast 里的静态引用求闭包，getattr、字符串拼出来的函数名、exec 这一类引到的定义不进；」（`research/scripts/admission.py:132`）。成因各一处：模块级赋值只认名字目标，「elif isinstance(node, (ast.Assign, ast.AnnAssign)):」（`research/scripts/admission.py:1877`）之下「if isinstance(target, ast.Name):」（`research/scripts/admission.py:1879`），元组解包的目标不进定义表；`top_level_definitions` 只扫 `tree.body` 的顶层语句，`if` / `try` 块里的 `def` 不进；分派表的值按原文进，「dispatch[key.value] = ast.unparse(value)」（`research/scripts/admission.py:1908`），再拿原文当名字入队，「queue = [name for name in CRASH_CASE_JUDGING_ENTRIES + tuple(dispatch.values()) if name in definitions]」（`research/scripts/admission.py:1913`），值是 lambda / `functools.partial` 时它指的函数不入队；`import` 进来的名字不在定义表里，那份模块的原文不进。四种都要先有人把判法改成那种写法（摘要跟着变一次，作废一次标记），之后再改判法才假复用；今天仓里一种都没有（① 的运行时集合全在闭包里）。

③ 54 号那一句字面不成立：文件头写「54 号不进指纹：它里面还定着结论的只剩流程的次序」（`research/scripts/admission.py:133`），54 号文件头同一句「这一份 54 号不进指纹：改它（出路句、快档、次序）不废旧标记。它里面还定着结论的只剩流程的次序」（`.claude/gate.d/54-layer0-replay.sh:40`）。实际上 54 号还把 `crash-case-command` 交的前三个词转给 `crash-case-judge`：「case_machine_cores="${command_words[0]}"」（`.claude/gate.d/54-layer0-replay.sh:245`），判法据它们免掉单线程那一格：「threads_explicitly_one = values["--threads-origin"] == "explicit" and values["--threads"] == "1"」（`research/scripts/admission.py:2184`）、「if judged_on_one_thread and machine_cores > 1 and not threads_explicitly_one:」（`research/scripts/admission.py:1621`）。③ 量的是：同一份日志，转过来的参数一换，判红变判绿。要 54 号先把这三个词转错（今天转得对），写下的绿标记才在修好 54 号之后照样作数。另两处 54 号自己定结论的：cargo 退非 0 判红（第 527–544 行那一段，`case_run_exit` 起），与双机分片开不开（第 468–473 行，调配置判法）；前者转错了判法还有「test result: ok. 1 passed」那一格兜着，后者只定怎么跑。

四句：② 不分辨 D2 与「整份模块进指纹」以外的写法——整份进指纹时四种都会变（弄坏开关 `whole-module-in-judging-digest` 那一格就是这一种），所以它分辨的正是 D2 与整份；准入模块看得到（四种都是 ast 里的字面结构）；满足第二轮判决 D2 那一行「限度写进文件头（`getattr`、字符串拼出的函数名看不见；Python 升级不进）」（`research/prompts/defs-gatebatch-m2-r2-main-verification.md:37`）之外的漏看；D2 照中。③ 分辨「54 号整份不进」与「54 号里转参数的那几行也进」；推翻：54 号今后不再转任何参数、判法自己调 `crash-case-command` 取线程数。

## 五、标记字段与线程来源（`probe_marker_fields.py`，`outputs/marker-fields.log`）

原样：

```
INFO	crash-case-command 退 0；前四个 ['32', '32', 'default']；命令：env -u SINGLEFS_LAYER0_SHARD -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<仓>/.git/singlefs-layer0-progress/0000000000000000000000000000000000000000000000000000000000000000 SINGLEFS_LAYER0_INPUT_FINGERPRINT=0000000000000000000000000000000000000000000000000000000000000000 SINGLEFS_LAYER0_THREADS=32 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_crash_injection -- --include-ignored --exact crash_injection_fast_tier_recovers_only_into_versions_the_model_committed --nocapture
ATTACK	BROKEN	① 调用方设着 SINGLEFS_CRASH_INJECTION_THREADS=1，照 crash-case-command 的命令起的这条用例 ⇒ 不应当看到 1（该清掉或设成配的线程数）	起的进程看到「SINGLEFS_LAYER0_THREADS=32 SINGLEFS_CRASH_INJECTION_THREADS=1」（退 0）
MARKER	configured_worker_threads=SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核
MARKER	CRASH_INJECTION_FINISHED seeds=[7,31) slices=24 worker_threads=1 elapsed_seconds=900.0
MARKER	started_worker_threads=读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；日志里有 LAYER0_PARALLEL_FINISHED 的另原样记在 parallel_finished=）
ATTACK	BROKEN	② 判 1 个线程跑完的日志、写标记 ⇒ configured_worker_threads= 不应当写一个这条用例不读的变量	judge 退 0、record 退 0；线程那几行见上面 MARKER
ATTACK	BROKEN	③ 调用方环境里 OMP_NUM_THREADS=1（没设 SINGLEFS_LAYER0_THREADS）⇒ crash-case-command 交的核数应当是本机的核数、64 片只起 1 个线程的日志应当判红	nproc --all 报 32；crash-case-command 交「核数 1、线程 1、default」；照它判 64 片 1 线程的日志退 0（LAYER0：1 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片））
```

① 用的是真仓登记表第 42 行（`crash-case:crash-injection-fast-tier`），`crash-case-command` 只读、不写（指纹给 64 个 0，进度目录只拼名字，现核没建出来）；起的是把 cargo 往后换成打变量的 `sh`，用例一条没跑。② ③ 在小仓上照第 42 行的登记（只有 `count-line=CRASH_INJECTION_FINISHED`）与一份带 `threads=` 的登记各判一次。

为什么：配的那一格按一个固定的变量名写：「return f"SINGLEFS_LAYER0_THREADS={threads}（{origin}），本机 {machine_cores} 核"」（`research/scripts/admission.py:1718`）；这条用例读的是另一个变量：「"SINGLEFS_CRASH_INJECTION_THREADS";」（`crates/singlefs-harness/src/crash_injection.rs:71`），`crash-case-command` 只清分片开关与续跑开关：「shard_setting = [] if break_is_set("keep-caller-shard-switch") else ["-u", LAYER0_SHARD_VARIABLE]」（`research/scripts/admission.py:1863`）。本机核数取 nproc：「没设取本机核数；本机核数取 nproc（在 environment 的 PATH 里找，它认 OMP_NUM_THREADS 与 CPU 亲和）。」（`research/scripts/admission.py:1831`）——这是代码注释写明的取舍，规则那一句写的是「机器多于 1 核却只用了 1 个线程，判红」（`.claude/rules/implementation-workflow.md:89`）。

危害：①② 只误导标记的字面与线程数，不改覆盖（快档的种子数、步数是用例里的常量）；`research/scripts/capped.sh` 会把 `SINGLEFS_CRASH_INJECTION_THREADS` 与 `SINGLEFS_LAYER0_THREADS` 设成同一个数（它的变量表第 25、26 行），经它跑时两格碰巧一样。③ 仓里没有脚本设 `OMP_NUM_THREADS`（`grep -rn OMP_NUM_THREADS` 现跑，命中的只有 `admission.py` 那一行注释与背景材料），走到要调用方环境漏进来。

四句：①② 判别子观测不到——准入模块不知道一条用例读哪个线程变量，登记行里没有这一格；要么登记行加一格，要么这一格改写成「配的：SINGLEFS_LAYER0_THREADS=…（这条用例读不读它，门禁不知道）」。③ 看得到（`nproc --all` 或 `os.cpu_count()` 不认 OMP）；满足第 89 行「机器多于 1 核却只用了 1 个线程」；标记字段的改法（分两格记）在 ③ 上照中，③ 另记一句「本机 1 核」是假的。

## 六、前两轮站住的形态：Y1、K3 floor-raise、J1-c（读代码，推的）

**Y1（快档与 `--full` 同一个函数算指纹）站得住。** 第二轮判它：「**没打中**：快档与 `--full` 都经 `write_crash_case_manifest` 同一个函数算」（`research/prompts/defs-gatebatch-m2-r2-main-verification.md:14`）。G2 之后多了第三个写标记的人（驱动单独跑），它不经 `write_crash_case_manifest`，自己调同一个子命令、同三个参数：「--judging-digest --toolchain --build-environment)" || fail "本机算不出输入指纹」（`research/scripts/layer0-shard-run.sh:173`），与 54 号「--judging-digest --toolchain --build-environment)"; then」（`.claude/gate.d/54-layer0-replay.sh:197`）逐词相同；`--merged-log` 时再与 54 号给的比：「if [[ "$layer0_shard_mode" == merged-log && "$local_fingerprint" != "$given_fingerprint" ]]; then」（`research/scripts/layer0-shard-run.sh:180`）。两边参数写了两份，以后改一边不改另一边只会对不上（假红）。推翻：驱动与 54 号在同一棵树、同一环境里算出不同的指纹。

**K3 floor-raise（登记表第 38 行）站得住。** 第二轮判它：「**没打中**：只跑了部分切片的日志判红」（`research/prompts/defs-gatebatch-m2-r2-main-verification.md:21`）。G1 之后单机那一趟由 `crash-case-command` 起，分片开关一律清掉（上一节第 1863 行），这条用例走不到只跑一片；片数守恒仍在 G2 新加的分片分支之前判：「if resumed_slices + freshly_run_slices != slices:」（`research/scripts/admission.py:1610`）先于「if "shards" in fields and not break_is_set("threads-ignore-shards"):」（`research/scripts/admission.py:1615`）。它没登记 `shard=`，驱动拒它（驱动第 93–94 行调 `crash-case-shardable`）。推翻：有一种起法让第 38 行这条用例的日志带上 `shards=`。

**J1-c（两趟并发只会假红）站得住。** 54 号文件头：「两趟 --full 同时跑同一条用例、同一个指纹时这里不加锁：后跑完的那一趟判红会删掉先跑完的那一趟刚写的绿标记，只会假红、不会假绿」（`.claude/gate.d/54-layer0-replay.sh:20`）。G2 之后驱动单独跑也会写、删同一组标记（驱动第 71–78 行只在判红时删，第 311 行判绿才写），标记写法是同目录临时文件再改名；共用的是进度目录（本机 `<common-dir>/singlefs-layer0-progress/<指纹>` 由 54 号单机、驱动的第 0 片与 merge 三处共用），账本的争用落在 crates 的续跑核对里（不归这一轮）。挡并发的那一句只点名 54 号：崩溃验证员第 1 步「另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号」（`.claude/agents/crash-verifier.md:27`），没点名单独跑的 `layer0-shard-run.sh`；不改变「只会假红」。推翻：两趟共用一份账本时，一趟读回另一趟写了一半的片而判绿（要看 `crates/singlefs-harness/src/layer0_progress.rs` 的核对，没攻）。

## 七、改法一览（攻方提的，只在我的模型上量过、被攻过零轮）

闸那几条（F1、F2、F3、F5、F10）打在 hooks 的副本上：`make_fix_copy.sh` 把真仓 `.claude/hooks` 三份拷进草稿目录 `fixcopy/`，给 `lib_heavy_tests.py` 打 `fix-lib-heavy-tests.diff`（59 行），其余链到真仓；再拿同两份探针喂副本的闸（`outputs/fix-l1-hook.log`、`outputs/fix-l1-p5.log`），副本上的 `heavy-test-guard.sh --selftest` 照跑（`outputs/fix-guard-selftest.log`）。副本上的数，不算入库装置上的数。

| 改法 | 做什么 | 修哪几格 | 在打中的格上 | 标注 |
|---|---|---|---|---|
| F3 | `libtest_lists_only` 见到 `--` 就停、交「不是只列」 | T5、T6、T7 | 副本上三格 `ATTACK holds`（退 2）；**另把 T9 判成重型**（`CONTROL BROKEN T9 … 退 2`）：`-- --ignored -- --list` 其实一条都不跑，F3 多拒它，与第 88 行那几条「接受的误拒」同类 | 量过 |
| F10 | `cargo_subcommand` 把子命令之后、`--` 之前的 `--config` / `--config=` 的值也收进 `configuration_values` | R9、R10、R11 | 副本上三格 `ATTACK holds`（退 2） | 量过 |
| F2 | systemd-run 的 `-p` / `--property` 值以 `Environment=` 开头时，里面的 `NAME=VALUE` 写在交回那条命令最前面 | S1、S2 | 副本上两格 `ATTACK holds`（退 2）；`EnvironmentFile=` 这一种没做（值是文件，推的：要按内容读） | 量过（`EnvironmentFile=` 推的） |
| F1 | systemd-run 那一格加 `--expand-environment`，strace 那一格加 `--output` | S3、S4 | 副本上两格 `ATTACK holds`（退 2）；别的包装程序要值的长选项没逐个补（推的：要照各自的 `--help` 列全） | 量过（补全推的） |
| F5 | `crash_case_function_runs_without_ignored` 调准入模块时接住异常，按「判不出」算 | P5 的 C、D | 副本上两格 `ATTACK holds`（闸退 2），对照 A、B 与带 `--include-ignored` 那一格照旧 | 量过 |
| F4 | 判 `#[ignore]` 时顺着测试目标里 `include!` / `#[path]` 的字面路径把那几份也读进来；`include!` 的参数不是字面量的整条按「判不出」 | I1、I2 | 没实现 | 推的 |
| F6 | 算出来的 include 另认 `use … include_str as <名>;` / `include_bytes` / `include` 改名之后的 `<名>!(` | N1、N2、N3 | 没实现；换成「任何 .rs 里有 `use` 引进 include 族宏」就按宽处理更省事，代价是那种写法出现时多重跑 | 推的 |
| F9 | runner / wrapper 参数里的相对路径另从每个登记用例的包目录解一遍，两处都在的都进 | P9a、P9b | 没实现；G1b 那种跑不起来的写法照旧进指纹，只多不少 | 推的 |
| FD2 | 判法摘要里：模块级语句不是 def / class / 名字赋值的（元组解包、`if` / `try` 块、`for`）整条原文进摘要；分派表的值不是名字时拒算（抛 InputManifestError，照 54 号判红）；判法入口的闭包里出现 `import` 进来的非标准库名字时拒算 | ② 四格 | 没实现 | 推的 |
| FD3 | 54 号不再转线程参数：`crash-case-judge` 自己按 `crash_case_worker_threads` 现取核数与线程数（或 `crash-case-record` / `judge` 都只收 `crash-case-command` 那一趟写的一份文件） | ③ | 没实现 | 推的 |
| FM | 标记里配的那一格写成「SINGLEFS_LAYER0_THREADS=…（这条用例读不读它，门禁不知道）」，或登记行加一格点名用例读的线程变量、由 `crash-case-command` 设它；本机核数改取 `nproc --all` / `os.cpu_count()` 与 CPU 亲和取小，不认 OMP_NUM_THREADS | 标记字段 ①②③ | 没实现 | 推的 |

`fix-l1-hook.log` 的汇总行 `SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=0 control_cells_broken=1`（坏的那一格就是 F3 多拒的 T9），`fix-l1-p5.log` 的 `SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=0 control_cells_broken=0`，副本自证末行 `✓ 自检通过（查了 661 种，其中该拒 122 种）` 与仓里那一份的种数相同（F1–F10 没给副本加自证格，也没配弄坏开关）。

## 没打中的形状（试过的与取样范围）

- P1 短选项合写：前两轮 L2、L4、L6、L7、L13 的五个形状在今天的闸上照拒（对照 L5 退 2）；另读了 `LAUNCHER_OPTIONS_WITH_VALUE` 每一格的短字母，与 util-linux 2.39.3 的 `flock` / `chrt` / `prlimit`、strace 6.8、GNU time 的要值短选项逐个对过，没找到「表里当要值、程序里其实不要值」的字母（会把命令词吞掉）。`chrt` 2.39.3 的优先级是必填的（`chrt --help` 现跑），`LAUNCHER_POSITIONAL_COUNT` 那一格对。
- P2 的 `-E NAME`（不带 `=`，取 systemd-run 自己的环境）：同一条命令里 `export` 过的变量本来就随 `command.environment` 往里传，没另喂。
- P3 的别的写法：`-qZ --list`（`-Z` 的值是 `--list`，libtest 当不稳定选项报错、不跑）；`--skip=--list`（一个词）照判。
- P4b：`#![ignore]`、`#[ignored]`、`#[ignore(…)]` 都不认成标了，对。
- P6：`include_str ! ( … )`（空白）、`include_str![…]` / `{…}`（别的定界符）、`::std::include_str!`、`r#include_str!` 都被正则认成算出来的或字面量，照对。
- D2：`getattr`、字符串拼函数名、`exec` 在判法那六个子命令的运行路径上一处都没有（① 的运行时集合）。
- G2 第 6 条清分片开关、`crash-case-shardable` 进摘要：两格照 G2 报告的对照复核过闸与代码字面，没找到反例（第二个 `-u` 的次序已由 G2 自己的 `--start-over` 那一格钉住）。
- J1-c 的假绿：想过两趟共用进度目录时一趟读回另一趟写了一半的片，落在 crates 的账本核对里，没攻（不归这一轮）。

上面 P3、P4b、P6 三条是拿仓里被判的两份模块直接判字面（一次性 python，按路径导入 `admission.py` 与 `lib_heavy_tests.py`，没存进模型目录），原样：

```
IGNORE_FORM #![ignore] False
IGNORE_FORM #[ignored] False
IGNORE_FORM #[ignore(x)] False
IGNORE_FORM # [ignore] True
IGNORE_FORM #[ignore = "x"] True
COMPUTED_INCLUDE include_str ! ( concat!("a","b") ) True
COMPUTED_INCLUDE include_str![concat!("a","b")] True
COMPUTED_INCLUDE include_str!{concat!("a","b")} True
COMPUTED_INCLUDE ::std::include_str!(concat!("a","b")) True
COMPUTED_INCLUDE r#include_str!(concat!("a","b")) True
COMPUTED_INCLUDE include_str!("x.tsv") False
COMPUTED_INCLUDE include_str!(r"x.tsv") False
COMPUTED_INCLUDE grab!(concat!("a","b")) False
LISTS_ONLY ['-qZ', '--list'] True
LISTS_ONLY ['--skip=--list'] False
LISTS_ONLY ['--include-ignored', '--skip=x', '--list'] True
```

`-qZ --list` 判成只列：libtest 把 `--list` 当 `-Z` 的值、报不稳定选项退出，一条都不跑（推的，没在小 crate 上跑），放行不造假绿。最后一行 `--skip=x` 写成一个词时 `--list` 是真选项，判只列对。`grab!(…)` 那一行就是 N1–N3 打中的那一种。

## 这条腿自己的限度

- 闸那一侧只喂 JSON、没喂看门狗（`research/scripts/agent-watch.py` 与闸共用 `classify` / `command_under_launcher`，推的：同样认不出 T5–T7、R9–R11、S1–S4）；P5 的 C、D 在看门狗那一层抛不抛、抛了怎么处置，没量。
- 语义只在草稿目录的小 crate 与本机 cargo 1.98.0、rustc 1.98.0、systemd 255、strace 6.8、util-linux 2.39.3 上量过；S1、S3 真起了 `systemd-run --user`（`--wait --pipe` 与 `--scope`，起的是 `printenv` / `printf`，没留单元）。
- D2 ① 的运行时集合只覆盖我造的那几条路径（单机日志、merge 日志各判绿一次，record、marker-check 作数那一支）；判红那几支（缺字段、片数不一、标记被改过）没逐条跑，那几支用到而闭包外的定义可能漏数。② 四种写法是我挑的，不是穷举。
- P9 只量了配置文件与环境变量两种写法、`sh <相对路径>` 一种形状；`build.rustc-wrapper` 这一类的参数（wrapper 的当前目录）没量。
- 标记字段 ① 只核了第 42 行一条用例读的变量；别的六条读哪个线程变量归本地攻方的 L3，没做。
- 攻方改法 F1–F10 都是我一条腿提的、只在副本上量过，没被任何一条腿攻过；F3 在 T9 上多拒，量出来了、没修。

## 没做什么

- 重型测试一条没跑：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test`、双机驱动（带不带 `--selftest` 都没跑）。准入模块只跑了 `crash-case-*` 子命令（小仓上，与真仓第 42 行那一条的 `crash-case-command`，只读），没跑它的 `--selftest`；闸只喂 JSON。
- 没攻 L2（分片判法与条款一致、第二轮的数）、L3（七条登记行逐格）、54 号与驱动里读发现日志的那几段；没读禁读清单里的文件。
- 没改仓里任何被判文件；交回前开工快照 14 个 `sha256sum -c` 现核全 OK。改法只打在草稿目录的 hooks 副本上。
- 没做 git 写操作。
- 登记给 three-way-attack 的门禁阶段：`stage-owners.tsv` 里没有，没跑。

## 草稿目录与清理

- 删了（交回之前）：`/tmp/claude-1000/defs-gatebatch-m2-r3-opus/semantics`（32M，小 crate 与它的编译目录）、`fixcopy`（236K，hooks 副本）、各探针建的小仓与镜像仓（`l1-*`、`p5-mirror-*`、`d2-trace-*`、`marker-fields-*`）。
- 留着：`/tmp/claude-1000/defs-gatebatch-m2-r3-opus/rerun-summary.txt`（最后一趟 `rerun.sh` 的汇总，已拷进模型目录 `outputs/rerun-summary.txt`）。
- 模型目录 `sha256sum -c SHA256SUMS` 现核全 OK；引文核对 `python3 research/scripts/cite-check.py research/prompts/defs-gatebatch-m2-r3-opus-output.md` 末行见交回。
