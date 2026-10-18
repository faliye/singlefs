# 重型门禁复用键审计（路子一：按文件头与门禁脚本读）

日期：2026-10-15。只读：没起 cargo、门禁、测试二进制，没改仓内文件。
标注：「量的」= 指到的那一行代码 / 文件头原文，或这一次跑出来的命令输出；「推的」= 由代码推出来的影响，没跑实测。
「整道键」= `scripts/gate/admission.py` 的 `stage_fingerprint`（全绿标记）与 `gate_reuse`（对 `refs/sop/staged-green` 那棵树比）；「库」= 比整道标记细一层、按项存判定的那一份。

## 0 共用层：整道键怎么算（七道里五道的整道复用都走它）

| 项 | 内容 | 依据 |
|---|---|---|
| 整道键的组成 | 登记路径下逐文件源码摘要 + **整张** `.claude/gate.d/stage-inputs.tsv` + 阶段脚本自己 + 本道登记行（只键与路径）+ 工具链 `cargo -V`/`rustc -V` + 构建环境 + 键的写法；登记了 `environment=` 的另加此刻环境 | 量的：`scripts/gate/admission.py:2797-2817`（2808 行把 `REGISTRATION_TABLE` 加进整道的路径，格键不加）、`:2810` |
| 两条复用路 | ① 全绿标记（指纹见上，在 git common-dir）；② 对上次整轮全绿那棵暂存树逐文件比摘要。②里没登记 `environment=` 的不比工具链、构建环境 | 量的：`admission.py:1558-1568`（①）、`:1586-1609`（②，`:1608-1609` 没 environment 直接可跳过） |
| 整张登记表进键 | 文件头自己写明代价：「清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样」 | 量的：`.claude/gate.d/stage-inputs.tsv:31`、`admission.py:1590-1592` |
| `.tsv` 摘要取原文 | 源码摘要库对 `.tsv` 归 raw，注释行里多一个空格摘要就变 | 量的：本轮命令输出（见第 9 节校验 ①）：`same: False` |
| 判法版本的写法不统一 | 整道键与崩溃枚举用例按源码摘要（注释不算），而 crates 变异、形式化、变异表只编译三处的判法版本按原文 sha256（注释也算） | 量的：`admission.py:840-865`（source_digests 只管登记文件，名字行一律原文 sha256）；各处见下 |

推的：整道键是「整道一份」的粒度，下面五道（崩溃点重放快档、QEMU、LKMM、研究复跑两格）的整道复用因此只要登记表里任何一行（哪怕别的实验行、注释）一动就整道作废；`stage-inputs.tsv` 在 2026-09-15 以来 194 次提交里被动过 12 次（量的：本轮 `git log --since=2026-09-15 --oneline -- .claude/gate.d/stage-inputs.tsv | wc -l` 输出 `12`，同期总提交 `194`）。

