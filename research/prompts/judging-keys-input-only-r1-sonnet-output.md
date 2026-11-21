# judging-keys-input-only-r1 正推腿报告（2026-11-20）

立场：正推。从 `crates/` 今天的实现与实现员的补丁草稿（`~/.local/share/singlefs/work/judging-keys-input-data-only/`，下称补丁草稿）推「键只有输入数据」之后各类改动走哪条路被抓。没读禁读清单里的文件。没写模型（没有模型目录、没有 sha256、没有复跑脚本）：全部结论来自读源码与 grep，写成「推」；要跑出数的命令在「要主 agent 跑的命令」一节。

## 一、结论先说

| 问 | 结论 | 把握 |
|---|---|---|
| 问 1 | 三样（对拍、哨兵、强制现判）按今天的写法，有四类改动完全或几乎抓不到（前三类是零路径，枚举与状态物化要新态里真有红且抽中才抓）：判法变弱（红变绿）、观察者函数自己的判法、给定状态与核对器、枚举与状态物化；另有三类只靠「抽中那一块」的概率抓：判法变严、GPU 内核单边改错、两边同改。对拍在补丁草稿里键只剩输入摘要，改完第一遍之后不再重比。 | 推 |
| 问 2 | 补丁要加的是一个旋钮：强制现判的块数从固定 1 块改成按比例或按轮转取，同一个旋钮罩住对拍库、给定状态表、核对表与观察者录入；另加录入哨兵与一个「全部强制现判」的开关。不把任何判器代码放回键。 | 推 |
| 问 3 | 录入键里留工具链、cargo 生效 profile、特性开关、依赖版本；不算违反「数据和怎么录入没有关系」。 | 推，翻面实验写在问 3 |
| 问 4 | 同一判定项标识下观察者判法意思变了，旧结论照复用，且今天的强制现判结构上抓不到它；标识要在意思变时换号，换号要靠一个「钉住」的检查逼人做决定。 | 推 |
| 问 5 | 按新键，旧的判定行一行都复用不到（版本是另一把值）；真正的风险是删不干净：待判行一行旧格式让服务整扫失败、删除按表标签前缀做而版本在键尾。 | 推 |
| 问 6 | 与已定项 17 其余几条有五处冲突或缺口，原句并排在问 6。 | 推 |

## 二、实现今天的样子（我自己读的，行号都是此刻现查）

只列后面用得上的：

- 复用哨兵的候选只取绿块：候选条件是「键相同、库里有不红的结论、归这台机器的块数」（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:18`），复用谓词要求没有红（`crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:359`）；记着红的块下一趟现判、不进候选（`crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:345`）。
- 抽几块：默认 3 块当哨兵（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:16`），强制现判固定 1 块（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:435`）。种子没设就按时刻与进程号现取，每趟不同（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:280`）。
- 哨兵只验「键相同的块这一趟没被现判」，不比结论：哨兵块这一趟交来存过就红（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:601`）。只有强制现判那一块比结论：现判与库里不同就报「判定不稳」（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:644`）。所以键里没有判器之后，哨兵的 3 块不检查判法对不对，判法有没有变只靠那 1 块。
- 一块 2^16 个态（`crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs:7`）。
- 抽哨兵只在两处调：崩溃枚举用例的判段（`crates/singlefs-verification/src/crash_state_enumeration/judging_stage.rs:156`）与崩溃放量流水线（`crates/singlefs-verification/src/crash_amplification/run_from_the_environment.rs:1382`），每条流每个判段一次。给定状态引擎把强制现判写死成不强制（`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074`）。对拍库的复用只认「两边一致的复用」（`crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs:362`），对拍三个文件里没有强制现判与哨兵（`grep -n -i 'forced\|强制现判\|sentinel\|哨兵'` 对 `crash_library/pairing_blocks.rs`、`crash_library/pairing_inputs_and_conclusions.rs`、`verdict_store/pairing_rows.rs` 我跑过，无输出即零命中；主 agent 复跑命令见第十一节第 1 条）。
- 对拍结论今天的键是输入摘要加两边判器各一把判法单元指纹（`crates/singlefs-verification/src/verdict_store/pairing_rows.rs:8`）。补丁草稿把两边与比法的代码从对拍键里拿掉：补丁草稿 `patch/crates.patch:9592` 写「进库里配对结论那一张表（两边判器与比法的代码不进键）」，`patch/crates.patch:9610` 写「输入摘要只由这条流的输入数据与要比的那几项定：两边判器与比法的代码都不进键」。kb 的各个键表同写：「CPU 一侧与 GPU 一侧的判器与核对代码都不进」（`.claude/kb/decisions/13-验证路线.md:460`）。
- 每趟判段收尾读已存块的计数向量，逐条不变量名要等于池级 checker 的清单，否则报不同（`crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs:132`）。
- 池级 checker 自己有回归语料：已知坏镜像表是「池级 checker 的回归语料」（`crates/singlefs-verification/src/known_bad_images.rs:1`），与变异表一起照单测与变异的复用键走：「照旧：测试闭包引到的产品代码与测试代码」（`.claude/kb/decisions/13-验证路线.md:461`）。这一条路不经崩溃判定库，改 checker 会让它重跑。

## 三、问 1：键里没有判器代码之后，哪几类改动三样都抓不到

约定：「库里那一份」指改动之前已存的绿结论；「新数据」指改动之后新键下第一次判的块；「强制现判」是每流每判段 1 块。

### 3.1 三样完全或几乎抓不到的四类

