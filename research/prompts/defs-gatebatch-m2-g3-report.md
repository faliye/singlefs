# G3 门禁批第三轮改法：续做报告（tooling-writer，接手撞限额的上一任）

写于 2026-09-27 JST 14:5x。关的是 `research/prompts/defs-gatebatch-m2-r3-main-verification.md` 第二节改法表十二条（F1、F2、F3、F5、F10、F4、F6、F9、FD2、FD3、FM、R1）。

## 结论

- 十二条都做完了：草稿副本上的改动已经逐份换进仓（同目录临时文件再 `mv`，换完 `cmp` 与草稿相同）；条款文字由上一任直接在仓里用 Edit 写好，我按代码核过，没再改。
- 规格「出口」列的自证全绿：admission 270 格、lib 167 种、guard 681 种、shard 驱动 13 格、54 号样本 2 个。十六个新弄坏开关各自判红（退 1）。
- 出口那一批门禁里，红的有 47、72、doc-lint、gate-lint、shell-lint、preflight-lint 这几道，点名的文件都不是这一轮改的（逐条见下），没修。
- 54 号样本没改：FD3 只改了 54 号调 `crash-case-judge` / `crash-case-record` 的参数（不再转 `--machine-cores/--threads/--threads-origin`），没改它的判法；两个样本不走线程判定，改前改后 stage-selftest 都是 2 格判对。「转过来的线程参数翻不了判定」这一格放在 admission 自证里，由开关 `judge-takes-forwarded-threads` 证红。
- 推翻条件：换进仓的任何一份被别的会话再改，下面的 `cmp` 与自证行就不再作数；`sha256sum` 对不上下一节的值，就要重跑。

## 核现场（接手时，JST 14:3x）

- 草稿 `base/` 存着开工时的 11 份原样。仓里 8 份代码与登记表跟 `base/` 相同（接手时与换进前各 `cmp` 了一次），也就是没有别的会话动过。
- `.claude/rules/implementation-workflow.md` 与 `.claude/agents/crash-verifier.md` 和 `base/` 不同，差异是上一任在仓里直接做的 Edit（会话记录里 4 次加 3 次，逐条对得上）。另外，`crash-verifier.md` 第 5 行 `model: opus` → `model: sonnet` 不是上一任改的（它的 Edit 里没有这一处），应是别的会话改的，我没碰。
- `.claude/main-agent.md` 从 00:38 UTC 起没动过，这一轮不涉及它。
- 负载（`ps`，JST 14:33）：别的会话的 `cargo test` 在跑（second_transaction_* 几个、e163-gpu、e161），没有 qemu、vm-bench、e152、fio，所以加了 `nice -n 19` 照跑。load 53。

## 改过的文件（这一轮的差，相对 `base/`，`git diff --no-index --numstat`）

```
246+ 23- .claude/hooks/lib_heavy_tests.py
36+ 1- .claude/hooks/heavy-test-guard.sh
622+ 106- research/scripts/admission.py
9+ 7- .claude/gate.d/54-layer0-replay.sh
1+ 1- .claude/gate.d/stage-inputs.tsv
26+ 13- research/scripts/layer0-shard-run.sh
5+ 1- research/scripts/layer0-shard-configuration-check.sh
30+ 2- research/scripts/layer0-shard-run-selftest.sh
2+ 1- .claude/rules/implementation-workflow.md
4+ 3- .claude/agents/crash-verifier.md
```

`crash-verifier.md` 那 4+ 3- 里有 1+ 1- 是别的会话改的 model 行。`stage-inputs.tsv` 只改了第 42 行（crash-injection-fast-tier，加 `threads=CRASH_INJECTION_FINISHED threads-variable=SINGLEFS_CRASH_INJECTION_THREADS`），第 34 行（E142）没碰。

`git diff --stat` 相对 HEAD 的原样如下。里面含 HEAD 之后别的会话还没提交的改动，比如 54 号的 166 行大多不是这一轮改的：

```
 .claude/agents/crash-verifier.md                   |   7 +-
 .claude/gate.d/54-layer0-replay.sh                 | 166 ++++-
 .claude/gate.d/stage-inputs.tsv                    |   4 +-
 .claude/hooks/heavy-test-guard.sh                  |  37 +-
 .claude/hooks/lib_heavy_tests.py                   | 269 +++++++-
 .claude/rules/implementation-workflow.md           |   3 +-
 research/scripts/admission.py                      | 728 ++++++++++++++++++---
 .../scripts/layer0-shard-configuration-check.sh    |   6 +-
 research/scripts/layer0-shard-run-selftest.sh      |  64 +-
 research/scripts/layer0-shard-run.sh               |  95 ++-
 10 files changed, 1203 insertions(+), 176 deletions(-)
```

