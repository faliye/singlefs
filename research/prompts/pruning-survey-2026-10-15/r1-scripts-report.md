# 第一轮全工程查复用键：脚本与驱动层（scripts/ 除 scripts/gate/）

日期 2026-10-15。被读的树：主工作区（HEAD c1e9b324；`git status --short -- scripts/ crates/` 只列出 scripts/kb/ 下 README.md、insert-row.py、replace-once.py 三份改动与新文件 lib_notation_check.py，四份都不涉及复用键）。
只读：没起 cargo、门禁、测试二进制、驱动与自证，没动任何判定库与标记，没写源码摘要缓存；只用 grep、sed、ls、du、git rev-parse 读。
每条结论标「量的（文件:行）」或「推的」。这里的「量的」指读到了算键或比键的那一行代码本身（或 ls / du 的原样输出），不是跑出来的观测。
三份审计报告与对账表只当线索：表里每一行的行号都是这一轮在现在的文件里重新 grep 出来的。

判据照派发提示：单元是函数；一条判定沿不沿用只许看它依赖的函数单元各自去注释的指纹、工具链、解析出的依赖版本。①–⑧ 照派发提示原义。

## 0 列全

命令（派发提示给的那一条，去掉 scripts/gate/）：

    grep -rln "sha256\|digest\|fingerprint\|judge.version\|JUDGE_RULES_VERSION\|changed.since\|reuse\|复用\|沿用" scripts/ --include=*.py --include=*.sh | grep -v '^scripts/gate/' | sort | wc -l
    163

另用一条宽的 grep（`grep -rlni "cache\|marker\|verdict.librar\|判定库\|标记\|library\|skip_if\|already.judged\|已判" scripts/ --include=*.py --include=*.sh`，去掉 scripts/gate/）补查，多出来的 68 份（`comm -13` 数的）逐个过了命中行：只有 scripts/crash/crash-amplification-lib.sh、crash-amplification-item.sh、
gpu-quiesce-run.sh、scripts/hosts/peer-host-lib.sh 四份与复用键有关，并进下面的主表；其余是 kb 文档标记、自证的 cache 字样、token 用量统计，不涉及判定复用。

163 份的去向（数用 comm 与 grep -c 数出来：主表直接列的 24 份 + e170 目录 58 份 + e171 目录其余 48 份 + 其余 33 份 = 163）：

| 去向 | 文件 | 理由 |
|---|---|---|
| 主表（第 1 节） | scripts/mutation/broad-mutation.py、mutation-prune.py、crates-mutation-rows.py、mutate.sh、mutation-shard-run.sh；scripts/crash/crash-amplification.sh、crash-amplification-two-hosts.sh、crash-point-replay.sh、crash-point-replay-shard-run.sh、lib_verdict_library.py；scripts/experiments/replay.sh、e165-pin-red-examples.py、e165-replay-mutation-kill-bounds.sh；e171_three_oracle_collusion_after_the_oracle_fixes/ 下 reuse_library.py、rows_orchestrator_revision_11.py、rows_orchestrator_segment_two_revision_11.py、segment_one_units_revision_11.py、count_reusable_units.py、segment_one_units_revision_8.py、rows_orchestrator_segment_two_revision_10.py、rows_orchestrator.py、row_driver.py、run_segment_one_on_the_revision_6_tree_multi_host.sh；e171_three_oracle_collusion/row_driver.py | 有判定库、续跑跳过或复用键 |
| 主表里作为同一机制的使用方点名 | e171_…_after_the_oracle_fixes/ 下 rows_orchestrator_segment_two.py、run_segment_two_on_the_revision_6_tree_multi_host.sh、segment_one_unit_revision_11.sh、run_segment_one_revision_11_multi_host.sh、run_segment_two_revision_11_multi_host.sh；scripts/hosts/multi-host-run.sh、run-with-memory-cap.sh | 它们本身不算键，但被别处的键整份收进去，或照别处的键续跑 |
| 不是复用键：e170 五个目录 58 份 | scripts/experiments/e170_*/ | sha256 只用于导出目录 manifest 核对、跑完核原目录逐文件 sha256 没变（还原核对，例 e170_o3_spec_executor/mutation_driver.py:47-55、148、191）、冻结行与照抄模块比对（…journal_header_487/run_known_bad_recopy_check.py:135-142）；没有一处按指纹跳过判定（量的：`grep -n "复用\|reuse\|沿用\|续跑"` 在这 58 份里只命中两份 witness_predicates.py 各一行注释） |
| 不是复用键：e171 两个目录其余 48 份 | e171_three_oracle_collusion/ 与 …_after_the_oracle_fixes/ 下的比对器、冻结 O3、坏实现表、占位核对、测试 | sha256 是产物与冻结树的证据（例 row_driver.py:235-245 的 snapshot_digest 只写进产物、不决定跑不跑）、按 sha256 钉住载入的模块（reuse_library.py PINNED）、占位里登记的 sha256 核对；不按它跳过判定 |
| 不是复用键：其余 33 份 | scripts/agents/（agent-handover、agent-watch、cite-check、three-way-materials）；scripts/crash/（check-segment-registry、cpu-judging-sites-check、lib_crash_point_replay_items）；scripts/devices/（build-patched-qemu、e152-run、e152-stage-root、snapshot-check、vm-bench、vm-geom）；scripts/experiments/（e152-tables、e165-compare-kill-bounds、e56-sweep）；scripts/hosts/（compile-then-swap、detached-run、host-dispatch、memory_peaks_lib、minimal-copy、source-stamp、target-prune、test-environment-check）；scripts/kb/（fetch-refs、history-consolidate、insert-row、lib_atomic_replace、migrate-changelog-format、stale-candidates、verify-citations） | 写前复核（读时与写前 sha256 比，insert-row.py:120 这一类）、下载包核对、交接摘要、进程号复用、编译目录与构建戳、开工快照核对、kb 文档；其中几处按指纹或 HEAD 判东西的列进第 3 节「射程外」 |

## 1 主表

每行一处。「量的」后面括的是算键或比键那一行；「推的」写明推的是哪一半。重做方向只写单元怎么切、键改成什么与落点文件。