**判法变弱（红变绿）。** 改 `singlefs-checker` 或 `crash::judge_crash_image` 的闭包，漏掉一条不变量、或把一条判红的条件放宽。库里那一份是绿的，不变；新数据按弱判法判，该红的判成绿。为什么三样都抓不到：哨兵与强制现判的候选只取不红的块（`crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:359`），旧红块这一趟本来就现判，现判成绿就把原红记录换掉（「键相同而库里记着红的，下一趟现判、不复用」，`.claude/kb/decisions/13-验证路线.md:440`），没有任何一处比「以前红、现在绿」；对拍只有两边出现分歧才红，两边同弱不红（CPU 参照路线走 `Cpu { why_not_the_gpu }` 的项没有对拍这一边）。抓它的只有 checker 自己的回归语料与变异表（上节末条），它们只覆盖语料里有的坏法，覆盖不到「只有崩溃枚举域里才出现」的状态。推翻它的现象：往 `singlefs-checker` 里放宽一条不变量，且那条不变量的已知坏镜像语料里没有对应坏法，跑一趟崩溃放量，三样与回归语料都不红。

**观察者函数自己的判法。** 观察者是测试代码里对恢复结果断言的闭包（登记名 `ObserverName`，今天只当索引、不进任何键：`crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs:8`）。录入段按「恢复报告分桶、读集见证」去重：同桶、同读集的状态不再真跑观察者，直接取库里那张镜像，桶键里放的是观察者代码那一把版本（`crates/singlefs-verification/src/crash_amplification/read_set_witnesses.rs:789`）。强制现判抽中一块时，若观察者录入块那一行已在库里，录入段就当录过、不重录（`crates/singlefs-verification/src/crash_amplification/observer_recording.rs:1016`），回录入段重录只在那一行不在库里时才做（`crates/singlefs-verification/src/crash_amplification/observer_recording.rs:1148`）。所以强制现判只重拼「观察者交回的盘上跑池级 checker」与结论拼块，不重跑观察者函数本身；观察者函数的断言放宽了，旧见证照取。键里去掉观察者代码单元之后（补丁草稿停在这一步，见问 4），桶键与见证没有东西随观察者改动而变。推翻它的现象：把一个交回镜像的观察者的断言放宽，库里有旧见证时，强制现判那一块的录入段仍然打出「重录」之外的路（读 `CRASH_POINT_REPLAY_REUSE_SENTINEL_FORCED_BLOCK_RECORDED_AGAIN` 行不出现，且观察者那一份结论与库里相同）。

**给定状态与第二段核对器。** 给定状态引擎判表时强制现判写死不强制（`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074`），这一路没有哨兵；核对器（事实表摘要为键的 `08` 与 `1F` 行）补丁草稿也把键收成只有事实表摘要，核对那一路同样没有强制现判（`grep -n -i 'sentinel|哨兵'` 对 `given_states_engine.rs`、`facts_verification.rs`、`facts_verifier_pending_rows.rs`：只有 `facts_verifier_pending_rows.rs` 里借用报块键的函数名，无抽样逻辑）。对拍（键只剩输入摘要）改完第一遍之后不再重比。三样里没有一样落在这两类上。推翻它的现象：改 `crash_given_states::judge_the_given_states_on_the_cpu` 或 `crash_facts.rs` 一处，库里已有的给定状态表结论与核对行在下一趟仍全复用且没有任何一行抽样日志点名它们。

**枚举与状态物化。** 块键 = 段写表与起步镜像的摘要、计划哈希（段展开、判器另读的输入、态数）、块起点（`crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs:296`）。键里是「第几号态」，不是态本身；号到持久集合的映射由枚举与物化代码定，它在验证包里，既不是产品代码也不是场景，今天按三类代码的表不进任何崩溃判定的键（表里没点名它，见问 6 末条）。映射一改（子集编号次序、展开上限、COW 写全落或全不落的编法），同键同号对到别的态，库里那一份结论挂在没判过的态上。三样里：对拍键只有输入摘要、改完不再重比；哨兵不比结论；强制现判只在抽中的那一块现判出的结论与库里不同时才抓，而号对到别的态之后新态若仍全绿，现判出的逐态判定向量与计数与库里相同，抓不到，所以这一类要新态里真有红才可能被抓，且仍要抽中。推翻它的现象：改枚举代码里子集编号次序（编号 0 到 N 的态集合不变、顺序变），库里已存的块不报任何红，块序号对到的持久集合却与改前不同（用 `crash_library_reader` 读同一块键的结论、再用现判一块的持久集合对比）。

### 3.2 只靠「抽中那一块」概率抓的三类

三类的共同点：一趟每流只有 1 块强制现判，被改坏的判法只改变 m 块（含受影响态的块数），候选 C 块（日志行 `candidates=` 给，`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:18`），每流每趟抓到的概率约 m/C。多趟之间种子不同（时刻与进程号），R 趟累计约 1-(1-m/C)^R；两台各自抽各自的份额，各算一次。这个式子是推的，C 没量过，`candidates=` 要主 agent 从真跑日志里读。

- **判法变严（绿变红）**：改 `singlefs-checker` 加一条不变量、或把判红条件收紧。库里受影响的绿块全部复用成绿。抓它的路：强制现判抽中受影响块；另有一条路，逐条不变量名清单变了时收尾读计数向量报不同（`crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs:132`）：只在增删不变量时响，且响了之后每趟都红，因为复用谓词不看计数向量（`crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:359`），旧行不会被现判换掉（见问 5 末条与问 2 的开关）。条件放严、清单不变的改动只剩抽样。
- **GPU 内核单边改错**：改 WGSL 内核，CPU 参照没改。新数据由对拍抓（两边都判、逐态比）；库里已有的块：对拍库复用「两边一致」那一行，不再比；判段里 GPU 路线的块照库复用，只有强制现判那 1 块真上卡。受影响态只落在已存块里时，概率同上。
- **两边同改**：同一个误解写进 CPU 参照与 WGSL，对拍两边一致，抓不到的部分同「判法变严」。

### 3.3 构造不出反例的几处（写查过哪些路径）

