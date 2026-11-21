# 旧账本开着项当场修的治理文件轮第一轮：正推腿报告（第 6 到 10 问）

日期：2026-11-19。主工作区 HEAD 4319aa2b0；读的是主工作区此刻的那一版（含别的会话未提交的改动），改了哪几行都对着 `git diff HEAD -- <文件>` 现取。
没有写模型，没有 `rerun.sh`；全部是读文件与只读命令（`grep`、`sed`、`git diff`、`git show`）。没往任务服务交任务。

## 先说读到的范围与偏差

- 读全了「范围」里的：`CLAUDE.md`、`.claude/main-agent.md`、`.claude/rules/implementation-workflow.md`、`.claude/agent-common.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/investigator.md`、`.claude/rules/code-discipline-by-review.md` 的改动行；四份门禁与钩子（`doc-text.sh`、`doc-process-records.sh`、`doc-experiments.sh`、`completion-cleanup-guard.sh`）的文件头与开关表；三份入口脚本 `e07/e53/e159-vm-bench-run.sh`；`selftest-registry.tsv`、`break-switch-replay-files.tsv`、`code-tooling.sh` 的登记行。
- 没读（不属于这几问）：`hook-swap.py`、`changed-paths.sh` 的实现、`e185` 三份、`memory_peaks_lib.py`、`project-code-guard.sh`。
- 背景材料给的行号已漂移（别的会话同期在改同一份文件）：`.claude/agent-common.md` 的「检出钩子 ⑭」今天在第 93 行、「写闸第十一道」在第 107 行（背景写的是 89、101）；`selftest-registry.tsv` 104–106、`break-switch-replay-files.tsv` 90–92、`code-tooling.sh` 2344 三处对得上。
- 这几份文件的 `git diff HEAD` 里有不在背景「七处」清单里的行（别的会话改的，这一轮没有判）：`.claude/main-agent.md` 第 99 行派发表里「经 `scripts/gate/hook-swap.py` 一次换上」那一截与第 57 行去掉 build；`.claude/rules/implementation-workflow.md` 第 60 行 clippy 点名包；`.claude/agents/tooling-writer.md` 第 31、35 行；`.claude/agents/investigator.md` 第 33 行里 prove-red 起法那一段；`.claude/gate.d/doc-text.sh` 里 comment-history 的两条新开关（`prefix-compounds-flagged`、`face-exception-widened`）与 `trilingual-*-comment-open-in-code-red` 几条。
- 引用：只有一条腿读到的范围：没有；两条腿都读「范围」里的文件。

## 第 6 问：定点改说的是不是用户那几句；没改的文件里还有没有旧说法

判定：**部分成立、部分不成立**（核心几句成立；有两处说得比用户宽、一处漏改、全仓还有旧说法与新写法并存）。

### 6.1 用户的原话与两句弹窗定案

`records/2026-11-17-subagent账本清欠调度记录.md:377`：「更改规则 删除agent优化记账的流程 以后agent的问题需要对话当场解决」；弹窗两句：账本「不删账本，只改规则」、射程「撞上就修，三方也当场」。

### 6.2 逐处对

| 改动处 | 新写的 | 对得上用户哪一句 | 判定 |
|---|---|---|---|
| `CLAUDE.md:21` | 子 agent 与协作工具的问题「不记账」「撞上就在当场的对话里解决」「当场走一轮三方，不登账本、不攒批」 | 原话「删除…记账的流程」「当场解决」；射程「撞上就修，三方也当场」 | 成立 |
| `CLAUDE.md:21` | 「旧账本，不再往里登」，路径还在 | 「不删账本，只改规则」 | 成立 |
| `CLAUDE.md:21` | 「也不先弹窗问要不要修」 | 用户两句里都没有这一句；从「撞上就修」推得出，但不是用户说的字 | 推不出（超出原话） |
| `.claude/main-agent.md:14` | 「子 agent 与协作工具的问题例外，撞上就当场修，不弹窗问」 | 同上，「不弹窗问」是主 agent 加的 | 推不出（超出原话） |
| `.claude/main-agent.md:25`（第 2 条） | 删「登进账本的编号小节」，补「不延后、不登账：当场修」 | 「删除…记账的流程」 | 成立 |
| `.claude/main-agent.md:27`（第 4 条） | 治理文件「不聚批：撞上当场派工具员改、换上、验，改完当场走一轮三方」 | 「当场」「三方也当场」 | 成立 |
| `.claude/main-agent.md:27` | 「同一时刻撞上的几条并进同一个工具员」 | 用户没说；与 `.claude/agents/tooling-writer.md:20`「可以不止一条，主 agent 按要改的文件聚簇」是一个意思 | 推不出（主 agent 自己加的，不矛盾） |
| `.claude/rules/implementation-workflow.md:21` | 「改完当场走一轮三方」 | 「三方也当场」 | 成立 |
| `.claude/agent-common.md:4` | 经过与带日期的记录进「`records/` 下这一轮的调度记录」，不再点名旧账本 | 「删除…记账的流程」 | 成立 |
| `.claude/agents/tooling-writer.md:20` | 「主 agent 当场写下的问题原文（现象、命令与输出、文件:行）」代替账本节号与条号 | 「对话当场解决」 | 成立 |
| `.claude/rules/code-discipline-by-review.md:54` | 单例补一种例外 | 弹窗选项「补例外」（`records/2026-11-17-subagent账本清欠调度记录.md:371`） | 见第 8 问 |

「不弹窗问」两处引文：`CLAUDE.md:21`「也不先弹窗问要不要修」、`.claude/main-agent.md:14`「子 agent 与协作工具的问题例外，撞上就当场修，不弹窗问」。调度记录 377 行里这一句是主 agent 自己的概括（「不弹窗问」），不在引号里的用户原话或两句弹窗定案里。要推翻「这是合理推论」的现象：用户另说一句「撞上了要先问我」。

### 6.3 改了的那几处自己漏改的同文件旧句（与新写法矛盾）

