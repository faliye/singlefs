# 拆分之后算键库与几道门禁格的一批治理改动（治理文件轮，第一轮）

方案按 `crates/` 今天的实现来谈：这一批没改 `crates/` 下的代码，判的是算键库、拆分族认法、几道门禁格、样本与登记表。调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md` 第 711 行起「拆分后算键」那一段（第 711–782 行）。开工快照 `research/prompts/split-closure-governance-r1-snapshot/judged-sha256.txt` 点名的 43 份就是被判的文件；`kb-sha256.txt` 记了 `.claude/kb/decisions/13-验证路线.md` 一份。

## 问题

脚本拆成「入口 + 同族部分文件」之后，算键库（`scripts/gate/lib_function_units*.py`）收闭包、几道门禁格认文件、排除表与登记表跟着改了一批。判这一批改得对不对：闭包收没收全、收没收多，拒算该拒的拒了没有，新登记的放行站不站得住，开关会不会红，排除表这一次涨行合不合用户那次定案。

## 改了什么（主 agent 的观测，工具员报告与换上结局）

1. **拆分族认法**：`scripts/gate/lib_lag_split_family.py`（`in_family`、`family_members`、`family_owner`、`keyed_family_growth`）认三种拆法：Python `<模块名>_<部分>.py`（「-」换「_」）、钩子同目录 `<名>-<部分>.sh/.py`、shell `.claude/gate.d/lib/<名>-<部分>.sh/.py`，`main.rs`、`mod.rs` 入口认整个目录。`scripts/gate/comment-history-check.py`、`scripts/gate/lib_verus_trait_impl_lag.py`、`scripts/gate/verus-proof-check.py` 与 `verus_proof_check_*` 照它判「只缩不涨」；`.claude/gate.d/comment-history-lag.tsv` 改成拆后路径。
2. **算键库收闭包**（A、Q、now、RUNTIME、X、Y、Z、P4 八份先后换上）：闭包跨被 source 的库折变量；拆出的 Python 模块照 import 它的入口解析目录；import 时执行的模块级语句进闭包；入口不在 import 时载入自证模块；`importer_call_sites` 只认同族。测试服务代码不进键：`is_test_service_code`，登记表 `scripts/gate/test-service-code-paths.tsv`（新建，25 行），权威是 `.claude/kb/decisions/13-验证路线.md` D13（验证路线） 已定项 17「三类代码」表；`RUNTIME_DATA_FILES`（`lib_function_units_registry.py:36`）登它。X：`exec(compile(<源码>, …), <模块对象>.__dict__)` 照「载入那一份」连，源码折不出照旧拒算 `compiled-part-exec`。Y：`$(<函数体只有一条 printf '%s\n' 的 shell 函数>)`、`IFS= read -r -d '' <变量> <<词` 读进的库正文照内联脚本折；库正文里模块级函数的参数照载入方的调用处折；库正文里的拒算落回它写的那一行；`UNKEYED_PROGRAM_WORDS` 加 `admission_run`（`.claude/gate.d/lib/code-tooling-shared-libraries.sh` 库正文里，判定库记账与算键）与 `TripLauncher.run`（`code-tooling-stage-break-switch-replay-cells.sh` 库正文里，起被复跑的自证）两行、删四行不再起子进程的旧行。Z：`TAIL_READ_SCOPES` 补登上游钩子 `discarded-output-count-check.sh` 的 `run_selftest` 那一处空元组；`shell-heredoc-captures-unfolded` 那一格夹具改成函数体一条 `cat` here-doc；删 `doc-decisions-status-cells.sh` 那一行用不到的 `UNKEYED_PROGRAM_WORDS`。`lib_function_units_registry.py` 里 `TAIL_READ_SCOPES` 另有别的会话在开工快照之前加的一行（`crate-cross-references-check.py` 的 `unkeyed_toolchain_and_configuration_lines`，登空元组），不是这一批，但在被判的那份文件里，照判。P4：删 `lib_function_units_closure_keys.py` 里读 crates 变异判法缓存目录的函数与相关真仓格（与变异键整包同一批，那一包归别的会话）。
3. **code-tooling 几格**：`.claude/gate.d/code-tooling.sh` 文件头开关表、`.claude/gate.d/lib/code-tooling-source-scan-cells.sh`、`code-tooling-rules-and-agents-cells.sh`（change-range 格）、`code-tooling-copy-closure-and-selftest-shape-cells.sh`、`scripts/gate/lib_test_target_sources.py`（`binary_target_source_files`）照拆后的文件认；`.claude/memory-bounded-parallelism.tsv` 入口改回 `code-tooling.sh`、加 e171 一行。
4. **doc-experiments change-range**：`.claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh` 第 1434 行列路径的 git 调用加 `-c core.quotepath=false`；1436、1481、1494 行尾加 `# change-range:exempt 报错文字，点名失败的是哪条子命令，不是调用`。
5. **doc-text date-once 的并行度**：`.claude/gate.d/lib/doc-text-notation-and-translation-cells.sh` 的 `cell_date_once` 改成 source `scripts/hosts/heavy-threads.sh` 取 `heavy_thread_default`，取不出整格判红；`.claude/gate.d/doc-text.sh` 加开关 `GATE_TEXT_NOTATIONS_BREAK=date-once-jobs-unregistered-passes`，样本 `date-once-thread-default-red`。
6. **三张排除表一次性涨行**：用户 2026-11-20 弹窗定「按基准登一次表」（原话的出处 `records/2026-11-16-三方豁免取消与存量补轮.md:757`），提交基准里就有、确实不是跑法的写法逐条登进 `.claude/heavy-thread-exempt`、`.claude/gate.d/hardcoded-quota-exempt.tsv`、`.claude/gate.d/repository-copy-closure-exempt.tsv`，每行写理由，只这一批放开一次「只缩不涨」。