- 改线程数、切段、`checking_threads()`：调查员报告的三处调用只定一块切几段、不改结论（依据来自背景材料的调查员报告，我没逐处复核，当线索）。推不出反例，没有就此判它安全。
- 改写库的格式（计数向量格式号、待判行格式号）：`COUNT_VECTOR_FORMAT_NUMBER`、`GPU_PENDING_ROW_FORMAT_NUMBER` 一类的号变了，旧行读不出就报错，不是假绿（`crates/singlefs-verification/src/verdict_store/gpu_pending_rows.rs:34`）；风险在问 5 的「读不出让整扫失败」。

## 四、问 2：补什么才抓得到，且不把判器代码放回键

原则：键保持只有输入数据；抓判法变化的责任全落在「抽样重判并与库里那一份比」这一个动作上，所以要补的是这个动作的覆盖面与强度，不是键。每条写成对实现的改动，代价写「多判多少」，并写它替掉了什么。库里一族的判定尺寸按 kb 里夹出来的量级：一次整库作废约 3.9×10⁹ 到 3.8×10¹⁰ 态（`.claude/kb/decisions/13-验证路线.md:480`，推的、没量过），下面的代价都按这个量级乘比例，实际要读 `candidates=` 行再算。

**R1 强制现判改成按比例的旋钮。** 改 `crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs` 的 `plan_the_reuse_sentinels_from_the_reuse_read`：强制现判那一次抽（`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:435`）换成「从候选里按比例 p 或按块键哈希取模取一批」，`ReuseSentinelsOfTheRun.forced` 从 `Option` 改成向量，`problems_after_judging`、`states_of_the_forced_block` 跟着逐块循环；p 取自新环境变量（与现有 `SINGLEFS_REUSE_SENTINEL_COUNT` 同写法，`crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs:16` 起那一族常量）。代价：每趟多判 p × 这趟复用的态数（p=1% 是全库量级的 1%，推）。替代：固定 1 块的抽签。p=1 就是「不复用的全量对照」，提交前判器改了的那一趟用它；平时取小值。哨兵的 3 块照留（它们验键稳定，不验判法）。
- 随机抽的缺点：陈旧上界不定。要保证每块至少每 K 趟被重判一次，用「块键哈希加一个轮转位」取模：轮转位取一趟一个整数，存在库里一行数据行里（不是键、不是块属性），多一个状态；不存就取日期加种子，只有概率保证。概率版本够不够用我没量；轮转版本要用户点头多一行库状态。

**R2 对拍库也强制现判。** 改 `crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs` 的按输入摘要复用那一处（`crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs:392` 与 `:402`）：命中且两边一致时，按 R1 同一个哈希与 p 选中的块不复用、重判、与库里那一份逐字节比，不同就报「对拍判定不稳」。代价：每趟多比 p × 这趟对拍块的态数。替代：对拍键里带两边判器指纹（今天的写法，`crates/singlefs-verification/src/verdict_store/pairing_rows.rs:8`；任一边的内核改动让整条对拍流全部重比，量级同上，这是用户这次要拆掉的耦合）。不加 R2，补丁草稿里的对拍就是只比一遍的单次检查（问 6 第 1 条）。

**R3 给定状态表与核对行也强制现判。** 改 `crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074` 那一处写死的不强制，改成按 R1 选；核对待判行（`FactsVerifierPendingRow`，字段表在 `crates/singlefs-verification/src/verdict_store.rs` 文件头「核对待判行」一行）加一个强制现判字节，并入补丁已经要做的格式号 1 升 2（今天的格式号 `crates/singlefs-verification/src/verdict_store/facts_verifier_rows.rs:28`），不再为它单独升一次号。代价：同 R2 量级。替代：无（今天这两路零抽样）。

**R4 强制现判那一块要真重跑观察者。** 改 `crates/singlefs-verification/src/crash_amplification/observer_recording.rs:1016` 起的 `observer_block_is_recorded_or_concluded`：强制现判的块不论录入块那一行在不在都算没录过（今天在就算录过，`crates/singlefs-verification/src/crash_amplification/observer_recording.rs:1018`）；录入那一块不走见证去重（桶键的 `observer_code` 那一段，`crates/singlefs-verification/src/crash_amplification/read_set_witnesses.rs:789`，随观察者标识走，见问 4）。代价：每趟多恢复 p × 块数 × 2^16 次（恢复是 CPU 上最贵的一步，没量）。替代：无。

**R5 录入哨兵。** 改 `crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs:70` 那一支「命中就不再录」：每趟从命中的清单里抽 1 项，在一次性库里重录、比 `0F` 流摘要，不同就红。代价：一项场景的录入时间（CPU）。替代：今天的录入键里带录制管线单元（`crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs:43`，补丁之后拿掉）。`grep -rniE 'recording_sentinel|RecordingSentinel|录入哨兵|录入强制' crates scripts .claude/rules` 零命中（我跑过，无输出），今天没有这一路。

**R6 让「计数向量对不上」的块自愈。** 改 `crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:358` 的复用谓词：计数向量里逐条不变量名与当前清单不同的块答不复用（`crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs:132` 已有这个比法，抽出来给复用谓词用）。代价：增删不变量那一趟现判全部受影响块（该有的重判）；省下：今天的写法下这类变化让每一趟都红且没有自愈路径。

**R7（要用户点头，与已定条款冲突）判器改动的提示。** 想要「判器代码变了就自动加大 p」需要一个不进键的变更检测：对判器入口算单元指纹、放在门禁里（`scripts/` 一侧）当触发器，指纹与键无关，只决定提交那一趟 p 取多少。冲突在三处：用户原话「这次改版就完全拆掉比较好」（`records/2026-11-19-判定键只算产品与测试代码.md:93`）、`块属性里不再记 GPU 判器是哪一版。`（`.claude/kb/decisions/13-验证路线.md:464`）、「手写的版本常量」不进键也不当复用依据（`.claude/kb/decisions/13-验证路线.md:436`）。我不推荐在没有用户点头时做；R1 到 R6 不需要它。

