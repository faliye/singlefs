# 重型门禁复用键审计（路子二：按判定库的实现读键的算法）

日期 2026-10-15；被读的树是工作区 HEAD c1e9b324（工作区里 crates/、scripts/ 下没有未提交改动，`git status` 开头那份快照只列了 .claude/、records/、research/ 下的文件）。
只读：没起 cargo、门禁、测试二进制，没动任何判定库；只用 grep、sed、git log 读代码。每条结论后面标「量的（文件:行）」或「推的」。
「量的」在这里是指读到了算键或比键的那一行代码本身；没有一条是跑出来的观测（只读，起不了）。

读法：每个库先找「算键」那几行（键由哪几样拼成），再找「比键」那一行（复用的判定是哪一个等不等的比较），最后看键里每一样变了会作废多大一片。

## 1 崩溃点重放（门禁 checker-tier-crash-point-replay，含 --full）与崩溃放量全量（scripts/crash/crash-amplification.sh）

### 1.1 一张表

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 外层：每条崩溃枚举用例的全绿标记（--full 逐条、快档 crash-case-replay 格只核它） | git common-dir 里 `singlefs-crash-case-green.<用例名>.<指纹>`，一条用例一个文件 | 整个 crates/ 与 Cargo.toml、Cargo.lock 下 git 列得出的文件（逐文件源码摘要，注释与空白不算），减去别的测试目标独占的测试文件、mutations.tsv、没人点名的 src/bin/；加 admission.py 里崩溃用例那一段判法的 ast 摘要、工具链、构建环境、双机驱动脚本（登记了分片的）、键的写法、这一条的登记行 | 整条用例：crates/ 下任何一份非测试代码的词法改了，这条用例整条重跑 cargo test（编译、录入段、问库）；判段由内层库剪 | 外层宽，但内层有块级库兜着，代价是编译加录入段，可以接受 | admission.py:2158-2176（清单），2168-2169（减去的两类），2133-2155；admission.py:2434（check_crash_case_marker）；stage-inputs.tsv:53-66（登记 crates/ 整个） |
| 内层：崩溃枚举用例库 | `<家目录>/crash-state-enumeration-library`（家目录默认 `~/.local/share/singlefs/crash-amplification`） | 块键（见 1.2）‖ 结论种类 ‖ 结论版本（见 1.3） | 见 1.4：判器闭包一变，GPU 那一种整库作废；设点文件或它 mod 进来的测试模块一变，那份文件的 CPU 结论与观察者结论整份作废 | 判器闭包那一半不合理（已知的那一处）；另有几处键过宽，见清单 | crash_library.rs:132；crash_state_enumeration.rs:1046、1234；verdict_store.rs:3119-3137 |
| 内层：给定状态库（随机历史两项、c561、几条经引擎的用例） | `<家目录>/given-states-library` | 块键：输入串 = 项名、种子与规模串、表名、基础镜像摘要、容器容量旋钮、每次写、接缝、每个状态的持久集合与自称版本；节点名 = `<设点文件>::<项名>::<表名>`；计划哈希 = 态数、写数、接缝数；块起点恒 0（一张表一块）‖ 结论种类 ‖ 版本 | 一张表一块；种子与规模串、项名、设点文件路径任一样变，整项每张表都换键 | 种子与规模串进块键不合理（键过宽），见清单 | crash_amplification.rs:12094-12145（12095 输入串第一行、12128 节点名）；11614-11622（文件头说种子与规模进块键） |
| 放量：library-by-conclusion-kind | `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind`，一份库装全部分项 | 同崩溃枚举用例库：块键 ‖ 结论种类 ‖ 版本 | 同 1.4；没有分项级的外层标记，每趟每项都重跑录入段（`admission: always`） | 判器闭包那一半不合理；录入段每趟重跑是推的代价，量级没量 | crash_library.rs:131；crash-amplification.sh:2、7；crash_amplification.rs:11859-11895 |
| 崩溃点重放入口 `--full` | 崩溃点重放部分带 `--start-over`（每条用例换 `<家目录>/crash-point-replay-start-over/<输入指纹>` 空库）；放量部分换 `<放量家目录>/crash-point-replay-full/<时戳>-<进程号>/library` 空库 | 不读库，照键写新库 | 全量：设计如此 | 合理（规则就这样定） | crash-point-replay.sh:16-17、567-568、631-632；admission.py:151 |

### 1.2 块键：每一样指到算它的那一行（量的）

块键 = 输入指纹 ‖ 节点名 ‖ 枚举计划哈希 ‖ 块起点（verdict_store.rs:22），实际取值：

- 输入指纹 = 这一步的复用键 `StepAttributes::reuse_key`（crash_amplification.rs:401），复用键 = sha256（「write_table_with_barriers=」带屏障的写表摘要、「parent_output=」父镜像摘要）（crash_identity.rs:103-118；两样在 crash_amplification.rs:604-611 算）。
- 节点名 = 身份串 `<设点文件的仓内路径>::<崩溃点名称>::<复用键十六进制>`（crash_identity.rs:71-78；crash_amplification.rs:614-615 拼，402 放进块键）。设点文件由 `file!()` 取（tests/common_crash_points/mod.rs:686-687）；崩溃枚举用例的设点文件就是用例文件、崩溃点名就是项名（crash_state_enumeration.rs:748-751）。
- 枚举计划哈希 = sha256（这个点管的每一段的「段在点内的序、态数或分片段的整域态数、展开方式名」，往下游找的发布，点的态数，判器另读的输入 judge_inputs）（crash_amplification.rs:624-733；judge_inputs 在 718-722）。分片段的块另把段序与编号高位拼进计划哈希（crash_amplification.rs:381-397）。
- 块起点 = 点内序号（不分片）或段内编号的低 64 位（分片）（crash_amplification.rs:353-378）。

比的地方：`conclusions_missing_for_block` 用（块键，种类，版本）拼成整把键字节去库里 `get`，有就算判过（verdict_store.rs:3119-3137），调用在 crash_amplification.rs:1678-1697。是逐字节全等，没有部分匹配。

### 1.3 结论版本：每一样指到算它的那一行（量的）

版本 = sha256（「singlefs conclusion version 1」、kind=种类、code=代码摘要、rustc=、cargo=、target=、extra_inputs=）（crash_identity.rs:604、635-656）。