## 1 checker-tier-crash-point-replay（含 `--full` 与崩溃放量 `scripts/crash/crash-amplification.sh`）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 快档 crash-case-replay（`cargo test --release -p singlefs-checker-tier --lib --tests` 不标 ignored 的用例 + 逐条核标记） | 无库；只有整道键 | 第 0 节整道键：路径 `crates/ Cargo.toml Cargo.lock scripts/gate/admission.py` 与七份崩溃脚本 + 整张登记表 + 阶段脚本；另一问 change-touches-crates 前缀加登记表、阶段脚本、准入模块 | 整道：登记路径或登记表任何一处变 ⇒ 整趟 cargo test 全跑 | 不合理（无库、整道一变全跑；登记表整张进键）。另：没登记 environment=，走②那条路时 rustc 升级不重跑（键过窄） | 量的：`.claude/gate.d/checker-tier-crash-point-replay.sh:351-356`、`:435`、`:55`（「快档不写真的判定库」）；`stage-inputs.tsv:36`（第三列无 environment=）；`admission.py:1608-1609` |
| 快档 feature-gated-tests（带 tests/ 里 cfg 点到的特性另跑一趟） | 无库：清掉调用方的 `SINGLEFS_CRASH_AMPLIFICATION_*`，「快档不写真的判定库」 | 同上整道键 | 同上，整趟全跑 | 不合理：经引擎判的用例（verdict-store）在快档里不进任何长期库，每趟从头判 | 量的：`checker-tier-crash-point-replay.sh:53-55` |
| `--full` 每条崩溃枚举用例（第一层：全绿标记） | git common-dir `singlefs-crash-case-green.<用例>.<指纹>`（本轮 `ls` 数到 22 份） | 整个 `crates/` 与 Cargo 清单、锁，减去别的测试目标独占的测试文件、`crates/mutations.tsv`、没人读的 `src/bin/`；加准入模块里判法的 ast 摘要、工具链、构建环境、这条登记行（连第三列）；`shard=across-machines` 的另加双机驱动四份（源码摘要） | 按用例：用例闭包里任何非测试代码变 ⇒ 这条用例标记作废、整条重枚举；块级判定由下一层库剪 | 基本合理：用例几乎依赖全部 crate（checker-tier 依赖 checker、core、format、harness、reference-filesystem，core 依赖 replay-prefix），按闭包收窄收不了多少（推的）；判法摘要按 ast 闭包，窄而对 | 量的：`admission.py:2158-2175`、`:1615-1630`、`:2716`；`checker-tier-crash-point-replay.sh:410-411`；`crates/singlefs-checker-tier/Cargo.toml:9-14`、`crates/singlefs-core/Cargo.toml:11-13` |
| `--full` 判定库（第二层：块） | `<家目录>/crash-state-enumeration-library`（崩溃枚举那一类的长期库，RocksDB） | 块键 = 输入指纹 ‖ 流名 ‖ 枚举计划哈希 ‖ 块起点；块键里的「输入指纹」实为复用键 = sha256(写表带屏障, 父输出)，只由流程定；判定行另带判法版本 = 判器闭包摘要 | 判器闭包（已抓到的那一处）任何一处变 ⇒ 整库不复用；闭包之外照常复用 | 已抓到的那一处，不重查。另见清单 7、8、9 三条同族问题 | 量的：`crates/singlefs-checker-tier/src/verdict_store.rs:16-24`、`:99-102`；`crash_identity.rs:92-115`；`crash_amplification.rs:399-401`；`scripts/crash/crash-point-replay.sh:7` |
| `--full --start-over` | `<家目录>/crash-point-replay-start-over/<输入指纹>`，开跑前清空 | — | 整库从头 | 按设计（用户点名才用） | 量的：`checker-tier-crash-point-replay.sh:19-20` |
| 崩溃放量全量 `crash-amplification.sh` | `<家目录>/library-by-conclusion-kind`（一份库装全部分项），没有整道标记 | 块键同上（流程）；结论按（块键，结论种类，版本）存：GPU 结论版本 = GPU 判器闭包；CPU 结论版本 = CPU 判器闭包 ∪ 设点文件与它 mod 进来的测试模块出发的闭包；观察者 = 观察者代码闭包 + 另读输入；版本另拼工具链 | 按点、按结论种类：对应闭包变 ⇒ 那一种结论整库重判 | 已抓到那一处的同族；CPU 结论多并进设点文件的闭包，见清单 8 | 量的：`scripts/crash/crash-amplification.sh:2`、`:7`、`:31-33`；`scripts/crash/crash-amplification-lib.sh:106-109`；`crash_identity.rs:581-600`、`:647-655` |
| `scripts/crash/crash-point-replay.sh --full` | 删这几条用例的全绿标记、带 `--start-over`；放量部分换一份新的空库 | — | 全部从头 | 按设计是「全量 = 清空复用」；若用户口径是任何重型都要能剪，这一档要用户另判（推的） | 量的：`scripts/crash/crash-point-replay.sh:16-17` |

## 2 checker-tier-qemu-device-streams

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| device-streams（六档虚机：direct、skip-new-pool-file-creation-barrier、page-cache、file-overwrite、second-instance、raise-rollback-floor） | 无库；只有整道全绿标记（git common-dir，本轮 `ls` 数到 2 份） | 第 0 节整道键：路径 `crates/ Cargo.toml Cargo.lock scripts/ research/results/ .claude/kb/layout/01-first-txn.md` + 整张登记表 + 阶段脚本 + 工具链与构建环境；第三列只有 command=、readwrite=、probe=，**没有 environment=** | 整道：上面任何一处变 ⇒ 六档全跑 | 不合理：① 整道一份标记，六档没有按档存；② 键过宽：整个 `scripts/`、整个 `research/results/`（任何一份新产物）、整张登记表、kb 布局表原文都进；③ 键过窄：QEMU 版本、内核镜像不进键，走②那条路（对暂存树比）时连 rustc 也不比 | 量的：`.claude/gate.d/checker-tier-qemu-device-streams.sh:64`、`:220-233`、`:452-457`；`stage-inputs.tsv:37`；`admission.py:1608-1609` |

## 3 checker-tier-lkmm

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| litmus-verdicts（herd7 判 litmus/ 下 8 份，含 4 份 nofence 对照） | 无库；只有整道全绿标记（本轮 `ls` 数到 3 份）与环境记录（本轮 `ls` 一份 `singlefs-gate-environment*` 都没有） | 第 0 节整道键：路径 `litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/` + 整张登记表 + 阶段脚本 + 工具链与构建环境 + environment=（`lkmm.sh --herd7-version`） | 整道：任一处变 ⇒ 8 份全判 | 不合理（轻度）：① 没有按 litmus 存；herd7 对一份 litmus 的判定只看那份文件、herd7 与内核 memory-model，`crates/` 只影响锚点那层静态核（推的）；② 键过宽：整个 `crates/`、rustc 工具链、整张登记表进键；③ 键过窄：内核树的 tools/memory-model 不进键（文件头自认要 SINGLEFS_GATE_FULL=1） | 量的：`.claude/gate.d/checker-tier-lkmm.sh:19-21`、`:58`、`:79-87`；`stage-inputs.tsv:38`；`.claude/scripts/lkmm.sh:340`；本轮 `ls litmus` 8 个文件 |