| 位置 | 旧句 | 与什么矛盾 |
|---|---|---|
| `.claude/main-agent.md:24`（第 1 条） | 「批内冒出的发现只进下一轮的清单，不当场派。」 | 第 4 条（27 行）与第 2 条（25 行）新写「撞上当场派工具员」。治理文件类的发现在批内冒出来时，第 1 条说不当场派，第 4 条说当场派；第 1 条没写例外 |
| `.claude/main-agent.md:38`（第 15 条） | 「没有会话认领、或对应的会话已经不在，自己做：并进这一轮下一批，照常走流程」 | 治理文件类的缺陷没人认领时，新写法是当场修，不是并进下一批 |
| `.claude/main-agent.md:100`（派发表「测试或门禁结果与预期不符」那一行） | 「先派 `investigator` 写复现与二分的命令…」之后「改法并进…第 4 条下一批」 | 门禁红的改法落在治理文件时，第 4 条新写不聚批；这一行没分 `crates/` 与治理文件两支。（同表第 98 行、第 94 行只指 `crates/` 与 `research/`，不矛盾） |

### 6.4 全仓搜：没改的规则、技能、定义、门禁文件头里还有没有旧说法

搜的范围：`CLAUDE.md`、`.claude/`（含 hooks、gate.d、rules、skills、agents，不含 `.claude/singlefs-ai-sop/` 与 `kb/experiments/`）、`scripts/`、`task-server/`、`prune-server/`、`gpu-verification-server/`、`README.md`、`briefs/`；词：`一周合一批|按批走|一批走一轮|登进账本|记进现行账本|记进账本|登账本|往账本|优化账本|subagent账本|自上而下清欠|账本自上而下`，另对 `main-agent.md`、`agent-common.md`、`agents/`、`rules/`、`skills/` 搜 `欠账|优化项|记账|账本`。命中 47 行（第一组）与 35 行（第二组）；其中引「一百七十四节」这类历史节号当来源的（`project-code-guard.sh`、`runner-dispatch-guard.sh`、`run-scope-guard.sh`、`task-server/` 等）说的是那一次发生的事，不算旧说法，不列。

仍在写旧做法、与新写法矛盾的（给 文件:行）：

| 文件:行 | 原文 | 矛盾在哪 |
|---|---|---|
| `.claude/gate.d/split-proposal-ledgers.tsv:6` | 「新欠账与优化项往这一份末尾追加」 | 新写法：旧账本不再往里登（`CLAUDE.md:21`） |
| `.claude/gate.d/doc-process-records.sh:458` | 「新欠账往现行账本（登记表里 current 那一行点名的那份）追加」 | 同上；这是 sealed-ledgers-unchanged 一格的「为什么」 |
| `.claude/gate.d/doc-process-records.sh:772` | 「或现行账本（.claude/gate.d/split-proposal-ledgers.tsv 里 current 那一份）的编号小节登一行」 | 这是延后项登记红行的出路，写「照 main-agent.md 第 2 条」，而第 2 条（`.claude/main-agent.md:25`）已删掉账本这一支；门禁文字与它指的规则对不上 |
| `.claude/gate.d/doc-process-records.sh:2917` | 「新欠账登这份」 | 同上，同一格的另一处文字 |
| `.claude/gate.d/doc-process-records.sh:454` | 「退役判据：现行账本不再往末尾追加新节」 | 这条判据现在已经满足（规则改成不再往里登），`ledger-section-numbers` 一格按它自己写的该删；用户说「不删账本」，没说不删这一格，要主 agent 定 |
| `.claude/gate.d/stage-owners.tsv:139` | 「由主 agent 与各会话追加」 | 现行账本不再有人追加 |
| `scripts/kb/stale-candidates-record-carriers.tsv:3` | 「现行账本，装着开着的欠账」 | 同上（仍把旧账本当「开着的欠账」的载体） |
| `.claude/rules/session-wrapup.md:15` | 「先记欠账、按批立」 | 说的是「踩过的坑做成会红的检查」的立法时机；门禁与钩子也在新规则的治理文件射程里，「先记欠账、按批立」与「不记账、当场」对不上；是否同一件事要主 agent 判射程 |
| `.claude/rules/show-me-test-by-review.md:12` | 「先记欠账、按批立，正在反复发生的当场立」 | 同上；它指向的上游规则 `.claude/singlefs-ai-sop/rules/sop-first.md:45` 写「一周合一批评估」，上游那份谁都不碰，只能记下 |

没有矛盾、只是描述还在运转的守卫（不改也不坏，调度记录 377 行也写了没动）：`.claude/agent-common.md:95`（检出钩子 ⑯）与 `:106`（写闸第十道）、`.claude/hooks/write-guard.sh:76`、`.claude/hooks/bash-command-detector.sh:362`，它们描述的往现行账本加节的拦截仍在代码里。

`.claude/rules/evidence-discipline.md:153`「新立的判据、撤回的数各记一行进项目的欠账表」说的是 `.claude/kb/checks-owed.md` 那张 kb 欠账表，不是旧账本，`CLAUDE.md:21` 也写了 kb 只管文件系统本身；不算矛盾。

推翻 6.4 的现象：这几处文字有人指出它们指的「欠账表」是 kb 的 `checks-owed.md`、不是子 agent 账本（我对 `session-wrapup.md:15`、`show-me-test-by-review.md:12` 的判定就是靠「它们说的是门禁立法时机」这一读法，没有别的证据说它们指旧账本）。

## 第 7 问：`agent-common.md` 的 ⑭ 与第十一道，对检出钩子文件头 ⑭ 与写闸

判定：**成立**（没有多写、没有写错；少写的几项都在指到的文件头里，不构成矛盾）。方法：把 `.claude/agent-common.md:93`（⑭）与 `:107`（第十一道）逐分句对 `.claude/hooks/bash-command-detector.sh` 的 ⑭ 段（294–334 行）与 `.claude/hooks/write-guard.sh` 的第十一道段（82–108 行）。都是读文件头与源码常量，没喂钩子，所以「两道都不拦」那一句是推的（见「要主 agent 跑的命令」）。

### 7.1 ⑭（`.claude/agent-common.md:93`）逐项