### 1.1 广谱变异（驱动 scripts/mutation/broad-mutation.py，规则库 scripts/mutation/mutation-prune.py）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| B1 | broad-mutation.py:1421-1451（1437 `rev-parse --verify <提交>`、1442 `git archive --format=tar <提交>`）；文件头 11-14 | 这一版的清单与结果表建在哪棵树上 | 提交号（`--commit`，默认 HEAD）+ 导出范围 export_paths（968-986，取 crates 变异登记表的拷贝范围加 .cargo） | ① | 树的来源不按提交：直接取给定的工作树内容（主工作区或副本）现切函数单元；「一版」不再是复用的容器。落点 broad-mutation.py 的 sync_tree、export_paths。量的 |
| B2 | broad-mutation.py:988-999（manifest_of_tree 逐文件 file_digest）、1011-1015（fingerprint_of_manifest，1014 每行「sha256  路径」）、1056-1075（version_key）、1171-1175（versions/<指纹>/）；文件头 52-56 | 这一版的结果表 results.tsv（每条变异的结局按清单序号存）、list.json、baseline.tsv、coverage-*.json、escalation/、--import-results 只认同一版（23、2690） | 源码指纹 = 导出树全部文件逐个原文 sha256 连路径；再拼 rustc -Vv、`judge <JUDGE_RULES_VERSION>`、`global <全局键 sha256>` | ②③④⑤（树里任一文件改一个注释或改名就是另一版；结果按序号存） | 删掉「版本」这一层的判定意义：每条变异的结论只按 B5 那把逐条键存取；序号只作这一趟的显示号。落点 broad-mutation.py 的 version_key、Library、select_items、import_results。量的 |
| B3 | broad-mutation.py:2521-2596（select_items：states 只取这一版 results.tsv），3297-3303（只有 `if arguments.changed_since:` 才 plan_changed_since、write_carried_rows） | 真跑选哪些条 | 不带 --changed-since 时：这一版结果表里没有定论 ⇒ 现判；结论表 conclusions.tsv 一条不查 | ③⑥（换一版就整版现判，结论表形同没有） | 不带参数的真跑先对每条查结论表（B5 的逐条键 + carry_decision），沿用不了才现判。落点 select_items、run_items。量的 |
| B4 | broad-mutation.py:3114-3128（old_tree_index：3122 `git archive <changed_since>`）、3131-3142（plan_changed_since，3138 `if changed is None and carry:`）、3438-3446（--list-items 只读那一支同一判法）；mutation-prune.py:661-681（changed_since） | --changed-since 下哪些条现判 | 对着那个提交导出的旧树比：文件新加、文件函数外部分、宏/include 依赖、所在 fn 项任一样与提交里那一份不同 ⇒「改到了」⇒ 一律现判，哪怕结论表里有同一指纹可沿用的一行 | ① | 删掉 changed_since 整层与 old_tree_index；「改没改」只由 carry_decision 拿结论表里记的指纹与此刻指纹比。落点 broad-mutation.py 的 plan_changed_since、old_tree_index、main 里 --list-items --changed-since 一支，mutation-prune.py 的 changed_since。量的 |
| B5 | mutation-prune.py:594-623（mutation_key，620 键 `[path, name, relative, replacement]`，613 指纹 `function.fingerprint + " " + index.outside_fingerprint`）；broad-mutation.py:2952-2978（Pruner.key_of、record_conclusion）；结论表列 467 | 结论表 conclusions.tsv 里一条变异的结论（抓到）能不能跨版本沿用 | 键 = 文件仓内路径、容器限定名（mod 路径 + impl/trait 头 + fn 名）、函数内相对起止、替换文；沿用另比：容器 fn 项词法指纹 + 同一文件函数外部分的词法指纹、依赖指纹（B7）、全局键（B9）、抓到它的测试指纹（B8） | ③⑤（同一文件里加一个 use、改一个类型或常量，整文件的变异都换指纹；文件挪目录、fn 改名、外层 impl 头一变，键就换） | 键 = 容器 fn 项去注释的词法指纹 + 函数内相对位置 + 替换文；函数外部分只取这个 fn 真用到的项（类型、常量、static、use、impl 头）各自的指纹；路径与名字只进报告。落点 mutation-prune.py 的 mutation_key、FileIndex，broad-mutation.py 的 Pruner。量的 |
| B6 | mutation-prune.py:604（切不开 ⇒ WHOLE_FILE、整文件词法指纹）、608（变异不在任何 fn 项里 ⇒ OUTSIDE、整文件函数外部分指纹） | 同 B5 | 整个文件的词法指纹，或整个文件函数外部分的词法指纹 | ③ | 函数外的项（const、static、struct、enum、impl 头、宏定义）各切一个单元；切不开的文件当解析缺陷修 FileIndex，不退回整文件。落点 mutation-prune.py 的 FileIndex.scan、mutation_key。量的 |
| B7 | mutation-prune.py:509-532（dependency_fingerprint：519 宏定义文件的 `file_fingerprint`；523 include 参数不是字面串 ⇒ `tree_fingerprint()`；528 include 文件原文 sha256）、493-507（tree_fingerprint：manifest 或逐文件原文 sha256 连路径） | 同 B5（依赖指纹一样变了不沿用） | 按文件算、挂在文件上：这份文件调到的每个宏名在树里所有定义文件的整文件词法指纹 + 定义文件路径；每个 include 的路径与原文 sha256；include 参数认不出就是整棵树的原文指纹 | ②③⑤（定义宏的文件里改任何一处，调这个宏的每份文件全部变异作废；一处 include 认不出整树一变全作废） | 宏取 macro_rules! 定义项本身去注释的指纹，只挂在真调它的那个 fn 上；include! 进来的 .rs 照函数单元切、数据文件取内容、挂在读它的 fn 上；整树退路删。落点 mutation-prune.py 的 TreeIndex.dependency_fingerprint、FileIndex.find_includes。量的 |
| B8 | mutation-prune.py:700-726（test_fingerprints：715 键 `(二进制 id, 模块路径::fn 名)`；725 认不出 ⇒ `"package:" + 整包指纹`）、589-591（package_fingerprint：包里每份文件词法指纹连路径）、656（carry_decision 只比这一个指纹） | 「抓到」沿用时抓它的那条测试还算不算同一条 | 测试的二进制 id 与用例名（名字）→ 那条测试 fn 项自己的词法指纹；认不出 ⇒ 整包指纹 | ③⑤⑦（只比测试函数自己：它调到的辅助函数、被测路径上的函数改了照沿用，会漏判；认不出时整包一变全作废；测试改名就换键） | 比那条测试执行过的每个函数单元的指纹（版本内覆盖已经记执行过的容器，照 crates-mutation-rows.py 的 executed_function_problem 那一套）；名字只作索引。落点 mutation-prune.py 的 test_fingerprints、catching_test_fingerprint、carry_decision，broad-mutation.py 的 record_conclusion 存执行函数集合。量的（键）；「会漏判」是读代码推的，没有造出实例 |
| B9 | broad-mutation.py:2869-2886（global_key_lines：2877 `file <路径> <sha256>`、2879 `build <sha256> <名字>`、2882 `exclusions <sha256>`）、2889-2911；469；mutation-prune.py:644-645（全局键不同不沿用） | 结论表整表能不能沿用 | rules <JUDGE_RULES_VERSION>；toolchain（rustc -Vv）；导出树里每份 Cargo.toml、Cargo.lock、.cargo/ 下文件的路径与原文 sha256；构建环境每项（admission.py build_environment_lines 的名字 + 内容 sha256，名字里带配置文件路径）；nextest / cargo-mutants / llvm-profdata 版本；排除表原文 sha256；限时参数；升档四个常量 | ②③④⑤（任何一个 crate 的 Cargo.toml 改一行注释、排除表加一行、构建配置文件挪个位置，整张结论表一条不沿用） | 全局键只留工具链、工具版本与 Cargo.lock 解析出的依赖（这条变异所在包的依赖闭包里每个包的名字、版本、来源）；Cargo.toml 只取影响编译的段、去注释、挂到包；排除表只作用于它点名的那几条；构建环境只取影响编译的变量值，不带路径名。落点 broad-mutation.py 的 global_key_lines、global_key_text、exclusions_digest_of。量的 |
| B10 | broad-mutation.py:469 `JUDGE_RULES_VERSION = "prune-r3-e1-e6-fresh-build-mac"`（进全局键第一行与版本指纹）；470 PRE_FRESH_BUILD_JUDGE_RULES_VERSION | 判法版本 | 手写常量串 | ④⑦（判法改了忘抬就照沿用；抬了就整表作废） | 判法版本 = 驱动与 mutation-prune.py 里决定结局的那几个函数（process_item 判结局、escalate_step、carry_decision、select_tests、limits_table 等）各自去注释的指纹；手写串只留作人看的名字。落点 broad-mutation.py、mutation-prune.py。量的（常量）；忘抬会漏判是推的 |
| B11 | mutation-prune.py:646-649（unviable ⇒ unviable-rejudged、missed ⇒ missed-rejudged）；CARRIED_OUTCOMES 637 只有 caught | 无效、没红两种结论 | 不沿用：每次现判 | ⑥（这两种结论不入可沿用的库） | 无效按「容器 fn 单元 + 依赖 + 工具链」沿用；没红按「覆盖到它的测试集合与各自执行过的函数指纹都没变、没有新测试覆盖到它」沿用。落点 mutation-prune.py 的 carry_decision。量的（规则）；没红沿用要防「新加的测试能杀它」，所以键里要有覆盖到它的测试集合，这一半是推的设计 |
| B12 | broad-mutation.py:2412-2431（2427 注释「升档判的抓到不进结论表、不跨版本沿用」）、文件头 154 | 升档判出的抓到（caught-by-timeout / caught-by-memory）、超时、内存撞顶 | 只记在这一版结果表与 escalated-verdicts.tsv（按版本指纹认，151-153）；不进结论表 | ③⑥ | 进结论表，键同 B5，另加升档的限时 / 上限档与基线的函数单元键；证据日志照存。落点 process_item、record_conclusion、escalation 登记。量的 |
| B13 | broad-mutation.py:1992-2010（ensure_coverage：1996 `version_file(f"{name}.json")` 在就跳过）；文件头 169-170 | 版本内按覆盖挑测试的覆盖记录 | 这一版的目录（B2 的整树指纹） | ③（每换一版整套测试插桩重跑一遍覆盖） | 覆盖按测试单元存：一条测试的覆盖键 = 它执行过的函数单元各自指纹 + 工具链；那些函数都没变就沿用。落点 ensure_coverage、coverage_step、Pruner.load_coverage。量的 |
| B14 | broad-mutation.py 文件头 108-113、ensure_baseline 2254 起；baseline.tsv 在版本目录（52-53） | 基线（定这一版各步超时、证没改过的树是绿的） | 版本目录 + 步名 + 线程数 | ③（不是判定本身，但每换一版重量一遍基线才开跑） | 基线按「这一步要跑的测试集合与它们执行过的函数单元」取键；没变就沿用秒数与峰值。落点 ensure_baseline。量的（存放位置）；代价没量 |
| B15 | broad-mutation.py:3406-3415（notice_if_head_moved：3412 `git diff --quiet <commit> HEAD`） | 不判，只提示树落后于 HEAD | 提交号对 HEAD | ①（只提示） | 删；提示改成「工作树里哪些函数单元与库里记的指纹不同」。量的 |

