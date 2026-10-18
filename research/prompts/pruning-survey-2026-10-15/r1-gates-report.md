# 第一轮全工程查复用键：门禁与阶段层（路子 gates）

日期：2026-10-15。只读：没起 cargo、门禁、测试二进制，没动判定库，没写源码摘要缓存，仓内文件一个没改。
射程：`.claude/gate.d/`（17 道阶段脚本，`ls .claude/gate.d/*.sh | wc -l` 本轮输出 `17`；每道表里至少一行、`lib/`、`stage-inputs.tsv`、`stage-owners.tsv` 与登记表）、`scripts/gate/`（`admission.py`、`stage-must-run.sh`、`gate-staged.sh`、`change-touches-crates.sh`、`changed-paths.sh`、`stage-run-or-skip.sh`、`lib_source_digest.py`、`verus-proof-check.py`）、`.claude/singlefs-ai-sop/scripts/gate.sh` 里与标记、复用、staged-green 有关的段。
门禁直接调、但住在 `scripts/mutation/`、`scripts/experiments/` 的两处判定库键（crates 变异行键、红例钉表装置键）顺手列了，标「scripts 路子的地盘」，由主 agent 并。
标注：「量的」= 指到的那一行代码或本轮命令的原样输出；「推的」= 由代码推出来的影响，没跑实测。行号都是本轮 `grep -n` / `sed -n` 现取。

## 结论（一句一条）

- 量的：门禁层的复用一共两层、两路。外层是「整道或整格」：① 全绿标记（`admission.py:2797-2817` 的 `stage_fingerprint` 当键，`:2820-2866` 存取），② 与 `refs/sop/staged-green` 那棵树逐文件比（`admission.py:1547-1629` 的 `gate_reuse`，ref 由 `gate-staged.sh:36-48` 整轮全绿才前移），外加四道再问一次 `change-touches-crates.sh`（对 HEAD 取 diff，`:57`、`:89`）。里层「按条」的判定库只有形式化（`verus-proof-check.py`、`e165-pin-red-examples.py`）、crates 变异（`crates-mutation-rows.py`）、mutation-tables-compile（研究门禁 `:385-432`）、复跑按行（`admission.py:3229-3256`）、崩溃枚举用例（`admission.py:2158-2175` 整条用例一格）几处。
- 量的：外层两路都违反判据。①路的键是文件级源码摘要加路径、整张 `stage-inputs.tsv`（整道键）、阶段脚本整份、登记行原文、手写写法档位；②路就是按「上一次整轮全绿的树」算改动范围，而且没登记 `environment=` 的道比完路径直接放行、不比工具链（`admission.py:1608-1609`）。
- 量的：门禁层所有摘要的最小单元是文件，不是函数：`lib_source_digest.py` 的 `.rs` 是整文件去注释词法，`.toml` `.lock` `.tsv` `.wgsl` `.json` 与 crates/ 外的 `.md` 取原文（`:152-162`）。本轮探针在同一份 `.rs` 里只改一个无关函数，摘要就变（见校验 ①）。
- 量的：崩溃枚举用例的判法摘要按 ast 名字闭包收函数（方向对），但每个函数取原文、连注释（`admission.py:2680-2683`、`:2777`）；本轮探针在闭包里一个函数体内加一行注释，摘要从 `29c7a10a5c84` 变成 `89e9bc4cfe08`（见校验 ②）。
- 量的：完全没有判定库、只有整道一个标记或连标记都没有的：QEMU 六档（一个标记）、LKMM 八份 litmus（一个标记）、崩溃点重放快档两格、模型对拍六格、experiment-replay 外层、research-unit-tests、code-tooling 的 research-script-selftests。其中崩溃点重放快档、模型对拍、experiment-replay 外层不写全绿标记，只剩树对比；research-unit-tests 与 research-script-selftests 连复用判定都不问，每趟全跑。
- 量的：键过窄会漏判的有三处：树对比那一路不比工具链（所有没登记 `environment=` 的道，`admission.py:1608-1609`，`stage-must-run.sh:43-46` 文件头自认）；QEMU 的 QEMU 版本、内核镜像不进键（`stage-inputs.tsv:37` 第三列只有前提）；LKMM 的内核树 `tools/memory-model` 不进键（`checker-tier-lkmm.sh:22` 自认）。
- 全表见下一节，共 68 行；「射程外」「没查到 / 判不了」两节在表后。

## 校验（本轮跑过的命令与原样输出）

① 源码摘要的粒度：在草稿目录写探针，直接 import `lib_source_digest.py` 调 `digests_of(条目, None)`（缓存给 None，不写缓存），三份合成内容：同一份 `.rs` 原样、只改无关函数 `unrelated` 的返回值、只在文件头加一行 `//` 注释；`Cargo.toml` 与 `stage-inputs.tsv` 各原样与加一行 `#` 注释。

```
$ nice -n 19 python3 -I digest_probe.py
rs [('rust', 'ec0fa0201dde'), ('rust', '41a26db31d75'), ('rust', 'ec0fa0201dde')]
toml [('raw', 'ae630b460f5f'), ('raw', '0776872797e5')]
tsv [('raw', '5dd1197866f4'), ('raw', '482ed8335b5c')]
```

读法（量的）：`.rs` 只加注释摘要不变（第一、三个相同），说明探针分得出「改注释」与「改代码」；只改别的函数摘要就变（第二个不同）＝文件级，违反「单元是函数」③。`.toml`、`.tsv` 加一行注释摘要就变＝原文进键②。

② 崩溃枚举用例判法摘要对注释敏不敏感：import `admission.py`，拿它自己的原文调 `crash_case_judging_digest_text`，两种改法：闭包里的 `judge_worker_threads` 函数体头加一行注释；闭包外的 `experiment_fingerprint` 函数体头加一行注释。

```
$ nice -n 19 python3 -I judging_probe.py
original 29c7a10a5c84 definitions 79
comment inside judging function 89e9bc4cfe08 definitions 79
comment inside experiment_fingerprint 29c7a10a5c84 definitions 79
```

