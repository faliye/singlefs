# m3-crash-enumeration-merge-r2 辩方腿（Sonnet）报告（2026-10-10）

立场：辩方，复核第一轮判决 `research/prompts/m3-crash-enumeration-merge-r1-main-verification.md`（L4），再把 G1–G14 逐条对第一节「用户已定」（L5），另带 L6 几句。辩护对象：第一轮被攻的形态（F1–F10）及它收严后的 G1–G14。
基线：`git rev-parse HEAD` = 31e47115471ba5502a4531d25dffcc921be36899 加工作区；开工 `sha256sum -c` 快照两份清单全过（crates 91 份、kb 5 份）。收工那一次见末节「没做什么」。
中途收到一条消息：线程上限改成 6。收到时我在读 `verdict_store.rs`，之后没起任何编译或测试；没有跑过 checker 档、GPU、门禁阶段。

## 0 先读这里（七条）

1. **第一轮判决二十来格里，打中的几格大多站得住，没有一格我能用今天的实现推翻。** 值得改的是判决「怎么改」那半句，不是「打中」那半句（第 1 节表）。
2. **判决 K3-1 的改法（G8 照抄的版本）自己打中 K1-2（G3/G4 的前缀共享）**：今天判法版本是整条流水线一份（`crates/singlefs-checker-tier/src/crash_amplification.rs:738`：「流水线用的判法摘要：一份、对全部崩溃点相同」），块键里没有版本（`crates/singlefs-checker-tier/src/verdict_store.rs:100`：「pub struct VerdictBlockKey」，四个字段），版本另接在后面。G8 把版本写成「这一项测试文件出发的闭包」，每项一个版本，同一个前缀块在每项下各存一行，等于没共享。出路：版本按节点取（见 2.1）。
3. **G8「∪ 引擎」若字面取整个 `crash_amplification.rs`，过敏且违背用户 2026-09-29 的拆分**：该文件 8122 行，其中 2837 行是 `mod tests`；词法摘要不跳过测试模块（`crates/singlefs-checker-tier/src/crash_identity.rs:275`：「去空白、行注释、块注释（可嵌套）与全部文档注释」）。改一条流水线单测就让 37 亿态整域重判。
4. **G7（计数随块存、每趟都判）与 G9（提交只等 GPU）撞：GPU 的结论字只有违例位，没有「判到 / 不适用」**（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:14`：「第 8..12 字池级 checker 的违例位」；`:68`：「部分没判与不适用一样不置违例位」）。逐条不变量的判到 / 不适用计数只有 CPU 一腿给得出，提交时 CPU 腿没跟上，K1-1 要补的空真守门在提交那一趟同样是「本次未跑」。
5. **G9 把「提交不等」从用户说的「CPU 对拍」扩到了「要真跑实现的观察者」**（逐状态可写挂载在内），用户弹窗只定了前者；而且没有定义「GPU 腿已存、CPU 腿还没跑」的账：块键加版本相同就复用，下一次提交会把 GPU 判过的块当作已判，CPU 对拍永远排不上。
6. **G5 末句（随机历史两项的镜像那一判上 GPU，「从它们录下的写流建判表」）今天读码看建不出**：建表入口只认三种展开，表达不了「一段里随机抽的一个子集」；两项自己的判又不是 `Layer0StateJudgement`。该句应当是「待核」，退路是 G5 自己的豁免表。
7. 第一轮两腿对一件事的说法相反：正推说闭包里没有 `core::mount`、`instance_table_sweep`；攻方那份被判不采的探针说有。**读码站攻方一边**（第 3 节）。判决 K3-1 没用到这句，不受影响；G8 受影响的是「测试文件闭包」那一截比想的小。

## 1 L4：第一轮判决逐格复核

判定三档：站得住 / 够不着 / 同样打中别的臂。「改法」一列单列，因为「打中」与「怎么改」在好几格上不是一回事。

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|
| K1-1 计数断言（打中） | **站得住**；任何带复用的替代方案都同样中（乙「只冷跑判计数」也在剪枝那趟失效） | 库里每个状态只存红项：`crates/singlefs-checker-tier/src/crash_amplification.rs:1162`：「EncodedVerdictVector(items.join」。`crates/singlefs-checker-tier/src/crash.rs:1274` 的文档写着「读方现算红绿与计数」，代码并没有存那份原始判定，文档与实现不符。改法不能靠「每状态多存一份原始判定」：判定向量编号是 u8，`crates/singlefs-checker-tier/src/verdict_store.rs:381`：「MoreThan256VerdictVectorsNotDecidedInTheFirstVersion,」，只能是 G7 写的「每块旁一份计数向量」那种旁表 | 找到今天某处已把绿状态上的分类计数存进库（`verdict_store.rs` 里 grep「计数」「tally」命中 0 处，数出来的） |
| K1-2 前缀共享下逐状态可写挂载（打中） | **站得住**（用户已定，辩方不翻）；但「92.8%」靠一句记录，没在同一判法版本下量过 | 数出自 `records/2026-10-01-删除与并发事务.md:1286`：「没扣掉与多记录流共享、判定库可复用的 1 358 954 604 态」；今天那一趟删除流复用 0（`records/2026-10-01-删除与并发事务.md:1292`：「reused=0 red=0」），原因是两项之间判法版本换了。共享在同一版本下是否成立没量过 | 同版本下跑多记录流再跑删除流，删除流 `reused` 不是约 13.6 亿 |
| K1-3 逐状态比什么（定义缺口） | **站得住，判轻**：写死「红项与恢复字都比」补上了落到的根，没补全 | `crates/singlefs-checker-tier/src/crash_amplification.rs:1788`：「pub fn recovery_words_of」只出四个数（看 journal 那遍的实例代号与 txg 低 32 位、两遍的结局类别）；GPU 结论字里恢复内核占 8 个（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:13`：「第 4 / 5 字它的走读结局与失败点」），第 5、7 字（两遍的失败点）不在这四个里，不看 journal 那一遍落到的根在 GPU 结论字里本来就没有这个字 | 造一个状态：两腿四个字相同、失败点或不看 journal 那遍的根不同，对拍仍绿 |
| K1-4 σ（没打中） | 无异议（我也没造出反例；读码，没跑） | 同判决 | 在 σ 历史上逐个比两边持久集合，层 0 有放量没有的 |
| K2-1 整条流记本次未跑（打中，小） | **站得住，机制现成** | 盖不住的段今天就按「不展开」处理（`crates/singlefs-checker-tier/src/crash_amplification.rs:172`：「if segment.len() <= self.0 {」，否则 `crates/singlefs-checker-tier/src/crash.rs:1869`：「Self::NotExpanded => 0,」），测试里预设的 ignore 是按点的（`crates/singlefs-checker-tier/src/crash_amplification.rs:194`：「pub ignored: bool,」）；G10 的「按段登记」用现有的点级 ignore 就能做 | 位置寻址第三条流的小段在展开 30 下今天并不被判（那就是判决说的「丢」不存在） |
| K2-2 第二条流两套造法（没打中，采做法） | 无异议 | 同判决（两腿都读码） | 两份造法在同一树上写表摘要不同 |
| K2-3 删除流差 15728640 | **站得住**；G13 要补的探针段形那一行只是注释 | `scripts/crash/crash-amplification.sh:42` 是注释行，不是运行时输入 | 放量那一项在今天的树上 `states` 不是 1463812278 |
| K3-1 判法版本罩不住引擎与观察者（打中） | **「打中」站得住；「改法」不是唯一不被打中的，字面照做还自相矛盾**（2.1、2.2） | 入口只有两份（`crates/singlefs-checker-tier/src/crash_identity.rs:839`：「pub const JUDGE_ENTRY_FILES」），流水线自己写明不罩（`crates/singlefs-checker-tier/src/crash_amplification.rs:739`：「设点的测试文件、流水线自己变了，摘要不变」）。乙（手抬版本号）同样被「忘了抬」打中；甲照字面（每项一个版本、∪ 整个引擎文件）被 K1-2 与用户 2026-09-29 的拆分打中 | 2.1、2.2 里的两条反例：造两项共前缀的流看它们的块键与版本；改 `crash_amplification.rs` 一条单测看摘要动不动 |
| K3-2 两份文件头口径（采） | 无异议 | 同判决 | — |
| K3-3 随机历史键没定义（打中） | **站得住，机制有现成一半** | 今天登记的崩溃注入用例的闭包里就收了随机历史脚手架与 `crash_injection.rs`（`crates/singlefs-checker-tier/src/crash_identity.rs:890`：「pub const RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED」那张白名单，用例 `crash_injection_campaign`），键里缺的是种子与抽法参数（`crates/singlefs-checker-tier/src/crash_injection.rs:226`：「/// 按种子抽这么多个（每个是「一段 + 那一段的一个真子集」」）。G8 写的「种子 + 规模参数 + 整份闭包」够；抽法（`CrashPointDraw` 三种）要算规模参数 | 白名单里的闭包算出来不含 `crash_injection.rs` |
| K4-1 按项登记太粗（打中） | **站得住**（用户已定按判法分界） | 同判决 | — |
| K4-2 个别状态判不了（定义缺口） | **站得住** | `crates/singlefs-checker-tier/src/crash_amplification.rs:2033` 是整趟停；G6 改成记不一致判红 | — |
| K4-3 io_min、清扫建不建得出表（核不动） | **仍核不动；G5 已把它当成立** | 本腿不跑 GPU | 在卡上对两流跑 `build_judge_tables` 看 `JudgeTablesRefusal`（命令在末尾） |
| K4-4 卡用不上（定义缺口） | **站得住**；用户 2026-09-29 原话支持退 78 | `scripts/crash/crash-amplification.sh:28`：「不接受默认降级」 | — |
| K5 提交代价（给数） | **站得住但口径要补一句**：弹窗上的 21.6–124.4 小时是「每个状态 CPU 判一遍」，不含逐状态可写挂载 | 攻方模型把删除流挂载另列约 50.8 小时（推的），正推下界两台 8.4 小时；弹窗交代（判决第三节）没提挂载 | 量出今天树上一次可写挂载的单态成本 |
| K6-1 重型闸按名字认（打中） | **站得住** | `.claude/hooks/lib_heavy_tests.py:585`：「CRASH_AMPLIFICATION_SCRIPTS = frozenset」（行号现查，T3 在改这份，别钉） | — |
| K6-2 漏 `--quick`（打中，小） | **站得住**；G2 已补 | `.claude/singlefs-ai-sop/rules/test-script-discipline.md:21`：「全量：快档与慢档的全部项」 | — |
| K6-3 D13 层表（采） | **站得住，判轻**：名字撞车没解 | G12 只说层表留「范围」一列，没说三层的名字要不要换；而「层 0」按 G1 退役、`.claude/kb/term-renames.md:4`：「除 `.claude/term-rename-exempt` 登记的几处之外出现旧名就判红」；D13 层表第一行就叫「层 0 冒烟」（`.claude/kb/decisions/13-验证路线.md:244`：「| 层 0 冒烟 |」） | 在 `term-renames.md` 登记旧名时给「层 0」写成只命中装置名的匹配（整串匹配做不到） |
| K7-1 与 m3-batch52-54 快照重叠 12 份（打中） | **已过期**：那一轮已写判决（`research/prompts/m3-crash-enumeration-merge-r1-main-verification.md` 之后），文件 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 在；与这一轮快照仍重叠 12 份文件但不再挡派发。现在挡的是第 56 批（还没有开工快照）与这一轮自己的快照 | `comm -12` 两份清单 = 12（本腿现数） | 第 56 批开工快照出来后再数一次 |
| K7-2 改英文标识符碰 E171 并换判法版本（打中） | **站得住** | 词法摘要保留标识符（`crates/singlefs-checker-tier/src/crash_identity.rs:275`：「去空白、行注释、块注释（可嵌套）与全部文档注释」，标识符不在去掉的里面） | — |
| 快档的域（打中，小） | **站得住** | 正推探针：第二条流甲二 278 对展开 6 的 261（判决已采） | — |

## 2 L4 补：判决 K3-1 改法的两处反例（对应 0.2、0.3）

### 2.1 每项一个版本，与前缀共享互相打中

- 事实：今天版本是一份（`crates/singlefs-checker-tier/src/crash_amplification.rs:738`：「流水线用的判法摘要：一份、对全部崩溃点相同」），块键是四个字段（`crates/singlefs-checker-tier/src/verdict_store.rs:100`：「pub struct VerdictBlockKey」），版本是读写时另接的后缀。两项共前缀 = 同一个块键，版本相同才命中同一行。
- G8 写的是判器闭包再并上「∪ 这一项测试文件出发的闭包 ∪」再并上引擎与观察者代码（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:30`：「∪ 这一项测试文件出发的闭包 ∪」）。若「这一项」读成每项一个版本，多记录流与删除流的公共前缀块各存一行，G3 的「身份与前缀共享」没了；这与 K1-2（用户已定）正面冲突。若读成每个节点一个版本，G8 要写清。
- 现成的节点级入口：每个崩溃点带设点的文件（`crates/singlefs-checker-tier/src/crash_amplification.rs:190`：「设这个点的函数所在的文件（仓内路径）」，字段 `code_file_in_repository`，公共前缀的点写在公共模块里），按它出发的闭包取版本，公共前缀的点自然共享，删除流自己的尾段才带删除流的测试文件。这个改法对 K3-1 的造出来的历史（改观察者整域照复用）同样不中，对 K1-2 也不中。是辩方的替代，没被攻过，零轮。
- 推翻它的现象：两项各自的版本在同一棵树上不同、而公共前缀块在库里只有一行。

### 2.2 「∪ 引擎」取整个文件过敏

- `crates/singlefs-checker-tier/src/crash_amplification.rs` 共 8122 行，`#[cfg(test)]` 的 `mod tests` 从第 3575 行到第 6411 行（`crates/singlefs-checker-tier/src/crash_amplification.rs:3575`：「mod tests {」），2837 行；摘要取全文件词法单元，不跳过测试模块（`crates/singlefs-checker-tier/src/crash_identity.rs:275`）。非测试部分里还有显存账、领块、对账（文件头第 5 点）这些不改结论的流水线。
- 用户 2026-09-29 的原话在 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:19`：「派活和判法拆出来最好」，今天的契约测试就守着它：`crates/singlefs-checker-tier/src/crash_identity.rs:1437` 的 `the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge` 断言改 `crash_amplification.rs` 摘要不动（`crates/singlefs-checker-tier/src/crash_identity.rs:1464`：「"crates/singlefs-checker-tier/src/crash_amplification.rs",」在那张「不该动」的表里）。G8 字面照做，这条测试要反过来改。
- 真正改结论的只有把判器字变成存进库的东西那几处：`crates/singlefs-checker-tier/src/crash_amplification.rs:1162`（红项编码）、`crates/singlefs-checker-tier/src/crash_amplification.rs:1788`（恢复字）、GPU 结论字换红项那个函数。辩方替代：把这几处搬进判器那一侧的文件（加进入口），其余留在流水线；这是用户已经认过的拆法。零轮。
- 推翻它的现象：把这几处搬走之后，仍有一条改流水线其余部分就改变存进库的结论的路径（例如分块切法改了块内序号）；那时块键里的计划哈希会变（`crash_amplification.rs` 文件头第 1 点），可以核。

## 3 两腿说法相反的一件事：再挂载代码在不在判法闭包里

- 正推（`research/prompts/m3-crash-enumeration-merge-r1-sonnet-output.md`）3.1 节写「`crash.rs` 与 `crash_judge_gpu.rs` 都不 `use` `core::mount` 与 `instance_table_sweep`……闭包里没有它们」，依据是对这两份文件直接 grep 0 处。闭包是传递的，直接 grep 不够。
- 读码：解析器对任何 `名::名` 路径表达式都收（`crates/singlefs-checker-tier/src/crash_identity.rs:699`：「fn references_of(」，路径与 `use` 同走）；闭包里的 `recovery` 引 `transaction`、`allocator`（`crates/singlefs-core/src/recovery.rs:89`：「use crate::transaction::{」，`crates/singlefs-core/src/recovery.rs:37`：「use crate::allocator::{」）；`transaction.rs` 非测试部分（`#[cfg(test)]` 在第 9469 行）第 1873 行写 `crate::mount::`（`crates/singlefs-core/src/transaction.rs:1873`：「crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(」），`allocator.rs` 第 2352 行同（`crates/singlefs-core/src/allocator.rs:2352`：「let reclaim_floor = crate::mount::reclaim_floor(」，测试模块在第 2561 行之后）。所以 `mount.rs` 在判法闭包里，`mount.rs` 引 `instance_table_sweep`、`unlink`，也在。
- 我另写了一个 60 行的 python 近似（模块级、去注释、按同一类路径收，草稿 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r2-sonnet/clos.py`）：入口两份、停止三份，收到 core 的 `mount`、`unlink`、`instance_table_sweep`、`admission`、`mounted_session` 等，与攻方探针汇总行一致；近似比真工具宽（真工具另有 `impl` 头收法），所以只当旁证，不当量过。
- 相反的证据我要摆出来：契约测试的说明写着「实现的写路径……都不在」（`crates/singlefs-checker-tier/src/crash_identity.rs:1343`：「实现的写路径、装置二进制都不在」）。「写路径」指哪几份没写明；它没有断言 `mount.rs` 不在。
- 影响：①「改了挂载代码而判法版本不换」今天不成立，所以判决 K3-1 不依赖这点是对的；② 反过来，改 `mount.rs`（7000 多行）、`transaction.rs` 任何一行都让整域换版本，K5 的「改到 core 的提交 = 整域重判」比想的更常见；③ G8 里「测试文件出发的闭包」比它看起来小：核心实现已经在判器闭包里，多出来的是测试文件、`common_crash_points` 与随机历史那部分。
- 推翻它的现象：主 agent 跑一次 `the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline`（`--nocapture`，打 `JUDGING_CODE_FILE` 行），`singlefs-core/src/mount.rs` 那一行不在。命令在末尾。

## 4 L5：G1–G14 对第一节「用户已定」

用户已定的各句，我按 `research/prompts/_m3-crash-enumeration-merge-r2-body.md:9` 那一行拆成 U1–U9：U1 要合；U2 快档与整域都合；U3 每次提交跑整域，剪枝或全量二选一；U4 CPU 对拍不抽样、日常剪枝跑、逐项可跑；U5 提交只等 GPU，CPU 对拍日常剪枝跑完；U6 逐状态可写挂载挂在标准前缀上、不得例外；U7 判一张镜像的全上 GPU、要真跑实现的只能 CPU，按判法分界、不按项开例外；U8 多机配置合成一份、默认开 GPU；U9 这些测试问题在 E171 冻结和提交前整域之前解决。

判定：照办 / 违背 / 绕开 / 照漏。没有一条「违背」；有三条「绕开」风险和六条「照漏」。

| G | 对哪句 U | 判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|
| G1 名字 | U1、U9 | 照办；**照漏**两处 | 改名射程只写了装置名、门禁名、脚本名、英文标识符（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:23`：「- G1 名字」）。没写：环境变量名（`SINGLEFS_LAYER0_*` 十几个，正推第一轮 6.2）、库目录默认路径（改了就是空库 = 下一次冷跑）、仓外的私有配置与第二台上的目录；D13 层表里的「层 0」（K6-3 行） | 全仓扫描旧名后上面三类都已在 G1 的清单里 |
| G2 一张表一个入口一道门禁 | U1、U2 | 照办 | 入口认 `--quick` 与「不带参数」，与规则一致（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:21`：「全量：快档与慢档的全部项」） | — |
| G3 一套引擎 | U1 | 照办 | 同判决 K2-2 | — |
| G4 观察者挂节点 | U6 | 照办字面；与 G8 的版本取法**矛盾**（2.1） | G4 写「观察者的代码进节点的计划哈希」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:26`：「观察者的代码进节点的计划哈希」），G8 又把观察者代码放进判法版本（第 30 行）。进两处，且若版本按项取，节点就不再共享，U6 的「标准前缀」落空 | 公共前缀节点在删除流与多记录流下是同一块键同一版本 |
| G5 能走 GPU 必须走 | U7 | 照办；末句**未核** | 末句：「随机历史两项的镜像那一判同样上 GPU（从它们录下的写流建判表）」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:27`）。建表入口 `crates/singlefs-checker-tier/src/crash_judge_tables.rs:200`：「pub expansion: &」只认三种展开（`crates/singlefs-checker-tier/src/crash.rs:1841`：「InPlaceSubsetsWithCopyOnWriteNoneOrAll,」是第三种），随机历史的崩溃状态是一段加一个抽出来的真子集（`crates/singlefs-checker-tier/src/crash_injection.rs:4032`：「fn draw_a_proper_subset(」），不是一种展开。两项的判也不是 `Layer0StateJudgement`：池级 checker 之外还有对理想模型的比较、记录核对器、已知红清单（`crates/singlefs-checker-tier/src/crash_injection.rs:4099` 的 `crash_state_observation`），自己还记逐条不变量的判绿 / 不适用（`crates/singlefs-checker-tier/src/crash_injection.rs:4058`：「tally.invariant_not_applicable」）。要上 GPU 得先加「给定状态表」那一种展开，G14 第三批没写 | 在卡上对一份随机历史的写表建判表成功，且逐状态与 CPU 对上。建不出则按 G5 自己的豁免表登记，不违背 U7 |
| G6 CPU 对拍 | U4 | 照办；比的字不全 | 见 K1-3 行：四个字，不含失败点两个字 | 造四字相同、失败点不同的状态，对拍仍绿 |
| G7 计数随块存 | U3、U5 | **绕开**（与 G9 撞） | 见 0.4：GPU 腿只有违例位。提交时能每趟判的只有闭式与按发布分的态数（结构量）；逐条不变量的判到 / 不适用计数、观察者计数要 CPU 腿。成功那句若写成「计数断言每一趟都判」就是假话 | GPU 内核补上「判到 / 不适用」的输出字（G14 第三批没写） |
| G8 判法版本 | U3、U6 | 见 2.1、2.2 | — | — |
| G9 两档 | U3、U5、U6 | **绕开**两处，**照漏**一处 | 见下三条 | 见下三条 |
| G10 装不进 u64 的段 | U3、U6 | 不构成「例外」；**判据偏松** | 登记的是做不到的段，不是选择；按段登记，用现成点级 ignore（K2-1 行）。判据写「装不进 u64」，而 u64 装得进的段（三十五写到六十三写）早就判不动：今天用「展开上限」登记（`scripts/crash/crash-amplification.sh:219` 的 `FULL_EXPAND_UP_TO=30`），`crates/singlefs-checker-tier/src/crash.rs:1860`：「2 态的写六十几个就到了」是 panic 的边，不是可行的边 | 加一条写数 31–63 的段，整域在 G10 字面下还是排它去判 |
| G11 跑在哪 | U8 | 照办；**前提已大半成立** | 两个入口今天就读同一份多机配置（`scripts/crash/layer0-shard-configuration-check.sh:33`：「配置文件：${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}」，`scripts/crash/crash-amplification-lib.sh:127`：「configuration_file="${SINGLEFS_MULTI_HOST_CONFIG:-$repository_root/multi-host.env}"」），两个开关键只在一处定义（`scripts/crash/layer0-shard-configuration-check.sh:20`：「两个开关键只在这一份里定义（别处照这里读，不另写一份）」）。真要合的是两套双机驱动与 GPU 卡配置；「默认 `ENABLE_GPU=1`」与今天「没写或写 0 算关」（`scripts/crash/layer0-shard-configuration-check.sh:22`：「GPU：写 1 算开，没写或写 0 算关」）相反，且配置是 git 忽略的私有文件，仓内改不到。无卡的机器每趟快档都退 78，与规则的整脚本口径相合（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:80`：「不悄悄退回 CPU 或单机」），但与规则对个别项的口径（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:53`：「要特定硬件的那几条，在没有那种硬件的机器上报」「本次未跑」）不同，G11 要写明取哪个 | 默认值改在 `.example` 模板与检查脚本里，用户确认无卡机器退 78 |
| G12 条款 | U2、U3 | 照办；时序见 L6 | D13 层表「层 0」撞名（K6-3 行） | — |
| G13 旧数 | U9 | 照办 | `scripts/crash/crash-amplification.sh:42` 是注释行 | — |
| G14 切批 | U9 | 照办；L6 另述 | — | — |

### G9 展开（三条）

1. **绕开 U5 的范围**：用户弹窗问的是「CPU 对拍什么时候跑、提交等不等它」，答「提交只等 GPU，CPU 对拍日常剪枝跑完」（`research/prompts/m3-crash-enumeration-merge-r1-main-verification.md:48`：「问「CPU 对拍什么时候跑、提交等不等它」」）。G9 把「提交不等」写成「CPU 那一截（对拍与要真跑实现的观察者）日常剪枝跑」（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:31`：「CPU 那一截（对拍与要真跑实现的观察者）日常剪枝跑」），连 U6 的逐状态可写挂载一起让到提交之后，并且只报「CPU 对拍本次未跑完」。按判决第三节所记，第二次弹窗只定了挂在哪、没记提交等不等。这一扩要么再弹一次，要么把成功那句写成「CPU 对拍与逐状态可写挂载本次未跑完」。判决第三节引的 K5 数（21.6–124.4 小时）也不含挂载（K5 行）。
2. **绕开 U4 的「剪枝」**：G9 写「键与判法版本都同才复用、与已判等价的才剪，等价关系有用例钉住」。今天没有任何「等价」剪枝（`crates/singlefs-checker-tier/src` 下 `crash_amplification.rs`、`crash_identity.rs`、`verdict_store.rs` grep「等价」「equivalen」「dedup」命中 0 处），所以这句要么是空话（只剩键同），要么偷偷新增一个功能；规则要求等价关系写成用例（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`：「剪掉的只能是两种：键相同、已经判过的；与已判状态等价的。等价关系写成用例钉住。」）。
3. **照漏：腿账**。块键加版本相同就复用（G8），而 G9 让 GPU 腿与 CPU 腿先后到：GPU 判过的块下一次提交被当「已判」，CPU 对拍与观察者没有「已跑」的记号可查，永远排不上。需要一份按（块键，版本，腿）记的账，或者把腿作为存储键的一维；两者 G7/G8 都没写，且判定向量编号只有 u8（`crates/singlefs-checker-tier/src/verdict_store.rs:163`：「pub struct VerdictVectorNumber(u8);」）装不下，只能是旁表。

## 5 L6（顺带）：G14 切批

- **撞在飞的工具批**：第一批要改入口、重型闸、登记表、门禁（G2），正文第二节说 T3 正在改同一批文件（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:18`：「T3 在改 `scripts/gate/admission.py`」），G14 没写等 T3 合入。
- **条款先于做法**：G14 第一批含 G12 的条款（`research/prompts/_m3-crash-enumeration-merge-r2-body.md:36`：「不碰 `crates/`」），那一批合入后，D13、`verification.md` 写的是「每次提交跑整域」，引擎要到第二、三批才有两档。第一轮正推腿的排法是条款最后（S4，依赖入口已切换）。中间那几次提交按条款要跑整域，而入口背后仍是两套旧测试：「不带参数 = 快档加整域」会把同一条流判两遍，正是用户 U1 要避免的「下一次又要重跑」。出路：条款挪到第三批之后，或者第一批的条款写成「本批至第四批之间的提交不跑整域，记一笔」，与 U9 一致。
- **第一次整套能跑**：快档要 G6（CPU 对拍）与 G5（GPU 判镜像）两头都有，第三批合入之后；整域要等第四批（改名换版本）之后才跑一趟，G14 末句已写。第二批与第三批各换一次判法版本（G8 把引擎并进版本，第三批动判器），不是「只重判一次」，只有这两批之间不跑整域才成立。
- **中间态**：第一批不碰 `crates/` 时，计数行前缀 `LAYER0…` 仍由旧测试打，门禁与 `admission.py` 照旧读，今天两套装置跑得起来；风险只在 G2 要同批改重型闸（`.claude/hooks/lib_heavy_tests.py:585`：「CRASH_AMPLIFICATION_SCRIPTS = frozenset」）：新入口名与旧入口名要同时登记到第四批才能删旧名。

## 6 要主 agent 跑的命令（本腿一条没跑）

1. 核 3 节（`mount.rs` 在不在判法闭包）。工作目录：仓根；checker 档单测，要带 `SINGLEFS_HEAVY_TESTS=user-request`：
   `SINGLEFS_HEAVY_TESTS=user-request bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 cargo test --release -p singlefs-checker-tier --lib the_judging_closure_of_the_judges_holds -- --nocapture 2>&1 | tee <日志>`
   看：`grep -c '^JUDGING_CODE_FILE' <日志>`、`grep 'singlefs-core/src/mount.rs\|singlefs-core/src/unlink.rs\|singlefs-core/src/instance_table_sweep.rs' <日志>`。
2. 核 K1-2 的共享是否在同版本下成立：同一个判法版本下先跑 `multi-record-publish` 再跑 `unlink-after-the-multi-record-publish`（展开 30），读两行 `CRASH_AMPLIFICATION_ITEM ... reused=`；删除流的 `reused` 不是约 13.6 亿则 92.8% 不成立。命令形态：`bash scripts/crash/crash-amplification.sh --item multi-record-publish` 然后 `--item unlink-after-the-multi-record-publish`（带 `SINGLEFS_HEAVY_TESTS=user-request`，GPU 清场与内存包装由脚本自己套）。
3. 核 G5 末句：在一张卡的机器上，对一份随机历史的写表（`run_crash_injection_campaign` 录下的）按 `crates/singlefs-checker-tier/src/crash_judge_tables.rs:194` 的 `JudgeFlow` 建判表，看能否表达「抽出来的一个子集」；与第一轮攻方报告「要主 agent 跑的命令」第 2 条（io_min、清扫）合并跑。

## 7 没做什么

- 没跑 checker 档、GPU、门禁阶段、崩溃放量；没编译任何 Rust；没跑模型（草稿目录只有我写的 `clos.py`，纯文本近似，结论只当旁证）。
- 没复核第一轮两腿的 K5 模型数（L2 归攻方）；没核 `scripts/experiments` 与 `research/results` 里的旧名归类。
- 没替主 agent撤销或维持判决：上面「站得住 / 判轻 / 已过期」只是复核结果。
- 「辩方替代」两条（2.1 节点级版本、2.2 把结论编码搬进判器一侧）没被攻过，零轮；它们是我提的收严，只在读码上推过。
- 收工 `sha256sum -c`：crates 91 份通过；kb 清单 `.claude/kb/checks-owed.md` 在我跑的过程中变了（开工时全过，收工时该份不符）——不是我改的，我也没引它。
- 仓内无副本、无编译目录；草稿 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r2-sonnet/clos.py` 留着（不删）。