换进仓那一步的原样：

```
换进 cmp 相同 .claude/hooks/lib_heavy_tests.py
换进 cmp 相同 .claude/hooks/heavy-test-guard.sh
换进 cmp 相同 research/scripts/admission.py
换进 cmp 相同 .claude/gate.d/54-layer0-replay.sh
换进 cmp 相同 .claude/gate.d/stage-inputs.tsv
换进 cmp 相同 research/scripts/layer0-shard-run.sh
换进 cmp 相同 research/scripts/layer0-shard-configuration-check.sh
换进 cmp 相同 research/scripts/layer0-shard-run-selftest.sh
```

没有新建门禁阶段或钩子，所以没有新写 `# gate-similar:` / `# hook-events:` 行。gate-overlap 判的是「改过的 6 份脚本对照已有的 134 份没有整段相同」。

## 每条改法：改前红、改后绿、弄坏开关判红

改前 = 攻方探针（`research/prompts/defs-gatebatch-m2-r3-opus-model/`，另有 `/tmp/claude-1000/gate-batch-m2-g3/myprobes/` 下三份改写）判仓里改前那一份（草稿 `probes-before/`）；改后 = 同一探针判草稿副本（`probes-after/`）；开关 = 仓里换进之前，草稿副本上 `rerun/` 那一批重跑（JST 14:3x–14:4x）。

| 改法 | 改前红的格 | 弄坏开关（自证） |
|---|---|---|
| F1 | l1-hook S3、S4 | LIB_HEAVY_TESTS_BREAK=launcher-long-options-partial |
| F2 | l1-hook S1、S2 | launcher-drops-property-environment |
| F3 | l1-hook T5–T7 | list-past-separator（lib 与 guard 两份自证都红） |
| F5 | l1-p5 C、D | judging-error-propagates（lib 与 guard 两份自证都红） |
| F10 | l1-hook R9–R11 | configuration-before-subcommand-only |
| F4 | l1-attrs I1、I2 | ADMISSION_BREAK=target-own-files-only |
| F6 | l1-manifest N1–N3 | include-alias-subtracts |
| F9 | l1-manifest P9a；myprobes p9b_g3 | runner-arguments-from-configuration-directory-only |
| FD2 | d2-g3 ② 四种写法 | digest-skips-other-statements、digest-allows-non-name-dispatch、digest-allows-imported-names |
| FD3 | d2-g3 ③ | judge-takes-forwarded-threads |
| FM | marker-g3 ①②③ | cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line |
| R1 | 改前驱动不经包装起第二台那一片、配置不要内存键（开关复现改前行为） | LAYER0_SHARD_RUN_BREAK=peer-without-memory-cap |

### 探针 SUMMARY 行原样（改前 → 改后）

```
before l1-hook: SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=10 control_cells_broken=0
after  l1-hook: SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=0 control_cells_broken=1
before l1-p5: SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=2 control_cells_broken=0
after  l1-p5: SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=0 control_cells_broken=0
before l1-attrs: SUMMARY L1 P4：include! 进来的同名定义: cells=10 attack_cells_broken=4 control_cells_broken=0
after  l1-attrs: SUMMARY L1 P4：include! 进来的同名定义: cells=10 attack_cells_broken=0 control_cells_broken=2
before l1-manifest: SUMMARY L1 P6 P7 P9：清单与指纹: cells=10 attack_cells_broken=5 control_cells_broken=0
after  l1-manifest: SUMMARY L1 P6 P7 P9：清单与指纹: cells=10 attack_cells_broken=1 control_cells_broken=0
before d2-g3: SUMMARY D2 判法摘要的闭包（G3 改写）: cells=5 attack_cells_broken=5 control_cells_broken=0
after  d2-g3: SUMMARY D2 判法摘要的闭包（G3 改写）: cells=6 attack_cells_broken=0 control_cells_broken=0
before marker-g3: SUMMARY 标记字段与线程来源（G3 改写）: cells=3 attack_cells_broken=3 control_cells_broken=0
after  marker-g3: SUMMARY 标记字段与线程来源（G3 改写）: cells=3 attack_cells_broken=0 control_cells_broken=0
before p9b-g3: SUMMARY P9b 改写：环境变量 runner 的参数从包目录解: cells=1 attack_cells_broken=1 control_cells_broken=0
after  p9b-g3: SUMMARY P9b 改写：环境变量 runner 的参数从包目录解: cells=1 attack_cells_broken=0 control_cells_broken=0
```