读法（量的）：闭包外改注释摘要不变（闭包法起作用，这一格是对照，证明探针会出「不变」）；闭包内只改注释摘要就变＝函数原文连注释进键②。两份探针脚本在会话暂存目录 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/`（`digest_probe.py`、`judging_probe.py`），只读、不写仓。

③ 哪几道不写全绿标记（量的）：

```
$ grep -n 'stage-marker' .claude/gate.d/checker-tier-crash-point-replay.sh .claude/gate.d/harness-model-differential-and-scenarios.sh .claude/gate.d/checker-tier-research-build-and-replay.sh | wc -l
0
```

（crates 变异的标记由 `scripts/mutation/crates-mutation-rows.py:2595-2600` 写，不在阶段脚本里；QEMU `:455`、LKMM `:85`、形式化 `:412`、广谱 `:807` 在阶段脚本里写。）

正推 / 反推 / 校验怎么走的：正推是逐函数读算键的代码（行号在表里）；反推是对每条「整片作废」的结论问「若键是函数级，该看到什么」——应当看到只改无关函数摘要不变，探针 ① 第二格看到的是变了；校验是另一条不共用读代码这条路的路子——直接跑算摘要的函数（探针 ①②），两条路一致。没跑的：整道键里「整张登记表进键」没有用真仓跑 `stage-fingerprint` 核（它会写源码摘要缓存，派发提示不许），只凭 `admission.py:1592`、`:2808` 两处独立代码与 `stage-inputs.tsv:30-31` 的口径句，标量的（代码），实际触发频率没量。

## 逐处表

列：编号 / 位置（文件:行号）/ 管哪种判定 / 键由什么组成（逐项）/ 违反（①–⑧）/ 量或推 / 重做方向（单元怎么切、键改成什么、落点）。「无」＝这一处没有复用、也不该有，或本身合规。

| 编号 | 位置 | 管哪种判定 | 键由什么组成 | 违反 | 量/推 | 重做方向 |
|---|---|---|---|---|---|---|
| G01 | `scripts/gate/admission.py:417-423` `RegistrationRow.fingerprint_text` / `text_with_conditions` | 整道标记、格标记、实验准入、崩溃用例标记里「登记行」那一项 | 键名（阶段文件名、`<阶段>:<格>`、`crash-case:<名>`、实验号）+ 登记路径原样（+ 第三列条件原样） | ⑤③ | 量的 | 登记行不进任何键；登记表只答「从哪几个入口起算」，键由入口沿调用图收到的函数单元组成。落点 `stage_fingerprint`（:2809）、`crash_case_manifest`（:2172）、`experiment_fingerprint`（:871） |
| G02 | `admission.py:598-617` `listed_input_files` | 上面全部的「输入文件」 | `git ls-files` 按登记的目录前缀列出的全部文件 | ③ | 量的 | 换成单元枚举：Rust 从入口 fn 起的调用闭包、Python 从入口 def 起的 ast 名字闭包、shell 从入口函数起的函数名闭包。落点 `admission.py` 新函数，单元切分放 `lib_source_digest.py` |
| G03 | `admission.py:840-865` `manifest_of_files` | 全部清单式键 | 每行「文件摘要␣␣路径」；门禁取 G04 的文件级摘要，实验与弄坏开关分支取文件原文 sha256（:858）；末尾名字行「sha256␣␣<名字>」名字里带路径或文件名 | ②③⑤ | 量的 | 清单行只写单元内容指纹，不带路径与文件名，按指纹排序去重 |
| G04 | `scripts/gate/lib_source_digest.py:152-162` `classify`、`:532-610` `digests_of`/`working_tree_digests`、`:95` `DIGEST_FORM` | 门禁全部源码摘要（整道、格、崩溃用例、树对比） | `.rs` 整文件去注释词法；`.sh` 整文件 `declare -f`；`.py` 整文件 tokenize 去注释；`.toml` `.lock` `.tsv` `.wgsl` `.json` 与 crates/ 外 `.md` 取原文；被 `include_str!` 读的、算出来 include 的整个 crate 取原文；写法名是手写串 | ②③④ | 量的（校验 ①） | 切到函数单元：Rust fn 项与 impl 方法、Python def、shell 函数，各自去注释取词法；数据文件按解析后内容（`.wgsl` 去注释取词法，`Cargo.toml` 取解析后的依赖与特性表，`Cargo.lock` 只取闭包用到的包与版本，`.tsv` 取被读到的那几行）；写法名改成算摘要那几个函数单元自己的指纹 |
| G05 | `admission.py:834-837` `key_form_line`、`:338` `INPUT_KEY_FORM_VERSION = "2"` | 整道与格标记、崩溃用例标记 | 手写档位 "2" + G04 的 `DIGEST_FORM` | ④ | 量的 | 删手写档位；键写法版本＝算键那几个函数单元的指纹，代码一变自动换 |
| G06 | `admission.py:643-658` `toolchain_line` | 整道与格标记、崩溃用例、实验、复跑行 | `cargo -V`、`rustc -V` 原样 | 无（本身合规；树对比那一路不用它，见 G12） | 量的 | 保留；`rustc -V` 不带 LLVM 版本，要不要换 `-Vv` 交主 agent（推的） |
| G07 | `admission.py:669-717` `build_environment_lines`、`:744-803` `program_contents`/`configured_program_lines` | 同上 | 仓根往上每层 `.cargo/config(.toml)`、`rust-toolchain(.toml)` 原文；`CARGO_HOME` 下配置原文；`RUSTFLAGS` 等变量值；runner / wrapper 指的程序与参数里的脚本整份原文；`RUSTC` 的 `-V` | ②③ | 量的 | 配置取 cargo 解析后的生效值（只取影响产物的键），runner / wrapper 取版本或函数单元指纹，不取文件原文 |
| G08 | `admission.py:868-876` `experiment_fingerprint`、`:1017-1038` `judge_inputs_unchanged`、`:1106-1171` `admit_experiment`、`:1174-1189` `stored_product_status` | 实验准入（装置开跑前「输入没变就拒跑」），doc-experiments 的 evidence-in-repo 格 | 登记路径下逐文件原文 sha256 + 登记行（G01）+ 工具链 + 构建环境；E142 一行登记了 7 份 kb 决策与布局 `.md`（`stage-inputs.tsv:50`）；结论只存在产物头，一个实验一个判定 | ②③⑤⑥ | 量的 | 可能归射程外（与 preflight `inputs-changed` 同类，交主 agent 判）。要改就：装置入口函数调用闭包单元 + 解析出的依赖版本 + 工具链；kb 文档进不进键见「判不了」 |
| G09 | `admission.py:2993-3015` `command_manifest` | `manifest` 子命令 | 同 G08 逐文件原文 + `--extra-file` 整份原文 | ②③⑤ | 量的（调用方这一路没找到，见「没查到」） | 随 G03 改；确无调用方就删 |
| G10 | `admission.py:1239-1252` `current_gate_environment` | 登记了 `environment=` 的道与格（LKMM、形式化 Verus 四格、广谱变异） | 条件原文 `environment=<仓内脚本>:<参数>`（带脚本路径）+ 探针整段 stdout | ⑤ | 量的 | 只取工具身份（版本串、可执行文件内容哈希），不带登记写法；并进逐单元键 |
| G11 | `admission.py:1285-1314` `judge_gate_environment`、`:1317-1351` `record_gate_environment` | 树对比那一路的环境判 | git common-dir 每道一份记录（stage、tree、environment_sha256）；比「记下那棵树」与暂存树在登记路径上的差（:1306） | ①⑧ | 量的 | 删；环境作为单元键的一项进判定库，不另存按树的记录（每道只一份、别的树判绿就覆盖） |
| G12 | `admission.py:1547-1629` `gate_reuse` | 所有经 `stage-must-run.sh` 的道与格 | 先查全绿标记（G16），没有再与 `refs/sop/staged-green` 那棵树比（:1575、:1597）；整道把整张 `stage-inputs.tsv` 加进比对路径（:1592）；阶段脚本整份（:1592）；格另比登记行文本（:1594）；没登记 `environment=` 的比完路径直接可跳过、不比工具链与构建环境（:1608-1609） | ①③⑤⑦ | 量的 | 删掉树对比整条路（连 `SINGLEFS_STAGED_TREE` 那一支）；复用只问判定库里的单元键；整道或整格「可跳过」由它下面每条判定都命中库推出。落点 `gate_reuse`、`stage-must-run.sh` |
| G13 | `admission.py:1523-1536` `registered_paths_differences`、`lib_source_digest.py:664-694` `tree_differences` | G11、G12 的比树 | 两棵树在登记路径上 `git diff --raw`，再逐文件按 G04 摘要比；文件模式变了也算变（:675-676） | ①③⑤ | 量的 | 删 |
| G14 | `admission.py:1490-1506` `cell_registration_lines` | 格的树对比 | `git show <树>:stage-inputs.tsv` 里这一格的行文本 | ①⑤ | 量的 | 删 |
| G15 | `admission.py:1368-1393` `closure_dependency_directories`、`:1405-1457` `closure_directory_unread_paths`、`:1460-1479` `cargo_closure_input_paths`、`:1482-1487` `stage_input_paths` | `closure=` 的两格（红例 complete / replay）、红例装置键（G47）、复跑行键（G24） | path 依赖传递闭包里每个 crate 目录整个（optional 也算），减 tests/、benches/、examples/、别的 bin、crate 根下 cargo 不读的；有 build.rs、`[[bin]]`、算出来 include 的整目录留 | ③ | 量的 | 从 bin 的 `main` 起按 fn 调用图收函数单元，不按 crate 目录。落点同函数，或并进 crates 侧判器闭包算法（见「判不了」） |
| G16 | `admission.py:2797-2817` `stage_fingerprint` | 整道与格的全绿标记 | 逐项：(a) G15 列出的每个文件「摘要␣␣路径」；(b) 整道另加整张 `stage-inputs.tsv`（:2808，`.tsv` 取原文）；(c) 阶段脚本整份（:2808）；(d) 登记行（整道键名＋路径，格另加第三列，:2809）；(e) 工具链；(f) 构建环境；(g) 键的写法（:2810）；(h) 登记了 `environment=` 的此刻环境（:2815） | ②③④⑤ | 量的 | 标记降为索引：每道每格下面每条判定各自的单元键都命中才报整道复用，整道不再有自己的键 |
| G17 | `admission.py:2820-2866` `stage_marker_path`/`check_stage_marker`/`write_stage_marker` | QEMU、LKMM、crates 变异、形式化各格、广谱变异的全绿标记 | common-dir 下文件 `singlefs-stage-green.<阶段或格名>.<指纹>` | ⑤⑥⑧ | 量的；新 clone 丢标记是推的 | 拆到判定单元（QEMU 每档、LKMM 每份 litmus…）各进判定库，库按单元键寻址、不带阶段名；改名不丢 |
| G18 | `admission.py:2158-2175` `crash_case_manifest`、`:2090-2155` `files_exclusive_to_other_test_targets`/`files_crash_cases_do_not_read` | 崩溃枚举用例全绿标记（`--full` 逐条跑、快档逐条核） | 整个 `crates/`、`Cargo.toml`、`Cargo.lock` 下 git 列出的文件，减别的测试目标独占的测试文件、`crates/mutations.tsv`、条件成立时的 `src/bin/`；每行「文件摘要␣␣路径」；加判法摘要（G20）、工具链、构建环境、双机驱动（G19）、键的写法（G05）、登记行连第三列 | ②③④⑤ | 量的（路子三日志「421 个文件减去 0 个」是线索，本轮没重核） | 用例键＝用例测试函数（连 setup）沿调用图收到的函数单元指纹 + 闭包用到的依赖版本 + 工具链；第三列里定日志判法的 count-line / exhaustive / threads 归判法函数单元，不按登记原文进。落点 `admission.py`「崩溃枚举用例」一节 |
| G19 | `admission.py:2178-2204` `shard_driver_lines`/`source_digested_lines`、`:326` `SHARD_DRIVER_FILES` | 登记了 `shard=across-machines` 的用例 | 四份双机驱动 shell 整份的文件级摘要，名字行 `<双机分片：路径>` | ③⑤ | 量的 | 取驱动里这条用例真走到的 shell 函数单元 |
| G20 | `admission.py:2716-2785` `crash_case_judging_digest_text`、`:2680-2683` `source_of_node`、`:2788-2793` `crash_case_judging_digest_line` | 全部崩溃枚举用例的「判日志、写标记」判法 | 从 `CRASH_CASE_JUDGING_ENTRIES`（:357）与分派表起的 ast 名字闭包；每个定义取原文（连注释、空行）按名字排，带「## 名字」头；加分派表、main 与全部模块级其余语句原文 | ②③⑤ | 量的（校验 ②） | 闭包法保留；单元内容改去注释（tokenize 或不带位置的 ast dump），每个函数单元各自指纹、名字不进；模块级其余语句只收闭包真引到的 |
| G21 | `admission.py:2213-2218` `crash_case_marker_path`、`:2377-2485` 读写核标记 | 同 G18 | common-dir 下 `singlefs-crash-case-green.<用例名>.<指纹>`；一条全量用例一个标记 | ⑤⑥⑧ | 量的 | 按单元键寻址，用例名只做显示；用例里面的块级复用归 crates 侧判定库（别的路子） |
| G22 | `admission.py:2552-2593` `crash_case_launch`、`:2595-2605` `crash_library_home` | `--full` 起用例时给的库目录 | `--start-over` 时 `<家目录>/crash-point-replay-start-over/<整条用例指纹>`，不然清掉调用方的库变量、用例照 crates 侧长期库 | 无（清空复用的口子） | 推的 | 不动；长期库的位置与键归 crates 路子 |
| G23 | `admission.py:3124-3141` `replay_shell_functions` | 复跑行键里的驱动函数与判一行的函数 | `replay.sh` 顶格函数原文，只去整行注释与行尾空白，行内注释留着（:3139） | ② | 量的 | 走 G04 的 `.sh` 那一套（bash 解析后 `declare -f`），按函数单元 |
| G24 | `admission.py:3156-3226` `research_binary_sources`/`research_source_paths`/`replay_mentioned_paths`/`replay_row_input_paths` | 复跑行键的「装置源码」 | 公共部分 `scripts/experiments/Cargo.toml`、`Cargo.lock` 与 `lib.rs` 带进来的全部（:3201）；装置顺 mod 带进来的整份文件；判不出时整个 `scripts/experiments/`；字面点到 `scripts/experiments/<子目录>/` 下的文件时取整个子目录（:3193）；`-p` 不带 `--bin` 取包闭包每个 crate 整目录（:3220） | ③ | 量的 | 装置 `main` 起的函数单元闭包 |
| G25 | `admission.py:3229-3256` `replay_row_keys_of_inputs` | 复跑表每一行能不能复用（experiment-replay 格里的按行库） | G24 的文件逐个摘要（源码摘要库不在时退回原文，:3241）+ 复跑表这一行原文（含实验号、产物文件名）+ 驱动闭包原文 + `<留存产物：名>` 与产物 sha256（:3251）+ 判一行的几个函数原文（:3236）+ 工具链 + 构建环境 | ②③⑤ | 量的 | 行键＝驱动或装置入口函数单元闭包 + 这一行的参数 + 期望产物内容 + 工具链；实验号、文件名不进。落点 `admission.py` replay 一节、`scripts/experiments/replay.sh` |
| G26 | `scripts/gate/stage-must-run.sh:56-61` | 所有调它的道与格 | 只转 `admission.py gate-reuse` | 同 G12：①③⑤⑦ | 量的 | 随 G12 改成问判定库 |
| G27 | `scripts/gate/gate-staged.sh:28-48`、`admission.py:265` `GATE_REUSE_REFERENCE` | G12 树对比那一路的基准树 | 整轮 `gate.sh --staged` 全绿才把 `refs/sop/staged-green` 前移到这次暂存树，只一条 ref | ①⑥ | 量的；「一道红，所有道下一趟只能与更早那棵树比」是推的 | 删 ref 的复用用途，gate-staged 只管跑整轮 |
| G28 | `scripts/gate/change-touches-crates.sh:57`（基准恒为 HEAD）、`:89-120` | 崩溃点重放快档（门禁 :354）、QEMU（:231）、模型对拍（:160）、experiment-replay（:339）的「这次没碰就跳过」 | `git diff --name-only HEAD` 加 `git status --porcelain`，按登记前缀匹配 | ①③ | 量的 | 删这一问；单元键命中库已经覆盖它。落点四道门禁里调它的几行与这份脚本 |
| G29 | `scripts/gate/stage-run-or-skip.sh:21-39` | 只转判定脚本退出码 | 无键 | 无 | 量的 | 不动 |
| G30 | `scripts/gate/changed-paths.sh`（`gate_diff_base`、`gate_changed_paths`、`gate_added_lines`） | doc-*、code-*、checker-independence-and-sync 的「这次新加的行」判据范围 | `GATE_BASE` / merge-base / HEAD 到工作区与暂存区的 diff | ①（字面），但这是判据语义（只判新增行），不是判定复用 | 量的 | 见「射程外」；不在剪枝改动里（推的，交主 agent） |
| G31 | `scripts/gate/verus-proof-check.py:788-794` `judge_version` | Verus proof、空真、必红世界三格的按条判定库 | 这份脚本整份原文 sha256 + `scripts/crash/lib_verdict_library.py` 整份原文 sha256 | ②③ | 量的 | 判法版本＝判那一项真调到的函数单元（去注释）指纹；存储库的读写函数不进 |
| G32 | `verus-proof-check.py:796-813` `shared_components`（`lib_verdict_library.py:52-76` `digest_file`/`digest_tree`） | 同上，每项共用的输入 | 核 crate 整个目录逐文件原文 sha256 带相对路径（跳过必红世界表）+ `verus-pins.tsv` 整份原文 + 工具链（`--tool-versions` 原文）+ 限时秒数 | ②③⑤ | 量的 | 每项键＝这一项涉及的 Verus 函数单元（被验函数与它 requires / ensures 用到的 spec fn、proof fn）去注释指纹 + Verus 与 vstd 版本；限时移出键（只影响超时，超时不入库） |
| G33 | `verus-proof-check.py:853-858` `check_proof` | proof 一项 | G32 一把，整个 crate 一项 | ③⑥ | 量的 | 按 Verus 函数单元拆项（能不能做到见「判不了」） |
| G34 | `verus-proof-check.py:1026`、`:1068` `check_vacuity` | 空真每处 requires 一项 | G32 + 项名 `vacuity:<核的路径>:<requires 行号>` 与说明 | ③⑤ | 量的；「上面插一行、下面的项全换键」是推的 | 项身份改为所属函数单元内容指纹 + 函数内第几处 requires；键＝那个函数单元及其依赖 + 插入点 |
| G35 | `verus-proof-check.py:1201-1202` `check_must_red_worlds` | 必红世界每行 | G32 + 这一行的名字、文件路径、原文、替换文、期望 | ③⑤ | 量的 | 去名字与路径；原文所在函数单元指纹 + 替换文 + 期望 |
| G36 | `verus-proof-check.py` toolchain 一项（`:683` `check_toolchain` 起，每趟现判） | Verus 安装与钉版本核对 | 不复用 | 无（便宜） | 推的 | 不动 |

### 逐处表（续：各道阶段脚本与登记表）

| 编号 | 位置 | 管哪种判定 | 键由什么组成 | 违反 | 量/推 | 重做方向 |
|---|---|---|---|---|---|---|
| G37 | `.claude/gate.d/checker-tier-crash-point-replay.sh:351-356` 快档外层 | 整道快档（crash-case-replay、feature-gated-tests 两格一起） | `stage-must-run`：整道键（`stage-inputs.tsv:36`：`crates/`、两份清单、`admission.py` 整份、七份崩溃脚本、合表 + 整张登记表 + 阶段脚本，G16）；再问 `change-touches-crates`（HEAD，G28）；这一道不写全绿标记（校验 ③），只剩树对比 | ①③⑤⑥⑦ | 量的 | 快档每条测试函数一条判定入库（键＝测试函数调用闭包单元 + 工具链 + 依赖版本），整道跳过由每条命中推出 |
| G38 | 同文件 `:433-441` `run_checker_package_tests`（crash-case-replay 格） | checker 档包不标 ignored 的全部测试 | 没有库：`cargo test --release -p singlefs-checker-tier --lib --tests` 整包一趟 | ⑥ | 量的 | 同 G37，按测试函数入库，只跑没命中的（`--exact` 名单）；落点这一段与 `scripts/gate/lib_libtest_result_lines.py` |
| G39 | 同文件 feature-gated-tests 格（格定义 `:10`，跑法在 `:595` `run_checker_quick_tier` 之后） | 快档带特性那一趟 | 没有自己的键，随整道 | ⑥ | 量的 | 同 G38，键另加特性组合 |
| G40 | 同文件 `:408-420` `write_crash_case_manifest`（`--full` 逐条） | 每条崩溃枚举用例 | `crash-case-manifest --judging-digest --toolchain --build-environment`：G18、G19、G20、G05 | 同 G18、G20 | 量的 | 同 G18、G20 |
| G41 | `.claude/gate.d/checker-tier-qemu-device-streams.sh:218-233`、`:450-457` | 六档虚机（direct、skip-new-pool-file-creation-barrier、page-cache、file-overwrite、second-instance、raise-rollback-floor） | 整道键（`stage-inputs.tsv:37`：`crates/`、`Cargo.toml`、`Cargo.lock`、整个 `scripts/`、整个 `research/results/`、`.claude/kb/layout/01-first-txn.md` 原文 + 整张登记表 + 阶段脚本）+ `change-touches-crates`（HEAD）；第三列只有 `command=` `readwrite=` `probe=`，没有 `environment=`：QEMU 版本、内核镜像不进键；六档一个标记；`--item` 跑完不写标记（:452-454） | ①②③⑥⑦ | 量的 | 每档一条判定入库，键＝那一档宿主与来宾里真跑到的函数单元闭包 + QEMU 版本 + 内核镜像内容哈希 + 工具链；`--item` 照常入库。落点 qemu 门禁、`stage-inputs.tsv:37`、`scripts/devices/` |
| G42 | `.claude/gate.d/checker-tier-lkmm.sh:56-61`、`:78-87`；`.claude/scripts/lkmm.sh:332-346`（找内核树） | `litmus/` 八份 litmus 的 herd7 判定 | 整道键（`stage-inputs.tsv:38`：`litmus/`、`lkmm.sh`、`fetch-deps.sh`、`lib.sh`、整个 `crates/` + 整张登记表 + 阶段脚本）+ herd7 版本（G10）；一个标记；内核树 `tools/memory-model`（模型 .cat / .bell）不进键（`checker-tier-lkmm.sh:22` 自认）；经 `stage-must-run` 走 G12 的树对比与 G11 的环境记录 | ①③⑤⑥⑦ | 量的 | 每份 litmus 一条判定，键＝litmus 内容（去注释）+ 它 `singlefs-models` 锚点指的那几个函数单元 + herd7 版本 + 模型文件内容。落点 `lkmm.sh`、lkmm 门禁、`stage-inputs.tsv:38` |
| G43 | `.claude/gate.d/checker-tier-crates-mutation-replay.sh:247-251` 外层；标记由 `scripts/mutation/crates-mutation-rows.py:2595-2600` 写 | 整张 `crates/mutations.tsv` 的整道复用（不选行时才问） | 整道键（`stage-inputs.tsv:45`：`crates/`、清单、锁、`litmus/`、`.cargo/`、内存包装、三份 python、判定库、`.claude/heavy-thread-default` + 整张登记表 + 阶段脚本）；经 `stage-must-run` 走 G12 树对比，没登记 `environment=`，树对比那一路不比工具链 | ①③⑤⑥⑦ | 量的 | 外层删，整道复用由每行命中推出；线程默认值文件不进键 |
| G44 | （scripts 路子的地盘，顺手列）`scripts/mutation/crates-mutation-rows.py:601-616` `judge_version`、`:640-642` `row_key` | crates 变异每行的判定库 | 判法版本＝登记行文本 + 阶段脚本、共用模块、`mutation-prune.py`、libtest 解析、`lib_verdict_library.py`、内存包装各整份原文 + 构建环境；行键＝六段原文（含变异名与文件路径）+ 内存上限 + 限时 + 工具链 | ②③⑤ | 量的 | 判法版本取判「抓没抓到」那几个函数单元；行键去变异名、路径、上限、限时，改成被改坏的函数单元指纹 + 替换文 + 点名测试的调用闭包 |
| G45 | `.claude/gate.d/checker-tier-formal-verification.sh:348-362`（按格问）、`:404-415`（写标记） | 六格的外层 | 格键（`stage-inputs.tsv:39-44`：各格登记路径 + 阶段脚本整份 + 本格登记行连第三列 + 工具链 + 构建环境，Verus 四格另加 `--tool-versions`）；阶段脚本一份六格共用；六格都登记了内存包装、`capped.sh`、`lib/stage-cells.sh`、`lib_verdict_library.py`；标记只在整道退 0 且没有本次未跑时才写（:404）；每格经 `stage-must-run` 走 G12 树对比（红例两格没登记 `environment=`，那一路不比工具链） | ①③⑤⑥⑦ | 量的 | 格级标记删，以按条库为准（G31–G35、G47）；基础设施脚本（内存包装、capped、stage-cells、存储库）不进判定键 |
| G46 | 同门禁 pinned-red-examples-complete 格（`stage-inputs.tsv:43`） | 红例账钉全（每趟现判，不按条复用） | 格键含 `crates/mutations.tsv` 整表原文、`Cargo.toml`、`Cargo.lock` 原文、`admission.py` 整份、`research/results/e165-*` 全部产物、`closure=` 包闭包 | ②③ | 量的 | 静态核表、便宜（推的）：可以不建库；要建就按账行入库，键＝那一行与它点到的钉表行 |
| G47 | （scripts 路子的地盘，顺手列）`scripts/experiments/e165-pin-red-examples.py:1162-1168` `judge_version`、`:1171-1186` `device_inputs_digest`、`:1204-1213` `row_components` | 红例钉表每行重放 | 判法版本＝脚本与判定库整份原文；装置一项＝装置 cargo 依赖闭包（G15）逐文件原文 sha256 + `Cargo.toml` + `Cargo.lock` + 工具链，全部行共用；每行另加这一行原文、档、引的变异行原文 | ②③ | 量的 | 装置键改为这一行的档与参数真走到的函数单元闭包 |
| G48 | `.claude/gate.d/checker-tier-broad-mutation.sh:258-431` library-version-is-staged-tree 格 | 广谱变异结论库「那一版」是不是这次暂存树 | 导出范围逐文件原文 sha256（:294-334）对 `manifest.tsv`；源码指纹（:345）；`rustc -Vv`；全局键（:413：`JUDGE_RULES_VERSION` 手写常量 `scripts/mutation/broad-mutation.py:469`、`Cargo.toml` / `Cargo.lock` / `.cargo`、构建环境、工具版本、排除表整份、升档常量、限时）；`version_key`（:423）；出路要求 `--sync-tree --commit` 再 `--changed-since HEAD`（:168、:249、:449） | ①②③④ | 量的 | 删「库版本＝整棵树」这一格；结论库按每条变异的单元键（被变异函数单元 + 跑到它的测试函数闭包 + 工具链 + 依赖版本）存取，门禁只判「这棵树上每条变异都有键命中且抓到」。落点广谱门禁、`broad-mutation.py`、`mutation-prune.py`、`.claude/skills/crash-test/commit-stages.md` 第 2b 步 |
| G49 | 同门禁 `:744-830`（`marker_inputs_clean`、`marker_check`、`write_marker_if_clean`、外层 `:827`） | 广谱变异整道复用 | 整道键（`stage-inputs.tsv:46` 全部 + 整张登记表 + 阶段脚本 + `--tool-versions`）+ 一个标记；`marker_inputs_clean` 比工作区与暂存区（护栏，不是键）；外层经 `stage-must-run` 走 G12 树对比 | ①③⑥ | 量的 | 同 G43 |
| G50 | `.claude/gate.d/checker-tier-research-build-and-replay.sh:327-345` experiment-replay 格 | 复跑表每一行 | 外层 `stage-must-run` 用整道键名（不是格键），所以整张登记表进；登记 `crates/`、清单、锁、`scripts/experiments/`、整个 `scripts/`、整个 `research/results/`（`stage-inputs.tsv:48`）；再问 `change-touches-crates`（HEAD，:339-341）；这一道不写标记（校验 ③）；没登记 `environment=`，树对比那一路不比工具链；里层按行键见 G25 | ①③⑥⑦ | 量的 | 外层删，只留按行库（G25 改后） |
| G51 | 同门禁 `:385-432`（库读写）、`:455-460`、`:504`、`:521` mutation-tables-compile 格 | 研究侧变异表只编译 | 格键（`stage-inputs.tsv:49`：整个 `scripts/mutation/tables/`、整个 `scripts/experiments/`、`mutate.sh`、判定库、`crash-amplification-lib.sh`、整个 `research/results/`）；按表一项（:424，项名＝表文件名）：表、源文件、它引的文件、`Cargo.toml`、`Cargo.lock`、`lib.rs`、`mutate.sh` 各整份原文 + 工具链；判法版本＝整份阶段脚本（五格共用）+ 判定库原文（:504、:521）；库在 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}`（:458）；不写格标记；格外层经 `stage-must-run` 走 G12 树对比（:455-457），没登记 `environment=` | ①②③⑤⑥⑦⑧ | 量的 | 一条替换文一项，键＝被替换的函数单元指纹 + 替换文 + 依赖版本 + 工具链；判法版本取 `mutate.sh --compile-only` 那几个函数单元；库挪到 `~/.local/share/singlefs/` 或 common-dir |
| G52 | 同门禁 `:187` 起 research-unit-tests 格（文件头 `:16`「每次都跑」） | research 工作区编译与单测 | 没有键：每趟 `cargo build --release` + `cargo test --release` 整个工作区 | ⑥ | 量的 | 按测试函数入库（同 G38） |
| G53 | 同门禁 wallclock-asserts-registered 格 | 挂钟断言登记 lint | 静态、便宜，没有键 | 无 | 量的 | 不动 |
| G54 | `.claude/gate.d/harness-model-differential-and-scenarios.sh:149-161`（六格经 `:239-244` 调它） | 模型对拍六格 | 每格问 `stage-must-run` 用整道键名，六格共用一把整道键（`stage-inputs.tsv:47`：整个 `crates/`、清单、锁、内存包装、`capped.sh` + 整张登记表 + 阶段脚本）+ `change-touches-crates`（HEAD）；不写标记（校验 ③）；没登记 `environment=`，树对比那一路不比工具链 | ①③⑥⑦ | 量的 | 每格一条判定入库，键＝那条测试函数调用闭包单元 + 工具链 + 依赖版本 |
| G55 | 同文件 `:132` one-scenario 格 | harness 用例粒度静态判 | 无键 | 无 | 量的 | 不动 |
| G56 | `.claude/gate.d/harness-test-environment.sh:134` machine-clean 格 | 跑完测试机器干不干净 | 判此刻状态，结论不能沿用 | 无 | 推的 | 不动 |
| G57 | `.claude/gate.d/checker-independence-and-sync.sh` 三格（layout-checker-sync 在 `:513-528` 取 `changed-paths` 基准） | 依赖方向、布局与 checker 同步、判器调用点 | 静态扫，无库 | 无（基准是判据语义，见射程外） | 量的 | 不动 |
| G58 | `.claude/gate.d/code-source-discipline.sh` 十五格 | 源码纪律静态判 | 静态扫，无库 | 无 | 量的 | 不动 |
| G59 | `.claude/gate.d/code-tooling.sh:311-428` research-script-selftests 格 | runner 表里全部研究脚本的自证 | 没有库：每趟分批全跑 | ⑥ | 量的（没库）；开销大是推的，没量 | 每条自证一条判定，键＝那份脚本自证入口的函数单元闭包 + 解释器版本；其余十几格静态，不动 |
| G60 | `.claude/gate.d/doc-decisions.sh`（改动范围 `:2331-2373`） | 决策文档 | 静态，无库 | 无（基准是判据语义） | 量的 | 不动 |
| G61 | `.claude/gate.d/doc-experiments.sh`（evidence-in-repo `:583-596`） | 实验页与产物 | 静态，无库；evidence-in-repo 用 G08 的实验指纹判「产物跟不跟得上输入」 | 无（剪枝口径）；随 G08 键过宽会多报「跟不上」是推的 | 量的 | 随 G08 |
| G62 | `.claude/gate.d/doc-process-records.sh` | 三方论证与同步记录 | 静态，无库；改动范围按 HEAD / `GATE_BASE` | 无（判据语义） | 量的 | 不动 |
| G63 | `.claude/gate.d/doc-registries.sh` | 登记表与引用 | 静态，无库 | 无 | 量的 | 不动 |
| G64 | `.claude/gate.d/doc-text.sh` | 文本写法 | 静态，无库 | 无 | 量的 | 不动 |
| G65 | `.claude/gate.d/stage-inputs.tsv:30-34` 与 `:36-66` 每一门禁行 | 登记表本身的口径 | 「清单自己也进门禁的比对」「宁宽勿窄」；全部行按目录前缀登记，崩溃用例行一律 `crates/ Cargo.toml Cargo.lock` | ③ | 量的 | 改成按入口登记（测试函数、bin、litmus、虚机档），整张表不进任何键；删 `:30-34` 那几句 |
| G66 | `.claude/gate.d/lib/stage-cells.sh` | 格的解析与汇总 | 无键（`.gate-cells` 是样本标记） | 无 | 量的 | 不动 |
| G67 | `.claude/gate.d/stage-owners.tsv` | 判红派给谁 | 无键 | 无 | 量的 | 不动 |
| G68 | `.claude/singlefs-ai-sop/scripts/gate.sh:46-115`（`--staged` 临时 worktree）、`:767-782`（`refs/sop/gate-ok`） | 整轮门禁 | 无复用判定；阶段退 77 记本次未跑；`gate-ok` 只给 show-me-test 定 diff 窗口 | 无（射程外） | 量的 | 不动 |