## 五、问 3：录入键留不留工具链与依赖版本

**结论：留。留了不违反「数据和怎么录入没有关系」。**

依据（都是推，没量过录出来的字节）：

1. 录入键今天由哪些东西组成：录入那一组判法单元的指纹与录入输入摘要拼起来取 SHA-256（`crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs:43`）。指纹是验证包构建脚本编进二进制的那一把（`judging_units_key` 的 `_toolchain` 参数其实没用，`crates/singlefs-verification/src/crash_identity/compiled_judging_units_key_table.rs:191`），构建脚本算它时带工具链行与「与 `Cargo.lock`（按闭包收窄」所指的那份收窄过的依赖行（`crates/singlefs-unit-graph/src/entry_groups.rs:5`），工具链行含进键的 profile 设置：`opt-level`、`debug-assertions`、`overflow-checks`、`lto`、`panic`、`codegen-units`（`crates/singlefs-unit-graph/src/build_toolchain.rs:25`）。补丁草稿停在录入键这一步没改（补丁草稿 `result.md` 的「停下交主 agent 的设计问题」第 2 条）。用户定的新写法只说「产品代码 + 场景」（`.claude/kb/decisions/13-验证路线.md:458`），没说工具链与依赖留不留；已定项 17 的第一条把工具链与依赖版本算进「每一份判定」的单元键（`.claude/kb/decisions/13-验证路线.md:435`），标题把它们归在「其余键」（标题行，转述），录入键不在「崩溃判定的键」之列，所以字面读法是留。
2. 录出来的数据会不会因为别的东西变：会，且其中有两样是「产品跑成什么样」，不是「怎么录」。
   - 特性开关：核心层 `test-only-publish-thresholds` 改发布触发阈值，`lookup-work-counters` 改计数（`crates/singlefs-verification/src/crash_identity/compiled_judging_units_key_table.rs:33` 起的两个常量），前者直接改场景里什么时候发布、写什么。
   - 工具链与 profile：`overflow-checks`、`debug-assertions`、`panic` 在产品代码有溢出或断言时改行为（产品代码里有溢出或 debug 断言才会不同，没查有没有）。
   - 依赖版本：产品包依赖很少：格式包没有依赖（`crates/singlefs-format/Cargo.toml:8`），核心层只有 `libc`（`crates/singlefs-core/Cargo.toml:14`）与 `vstd`，产品 GPU 包钉死 `wgpu = "=30.0.1"`（`crates/singlefs-background-gpu/Cargo.toml:17`）。所以依赖版本对录出的字节影响大概率很小，只有走产品 GPU 后台计算的场景里 `wgpu` 版本才真起作用。这个小不是留的理由，也不是拿掉的理由：它进键的代价是零。
   - 录制管线（录制器怎么把写记成表）：这才是「怎么录」。它不改产品写什么，只改记什么；它不该进录入键，与用户原话「数据和怎么录入没有关系吧。 我怎么想都不应该产生关系。」（`records/2026-11-19-判定键只算产品与测试代码.md:92`）一致。
3. 多留比少留安全，且代价只有重录：录入键只管「这份写流录过没有」，命中就跳过录入（`crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs:70`）；下游每一块的块键是内容摘要（段写表与起步镜像的摘要、计划哈希、块起点，`crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs:296`）。工具链一变而录出的字节没变，重录得到同一批块键，已存结论照复用，多花的只有 CPU 录入时间。反过来，工具链变了、字节变了而录入键不变，下一趟读到的是旧编法录出的旧流，新工具链下的产品行为从没被录过，也没有任何路径发现它（这就是问 1 的「录入漂移」，下面 R5 抓它）。
4. 与问 6 的关系：留它们不与「键过窄算违反」冲突，拿掉它们才可能冲突。

**按包过滤口子的副作用（要主 agent 知道）**：`crates/singlefs-unit-graph` 今天没有按包过滤的口子（`grep -rn -i 'package_filter|only_these_packages|keep_only_packages|按包过滤|only_packages' crates/singlefs-unit-graph/src` 零命中，我跑过，无输出）。加口子之后依赖行收窄按过滤后的闭包算（`crates/singlefs-unit-graph/src/lock_file.rs:1` 的收窄规则），只有产品与场景真引到的依赖留下，验证包里只给判器与服务用的依赖（经 prune-server 客户端、GPU 一侧）自然离开录入键；工具链行与 profile 行与单元无关，口子不动它们。场景与录制管线同在验证包里，按包分分不开；补丁草稿的做法（A）要在登记项加「只留这几个包」一栏，但验证包本包里谁算场景、谁算管线（`memory_pool.rs`、`fault_injection.rs`、`crash_point_recorder.rs`、`scenario.rs`）三类代码的表没点名（`.claude/kb/decisions/13-验证路线.md:452` 只点名了录制、核对、GPU 判器三段），收窄过头就是键过窄，收窄不够就是录制管线又进了键；这件事要在加口子之前先把验证包文件逐个归类。

什么现象会推翻问 3 的结论：同一场景在两种工具链（debug 对 release、开关特性与否、升一次 Rust）下各录一遍，所有登记场景的 `0F` 流摘要逐个相同，就说明这些东西今天对录出的字节没有影响，留它们是无用但无害的保险（结论仍是留，理由换成「代价是零」）。复核不了：没有一条现成的命令打印流摘要，要在装置里写读 `0F` 行的对比，不在只读命令范围内；写进「要主 agent 跑的命令」第 4 条作为设计，不是现成命令。

## 六、问 4：观察者判定项标识何时换号