### 1.2 crates 变异（scripts/mutation/crates-mutation-rows.py，门禁 checker-tier-crates-mutation-replay 的判法；双机 scripts/mutation/mutation-shard-run.sh）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| C1 | crates-mutation-rows.py:601-619（609-615 named_lines 全用 read_bytes，618 `manifest_of_files(root, [], named_lines)`）；目录 2201-2205 `singlefs-mutation-rows/<判法版本>/` | 判法版本（定整格记录目录：4100 条的全部判定、覆盖、快照） | 登记行文本（stage-inputs.tsv 里本道那几行，带登记的路径）、阶段脚本 .claude/gate.d/checker-tier-crates-mutation-replay.sh、crates-mutation-rows.py、mutation-prune.py、scripts/gate/lib_libtest_result_lines.py、scripts/crash/lib_verdict_library.py、这一趟用的内存包装 scripts/hosts/run-with-memory-cap.sh，全部整份原文字节；再加构建环境各项 | ②③⑤（这几份里任一份改一句注释、改一个与判定无关的调度或退出码，整格目录换掉；mutation-prune.py 是广谱变异共用的，给广谱变异改切法同样作废这里；登记行改一个路径同样作废） | 判法版本只取决定「这一行抓没抓到」的函数单元（verdict_of_run、results_by_test、not_run_reason、wrapper_outcome、reuse_decision 与它调的比较函数、FileIndex 切法）各自去注释的指纹；内存包装、存储与派活、登记行不进。落点 crates-mutation-rows.py 的 judge_version。量的 |
| C2 | crates-mutation-rows.py:284 `BASE_DIRECTORIES_KEPT = 3`、711-726（prune_base_directories，721 删第 3 个以后的）；文件头 84 | 判法版本目录的去留 | 按目录修改时刻只留最新 3 个 | ⑧（来回改过 3 次，旧目录连同全部记录删掉；磁盘上现在 3 个目录，`ls .git/singlefs-mutation-rows/` 量的） | 记录不按判法版本分目录，按 C3 的逐条键存；删的判据改成「多久没被任何一趟引用」。落点 prune_base_directories、run_stage 里 records_directory 的取法。量的 |
| C3 | crates-mutation-rows.py:640-642（row_key：641 `"\t".join(row.fields) + memory_max + timeout_seconds + toolchain`）；六段名 336 | 一条变异的判定记录 mutation.<行键> 与基线记录 baseline.<行键> | 这一行六段原样（变异名、文件路径、原文、替换文、cargo test 参数、必须红的测试名）+ 这一趟每条内存上限 + 每条限时（都是全表一个值，文件头 187 自己写「一变全表判定记录作废」）+ cargo -V / rustc -V | ③⑤（改变异名、挪文件、改测试名都换键现跑；上限或限时调一次全表作废） | 行键 = 原文、替换文、cargo 参数里决定编译与选测的部分；被变异的 fn 单元交给复用判据比；名字、路径只作索引；上限与限时只进「超时 / 撞顶 / 没红」三档的记录，抓到的不看。落点 row_key、reuse_decision、import 的核对。量的 |
| C4 | crates-mutation-rows.py:999-1030（mutated_function_problem：1012 非编进测试二进制的 .rs 按整份 sha256；1015 切不开按整份；1024 原文在函数外 ⇒ 比整份文件的函数外部分） | 「被变异的函数变了」那一条复用判据 | 被变异文件在覆盖那份快照与此刻的原文 sha256；切得开时比包住原文的最外层 fn 项词法指纹，原文在 fn 外就比整文件函数外部分 | ②③ | 非 .rs（litmus 这类）按它自己的单元（一条 litmus 一项）取内容；函数外的变异按项单元比；切不开的修切法。落点 mutated_function_problem 与 mutation-prune.py 的 FileIndex。量的 |
| C5 | crates-mutation-rows.py:1033-1055（executed_function_problem：1046 `<整个文件>` 或切不开 ⇒ 整份比；1048-1050 `<函数外>` ⇒ 整文件函数外部分） | 「执行过的函数变了」那一条 | 覆盖记录里执行过的每个「文件 + 限定名」；落在 fn 外的记 <函数外>、切不开记 <整个文件> | ③⑤（执行过的条目以路径 + 限定名为键，改名即算变了） | 执行过的单元按函数指纹记（不靠路径名对齐），函数外按项单元记。落点 executed_function_problem、covered_containers（mutation-prune.py:748）。量的 |
| C6 | crates-mutation-rows.py:942-970（outside_problem：949 函数外词法变 ⇒ 现跑；958 新加任何非测试函数 ⇒ 现跑；属性、const fn、函数体里有 impl ⇒ 现跑）、972-997（path_effect：编进去的非 .rs 一律 outside；985 包目录里非 .rs 一律 outside；981 不在任何包目录的一律 outside）、1058-1071 | 「函数外有改动」那一条 | 覆盖快照与此刻之间每个变了的文件：编进去的 .rs 看整文件函数外词法与「有没有新函数」；编进去的数据文件、涉及包目录里的 Cargo.toml 与测试数据、根 Cargo.toml、Cargo.lock、.cargo/、litmus/ 按文件原文变没变 | ②③（同一个编进测试二进制的文件里加一个从没被调的函数、改一个与用例无关的类型，这一组合的每一行都现跑；任何 Cargo.toml 注释、Cargo.lock 里与这包无关的一项变了也一样） | 文件拆项单元，只看执行过的函数单元真引用到的项；Cargo.toml / Cargo.lock 换成解析出的依赖版本（这一测试二进制的依赖闭包）；数据文件挂在读它的函数单元上。落点 outside_problem、path_effect、outside_change_problem。量的 |
| C7 | crates-mutation-rows.py:2229-2236（基线 ok 记录要 `baseline.get("snapshot") == current["id"]`）、729-753（snapshot_of / snapshot_identity：拷贝范围里 scripts/ 外每个文件原文 sha256 连路径） | 基线 ok 能不能复用（不复用就要先跑基线批） | 行键（C3）+ 整份快照号 | ②③（树里任一文件改一个字节，基线全部重跑） | 基线按「这一批点名的测试执行过的函数单元」比（覆盖记录已有）。落点 run_stage 里基线复用那一段。量的 |
| C8 | crates-mutation-rows.py:649-653（combo_key：652 `--` 前参数、`--` 后开关、点名用例全名、工具链）、656-657（coverage_id = sha256(组合键 + 快照号)） | 覆盖记录（复用判据的输入） | 组合键带用例全名；覆盖号带整份快照号 | ③⑤（测试改名覆盖作废；覆盖只在「这一组合在这份快照上还没有」时记，快照是整树） | 覆盖按测试的函数单元指纹存（测试 fn 本身 + 它执行过的单元），名字只作索引。落点 combo_key、coverage_id、CoveragePlanner。量的（键）；整树快照对覆盖复用的实际影响没量 |
| C9 | mutation-shard-run.sh:31-35（第二台 GATE_MUTATION_RECORDS_HOME=<第二台>/mutation-kept/records）、383-416（拷回、import）；crates-mutation-rows.py:2793-2850（command_import 核判法版本、行键、复用判据） | 第二台判过的行 | 第二台自己一份判定库（同样按 C1 的判法版本分目录、只留 3 个），拷回经 import 逐条核再写进本机库 | ⑧（两份库各存一份同样的判定；第二台那份随判法版本换目录一样丢） | 一份库：第二台判完的记录当场导入本机库为准，第二台不留长期库或只留一份按逐条键的副本。落点 mutation-shard-run.sh ⑥⑦ 两步、crates-mutation-rows.py 的 records_home。量的 |
| C10 | crates-mutation-rows.py:691-697（reuse_refusal：SINGLEFS_GATE_FULL=1、GATE_MUTATION_START_OVER=1） | 整表一律现跑 | 环境变量 | ⑥（设计如此的全量开关） | 判不了：要不要留全量开关见第 2 节。量的 |