| agent-common 的句子 | 对到的钩子文件头 | 判 |
|---|---|---|
| 「`tooling-writer` 在 Bash 里往两个服务 `src/` 下已存在的文件与两个服务已存在的 `Cargo.toml`、`Cargo.lock` 写、删、挪走或换成链接」 | `.claude/hooks/bash-command-detector.sh:2654` `SERVICE_SOURCE_AGENTS = ("tooling-writer",)`；294 行起头同句；认的写法含 rm、mv 挪走、ln 换链接 | 对 |
| 「落点与写闸第十一道同一组，解开符号链接再判」 | `.claude/hooks/bash-command-detector.sh:295` 「落点是同一组」；「落点先规整 `..`、逐段解开符号链接」 | 对 |
| 「认哪些写法、哪些判不到，以检出钩子文件头 ⑭ 那一段与它「判不到的」那几行为准」 | `.claude/hooks/bash-command-detector.sh:329` 「判不到的（这一项不罩它们）」确有这一段 | 对（指路，不抄清单） |
| 「解包与打补丁按往哪个目录里写判，在仓根、两个服务目录或 `src/` 里做的一概拒，只新建文件的也拒」 | `.claude/hooks/bash-command-detector.sh:331` 「只新建文件的也拒」，同段「在仓根、prune-server/、gpu-verification-server/ 或 src/ 里解包、打补丁的一概拒」 | 对 |
| 「从服务源码建出的硬链接，落在草稿目录里也认」 | `.claude/hooks/bash-command-detector.sh:305` 「落在仓外的草稿目录里的硬链接也认」 | 对 |
| 放行：「落点不存在的新建（解包、打补丁、`ln` 从服务源码建的除外）、草稿目录与仓副本里与服务源码不同 inode 的同名文件、主 agent、别的 agent 类型与认不出类型的子 agent 调用」 | ⑭ 段「放行：新建的文件（落点不存在）；草稿目录与仓副本里的同名文件；主 agent、别的 agent 类型；认不出 agent 类型的子 agent 调用」，「往已经是服务源码硬链接的文件写也拒（按设备号与 inode 号认）」 | 对 |
| 出路：仓副本里改、存成补丁、打补丁命令写进报告交主 agent | ⑭ 段「出路：在仓副本里改…打补丁的命令写进报告」 | 对 |

少写（不算错）：⑭ 的「前台、run_in_background 一样拒」「两个服务的 `target/`、`--dry-run` 的定点写脚本放行」没抄；前一句由 `.claude/agent-common.md` 检出钩子总括里「其余几处都拒」盖住，后两句是放行项，抄不抄不影响判法。

### 7.2 第十一道（`.claude/agent-common.md:107`）逐项

| agent-common 的句子 | 对到的写闸文件头与常量 | 判 |
|---|---|---|
| 目录 `scripts/`、`.claude/gate.d/`（`fixtures/` 除外）、`.claude/scripts/`、`task-server/`、`prune-server/`、`gpu-verification-server/` 下的 `.sh`、`.py` 或带执行位的文件；两个服务的 `target/` 除外 | `.claude/hooks/write-guard.sh:748` 起的 `SWAP_ONLY_AGENTS = ("tooling-writer",)`、`SWAP_ONLY_DIRECTORIES`、`SWAP_ONLY_EXCLUDED`（`.claude/gate.d/fixtures`、两个服务的 target） | 对 |
| 两个服务 `src/` 下已存在的文件（不限后缀）、两个服务已存在的 `Cargo.toml`、`Cargo.lock` | `SWAP_ONLY_SOURCE_DIRECTORIES`、`SWAP_ONLY_SOURCE_FILES`；文件头「不看后缀与执行位，一律拒」 | 对 |
| 「Bash 里往后两类已存在的文件写…由检出钩子 ⑭ 执行前拒」 | 写闸文件头「Bash 里往这几处已存在的文件写（追加、cp、定点替换工具这些）归 bash-command-detector.sh 的 ⑭，判的是同一组落点」 | 对 |
| 「Bash 里用定点替换工具…分几次改主工作区里已存在的脚本，两道都不拦，同样不许这样改」 | `.claude/hooks/write-guard.sh:107` 「Bash 里用定点替换工具分几次改同一份脚本，这道闸看不见。」；⑭ 的落点组是两个服务的 `src/` 与构建清单，不含 `scripts/` 下的脚本（推的：没喂钩子） | 对 |
| 出路：新版与规格（`make-swap-spec.py` 生成）写草稿目录、`hook-swap.py` 换上命令写进报告；`--verify` 按 `selftest-temporary-service.tsv` 分两种，门禁阶段另走一支 | 写闸文件头出路段：required 不加 `--verify`、own-server 加、门禁阶段单走一支；`make-swap-spec.py` 在出路里 | 对 |
| 「新建的文件一次写全」 | 写闸文件头放行「新建文件」；`.claude/agents/tooling-writer.md` 第 3 步「新建的脚本用 Write 一次写全」 | 对 |
| 「主 agent 与别的 agent 类型两道都不判」 | 写闸「放行：…别的 agent 类型与主 agent」；⑭「主 agent（输入里没有 agent_type）、别的 agent 类型」 | 对 |

少写（不算错）：写闸文件头的「钩子（`.claude/hooks/`）不在这一道的目录里：钩子照旧只经 hook-swap.py 的钩子规格换上」（`.claude/hooks/write-guard.sh:102`）在 `.claude/agent-common.md:107` 里没写；`.claude/agents/tooling-writer.md` 第 3 步写了，`.claude/agent-common.md` 没有也不错。

推翻 7 的现象：用 tooling-writer 的身份喂 `bash scripts/kb/replace-once.py` 改 `scripts/` 下已存在的 `.sh`，两道里有一道拒了（则「两道都不拦」写错）。

## 第 8 问：单例例外（`.claude/rules/code-discipline-by-review.md:54`）

判定：与两份规则**不冲突（成立）**；射程**不成立**：今天 `crates/` 里 38 处 `static … OnceLock`，落在「外部句柄」里的明确只有 3 处，另有 4 处要读类型才知道，其余 31 处不在例外里。

### 8.1 与 `.claude/singlefs-ai-sop/rules/code-discipline.md`、`.claude/rules/machine-first.md` 有没有冲突