## 已经量的（结局行 type= exit= 那一段，日志在任务服务日志目录，号见调度记录）

- 算键库：Z、P4 与变异键整包都换上之后，`lib_function_units.py --selftest` exit=1、「787 格」里只红一格，是别的会话改 `scripts/gate/crate-cross-references-check.py:919` 带出的尾巴读法（不在这一批，已归它）；P4 换上前（Z 换上后）那一趟 exit=0「788 格都对…弄坏开关 344 个各红在点名的格」。X、Y、Z 新加的开关各单跑 exit=1、红在点名的格。
- import 算键库的脚本：lib_verdict_library、crates-split-pure-move、rust-name-collisions、checker-direction-overlap、verus-proof-check（「196 格都判对；弄坏开关 122 个，每个都让至少一格判错」）自证 exit=0；admission 自证在变异键整包换上前一直 exit=1、43 格「读不了测试服务代码登记表」，整包换上后的那一趟归别的会话在交。
- 算键结果：`key . .claude/gate.d/code-tooling.sh` exit=4、只 5 行 `external-program-unkeyed`（命令行 `key` 不给外部程序取版本，门禁经 admission 算键时给）；`closure . .claude/gate.d/code-tooling.sh` exit=0、0 行拒算；`key . .claude/gate.d/doc-decisions.sh` exit=0。
- 门禁 code-tooling 的 stage-break-switch-replay 格：点两条 memory-bounded 开关与点 agent-write-scope 13 条开关两趟，开跑前「现判 2、复用 0、拒算 0」「现判 13、复用 0、拒算 0」，每条「点了没错：无」，多判错的是 function-units-two-implementations-agree 那 10 份（等 Rust 一侧补丁合入后整组重录）与 13 条那一趟里的 `mutation-list-items-reuse-three-questions`（那一趟跨了变异键整包换上，归别的会话核）。
- doc-text 样本档 exit=0；开关 `date-once-jobs-unregistered-passes` 下只红点名那一个用例。
- 三张排除表：`--check heavy-thread-defaults,hardcoded-quota,repository-copy-closure` 三处「排除表涨了」各 13、4、9 行，与新登的逐行对上。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。

1. **拦不拦得住它要拦的事**：拆分族认法拿「拆出一份新部分文件、滞后表里旧路径那一行没删」与「一份不同族、名字前缀相同的文件」各喂；算键库拿「库正文里起一个折不出的命令」「`$(函数)` 的函数体是两条 printf」「`read -d ''` 从 `"$1"` 那份文件读」「exec 进模块对象而模块对象被交出去」各喂。每一条现在由哪一行判、判成什么；该拒算而收成键的、该收而漏的，就是打中。
2. **误伤与绕过**：`UNKEYED_PROGRAM_WORDS` 新加的 `admission_run`、`TripLauncher.run` 两行与 `TAIL_READ_SCOPES` 新加的那一行，让哪些单元不进哪几条判定的键；对照 `.claude/rules/verification.md`「一个门禁：两种跑法」里「两种排除不在这句全称里」那一句与 `.claude/rules/verification-reference.md`「单元键的组成与排除」，这几行算不算那两种之外的第三种排除；改了 admission.py 或被复跑的自证，会不会有判定该作废而复用了。
3. **复用键与多机**：测试服务代码不进键（D13（验证路线） 已定项 17「三类代码」表）在 `is_test_service_code` 与 `test-service-code-paths.tsv` 上落得对不对，表里 25 行与 kb 那一行逐项对得上没有；拆后闭包里有没有随主机、编译目录、仓根绝对路径变的成分。
4. **不兼容旧格式与旧开关**：拆分族认法、算键库有没有为拆前旧路径、旧开关名留的读法或分支（`.claude/agent-common.md`「写」一节「里程碑 17 收尾之前不兼容任何旧东西」）；`comment-history-lag.tsv` 有没有拆前路径残留。
5. **排除表与用户定案**：三张排除表这一次新登的行，逐行是不是「提交基准里就有、确实不是跑法的写法」（自证造的假值、钉住一份脚本的单文件拷贝、`${X:-1}` 这类不并行的写法），有没有借这次放开把真该改的跑法也登进去；`cell_date_once` 改成取登记值之后，doc-text 那一处是不是已经不该在排除表里。

## 分工

正推腿判第 1、3、4 格；攻方腿判第 2、5 格与「已经量的」那一节里每一句能不能从调度记录与日志里核到。