改后还剩的 BROKEN 格原样（逐条判过，都不是没修好）：

```
CONTROL	BROKEN	T9 -- --ignored -- --list（只剩过滤词 --list，一条都不跑：放行是对的） ⇒ 应当放行	退 2；「bash <仓>/research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- --ignored -- --list」；✗ 重型测试被拒：cargo test --test record_checker_judges_absence_by_the_persisted_set 带 --ignored，跑到登记的崩溃枚举用例（record_checker_jud
ATTACK	BROKEN	P9b 环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="sh ../../tools/r.sh" ⇒ 改 tools/r.sh 指纹应当变	改前 b2988232c88cd188… 改后 b2988232c88cd188…（文件数 / 减去数 改前 4/3 改后 4/3）
CONTROL	BROKEN	I3 只有 include! 进来的一份（测试目标里没有字面的 fn the_case(）：判不出，按没标算（P5）：闸（cargo test --release，不带 --ignored） ⇒ 应当拒	闸退 0
CONTROL	BROKEN	I3 只有 include! 进来的一份（测试目标里没有字面的 fn the_case(）：判不出，按没标算（P5）：crash-cases 自查 ⇒ 应当退 2	自查退 0：crash-case:own	pkg	own_case	the_case
```

- T9：F3 定下的代价，`-- --ignored -- --list` 多拒了，判决第四节第 2 条与用户 JST 14:0x 已接受。
- I3：include! 进来的那一份标了 `#[ignore]`；F4 顺着 include! 读到了它，放行是对的。攻方写的「判不出，按没标算」是改前的预期。
- P9b（l1-manifest 那一格）：攻方探针先设 runner 环境变量再 pop 掉，这个变量其实没带进准入模块。改写成只设不 drop 的 `myprobes/probe_p9b_g3.py` 改前 BROKEN、改后 holds（上面 p9b-g3 那两行）。

### 弄坏开关 → 判红（草稿副本，每个开关贴第一条 ✗ 原样与退出码）