- 新写的句子是「例外只有一种：进程级、建好之后只读的外部句柄」（`.claude/rules/code-discipline-by-review.md:54`），挂在「留下「默认不可变」「不用全局可变状态；不用单例」」之后。
- `grep -nE '单例|全局|OnceLock|静态|static|lazy'` 对 `.claude/singlefs-ai-sop/rules/code-discipline.md` 与 `.claude/rules/machine-first.md` 两份：零命中。上游那份没有任何一句要求「不用单例」，`code-discipline-by-review.md:54` 的「留下」是这份审过流行做法的清单自己的一条；所以例外没有可冲突的上游条款。
- `machine-first.md` 的「留下的（与谁读代码无关）」里有「测试可复现」「推理范围要有界」；进程级全局状态天然拉宽推理范围、让用例之间共用状态。例外写了「建好之后还会被改的状态不在例外里」，这一句把可变的挡在外面，所以与那两条不冲突；没有更多文字可判。
- 与用户定案对：`records/2026-11-17-subagent账本清欠调度记录.md:371` 写「补例外」，并写明「真卡共用模块的 `OnceLock` 不改」；规则句比调度记录多了括注「GPU 实例这类，建一次代价大、重复建会出错的」与最后一句「建好之后还会被改的状态不在例外里」，都是收窄，没放宽。

### 8.2 今天 `crates/` 里 `OnceLock` 的几处各落不落在里面

命令（量的）：`grep -rn OnceLock crates/ | wc -l` 得 89 行，`grep -rln OnceLock crates/ | wc -l` 得 26 个文件（其中 3 行在 `crates/mutations.tsv`）。`static … OnceLock` 声明（含类型折行的）共 38 处，清单在我的草稿目录 `statics.txt`。其余是结构体字段、局部变量、`use`、文档注释，不是全局，不归「不用单例」管。

按声明处的类型与上方文档注释分三类（落在没落，靠读声明与文档注释，是推的）：