## 4 checker-tier-crates-mutation-replay

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 整道（短路一层） | git common-dir 整道全绿标记（模块写，本轮 `ls` 一份都没有） | 第 0 节整道键：路径见 `stage-inputs.tsv:45` + 整张登记表 + 阶段脚本 | 整道 | 合理：只是短路，真正的剪枝在下一层 | 量的：`scripts/mutation/crates-mutation-rows.py:2595-2600` |
| crates-mutation-rows（逐条变异） | git common-dir `singlefs-mutation-rows/<判法版本>/`（本轮 `ls` 有 3 个判法版本目录） | 判法版本 = 登记行（键与路径）+ 阶段脚本 + 本模块（4044 行）+ `mutation-prune.py` + `lib_libtest_result_lines.py` + `lib_verdict_library.py` + 内存包装，**全按原文字节**，再加构建环境；行键 = sha256(六段原样 + 每条内存上限 + 每条限时 + 工具链)；复用判据另比快照与覆盖：被变异的函数、执行过的函数、函数外改动 | 判法版本变 ⇒ 整张重跑（换一格目录，旧记录一条不用）；行键变 ⇒ 那一行；源码变 ⇒ 按覆盖只重跑执行过改动函数的行 | 源码这一侧合理（按执行过的函数剪，七道里最细）。不合理：① 判法版本按原文字节，6 份脚本任何一处注释、自证代码改动都整张作废；② 每条内存上限、限时进行键，文件头自认「一变全表判定记录作废」，而「抓到」的结论与上限、限时无关（撞顶、超时单列不算抓到）；③ 只留最新 3 个判法版本目录，来回改超过 3 次就丢 | 量的：`crates-mutation-rows.py:58-61`、`:84`、`:86-101`、`:187`、`:609-618`；`.claude/gate.d/checker-tier-crates-mutation-replay.sh:83-87`、`:72-73`（撞顶、超时单列不算抓到） |

## 5 checker-tier-formal-verification（六格，按格 + 按条两层）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 按格（六格各一份标记） | git common-dir `singlefs-stage-green.checker-tier-formal-verification.sh:<格>.<指纹>`（本轮 `ls` 六格各 3–4 份） | 格键：这一格自己的登记行（键、路径、第三列）+ 登记路径源码摘要 + 阶段脚本 + 工具链与构建环境 + environment=（Verus 那四格）；整张登记表不进 | 按格 | 合理：七道里唯一不把整张登记表放进键的（格键的写法） | 量的：`.claude/gate.d/checker-tier-formal-verification.sh:36-46`；`admission.py:1590-1595`、`:2807-2809` |
| verus-toolchain-pinned、pinned-red-examples-complete | 不进按条库，每趟现判 | — | 每趟 | 合理（推的：只核登记与表，轻） | 量的：`checker-tier-formal-verification.sh:48-49` |
| verus-proof、verus-vacuity（每处 requires 一项）、verus-must-red-worlds（每行一项） | git common-dir `singlefs-formal-verdicts/<kind>/`（本轮 `ls` 有 must-red-worlds、proof、vacuity、pinned-red-examples-replay 四类） | 输入：核 crate `crates/singlefs-replay-prefix/` 整个目录逐文件原文 sha256（跳过必红世界表）、`verus-pins.tsv`、工具链、限时；必红世界每行另加自己那一行；判法版本 = `verus-proof-check.py` 与 `lib_verdict_library.py` 两份原文 sha256 | 共用输入任一样变、或判法版本变 ⇒ 整格每一项都现判 | 不合理（轻度）：① 核 crate 与判法脚本都按原文字节，改一行注释整格全判；② 空真按 requires 分项，键却是整个 crate（推的：Verus 模块化验证，一处 requires 的空真只看那个函数与它用到的 spec） | 量的：`checker-tier-formal-verification.sh:47-53`；`scripts/gate/verus-proof-check.py:793`、`:801-813`；`scripts/crash/lib_verdict_library.py:52-76` |
| pinned-red-examples-replay（钉表每行一项） | 同上 `singlefs-formal-verdicts/pinned-red-examples-replay/` | 每行：这一行 22 段、档、引的变异行；共用：e165 装置的整个 cargo 依赖闭包逐文件原文 sha256 + 仓根 Cargo.toml、Cargo.lock + 工具链；判法版本 = `e165-pin-red-examples.py` 与判定库两份原文 sha256 | 闭包里任何一个文件变（含 core 任何一处）⇒ 整格每一行都现判 | 推的：依赖实现是真依赖（重放的就是今天的实现），整格作废方向是多跑；可按覆盖收窄，优先级低。判法版本按原文字节同上 | 量的：`scripts/experiments/e165-pin-red-examples.py:93-97`、`:1162-1179` |