```
[lib-launcher-long-options-partial exit=1]
  ✗ lib_heavy_tests 自检：systemd-run --user --scope --expand-environment no（值另起一个词）包一层 应当是 full-cargo，实际 None
[lib-launcher-drops-property-environment exit=1]
  ✗ lib_heavy_tests 自检：systemd-run -p Environment= 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
[lib-list-past-separator exit=1]
  ✗ lib_heavy_tests 自检：点名崩溃枚举用例、-- --include-ignored the_full_case -- --list（第二个 -- 之后 --list 是过滤词） 应当是 crash-case-cargo，实际 None
[guard-list-past-separator exit=1]
  ✗ lib_heavy_tests 自检：点名崩溃枚举用例、-- --include-ignored the_full_case -- --list（第二个 -- 之后 --list 是过滤词） 应当是 crash-case-cargo，实际 None
[lib-judging-error-propagates exit=1]
  ✗ lib_heavy_tests 自检：admission.py 导入得了、判的那一刻抛 NameError：点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 按判不出算 应当是 crash-case-cargo，实际 抛了 NameError("name 'definitions_marked_ignored' is not defined")（闸的入口接住之后放行）
[guard-judging-error-propagates exit=1]
  ✗ lib_heavy_tests 自检：admission.py 导入得了、判的那一刻抛 NameError：点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 按判不出算 应当是 crash-case-cargo，实际 抛了 NameError("name 'definitions_marked_ignored' is not defined")（闸的入口接住之后放行）
[lib-configuration-before-subcommand-only exit=1]
  ✗ lib_heavy_tests 自检：--config 写在 test 之后定 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
[adm-target-own-files-only exit=1]
  ✗ crash-cases 自查（带进来的源文件）：debug 下标 ignore、release 下 include!("common/release_case.rs") 进来一份不标的同名用例 ⇒ 退 2 并说没标 #[ignore]：退 0，stdout「crash-case:own	pkg	own_case	the_case」
[adm-include-alias-subtracts exit=1]
  ✗ 崩溃枚举用例的输入：用例里 use core::include_str as grab; 再 grab!(concat!(…))（改名之后的调用认不出），改 other_target.rs 指纹变：改之前 45c94b5799c96b08，改之后 45c94b5799c96b08
[adm-runner-arguments-from-configuration-directory-only exit=1]
  ✗ 构建环境：仓根 .cargo/config.toml 里 runner = "sh ../../tools/r.sh"（从包目录起才解得到），改 tools/r.sh 指纹变：改之前 86e3561cda32f894，改之后 86e3561cda32f894
[adm-digest-skips-other-statements exit=1]
  ✗ 判法摘要换写法：元组解包赋值：PASSED_ONE_TEST_FORM 挪进 `A, B = …`，再把判绿的正则放宽成「test result: 」开头都算 ⇒ 第二步摘要变：第一步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'，第二步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'
[adm-digest-allows-non-name-dispatch exit=1]
  ✗ 判法摘要换写法：分派表的值不是名字：crash-case-judge 那一项写成 lambda，再改 command_crash_case_judge 让它一律判绿 ⇒ 两步都拒算：第一步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'，第二步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'
[adm-digest-allows-imported-names exit=1]
  ✗ 判法摘要换写法：判法挪进 import 的别的模块：`from judging_helpers import fields_of_line`（再改那份模块，准入模块原文一个字不变） ⇒ 两步都拒算：第一步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'，第二步 b'## BREAK_VARIABLE\nBREAK_VARIABLE = "ADMISSION_BREAK"\n## COUNT_LINE_PREFIX_FORM\nC'
[adm-judge-takes-forwarded-threads exit=1]
  ✗ crash-case-judge 自己现取线程数：没设 SINGLEFS_LAYER0_THREADS、转过来 --threads 1 --threads-origin explicit（54 号改前的转法） ⇒ 退 2：退 0
[adm-cores-from-nproc exit=1]
  ✗ 本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 ⇒ 仍是 os.cpu_count() 与 CPU 亲和的小者（32），线程数同它、default：交的是 (1, 1, 'default')
[adm-thread-variable-ignored exit=1]
  ✗ crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1 ⇒ 起的用例看到配的 32（盖掉调用方的）：用例看到「1 32」
[adm-threads-skip-self-contained-finish-line exit=1]
  ✗ 真仓登记的 crash-case:crash-injection-fast-tier：本机 32 核、线程数没显式设，跑完那一行报 1 个线程跑了 24 片 ⇒ 红：判成绿：[]
[shard-peer-without-memory-cap exit=1]
  ✗ ⑧ 第二台那一片经 research/scripts/run-with-memory-cap.sh 起（上限是配置的 PEER_MEMORY_CAP=1G）：驱动的输出里 ④ 那一行没写第二台那一片经 run-with-memory-cap.sh 1G 起；假 cargo 在第二台那一片记下的 cgroup 不在 singlefs-memory-cap- 的 scope 里：0::/user.slice/user-1000.slice/session-1.scope 
```

同一批里不带开关的四份（草稿副本）：

```
[adm-plain exit=0]
  ✓ admission.py 自证通过：270 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch、target-own-files-only、include-alias-subtracts、runner-arguments-from-configuration-directory-only、digest-skips-other-statements、digest-allows-non-name-dispatch、digest-allows-imported-names、judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）
[lib-plain exit=0]
  ✓ lib_heavy_tests 自检通过（查了 167 种：cargo 与包装过的命令行 65 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 46 种、直接执行的测试二进制 21 种、按名字判的脚本 6 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种、导入得了而判的时候抛异常 2 种）
[guard-plain exit=0]
  ✓ 自检通过（查了 681 种，其中该拒 129 种）：子 agent 跑层 0、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表拒绝，崩溃验证员与门禁分诊带前缀跑各自那一份放行、越出那一份或不带前缀拒绝，主 agent 带 commit / user-request 放行、不带或带别的值拒绝；写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；同一条命令里 heredoc（cat / tee）写出或 cp 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法照旧记检出；直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；子 agent 不经 run-with-memory-cap.sh 跑 cargo test / run / bench 与 cargo 编出来的二进制拒绝、经它包着的（连同 bash -c 与它起的脚本）放行；只跑动到的测试目标、fmt / clippy / build、54 / 55 / 57 / 59 / 87 之外的门禁阶段、把名字当参数的放行
[shard-plain exit=0]
  ✓ layer0-shard-run.sh 自证通过：13 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）
```