代码摘要（judging_closure.rs:1060-1100）逐行拼：
- 每个入口一行 `entry`；GPU 结论的入口只有 `crash_judge_gpu.rs`（judging_closure.rs:1139），CPU 结论的入口是 `crash.rs` 加设点文件（judging_closure.rs:1141），观察者的入口是观察者代码文件，而崩溃枚举用例给的观察者代码文件就是用例文件（crash_state_enumeration.rs:1000、1009）。
- 停止表每一份一行 `stop`（judging_closure.rs:1070-1072；停止表 = 判器两份 ∪ 流水线七份，666-675、691-722）。
- 闭包收到的每份文件一行 `file <仓内路径> <raw|lexical> <摘要>`：被 `include_str!` 读的（五份 wgsl 内核）取原文 sha256，其余 .rs 取词法单元串的 sha256（judging_closure.rs:856-877）。词法只去空白与注释（judging_closure.rs:18-21），**不去 `#[cfg(test)]` 测试模块**：judging_closure.rs 里除了自己的 `mod tests`（1331）没有一处认 `cfg(test)`。
- `Cargo.lock` 一行，闭包碰到的每个 crate 的 `Cargo.toml` 一行，都取原文 sha256（judging_closure.rs:993-1006）。
- 工具链三行（crash_identity.rs:646-653）。

GPU 判器文件 `use crate::crash`（crash_judge_gpu.rs:30），所以 GPU 版本的闭包含 CPU 判器 `crash.rs` 与它够到的全部（量的：引用那一行；闭包具体收了多少份没跑出来，推的）。判器闭包至少含恢复、池级 checker 走树、格式、内存池（crash_identity.rs:918-931 那条用例断言它们在）。

每个点用哪一把：GPU 一把对全部点（crash_amplification.rs:12979），CPU 按设点文件各一把（12962-12976）；开跑前整张摘要表编译时与跑时比，有一个入口不等整趟不写库（crash_amplification.rs:12951-12958，crash_identity.rs:771-787）。

### 1.4 哪一类改动作废多大一片（每条指到判等不等的那个比较）

比较只有一处：（块键，种类，版本）整把键字节全等（verdict_store.rs:3127-3133）。所以下面每一条的粒度由「这一样进的是块键还是版本、版本对多少个点共用」决定。

| 改动 | 进了哪 | 作废粒度 | 量的 / 推的 |
|---|---|---|---|
| 判器闭包里任何一份 .rs 的词法变了（含它们 `#[cfg(test)] mod tests` 里加减一条单测、只改一个标识符名） | 版本的代码摘要 | GPU 那一种：全部点同一把（crash_amplification.rs:12979），整库作废；CPU 那一种：每份设点文件那一把都含判器闭包，也全部作废；观察者结论里拼着池级 checker 那一把（crash_amplification.rs:14082-14110），也全部作废 | 量的（算法与比较）；测试模块在 crash.rs 第 2785 行起到 3802 行、walk.rs 第 13233 行起到 14356 行、recovery.rs 第 5842 行起到 6332 行（`grep -n '^#\[cfg(test)\]'` 头一处到文件尾，当它是测试模块是推的） |
| 五份 wgsl 内核只改一行注释 | 版本（原文 sha256） | 同上，GPU 整库作废 | 量的 judging_closure.rs:866-868 |
| 判器闭包里一份文件改名或挪目录（内容不变） | 版本（每行带仓内路径） | 同上 | 量的 judging_closure.rs:1073-1085 拼 `file <路径>` |
| `Cargo.lock` 任何一处变了（给与判器无关的 crate 加依赖也算）、闭包碰到的任一 crate 的 `Cargo.toml` 原文变了（只改注释也算） | 版本 | 三种结论全部、整库作废 | 量的 judging_closure.rs:993-1006 |
| 停止表或流水线表加减一份 | 版本（每份一行 stop） | 整库作废 | 量的 judging_closure.rs:1070-1072 |
| rustc / cargo / 目标三元组变了 | 版本 | 整库作废 | 量的 crash_identity.rs:646-653；合理 |
| 设点的测试文件（或它 `mod` 进来的公共测试模块）词法变了，哪怕改的是同一份文件里别的快档用例 | 那份文件的 CPU 版本与挂在它上面的观察者版本 | 用这份文件设点的全部点：CPU 结论、观察者结论作废；GPU 结论不动 | 量的 judging_closure.rs:1141；crash_state_enumeration.rs:1000、1009 |
| 设点文件改名、崩溃点名或项名改名（流程一字不变） | 块键的节点名 | 那份文件 / 那个点名下的全部块，三种结论全作废 | 量的 crash_identity.rs:71-78；crash_amplification.rs:402、614-615；crash_state_enumeration.rs:748-751 |
| 流程里一步的写或它之前的镜像变了 | 块键的输入指纹 | 这一步与它之后共前缀的点（父镜像传下去） | 量的 crash_amplification.rs:604-615；合理，前缀树本来就这样 |
| 一个点里某一段的展开方式或态数变了（例如展开深度从 6 抬到全量后多展开了一段） | 计划哈希（整点一份） | 这个点的全部块，连同没变的那几段 | 量的 crash_amplification.rs:624-733；不分片的段块起点是点内序号，前面一段态数变了后面全挪位（推的） |
| 判器另读的流程输入（版本表这一类）变了 | 计划哈希 | 这个点 | 量的 crash_amplification.rs:718-722；合理 |
| 给定状态：种子数、规模参数、权重写法串变了（已有的表内容不变） | 块键的输入串 `seeds_and_scale=` | 这一项的每一张表 | 量的 crash_amplification.rs:12095；串的写法例 crash_injection_campaign.rs:81-91 带 seed_count |
| 配置从 GPU 换成 CPU（或反过来） | 结论种类 | 这一趟只认自己那一种（crash_amplification.rs:11853-11857；crash_state_enumeration.rs:1234-1235），另一种判过的一块都不借 | 量的；按 J8 分开存是定过的，借不借另判 |
| 流水线七份（放量、派活、存储、进度、库、枚举入口）变了 | 都不进 | 不作废 | 量的 judging_closure.rs:691-722；合理 |
| 判器闭包之外、设点文件闭包之外的代码变了 | 都不进（外层标记失效，内层照复用） | 只重跑编译与录入段 | 量的 admission.py:2158-2176 与上面那一处比较；合理 |

### 1.5 对照口径：不合理的地方