**结论：同一标识下观察者判法的意思变了，旧结论照复用，属于问 1 的「观察者函数自己的判法」那一类，且今天的强制现判结构上抓不到它。标识要在意思变时换号；换不换由一个钉住的检查逼出来，由主 agent（或三方）判「意思变没变」。**

依据：

1. 同一标识指什么。今天结论种类里没有观察者名，观察者是哪一个由它那一组判法单元的键担（`crates/singlefs-verification/src/crash_identity/conclusion_kinds_and_versions.rs:8`、`crates/singlefs-verification/src/crash_amplification/observer_definitions.rs:360` 起的 `observer_versions_of_the_keys`）。补丁草稿停在这里：观察者自己的代码单元「照旧进键」，原因是名字不进键、两个观察者没有东西分开（补丁草稿 `result.md` 的设计问题 1：「观察者自己的代码单元加上额外输入照旧进键」）。用户后来定了用判定项标识分开（`records/2026-11-19-判定键只算产品与测试代码.md:96`「同一块上的几个观察者在结论的键里用判定项标识分开」），也就是名字或登记序号这一类稳定标识进键。
2. 标识不变而判法变会怎样：结论块（`ObserverVersions.conclusion`）、录入块（`.observer`）、去重桶（`.observer_code`，`crates/singlefs-verification/src/crash_amplification/read_set_witnesses.rs:789`）三处的键都不变，旧行全复用；强制现判只重拼池级 checker 那一半，不重跑观察者函数（问 1 的观察者一类）。所以这一类被抓的概率是 0，不是 m/C。
3. 为什么不能靠观察者代码进键：用户已经定了不进（「观察者的代码都不进键」，`.claude/kb/decisions/13-验证路线.md:444`）。
4. 谁判、怎么钉住（对实现的改动）：
   - 标识 = 观察者名加一个修订号，写在每个观察者的登记处（`grep -rn 'ObserverName::new(' crates --include=*.rs | wc -l` 我跑过，28 处命中，其中 `tests/` 目录里 10 处；造名字的有写死的串与按观察者的 `name()` 现造两种，后者如 `crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:757`）；修订号进结论键、录入块键、桶键。
   - 钉住：加一张钉表（形态同 `.claude/gate.d/verus-pins.tsv` 那一类：每个观察者一行，记它的函数单元指纹与修订号），一个检查按单元图现算每个观察者的指纹，与钉表比：不同就红，红的出路两条——修订号加一（意思变了，旧结论作废）或重钉指纹并在提交说明里写「意思没变」。检查读的是指纹，指纹不进任何键，所以不违反「观察者代码不进键」。
   - 谁判「意思变没变」：改 `crates/` 都走三方，这个判断是代码轮里的一格；没三方的小改（重命名、提取变量）由主 agent 判。
   - 代价：每次改观察者多一次钉表更新（观察者很少改，没量）；加这个检查要在 `crates/singlefs-unit-graph` 或脚本侧留一个按入口算指纹的口子，而用户定了判器那几组的入口登记「整批删掉」（`records/2026-11-19-判定键只算产品与测试代码.md:96`），所以这一格与「完全拆掉」有张力，要用户点头（写进问 6）。
5. 不要钉表的替代：每次提交前对观察者跑一次不复用的全量对照（R1 取 p=1，只对观察者那几族）；代价按问 2；它不用任何指纹，但没有人逼人去做，所以只在「人记得」时有效。

## 七、问 5：删旧库删不删得干净

**结论：按新键，旧的判定行一行都复用不到；白白重判的风险不在键，在「删多了」（把键没变的族一起删）；被新代码误读的风险在「删漏了」，其中待判行一行旧格式就能让 GPU 服务整扫失败。** 都是推，没碰真库；数条数的命令在「要主 agent 跑的命令」。

### 7.1 哪些旧行新键下还复用得到

- 判器那一半结论：结论块表的键是「块键 ‖ 结论种类 ‖ 版本（32 字节）」（`crates/singlefs-verification/src/verdict_store.rs:39`）。新版本是结论种类定的定值（补丁草稿 `result.md` 的「结论」：「三种判出的结论版本只由结论种类定」，`result.md:8`），旧版本是判法单元指纹的哈希，两者不可能相同，所以判定块、违例、恢复字、计数向量、块属性、点名的被抛弃根（表 `01`、`04`、`09`、`0A` 到 `0E`、`18`）一行也复用不到。其中最新一版旧版本下的行按内容其实仍然正确（判器没变过的话），删掉等于全量重判；这是用户定了「不做迁移」要付的，量级同问 2 的 3.9×10⁹ 到 3.8×10¹⁰ 态（推）。副作用是好的：库里早先被旧判器判绿的陈旧行也一并没了，问 1 的陈旧窗口从这一刻重新计起。
- 观察者那几张（`15`、`16`、`17`、`1A`）与观察者结论：键里带观察者代码那一把版本或桶键（`crates/singlefs-verification/src/crash_amplification/read_set_witnesses.rs:786`），补丁草稿保留观察者代码进键，用户后定改成标识；无论哪一种，旧键与新键都不相同，复用不到。
- 对拍结论（`19`）：旧键「输入摘要（32 字节）‖ 一边的判法单元指纹（32 字节）‖ 另一边的判法单元指纹（32 字节）」（`crates/singlefs-verification/src/verdict_store.rs:83`），新键只有输入摘要，长度不同，复用不到。
- 录入清单（`10`）：键含录入那一组的指纹，新键另立一组，复用不到。
- **键恰好相同的，只有两类**：一是只按内容寻址、不带版本的表：录下的流程（`0F`）、判表（`11`）、给定状态表（`12`）、观察者基底（`14`）、读集行（`1B`）、对拍输入（`1C`）、判定向量两张表（`02`、`03`）。它们按内容取摘要为键，新代码重录得到同一批字节就是同一把键，所以删它们不白重判结论，只多一遍录入写入；留着也无害，但没有清单（`10`）指它们了。二是 Python 五族（`verdict-library`、`unit-verdicts`、`mutation-units`、`broad-mutation`、`apparatus-items`）：单测与变异的复用键「照旧」（`.claude/kb/decisions/13-验证路线.md:461`），键没变的行（测试闭包没动的）今天与明天读到的是同一把键，整库删会白白重判它们。用户说的是「里按旧键存的、新键下复用不到的判定与录入数据都删掉」（`records/2026-11-19-判定键只算产品与测试代码.md:94`），所以删除的范围应只是 Rust 崩溃判定库那几个标签前缀，不是整份 prune-server 库。