## 6 checker-tier-broad-mutation（门禁只读结论；驱动 `scripts/mutation/broad-mutation.py` 与规则库 `mutation-prune.py`）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 门禁两格（library-version-is-staged-tree、gate-verdict） | git common-dir 整道全绿标记（本轮 `ls` 一份都没有） | 第 0 节整道键：路径见 `stage-inputs.tsv:46` + 整张登记表 + 阶段脚本 + environment=（`--tool-versions`） | 整道 | 合理：门禁本身只读库目录、不跑变异（轻），整道粒度够用（推的） | 量的：`.claude/gate.d/checker-tier-broad-mutation.sh:51-56`、`:827-835`、`:873-876` |
| 驱动：版本目录 | `${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}/versions/<指纹>/` | 这一版指纹 = sha256(源码指纹（导出范围 crates/、Cargo.toml、Cargo.lock、litmus/、.cargo/ 逐文件 sha256）、rustc -Vv、判法版本 JUDGE_RULES_VERSION、全局键 sha256) | 每次换树就是新一版；新版里的条靠结论表沿用 | 合理：版本只是容器，剪枝在结论表 | 量的：`scripts/mutation/broad-mutation.py:49-56` |
| 驱动：结论表 `conclusions.tsv`（跨版本沿用） | 同一库目录，只追加 | 条键 =（文件、限定名、函数内相对起止、替换文）；沿用要：全局键相同、上次是抓到（无效、没红每次重判）、变异所在 fn 项与函数外部分指纹相同、宏与 include 依赖指纹相同、抓到它的那条测试的 fn 项指纹相同。全局键 = rules 判法版本、toolchain、**导出树里每一份 Cargo.toml、Cargo.lock 与 .cargo/ 下的文件**、构建环境、nextest / cargo-mutants / llvm-profdata 版本、排除表、限时参数、升档常量 | 全局键任一行变 ⇒ 整版每条重判；其余按条 | 不合理：① 键过宽：任何一个 crate 的 Cargo.toml、Cargo.lock、整个 .cargo/ 进全局键；文件头给的「等价变异写进 .cargo/mutants.toml 的 exclude_re」出路本身就让整版重判；排除表只删条，删一条不该让别的条作废（推的）。② 键过窄（会漏判）：「抓到」沿用只比抓到它的那条测试自己的 fn 项，不比测试调到的辅助函数、也不比从测试到被变异函数之间的调用路径；那几处改了，测试可能已经杀不掉这条，却照沿用（推的，代码量的）。③ 判法版本是手写串，驱动判法改了忘抬就照沿用（推的） | 量的：`broad-mutation.py:57-66`、`:158-160`、`:164-165`、`:178-185`；`scripts/mutation/mutation-prune.py:17-25`、`:640-658` |

## 7 checker-tier-research-build-and-replay：experiment-replay 与 mutation-tables-compile 两格

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| experiment-replay（整格一层） | **没有全绿标记**：阶段脚本里 `stage-marker-write` 出现 0 次；只靠对上次整轮全绿那棵暂存树比 | 整道键（键是整道 `checker-tier-research-build-and-replay.sh`，不是格键）：路径 `crates/ Cargo.toml Cargo.lock scripts/experiments/ scripts/ research/results/` + 整张登记表 + 阶段脚本；另一问 change-touches-crates 前缀 `crates/ scripts/experiments/ scripts/ research/results/` | 整格：上面任何一处变、或上一轮整轮没全绿（staged-green 没前移）⇒ 交下一层 | 不合理：① 没有标记，主 agent 在工作区单跑绿过也替整轮作不了保；② 键过宽（整个 scripts/、research/results/、整张登记表）；③ 走对暂存树比那条路不比工具链（rustc 升级整格被跳过，下一层按行的工具链键也不起作用） | 量的：本轮 `grep -c 'stage-marker-write'` 输出 `0`；`.claude/gate.d/checker-tier-research-build-and-replay.sh:29-38`、`:337-341`；`stage-inputs.tsv:48`；`admission.py:1608-1609` |
| experiment-replay（按行一层，`scripts/experiments/replay.sh`） | `${REPLAY_VERDICT_LIBRARY:-<git common-dir>/singlefs-replay-verdicts}/<键>/`（本轮 `ls` 默认位置**不存在**） | 行键 = 装置源码（[[bin]] 的 path 顺着 mod / #[path] / include! 带进来的，驱动函数写 @ 的取 replay.sh 函数闭包原文；`-p <包>` 取 cargo 依赖闭包）+ 公共部分（research 的 Cargo.toml、Cargo.lock、lib.rs 与它带进来的）+ 复跑表这一行五列 + 判一行的那几个函数原文 + 留存产物 sha256 + 工具链与构建环境；逐文件源码摘要 | 按行 | 基本合理：判法版本只取判一行的那几个函数，窄而对。不合理（轻度）：① `lib.rs` 改了每一行都重跑（文件头自认）；② 键过窄：驱动运行期拼出来的路径、子进程再读的文件、GPU 开关不进键（文件头自认） | 量的：`scripts/experiments/replay.sh:33-38`、`:55`；`admission.py:67-80` |
| mutation-tables-compile（整格一层） | 同上：没有全绿标记，只对暂存树比 | 格键 `…:mutation-tables-compile`：路径 `scripts/mutation/tables/ scripts/experiments/ scripts/mutation/mutate.sh scripts/crash/lib_verdict_library.py scripts/crash/crash-amplification-lib.sh research/results/` + 这一格登记行 + 阶段脚本 | 整格 | 不合理（轻度）：没有标记；整个 `research/results/` 进键（登记注释说是装置 include_str! 读留存产物，宁宽勿窄） | 量的：`checker-tier-research-build-and-replay.sh:46-47`、`:455-457`；`stage-inputs.tsv:49` |
| mutation-tables-compile（按表一层） | `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-verdicts/`；gate.sh 没设时把 GATE_CROSS_RUN_TMPDIR 设成 `${TMPDIR:-/tmp}`；本机 TMPDIR 没设，即 `/tmp`（本轮 `ls` 不存在；开机于 2026-10-11） | 输入：表、源文件、源文件 include_str! / #[path] 引的文件、research 的 Cargo.toml、Cargo.lock、lib.rs、mutate.sh、工具链，**全按原文 sha256**；判法版本 = 整份阶段脚本（四格合一份，564 行）+ `lib_verdict_library.py` 两份原文 sha256 | 判法版本变 ⇒ 整格每张表重编；lib.rs、Cargo.lock 变 ⇒ 每张表重编 | 不合理：① 库在 /tmp，重启即丢（Rust 那一侧的崩溃判定库对这种目录直接拒绝开库）；② 判法版本是整份阶段脚本原文，改单测格、挂钟格、复跑格的任何一个字（含注释）都让这一格整库作废 | 量的：`checker-tier-research-build-and-replay.sh:48-52`、`:411-421`、`:458-460`、`:504`；`.claude/singlefs-ai-sop/scripts/gate.sh:170-177`；`crates/singlefs-checker-tier/src/crash_library.rs:55-80`；本轮 `echo $TMPDIR` 输出 `TMPDIR=<unset>`、`uptime -s` 输出的开机日期是 2026-10-11 |