- 判器闭包一变整库作废（已抓到的那一处，不重复）。按实现补三点它比「判器改了」还宽：闭包里的测试模块进摘要；闭包文件只改名也换版本；`Cargo.lock` 与各 `Cargo.toml` 原文进摘要（只改注释也换）。量的，见 1.4 前四行。
- 设点文件与标签进键：节点名带设点文件路径与崩溃点名（crash_amplification.rs:402），CPU 与观察者版本带整份设点文件（judging_closure.rs:1141、crash_state_enumeration.rs:1000），同一份测试文件里改一条快档单测，整条流的 CPU 与观察者结论作废。量的。
- 给定状态的种子与规模串进块键：表的内容（写、持久集合、基础镜像、旋钮）已经全在输入串里（crash_amplification.rs:12094-12122），种子数从 N 加到 M，前 N 张表内容不变却全换键。量的（键）；表内容随种子确定，推的。
- 计划哈希按整个点算：一个点里多展开一段，整个点的块都换键。量的（算法）；代价多大没量。
- 放量没有分项级的外层标记，录入段（CPU 跑流、排计划、写判表进库）每趟每项照跑（crash-amplification.sh:2 `admission: always`）。推的代价，没量。
- GPU 与 CPU 两种结论互不借：J8 定的分开存，按口径「入库就要能剪枝」，切一次 GPU 开关就整库从头判。量的（比较只认自己那一种）；该不该借要判两种结论等价，这里不判。

### 1.6 最小改法（方向与落点，不写代码）