### 1.3 实验变异表（scripts/mutation/mutate.sh）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| M1 | mutate.sh 文件头 1-9（admission: always）；整份没有判定库（`grep -n "sha256" mutate.sh` 只命中 1163、1277、1662 三处，都是源文件改前改后的还原核对） | 一张实验变异表每条「抓到 / 没红 / 无效 / 超时」（平常跑），与 --compile-only 每条「编得过 / 编不过」 | 无：每次调整张表基线 + 每条变异 + 还原之后那一次全部现跑 | ⑥ | 按条入库：一条变异的键 = 原文、替换文、被改源文件里包住原文的 fn 单元指纹与它依赖的单元、cargo 参数与 features、工具链与解析出的依赖版本；判定库用 lib_verdict_library.py（先补上 L1 说的函数单元摘要）。落点 mutate.sh；门禁 mutation-tables-compile 格那一层按表的库归 gates 路。量的 |

### 1.4 崩溃点重放与崩溃放量的驱动层（scripts/crash/；块键、结论版本本身在 crates/singlefs-checker-tier，归 crates 路，这里只列驱动层的）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| K1 | crash-amplification-item.sh:762-765（录入段 `crash_amplification_record_on_this_host` 无条件先跑）；crash-amplification-lib.sh:360-380；crash-amplification.sh:2（admission: always） | 放量每一项的录入段（CPU 跑流、排计划、把判表写进库） | 无：每趟每项都跑；没有分项级的键 | ⑥ | 录入段按块键（这条流的写表与父镜像摘要，crates 侧已有）先查库，库里已有这条流的录入就跳过；落点 crash-amplification-item.sh、crash-amplification-lib.sh，crates 侧 crash_library.rs 的两段。量的（无条件调用）；录入段每趟多花多久没量 |
| K2 | crash-amplification-lib.sh:106-120（113 `CRASH_AMPLIFICATION_LIBRARY_SUBDIRECTORY="library-by-conclusion-kind"`，108-109 旧库 `<家目录>/library` 引擎开库就拒、不删不读） | 放量的长期判定库在哪 | 库目录名 = 存法名 | ⑧（存法一改就换目录、旧判定全不读；家目录里现在 `library` 272M、`layer0-enumeration-library` 624M 留着没人读，`du -sh` 量的） | 库目录名固定、不带存法名与类名；存法升级时库内迁移或旧记录按逐条键直接认。落点 crash-amplification-lib.sh 与 crates 侧 crash_library.rs 的开库。量的 |
| K3 | crash-point-replay.sh:7、27-30、72-73、210；crash-point-replay-shard-run.sh:85（`crash-state-enumeration-library`） | 同一批崩溃状态的判定存在哪 | 按库类分三份：崩溃枚举用例 crash-state-enumeration-library、放量 library-by-conclusion-kind、给定状态 given-states-library | ⑧（同一状态在两份库里各判一次；库名随类名、术语改名换过三次：家目录里 layer0-enumeration-library、library、crash-state-enumeration-library 同时在，ls 量的） | 一份库，键只由块内容（写表、父镜像、判器函数单元）组成，不分「哪个入口判的」。落点 crash-point-replay.sh、crash-point-replay-shard-run.sh:85，crates 侧 crash_library.rs 的 CrashLibraryClass。量的（路径）；「同一批状态两库各判」的条数没量，取自路子三日志（线索） |
| K4 | crash-point-replay.sh:63-64、517-518（快档那几趟 `-u SINGLEFS_CRASH_AMPLIFICATION_*`）、212-216（BOTH_TIERS_GROUPS：GPU 与 CPU 小输入对拍、剪枝自证、钉回的红例） | 快档那一段（每种跑法都跑）与两档都跑的三类 | 无外层键：每趟把合表点名的快档测试目标与这三类整个 cargo test 一遍；只有经给定状态引擎的那几个目标落到 crates 侧的库 | ⑥ | 每个测试目标（或每条用例）按它执行过的函数单元记绿（同 crates 变异的覆盖做法），对拍类按两个判器的函数单元指纹记；经引擎的照库。落点 crash-point-replay.sh 的快档那一段与 BOTH_TIERS_GROUPS。量的 |
| K5 | crash-point-replay.sh:14-17、563-576（--full：调 admission.py crash-case-manifest 算指纹、删那一格全绿标记，带 --start-over）、626-636（放量部分每趟新建 `crash-point-replay-full/<时戳>-<进程号>/library`，跑完不删） | 全量（清空复用） | 时戳与进程号 | ⑥⑧（设计如此的全量；每趟建一份之后谁都不读的库） | 判不了要不要留（第 2 节）；留的话至少不建没人读的库，全量只「只记不读」写进那一份长期库。落点 run_enumeration_part、run_pipeline_part。量的 |
| K6 | crash-point-replay.sh:571-573 → scripts/gate/admission.py crash-case-manifest / crash-case-marker-path（gates 路） | 崩溃枚举用例的全绿标记 git common-dir `singlefs-crash-case-green.<用例名>.<指纹>` | 用例名进文件名；指纹由 admission.py 算（gates 路查组成） | ⑤⑧（用例改名旧标记全作废：`ls .git/singlefs-crash-case-green.*` 里还留着 layer0-first-stream、layer0-second-stream 等改名前的名字，量的） | 标记改成按用例函数单元闭包的指纹取，名字不进；归 gates 路改 admission.py。量的（文件名） |
| K7 | scripts/gate/admission.py:326-327 `SHARD_DRIVER_FILES`（gates 路）收 crash-point-replay-shard-run.sh、crash-amplification-two-hosts.sh、crash-amplification-lib.sh、crash-point-replay-shard-configuration-check.sh 整份；连带 scripts/hosts/peer-host-lib.sh:5、scripts/crash/gpu-quiesce-run.sh:94、scripts/hosts/multi-host-run.sh:41 的 copy-kept / gate-similar 注释 | 登记了 shard=across-machines 的三条崩溃枚举用例的全绿标记 | 四份驱动脚本的原文进这三条用例的输入指纹 | ②③（驱动脚本改一句注释或改一处与判定无关的调度，三条用例的标记全作废；为了躲它，清场与第二台那几段在 gpu-quiesce-run.sh、peer-host-lib.sh 里各抄了一份） | 驱动脚本不进判定键（它们只派活，判定在用例与判器里）；把三份 copy-kept 并回一份。落点 admission.py（gates 路）与这三份脚本的重复段。量的（四份清单与三处注释原文） |