## 8 崩溃判定库里与已抓到那一处同族的三点（只按文件头与键的定义读，不重查已抓到的那一处）

| 点 | 内容 | 依据 |
|---|---|---|
| 判器闭包摘要另拼 Cargo.lock 与收到的各 crate 的 Cargo.toml 原文 | 任何一个依赖的版本抬一下（Cargo.lock 变），判法版本就变，整库判定作废；与「只抬一个表上限常量」是同一族、范围更大 | 量的：`crates/singlefs-checker-tier/src/crash_identity.rs:510-513` |
| CPU 结论的版本另并进「设这个点的文件与它 mod 进来的测试模块出发的闭包」 | 设点文件出发的闭包顺着 use 收到被测实现（停止表只挡判器与流水线五份）；流程的变化已经在块键（写表、父输出）里，这一并让被测实现的改动也作废 CPU 结论（推的：没逐个列闭包收到了哪些文件） | 量的：`crash_identity.rs:11-12`、`:586-589` |
| 文件头与代码不一致 | 文件头写「复用键罩三样属性：写表、父输出、判法代码……连同 Cargo.lock 与工具链」，代码里复用键只有写表与父输出两样（判法在判定行上另分） | 量的：`crash_identity.rs:6-8` 对 `:92-115` |

## 不合理清单