1. 版本摘要去掉判器闭包文件里的 `#[cfg(test)]` 项：落点 `crates/singlefs-checker-tier/src/judging_closure.rs` 的词法与收闭包两处（`lexical_tokens`、`references_of` 不收测试模块里的 `use`），`build.rs` 同一套随之变；对拍用例照旧在 `crash_identity.rs` 的测试里钉。
2. 版本摘要里的文件行不带路径、只带内容摘要的多重集（或按内容排序），改名不换版本：落点 `judging_closure.rs` 的 `conclusion_code`。
3. `Cargo.lock` 只取闭包碰到的那几个包及其依赖的条目，`Cargo.toml` 取源码摘要（去注释）：落点 `judging_closure.rs` 的 `manifests_of`。
4. 节点名只用复用键，不带设点文件路径与崩溃点名；路径与名字只进覆盖报告与路径索引表：落点 `crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `block_key_under` 与 `crash_identity.rs` 的 `identity_text` 用法。给定状态那一路同样：节点名去掉设点文件与项名、输入串去掉 `seeds_and_scale`：落点 `crash_amplification.rs` 给定状态块键那一段（12094-12145）。
5. CPU 与观察者版本里的设点文件闭包只取被判用例真读的部分（入口从用例函数而不是整份文件出发），或干脆只靠块键里已有的流程摘要：落点 `judging_closure.rs` 的 `ConclusionCodeEntry::entry_files` 与 `crash_state_enumeration.rs` 给观察者的代码文件那两行。这一条要设计判断（J8 定过设点闭包进版本），交三方。
6. 计划哈希按段拆：块键改成按「段的展开方式与态数」那一段的哈希加段内序号，点内别的段变了不连累：落点 `crash_amplification.rs` 的 `plan_crash_points` 与 `block_key`。要设计判断。
7. 判器闭包变了之后的「重贴」：用户 2026-10-15 已定立题，落点 `verdict_store.rs`、`crash_identity.rs`，不重复。

## 2 checker-tier-qemu-device-streams

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| device-streams（六档 `direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor`） | 没有判定库；只有整道一个全绿标记 git common-dir `singlefs-stage-green.checker-tier-qemu-device-streams.sh.<指纹>` | 指纹 = 登记路径 `crates/ Cargo.toml Cargo.lock scripts/ research/results/ .claude/kb/layout/01-first-txn.md` 下 git 列得出的每个文件的源码摘要，加整张登记表 `.claude/gate.d/stage-inputs.tsv`、阶段脚本自己、登记行（键与路径）、工具链、构建环境、键的写法 | 整道：上面任何一份变了（scripts/ 下 1351 份文件里随便一份、research/results/ 里新落一份产物、登记表里给别的实验加一行），六档全部重跑真虚机 | 不合理：没有按档的库；键过宽 | stage-inputs.tsv:37；admission.py:2797-2817（2808 把整张登记表并进非按格的键）、2828-2841（比较：标记文件在不在）；qemu 门禁:64、215、452-456 |

- `--item <模式>` 只跑点名的档时不写标记（qemu 门禁:22、452-454），所以连「只重跑变了的那一档」也做不到复用。量的。
- `git ls-files scripts research/results | wc -l` 本次输出 `1351`；QEMU 这一道真用到的只有 crates 的装置、vm 那几份脚本、check-segment-registry.py、replay.sh 的 E142 行与那一份 E142 产物、布局表（qemu 门禁:253-270 读这几样）。键里其余一千多份文件变了也整道重跑。量的（键）；「真用到的只有这些」是读门禁脚本推的。
- 最小改法：按档建库（键 = 那一档跑的装置的 cargo 依赖闭包 `closure=`、虚机脚本与内核镜像摘要、那一档的模式参数、E142 产物与布局表那一行、工具链），`--item` 跑完也按档记；登记行的 `scripts/` 与 `research/results/` 收窄到真读的几份。落点 `.claude/gate.d/checker-tier-qemu-device-streams.sh`、`.claude/gate.d/stage-inputs.tsv` 第 37 行；判定库可用 `scripts/crash/lib_verdict_library.py`。

## 3 checker-tier-lkmm

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| litmus-verdicts（8 份 litmus） | 没有判定库；只有整道全绿标记 | 指纹 = `litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/` 的源码摘要，加整张登记表、阶段脚本、登记行、工具链、构建环境、herd7 版本（environment=） | 整道：crates/ 下任何一份 .rs 的词法变了，8 份 litmus 全部重跑 herd7 | 不合理：没有按条的库；herd7 的判定只看 litmus 文件、herd7 版本与内核树的 tools/memory-model，crates/ 只用来 grep 锚点与测试名 | stage-inputs.tsv:38；lkmm 门禁:58、85；lkmm.sh:311（`grep -rqF … "$ROOT/crates"` 只查名字在不在） |

- 键过窄的一处：内核树 tools/memory-model 不进键（stage-inputs.tsv:38 的说明句与 lkmm 门禁:22 自己写着），模型换了不重判。量的（不在键里），模型会不会换是推的。
- 最小改法：每份 litmus 一条记录，键 = 那份 litmus（与它的对照组）原文、herd7 版本、tools/memory-model 的目录摘要；锚点与测试名那几条静态检查每趟照跑（便宜）。落点 `.claude/scripts/lkmm.sh` 与 `.claude/gate.d/checker-tier-lkmm.sh`，判定库用 `scripts/crash/lib_verdict_library.py`。herd7 一条跑多久没量，代价大不大是推的。

## 4 checker-tier-crates-mutation-replay（4100 行变异，`grep -c -v '^#\|^$' crates/mutations.tsv` 本次输出 `4100`）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 外层全绿标记 | git common-dir `singlefs-stage-green.checker-tier-crates-mutation-replay.sh.<指纹>` | 登记路径（crates/、Cargo、litmus/、.cargo/、几份脚本、heavy-thread-default）源码摘要 + 整张登记表 + 阶段脚本 + 工具链 + 构建环境 | 整道；失效后进内层逐行判 | 宽，但有内层兜着，合理 | stage-inputs.tsv:45；admission.py:2797-2817 |
| 内层：判定库 | git common-dir `singlefs-mutation-rows/<判法版本>/`，一条一个文件 `mutation.<行键>`，另挂覆盖记录、快照、源码副本 | 判法版本（目录）= 登记行（键与路径）、阶段脚本、crates-mutation-rows.py、mutation-prune.py、lib_libtest_result_lines.py、lib_verdict_library.py、这一趟用的内存包装 run-with-memory-cap.sh，**全部按原文字节**，加构建环境；行键 = 这一行六段原样 + 内存上限 + 限时 + `cargo -V`/`rustc -V` | 判法版本：任何一份那几个脚本原文变了（只改一行注释也算），换目录、4100 条一条不用；行键：这一行或上限、限时、工具链变了只作废这一行（上限与限时是全表共用的，变了全表）；源码变了按覆盖逐行判 | 行级复用合理；判法版本按原文字节、且含共用的内存包装，键过宽 | crates-mutation-rows.py:601-619（判法版本，`read_bytes` 读原文）、640-642（行键）、1074-1102（比较：复用判据）、61、84、187 |

- 按覆盖复用的比较（crates-mutation-rows.py:1074-1102）：没有这个行键的记录 → 现跑（1078-1081）；上次不是抓到 → 现跑（1082-1083）；快照一字未动 → 复用（1084-1085）；否则比被变异的函数（999-1030）、执行过的函数（1033-1055）、函数外改动（1058-1071），三样都过才复用。这是按函数级剪的，合理。量的。
- 不合理的一处：判法版本拿原文字节（`read_bytes`，crates-mutation-rows.py:609-615），而且把 `scripts/hosts/run-with-memory-cap.sh` 整份放进去。内存包装是全仓重活共用的一份，给它改一句注释或加一个与本门禁无关的退出码，4100 条变异的记录全部作废、换目录（判法版本只留最新三个目录，crates-mutation-rows.py:84）。量的。
- 次要：每条变异的上限与限时进行键（crates-mutation-rows.py:61、187 自己写「一变全表判定记录作废」）。抓到的判定只有在「被上限或限时打断才算抓到」那一种上才随上限变，为了它让全表作废偏宽。推的。
- 最小改法：判法版本里的几份脚本改取源码摘要（admission.py 的 `manifest_of_files(..., source_digests=True)` 那一套，注释不算），内存包装只取它决定判定的那一部分（退出码表）或挪出判法版本、进行键里的「这一趟每条上限」旁边；落点 `scripts/mutation/crates-mutation-rows.py` 的 `judge_version`。上限与限时只对「没抓到 / 超时 / 撞顶」的记录作废，抓到的照复用：落点同一份的 `reuse_decision`。

## 5 checker-tier-formal-verification（按格外层标记，格里按条的判定库）

外层：每格各问一次 stage-must-run（formal 门禁:343-357），格的键 = 这一格登记行的路径（stage-inputs.tsv:39-44）源码摘要 + 阶段脚本 + 这一格自己的登记行 + 工具链 + 构建环境 + Verus 工具版本（environment=），整张登记表不进（admission.py:2808）。内层判定库在 git common-dir 的 `singlefs-formal-verdicts/`（formal 门禁:298），存法与比较是 lib_verdict_library.py：键 = sha256（输入指纹 NUL 判法版本）（lib_verdict_library.py:84-86），比较 = `keys/<键>.json` 在不在、读得出、verdict 是 green、记的键与算的相同（lib_verdict_library.py:149-153）。

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| verus-toolchain-pinned | 每趟现核、不复用 | — | — | 合理（便宜） | verus-proof-check.py:62 |
| verus-proof（一项：整个核 crate） | 上面那个库，kind=proof | 判法版本 = verus-proof-check.py 与 lib_verdict_library.py 两份原文 sha256；输入 = 核 crate `crates/singlefs-replay-prefix/` 整个目录逐文件原文 sha256（跳过 target/ 与必红世界表）、verus-pins.tsv 原文、工具链、限时 | 一项，Verus 本来就整 crate 验 | 合理；判法版本按原文，改这两份脚本的注释也作废，偏宽 | verus-proof-check.py:788-794、796-815（807 核 crate 用 `digest_tree` 取原文）；lib_verdict_library.py:52-76 |
| verus-vacuity（每处 requires 一项） | 同库 | 共用那几样 + 「这一处」（项名与函数说明） | 共用那几样任何一样变了，每一处都现判（核 crate 里任何一份文件原文变了，加一个空行也算：verus-proof-check.py 文件头 65-70 行自己写着） | 键过宽：一处 requires 的空真检查只看它所在函数与它调到的规格，核 crate 别的函数变了也全部重判 | verus-proof-check.py:65-70、796-815 |
| verus-must-red-worlds（每行一项） | 同库 | 共用那几样 + 「这一行」原样；必红世界表不进共用 | 改表里一行只现判那一行；核 crate 变了全部现判 | 行级合理；核 crate 那一半同上偏宽（必红世界要在改过的核上验，全部重验大体是对的，推的） | verus-proof-check.py:67-70 |
| pinned-red-examples-complete | 每趟现判 | — | — | 合理（只核表，便宜） | e165-pin-red-examples.py:90 |
| pinned-red-examples-replay（钉表 360 行，`grep -vc '^#\|^$' .claude/gate.d/e165-pinned-red-examples.tsv` 本次输出 `360`） | 同库，kind=pinned-red-examples-replay | 判法版本 = e165-pin-red-examples.py 与 lib_verdict_library.py 原文 sha256；共用一样「装置」= 装置 e165_replay_bounded_enumeration 的 cargo 依赖闭包逐文件**原文** sha256（`input_manifest` 默认 `source_digests=False`）+ 仓根 Cargo.toml、Cargo.lock + 工具链；每行另加这一行、档、引的变异行 | 装置闭包里任何一份文件原文变了（注释也算），360 行全部重放 | 键过宽：闭包是 checker 档、core、harness、checker、format、replay-prefix、reference-filesystem 几个 crate 的非测试源码整片，改一句注释也全表重放；按原文而不是源码摘要，与外层标记的口径不一致 | e165-pin-red-examples.py:1162-1168、1171-1187（1180 调 `input_manifest` 不带 `source_digests`）、1204-1214；admission.py:806 |

- 外层与内层口径不一致：外层格的键取源码摘要（注释不算），内层判定库全取原文 sha256（verus-proof-check.py:807、e165-pin-red-examples.py:1180）。只改注释的那一趟外层整格跳过、走不到内层；外层因为别的输入失效、进了内层时，先前只改过注释的文件也让内层全部重判。量的（两处算法）；碰上的频度没量。
- 最小改法：内层两份判法的「核 crate」「装置」与判法版本都改取源码摘要（admission.py 的 `manifest_of_files(..., source_digests=True)` / `lib_source_digest.py`）；落点 `scripts/gate/verus-proof-check.py` 的 `shared_components`、`judge_version`，`scripts/experiments/e165-pin-red-examples.py` 的 `device_inputs_digest`、`judge_version`。空真按「这一处所在函数与它调到的函数」取键要设计判断，交三方；钉表重放按「这一行点名的变异改到的那份文件与它的调用闭包」收窄同样要设计判断。

## 6 checker-tier-broad-mutation（驱动 scripts/mutation/broad-mutation.py 与剪枝规则 mutation-prune.py）

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| 门禁 gate-verdict 等格（只核库） | 外层全绿标记，键 = 登记路径（含整份 admission.py、stage-cells.sh、run-with-memory-cap.sh、capped.sh 等）源码摘要 + 整张登记表 + 工具与环境 | 整道 | 门禁本身不跑变异，便宜；合理 | stage-inputs.tsv:46；broad 门禁:807、827 |
| 这一版的结果表 | `${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}/versions/<这一版的指纹>/results.tsv` | 这一版的指纹 = sha256（源码指纹：导出树里全部文件逐个**原文** sha256、rustc -Vv、JUDGE_RULES_VERSION、全局键 sha256） | 源码任何一处变了（注释也算）就是新的一版，新一版的结果表是空的 | 单看这一层是整库作废；剪枝靠下面那张跨版本的结论表 | broad-mutation.py:53-58、2869-2886 |
| 跨版本的结论表 | 同库目录 `conclusions.tsv`（只追加，同一个键取最后一行） | 键 =（文件、包住它的最外层 fn 的限定名、函数内相对起止、替换文）；沿用条件 = 全局键相同 ∧ 上次是抓到 ∧ fn 项词法指纹加函数外指纹相同 ∧ 宏与 include 依赖指纹相同 ∧ 抓到它的那条测试的 fn 指纹相同；全局键 = 判法版本、工具链、每份 Cargo.toml / Cargo.lock / .cargo/ 下文件的**原文** sha256、构建环境、nextest / cargo-mutants / llvm-tools 版本、排除表原文 sha256、限时参数、升档常量 | 函数级：改一个函数只重判落在它里面的变异；全局键任何一样变了整表一条不沿用 | 函数级那一半合理；全局键里 Cargo.toml 原文与排除表进键，键过宽；沿用只在 `--changed-since` 下发生，见下 | mutation-prune.py:594-623（键）、640-658（比较 carry_decision）；broad-mutation.py:2869-2886（全局键）、469（判法版本手写常量） |

- 不带参数就不剪枝：选条默认是「这一版还没有定论的全部条」（broad-mutation.py:26），结论表的沿用只在 `--changed-since` 那一支里调（broad-mutation.py:3302-3303、3131-3142）。新一版上直接跑，全部变异现判。这与 test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」不符。量的。
- 剪枝绑在提交上：`--changed-since <提交>` 对着那个提交算「这次改到的」，**改到的一律现判，哪怕结论表里已有同一指纹、可以沿用的那一行**（broad-mutation.py:3138 `if changed is None and carry:`，改到了就走 else 现判）。提交流程用 `--changed-since HEAD`（commit-stages.md:26）。于是提交不上去、反复重做 ①–④ 的那几趟，每一趟都把「自 HEAD 以来改到的函数」上的变异整片重判一遍，上一趟已经判过、指纹一字未动的也不沿用。沿用本来只需要 `carry_decision` 那一处按内容比（它已经比 fn 项指纹、函数外指纹与依赖指纹），提交那一层是多出来的。这正是用户 2026-10-15 说的「commit 和 剪枝 复用无关」那一类。量的（3138 那一行与 mutation-prune.py:640-658、661-681）。
- 全局键里的 `file <路径> <sha256>` 取原文（broad-mutation.py:2875-2877，sha256 来自 manifest），任何一个 crate 的 Cargo.toml 只改一行注释，整表一条不沿用；排除表（只影响第二段慢档那一步）加一行，第一段的结论也全部作废。量的（键）；排除表只影响第二段是读驱动文件头 92-104 推的。
- 键过窄的风险：判法版本 JUDGE_RULES_VERSION 是手写常量（broad-mutation.py:469），mutation-prune.py 与驱动本身的代码不进全局键；改了剪枝规则忘了抬它，旧结论照沿用。推的（靠人记得）。
- 「没红」「无效」每次重判（mutation-prune.py:646-649）：没红是故意的（让新加的测试有机会杀它），无效（编不过）每次重判是多付一次编译，函数与依赖没变时结局不会变。推的。
- 最小改法：① 不带参数的真跑也走结论表沿用（`changed_since` 那一层去掉、只留 `carry_decision`），`--changed-since` 只留作「只读列出改到哪些」；落点 `scripts/mutation/broad-mutation.py` 的选条与 `plan_changed_since`。② 全局键里的 Cargo.toml 取源码摘要（toml 去注释），排除表只进第二段那一步的结论键；落点 `broad-mutation.py` 的 `global_key_lines` 与结论表的写行。③ 判法版本改成「驱动与 mutation-prune.py 的源码摘要」或至少在自证里核「改了剪枝规则而常量没抬」；落点 `broad-mutation.py:469` 与 `mutation-prune.py`。

## 7 checker-tier-research-build-and-replay 的 experiment-replay 与 mutation-tables-compile 两格

| 格 | 库在哪 | 键的组成 | 作废粒度 | 合理与否 | 依据 文件:行号 |
|---|---|---|---|---|---|
| experiment-replay 外层 | **不写全绿标记**（这一道不调 stage-marker-write，`grep -ln stage-marker-write .claude/gate.d/*.sh` 本次只列出 qemu、broad、formal、lkmm、crates-mutation 五份），只认 `refs/sop/staged-green` 那棵整轮全绿的暂存树 | 登记路径 `crates/ Cargo.toml Cargo.lock scripts/experiments/ scripts/ research/results/` 在那棵树与这一次暂存树之间逐文件源码摘要比，加整张登记表、阶段脚本 | 整格；而且只有整轮门禁上一次全绿过、ref 前移了才可能跳过：别的门禁红着、整轮没绿，这一格每趟都进 replay.sh | 不合理：外层复用挂在「整轮全绿」上，不是按自己这一格判绿记的 | stage-inputs.tsv:48；admission.py:1547-1629（标记那一支 1557-1568 找不到标记，落到 1569 起只认 staged-green） |
| experiment-replay 内层：按行复用 | `${REPLAY_VERDICT_LIBRARY:-<git common-dir>/singlefs-replay-verdicts}/<键>/` | 键 = 这一行装置源码（`[[bin]]` 源文件顺 mod / #[path] / include! 带进来的与 include_str! 点名的；驱动函数行取 replay.sh 里那个函数的闭包原文，它点到的 research 二进制、`-p <包> --bin` 的 cargo 依赖闭包，`-p <包>` 不带 `--bin` 的取依赖闭包里每个 crate 目录整个）、公共部分（research 的 Cargo.toml、Cargo.lock、lib.rs 与它带进来的）、复跑表这一行原样、驱动函数原文、留存产物 sha256、replay.sh 判一行的几个函数原文（去整行注释）、工具链、构建环境；逐文件取源码摘要 | 按行；公共部分变了每一行都重跑（admission.py 文件头 75 行自己写着） | 行级合理；公共部分与「不带 --bin 的整包目录」偏宽 | admission.py:3199-3226、3229-3256；比较 replay.sh:2474（这把键的目录里 verdict 与 product 都在、不是 --full） |
| experiment-replay 开跑前的固定开销 | — | — | 每一趟不管多少行能复用，先试编 checker 档 gpu 特性并清场（配置开了 GPU 时）、再整编 research 工作区 | 不合理（推的代价）：全部行复用的一趟照样清一次 GPU、编一次 | replay.sh:2438-2439 在逐行复用（2474）之前 |
| mutation-tables-compile 外层 | 同 experiment-replay：只认 staged-green，不写标记 | 登记路径 `scripts/mutation/tables/ scripts/experiments/ scripts/mutation/mutate.sh scripts/crash/lib_verdict_library.py scripts/crash/crash-amplification-lib.sh research/results/` | 整格 | 同上不合理 | stage-inputs.tsv:49 |
| mutation-tables-compile 内层：按表复用 | `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-mutation-tables-compile-verdicts/`；gate.sh 把 GATE_CROSS_RUN_TMPDIR 设成开跑前的 `${TMPDIR:-/tmp}`（.claude/singlefs-ai-sop/scripts/gate.sh:176） | 一张表一项：表、源文件、源文件 include_str! / include_bytes! / #[path] 引的文件、research 的 Cargo.toml、Cargo.lock、lib.rs、mutate.sh、工具链，全部**原文** sha256；判法版本 = 整份阶段脚本 checker-tier-research-build-and-replay.sh 与 lib_verdict_library.py 的原文 sha256 | 按表：表里改一行，整张表每一条重编；源文件只改注释也整张重编；阶段脚本里任何一处（包括 experiment-replay、research-unit-tests 那几格的代码与注释）变了，所有表全部重编 | 不合理：粒度是表不是条；判法版本拿整份三格合用的阶段脚本；按原文；库默认落在 /tmp，重启就没 | research 门禁:411-421（键）、458（库目录）、504、521（判法版本取的两份）；lib_verdict_library.py:149-153（比较） |

- lib.rs 只取它自己一份（research 门禁:417），lib.rs 里的 `mod admission_tests;` 这类内联模块不另读文件（scripts/experiments/lib.rs:165、212、384 是内联 `mod … {`），所以暂时没漏；以后 lib.rs 拆出 `mod x;` 文件，这一格的键会漏（键过窄，推的）。
- 最小改法：① 这一道 experiment-replay 与 mutation-tables-compile 两格判绿后照 qemu、lkmm 那样调 `admission.py stage-marker-write <根> <阶段>:<格>` 写按格的标记；落点 `.claude/gate.d/checker-tier-research-build-and-replay.sh`。② replay.sh 先算各行的键、全部能复用就不试编、不清场、不整编；落点 `scripts/experiments/replay.sh` 2438-2439 与逐行循环的次序。③ mutation-tables-compile 改成一条变异一项（键 = 这一条的原文与替换文、源文件源码摘要与它的依赖），判法版本只取这一格自己的函数与 mutate.sh 的源码摘要，库挪到 `~/.local/share/singlefs/` 或 git common-dir 下；落点同一份阶段脚本 378-432、458。

## 8 代码与文件头不一致

| 文件头怎么说 | 代码怎么做 | 依据 |
|---|---|---|
| crash_identity.rs:6-8「复用键罩三样属性：……写表；……父输出；判法代码……连同 `Cargo.lock` 与工具链」 | `StepAttributes::reuse_key` 只拼写表与父输出两样，判法另成版本、不进复用键（同一份文件 92-93 行也这样写） | crash_identity.rs:103-118 |
| crash_identity.rs:4-5「前两段是标签，只给覆盖报告；哈希串是复用键」 | 块键里的节点名是整个身份串，带设点文件路径与崩溃点名 | crash_amplification.rs:402、614-615 |
| crash_amplification.rs:272「块键里的节点名只有它，路径不进块键（改名不使判定失效）」 | 「它」= 身份串本身就带设点文件路径与崩溃点名，改这两样就换块键；只有点路径 `path_number`（`<流程>.<周期>.<步>` 串）不进块键 | crash_amplification.rs:614-622 |
| crash_amplification.rs:1131-1132「判法改了每个点的复用键都变、树不动」 | 判法不进复用键，进的是结论版本；而且这一套 `TheJudgeVersion` 产品路径不用（1645-1646） | crash_identity.rs:92-93、103-118 |
| verdict_store.rs:88「输入指纹：门禁 checker-tier-crash-point-replay 给每条崩溃枚举用例算的输入指纹（scripts/gate/admission.py）」 | 输入指纹是这一步的复用键，或给定状态表的输入串摘要；admission.py 的用例指纹只进全绿标记 | crash_amplification.rs:401、12131 |
| verdict_store.rs:16-24 表头写判定块按「块键 ‖ 判法版本」存、「复用只认同一判法版本」 | 产品路径用 26-36 行那一套：（块键，结论种类，版本），GPU、CPU、观察者三把各算 | crash_amplification.rs:1644-1655、12979 |
| crash_state_enumeration.rs:8「改了用例文件只换它那把 CPU 版本」 | 观察者的代码文件也是用例文件，改用例文件同样换观察者版本；用例文件改名还换块键 | crash_state_enumeration.rs:750、1000、1009 |
| crash-amplification.sh:7、33，crash-point-replay.sh:7「同一判法版本下判过的块全部复用」 | 版本按结论种类与设点文件分几把，GPU 一把、CPU 每份设点文件一把，「同一判法版本」不是一个数 | crash_amplification.rs:12942-12985 |
| stage-must-run.sh:21-23、admission.py:339、2849-2850 列写全绿标记的阶段只有 qemu、lkmm、crates-mutation、formal 四道 | checker-tier-broad-mutation 也写 | broad 门禁:807 |
| test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」（规则） | broad-mutation.py 不带参数在新一版上全部现判，剪枝只在 `--changed-since` 下 | broad-mutation.py:26、3302-3303 |

## 9 不合理清单

「已知」= 派发提示给的参照那一处；其余是这一路按代码新找的。每条后面写落点。

1. （已知）判器闭包里任何一处代码变了，GPU 结论整库作废，CPU 与观察者结论随之作废。crash_amplification.rs:12979、14082-14110；verdict_store.rs:3127-3133。落点：重贴机制，用户已立题。
2. 判器闭包的摘要连测试模块一起算：crash.rs、walk.rs、recovery.rs 等文件里 `#[cfg(test)] mod tests` 的词法进摘要，测试模块里的 `use` 还把文件拉进闭包，于是给判器加减一条单测就整库作废。量的 judging_closure.rs:18-21、856-877（只去注释，不认 cfg(test)）、669（停止表自己写着判器文件测试模块里的 `use super::…`）。落点 `judging_closure.rs`、`build.rs`。
3. 判器闭包的摘要带每份文件的仓内路径、`Cargo.lock` 整份原文、闭包碰到的每个 crate 的 `Cargo.toml` 原文：判器文件只改名、Cargo.toml 只改注释、给与判器无关的 crate 加依赖，都整库作废。量的 judging_closure.rs:1073-1085、993-1006。落点 `judging_closure.rs` 的 `conclusion_code`、`manifests_of`。
4. 块键的节点名带设点文件路径与崩溃点名：测试文件改名、崩溃点或项改名（流程一字不变），那条流的块全部换键。量的 crash_amplification.rs:402、614-615；crash_identity.rs:71-78。落点 `crash_amplification.rs` 的 `block_key_under`。
5. CPU 结论与观察者结论的版本含整份设点文件闭包：同一份测试文件里改一条与这条流无关的快档单测，这条流的 CPU 与观察者结论全部作废。量的 judging_closure.rs:1141；crash_state_enumeration.rs:1000、1009。落点 `judging_closure.rs` 的入口取法；要设计判断（J8）。
6. 给定状态表的块键含 `seeds_and_scale` 串、项名与设点文件：加种子、改权重写法串，已判过、内容不变的表全部换键。量的 crash_amplification.rs:12094-12145。落点同一段。
7. 枚举计划哈希按整个崩溃点算：点里一段的展开方式或态数变了，这个点没变的那几段也全部换键。量的 crash_amplification.rs:624-733。落点 `plan_crash_points`、`block_key`；要设计判断。
8. GPU 与 CPU 两种结论互不借：切一次 ENABLE_GPU，这一趟要的那一种从头判。量的 crash_amplification.rs:11853-11857；crash_state_enumeration.rs:1234-1235。该不该借要先判两种结论逐位等价，交三方。
9. 崩溃放量没有分项级的外层标记，每趟每项的录入段照跑。量的 crash-amplification.sh:2；代价推的。落点 `scripts/crash/crash-amplification.sh` 或 `crash_library.rs` 的 `two_stages`（录入段按设点文件闭包与流程参数取键，库里已有那份清单就跳过）。
10. QEMU 六档没有判定库，只有整道一个标记；键里有整个 scripts/、research/results/ 与整张登记表；`--item` 跑完不写标记。量的 stage-inputs.tsv:37；admission.py:2808；qemu 门禁:452-454。落点 qemu 门禁与 stage-inputs.tsv:37。
11. LKMM 8 份 litmus 没有按条的库，crates/ 任何一处词法变了全部重跑 herd7；反过来 tools/memory-model 不进键（过窄）。量的 stage-inputs.tsv:38；lkmm.sh:311。落点 `.claude/scripts/lkmm.sh`、lkmm 门禁。
12. 凡是整道的标记（qemu、lkmm、crates-mutation、broad）与整道的 staged-green 比对（研究复跑），都把整张 `.claude/gate.d/stage-inputs.tsv` 并进键：给任何一个实验或别的门禁登记、改一行，这几道外层同时失效。量的 admission.py:2808、1592。落点 `admission.py` 的 `stage_fingerprint` 与 `gate_reuse`：整道也改成只比自己那几行（与按格的键同一种做法，admission.py:1590-1595 已有）。
13. crates 变异的判法版本按原文字节，且含全仓共用的内存包装 run-with-memory-cap.sh：改一句注释，4100 条记录全部换目录。量的 crates-mutation-rows.py:609-615。落点 `crates-mutation-rows.py` 的 `judge_version`。
14. crates 变异的每条上限与限时进行键，调一次全表作废（抓到的判定多半不随它变）。量的 crates-mutation-rows.py:61、640-642；「多半不随」是推的。落点 `row_key` 与 `reuse_decision`。
15. 形式化内层按原文：钉表重放 360 行共用一把「装置闭包」键，闭包里任一份文件改注释或改与那一行无关的代码，360 行全部重放；空真每一处以整个核 crate 为键。量的 e165-pin-red-examples.py:1171-1187；verus-proof-check.py:796-815。落点这两份的 `device_inputs_digest`、`shared_components`。
16. 广谱变异不带参数不剪枝：新一版上直接跑，全部变异现判。量的 broad-mutation.py:26、3302-3303。落点 `broad-mutation.py` 选条。
17. 广谱变异的剪枝绑在提交上：`--changed-since HEAD` 下凡「自 HEAD 以来改到的」一律现判，不看结论表里同一指纹的那一行能不能沿用；提交不上去、反复重来的每一趟都把同一片重判。量的 broad-mutation.py:3131-3142（3138）；commit-stages.md:26。落点 `plan_changed_since`。
18. 广谱变异全局键里每份 Cargo.toml 取原文、排除表取整份：改注释、加一条排除，整张结论表一条不沿用；判法版本是手写常量，剪枝规则改了忘抬就过窄。量的 broad-mutation.py:2875-2877、2882、469。落点 `global_key_lines`。
19. 研究复跑两格不写按格标记，外层复用只认整轮全绿那棵树：别的门禁红着时这两格每趟都进去。量的 admission.py:1557-1581；`grep -ln stage-marker-write` 的结果。落点 research 门禁。
20. replay.sh 不管多少行能复用，开跑前先试编 checker 档 gpu 特性、清一次 GPU、整编 research 工作区。量的 replay.sh:2438-2439（复用判在 2474）。落点 replay.sh。
21. mutation-tables-compile 一张表一项（改一行整张重编）、判法版本取整份三格合用的阶段脚本、全按原文、库默认落在 `${TMPDIR:-/tmp}` 下重启即失。量的 research 门禁:411-421、458、504；gate.sh:176。落点 research 门禁 378-432、458。

### 最该先改的三条

- 第 17 条（连带第 16 条）：最直接对上用户 2026-10-15 的话「commit 和 剪枝 复用无关」。正确性不靠它：`carry_decision` 已经按 fn 项、函数外与依赖指纹逐条比（mutation-prune.py:640-658），提交那一层只会多判、不会少判，删掉它不丢判别力（推的，要拿自证 `--changed-since 没有只现判改到的与沿用不了的` 那一格改写后核）。改动只在驱动里一处选条逻辑；广谱变异一趟以小时计（commit-stages.md:26 的原话），每多一次提交尝试就省一整片。
- 第 2、3 条：与已知那一处叠在一起。重贴机制做出来以前，它们让「判器没改、只加了单测或改了注释与清单」也整库重判；做出来以后，它们让重贴要跑的对拍次数翻好几倍。改动集中在 `judging_closure.rs` 一份（`build.rs` 照同一套），崩溃点重放全量是以天计的活（admission.py:238「覆盖写固定脚本流全量约 2.3 天（推的）」）。
- 第 13 条：4100 条变异，一句注释就全表作废，而改法只是 `judge_version` 里换成源码摘要、把内存包装挪出去，几行的事。内存包装是全仓重活共用的一份，被改的机会比变异门禁自己的脚本多。

第 10、11 条（QEMU、LKMM 根本没有库）按口径最不合理，但它们每趟的绝对耗时这一路没量，排在上面三条之后；要排先后，以路子三的日志耗时为准。

## 10 推翻条件（什么现象会推翻这些结论）

只读，起不了 cargo 与门禁，下面的现象一条都没在副本里造过；每条写的是「造出来就推翻」的那个观测，交主 agent 或路子三去看。

- 第 2 条：在仓副本里只往 `crates/singlefs-checker-tier/src/crash.rs` 的 `#[cfg(test)] mod tests` 里加一条空单测，跑 `crash_identity` 的 `the_conclusion_code_digest_table_compiled_in_equals_the_one_computed_from_the_tree` 一类用例打出的表（或 OUT_DIR 下 `conclusion_code_digests.tsv`）：`gpu_judge` 那一行的摘要不变，就推翻。反推我做了一步：在 judging_closure.rs 里找过 `cfg(test)`，除了它自己的测试模块（1331）一处都没有，没找到推翻它的代码。
- 第 3 条：只改闭包里一份 .rs 的文件名（同时改 `mod` 声明），`gpu_judge` 摘要不变，就推翻；只改 `crates/singlefs-core/Cargo.toml` 一行注释，摘要不变，就推翻。
- 第 4、6 条：给一条流的崩溃点改名（或给随机历史加一个种子）后跑同一条命令，ITEM 行里旧点（旧表）的块报复用而不是现判，就推翻。
- 第 13 条：只改 `scripts/hosts/run-with-memory-cap.sh` 一行注释，`python3 scripts/mutation/crates-mutation-rows.py judge-version <根>` 打的摘要不变，就推翻。
- 第 17 条：在一棵「自 HEAD 改过某个函数、而结论表里已有那个函数此刻指纹的抓到行」的库上跑 `--changed-since HEAD --list-items`，那几条报「沿用」而不是现判，就推翻（驱动的只读那一支 broad-mutation.py:3435-3446 与真跑同一个判法，3445 那一行同样是「没改到且沿用得了才沿用」）。
- 第 19 条：别的门禁红着、整轮没绿时，`bash scripts/gate/stage-must-run.sh <根> checker-tier-research-build-and-replay.sh` 判可跳过，就推翻。
- 第 21 条（库在 /tmp）：gate.sh 起门禁时 TMPDIR 不在 /tmp 之下、而在持久目录，就只剩「门禁外单跑落 /tmp」那一半。

校验（换一条不共用代码的路再做一遍）这一路没做：独立的路就是路子三的真实日志复用数，本报告只做了正推（读算键与比键的代码）与上面写的反推。

## 11 没做什么

- 没起 cargo、门禁、测试二进制与任何自证，没动任何判定库与标记；上面所有「量的」都是读代码量的，不是跑出来的观测。
- 判器 GPU 闭包具体收了哪些文件、多少份，没跑 `the_judging_closure_of_the_judges_…` 那条用例去列；只凭 `crash_judge_gpu.rs:30` 引 `crate::crash` 与 crash_identity.rs:918-931 的断言推它含 CPU 判器的闭包。target/ 下 build.rs 的 `output` 有 243 行 rerun-if-changed，是全部入口的并集，分不出 GPU 那一份，没用来作证。
- 每道门禁一趟多久、每类改动隔多久发生一次，没量（`git log` 按提交数过的几个数大批合并提交压过，不可信，没写进结论）；「最该先改的三条」的排序按代码里的规模（4100 行、360 行、以天计的全量）与原话推的。
- 没读 research-unit-tests、wallclock-asserts-registered 两格（不在射程），没读 lib_source_digest.py 的细节（只引了它的文件头说法），没逐行读 crash_library.rs 的 `two_stages` 与 GivenStatesEngine 的读库那一段。
- 第 5、6 条（设点闭包进版本、种子与规模进块键）是第三轮判决 J8 / H8 定过的写法，这里只按口径判它宽，改不改要设计判断，没判。
- 没建仓副本与编译目录；草稿目录里只有派发时就在的 prompt.md、.owner、.writer-lock 与这份报告。