## 出口：换进仓之后的自证与门禁（仓里跑，JST 14:4x，`nice -n 19`，每件退出码进自己的文件，16 件对上 16 个文件）

命令见 `research/prompts/gate-batch-m2-g3-tmp-evidence/final.sh`：54 号样本用 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh <只放 54 号、它的样本与 stage-inputs.tsv 的临时 gate.d>` 跑，没直接起 54 号。登记给 tooling-writer 的阶段是 47、62、63、72、73。

```
[admission exit=0]
  ✓ admission.py 自证通过：270 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch、target-own-files-only、include-alias-subtracts、runner-arguments-from-configuration-directory-only、digest-skips-other-statements、digest-allows-non-name-dispatch、digest-allows-imported-names、judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）
[lib exit=0]
  ✓ lib_heavy_tests 自检通过（查了 167 种：cargo 与包装过的命令行 65 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 46 种、直接执行的测试二进制 21 种、按名字判的脚本 6 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种、导入得了而判的时候抛异常 2 种）
[guard exit=0]
  ✓ 自检通过（查了 681 种，其中该拒 129 种）：子 agent 跑层 0、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表拒绝，崩溃验证员与门禁分诊带前缀跑各自那一份放行、越出那一份或不带前缀拒绝，主 agent 带 commit / user-request 放行、不带或带别的值拒绝；写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；同一条命令里 heredoc（cat / tee）写出或 cp 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法照旧记检出；直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；子 agent 不经 run-with-memory-cap.sh 跑 cargo test / run / bench 与 cargo 编出来的二进制拒绝、经它包着的（连同 bash -c 与它起的脚本）放行；只跑动到的测试目标、fmt / clippy / build、54 / 55 / 57 / 59 / 87 之外的门禁阶段、把名字当参数的放行
[shard exit=0]
  ✓ layer0-shard-run.sh 自证通过：13 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）
[stage54 exit=0]
  ✓ 有样本的阶段判得都对（2 个样本），0 个阶段仍未自检
[g47 exit=1]
             ✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 2 处：
[g62 exit=0]
  ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）
[g63 exit=0]
  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（4 个有 Write 或 Edit 的定义、28 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 13 个文件），共用重型测试判定模块的 36 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 13 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT）
[g72 exit=1]
  ✗ 改过的定义与共用约束里 2 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）：
[g73 exit=0]
  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录（research/scripts .claude/hooks .claude/scripts），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 4 个目录；执行位：判过（2 个目录）
[doclint exit=1]
  ✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、2 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 522，跳过 0）
[ruleslint exit=0]
  ✓ 规则只写怎么做（语言 zh；扫了 29 份文件 1967 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在引号或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1844 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=）
[gatelint exit=1]
  ✗ 门禁自检失败：45 处（共 667 个脚本、1093 条拒绝）