| 号 | 门禁 / 格 | 哪一类 | 问题 | 最小改法（方向与落点文件） | 依据 |
|---|---|---|---|---|---|
| 1 | 整道键（崩溃点重放快档、QEMU、LKMM、crates 变异整道、广谱变异门禁、研究复跑 experiment-replay） | 键过宽 | 整张 `stage-inputs.tsv` 进整道键，改任何别的行或注释（加一个实验行也算）整道作废 | 整道键也照格键的写法，只放本道自己的登记行（键、路径、第三列），不放整张表：落点 `scripts/gate/admission.py` 的 `gate_reuse` 与 `stage_fingerprint`；同步改 `stage-must-run.sh`、`stage-inputs.tsv` 文件头那两句 | 量的 `admission.py:1590-1592`、`:2808`；`stage-inputs.tsv:31`；第 9 节 ① |
| 2 | crates 变异、形式化四类按条库、变异表只编译 | 键过宽 | 判法版本按原文字节算（注释、自证代码、文件头也算），改一个字整库作废；整道键那边早已按源码摘要 | 判法版本的各份脚本改用 `scripts/gate/lib_source_digest.py` 的源码摘要；再进一步照崩溃枚举用例的判法摘要（ast 闭包，`admission.py:2716`）或 replay.sh 的 REPLAY_JUDGING_FUNCTIONS 只取判法函数。落点 `scripts/mutation/crates-mutation-rows.py` 的 `judge_version`、`scripts/gate/verus-proof-check.py`、`scripts/experiments/e165-pin-red-examples.py` 的 `judge_version`、研究复跑阶段脚本里 mutation-tables-compile 那段 | 量的 `crates-mutation-rows.py:609-618`、`verus-proof-check.py:793`、`e165-pin-red-examples.py:1168`、`checker-tier-research-build-and-replay.sh:421`、`:504`；第 9 节 ③ |
| 3 | crates 变异 | 键过宽 | 每条内存上限、限时进行键，一变全表作废；而「抓到」与上限、限时无关（撞顶、超时单列）；广谱变异那边已明写 `--memory-cap`、`--timeout-floor` 不进全局键 | 上限、限时从行键里拿掉，只留在「没红、撞顶、超时」那几档的记录里（它们本来就不复用）。落点 `crates-mutation-rows.py` 的行键与 `reuse_decision` | 量的 `crates-mutation-rows.py:61`、`:187`；`checker-tier-crates-mutation-replay.sh:72-73`；`broad-mutation.py:64` |
| 4 | 广谱变异结论表 | 键过宽 | 全局键收每一份 Cargo.toml、Cargo.lock、整个 .cargo/、排除表：改一个 crate 的清单、写一条等价变异排除，整版每条重判 | 排除表与 `.cargo/mutants.toml` 的 exclude_re 移出全局键（只删条，不改别的条的结论）；Cargo.toml 按变异所在包与它的依赖闭包收窄进条键。落点 `scripts/mutation/broad-mutation.py` 的 `global_key_lines`、`scripts/mutation/mutation-prune.py` 的 `carry_decision` | 量的 `broad-mutation.py:57-63`、`:164-165`、`:178-181` |
| 5 | 广谱变异结论表 | 键过窄（会漏判） | 「抓到」沿用只比抓到它的测试自己的 fn 项，不比测试调到的辅助函数与调用路径 | 沿用条件改成 crates 变异那一套：比覆盖记录里那条测试执行过的函数集合（版本内已经插桩跑覆盖，C2）。落点 `mutation-prune.py:640-658` 与驱动记结论的那一段 | 量的 `mutation-prune.py:23`、`:656-657`；对照 `crates-mutation-rows.py:94` |
| 6 | 广谱变异结论表 | 键过窄 | 判法版本是手写串 JUDGE_RULES_VERSION，驱动判法改了不抬就沿用旧结论 | 判法版本改成驱动与规则库判法函数的摘要（同第 2 条的做法），手写串留作人看的名字。落点 `broad-mutation.py` | 量的 `broad-mutation.py:183-185` |
| 7 | 崩溃判定库 | 键过宽 | 判器闭包摘要拼了 Cargo.lock 与各 crate 的 Cargo.toml，依赖一动整库作废 | 只取闭包里收到的 crate 在锁里那几项。落点 `crates/singlefs-checker-tier/src/crash_identity.rs` 的闭包算法（`judging_closure.rs`，归已抓到那一处的同一批） | 量的 `crash_identity.rs:510-513` |
| 8 | 崩溃放量 CPU 结论 | 键过宽（推的） | CPU 结论版本并进设点文件出发的闭包，被测实现改了 CPU 结论也作废 | 设点文件闭包只收判法读的部分（停在被测实现门外），或拆成单独一把版本。落点 `crash_identity.rs` 的 `ConclusionVersionRequest::CpuJudge` | 量的 `crash_identity.rs:586-589` |
| 9 | 崩溃判定库文件头 | 文件头错 | 文件头说复用键罩判法代码，代码里没有 | 改文件头那三行。落点 `crash_identity.rs:6-8` | 量的 `crash_identity.rs:6-8`、`:92-115` |
| 10 | 崩溃点重放快档两格 | 没有库、整道一变全跑 | 不标 ignored 的用例与带特性那一趟每趟全跑，经引擎判的用例在快档里不进任何长期库 | 带 `verdict-store` 特性、经引擎判的快档用例开一类快档自己的长期库（`crash_library.rs` 的 CrashLibraryClass 加一类）；其余按测试目标记标记、键取该目标的闭包。落点 `.claude/gate.d/checker-tier-crash-point-replay.sh:53-55` 与 `crash_library.rs` | 量的 `checker-tier-crash-point-replay.sh:55`、`:435` |
| 11 | QEMU | 整道一变全跑、键过宽 | 六档一份标记；整个 scripts/、research/results/ 进键 | 按档各记一份标记（键 `<阶段>:<档>`，照形式化的格键写法），路径收窄到虚机装置与复跑脚本、只取 E142 那一份产物。落点 `.claude/gate.d/checker-tier-qemu-device-streams.sh` 与 `stage-inputs.tsv:37` | 量的 `checker-tier-qemu-device-streams.sh:64`、`:455`；`stage-inputs.tsv:37` |
| 12 | QEMU、崩溃点重放快档、研究复跑两格 | 键过窄 | 对暂存树比那条路不比工具链；QEMU 版本与内核镜像任何一条路都不进键 | 这几行第三列补 `environment=`（`qemu-system-x86_64 --version` 与内核镜像 sha256、`rustc -V`）；或让 `gate_reuse` 那条路也比 toolchain_line。落点 `stage-inputs.tsv:36-37`、`:48-49`，`admission.py:1608-1609` | 量的 `admission.py:1608-1609`；`stage-must-run.sh` 文件头「管不到的」 |
| 13 | LKMM | 整道一变全跑、键宽且窄 | 8 份 litmus 一份标记；整个 crates/ 与 rustc 进键；内核 memory-model 不进 | 按 litmus 存 herd7 判定（键：那份 litmus、herd7 版本、memory-model 的 .cat/.bell 摘要），锚点静态核每趟现判（轻）。落点 `.claude/scripts/lkmm.sh` 与 `stage-inputs.tsv:38` | 量的 `checker-tier-lkmm.sh:19-21`；`stage-inputs.tsv:38` |
| 14 | 研究复跑 experiment-replay、mutation-tables-compile | 没有整格标记 | 只有对上次整轮全绿暂存树比一条路；experiment-replay 用的还是整道键 | 两格各写格标记（照形式化 `stage-marker-write <阶段>:<格>`），experiment-replay 改登记成格键 | 量的 本轮 grep 输出 `0`；`checker-tier-research-build-and-replay.sh:337-338` |
| 15 | mutation-tables-compile 按表库 | 库会丢 | 库在 /tmp，重启即空 | 挪到 git common-dir（与其余几份库一处）。落点 `checker-tier-research-build-and-replay.sh:458` | 量的 `:458`；`gate.sh:176` |
| 16 | mutation-tables-compile 按表库 | 键过宽 | 判法版本是整份四格合一的阶段脚本原文 | 判法版本只取这一格的函数与 `mutate.sh --compile-only`（同第 2 条）。落点同上 `:504`、`:521` | 量的 `:421`、`:504` |
| 17 | 复跑按行 | 键过宽（轻） | research 的 lib.rs 改了每一行都重跑 | 先不改（文件头自认、推的代价小）；要改就按 bin 带进来的 lib.rs 项收窄。落点 `admission.py` 的 `replay_row_input_paths` | 量的 `admission.py:75` |
| 18 | 复跑按行 | 键过窄（轻） | GPU 开关、运行期拼出来的路径不进键 | GPU 开关进行键（ENABLE_GPU 的值一行）。落点 `scripts/experiments/replay.sh` 与 `admission.py:67-80` | 量的 `replay.sh:55`；`admission.py:79-80` |
| 19 | 形式化 Verus 按条 | 键过宽（轻） | 核 crate 整个目录原文进每一项的键；空真按 requires 分项却共用整个 crate | 先换源码摘要（第 2 条）；空真按 requires 所在函数与它用到的 spec 收窄（推的，Verus 模块化验证）。落点 `verus-proof-check.py:801-813` | 量的 `verus-proof-check.py:807` |
| 20 | 形式化钉表重放 | 键宽（推的，方向多跑） | e165 装置整个依赖闭包原文进每一行的键 | 优先级低；要改就照 crates 变异按覆盖记执行过的函数 | 量的 `e165-pin-red-examples.py:1172-1179` |
| 21 | crates 变异判定库 | 库会丢（轻） | 只留最新 3 个判法版本目录，来回改超过 3 次旧的被删 | 第 2 条做了之后判法版本少变，这条可以不改；要改就按「最近被用到」留 | 量的 `crates-mutation-rows.py:84` |