### 1.5 Python 一侧判定库共用部分（scripts/crash/lib_verdict_library.py）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| L1 | lib_verdict_library.py:52-60（digest_file 原文 sha256）、63-76（digest_tree：目录下每个文件「sha256  相对路径」）、79-86（input_fingerprint、key_of）、142-153（lookup：`keys/<键>.json` 在、verdict=green、key 相同） | 形式化两份判法、e165 钉表重放、crates 变异的写读部分、门禁 mutation-tables-compile 格都经它 | 键 = sha256(输入指纹 NUL 判法版本)；输入指纹 = 各 component「名字 NUL 摘要」；共用库只给原文摘要与目录摘要两种取法 | ②③⑤（库本身不错，但它只提供文件原文与整目录（带路径）两种摘要，用它的几家因此都按文件 / 目录取键） | 加函数单元摘要（去注释的 fn 项指纹，切法与 mutation-prune.py 的 FileIndex 共用一份；Rust 文件以外的按各自单元）与「解析出的依赖版本」两种取法；digest_tree 不再给判定键用。落点 lib_verdict_library.py。量的 |

### 1.6 形式化钉表重放（scripts/experiments/e165-pin-red-examples.py --replay）与 E165 第二段续跑

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| E1 | e165-pin-red-examples.py:1162-1168（judge_version = digest_file(本脚本) + digest_file(lib_verdict_library.py)） | 钉表 360 行每一行的判定（判定库 kind=pinned-red-examples-replay） | 两份脚本整份原文 | ②③（改一句注释、改 --list-items 的打印，360 行全部重放） | 判法版本取判一行的那几个函数单元（replay 里比逐态行、判装置报错的函数）去注释的指纹。落点 judge_version。量的 |
| E2 | e165-pin-red-examples.py:1171-1187（device_inputs_digest：1180 `admission.cargo_closure_input_paths(...) + ["Cargo.toml", "Cargo.lock"]` 交 `input_manifest`，逐文件原文）、1308（每行共用这一样「装置」） | 同上 | 装置 e165_replay_bounded_enumeration 的 cargo 依赖闭包（按 crate 目录收的每份文件）原文 + 仓根 Cargo.toml、Cargo.lock 原文 + cargo -V、rustc -V；给了 --binary 另加可执行文件内容 | ②③（闭包里任一 crate 任一文件改一句注释，360 行全部重放） | 每一行的键 = 装置从入口到判这一态走到的函数单元（装置 bin 的调用闭包，函数级、去注释）+ 这一行的状态输入 + 引的变异；Cargo.lock 换解析出的依赖版本。落点 device_inputs_digest、row_components，闭包算法在 admission.py（gates 路）。量的 |
| E3 | e165-pin-red-examples.py:1204-1214（row_components：「这一行」22 段原样、「档」、「引的变异行」= 变异表里那一行原样） | 同上 | 22 段里含红例账路径:行号（例 `e165-…-segment2-red-examples.tsv:2`）与变异名（`crates/mutations.tsv#步 3：…`）；引的变异行按名字取整行 | ⑤（红例账改名、挪行，变异改名，这一行就换键） | 只取决定判定的段：状态输入（域、规模、序号）、期望逐态行、变异的原文与替换文；路径与名字只作报告。落点 row_components。量的（段的内容取自 `.claude/gate.d/e165-pinned-red-examples.tsv` 第一行数据行） |
| E4 | e165-replay-mutation-kill-bounds.sh:1815-1829（recorded_items：结果文件里 `name=kill_bound item=<变体名> … complete=yes` 的行就算判过）；调用 1984、2116、2157、2264、2312、2420 | E165 第二段每个变体的杀伤界（续跑时哪些变体不再跑） | 只有变体名（item=），在同一份结果文件里 | ⑤⑦（键只有名字：结果文件留着而代码或变异行改了，续跑照跳过） | 续跑记录带这一变体的内容键（变异行原文与替换文、被改函数单元指纹、装置函数单元闭包、界与档），对不上重跑。落点 recorded_items 与写行的 main_run。量的 |