## 按违反条目反查（编号见上表，便于并）

- ①（按树、HEAD、refs 算改动）：G11、G12、G13、G14、G26、G27、G28、G37、G41、G42（连环境记录比树）、G43、G45、G48、G49、G50、G51、G54；G30、G68 是判据语义，另列射程外。
- ②（原文进键）：G03、G04、G07、G08、G09、G18、G20、G23、G25、G31、G32、G41（布局 `.md`）、G44、G46、G47、G48、G51。
- ③（文件、目录、包、整表级）：G01–G04、G07–G09、G12、G13、G15、G16、G18、G19、G20（模块级其余语句）、G24、G25、G28、G31–G35、G37、G41–G51、G65。
- ④（手写版本常量）：G04（`DIGEST_FORM`）、G05（`INPUT_KEY_FORM_VERSION`）、G16、G18、G48（`JUDGE_RULES_VERSION`）。
- ⑤（路径、名字等非函数内容）：G01、G03、G08–G10、G12–G14、G16–G21、G25、G32、G34、G35、G37、G42–G45、G51。
- ⑥（没有库、只有整道整格一个标记）：G08、G17、G21、G27、G33、G37–G39、G41–G43、G45、G49–G52、G54、G59。
- ⑦（键过窄会漏判）：G12 / G26（树对比不比工具链），落到没登记 `environment=` 的 G37、G41、G43、G45（红例两格）、G50、G51、G54；另 G41（QEMU 版本、内核镜像）、G42（`tools/memory-model`）。
- ⑧（库会丢、改名丢、重复判）：G11（环境记录每道一份、覆盖）、G17、G21（改名丢标记）、G51（库在 `/tmp`）。两库重复判（崩溃点重放与崩溃放量）在 crates 路子，不在这一路的射程。

