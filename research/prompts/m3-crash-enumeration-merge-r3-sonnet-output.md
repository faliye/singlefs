# m3-crash-enumeration-merge-r3 辩方腿（Sonnet）报告（2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 H1 H2 H3 H4 H5 H6 H7 H8 H9 H10 H11 H12 H13 H14 L1 L2 L3 L4 L5 L6 M1 M2 M3 M4 M5 M6 K1 K2 K3 K4 K5 K6 K7 W -->

立场：辩方。格 M5（复核第二轮判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 逐格，并查 H1–H14 有没有照错、照漏它）与 M6（H1–H14 对正文第一节「用户已定」）。不读禁读清单里的攻方报告。
基线：开工 `sha256sum -c --quiet` 两份快照清单（`research/prompts/m3-crash-enumeration-merge-r3-snapshot/crates-sha256.txt`、`kb-sha256.txt`）都退 0（crates 91 份、kb 4 份）；收工那一次见末节。
中途收到主 agent 一条定义改动通知（`implementation-writer.md` 加一句，只管实现员，两份文件不在快照里）：与本腿无关，照原样做完。
没有编译任何 Rust，没有跑 checker 档、GPU、门禁阶段、崩溃放量；草稿目录 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-sonnet/` 空着。
引 kb 条款写 kb 文件名加行号，行号是在 kb 文件里 `grep -nF` 现查的；用户原话优先引调度记录 `records/2026-09-30-里程碑三开工第一轮.md` 的行，正文里的原话在记录里找不到的单列（M6）。

## 0 先读这里

1. M5：第二轮判决每一格「打中」那半句，我没有一格能用今天的树推翻（逐条 `grep -nF` 重核了判决第一节第 ②–⑥ 格的代码事实，全部还在）。站不稳的是「怎么改」的几处，和 H 照抄时丢的东西（第 1、2 节）：
   - 判决 L3-1「编译时算进二进制（`build.rs`）」没核可行性：包里今天没有 `build.rs`，算闭包的 `crash_identity.rs` 自己引 `singlefs_harness` 与 `crate::layer0_progress`，`build.rs` 调不到它（M5-14）。
   - H8 把判决 L3-4 的「驱动版本与卡型」写成「驱动与 wgpu 版本」，丢了卡型（M5-17）。
   - 判决 L6-3 与第一轮 K4-3 的「进实现前事实核」两件（随机历史判表建得出、io_min 8192 与清扫小环池判表建得出），H1–H14 一句没带（M5-19）。
   - H7 把判决 L1-4 的「第三批之前提交那一趟逐条计数报本次未核」收窄成「对照被授权跳过的那一趟」：站得住，因为 H6 让 CPU 在 GPU 机器上默认也判（M5-18）。
2. M6：H1–H14 没有一条正面违背用户原话。有五处「绕开 / 二读 / 照漏」要主 agent 现场定或问用户（第 3 节）：
   - H11「没写 `ENABLE_GPU` 算开」与 H5「没配 GPU 走 CPU 判器」同一份正文里互相打架，也与用户「没有配置gpu就cpu跑」有第二种读法（M6-6）。
   - H5 豁免表的「明确的豁免」有「登记过」与「用户明确授权」两种读法，今天的豁免行里「随机历史的记录核对器」按用户 2026-10-10 的话属判镜像一类（M6-4）。
   - H9「轻型是重型的子集」× H10「重型登记段不展开」：登记段在轻型的甲二域里若仍有态，子集就破（M6-10）。
   - H12 的条款清单漏 `.claude/agent-common.md` 第 52 行，那一句字面禁止子 agent 跑 checker 档，与用户「轻型是交给写代码的实现器和检测器去跑的」顶（M6-12）。
   - 用户 2026-10-10 弹窗答里的「2 硬盘重跑存入db」一步，正文第一节没抄、H1–H14 没处理（M6-7）。
3. 单样本：判决里「没打中」的 L3-5、K1-4 只有攻方一条腿的一次抽样（辩方那条腿没覆盖 L3），按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 记「单样本」，不支撑任何结论；H 没有哪条依赖它们。

## 1 M5：第二轮判决逐格复核

判定三档：站得住 / 够不着 / 同样打中别的臂；「轻重」另列。判决文件简称 V2（`research/prompts/m3-crash-enumeration-merge-r2-main-verification.md`），正文简称 B3（`research/prompts/_m3-crash-enumeration-merge-r3-body.md`）。

| 编号 | 判决条目（V2 行） | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|
| M5-1 | 第一节核查（第 13–17 行） | 站得住 | 第 ②–⑥ 格的代码事实今天原样还在：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3528`：「var remaining = ordinal_lo - tab(T_SEGMENTS, s * 6u + 2u);」；`crates/singlefs-checker-tier/src/crash_verify_gpu.rs:304`：「segment.states.end - segment.states.start < u64::from(u32::MAX),」；`crates/singlefs-checker-tier/src/crash_amplification.rs` 共 8122 行、`mod tests {` 在第 3575 行；`grep -c InPlaceSubsets` 在 `crash_judge_tables.rs`、`crash_amplification.rs`、`crash_judge_common.wgsl` 都是 0（本腿现跑）；`crates/singlefs-core/src/transaction.rs:1873`：「crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(」 | 第 56 批合入后这几处行号移动或内容变（快照 91 份此刻全过，所以今天不成立） |
| M5-2 | L1-1、K3-1 改法（V2:23） | 站得住：打中，改法也不被同样打中 | 改法 V2:23：「取设这个点的文件出发的闭包 ∪ 判器闭包」。H3 要共享的前提「B、C 两步写的东西相同」可核：`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:433`：「let second_content = content_needing(2, 1);」与 `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1099`：「let second_content = content_needing(2, 1);」，第三步同为 (3, 2)（两文件各 437、1103 行）；起点同走 `common_crash_points::record_pool_build`（431、1097 行）。块键是 (流程输入指纹, 节点名, 计划哈希, 块起点)，节点名带设点文件（`crates/singlefs-checker-tier/src/crash_amplification.rs:253`：「enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),」），所以 B、C 设点文件换成公共模块后两流身份同、版本同。轻重：计划哈希已罩判器另读的流程输入（`crates/singlefs-checker-tier/src/crash_amplification.rs:233`：「/// 判器判这个点的状态时另读的流程输入的摘要（`JudgeInputsOfTheFlow::digest_for_a_point`）；进 `plan_hash`。」），所以 CPU 版本里的「设点文件闭包」对判定输入是兜底、偏保守（公共测试模块 `#[path]` 一变，整域 CPU 一侧跟着重判）；不判错，`.claude/agent-common.md:59`：「**崩溃点测试准确率和正确率优先，其次再衡量时间成本**」，不建议收窄 | 两流 B、C 的写表摘要不同（那时挪进公共模块也不共享）；H3 的「两份造法写表摘要相等」快用例要把 B、C 两步也比进去才测得到 |
| M5-3 | L1-2、K3-1（V2:24） | 站得住，补一处机械后果 | 派活不进闭包的契约在 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:19`：「//! 几张卡怎么用，在 `crate::crash_judge_dispatch`——那一份不在判法闭包里（用户 2026-09-29 定「派活和判法拆出来最好」），」。判决要搬进判器一侧的红项编码今天是 `crates/singlefs-checker-tier/src/crash_amplification.rs:1161`：「fn encoded_verdict(items: &[String]) -> EncodedVerdictVector {」，返回类型在 `crates/singlefs-checker-tier/src/verdict_store.rs:158`：「pub struct EncodedVerdictVector(pub Vec<u8>);」。连类型一起搬，闭包按模块引用会把存储模块 `verdict_store.rs` 拉进判器版本（该文件不在停点名单里，名单见 `crates/singlefs-checker-tier/src/crash_identity.rs:839`：「pub const JUDGE_ENTRY_FILES: &[&str] = &[」那一段）；搬时只让判器一侧交字节、流水线包类型即可。恢复字 `crates/singlefs-checker-tier/src/crash_amplification.rs:1788`：「pub fn recovery_words_of(judgement: &crate::crash::Layer0StateJudgement) -> [u32; 4] {」只依赖 `crash.rs`，已在闭包里 | 搬完后判器闭包里出现 `verdict_store.rs`（闭包契约测试或 H2 门禁会红） |
| M5-4 | L1-3（V2:25） | 站得住 | 改法 V2:25：「观察者挂在哪个节点由登记表静态列出，单跑一项照表跑它经过的节点上登记的全部观察者」；H4 照抄且加「观察者结果按（块键，观察者版本）存」 | 单跑一项与整域在同一节点上跑的观察者集合不同 |
| M5-5 | L1-4、辩方 G7（V2:26） | 站得住 | GPU 结论字没有判到 / 不适用位：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:13`：「//! 实例代号与 checkpoint_txg 低 32 位；第 4 / 5 字它的走读结局与失败点；第 6 / 7 字不看那一遍的；」（字布局 8–14 行，只有红位、违例位、走到第几步、判不了原因） | 找到 GPU 结论字里已有的逐条判到位 |
| M5-6 | L1-5（V2:27） | 站得住；同类先例 | 规则只给上限不要求实现等价剪枝：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`：「剪掉的只能是两种：键相同、已经判过的；与已判状态等价的。等价关系写成用例钉住。」。`grep -c 等价` 在 `crash_amplification.rs`、`crash_identity.rs`、`verdict_store.rs` 都是 0（本腿现跑）。H8「不做等价剪枝」不违反这条规则 | 规则改成「必须有等价剪枝」 |
| M5-7 | L1-6（V2:28） | 站得住 | H6 的八个字 = 两遍各自的根两个字（实例代号、txg 低 32 位）+ 两遍结局 + 两遍失败点；GPU 缺不看 journal 那一遍的根两个字，布局见 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:13` 同段。只比 txg 低 32 位是判决接受的界 | 造 txg 高位不同、低 32 位相同的两个落点（现实里 txg 要过 2^32 次发布） |
| M5-8 | L1-7、辩方 G10（V2:29） | 站得住；轻重：必要项与预留要分开 | 必要项：判表打包不核段长，`crates/singlefs-checker-tier/src/crash_judge_tables.rs:1265`：「let (start_lo, start_hi) = split(segment.states.start);」；内核只用低 32 位，见 M5-1 第一条。预留项：lo / hi 满 64 位余数只有在登记的展开深度大于 32 时才用得到，今天展开上限是 30（`research/scripts/crash-amplification.sh:41`：「#   取值：登记分项里最长一段的写数。今天是 30 写」）。V2:29 把两件并列写，H10 照抄并列，不算错；要实现员分得清哪件是第三批必做 | 登记深度被用户或主 agent 定到 33 以上（那时 lo / hi 也是必做） |
| M5-9 | L1-8（V2:30） | 站得住 | H8「账按（块键，结论种类，版本）记」；恢复字另表：判定向量编号只有 u8（`verdict_store.rs` 的 `VerdictVectorNumber(u8)`，辩方第二轮已查） | 实现时发现旧库格式装不下三类账 |
| M5-10 | L2-1、L2-2、L2-3、L2-5（V2:31） | 站得住 | 判决自己写「都是推的，今天的树没量」；H13「第一轮判决 K1-2 的 92.8%、两轮的挂钟数作废」照办；B3 第二节「这一轮不引旧速率当依据」 | 今天的树上量出来的速率与旧数同量级（那只是旧数碰巧对，不改变它们不能引） |
| M5-11 | L2-6（V2:32） | 站得住 | 判决两条独立路；本腿补第三条：B、C 内容同（M5-2），只差设点文件，与判决「共享的只有取号暖机与 A」一致 | 挪进公共模块后删除流 `reused` 仍只有 16 777 259（H3 落地后第一次整域的 ITEM 行） |
| M5-12 | L2-4 与第三节三问（V2:33、52–58） | 站得住 | 三问用户都答了：挂载提交前要判（`records/2026-09-30-里程碑三开工第一轮.md:951`：「RUST实现必须检查，不检查如何保证代码是对的？」）；B、C 挪进公共模块；WGSL 与 Rust 两份怎么一致，用户挪到合谋（`records/2026-09-30-里程碑三开工第一轮.md:953`：「你如果说合谋就等会再说。」）。H1–H14 没有哪条再碰 L2-4 | 用户把 L2-4 拉回这一轮 |
| M5-13 | L3-2（V2:35） | 站得住 | `research/scripts/crash-case-check.py:95`：「ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)\s*(?:::<[^>]*>)?\s*\(")」：placement 按函数名认用例，名字退役后认不出，改名不同批就空判；H2、H1 都写了同批改 | 改名后 placement 仍认得出一项登记用例（那就是该格没中，改法多余） |
| M5-14 | L3-1（V2:34） | 「打中」站得住；「怎么改」机械可行性没核，够不着 | 改法 V2:34：「版本在编译时算进二进制（`build.rs` 照同一套闭包算）」。今天 `crates/singlefs-checker-tier/` 下 `ls build.rs` 报不存在，`Cargo.toml` 里 `grep -c build` 是 0（本腿现跑）。「同一套闭包」的实现 `crates/singlefs-checker-tier/src/crash_identity.rs:16`：「use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};」与 `crates/singlefs-checker-tier/src/crash_identity.rs:19`：「use crate::layer0_progress::Layer0ToolchainIdentity;」：它引了 harness 与本包另一个模块，`build.rs` 先于本包编译，调不到本包的库代码。出路只有两条：`#[path]` 把 `crash_identity.rs` 编进 `build.rs` 并给包加 harness 的 `[build-dependencies]`（harness 不依赖 checker 档，无环，但 harness 要为宿主多编一遍），或把闭包算法抽出一份小代码两处共用；后一条碰 `.claude/kb/decisions/13-验证路线.md:395`：「两档都要的一小段代码宁可各留一份」那一句的口径。替代方案（跑前跑后各算一次指纹，层 0 `--full` 今天的做法）解不了判决点名的「编完到现算之间被改」，所以判决选编译时算是对的，只是要把这一步算进第四批之前的工作量 | 实现员用 `#[path]` 加 `[build-dependencies]` 编过，并证明闭包里文件一变 `cargo` 会重跑 `build.rs`（`cargo:rerun-if-changed` 按算出的闭包逐个发） |
| M5-15 | L3-3、L3-4、L3-5（V2:36–38） | L3-3 站得住；L3-4 见 M5-17；L3-5 单样本 | 19 份首行数：对 `crates/singlefs-checker-tier/tests/*.rs` 每份取第一行，含 `layer0_progress` 的 15 份、含 `crash_amplification` 的 11 份、两者之一的 19 份（本腿现跑），与 V2:36 的「19 份」对上。L3-5 V2:38：「| L3-5 | 攻方 | 没打中 |」只有攻方一条腿一次抽样，辩方那条腿没覆盖 L3 | 另抽一次样发现旧库或旧全绿标记被新形态误复用 |
| M5-16 | L6-1…L6-5 与辩方 L6、K6-3、K7-1、G11（V2:39–48） | 站得住 | H12、H14 各自照办：第一至第四批照今天条款，与用户 `records/2026-09-30-里程碑三开工第一轮.md:932`：「合并之前不跑层 0 `--full` 与崩溃放量全量，合完只跑一趟」同向；H14 第一批等 T3a、T3b、G10 挪二批、判表段展开一列在三批，都在。L6-3 的「实现前事实核」H 没带，见 M5-19 | 无 |