### 1.7 实验复跑（scripts/experiments/replay.sh；键的算法在 scripts/gate/admission.py，归 gates 路，这里列 replay.sh 一侧与键的组成）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| R1 | replay.sh:33-38、2401-2420（`admission.py replay-row-inputs` 给每行一把键）、2457-2480（2474 键目录里 verdict 与 product 都在、不是 --full ⇒ 复用）；组成见 admission.py:67-80 | 复跑表每一行的判定（字节一致 / 仅计时不同 / 已归档只跑不比） | 装置源码：[[bin]] 那份 path 顺 mod / #[path] / include! 带进来的整份文件与 include_str! 点名的文件；@驱动函数行取 replay.sh 里那个函数与它调到的顶格函数原文（只去整行注释与行尾空白）；`-p <包> --bin` 取 cargo 依赖闭包，`-p <包>` 不带 --bin 取依赖闭包里每个 crate 目录整个；字面写出的 scripts/experiments/<子目录>/ 取整个子目录；公共部分 scripts/experiments/Cargo.toml、Cargo.lock、lib.rs 与它带进来的；复跑表这一行五列原样；REPLAY_JUDGING_FUNCTIONS 那几个函数原文；留存产物 sha256；工具链、构建环境。逐文件取源码摘要（admission.py:67-68） | ③（文件级、包目录级、子目录级；lib.rs 改了每一行都重跑）②（驱动函数只去整行注释，行尾注释进键）⑤（复跑表五列里的实验号、产物名进键） | 每一行的键 = 装置入口（或驱动函数）出发的调用闭包里每个函数单元的去注释指纹 + 这一行参数 + 留存产物内容 + 工具链与解析出的依赖版本；lib.rs 拆项单元；产物名只作索引。落点 admission.py 的 replay-row-inputs（gates 路）与 replay.sh 的 REPLAY_JUDGING_FUNCTIONS 取法。量的 |
| R2 | replay.sh:55（「按行复用的键不含这个开关：GPU 开关变了，要拿到带 GPU 判段的结论得用 --full」） | 同上 | ENABLE_GPU 与 CHECKER_TIER_FEATURES 不进键 | ⑦（开关换了照复用不带 GPU 判段的结论） | GPU 开关与特性集进行键。落点 replay.sh 与 admission.py replay-row-inputs。量的（文件头自己写着） |
| R3 | replay.sh:2438-2439（resolve_checker_tier_features 试编 gpu 特性并清场、build_research_binaries 整编）在 2474 判复用之前 | 不是键：全部行都能复用的一趟照样编、照样清场 | — | ⑥（剪枝省不到这一段） | 先算全部行的键，全复用就不编不清场；只给要现跑的行编。落点 replay.sh 主流程次序。量的（次序）；代价没量 |
| R4 | replay.sh:27-28、2332-2395（准入：admission.py 判「输入没变」退 77 ⇒ 记「输入没变」、沿用留存产物结论）；2172-2190（产物头 `E7INPUT name=crates_snapshot` 是开工快照汇总 sha256） | 登记了准入的实验行 | 准入键（实验号或 ADMISSION_ROW_KEYS 指明的键）→ stage-inputs.tsv 登记的路径下文件的摘要，写进产物头 E7INPUT 与留存产物比 | ③⑤（登记的是路径；键名是实验号） | 并进 R1 的逐行函数单元键，删按登记路径的准入指纹这一路。落点 replay.sh 准入那一段、admission.py（gates 路）。量的（replay.sh 一侧）；登记路径的粒度引自 admission.py 文件头，没逐个核 |
| R5 | replay.sh:32、2288-2293、2421（--full 或 SINGLEFS_GATE_FULL=1：只记不读） | 全量 | 环境变量 / 参数 | ⑥（设计如此） | 判不了，见第 2 节。量的 |

### 1.8 E171 实验装置的复用库与续跑（scripts/experiments/e171_three_oracle_collusion*/）