| 类 | 处数 | 哪几处 |
|---|---|---|
| 落在例外里：进程级外部句柄 | 3 | `crates/singlefs-verification/src/gpu_unit_checks.rs:625`「static VULKAN_INSTANCE: std::sync::OnceLock<wgpu::Instance>」（上方文档写了为什么只建一份：静态 TLS 被 ICD 吃光）；`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:681` 测试进程那一份 Vulkan 实例（上方注释写了为什么一个进程只建一个）；`crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/oracle_verdicts_of_a_state.rs:138` 判定库连接 `LibraryOpenInThisProcess`（文档只写了「这个进程的判定库」，没写为什么只建一次；推的：算连服务的句柄） |
| 要读类型才知道（卡租约、卡池、开卡记录） | 4 | `gpu_card_configuration.rs:511` `CARDS_LEASED_BY_THIS_PROCESS`；`gpu_unit_checks.rs:3065` `GPU_CARD_POOL`；`e202_gpu_cpu_gc_comparison.rs:4343` `OPENINGS_NOT_RETURNED_IN_THIS_PROCESS`、`:5782` `CARDS_SEEN_BY_THIS_PROCESS`。是句柄还是会变的状态，我没读到类型的定义，标「推不出」 |
| 不在例外里 | 31 | 4 处 `GivenStatesEngine`（`e156_allocation_basis_counts.rs:721`、`e168_delete_ring_window_borrow.rs:1368`、`e171_three_oracle_collusion.rs:2008`、`e171_three_oracle_collusion_after_the_oracle_fixes.rs:5556`；`crates/singlefs-verification/src/crash_amplification.rs:26268`「库与账各一把锁」，建好之后里面还在变，被例外末句挡在外）；算出来的查表与摘要 11 处（`crates/singlefs-core/src/checksum.rs:25` 与 `:200` 的 CRC 表、`crates/singlefs-checker/src/lib.rs:122`、`crates/singlefs-unit-graph/src/rust_items.rs:114` 与 `wgsl.rs:251` 的词集、`crash_amplification.rs:27631` 与 `:27646`、`e185_gc_batch_wall_clock_split.rs:4012`、`e180_admission_reading_versus_writable_bytes.rs:850`、`gpu_unit_checks.rs:1020` 的起点时刻、`crash_library.rs:100` 的工具链身份）；缓存的一次性数据与测试夹具 16 处（`e182_unreadable_abandoned_root_forks.rs` 的四个 `HISTORY`、`e165_replay_bounded_enumeration.rs` 的两个 `PINNED`、`mount_runtime_accept_raises_the_floor_for_space.rs:340-342` 三个 `FilledPool`、`record_checker_breaks_the_history_at_the_power_loss_seam.rs:131`、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:3439`、`crash_identity.rs:637`、`gpu_unit_checks.rs:2210`、`gpu_card_configuration.rs:156` 与 `:810`、`common_discrete_gpus_of_this_host/mod.rs:666`） |

读法后果：这 31 处里有生产代码（`crates/singlefs-core/src/checksum.rs:25`、`crates/singlefs-checker/src/lib.rs:122`、`crates/singlefs-unit-graph/src/rust_items.rs:114`），都是「建好之后只读」，但不是「外部句柄」。按 `.claude/rules/code-discipline-by-review.md:54` 的字面，「不用单例」仍管着它们，例外不罩它们。这条规则只在 review 判（`.claude/rules/machine-first.md` 的分工），没有门禁，所以不会红；但 review 的人按字面读会逐处打回，或者要自己再加一条没写的例外。写这句的人要答的问题是：只读的计算出来的表要不要同样算例外。

推翻 8.2 的现象：这 31 处里有一处其实是进程外的句柄（例如 `LibraryOpenInThisProcess` 之外还有哪个类型包着服务连接），或写这条例外的人说「查表与夹具本来就不算单例」。

## 第 9 问：开关点名与登记行

判定：**四份门禁与钩子的点名、三份入口的三张登记行都对得上脚本今天的开关表与格名（成立，静态核）；一处登记行的开关数过期（不成立）**。「红在点名的格」这一半没有重跑，读数是主 agent 背景里报的，在这里标「复核不了，待主 agent 跑」（命令见末尾）。

### 9.1 四份文件头新增或改动的点名

| 文件 | 开关 → 点名的格 | 脚本里有没有这条开关、样本在不在 | 判 |
|---|---|---|---|
| `.claude/gate.d/doc-text.sh` | `GATE_TEXT_NOTATIONS_BREAK=prefetch-error-dropped` → `trilingual-changed-together-git-error-red`（文件头 238–239 行，另在三处样本清单里补了该格名） | 代码里 `.claude/gate.d/doc-text.sh:2989` 认这条开关；样本目录 `.claude/gate.d/fixtures/doc-text.sh/trilingual-changed-together-git-error-red/` 在，`expect` 要「git 的报错原文：」「diff.renames」 | 对 |
| `.claude/gate.d/doc-process-records.sh` | `DOC_PROCESS_RECORDS_BREAK=change-range-error-dropped` → `batch-scope-git-error-red`（473–474 行；格说明 495–496 行；红样本清单两处补了格名） | `load_change_range` 里 `[[ "${DOC_PROCESS_RECORDS_BREAK:-}" == change-range-error-dropped ]]`；样本目录 `.claude/gate.d/fixtures/doc-process-records.sh/batch-scope-git-error-red` 在 | 对 |
| `.claude/gate.d/doc-experiments.sh` | `EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=change-range-error-dropped` → `preregistration-parallelism-git-error-red`；`…=replay-exact-ignored` → `evidence-in-repo-green` | `print_change_range_error` 里认前一条；Python 段里 `!= "replay-exact-ignored"` 认后一条；样本目录 `preregistration-parallelism-git-error-red`、`evidence-in-repo-green`（装置 E912）、`evidence-in-repo-red`（E913、E914）都在 | 对 |
| `.claude/hooks/completion-cleanup-guard.sh` | `COMPLETION_CLEANUP_BREAK=handback-declaration-unchecked` → `handback-bad-declaration-blocked`（346–347 行）；另在 `build-directory-unrecognized` 下补了 `swap-tree-gone-not-listed`（155 行） | 开关表 `BREAK_TABLE` 第 445 行有这对；代码第 1905 行 `broken("handback-declaration-unchecked")`；自证第 2581 行 `check("handback-bad-declaration-blocked", …)`；`swap-tree-gone-not-listed` 在第 2727 行有这一格 | 开关与格名对；`build-directory-unrecognized` 为什么红 `swap-tree-gone-not-listed` 我读不出因果（那一格只核树没了、结局 pass），靠主 agent 报的「复跑了 73 条开关…逐格相同 73 条」，我没重跑 |

另：`doc-text.sh` 里 comment-history 的两条新开关（`prefix-compounds-flagged`、`face-exception-widened`）与 `trilingual-untranslated-comment-open-in-code-red` 等几条不在这一轮改动清单里，没判。

### 9.2 三张登记

脚本的开关表与格：`scripts/experiment-apparatus/e07-vm-bench-run.sh` 文件头 `E07_VM_BENCH_RUN_BREAK=` 现数 12 条（`grep -cE '^#   E07_VM_BENCH_RUN_BREAK='` 得 12；e53、e159 各 12）；自证 `cell_` 函数 12 个，`run_selftest` 里一格一行派 12 格；文件头点名的「」原文 13 行（一条开关 `output-delivered-before-judging` 点名两格）逐条对上 11 个格名（12 个 `selftest_expect` 里 `cell_no_arguments` 那一格没有开关点名它，不算错）。e53、e159 两份的开关表、格函数与 e07 逐行同构，两两 `diff` 只差实验名、二进制名与各格的参数样例（我把实验名归一后逐行比的，比对文件在草稿目录 `n07.txt`、`n53.txt`、`n159.txt`）。

| 登记行 | 内容 | 对脚本 | 判 |
|---|---|---|---|
| `.claude/hooks/selftest-registry.tsv:104`、`:105`、`:106` | 种类 `selftest`，「十二格并行；不起 cargo、编译、虚机」 | 12 格并行；假 cargo、假 `scripts/devices/vm-bench.sh`；脚本里 `prune` 零命中，所以不该登进 `selftest-temporary-service.tsv`（那张表里也没有这三份） | 对 |
| `.claude/gate.d/break-switch-replay-files.tsv:90`、`:91`、`:92` | 「十二条开关（文件头 E07_VM_BENCH_RUN_BREAK= 现数）；自证十二格并行」 | 12 条、12 格 | 对；与同批的 e56 那一行（89 行）相比，这三行没写「拒算 0」，是没报、不是错 |
| `.claude/gate.d/code-tooling.sh:2344` | `"bash scripts/experiment-apparatus/e07-vm-bench-run.sh --selftest" "…e53-…" "…e159-…"` 三条写成字面命令词 | 在 `SELFTEST_RUNNERS`（2262 行起）里，写法与 2343 行 e56 的一样；`selftest-registry.tsv` 里是 `selftest` 类，不需要 `--with-temporary-service` 包装 | 对 |

### 9.3 对不上的一处

`.claude/gate.d/break-switch-replay-files.tsv:10` 完工清理闸那一行写「七十二条开关（文件头 COMPLETION_CLEANUP_BREAK= 现数）」「60 ×（1 + 72 ÷ 31 向上取整）= 240」；今天文件头 `grep -cE '^#   COMPLETION_CLEANUP_BREAK='` 得 73，`BREAK_TABLE` 里 `grep -cE '^    \("[a-z-]+", "'` 也得 73（加了 `handback-declaration-unchecked` 一条）。登记行说的是旧数（它自己也写了「这一版没有照这一格的判法复跑过基准趟」）；限时 480 没变，不影响这一格判红，只是开关数过期。

推翻 9 的现象：主 agent 在换上之后的树上跑 `break-switch-replay` 那一格，点名的格与实际红的格有一处不同；或 `doc-text.sh`、`doc-process-records.sh`、`doc-experiments.sh` 三份的新开关打开后红的不是点名的那一格。这一半我没量。

## 第 10 问：`investigator.md:33` 与 `tooling-writer.md` 第 20 行，对 `implementation-writer.md` 第 1b 步与「子 agent 不交任务」是不是一个说法

判定：`tooling-writer.md:20` **成立**；`investigator.md:33` **不成立**（交接行的「在里面跑」与同一行前文对不上；「留给合入后验证」搬来了一个这一支没有的下游；它指向的实现员定义自己没改全；另有一处共用约束措辞待统一）。

### 10.1 `.claude/agents/tooling-writer.md:20`

只改了「关的是哪几条」的取材：账本节号与条号换成「主 agent 当场写下的问题原文（现象、命令与输出、文件:行）」。这一行不写命令，不碰 clippy、build、起任务；与 `.claude/agents/implementation-writer.md` 第 1b 步、`.claude/agent-common.md` 的「子 agent 不交任务」不发生交叉。派发闸只核「## 关的是哪几条」这一节在、节下不空（`.claude/hooks/runner-dispatch-guard.sh:2549` 的 ⑱），不核内容里有没有账本节号。成立。

（`.claude/agents/tooling-writer.md` 第 31、35 行同期也有改动，不在这一问射程；我只核了第 35 行「弄坏开关」那一句与 `.claude/agent-common.md:60` 把「弄坏开关」列进「要主 agent 跑的命令」一致。）

### 10.2 `.claude/agents/investigator.md:33`

| 哪一处 | 位置 | 对不上的两句 | 判 |
|---|---|---|---|
| 搬来的「合入后验证」 | `.claude/agents/investigator.md:33` 对 `.claude/agents/implementation-writer.md:32`（第 1b 步） | investigator 写「clippy（不写 cargo build，链接留给合入后验证，照…第 1b 步）」，是把 1b 步的话搬过来；1b 步的「合入后验证」是实现员的补丁在验证段做的（`.claude/agents/implementation-writer.md:32` 「不写 cargo build，链接留给合入后验证」），而「查到就修」这一支「这一支没有段树与快照」（`.claude/agents/investigator.md:33`），并且「调查员「查到就修」那一支是例外，照旧在主工作区改」（`.claude/main-agent.md:47`），改动不经补丁、不进合入后验证。这一支没有「合入后验证」可留。`scripts/gate/merge-and-verify.sh:7` 写「合入后验证做 fmt、clippy、测试、验证包四步，不跑 cargo build」（clippy `--all-targets` 已做类型检查），所以连实现员那一支「链接」也只由点名的测试编出的测试二进制链一遍；这一支点名的测试函数同样会链测试二进制，但没有哪一步链 bin 目标（推的：没跑过）。不是「不能这样写」，是「留给合入后验证」这五个字在这一支没有对象 | 不成立 |
| 交接行的「在里面跑」 | `.claude/agents/investigator.md:33` 对 `.claude/agents/implementation-writer.md:48`（第 3a 步） | investigator 写「交接 在里面跑 prove-red.sh」（`.claude/agents/investigator.md:33` 的 `.deliverables` 那一行）；实现员第 3a 步现在写成 `--cwd <仓根>`、`--copy <副本>`（「从主工作区的仓根起主工作区那一份证红脚本、`--copy` 指到副本：起副本里那一份时它把副本认成主工作区、退 2」，`.claude/agents/implementation-writer.md:48`），`scripts/mutation/prove-red.sh:171` 也写「--copy 必须是一份副本…不是这份脚本所在的主工作区」。主 agent 不在副本里跑它，只把副本当参数给；所以交接行里「在里面跑」与同一行前面刚写的「--cwd 写仓根」自己对不上。副本要留到主 agent 跑完是对的（要在 `.deliverables` 登「交接」），说法要改成「`--copy` 的参数」之类 | 不成立 |
| 实现员定义自相矛盾 | `.claude/agents/implementation-writer.md:51`（第 4 步）对同文件第 1b 步 | 第 1b 步写「不写 cargo build」；第 4 步「验证命令只到这几样」里仍有一项「cargo build --offline --all-targets」。`.claude/agents/investigator.md:33` 要求「照…第 1b 步」，而它指向的实现员定义自己前后不一。 | 不成立（同一份定义内的矛盾，改在实现员定义） |
| 共用约束清单措辞 | `.claude/agent-common.md:60` 与 `:69` | 「子 agent 不交任务」那一条的清单仍写「clippy / build」：「测试、带 `--item` 的门禁与崩溃放量入口、clippy / build、证红」；`:69` 「clippy / build 逐条的判法在 `.claude/hooks/project-code-guard.sh` 文件头」。这两处是按类别点名「要写进报告的命令种类」，没写 build 该不该写；与 1b 步的「不写 cargo build」不直接冲突，但读的人会问「build 到底写不写」 | 推不出（措辞不一致，不构成两种做法） |

investigator 另一处与实现员一致：`clippy` 点名改到的包、`-D` 照 `check.sh` 现抄由 1b 步写，investigator 是「照第 1b 步」指过去，不另写；`cargo fmt` 自己跑、命名检查 `<根>` 写仓根与 `.claude/agent-common.md:118` 一致。

推翻 10.2「搬来的「合入后验证」」一行的现象：主 agent 说明「查到就修」那一支改完之后也要走合入后验证（那就是 `.claude/main-agent.md:47` 那条例外写错了），或有一步在这一支上编 bin 目标。

## 要主 agent 跑的命令

没有自己起项目代码。下面三条只为复核 9.3 与 9.1 里我没重跑的那一半，其余读数都是读文件得的。

1. 完工清理闸开关复跑：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 一格点名 .claude/hooks/completion-cleanup-guard.sh; 范围=2; 为什么=核登记行说七十二条而文件头今天数得 73 条，及 build-directory-unrecognized 下新补的 swap-tree-gone-not-listed 点名对不对' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/completion-cleanup-guard.sh`；在哪个目录起：仓根；看哪一行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`（`exit=0` 才算点名的格与实际红的格逐格相同），`task-server/task log <号>` 里该格报的复跑条数（应是 73）与 ✗ 行。命中即打中：条数不是 73，或有一条开关红的格与点名不同。读数没量过，标「推的」。
2. 三份入口开关复跑：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 一格点名 e07、e53、e159 三份入口; 范围=4; 为什么=核 break-switch-replay-files.tsv 第 90 到 92 行登记的十二条开关与格名' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/experiment-apparatus/e07-vm-bench-run.sh --item scripts/experiment-apparatus/e53-vm-bench-run.sh --item scripts/experiment-apparatus/e159-vm-bench-run.sh`；在哪个目录起：仓根；看哪一行：同上，三份各报 12 条、点名的格与实际红的格逐格相同、拒算 0。读数没量过，标「推的」。
3. 第 7 问「两道都不拦」没有喂过钩子；要证它，需要一份以 `tooling-writer` 身份喂 `bash scripts/kb/replace-once.py` 改 `scripts/` 下已存在 `.sh` 的钩子样本；我拼不出能照喂的整条（钩子喂入的 JSON 形态由 `.claude/hooks/bash-command-detector.sh --selftest` 的样本给），所以这一格写「待主 agent 跑」，不拼命令。