### 7.2 删不干净会被新代码误读的几类

1. **待判行（`1D`、`1F`、`20`）**：补丁把待判行格式号由 1 升到 2（今天是 `GPU_PENDING_ROW_FORMAT_NUMBER: u8 = 1`，`crates/singlefs-verification/src/verdict_store/gpu_pending_rows.rs:34`），读不出的一行让整张表的全扫失败：「`GpuPendingRow::from_value_bytes(&value_bytes).ok_or_else(not_in_the_layout)?;`」（`crates/singlefs-verification/src/verdict_store/gpu_pending_rows.rs:347`）。GPU 服务扫不了就点名一次、交空，也就是这一族一块都不取（`gpu-verification-server/src/library_pending_pool/image_judgement_pending_rows.rs:20`，认领那一回同写法 `gpu-verification-server/src/library_pending_pool/claims_across_the_families.rs:40`）。这不是假绿而是整道停转，且不自愈：只要库里留一行旧格式待判行（跑到一半被杀的那一趟留下的）。记录里定的次序是「新键的代码合进主工作区、GPU 服务照新代码重编重起之后做」删库（`records/2026-11-19-判定键只算产品与测试代码.md:94`）；这几张表要在服务重起之前删，或重起之后立刻删、删完再放判段。
2. **判表待判行与判表结论字（`20`、`21`）、对拍结论（`19`）**：键的段数变了（三段变两段、三段变一段，`crates/singlefs-verification/src/verdict_store.rs:125`、`crates/singlefs-verification/src/verdict_store.rs:83`），按整表前缀扫的读法碰到旧键就报不合布局（我没读解析函数，推的）。
3. **库布局号**：补丁草稿没碰布局号，今天开库只认 6（`crates/singlefs-verification/src/verdict_store/library_opening_and_layout.rs:7`，`grep -n 'LIBRARY_LAYOUT' patch/crates.patch` 零命中，我跑过）。表的键与值格式变了而布局号不变，没重编的二进制（第二台上没重编的那份、没重起的服务）照旧开库、照旧往库里写旧格式，删完又被写回。出路：布局号升到 7 并把删库那一步写成「删表 → 写布局号」：旧二进制开库就拒（`crates/singlefs-verification/src/verdict_store.rs:145`），新二进制头一次开发现布局行不在就写 7。这不是迁移，是「读者只认这一版」。
4. **判定向量两张表（`02`、`03`）删不掉**：写批对这两张表的删除一律整批拒（「删这两张表的行整批拒」，`crates/singlefs-verification/src/verdict_store.rs:32`）。用户要的「删完回读条数为 0」对这两张不成立；它们按内容寻址、不依赖任何版本，留着无害，要在删前清单里把它们标成「保留、不计入回读为 0」。
5. **删的方式**：按标签前缀删（`prune_server_client.py delete-prefix --hex <前缀>`，`scripts/prune-server/prune_server_client.py:138`）。判定块的键是「标签 ‖ 块键 ‖ 种类 ‖ 版本」，版本在键尾（`crates/singlefs-verification/src/verdict_store.rs:39`），所以删完之后新代码写进同一标签的行，以后没有办法只删「某一版」：只能整标签删。这一点决定问 2 的开关是唯一的出路：陈旧行没有选择性删除，只能靠强制现判换掉。

### 7.3 建议的删前清单

先在删之前按表数条数与大小（命令在「要主 agent 跑的命令」第 2 条），分三栏：全删（`01`、`04`、`09`、`0A` 到 `0E`、`10`、`13`、`15` 到 `19`、`1A`、`1D`、`1E`、`1F`、`20`、`21`）、保留（`02`、`03`、`00`）、可删可留（`0F`、`11`、`12`、`14`、`1B`、`1C`）。分栏是我对表头（`crates/singlefs-verification/src/verdict_store.rs:1` 起）逐表读出来的；核对表 `08` 的键在表头的表格里没列，只在说明里提到由 `facts_verifier_row_key` 给（`crates/singlefs-verification/src/verdict_store.rs:114`），它的分栏（全删，按事实表摘要寻址、补丁后键里没有版本）是推的，没读 `08` 的编码函数。

## 八、问 6：与已定项 17 其余几条的冲突

「冲突」= 两句话照字面不能同时成立；「缺口」= 不冲突但规则没写怎么办。每条两句并排，改法写在后面。

1. **冲突：对拍键只有数据 vs 靠对拍抓判法改动。**
   - 「CPU 一侧与 GPU 一侧的判器与核对代码都不进」（`.claude/kb/decisions/13-验证路线.md:460`，对拍结论表的键）。
   - 「GPU 判器是加速路径，它与定义那一组一不一致、判法改了对不对，由对拍（GPU 与 CPU 对拍用例）与复用哨兵抓」（`.claude/kb/decisions/13-验证路线.md:466`）。
   - 两句合起来的实现是补丁草稿里的对拍：键只有输入摘要（补丁草稿 `patch/crates.patch:9610`），复用谓词只认「两边一致的复用」（`crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs:362`），没有强制现判。这样「靠对拍抓」只在第一遍成立。改法：二选一——对拍库也进强制现判（问 2 的 R2），或对拍键重新带两边指纹（与 460 行那一句冲突、也是用户这次要拆的耦合）。我推荐前者。