## 推翻条件（什么现象会推翻这些结论）

- G04 / G03「门禁摘要是文件级」：在一份 `.rs` 里只改一个与被测路径无关的函数，`lib_source_digest.py digest` 打的摘要不变，就推翻。本轮造过这个现象的反面（校验 ① 第二格，摘要变了），也造过对照（只加注释摘要不变）。
- G20「判法摘要连注释」：在 `judge_worker_threads` 体内加注释后 `crash_case_judging_digest_text` 的摘要不变，就推翻。本轮造了一次，变了（校验 ②）；对照格（闭包外加注释）不变。
- G12 / G16「整张登记表进整道键」：在 `stage-inputs.tsv` 里只改一个实验行，`admission.py stage-fingerprint <根> checker-tier-lkmm.sh` 打的指纹不变，就推翻。本轮没造（会写源码摘要缓存，派发提示不许），只凭 `:1592`、`:2808` 两处代码与 `stage-inputs.tsv:30-31` 口径句。
- G12「树对比那一路不比工具链」：一道没登记 `environment=` 的阶段，在 `refs/sop/staged-green` 树与暂存树相同、只换了 rustc 的情况下 `stage-must-run.sh` 退 0（要跑），就推翻。本轮没造（要动 ref 与工具链）。
- G37、G50、G54「不写标记」：这三道阶段脚本里出现 `stage-marker-write` 调用，或别的文件替它们写 `singlefs-stage-green.<它们的名字>.*`，就推翻。本轮 `grep` 计数 0（校验 ③）；别的文件替写这一半只查了 `crates-mutation-rows.py`，没全仓搜。
- G41「QEMU 版本不进键」：`stage-inputs.tsv:37` 第三列出现 `environment=`，或 qemu 门禁另把 `qemu-system-x86_64 --version` 拼进指纹，就推翻。现查第三列只有 `command=qemu-system-x86_64 readwrite=/dev/kvm probe=scripts/devices/vm-kernel.sh:--check`。