### 1.1 H1–H14 照错、照漏第二轮判决

| 编号 | H 与判决 | 辩方判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|
| M5-17 | H8 对 L3-4 | 照漏（小） | V2:37：「GPU 判器版本带驱动版本与卡型」；B3:38：「驱动与 wgpu 版本」，没有卡型。卡型换了而驱动版本号相同（同驱动下换另一型号的卡，显存与绑定上限不同，判表装不装得下也跟着变，见 H6 的「写哪张表满了」）时，旧结论照复用 | 能证明判表装得下与否、结论字内容都与卡型无关（那时 wgpu 版本 + 驱动够，判决那半句多余） |
| M5-18 | H7 对 L1-4 | 站得住：H7 比判决收窄得对，但有一个后果要说破 | 判决 V2:26：「第三批之前提交那一趟逐条计数报」本次未核；B3:37 H7 收窄成：「第三批之前、配了 GPU 而对照被授权跳过的那一趟」。收窄的理由是 H6 让 CPU 在配了 GPU 的机器上默认也判（B3:36），逐条计数由 CPU 那一遍给出，判决那句绝对写法会在对照在跑时多报「未核」。后果：用户授权跳的是「对照」，H7 让这一次的空真守门也一起跳（计数守门函数 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:133`：「fn assert_checker_counts(」）；弹窗授权那一句要把这一条写进选项说明，不然用户授权的比实际跳的少 | 第三批前 GPU 结论字补上判到位；或用户说空真守门不随对照一起跳 |
| M5-19 | H1–H14 对 L6-3、第一轮 K4-3 | 照漏 | V2:41：「判表建得出与否进实现前事实核」；第一轮判决 `research/prompts/m3-crash-enumeration-merge-r1-main-verification.md:36`：「| K4-3 io_min 8192、清扫小环池建不建得出表 | 两腿核不动（不跑 GPU） | 核不动，进第二轮实现前的事实核 |」。B3 里 `grep -c 事实核` 是 0（本腿现跑）。登记表里就有这一条用例：`.claude/gate.d/stage-inputs.tsv:56`：「crash-case:writable-mount-of-an-io-min-8192-pool」。挡第三批（判表）与 H5 豁免表的内容（建不出就得多一行豁免） | 主 agent 在 H14 里写了这两件的事实核（命令、归哪一批） |
| M5-20 | H11 对 L3-3 | 站得住，一个句子没定 | V2:36：「配置显式 `ENABLE_GPU=0` 时挂特性的那几格报「本次未跑」、整域不认这一趟」；B3:41 改成「镜像照 H5 由 CPU 判」，整域不再因此不认，这与用户 `records/2026-09-30-里程碑三开工第一轮.md:951`：「并定判镜像照用户配置走：没配 GPU 就 CPU、配了分片就分片、有 GPU 就 GPU」同向，采 H11。没定的：这一趟「对照」不存在，成功句写什么（H6 只定了「用户授权跳过」那一种） | H11 补一句无 GPU 配置那一趟的成功句 |
| M5-21 | H2 对 G2（第二轮没攻） | 不一致，零轮 | B3:32 写「一项一条流」；登记表里的 13 条不是一条一流：`.claude/gate.d/stage-inputs.tsv:54`：「crash-case:layer0-tree-split-streams」、`.claude/gate.d/stage-inputs.tsv:55`：「crash-case:layer0-position-addressed-tree-streams」是多条流，随机历史两条（57、58 行）不是流。合表后项数不是 13 + 8，「项」要先定义（一条流？一个测试目标？） | H2 补「项 = …」一句 |
| M5-22 | H3、H4、H6、H9、H10、H12、H13、H14 对判决 | 站得住（逐条对得上） | H3 ↔ L2-6 与用户 B、C 共享；H4 ↔ L1-3；H6 ↔ L1-6、K4-2；H10 ↔ L1-7、L6-1；H12 ↔ L6-5；H13 ↔ K2-3 补一处：`research/scripts/crash-amplification.sh:42`：「#   出处是第 32 批 Q 探针真跑出来的段形状 PROBE_SEGMENT_SIZES [2,」；H14 ↔ L6-1、L6-2、L6-3、辩方 T3 | 无 |
| M5-23 | H1 对辩方 K6-3 | 站得住，小缺 | H1 退役 D13 层表三层的名字（B3:31），`.claude/kb/decisions/13-验证路线.md:245`：「| 层 1 常规 |」层 1、层 2 没有新名；H12 又说层表只留「范围」一列 | H1 或 H12 补层 1、层 2 的新名或明说层 1、2 不改名 |

## 2 M6：H1–H14 对第一节「用户已定」

判定四档：照办 / 绕开 / 违背 / 照漏；有第二种读法的另写「二读」。用户原话优先引调度记录 `records/2026-09-30-里程碑三开工第一轮.md`（简称 R）。没有一条判「违背」。

| 编号 | 用户原话（出处） | 对应 H | 判定 | 证据 | 什么现象会推翻 |
|---|---|---|---|---|---|
| M6-1 | 要合，快档与整域都合（R:931） | H1、H2 | 照办 | `records/2026-09-30-里程碑三开工第一轮.md:931`：「两个的快档和慢档都要合并到一起。」；H2 一张表一个入口一道门禁 | 合并后仍留两个入口 |
| M6-2 | CPU 对拍不抽样、日常剪枝、逐项可跑（R:932）与两档里「轻型跑没有剪枝和部分跑」（R:955） | H8、H9 | 照办，但两句并排没写谁取代谁：二读 | `records/2026-09-30-里程碑三开工第一轮.md:932`：「CPU 对拍 肯定也是日常剪枝跑」；`records/2026-09-30-里程碑三开工第一轮.md:955`：「轻型跑没有剪枝和部分跑，轻型就是跑所有」。读法一（H9 取的）：后一句取代前一句，剪枝只在重型，逐项只在重型的「逐条」。读法二：前一句管整域上的 CPU 对拍，后一句管轻型，两句并存，那就要有「日常跑一遍重型剪枝」这一档，H9 里没有（重型只在提交时看，B3:39 写「是提交时看的总门闸」）。 B3 第一节把两句并排抄，没写取代 | 用户说「日常」就是轻型（读法一对）；或说整域要日常剪枝跑（H9 要加一档） |
| M6-3 | 逐状态可写挂载「必须挂前缀，挂标准前缀，不得例外」（R:944 转述） | H4、H5 | 照办 | `records/2026-09-30-里程碑三开工第一轮.md:944`：「分界按判法写死，不按项登记例外」；H4 观察者挂在登记了名字的标准前缀上，共享节点对每一项每个状态都跑 | 单跑一项时经过的共享节点上有观察者没跑 |
| M6-4 | 「测试能走gpu的必须走gpu 除非明确的豁免」（R:946） | H5 | 照办，两处二读 | `records/2026-09-30-里程碑三开工第一轮.md:946`：「测试能走gpu的必须走gpu 除非明确的豁免」；H5 把「明确的豁免」做成豁免表（B3:35：「门禁核两边逐项对上」）。二读一：「明确」= 用户明确授权每一行；H5 读成「登记过」。H6 对「对照」就写了用户授权（B3:36），H5 对豁免行没写，两处口径不同。二读二：豁免行里有一行按用户 2026-10-10 自己的分界属判镜像一类：R:944 主 agent 答、用户未驳的是「判一张镜像的（恢复、checker、oracle、记录核对器）全上 GPU」，B3:35 的「随机历史的理想模型比较与记录核对器」把记录核对器也列进豁免，这是按项登记的例外，理由要写成判法上的（它收的是多镜像加持久集合的入参，见 `.claude/kb/decisions/13-验证路线.md:204` 的「崩溃后镜像」是两份），否则违 R:944 的「按判法写死」。还有 B3:35：「甲二在第三批之前登记一行临时豁免」，这是一条定期欠账，用户要知情 | 用户说豁免表由主 agent 登记即可（二读一不成立）；或豁免行的理由栏每行都是「入参形状 / 被测对象就是 Rust 实现」 |
| M6-5 | 「这一项不得欠账 必须做完」（C626，R:946） | H5、H14 | 照漏（批次与形状） | `.claude/kb/checks-owed.md:486`：「一格门禁：代码里走 GPU 的计算类与登记逐个对上，多一类或少一类判红」，登记的是走 GPU 的类（`gpu-cards.env.example:31`：「GPU_COMPUTATION_CLASSES='gpu_unit_checks crash_verify_gpu crash_judge_gpu'」）；H5 的表登记的是不走 GPU 的截（补集）。H5 没写两张表的关系（替换、并存、互核），H14 没有哪一批写「C626 在此批关」；门禁的代码一侧要等第二批的引擎才有，第一批（不碰 crates）做不全 | H14 点名关 C626 的批次与两张表的关系 |
| M6-6 | 多机配置合成一份、默认开 GPU（B3:10）；「1 用户电脑没有配置gpu就cpu跑 2 用户电脑配置了分片就分片跑 3 用户电脑有GPU 那么就用GPU跑」（B3:13） | H5、H11 | 二读，H5 与 H11 字面互相打架 | B3:35：「没配 GPU 走 CPU 判器」；B3:41：「没写 `ENABLE_GPU` 算开」。没写 = 算开，则没有卡也没写开关的机器按 H5 末句「配了 GPU 而卡用不上退 78」，与用户 (1)「没有配置gpu就cpu跑」顶。调和读法有三：A，「配置」= `ENABLE_GPU` 一项，没写算开，无卡机器退 78（违 (1) 的字面）；B，「配置」= 显卡配置（`gpu-cards.env`），开关缺省开，但显卡配置里没写卡就走 CPU，写了卡而用不上才退 78（今天开了 GPU 而显卡配置一份都没写是红：`research/scripts/crash-amplification-lib.sh:319`：「✗ 配置开了 GPU（ENABLE_GPU=1），显卡配置里 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写」，B 要把这一红改成走 CPU）；C，开关缺省仍算关（今天 `research/scripts/layer0-shard-configuration-check.sh:22`：「ENABLE_GPU              GPU：写 1 算开，没写或写 0 算关」），违「默认开 GPU」。分片与 GPU 的关系 H5、H11 读成正交（2×2），有现成支撑：`research/scripts/crash-amplification-two-hosts.sh:4`：「用户 2026-09-28 定四种跑法只由配置的 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两个开关定」，用户 (2)(3) 的三条不必读成三选一优先级。另：(1)(2)(3) 这段原话在调度记录里找不到（`grep -n 用户电脑` 在 R 里 0 命中），R:951 只有转述：「并定判镜像照用户配置走：没配 GPU 就 CPU、配了分片就分片、有 GPU 就 GPU」，原话出处只在正文 | 用户一句话定 A / B / C；或在记录里补上原话 |
| M6-7 | 「崩溃放量 1 内存中跑 存入db 2 硬盘重跑存入db 3 wgsl对拍」（R:951） | H8、H9 | 照漏（第 2 步） | `records/2026-09-30-里程碑三开工第一轮.md:951`：「崩溃放量 1 内存中跑 存入db 2 硬盘重跑存入db 3 wgsl对拍」。正文第一节抄的是后来两句（B3:12、13），没抄这一句；H1–H14 没一处讲「硬盘重跑」。读法：2 = 从盘上的库读回已判块再存（就是 H8 的复用），或另有一次用盘上镜像重跑的步骤；后来 R:953：「每一次 数据都是 内存中模拟跑出来的」 像是取代了后者。库是 RocksDB 的事 H8 已照办：`crates/singlefs-checker-tier/src/verdict_store.rs:2`：「库是 RocksDB（用户 2026-09-28 选定）」 | 主 agent 问用户「硬盘重跑」指什么 |
| M6-8 | 「提交前要必须判定，根据情况选择全跑或者部分跑」（R:951） | H9 | 照办；缺：谁选、凭什么选 | `records/2026-09-30-里程碑三开工第一轮.md:951`：「提交前要必须判定，根据情况选择全跑或者部分跑。」；H9 给三种跑法（剪枝、逐条、全量）没写由谁在提交时选、选的判据；B3:39：「都判完才算判定」在「逐条」只判几项时口径是「点名的判完」还是「整域都判完」没写 | H9 补选择人与判据 |
| M6-9 | 「对照可选跳过，但是要用户授权跳过」（R:951） | H6 | 照办（读法对） | 同句。「对照」= GPU 与 CPU 对拍：R:951 同条先说挂载，再说对照，R:954 说「wgsl对拍」；H6 对照默认跑、跳过要用户当场授权并记一行，读对。后果见 M5-18 | 用户说对照指别的东西 |
| M6-10 | 两档（R:955） | H9、H10 | 照办，但「轻型是重型的子集」有一个口子 | `records/2026-09-30-里程碑三开工第一轮.md:955`：「轻型是交给写代码的实现器和检测器去跑的」、「轻型跑没有剪枝和部分跑，轻型就是跑所有」；H9 逐字照办。口子：轻型的域含甲二（`crates/singlefs-checker-tier/src/crash.rs:1839`：「/// m = 0 时 c > 0 是 2^(k+1) − 1、c = 0 是 2^k − 1（与全量相同）。」，COW 段至少有 1 态），重型按 H10 对整域展不开的段「不展开、计入跳过」。第三条流有 322 写的单元写段（`crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:406`：「第三条流的单元写段 322 写，debug 与 release（工作区开着溢出检查）都会 panic」），轻型甲二在这一段有态而重型登记跳过时，轻型里有的重型里没有，违「轻中有的重一定要有」。另 B3:39 同时写「轻型是重型的子集」与「剪枝自证（关掉复用在小输入上逐项相同）在轻型里」，自证只在轻型里就也破子集；随机历史两项「小种子数」要写成前 N 个种子、其余参数同，否则抽法变了子集不成立（未核） | 第三条流今天的段形状里这一段甲二态数为 0（本腿没跑，需要 checker 档，见第 4 节命令 2）；或 H10 写明登记段在轻型同样不展开 |
| M6-11 | 同上「轻型就是跑所有」 | H2 | 小缺 | B3:32：「`--item <项>`」；`--quick` 与 `--item` 可否同用没写，同用就是轻型的部分跑 | H2 写「`--item` 只对整域」 |
| M6-12 | 轻型交给实现器和检测器（R:955）、T4（R:937） | H12 | 照漏（一份条款） | `.claude/agent-common.md:52`：「重型测试就是 checker 档那一批」那一条，后文「子 agent 一律不跑，只跑 harness 档」，与轻型含 checker 档崩溃重放、由子 agent 跑顶。B3:42 H12 条款清单：D13 已定项 9、15、`verification.md`、`implementation-workflow.md`、`.claude/main-agent.md`、`commit-stages.md`，没有 `agent-common.md`。R:937：「T4：重型闸」那条管闸、实现员 / 调查员定义、`main-agent.md`、`implementation-workflow.md`，也没点名 `agent-common.md`；改共用约束与 agent 定义要用户逐份豁免（R:951 ③ 的先例） | H12 或 T4 点名 `agent-common.md` 第 52 行 |
| M6-13 | 「这些测试的问题 必须要本轮测试前解决 不能往后拖」（R:939） | H12、H14 | 照办 | `records/2026-09-30-里程碑三开工第一轮.md:939`：「这些测试的问题 必须要本轮测试前解决 不能往后拖」；H14 四批加「之后整域一趟、E171 冻结」都在 E171 冻结之前；甲二临时豁免第三批删，不越出本轮。H12 的新条款四批后才生效与 R:932 同向 | 豁免行第三批没删（欠账越过冻结） |
| M6-14 | B、C 挪进公共模块共享（弹窗） | H3 | 照办 | 前提核过，见 M5-2 | 见 M5-2 |
| M6-15 | GPU 与 Rust 两份怎么一致，挪到合谋再谈（R:953） | 无 | 照办（H 没碰） | `records/2026-09-30-里程碑三开工第一轮.md:953`：「你如果说合谋就等会再说。」 | H 里出现 WGSL 与 Rust 一致性的条款 |

## 3 要主 agent 判 / 问用户的几件（按挡哪一批）

1. M6-6（挡第一批的 `multi-host.env.example` 默认值与 H11 的判法）：用户一句话定 A / B / C。我倾向 B 不是结论，只是它是唯一同时不违「默认开 GPU」与「没有配置gpu就cpu跑」两句字面的读法；没量过，按读码推的。
2. M6-4（挡第一批的豁免表）：豁免行是否每行要用户知情；「随机历史的记录核对器」一行的理由写成判法上的还是撤掉。
3. M6-7（挡第一批的入口语义）：问用户「硬盘重跑」指什么。
4. M6-10（挡第二批的段登记）：H10 写明登记段在轻型是否同样不展开。
5. M6-12（挡第一批的条款文本）：H12 清单或 T4 点名 `.claude/agent-common.md` 第 52 行。
6. M5-14、M5-19（挡第二、三、四批）：版本编译时算的做法，与两件判表事实核，要在实现前定。

## 4 要主 agent 跑的命令

1. 重核本报告里所有「本腿现跑」的读码命令（轻，工作目录仓根，可原样跑）：
   - `ls crates/singlefs-checker-tier/build.rs; grep -c build crates/singlefs-checker-tier/Cargo.toml`（预期：不存在、0）
   - `grep -c 等价 crates/singlefs-checker-tier/src/crash_amplification.rs crates/singlefs-checker-tier/src/crash_identity.rs crates/singlefs-checker-tier/src/verdict_store.rs`（预期三个 0）
   - `grep -c 事实核 research/prompts/_m3-crash-enumeration-merge-r3-body.md`（预期 0）
   - `for f in crates/singlefs-checker-tier/tests/*.rs; do head -1 "$f"; done | grep -cE 'layer0_progress|crash_amplification'`（预期 19）
   - `grep -n 用户电脑 records/2026-09-30-里程碑三开工第一轮.md`（预期无输出）
2. 核 M6-10 的口子（checker 档，我不能跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request`，内存与线程照派发提示）：读第三条按位置寻址流的段形状与每段原地写 / 单元写个数，算 322 写那一段按甲二的态数（公式在 `crates/singlefs-checker-tier/src/crash.rs:1837`–`1839` 的文档）：
   `SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test --release -p singlefs-checker-tier --test crash_enumeration_position_addressed_trees -- --nocapture 2>&1 | tee <日志>`，看日志里 `LAYER0_POSITION_ADDRESSED` 行的 `segments=`；该测试里被 `#[ignore]` 的全展开那条不要加 `--include-ignored`。判法：322 写那一段的原地写个数 k 若不是 0 以外的任何数，甲二态数 ≥ 1，口子成立。
3. 核 M5-14：不需要命令，是实现前事实核（`#[path]` 加 `[build-dependencies]` 编不编得过）。

## 5 没做什么

- 没跑 checker 档、GPU、门禁阶段、崩溃放量，没编译任何 Rust，没跑模型；M6-10 的口子只读码推出，态数没量。
- 没重核判决第一节第 ① 格的两条独立路、第 ⑦ 格；没复跑第二轮攻方模型；没核 `crash-case-check.py` placement 逻辑的全部分支（只核了函数名正则那一行）。
- 没核 M5-17 里「卡型换了判表装得下与否会变」，那是推的。
- 没替主 agent 撤销或维持判决，上面的「站得住 / 照漏 / 二读」只是复核结果。
- 收工 `sha256sum -c --quiet` 两份快照清单：crates 91 份退 0，kb 4 份退 0（与开工一致）。
- 仓内无副本、无编译目录；草稿目录 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r3-sonnet/` 是空的。

更正（第 4 节命令 2 的判法一句写乱了）：判法是读出 322 写那一段的原地写个数 k 与单元写个数 c，按甲二公式算态数 3^m·2^(k−m)·(c>0 时 2，否则 1)−1；只要这个数 ≥ 1（c > 0 时 k = 0 也有 1 态），轻型甲二域在这一段有态而重型登记跳过，口子成立；态数为 0 才推翻。