[shelllint exit=1]
[preflightlint exit=1]
  ✗ 准入与运行条件：判了 141 个脚本（登记目录 53、脚本 3、钩子 9、门禁阶段 76），1 处不合格；排除 192 个：.claude/gate.d/lib-format-const.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-history-brief.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-index-vs-body.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-item-ref-status.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-manifest.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-open-item-review.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-owed.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-prime-marks.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/hooks/lib_heavy_tests.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；.claude/hooks/lib_selftest_scratch.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；.claude/hooks/lib_shell_words.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；research/scripts/changed-paths.sh（被门禁阶段 source 的函数库（gate_diff_base、gate_changed_paths），不单独调）；research/scripts/stage-run-or-skip.sh（被门禁阶段 source 的函数库（stage_run_or_skip）；它的 --selftest 由 47 号跑）；research/scripts/lib_atomic_replace.py（被 replace-once.py、insert-row.py import 的函数库，不单独调）；research/scripts/e125-zoned-wp-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-tear-injector.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-thin-neighbour.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-thin-rmw.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-run.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-stage-root.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-tables.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-compute.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-prove-gpu.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-transfer.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e34-iomin-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e56-report.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e56-sweep.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e6-units.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e72-devtable-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/local-repro-tuned.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/local-repro.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/stripe-map-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/vm-bench.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/vm-geom.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e100_system_configuration_slot.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e101_node_tag_reserve.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e102_unit_class_registry.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e103_inode_update_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e104_current_version.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e105_extent_leaf_packed.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e106_stripe_member_table.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e107_stripe_table_wa.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e108_plaintext_layer_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e109_position_authority.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e110_stripe_table_steady.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e111_stripe_table_key.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e112_old_writer_unknown_tree.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e113_unknown_tree_full_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e114_pack_ledger.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e116_pack_settle.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e117_reserved_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e118_single_disk_recovery.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e119_slot_tiers.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e120_tier_ratio.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e121_capacity_tiers.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e122_directory_locality.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e123_reuse_window_versus_rollback_depth.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e127_group_identity_under_split_merge.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e128_pointer_birth_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e12_lifecycle.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e131_livelist_carrier.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e132_livelist_carrier_recount.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e133_map_key_format_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e134_map_key_slot_baselines.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e135_rollback_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e136_fork_cost_rows.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e137_map_key_performance.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e138_per_disk_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e139_tightened_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e140_header_alignment.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e141_switch_reserve_mount_admission.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e142_region_diff_independent.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e144_header_checksum_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e145_self_describing_node_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e146_livelist_entry_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e149_pack_container_repair_options.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e14_discrimination.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e150_rollback_reuse_of_abandoned_roots.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e151_arrival_and_container_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e152_file_system_benchmark.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e153_ledger_shape_and_ring_holes.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e154_two_gates_serial_rejudge_and_reclaim_timing.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_fsync_write_volume.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_third_run_release_cascade.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e160_random_small_read_share.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e16_journal.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e17_merge.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e18_branch.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e19_defer.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e20_fanout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e21_cpu.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e23_journal_geom.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e24_recovery.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e25_journal_reserve.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e26_accounting.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e27_snapshot_accounting_risk_paths.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e28_map_rebuild.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e29_blast_radius.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e30_range_rebuild.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e31_aad_snapshot.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e32_journal_timeline.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e33_pin_rules.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e34_ring_iomin.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e35_head_forms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e36_slot_mapping.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e37_log_epoch.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e38_accounting_copy_on_write.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e39_back_chain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e40_checksum_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e41_root_ring_geom.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e42_transaction_records.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e43_extension_point_budget.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e44_jsn_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e45_span_ring_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e46_region_spacing.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e47_ring_loss.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e48_ring_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e49_chain_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e50_ring_slots.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e51_chain_chances.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e52_head_mechanisms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e53_ring_failure.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e54_accounting_generations.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e56_epsilon.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e57_field_authority.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e58_csum_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e59_message_recompute.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e60_rebalance.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e61_chain_hash.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e62_ring_home.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e63_width_rule.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e65_write_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e66_small_files.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e67_device_subset.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e68_inline_threshold.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e69_backref_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e6_multicore.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e6_units.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e70_ckpt_thresholds.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e71_accounting_keys.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e73_key_range.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e74_allocation_records.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e75_record_size.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e76_payload_checksum.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e77_publish_order.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e78_replay_start.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e79_root_record.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e7_index.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e80_partial_stripe.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e81_commit_fixpoint.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e82_admission_overlay.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e83_tombstone_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e84_tombstone_pinning.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e85_unit_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e86_scan_step.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e87_fixed_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e88_impostor_orphan.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e89_interval_frontier.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e8_split.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e90_tree_aad.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e91_ring_admission.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e92_reuse_requirement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e93_aging_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e94_move_touchset.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e95_node_layout_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e96_hybrid_consistency.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e97_entry_encoding.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e98_inode_record.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e99_writebuffer_sequence.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e9_keylayout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e158_root_choice_repair.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs（还没改完：它在 crates/ 里，加 preflight 调用是 crates 代码改动，要带测试并走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/first_transaction_on_device.rs（在虚机的 busybox initramfs 里跑的静态二进制，那里没有 python3，preflight 判不了条件）；crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs（还没改完：它在 crates/ 里，加 preflight 调用是 crates 代码改动，要带测试并走代码三方（门禁 56 号））；.claude/scripts/preflight.sh（被项目脚本 source 的函数库：找到规范副本里的 preflight.sh（本树或主仓）就 source 它，找不到就定义一个当场判红的 preflight；不单独调）；.claude/scripts/project_preflight.py（被项目的 python 脚本 import 的函数库：找到规范副本里的 preflight.py（本树或主仓）再转调，找不到判红；不单独调）；.claude/gate.d/lib-governance-refs.py（被门禁 10 号调的函数库（治理文档里的门禁号、路径与「小节」指不指得到），不单独调）