## 射程外（撞见了，另列）

- `scripts/gate/changed-paths.sh` 与用它的 doc-*、code-*、checker-independence-and-sync 各格：按 `GATE_BASE` / merge-base / HEAD 取「这次新加的行」，字面上是①，但它定的是判据范围（只判新增行），不是沿用旧判定。要不要也改，交主 agent。
- `.claude/singlefs-ai-sop/scripts/gate.sh:767-782` 的 `refs/sop/gate-ok`：给 show-me-test 定 diff 窗口，同上。
- `.claude/gate.d/crash-point-replay-segments.tsv` 第五列「段写表摘要」：是钉住引擎按段登记的那一段写表的身份（核不上引擎拒跑），不是复用键；它按段号寻址，段号挪位就核不上（表头自己写明），属登记口径，不归剪枝。
- `.claude/gate.d/crash-point-replay-items.tsv`、`verus-pins.tsv`、`e165-*` 三张账表、`crates-mutation-light-rows.tsv`：登记表，本身不是键；它们作为整份原文进键的地方已经在 G32、G46 列了。`crates-mutation-light-rows.tsv` 管的是轻型变异（射程外）。
- `stage-inputs.tsv:36` 把 `scripts/crash/crash-amplification-lib.sh` 登记了两次（重复项，不影响键，量的）。
- `scripts/gate/lib_source_digest.py:365-461` 的摘要缓存：命名空间带库自己、规范副本、Python 与 bash 版本，按内容寻址，是算摘要的缓存不是判定的键，不违反。
- 轻型三条（`crash-point-replay.sh --quick`、形式化 `--quick`、crates 变异 `--quick`）与 preflight 的 `inputs-changed`：照派发提示不在射程，没查。