## 引文核对（原文逐句，给 cite-check 逐条比；上文表格里的 文件:行 与这里同一行）

### 第 6 问
- 「更改规则 删除agent优化记账的流程 以后agent的问题需要对话当场解决」（`records/2026-11-17-subagent账本清欠调度记录.md:377`）
- 「不删账本，只改规则」（`records/2026-11-17-subagent账本清欠调度记录.md:377`）
- 「撞上就修，三方也当场」（`records/2026-11-17-subagent账本清欠调度记录.md:377`）
- 「也不先弹窗问要不要修」（`./CLAUDE.md:21`）
- 「子 agent 与协作工具的问题例外，撞上就当场修，不弹窗问」（`.claude/main-agent.md:14`）
- 「子 agent 与协作工具的事不延后、不登账：当场修，照第 4 条」（`.claude/main-agent.md:25`）
- 「同一时刻撞上的几条并进同一个工具员」（`.claude/main-agent.md:27`）
- 「批内冒出的发现只进下一轮的清单，不当场派。」（`.claude/main-agent.md:24`）
- 「并进这一轮下一批，照常走流程」（`.claude/main-agent.md:38`）
- 「测试或门禁结果与预期不符」（`.claude/main-agent.md:100`）
- 「新欠账与优化项往这一份末尾追加」（`.claude/gate.d/split-proposal-ledgers.tsv:6`）
- 「新欠账往现行账本（登记表里 current 那一行点名的那份）追加」（`.claude/gate.d/doc-process-records.sh:458`）
- 「退役判据：现行账本不再往末尾追加新节」（`.claude/gate.d/doc-process-records.sh:454`）
- 「或现行账本（.claude/gate.d/split-proposal-ledgers.tsv 里 current 那一份）的编号小节登一行」（`.claude/gate.d/doc-process-records.sh:772`）
- 「新欠账登这份」（`.claude/gate.d/doc-process-records.sh:2917`）
- 「由主 agent 与各会话追加」（`.claude/gate.d/stage-owners.tsv:139`）
- 「现行账本，装着开着的欠账」（`scripts/kb/stale-candidates-record-carriers.tsv:3`）
- 「先记欠账、按批立」（`.claude/rules/session-wrapup.md:15`）
- 「先记欠账、按批立，正在反复发生的当场立」（`.claude/rules/show-me-test-by-review.md:12`）
- 「一周合一批评估」（`.claude/singlefs-ai-sop/rules/sop-first.md:45`）