| 编号 | 位置 | 管哪种判定 | 键由什么组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| X1 | reuse_library.py:89（KEY_SCHEMA_LINE）、207-219（build_lines）、172-178（closure_lines）、234-243（research_lines）、499-513（reading_unit_sections：512 变异驱动单元的 research 取整个研究目录）、516-523、535 与 546（mutation_lines 带 `row {row.number}`）；使用方 segment_one_units_revision_11.py、rows_orchestrator_revision_11.py、rows_orchestrator_segment_two_revision_11.py、segment_one_unit_revision_11.sh、count_reusable_units.py | E171 修订 11 的每个读数单元与驱动行（装置、E170 导出、O3 执行器、变异驱动、逐行驱动） | 键文本 = 手写 schema 行 + 十节：[kind]；[build] 工具链 sha256、根与每个 crate 的 Cargo.toml、Cargo.lock、.cargo/ 每个文件「路径 原文 sha256」、构建环境；[closure] dep-info 列的每个源文件「路径 原文 sha256」（驱动行用改坏后的内容）；[env] 闭包里读的变量设没设；[mutation] 行号、选中行原文、变异表那一行原文；[args]；[limit]；[upstream] 上游读数每个文件 sha256；[research] 研究一侧文件「路径 原文 sha256」，变异驱动取整个目录；[python] | ②③④⑤（dep-info 是文件级：被编进去的任一文件改一句注释，这个装置的全部单元换键；行号进键；schema 是手写串） | [closure] 改成装置入口出发的函数单元闭包、去注释；[build] 换 Cargo.lock 解析出的依赖版本与去注释的清单段；[mutation] 去掉行号，只留原文与替换文；[research] 按 Python 函数单元；schema 行换成键算法本身那几个函数的指纹。落点 reuse_library.py。量的 |
| X2 | reuse_library.py:88 `DEFAULT_LIBRARY_ROOT = ~/.local/share/singlefs/reuse/e171-r1` | 同上 | 库目录名带轮名 | ⑧（下一次重跑另起一轮名就是另一份空库；`ls ~/.local/share/singlefs/reuse/` 现在只有 e171-r1） | 库目录名不带轮名、按键共用。落点 DEFAULT_LIBRARY_ROOT。量的 |
| X3 | segment_one_units_revision_8.py:211-300（续跑：序列日志有「✓ 单元 <名> 绿」行 + 产物在且行数同 + resume-manifest.tsv 里每个文件原文 sha256 不变） | 修订 8 第 0–10 步的单元续跑（同一趟被停后再起） | 单元名、第几次、产物路径与行数、清单里文件原文 sha256 | ②③⑤（按单元名认；清单里任一文件变就整单元重跑） | 续跑改用 X1 那一套按内容的单元键（修订 11 已在这份上加了一层）；旧修订的装置是冻结的，改不改见第 2 节。量的 |
| X4 | rows_orchestrator_segment_two_revision_10.py:329-350（record_or_check_inputs：inputs.sha256 逐行核，任一文件变了整段停） | 修订 10 第二段续跑 | 输入清单每个文件原文 sha256 + run-values | ②③ | 同 X3。量的 |
| X5 | row_driver.py（…_after_the_oracle_fixes/）:500-507、e171_three_oracle_collusion/row_driver.py:494（--skip-finished：行目录里有 row-result.tsv 就读回、不重跑）；rows_orchestrator.py 文件头 44、55 与 1067 与 rows_orchestrator_segment_two.py:13、53、812（先把以前各趟取回目录里有 row-result.tsv 的行收进来） | 逐行驱动的一行 | 行号（目录名 row-<n>）与 row-result.tsv 在不在 | ⑤⑦（键只有行号：树或变异行改了，旧行照收） | 行目录里记这一行的内容键（X1 的驱动行十节），收旧行之前比键。落点 row_driver.py、rows_orchestrator.py、rows_orchestrator_segment_two.py。量的 |
| X6 | run_segment_one_on_the_revision_6_tree_multi_host.sh:6、14-17、42、138-144（续跑前 resume_check.py check 核前几步记了绿、产物行数、清单 sha256） | 修订 6 第 0–10 步续跑 | 由草稿里的 resume_check.py 定（42 `RESUME_CHECK=$SCRATCH/resume_check.py`，37 行说它没入库） | 判不了（判法不在仓里） | 见第 2 节。量的（路径） |

## 1.9 与对账表 28 条的对照（只列落在 scripts/ 这一路的）

| 对账表号 | 这里的行 | 说明 |
|---|---|---|
| 19 | B4 | 现查：broad-mutation.py:3138 与 mutation-prune.py:661-681。另找到同一判法的只读一支 3438-3446 |
| 20 | B3 | 反推做过：`grep -n "pruner.carry("` 只命中 3137（changed_since 那一支）、3314（里程碑全量只记快照、不落沿用行）、3444（只读那一支）；沿用行只由 3304 的 write_carried_rows 落，它在 `if arguments.changed_since:` 里，没找到别的沿用路径 |
| 21 | B9、B10 | 另补：全局键的 build 行名字带配置文件路径（2879） |
| 22 | B8 | 另补：认不出的测试退整包指纹（725） |
| 15、16 | C1、C2、C3 | 另补：mutation-prune.py 进 crates 变异判法版本，广谱变异改切法连带作废这里 |
| 18 | E1、E2 | verus-proof-check.py 在 scripts/gate/，归 gates 路 |
| 25、27 | R1、R2、R3 | 研究复跑门禁那一层（标记、mutation-tables-compile 库在 /tmp）归 gates 路 |
| 10、11、13 | K1、K2、K3、K4 | 驱动层那一半；块键与结论版本归 crates 路 |
| 12 | K6 | 用例级标记的键在 admission.py，归 gates 路；这里补了「用例名进标记文件名、改名后旧标记残留」 |
| 17 | — | 第二台缺 llvm-tools 不是键的问题，这一路没核 |

对账表里没有、这一路新找到的：B1、B2、B5、B6、B7、B11、B12、B13、B14、B15、C4、C5、C6、C7、C8、C9、M1、K5、K7、L1、E3、E4、X1–X6。

## 2 没查到 / 判不了

- 全量开关要不要留（C10、K5、R5，以及 broad-mutation.py 的 --milestone-full）：派发判据说「剪枝就是单元的」，没说全量开关要不要删。它们都是「明说不复用」的开关，不是键错；留不留归主 agent 问用户。
- B11 里「没红」能不能沿用：照判据，没红的结论依赖的是「覆盖到它的那些测试」，键里要有测试集合；这是设计判断（今天的规则是故意每次重判，好让新测试有机会杀它），这里只写方向，没判。
- X3、X4、X6（E171 旧修订的续跑）：那几份是已冻结的旧修订装置（修订 6、8、10），改它们要不要连同重跑，判不了；X6 的判法 resume_check.py 在草稿里、仓里没有（run_segment_one_on_the_revision_6_tree_multi_host.sh:37、42），没读到。
- K3 里「同一批状态在两份库里各判一次」的条数、K1 录入段每趟多花多久、R3 全复用的一趟编译与清场多久：都没量（只读，起不了）。
- 崩溃库的块键、计划哈希、结论版本、判器闭包（crates/singlefs-checker-tier 下）：归 crates 路，这里没读。驱动层能看到的一处牵连：crash-amplification.sh 的 FULL_EXPAND_UP_TO 与 crash-point-replay.sh 的 LIGHT_EXPAND_UP_TO=6 不同，展开深度进不进计划哈希、轻型判过的块重型能不能照键复用（crash-point-replay.sh:45 只对分片段说了「轻型判过的块重型照键复用」），驱动层判不了。
- 家目录 `~/.local/share/singlefs/crash-amplification/` 下另有 amplification-unit-tests-library（918M）、facts-pairing-library、judge-tables-pairing-library、given-states-library-of-harness-scenario-images-judged-afresh（1.8G）、given-states-library.polluted-by-gpu-fault-tests-2026-10-11、mutation-replay（981M）（`du -sh` 量的）：这些库名由 crates 侧的库类定，谁建、谁读、是不是同一批状态重复判，归 crates 路；这里只记它们在。
- broad-mutation 的结论表现在只有 3 行（表头加 2 行，`wc -l` 量的），versions/ 下 2 版：结论表实际上没被用起来，B3 的后果在真库上的规模没法从库里看。
- admission.py 的 crash-case 指纹、整道键、replay-row-inputs 的实现：在 scripts/gate/，按派发提示不在这一路；凡是牵到的（K6、K7、R1、R4、E2 的闭包算法）只写落点，组成取自 admission.py 文件头与常量行，没逐行核算法。