## 没查到 / 判不了

- `admission.py manifest` 子命令（G09）的调用方：在 `.claude/gate.d/`、`scripts/`、`.claude/singlefs-ai-sop/scripts/` 下 grep `admission.py` 的调用，只找到 `experiment`、`gate-reuse`、`stage-marker-*`、`crash-case-*`、`replay-row-*`、`paths`、`gate-preconditions`、`gate-record-environment`，没找到 `manifest`；research/ 与 `.claude/skills/` 没搜。是死代码还是别处在调，判不了。
- G08 实验准入算不算「射程外」：它与 preflight 的 `inputs-changed` 同为「输入没变就不重跑」，但住在 `admission.py`、键也由它算。派发提示只把 `inputs-changed` 划出射程，这一处交主 agent 判。若要改，「kb 决策文档进不进实验的键」是设计判断（`.claude/singlefs-ai-sop/rules/preflight-discipline.md` 第 27 行要求登记「代码、决策、跑前登记」，与「只看函数单元、工具链、依赖版本」相反），不归这一路定。
- G33 Verus 能不能按函数单元拆项（只验改到的 fn 与依赖它的 proof）：要看 Verus 的 `--verify-function` / 按模块验在 `--no-cheating` 下的语义与 verified 计数怎么对，本轮没查 Verus 文档，判不了。
- G15 / G18 / G24 的「函数单元调用闭包」用哪一份实现：crates 侧已有判器闭包算法（`crates/singlefs-checker-tier/src/judging_closure.rs`，归 crates 路子），门禁侧要不要复用它、还是在 `lib_source_digest.py` 另做一份 Rust 调用图，属设计判断。trait 方法、宏展开、泛型分派下按名字收闭包会不会漏（⑦），本轮没查。
- G41 QEMU 每档「真跑到的函数单元」要跨宿主与来宾两个二进制收闭包；来宾 initramfs、内核镜像怎么取内容哈希，本轮没查 `scripts/devices/`。
- G42 litmus 的 `singlefs-models` 锚点指到的「函数单元」：锚点今天指的是文件（`stage-inputs.tsv:38` 注释写指 `transaction.rs` 与 `recovery.rs`），锚点能不能细到 fn，本轮没查 `lkmm.sh` 的锚点文法。
- G59 research-script-selftests 每趟实际开销：没量（没跑），「开销大」是推的。
- 外层「整道复用由每条命中推出」之后，没有按条库的格（G38、G52、G59 的测试与自证）要先建库，库放哪、谁写：本轮只给方向，没核 `lib_libtest_result_lines.py` 能不能逐条认出测试名与结局。
- 用户口径「整文件、指纹等等方案全部删掉重做」与 G06、G07：工具链与构建环境不是函数单元，判据明许工具链；构建环境（`RUSTFLAGS`、runner）算不算工具链，交主 agent 定。

