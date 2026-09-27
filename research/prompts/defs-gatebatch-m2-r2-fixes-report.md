# 门禁批第二轮改法（G1）交回：P1–P7、P9、P10、D2、标记字段，加 singlefs-39 的 54 号替换

写于 2026-09-26 UTC（交回时刻见文末）。规格 `/tmp/claude-1000/gate-batch-m2-r2-fixes/spec.md`，判决 `research/prompts/defs-gatebatch-m2-r2-main-verification.md` 第二、三节。

## 结论

- 判决第二节 P1–P7、P9、P10、D2、标记字段全部落进仓里五份文件；singlefs-39 的 54 号替换先套上（命中 1 次、新 inode，与 `54-after.sh` 逐字节相同）。P8 没做（照规格）。G3 写进 `admission.py` 文件头「管不到的」一段。
- 改前在攻方两个探针上各格原样判错，改后同一探针在仓的临时拷贝上复跑，剩下的是否只有 L12、R5、M3、G3 见「rerun.sh 复跑」一节。
- 三份自证在仓里全过：`admission.py` 207 格（原 167 格）、`lib_heavy_tests.py` 132 种（原 109 种）、`heavy-test-guard.sh` 658 种（原 628 种）；新加的每一格都有弄坏开关，开关打开时对应那几格转红（原样行见「弄坏开关证红」一节）。
- 47 号退 1、doc-lint 退 1，红的都不在这一批的文件里（check-segment-registry.py 的 E142 段序列、kb 里的编号引用），见「门禁与 lint」一节；62、63、gate-lint、shell-lint 全绿。
- 推翻条件：真仓里出现这一批改法之后照样放行的写法（例：`strace -E CARGO_TARGET_…_RUNNER=…` 这一类，见「看到但没做」），或 54 号 `--full` 真跑一趟时 `crash-case-command` 交的命令与用例实际要的不一样（这一批只在假 cargo 上证过）。

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

## 与实分一交来的四份 diff 相邻的地方（`/tmp/claude-1000/impl-shard-1/deliver/`，这一次没套）

重打时按新代码手工对，下面几处上下文已经变了，原 hunk 不会直接套上：
- `54-layer0-replay.sh`：文件头「断点续跑」那三行（它在后面加双机分片四行）我改写了；`run_crash_case` 整个改写成 `run_crash_case <日志> <命令…>`，前面多了 `read_crash_case_command`（它要在 `run_crash_case` 后面加 `run_crash_case_in_two_shards`）；循环里「开跑」那一句与 `if ! run_crash_case …` 那一段我改了（它把这一段换成分片 / 单机两支）。它在「--full 开跑」那一句之后加的分片判定那几行，上下文我没动。
- `admission.py`：文件头第三列说明（test= 那两行我改成四行，它改「认四种 → 五种」与 threads= 那一行）、子命令表（crash-case-manifest / crash-case-record 两条我改了，它在 crash-case-marker-path 之后加 crash-case-shardable）、弄坏开关那一段（同一行我续了 8 个开关，它续 threads-ignore-shards）、「崩溃枚举用例」一节开头的注释（我改了 ① 与 ② ③）、自证成功那一句（同一行两边都续）、`FAKE_CARGO_FOR_STAGE` 的 printf 行（我加了第 9 列线程数）、分派表 COMMANDS 与 main 的用法句（两边都加一项）。`parse_crash_case`、`judge_worker_threads` 两边都没冲突：它改的那几处我没动，但我的判法摘要自证有一格改 `parse_crash_case` 里 `if prefix not in count_lines]` 那一串，它要保证那一串在函数里仍恰好一处。
- `lib_heavy_tests.py`：它加的 KNOWN_SCRIPT_LOCATIONS 一项、classify 里 layer0-shard-run.sh 那一支、自证里三格与一个按名字判的格，所在的上下文我没改，按行号偏移应能套上（没试）。
- `stage-inputs.tsv`：我没改。

## 看到但没做（交主 agent 定）

- `strace -E VAR=VAL` / `--env=VAR=VAL` 同样给里面那条命令设环境变量（`strace -h` 现跑第 15 行 `  -E VAR=VAL, --env=VAR=VAL`），与 R2、R3 同一类；判决的 P2 只点名 systemd-run，我只做了 systemd-run（`LAUNCHER_ENVIRONMENT_OPTIONS` 一张表，加一行就罩上，没加）。闸对 `strace -E CARGO_TARGET_…_RUNNER=… cargo test -p … --test <登记目标>` 现在放行（推的，没喂闸）。
- `.claude/gate.d/stage-inputs.tsv` 第 16 行 `#     「崩溃枚举用例」一节），再加判它的 54 号、准入模块 admission.py、工具链、构建环境与这一行本身。` 与第 26 行注释里「（它也进每条用例的指纹）」，D2 之后是过时的话（进的是判法摘要，54 号不进）；规格不许我改这份，留给补 c561 那一行字段的那一次一并改。
- 实分一重打时：`crash-case-shardable` 定「这条用例怎么跑」，要加进 `CRASH_CASE_JUDGING_SUBCOMMANDS`；双机驱动 `research/scripts/layer0-shard-run.sh` 自己起 cargo、不经 `crash-case-command`，D2 之下它不在任何一条用例的指纹里（与 54 号剩下的那一份同形），它改了旧标记照样作数。
- `crash-case-command` 起用例时没清调用方的 `SINGLEFS_LAYER0_SHARD`（攻方 K3 记过，假红方向）；没做，不在这一批改法里。
- `.claude/rules/implementation-workflow.md` 第 89 行「这一趟跑了至少两片…判红」那一句对 c561 不成立（K3 c561）；判决只要标记字段的字面，规则那一句没动。