[overlap exit=0]
  ✓ 相对 262c02d3e47a：新加的门禁与钩子 0 个（无）都写明了比过谁，改过的 6 份脚本对照已有的 134 份没有整段相同
```

红的几道逐条看点名，都不在这一轮的改动里，照写，没修：

```
  ✗ python3 research/scripts/check-segment-registry.py --selftest 没过（退出码 1）：
      ✗ --selftest 没通过：
             ✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 2 处：
  ✗ 改过的定义与共用约束里 2 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）：
      .claude/agents/experiment-runner.md
      .claude/agents/implementation-writer.md
     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，
  ✗ 编号 T1 已有登记位，却被 not-numbers 声明成术语
  ✗ .claude/kb/experiments/158-择根与修复四岔路.md  8 处编号引用没带简称或简称不符
  ✗ research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs:726 fn main 的第一句不是 preflight(…)
```

- 47 号：红的是 `research/scripts/check-segment-registry.py --selftest`（段序列登记表与 E142 产物），不是这一轮改的。本轮改的四份自证（admission、shard 驱动与它的 selftest、hook 两份）在上面单跑，都是绿的。
- 72 号：点名 `.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`，这两份是别的会话改的。`crash-verifier.md` 不在红表里，因为判决 `defs-gatebatch-m2-r3-main-verification.md` 第五节按路径点了它。用户定的逐份豁免登记由主 agent 做，我没写 `.claude/agent-def-review-exempt`。
- doc-lint：点名 `.claude/kb/experiments/158-择根与修复四岔路.md` 的编号引用。
- preflight-lint：点名 `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs:726`。
- gate-lint（45 处）、shell-lint（61 处）：用 `grep -E 'lib_heavy_tests|heavy-test-guard|admission.py|54-layer0|stage-inputs|layer0-shard'` 在两份输出里各数出 0 行，这一轮的文件一处都没被点名。73 号（研究脚本、hook 与 .claude/scripts 的门禁纪律）是绿的。

上面 shell-lint 那格是空的：它的日志里有非文本字节，`grep` 没按文本读。原样末行补在这里：

```
  ✗ shell 纪律检查失败：61 处（共检查 532 个脚本）
  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。
  ✗ 门禁自检失败：45 处（共 667 个脚本、1093 条拒绝）