## 要主 agent 跑的命令

没有。这一轮只读，不起重活；推翻条件里两条没造的（整张登记表进整道键、树对比不比工具链）要在仓副本上动 `stage-inputs.tsv` 与 rustc 再跑 `python3 scripts/gate/admission.py stage-fingerprint <副本根> checker-tier-lkmm.sh` 与 `bash scripts/gate/stage-must-run.sh <副本根> checker-tier-qemu-device-streams.sh` 才核得动，会写副本 git common-dir 里的源码摘要缓存；要核时在仓副本（`rsync -a --exclude target --exclude .git` 之后 `git init` 一份）里跑，前台跑得完、不是重型。

## 没做什么

- 没修、没判该怎么改（重做方向只写方向与落点，不写代码）；仓内文件一个没改。
- 没跑重型测试、门禁、cargo、测试二进制；没动任何判定库与全绿标记；没写源码摘要缓存（两份探针都只在内存里算，`lib_source_digest.py` 的缓存给 None）。
- `scripts/mutation/`、`scripts/experiments/`、`scripts/crash/`、`crates/` 里的判定库只在门禁直接调到的地方顺手列了 G44、G47（与 G31–G32 用到的 `lib_verdict_library.py`），没逐函数查；崩溃判定库（`verdict_store.rs`、`crash_identity.rs`、`judging_closure.rs`、`crash_library.rs`）、崩溃放量、广谱驱动、`replay.sh` 本体归别的路子。
- 三份旧审计报告与对账表只当线索读，表里每一行的行号是本轮现查；没逐条核旧报告的每一句。
- 两份探针脚本留在会话暂存目录（见校验节），没建仓副本、没建编译目录。