## 最该先改的三条

1. **第 1 条（整张登记表进整道键）**。理由：一处改动牵五道重型（QEMU 六档虚机、崩溃点重放快档整趟 cargo test、LKMM、研究复跑整格、crates 变异整道短路），而改登记表是最常见的改动之一（2026-09-15 以来 194 次提交里 12 次，量的）；改法只动 `admission.py` 两处，格键那套写法已经在形式化六格上跑着（量的 `admission.py:1590-1595`），风险小。
2. **第 2 条（判法版本按原文字节）**。理由：三份库（crates 变异整表、形式化四类按条、变异表只编译）一次注释改动就整库作废，与用户说的「每次重跑是浪费」正面冲突；仓里已有两种更窄的现成写法（源码摘要库、ast 判法摘要），换过去不用新造判法。
3. **第 5 条（广谱变异「抓到」沿用的键过窄）**。理由：这一条不是多跑，是会漏判：测试的辅助函数或调用路径改了而照旧沿用「抓到」，门禁可能假绿（推的，代码量的 `mutation-prune.py:640-658`）；项目口径是准确率优先，漏判比多跑更该先修，而版本内覆盖已经在跑，执行过的函数集合有现成来源。

其次是第 4 条（广谱变异全局键收 .cargo/ 与排除表，写一条等价排除就整版重判）与第 3 条（crates 变异上限、限时进行键）。

## 9 校验（本轮跑过的命令与原样输出）

① 登记表注释行里多一个空格，源码摘要变不变（`.tsv` 取原文）：

```
$ nice -n 19 python3 -I -c '…读 scripts/gate/lib_source_digest.py，对 stage-inputs.tsv 原文与「# 宁宽勿窄」那一行多一个空格的副本各算 digests_of…'
classify: raw
orig: ('raw', 'ffc34c4e99e3f0316b7429620a77b0ce3a7e9564f2e5b9f397de6d827c2231f2')
one space added in a # comment line: ('raw', '57c8dd7fe38e192feb1075c8305d959a83b5a06c089624234520aa936a6f2dca')
same: False
```

② 哪几道阶段脚本里写全绿标记（研究复跑为 0；crates 变异那 1 处在文件头，真写在 `crates-mutation-rows.py:2595-2600`；崩溃点重放写的是逐用例标记）：

```
$ grep -c 'stage-marker-write' .claude/gate.d/checker-tier-{lkmm,qemu-device-streams,crates-mutation-replay,research-build-and-replay,formal-verification,broad-mutation,crash-point-replay}.sh
.claude/gate.d/checker-tier-lkmm.sh:2
.claude/gate.d/checker-tier-qemu-device-streams.sh:2
.claude/gate.d/checker-tier-crates-mutation-replay.sh:1
.claude/gate.d/checker-tier-research-build-and-replay.sh:0
.claude/gate.d/checker-tier-formal-verification.sh:2
.claude/gate.d/checker-tier-broad-mutation.sh:4
.claude/gate.d/checker-tier-crash-point-replay.sh:0
```

③ 判法版本那几份代码有没有用源码摘要（全 0，即全按原文）：