### 第 7 问
- 「以检出钩子文件头 ⑭ 那一段」（`.claude/agent-common.md:93`）
- 「两道都不拦，同样不许这样改」（`.claude/agent-common.md:107`）
- 「落点是同一组」（`.claude/hooks/bash-command-detector.sh:295`）
- 「落在仓外的草稿目录里的硬链接也认」（`.claude/hooks/bash-command-detector.sh:305`）
- 「判不到的（这一项不罩它们）」（`.claude/hooks/bash-command-detector.sh:329`）
- 「只新建文件的也拒」（`.claude/hooks/bash-command-detector.sh:331`）
- 「SERVICE_SOURCE_AGENTS = ("tooling-writer",)」（`.claude/hooks/bash-command-detector.sh:2654`）
- 「钩子（.claude/hooks/）不在这一道的目录里」（`.claude/hooks/write-guard.sh:102`）
- 「Bash 里用定点替换工具分几次改同一份脚本，这道闸看不见。」（`.claude/hooks/write-guard.sh:107`）
- 「SWAP_ONLY_AGENTS = ("tooling-writer",)」（`.claude/hooks/write-guard.sh:748`）

### 第 8 问
- 「例外只有一种：进程级、建好之后只读的外部句柄」（`.claude/rules/code-discipline-by-review.md:54`）
- 「补例外」（`records/2026-11-17-subagent账本清欠调度记录.md:371`）
- 「static VULKAN_INSTANCE: std::sync::OnceLock<wgpu::Instance>」（`crates/singlefs-verification/src/gpu_unit_checks.rs:625`）
- 「库与账各一把锁」（`crates/singlefs-verification/src/crash_amplification.rs:26268`）
- 「static TABLES: std::sync::OnceLock<[[u32; 256]; 8]>」（`crates/singlefs-core/src/checksum.rs:25`）
- 「static TABLE: std::sync::OnceLock<[u32; 256]>」（`crates/singlefs-checker/src/lib.rs:122`）
- 「static KEYWORDS: OnceLock<HashSet<&'static str>>」（`crates/singlefs-unit-graph/src/rust_items.rs:114`）
- 「static LIBRARY: std::sync::OnceLock<LibraryOpenInThisProcess>」（`crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/oracle_verdicts_of_a_state.rs:138`）

### 第 9 问
- 「GATE_TEXT_NOTATIONS_BREAK=prefetch-error-dropped」（`.claude/gate.d/doc-text.sh:238`）
- 「prefetch-error-dropped」（`.claude/gate.d/doc-text.sh:2989`）
- 「DOC_PROCESS_RECORDS_BREAK=change-range-error-dropped」（`.claude/gate.d/doc-process-records.sh:473`）
- 「COMPLETION_CLEANUP_BREAK=handback-declaration-unchecked」（`.claude/hooks/completion-cleanup-guard.sh:346`）
- 「("handback-declaration-unchecked", "handback-bad-declaration-blocked")」（`.claude/hooks/completion-cleanup-guard.sh:445`）
- 「十二格并行」（`.claude/hooks/selftest-registry.tsv:104`）
- 「十二条开关（文件头 E07_VM_BENCH_RUN_BREAK= 现数）」（`.claude/gate.d/break-switch-replay-files.tsv:90`）
- 「七十二条开关（文件头 COMPLETION_CLEANUP_BREAK= 现数）」（`.claude/gate.d/break-switch-replay-files.tsv:10`）
- 「"bash scripts/experiment-apparatus/e07-vm-bench-run.sh --selftest"」（`.claude/gate.d/code-tooling.sh:2344`）