```

更正上面 gate-lint / shell-lint 那一条：用 `grep -a` 重数，gate-lint 0 行，shell-lint 7 行。这 7 行全是 `grep: …/.claude/gate.d/fixtures/54-layer0-replay.sh: Is a directory`（6 行）与一行 `awk: warning: … is a directory: skipped`，是 lint 把样本目录当成脚本读时出的噪声，不是违规条目，也没点名这一轮改的文件。

## 54 号样本

- 没改 `.claude/gate.d/fixtures/54-layer0-replay.sh/`。FD3 在 54 号只动了两处：`crash-case-judge` / `crash-case-record` 两次调用里去掉 `--machine-cores/--threads/--threads-origin`；文件头「54 号不进指纹」那一段与 `case_threads` 那句注释跟着改了（改前改后的 diff 是 9+ 7-）。判法（读发现日志、判红的那几段）一字没动。
- 两个样本（red：two_signatures、unfinished、many_signatures；green：clean_stream、no_enumeration）的登记行都没写 `threads=`，不走线程判定，所以 FD3 前后判的结果一样。上一任在只放 54 号的临时 gate.d 上跑过：改前与改后都是 2 格判对（`stage54-base.log`、`stage54-after.log`）。换进仓之后再跑一遍，还是 2 格判对（上面 stage54 那一格）。
- 「54 号改了转法也翻不了线程判定」这件事，样本造不出来：要翻，就得改 54 号本身。所以这一格放在 `admission.py --selftest`：「crash-case-judge 自己现取线程数」那几格，由开关 judge-takes-forwarded-threads 证红。
- 规格要求的那一句（「54 号不进指纹：它里面还定着结论的只剩流程的次序」）改成了现在的写法：点名 54 号里还定着结论的是流程次序与两处第二道判红，它们的第一道在判法摘要里。判法摘要是 FD2 改过的那一份。

## 条款文字（第二种活）

改过的定义与规则（上一任在仓里用 Edit 写的，我对照换进仓的代码核过，没再改）：
- `.claude/rules/implementation-workflow.md`：「测试与崩溃检测优先多线程」一节加「双机分片」一条（第 88 行）；「门禁管哪一半」一段加上双机分片的逐片线程判定与驱动的几项判定（第 90 行）。其中点名的 `judge_threads_of_each_shard`、`PEER_MEMORY_CAP` 在换进仓的 `admission.py` 与两份分片脚本里都有。
- `.claude/agents/crash-verifier.md`：第 1 步加上「有不带 --selftest 的 layer0-shard-run.sh 在跑时不起 54 号」；加第 1c 步（跑配置检查 `--emit-assignments` 与登记表 awk、抄 `PEER_SSH_HOST=` / `PEER_MEMORY_CAP=`、驱动只由 54 号 `--full` 调）；第 4 步加上双机那几条要抄的驱动行，以及第二台退 250–254 怎么办。R1 要求的 J1-c「点名单独跑的驱动」那一句，落在第 1 步与第 1c 步。
- rules-lint（项目本地那一道）绿，doc-lint 没点名这两份。定义三方（72 号）没走：用户已定逐份豁免，登记由主 agent 做。

## 交给主 agent 的

- 仓根 `layer0-shard.env.example` 要加内存键：成品在 `/tmp/claude-1000/gate-batch-m2-g3/deliver/layer0-shard.env.example`，请主 agent 换进去（与仓里那一份的差是「七个键」改成「八个键」，加两行说明与 `PEER_MEMORY_CAP=24G`）。
- 本机的 `layer0-shard.env`（git 忽略）现在有 5 个键。换进仓之前它就缺 `QUIESCE_STOPPED_CHECK_COMMAND`、`QUIESCE_STARTED_CHECK_COMMAND`，配置检查已经在拒（接手时现跑的原样：「✗ 双机分片不能用：/home/fy5090/code/singlefs/layer0-shard.env 缺键 QUIESCE_STOPPED_CHECK_COMMAND QUIESCE_STARTED_CHECK_COMMAND（不清场的两对写成空串，键照样要写）」）。现在又多缺一个 `PEER_MEMORY_CAP`。要走双机分片，得由用户补这三个键；不补，54 号 `--full` 就照单机跑。
- `crash-verifier.md` 的逐份豁免登记（`.claude/agent-def-review-exempt`）由主 agent 做。另外，这份文件第 5 行 `model: opus` → `sonnet` 是别的会话改的，不在这一轮里。
- 判决第三节第 3 条：FD2、FD3、FM 改了准入模块的判法，崩溃枚举用例的指纹因此都变了，下一趟 `--full`（提交时）要全跑。

## 草稿目录里留下的

- 留着：`/tmp/claude-1000/gate-batch-m2-g3/base/`，开工时 11 份被判文件的原样（660K）。主 agent 逐条核「改前红」时，拿攻方探针指到这一份就能复现。它是逐份原样，不是仓副本。另外 `deliver/`、`myprobes/`、各批日志（`probes-before/`、`probes-after/`、`rerun/`、`final/`、`breaks-adm/`、`baseline-runs/`、`shard-runs/`）也留着。
- 删了：`copy/`（仓副本，已和仓里 cmp 相同）、`shardrun/`（287M 的 rsync 仓副本）、`probe-scratch/`、`stage54-base/`、`stage54-after/`、`stage54-final/`。shard 驱动自证在 `/tmp/tmp.*` 下建的临时目录由它自己删了（日志末行「自证的临时目录 … 删掉」）。
- 上一任留下的三个后台任务（bjabgf8g7、bejo1m08a、bdmq5dh11）：`ps` 里已经没有 admission、layer0-shard、lib_heavy、heavy-test-guard 的进程。

## 没做什么

- 没跑重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test`、`check.sh`、不带 `--selftest` 的双机驱动。54 号只经 stage-selftest 跑了样本。
- 没走定义三方（用户已定逐份豁免），没写 `.claude/agent-def-review-exempt`，没提交。
- 没修出口里红的 47、72、doc-lint、gate-lint、shell-lint、preflight-lint：点名的文件都不在这一轮的改动里。
- 72 号那一道（规格出口列了）已跑，结果见上。
- R1 里「输入指纹两台不同」与「第二台退 250–254」两支没有自证格，驱动的这两支没造红输入。条款里照实写了「没有自证格」。
- 没改 `layer0-shard.env.example`（在仓根，不在写范围里；成品见 deliver/）与 `layer0-shard.env`。