2. **冲突：手写版本常量不进键、不当复用依据 vs 结论种类定值版本与判定项标识。**
   - 「手写的版本常量」（`.claude/kb/decisions/13-验证路线.md:436`，一律不进键、也不许当复用依据）。
   - 补丁草稿里三种结论的版本是「只由结论种类定」的定值（补丁草稿 `result.md:8`），用户定的判定项标识（`records/2026-11-19-判定键只算产品与测试代码.md:96`）进键，问 4 的修订号也是手写的。
   - 改法：436 那一句加例外——结论种类与判定项标识（含修订号）不算「手写的版本常量」，理由是它们不随代码改动而变，只在判法意思变时手动换，且换号由钉住的检查逼出（问 4）。
3. **冲突：测试服务代码「不进任何键」 vs 单测与变异的复用键「照旧」。**
   - 三类代码表里测试服务代码行写「不进任何键」，且点名「录制、核对（CPU 参照 `crash_facts.rs` 与 GPU 内核 `crash_verify_gpu.rs` 都算）」（`.claude/kb/decisions/13-验证路线.md:452`）。
   - 「照旧：测试闭包引到的产品代码与测试代码」（`.claude/kb/decisions/13-验证路线.md:461`）。
   - 验证包里有测试直接测 `crash_facts.rs` 一类文件，如「GPU 内核与 CPU 参照按同一张事实表判每个状态」（`crates/singlefs-verification/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs:2`）：它们在测试闭包里，按 461 要进单测键，按 452 又不进任何键。字面读法下，改录制或核对代码不让测它们的单测重跑。我没查哪些测试的闭包到得了这几个文件（没跑单元图），这是字面冲突，不是量到的影响。改法：452 那格把「不进任何键」改成「不进崩溃判定的键与录入键」，单测与变异键照 461 取。
4. **缺口：「改键让旧库作废不算代价」没有对应的开关。**
   - 「改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。」（`.claude/kb/decisions/13-验证路线.md:441`）。
   - 键里没有判器之后，改判法不改键，「改完重跑」让库里的陈旧行原样复用；今天删旧库是一次性的，之后没有选择性作废（版本在键尾，问 5）。改法：把问 2 的强制现判旋钮（R1，p=1）写成「判法意思变时改完重跑的那一个办法」，并写进已定项 17。
5. **缺口：键过窄的判据没指到枚举与状态物化。**
   - 「键过窄（引用的常量、类型、宏漏收）」（`.claude/kb/decisions/13-验证路线.md:436`）。
   - 崩溃判定的键只有输入数据（`.claude/kb/decisions/13-验证路线.md:444`），块键里是态号不是态（`crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs:296`）；号到持久集合的映射由枚举代码定，三类代码表没点名它。改法：表里点名它归哪一类；若归「不进键」，则把「枚举定义」写成块键的一项输入（一个由枚举代码手写的定义号，又落回冲突 2 的例外）或用问 2 的 R1 抽样盖。
6. **缺口：复用哨兵的写法与实现不一致。**
   - 「每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判」（`.claude/rules/verification.md:41`）。
   - 实现只在崩溃放量与崩溃枚举用例判段抽、候选只取绿块、每流 1 块（本报告第二节），给定状态、核对、对拍都不抽。规则没说「哪些判定要抽」。改法：规则句加「按判定入库的每一类」，实现按问 2 补齐。
7. **缺口：录入键的组成表里没写工具链与依赖。**
   - 「产品代码 + 场景；另立一组判法单元，只收产品代码与场景的单元，录制管线排出去」（`.claude/kb/decisions/13-验证路线.md:458`）。
   - 已定项 17 第一条写「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」（`.claude/kb/decisions/13-验证路线.md:435`）。改法：录入键那一行补「加工具链、生效 profile、特性开关、闭包真引到的依赖版本」（问 3）。

**不冲突、与新写法一致的**：「收不全…拒算键、照跑，不静默少收」（`.claude/kb/decisions/13-验证路线.md:437`）：录入键按场景入口收闭包，收不全就拒算、照录，方向是多录不是少录；「红的也照键记」（`.claude/kb/decisions/13-验证路线.md:440`）：按数据键记红照旧成立，缺口只在「红变绿」没人比（问 1）。

## 九、推翻条件（问 2 到问 6）

- 问 2：R1 到 R5 落地后，故意把池级 checker 放宽一条不变量（该不变量的已知坏镜像语料里没有对应坏法）、把 WGSL 内核改错一个比较、把观察者断言放宽，p 取 1 跑一趟，三种改动的强制现判都红，则 R1 到 R4 站得住；任何一种仍然全绿，推翻对应那一条。p 取小值（如 1%）时是否够用没量：要读 `candidates=`（第三节 3.2）与一趟实际重判态数。
- 问 4：把一个交回镜像的观察者断言放宽后，库里有旧见证时强制现判那一块打出重录行且观察者那份结论变了，就推翻「抓不到」。钉表那一条的推翻：改观察者的函数体（不改意思，只提取变量），钉表检查不红，说明指纹没有按单元算对。
- 问 5：删库前的 `--dry-run` 条数里，`19`、`1D`、`1F`、`20`、`21` 之外还有会被新代码按整表前缀扫的表，那一张也要进「全删」栏；`1D` 只剩新格式行时不会整扫失败，推翻 7.2 第 1 条的前提是新服务读不出一行时跳过那一行而不是整表交空（我读的是今天的 `candidates()`，补丁里服务一侧的改动没读：`~/.local/share/singlefs/work/judging-keys-service-side/` 未读）。
- 问 6：照每条的改法改 kb 后，对 `kb-spec-check.py` 与三语同步的门禁不红即可；冲突 3 的推翻是单元图里列出验证包某个对 `crash_facts.rs` 的测试，其闭包不含 `crash_facts.rs`。