### 第 10 问
- 「不写 cargo build，链接留给合入后验证」（`.claude/agents/implementation-writer.md:32`）
- 「cargo build --offline --all-targets」（`.claude/agents/implementation-writer.md:51`）
- 「起副本里那一份时它把副本认成主工作区、退 2」（`.claude/agents/implementation-writer.md:48`）
- 「链接留给合入后验证」（`.claude/agents/investigator.md:33`）
- 「这一支没有段树与快照」（`.claude/agents/investigator.md:33`）
- 「交接 在里面跑 prove-red.sh」（`.claude/agents/investigator.md:33`）
- 「那一支是例外，照旧在主工作区改」（`.claude/main-agent.md:47`）
- 「合入后验证做 fmt、clippy、测试、验证包四步，不跑 cargo build」（`scripts/gate/merge-and-verify.sh:7`）
- 「不是这份脚本所在的主工作区」（`scripts/mutation/prove-red.sh:171`）
- 「clippy / build、证红」（`.claude/agent-common.md:60`）
- 「clippy / build 逐条的判法在」（`.claude/agent-common.md:69`）

## 顺带（不在第 6 到 10 问里，给主 agent 看）

- `.claude/agent-common.md:129` 写「装置或变异表改了而 `research/results/` 里没有一份不比它旧的产物」都红；这一轮 `doc-experiments.sh` 给 evidence-in-repo 加了「复跑表 exact 行」一路（`.claude/gate.d/doc-experiments.sh:820` 「不要求新产物、也不比修改时刻」），装置比产物新、而复跑表里是 exact 行且点名的产物在盘时放行。共用约束那一句没写这个例外，读的人会以为装置一改就红。条款对不对归攻方腿第 2 问判。
- `.claude/agent-common.md:109` 的完工清理闸一条没写这一轮新加的「交回时 `.deliverables` 写法不合当场拒」（`grep -c '写法不合'` 对第 109 行得 0）；闸的文件头 `.claude/hooks/completion-cleanup-guard.sh:108` 起写了。共用约束只写「交回、收工时…还有仓副本…」，没提这一拒。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 6 问：每处说的是不是用户那几句 | 部分不成立 | 核心几句（不记账、当场修、三方当场、不删账本）成立；「也不先弹窗问要不要修」（`./CLAUDE.md:21`）与「不弹窗问」（`.claude/main-agent.md:14`）与「同一时刻撞上的几条并进同一个工具员」（`.claude/main-agent.md:27`）是主 agent 加的，推不出 |
| 第 6 问：改了的文件里漏改的旧句 | 不成立 | `.claude/main-agent.md:24`、`:38`、`:100` 三处仍写「并进下一批」「不当场派」，与第 2、4 条新写法并存 |
| 第 6 问：全仓旧说法 | 不成立 | 门禁与登记表里 7 处（`split-proposal-ledgers.tsv:6`、`doc-process-records.sh:458`、`:772`、`:2917`、`:454`、`stage-owners.tsv:139`、`stale-candidates-record-carriers.tsv:3`）仍写往现行账本追加；2 处规则（`session-wrapup.md:15`、`show-me-test-by-review.md:12`）与上游 `sop-first.md:45` 仍写「先记欠账、按批立」「一周合一批评估」，是否同一件事要主 agent 判射程 |
| 第 7 问：⑭ 与第十一道对钩子头与写闸 | 成立 | 逐分句对上，没有多写与写错；背景给的 89、101 行已漂成 93、107 行；「两道都不拦」是推的（没喂钩子） |
| 第 8 问：例外与两份规则 | 成立（无冲突） | 上游 `code-discipline.md` 与 `machine-first.md` 里没有单例条款可冲突 |
| 第 8 问：例外的射程 | 不成立 | `crates/` 里 38 处 `static … OnceLock`：3 处明确在内，4 处推不出，31 处（含 `singlefs-core`、`singlefs-checker`、`singlefs-unit-graph` 的生产代码查表）不是外部句柄、不在例外里 |
| 第 9 问：四份文件头点名 | 成立（静态核） | 开关、点名的格、样本目录与代码里的开关名逐条对上；红在点名的格那一半没重跑，待主 agent 跑 |
| 第 9 问：三张登记 | 成立（静态核） | 104–106、90–92、2344 与脚本的 12 条开关、12 格、`selftest` 类一致 |
| 第 9 问：完工清理闸登记行 | 不成立 | `break-switch-replay-files.tsv:10` 写七十二条，文件头与开关表今天数得 73 |
| 第 10 问：`tooling-writer.md` 第 20 行 | 成立 | 只换了问题取材，不碰命令 |
| 第 10 问：`investigator.md` 第 33 行 | 不成立 | 「链接留给合入后验证」搬来一个这一支没有的下游；交接行「在里面跑 prove-red.sh」与同行前文「--cwd 写仓根」对不上；指向的实现员定义第 4 步（51 行）仍写 `cargo build`；共用约束 60、69 行仍写「clippy / build」（措辞，推不出） |

## 没做什么

- 不判别的腿的格（第 1 到 5 问）没碰，不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数（第 9 问红在点名的格、完工清理闸的 73 条、三份入口的 12 条）没复跑，命令在上面「要主 agent 跑的命令」一节，读数没量过的一律是「推的」。
- 没喂钩子（第 7 问的「两道都不拦」是读文件头与源码常量推的）；没读 `hook-swap.py`、`changed-paths.sh`、E185 三份与 `memory_peaks_lib.py` 的实现。
- 第 8 问里「不在例外里」的 31 处与「推不出」的 4 处是读声明与上方文档注释分的类，没有逐个读类型的完整定义（只读了 `GivenStatesEngine` 与 `LibraryOpenInThisProcess` 的文档与字段）。
- 第 6 问全仓搜的范围写在 6.4 开头；不含 `.claude/singlefs-ai-sop/` 内除 `rules/sop-first.md` 之外的文件、不含 `.claude/kb/experiments/`，没搜 `research/` 与 `records/`（那里是当时的记录）。
- 清理：没有仓副本、没有编译目录、没有往 `crates/` 或别处写。没删 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r1-forward`：里面只有 328K 的文本读数（`statics.txt`、`diffs.txt`、`n07.txt` 等），交主 agent 核查时要用。