## 推翻条件

- P1–P3、P5、P10：真仓里有这一批自证与探针之外、闸照样放行的写法（例：`strace -E` 那一类），或看门狗在进程那一层认不出这几种（看门狗与闸共用 `command_under_launcher` / `classify`，没喂看门狗，推的）。
- P4、P4b：闸与自查之外另有一道按编译结果判 ignore 的检查，或 rustc 不认 `# [ignore]`（攻方 `outputs/semantics.log` 量过它认）。
- P6、P7、P9：include 的参数以字面量开头却读了别的文件（改不了的：`include_str!("…")` 字面量读的就是那一份，已由「按整词点名」那一条罩着）；runner 参数里的相对路径其实按测试二进制的工作目录（包目录）解，而不是 `.cargo` 那一层（我按攻方模型取的后者，推的，没量 cargo 怎么交参数给 runner 进程的当前目录）。
- D2：闭包之外的某个定义其实决定日志判不判绿（`getattr` / 字符串拼函数名这一类，文件头已声明），或 54 号 `--full` 真跑时 `crash-case-command` 交的命令与原来 54 号自己拼的那一条不同（逐词对过：`env -u SINGLEFS_LAYER0_START_OVER`（或 `=1`）、进度目录、输入指纹，再加 `SINGLEFS_LAYER0_THREADS=<n>`，然后 `cargo test --release -p … --test … -- --include-ignored --exact … --nocapture`；多出来的只有显式传线程数这一项，原来是 54 号 export 进环境）。
- 标记字段：c561 那种没登记 threads= 的用例，`started_worker_threads=` 记「读不到」；若有人按旧格读 `worker_threads=`（我在仓里 `.claude`、`research/scripts`、`records` 下 grep 过，只有 54 号与 admission.py 用它），会读不到。

## 没做什么

- 重型测试一条都没跑：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的目标、全量 `cargo test` 都没起；54 号只在 `admission.py --selftest` 的假 cargo 场景里跑。真 `--full` 第一次跑 `crash-case-command` 交的命令，是提交时崩溃验证员那一趟（判决 K2：这一轮改法同样只在模型上证过）。
- `rerun.sh` 里 `verify_semantics.sh` 在小 crate 上编过、跑过（经 `capped.sh 4` 与 `run-with-memory-cap.sh`），这是攻方探针本身的一步，不是这一批的验证链。
- 没改 `.claude/gate.d/stage-inputs.tsv`、别的 gate.d 阶段、`crates/`、`.claude/kb/`；没套实分一的四份 diff；P8 没做；47 号与 doc-lint 的红不是这一批的，没修。
- 没做 git 写操作，没提交、没暂存。
- 看门狗（`research/scripts/agent-watch.py`）没喂这几种命令：它与闸共用同一个 `classify` / `command_under_launcher`，认不认得出是推的。
- 登记给我这一类的门禁阶段：派发没点名 agent 类型，`stage-owners.tsv` 里按「general-purpose」没有登记行，没跑额外阶段。

## 草稿目录与清理

- 删了（交回之前，`du -sh` 记的大小）：`/tmp/claude-1000/gate-batch-m2-r2-fixes/dev`（172M，开发用的仓拷贝）、`before-repo`（172M，改前探针用的仓拷贝）、`rerun-repo`（360M，rerun.sh 用的仓拷贝，含 .git）、`rerun-scratch`（205M，含 `semantics/crate/target` 编译目录）、`probe-scratch-before`（3.9M，探针小仓）；`mut54`（开发副本的一份拷贝）判完当场删了。
- 留着：`backup/`（五份改前原件，`cp -p`）、`my-changes.diff`、各份日志（`before-k1-*.log`、`rerun-out/`、`final-*.log`、`lib-break-*.log`、`guard-*break-*.log`、`lint-*.log`、`demo-d2-marker.log`、`mut54-selftest.log`）、`demo_d2_marker.py`、`54-before-d2.sh`（套完 singlefs-39 替换、D2 之前的 54 号，演示用）。
- 攻方的草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r2-opus/` 不是我建的，没动（我的探针与复跑都把 SCRATCH_ROOT 指到了自己的草稿目录）。

交回时刻：2026-09-26T17:06Z（JST 2026-09-27T02:06）。