```
$ grep -c 'source_digest\|lib_source_digest\|source_digests=True' <六份>
.claude/gate.d/checker-tier-research-build-and-replay.sh:0
scripts/experiments/e165-pin-red-examples.py:0
scripts/gate/verus-proof-check.py:0
scripts/mutation/crates-mutation-rows.py:0
scripts/mutation/broad-mutation.py:0
scripts/crash/lib_verdict_library.py:0
```

④ git common-dir 里此刻有哪些标记与库（独立于读代码的另一条路：研究复跑、crates 变异、广谱变异都没有整道标记，复跑按行库与环境记录不存在）：

```
$ cd "$(git rev-parse --git-common-dir)" && ls -d singlefs-stage-green.* | sed 's/\.[0-9a-f]\{64\}$//' | sort | uniq -c
      4 singlefs-stage-green.checker-tier-formal-verification.sh:pinned-red-examples-complete
      4 singlefs-stage-green.checker-tier-formal-verification.sh:pinned-red-examples-replay
      3 singlefs-stage-green.checker-tier-formal-verification.sh:verus-must-red-worlds
      3 singlefs-stage-green.checker-tier-formal-verification.sh:verus-proof
      3 singlefs-stage-green.checker-tier-formal-verification.sh:verus-toolchain-pinned
      3 singlefs-stage-green.checker-tier-formal-verification.sh:verus-vacuity
      3 singlefs-stage-green.checker-tier-lkmm.sh
      2 singlefs-stage-green.checker-tier-qemu-device-streams.sh
ls: cannot access 'singlefs-gate-environment*': No such file or directory
ls: cannot access 'singlefs-replay-verdicts': No such file or directory
```
（同一次 `ls`：`singlefs-crash-case-green` 22 份，`singlefs-mutation-rows/` 下 3 个判法版本目录，`singlefs-formal-verdicts/` 下 must-red-worlds、pinned-red-examples-replay、proof、vacuity。）

⑤ 变异表只编译那份库：

```
$ echo "TMPDIR=${TMPDIR:-<unset>}"; ls -d /tmp/singlefs-mutation-tables-compile-verdicts
TMPDIR=<unset>
ls: cannot access '/tmp/singlefs-mutation-tables-compile-verdicts': No such file or directory
```

⑥ 登记表改动频度：`git log --since=2026-09-15 --oneline -- .claude/gate.d/stage-inputs.tsv | wc -l` 输出 `12`；同期 `git log --since=2026-09-15 --oneline | wc -l` 输出 `194`。

## 推翻条件

- 第 1 条：整道阶段的 `stage_fingerprint` 在只改登记表别的行之后算出同一个指纹 ⇒ 推翻（今天读到 `admission.py:2808` 把整张表加进路径，校验 ① 证明表的摘要随注释变）。
- 第 2 条：在 `crates-mutation-rows.py` 文件头只改一行注释，`crates-mutation-rows.py judge-version <根>` 打的数不变 ⇒ 推翻（今天读到 `:610-615` 是 `read_bytes` 原文、校验 ③ 为 0）。这一步要起 cargo（工具链进构建环境），本轮没跑。
- 第 5 条：在一条「抓到」的变异上，只改抓到它的测试调到的辅助函数（测试 fn 项与被变异函数都不动），`broad-mutation.py --changed-since … --list-items` 把它列成「现判」⇒ 推翻。本轮没跑（要起驱动）。
- 第 15 条：gate.sh 起这一道时 `GATE_CROSS_RUN_TMPDIR` 指到 /tmp 之外的长期目录 ⇒ 推翻（今天读到 `gate.sh:176` 没设时取 `${TMPDIR:-/tmp}`，本机 TMPDIR 没设）。
- 第 14 条：研究复跑两格经别的脚本写 `singlefs-stage-green.checker-tier-research-build-and-replay.sh*` ⇒ 推翻（校验 ② 为 0、④ 里没有这种文件）。

## 没查到 / 判不了

- 复跑按行库（`singlefs-replay-verdicts`）默认位置不存在：是没跑过、还是 `REPLAY_VERDICT_LIBRARY` 指到了别处，没查。
- 崩溃放量的枚举计划哈希随展开深度怎么变（抬一档展开，是只多判新加的态，还是那几个点整点重判）：只读到「进计划哈希的名字要分得开两种展开方式」（`crates/singlefs-checker-tier/src/crash.rs:1941`），没往下查。
- 各道一趟实际多长、作废一次浪费多少挂钟：没量；崩溃点重放快档、QEMU 六档、LKMM 的单趟时长都没查到产物。
- `--full`（`crash-point-replay.sh --full`、`--start-over`）清空复用是设计如此；按用户口径要不要也剪，判不了，交用户。
- CPU 结论设点文件闭包实际收到了哪些被测实现文件：没逐个列（第 8 条标推的）。

## 没做什么

- 没起 cargo、门禁阶段、测试二进制、驱动与自证；推翻条件里要起它们的那几步没跑。
- 没改仓内任何文件；没修，改法只写方向与落点。
- 已抓到的那一处（判器闭包整库作废）没重查。
- 轻型三条不在射程内，没看。
- 没建仓副本与编译目录；草稿目录里只有这份报告与派发闸放的 `prompt.md`、`.owner`、`.writer-lock`。