## 3 射程外（撞见的，不在改的射程里）

| 位置 | 是什么 | 为什么射程外 |
|---|---|---|
| scripts/crash/crash-point-replay.sh 的 --quick 那一支（文件头 27-46） | 轻型崩溃：用放量长期库照键复用 | 轻型门禁 |
| scripts/mutation/crates-mutation-light-rows.py | 轻型 crates 变异的判法 | 轻型门禁；`grep` 里没有判定库与复用键 |
| scripts/hosts/host-dispatch.py:23-26、570、1336-1356；scripts/hosts/detached-run.sh:56-57、241 | 长活豁免表 `.claude/long-run-live-hosts-exempt` 按脚本整份 sha256 认 | 不是判定复用，是豁免登记；整份 sha256 的写法与 ② 同形，撞见记一笔 |
| scripts/devices/snapshot-check.sh:25-42（41 `git show HEAD:<路径>` 比快照） | 三方轮开工快照核对，对不上再比 HEAD | 不是剪枝；按 HEAD 比与 ① 同形，撞见记一笔 |
| scripts/devices/build-patched-qemu.sh:16、32 | 真编 QEMU 的私有目录戳（版本、.dsc 与补丁 sha256、configure 参数），戳对得上不重编 | 构建缓存，不是判定 |
| scripts/hosts/source-stamp.py、minimal-copy.py、compile-then-swap.py、target-prune.py | 编译目录复用前 touch、拷副本、草稿编完换上、清旧产物 | 构建缓存 |
| scripts/kb/verify-citations.sh:154 | 文献抽出文本跨轮缓存（GATE_CROSS_RUN_TMPDIR） | 文档门禁的缓存 |
| scripts/agents/three-way-materials.py:24-25、236、307 | 三方轮材料 --resume 沿用清单、kb-sha256.txt 快照 | 材料快照，不是判定 |
| scripts/hosts/test-environment-check.py:91-99 | 有意跨轮复用的缓存目录白名单 | 环境检查 |
| scripts/experiments/replay.sh:1540-1575（e170_r2_shared_exports 的 `.done` 完成标记） | 同一趟里两组导出各造一次、后来的行直接用 | 只在这一趟的 OUT_DIR 里，跨趟不留（文件头 12 行「跑前删本轮输出」） |
| preflight 的 inputs-changed | `grep -rh "^# admission:" scripts` 只有 `always` 一种（103 份） | scripts/ 这一路没有一份用 inputs-changed |

## 4 推翻条件（什么现象会推翻表里的结论；只读，一条都没造）

| 行 | 造出来就推翻的观测 |
|---|---|
| B3 | 在一个结论表里已有某条变异此刻指纹的「抓到」行的库上，不带 --changed-since 真跑新的一版：那一条报 carried 而不是现判 |
| B4 | 同一库上带 `--changed-since HEAD --list-items`：自 HEAD 改过、而结论表里已有它此刻指纹的那条，打 `carry` 而不是 `judge:function-changed` |
| B5、B6 | 只把被变异文件挪个目录（内容不变），或在同一文件里加一个与该函数无关的 use：这一文件的变异照沿用 |
| B7 | 只改定义宏的那份文件里与宏无关的一个函数：调这个宏的文件里的变异照沿用 |
| B8 | 只改抓到它的那条测试调的辅助函数（测试函数本身不动）：carry_decision 判成 catching-test-changed |
| B9、B10 | 只改某个 crate 的 Cargo.toml 一行注释：`global_key_lines` 打的全文不变 |
| C1 | 只改 scripts/hosts/run-with-memory-cap.sh 一行注释：`python3 scripts/mutation/crates-mutation-rows.py judge-version <根>` 打的摘要不变 |
| C3 | 只改 crates/mutations.tsv 某一行的变异名：那一行照复用 |
| C6 | 在一个编进测试二进制、被执行过的 .rs 里加一个从没被调的非测试函数：同组各行照「按覆盖复用」 |
| K1 | 同一条 crash-amplification.sh --item <项> 连跑两趟：第二趟的 record.log 不存在或没有 CRASH_AMPLIFICATION_RECORDED 行 |
| K7 | 只改 crash-amplification-lib.sh 一行注释：admission.py crash-case-manifest 给那三条用例算的指纹不变 |
| E2 | 只改 e165 装置依赖闭包里某个 crate 的一行注释：360 行照复用 |
| E4 | 结果文件留着、只改 tsv-2 那条变异行的替换文：续跑重跑 tsv-2 |
| R2 | 只切 ENABLE_GPU：复跑表各行的键变了 |
| X1 | 只在装置 dep-info 列到的一份文件里改一句注释：那个装置的单元键不变 |
| X5 | 改了变异表某行之后带 --skip-finished 重起：那一行的旧 row-result.tsv 不被收 |

三次推导：每一行的正推是读算键与比键的那几行代码（行号在表里）；反推每行至少做了一步「找有没有别的路径能让它不成立」（例 B3 的 `grep -n "pruner.carry("`、C1 的 `read_bytes` 与 `manifest_of_files(root, [], …)` 只走名字行、K2 / K3 的 ls 与 du、X2 的 ls）；
校验（换一条不共用代码的路再做一遍）只做了磁盘一侧：K2、K3、K6、C2、X2 用 ls / du 看到了库目录与标记文件的实际形态，与代码推的一致。其余各行的校验路子是真跑上面那张表里的观测，这一轮只读，没做，这些行照规则只算「代码读出来的」，不算三步走完。

## 5 要主 agent 跑的命令

没有。这一路只读；上面推翻条件里要起驱动、门禁或 cargo 的，留给改动批之后的攻方或验证去造。

## 6 没做什么

- 没起 cargo、门禁、驱动、测试二进制与任何自证；`crates-mutation-rows.py judge-version`、`broad-mutation.py --list-items --changed-since` 这类只读子命令也没跑（派发提示只许读文件、grep、git log/show、python3 解析）。
- 没改仓里任何文件；改法只写方向与落点。没建仓副本、没建编译目录；草稿目录里只有派发时就在的 prompt.md、.owner、.writer-lock，与这一轮写的 report.md、files.txt、files-broad.txt、hitcounts.txt、main-set.txt、rest.txt（都是 grep 清单）。
- scripts/gate/ 下的 admission.py、verus-proof-check.py 等没逐行读算法（不在这一路）；凡牵到的只引了文件头与常量行。
- crates/ 下的崩溃库键（块键、计划哈希、结论版本、判器闭包）没读，归 crates 路。
- 每类改动多久发生一次、作废一次多花多少挂钟，一个都没量。
- e170 目录 58 份与 e171 目录其余 48 份是按 grep 命中的那几行判「不是复用键」的，没有逐份整读。