## 十、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问 1 判法变弱 | 冲突（规则说靠对拍与哨兵抓，实现抓不到） | 红块不进候选，红变绿没有任何比较（`crates/singlefs-verification/src/crash_amplification/block_recording_and_storage.rs:359`） |
| 问 1 观察者函数自己的判法 | 冲突 | 强制现判不重跑观察者，桶键没了代码就取旧见证 |
| 问 1 给定状态与核对器 | 冲突 | 强制现判写死不强制（`crates/singlefs-verification/src/crash_amplification/given_states_engine.rs:1074`），核对行零抽样 |
| 问 1 枚举与状态物化 | 规则没说 | 块键里是态号，号到态的映射不在任何键里；新态全绿时强制现判也抓不到 |
| 问 1 判法变严 / GPU 内核单边 / 两边同改 | 一致（靠概率抓） | 每流每趟约 m/C，C 没量 |
| 问 1 录入漂移 | 规则没说 | 录入键不含管线，管线改了不重录，也无路径发现（问 3） |
| 问 2 R1 到 R6 | 一致（改 `crates/` 即可，不动键） | 强制现判按比例、覆盖对拍库、给定状态、核对、观察者，另加录入哨兵与计数向量自愈 |
| 问 2 R7 判器改动提示 | 冲突 | 与「完全拆掉」「手写版本常量」「块属性不再记」三处冲突，要用户点头 |
| 问 3 工具链与依赖留不留 | 一致 | 留，代价只有重录；不违反「数据和怎么录入没有关系」 |
| 问 4 判定项标识换号 | 规则没说 | 要换号；钉表检查逼出，指纹不进键 |
| 问 5 旧判定在新键下复用 | 一致 | 判器那一半一行也复用不到；键相同的只有按内容寻址的表与 Python 五族 |
| 问 5 删不干净被误读 | 冲突（记录里的删库次序） | 旧格式待判行让服务整扫失败，要在服务重起前删；布局号没升；`02`、`03` 删不掉 |
| 问 6 与已定项 17 其余几条 | 冲突 3 处、缺口 4 处 | 见第八节原句并排 |

## 十一、要主 agent 跑的命令

没写模型、没有 `rerun.sh`、没有探针，所以没有复跑脚本与 sha256；下面是读数命令。都经任务服务交，在仓根起。

1. 复核「对拍与哨兵两个文件里没有强制现判」（我跑过，零命中，主 agent 复跑留证）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=只读 grep; 范围=3 个文件; 为什么=复核正推腿报告第二节第 4 条' task-server/task submit -- grep -n -i -E 'forced|强制现判|sentinel|哨兵' crates/singlefs-verification/src/crash_library/pairing_blocks.rs crates/singlefs-verification/src/crash_library/pairing_inputs_and_conclusions.rs crates/singlefs-verification/src/verdict_store/pairing_rows.rs`
   - 起点：仓根。结局：`TASK_SERVER_RESULT` 行 `type=command-failed exit=1`（grep 零命中退 1）算与报告一致；有命中就推翻那一句。
2. 删旧库之前数各标签前缀的条数（只数不删）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=prune-server 库按标签前缀数条数（只数不删）; 范围=崩溃判定库各表; 为什么=问 5 的删前清单' task-server/task submit -- python3 scripts/prune-server/prune_server_client.py delete-prefix --dry-run --hex 01 --hex 04 --hex 09 --hex 0A --hex 0B --hex 0C --hex 0D --hex 0E --hex 10 --hex 13 --hex 15 --hex 16 --hex 17 --hex 18 --hex 19 --hex 1A --hex 1D --hex 1E --hex 1F --hex 20 --hex 21 --hex 0F --hex 11 --hex 12 --hex 14 --hex 1B --hex 1C --hex 02 --hex 03`
   - 起点：仓根。结局：`type=succeeded exit=0`；日志里每个前缀一行条数、末行 `PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=… prefixes=29 keys=<总数>`（`--dry-run` 的文件头说明在 `scripts/prune-server/prune_server_client.py:138` 起，没读 `--dry-run` 的实现，写不写进库以文件头为准）。`1D`、`1E` 在库里的键空间与协议里 `gpu-pending/…`、`gpu-rejudged/…` 是同一族（`prune_server_client.py` 文件头「GPU 判的三族」），数不到时以文件头为准另数。
3. 读 `candidates=`（C）：下一趟崩溃放量或崩溃枚举用例的真跑日志里，`grep -h CRASH_AMPLIFICATION_REUSE_SENTINELS <日志>`（`task-server/task log <号> --all` 取日志）。每条流一行。这是问 1 概率式里 C 的唯一来源；仓里 `research/results` 下我 grep 过，没有这一行（无输出）。
4. 复核不了：问 3 的 `0F` 流摘要跨工具链对比、问 1 各类「推翻它的现象」里要改坏判器再跑的几条。原因：要改 `crates/` 编译再跑，我不编译、不起装置；也没有现成的命令打印流摘要。需要的话，新派实现员在草稿副本里做。

## 十二、没做什么

- 不判别的腿的格（辩方与攻方的格）没碰；没替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数（`candidates=`、各前缀条数、对拍与哨兵的实跑结果）没复跑，命令在上一节。
- 没写模型、没编译、没跑测试、没读禁读清单里的文件。
- 没读 `gpu-verification-server` 在补丁后的新样子（服务一侧补丁没读，只读了主工作区今天的服务代码）；问 5 第 1 条对「读不出一行就整扫交空」的推，建立在今天的 `candidates()` 上。
- 补丁草稿不在仓里，引它时写的是草稿目录里的相对路径，行号取自草稿此刻的样子。
