# 回扫报告：JOURNAL_HEADER_BYTES 307 → 311（含「填充1」→「记录标志1」）

时刻 2026-09-24 11:32 UTC（东京 20:32）。**本机同时有别的会话在写 kb**：开工时 `.claude/kb/decisions/23` 是唯一在改的 kb 文件，扫描过程中 `.claude/kb/layout/01-first-txn.md` 被另一会话**已经改完**（现查：该文件「六、journal」整节已是 311/记录标志1/3785/201，我原先记的 9 处命中已归零，只剩历史版本节 1 处不算）。下表按最后一次现查（11:32 UTC）的状态给，未提到「已修好」的文件我逐一现查过仍未改。

## 一、逐类搜索：命令与计数（全部现跑，输出未截断，仅展示计数；命中清单见二）

| 类 | 命令 | 计数 |
|---|---|---|
| 字面量 307（全仓，`\b307\b`） | `grep -rnw '307' --exclude-dir=.git --exclude-dir=target .` | 348 |
| 同上，限定「活文档」范围（`.claude/kb/{decisions,experiments,layout,milestone,invariants.md,verification-build.md,freeze-layer-membership.md,experiments.md,checks-owed.md}` + `crates/` + `research/e7-index-bench/src/bin/` + `research/mutations/`，且排除 `*-history.md` 与 `decisions-history/`） | 85 |
| 「307 字节头」「头 307 字节」 | 含在上一行的字面量搜索里（无需单独命令，见二.1） | — |
| `4096 − 307` 及其结果 3789 | `grep -rnw '3789' --exclude-dir=.git --exclude-dir=target .` | 27（含历史/冻结） |
| `512 − 307` 及其结果 205 | `grep -rnw '205' --exclude-dir=.git --exclude-dir=target .` | 184（绝大多数是不相干的巧合数，见二.5） |
| 60%（307÷512） | `grep -rn '60%' --exclude-dir=.git --exclude-dir=target .` | 33（绝大多数不相干） |
| 「填充 1」（十字段第 4 个，改名「记录标志 1」） | `grep -rn '填充 1' --exclude-dir=.git --exclude-dir=target .` | 19 |
| 「对齐填充」（同一字节的另一种写法） | `grep -rn '对齐填充' --exclude-dir=.git --exclude-dir=target .` | 4 |
| 「六笔已定增量」（现在七笔） | `grep -rn '六笔已定增量' --exclude-dir=.git --exclude-dir=target .` | 2 |
| 旧偏移 91/95/283/291 当「载荷校验和/新根段/fsid/MAC」讲 | `grep -rn '偏移 283\|偏移 291\|(283\|(291' .claude/kb crates research/e7-index-bench/src/bin research/mutations` | 0（`crates/` 已全部改到新偏移，kb 里也没有残留） |
| 测试/门禁钉的产物值 628216162 | `grep -rl '628216162' --exclude-dir=.git --exclude-dir=target .` | 21 个文件（全部落在门禁 55 号 fixtures、E142 kb 页、milestone、`first_transaction_step_five_publish.rs` 的说明性注释与 `research/results/`，见二.6） |
| 第二遍：按量名/常量名找「不含 307 却在算同一个量」的更早一代残留（78/84/86/95/303） | `grep -rn '现行头\|现行.*记录头\|记录头.*现行\|头宽.*现行' --exclude-dir=.git --exclude-dir=target .`（另加 `JOURNAL_HEADER_BYTES` 全仓扫） | 18 处，见二.7 |

## 二、命中清单

### 1. 要改（kb 正文，现状句，非实验产物）

| 文件:行 | 原文（整行/关键片段） | 该改成什么 |
|---|---|---|
| `.claude/kb/freeze-layer-membership.md:45` | `journal 记录格式（码 17 自证单元，记录 4096、头 307、点名项、载荷校验和的覆盖范围、事务边界）` | 「头 307」→「头 311」 |
| `.claude/kb/verification-build.md:47` | `自证单元（根记录 457 字节、journal 记录头 307 字节，D22 已定项 7 与 D23 已定项 4）的实现一份` | 「307 字节」→「311 字节」 |
| `.claude/kb/verification-build.md:65` | `写记录｜4 KiB 定长记录、头 307 字节、每项 56 字节（已定项 4/12）；一个事务可跨多条…` | 「头 307 字节」→「头 311 字节」 |
| `.claude/kb/invariants.md:83` | `I-8.6…反向链 = CRC32C(本实例内逻辑前一条记录的 307 字节头…)；…（…链按实例算与头宽 307 是 2026-09-14 用户定案）` | 前半句「307 字节头」→「311 字节头」直接改；后半句「头宽 307 是 2026-09-14 用户定案」现在不是真话——头宽后来又从 307 改到 311（2026-09-24），这一句要写成「链按实例算是 2026-09-14 用户定案，头宽随 D23 已定项 4 现行 311」，不能原样把 307 换成 311（那样会把 2026-09-24 的事说成 2026-09-14 定的） |
| `.claude/kb/decisions/18-块里携带什么信息.md:262` | `\| 17 \| journal 记录 \| 记录 \| D23 已定项 1 的 A 条 + 已定项 4 的 307 字节头（含已定项 15 的新根段 188、fsid 8 与 MAC 16）\|...` | 「307 字节头」→「311 字节头」 |
| `.claude/kb/decisions/23-journal的角色与格式.md:14` | 索引表：`\| 4 \|...\| 能；头 307 字节 = 十个字段 78 + 三笔增量 17 + 新根段 188 + fsid 8 + MAC 16 **状态：已定。**\|` | 「头 307 字节 = 十个字段 78 + 三笔增量 17…」→「头 311 字节 = 十个字段 78 + 七笔增量 233（事务边界 9 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16）」（同一份文件的 139 行「已定项 4」正文已经是 311，这一行索引表没跟上） |
| `.claude/kb/decisions/23-journal的角色与格式.md:305` | `- 与已定项 4 相容：记录头 307 字节要完整落在一个 physical_block_size 内…` | 「307 字节」→「311 字节」 |
| `.claude/kb/decisions/23-journal的角色与格式.md:339` | `- 用户弹窗定案取整条（主 agent 推荐只罩头 [0, 307)）…` | 「[0, 307)」→「[0, 311)」 |
| `.claude/kb/milestone/01-first-txn.md:169` | `journal 记录 4 KiB。头 = 78 加六笔已定增量（事务号 8 + 提交标记 1、反向链 4、载荷校验和 4、新根段 188…、fsid 8、MAC 16）= 307…` | 「六笔已定增量」→「七笔已定增量」，插入「本次发布内序号 4」，「= 307」→「= 311」 |
| `.claude/kb/milestone/01-first-txn.md:178` | `一条 4096 记录最多装 ⌊(4096 − 307) / 56⌋ = 67 项）；t9 的反向链 = w4 记录头 307 字节的 CRC32C…` | 「4096 − 307」→「4096 − 311」（商仍是 67）；「w4 记录头 307 字节」→「311 字节」 |
| `.claude/kb/milestone/01-first-txn.md:250` | `journal 记录头 307（新根段 188 + fsid 8 + MAC 16）` | 「307」→「311（新根段 188 + fsid 8 + MAC 16 + 本次发布内序号 4）」 |
| `.claude/kb/milestone/02-second-txn.md:103` | `反向链 = A 那条记录头 307 字节的 CRC32C（本实例内逻辑前一条，已定项 19 ②）` | 「307 字节」→「311 字节」 |
| `crates/singlefs-core/src/journal.rs:21` | `/// magic 4 + 类型 2 + 算法类型 1 + 填充 1 + 记录长度 4 + …` | 「填充 1」→「记录标志 1」（数值 78/311/写偏移全部已经是新值，唯独这一行注释的字段名没跟 2026-09-24 的改名） |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs:3200` | `/// …magic 4 + 类型 2 + 算法 1 + 填充 1 + 记录长 4 之后是点名项数；…` | 同上，「填充 1」→「记录标志 1」（该文件里两处偏移常量 `JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET=95`、`JOURNAL_RECORD_HEADER_BYTES=311` 已经是新值，只有这一行注释没跟） |

### 要人看（.claude/kb/verification-build.md:221，公式要重排不是单纯换数字）

原文：`已收口：记录头 307 字节 = 十个基础字段 78 + 事务号 8 + 提交标记 1 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16…`——把 307 直接换成 311，等号右边加总仍是 307，算式变成假话；要在「提交标记 1」之后插入「本次发布内序号 4」这一项，左边才能配平到 311。同一节表格上一格（第 7 问）本身在narrat「2026-09-12 时」的历史困惑，是否要整节挪进历史版本或原地改成 311 公式，交人判。

### 要人看（`.claude/kb/milestone/01-first-txn.md:159`，「现状（日期）」句，back_chain 数值待 E142 重跑核实）

原文：`…jsn 3 点名 8 项、反向链与产物 back_chain=628216162 逐字相同——两套装置（实验与实现）对同一个量落到同一个数…`。头宽改到 311 后，`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` 已经把这条断言的期望值改成 `1057457588`（该文件 63–65、515 行的说明性注释已正确写明 628216162 是「307 字节头下的值」）；这句 milestone 现状描述如果照抄新数，应写「back_chain=1057457588 逐字相同」，但 **E142（第一个事务的干跑）本身还没有按 311 重新跑一遍**（这是它引用的实验），产物第 37 行仍是 307 字节头下的 628216162。是否现在就把 milestone 这句改成 1057457588（实现侧已经这样断言），还是等 E142 真正重跑出新产物再改，交人判。

### 2. research 实验装置常量与产物，需实验执行员同步改常量、重跑、更新 kb 页与 `research/results/`（本轮只列文件:行，不改，不重跑）

以下每组文件对同一个量（journal 记录头及其派生的余量/商/百分比）表述一致：kb 实验页仍引「现行头 307」佐证结论，对应的 `research/e7-index-bench/src/bin/*.rs` 装置里 `const JOURNAL_HEADER_BYTES: u64 = 307`（或改名前的 `RECORD_HEADER_BYTES`）没有跟着改，两边都要与实验源码的 const、单测、`research/results/` 产物三处一起动（`format-evolution.md`「改一个格式常量」）。

| 量 | kb 实验页命中（文件:行） | 对应 research 装置源码命中（文件:行） | mutations.tsv 锚点 |
|---|---|---|---|
| E23（journal 几何）现行头/余量/商 | `.claude/kb/experiments/23-journal几何.md:65,66`（「现行头是 307 字节、512 上余 205…4096 上余 3789」「引用头部字节数时用 307」） | — | — |
| E39（反向链挡不挡得住残留记录）四档头宽 | `.claude/kb/experiments/39-反向链挡不挡得住残留记录.md:59`（「现行头不含反向链是 303…四档是 304/305/307/311 字节…」）——**要人看**：303 本身是「307−反向链4」推出来的，头宽再变一次之后四档要重新枚举，不是简单替换一个数 | — | — |
| E42（一事务几条记录） | `.claude/kb/experiments/42-一事务几条记录.md:56`（「现行 307（十个基础字段 78 加后定的增量）」） | — | — |
| E43（扩展点字节上限）自证单元余量/上界 | `.claude/kb/experiments/43-扩展点字节上限.md:88,188,193,194,200,220,252`（均在「## 历史版本」之前的主体段落，非历史；254 行起的历史版本节里 256、258、264 三处属历史叙述，不改） | `research/e7-index-bench/src/bin/e43_extension_point_budget.rs:173,177,178,519,522,523,535,536,537,541,542`（`const JOURNAL_HEADER_BYTES=307`，注释「头是 307 字节…余 205…4096 上余 3789」，`assert_eq!(self_witness_room(…,512),205)`、`assert_eq!(…,4096),3789)`、`min(512−307,256−1)`、「紧 4.2 倍（864÷205）」） | 未见 e43 专属 mutations 文件命中（检查过 `research/mutations/`，没有 e43 那张表） |
| E49（反向链宽度 32 还是 64） | `.claude/kb/experiments/49-反向链宽度32还是64.md:1,7,80,145`（标题与三处正文，均「现行不含反向链的记录头是 303…要按 303 重算」） | — | — |
| E49 对应的欠账 C200 | `.claude/kb/checks-owed.md:186`——**要人看**：C200 的还账计划写死「按 base = 78 与 base = 91 各重跑一次」，这是更早一代（86→78 那一轮）的计划，头宽后来又经 78→95→277→307→311 好几轮，这两个 base 已经不对应任何现行口径，需要人重新定这笔账该按哪个 base 还 | — | — |
| E61（反向链 hash 算法均匀性） | `.claude/kb/experiments/61-反向链hash算法的均匀性.md:95`（「现行头是 307（十个字段 78 + 三笔已定增量 17 + 新根段 188 + fsid 8 + MAC 16）」——公式也要加「本次发布内序号 4」） | `research/e7-index-bench/src/bin/e61_chain_hash.rs:128`（`header_bytes[7]=0; // 对齐填充`，三档头宽模型停在 86/95/99，本身已知停旧档） | — |
| E75（记录尺寸与环几何） | `.claude/kb/experiments/75-记录尺寸与环几何.md:34`（「现行头 307 另含新根段 188、fsid 8 与 MAC 16」，「三笔已定增量」也要变四笔） | — | — |
| E116（打包容器的账） | `.claude/kb/experiments/116-打包容器的账·补元数据写与整理策略.md:41,81,89,99,118,119,125`（回本比、稳态占用等具体数值都是按 307 跑出来的产物引用，需重跑才能给新数） | `research/e7-index-bench/src/bin/e116_pack_settle.rs:84,156,186,337`（`const JOURNAL_HEADER_BYTES: u64 = 307`） | `research/mutations/e116_pack_settle.tsv:16`（锚点 `const JOURNAL_HEADER_BYTES: u64 = 307;`） |
| E142（第一个事务的干跑）判决产物 | `.claude/kb/experiments/142-第一个事务的干跑.md:114`（「判决」节，`E7RESULT name=width structure=journal_header expected=307 actual=307`，整节声明「整行抄自产物」，产物本身要重新出）；`:168`（G6「previous_hash=CRC32C(…307 字节头…)」，主体段落非历史） | `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`（11 处：常量定义、断言、注释，implementer 报告第十节已逐条列出） | `research/mutations/e142_first_transaction_dry_run.tsv:23`（`M23_journal_header_95` 锚点 `const JOURNAL_HEADER_BYTES: u64 = 307;`） |
| E157（并行线一两条条款的计数模型） | `.claude/kb/experiments/157-并行线一两条条款的计数模型.md:12`——**这一处是历史事件句，不改**：它叙述的是「2026-09-22 那次跑」当时装置常量与 crates 逐项对拍的结果，那时源码确实是 307，不是现状声明 | `research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:70,74,310,506,507`（`const JOURNAL_HEADER_BYTES: u64 = 307`） | 未见专属 mutations 文件 |
| E155（四份变体） | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:120`——**历史事件句，不改**：叙述「第四次跑第一段（2026-09-21）」当时的 sha256 停机自检，那时源码确是 307 | `research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs:44`、`e155_fsync_write_volume.rs:26`、`e155_second_run_fsync_write_volume.rs:43`、`e155_third_run_release_cascade.rs:35`（均 `const RECORD_HEADER_BYTES: u64 = 307`） | — |
| E159（fsync 等待组提交） | 未见对应 kb 实验页正文引用 307 | `research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs:47`（`const RECORD_HEADER_BYTES: u64 = 307`） | — |
| E91（环准入）——**分不清，要人看** | 未见 kb 实验页命中 | `research/e7-index-bench/src/bin/e91_ring_admission.rs:17`：`const RECORD_HEADER_BYTES: u64 = 78; // D23 已定项 4（现行头宽）`——这个 78 本身就不是「记录头」的现行值（78 只是十个固定字段），是更早一代就写错的量，还是这个装置本来就只关心十个固定字段、注释写错了「现行头宽」四个字，两种可能都存在，需要人核这个装置到底在算哪个量 |
| E143（一事务一单元下的 journal 账）——**分不清，要人看**（implementer 报告已同样标注） | 未见 kb 实验页命中 | `research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs:12,18,73,143,195,196,200`：`JOURNAL_HEADER_BYTES_TODAY=95`——这是另一套头宽口径（今天实现之外假想的「每单元一条记录」方案），不确定要不要跟这次改动走 |
| `.claude/kb/experiments.md`（实验索引页）——**要人看**，E49 那一行「不同的数、同一个量」 | `.claude/kb/experiments.md:70`：`E49…现行记录头 78，该表里 base = 78 那一档…`——78 是十个固定字段的宽度，不是整个记录头的现行值（不论 307 还是 311 都不对），这条索引摘要本身写错了，需要人定它原意是想说哪个数再改；同页 43 行「现行 10 个字段 78」这一句本身仍然正确（十字段总宽不受这次改动影响），不用动 |

### 3. 历史版本节 / `*-history.md` / `records/` / `research/prompts/` / `research/results/` / 门禁 fixtures，冻结证据，不改

| 文件:行（或范围） | 为什么不改 |
|---|---|
| `.claude/kb/decisions-history.md`、`.claude/kb/decisions-history/2026-09.md`（全部命中，含 289、28-36、6178、7739、1221、2465、3545、3703、4081、4527、4795、5073、5744、5749、5778、5945、6050、6177、6180、6185、6187、6326、6394、6397、6398、6629、7278、7664、7760、8003、8032、8250、8608、8612、8615） | 决策变更史本身：每一条都在记「那一次改成了什么」，把旧值换成新值这句话就不再是「改前」的原文了 |
| `.claude/kb/experiments-history.md`（207、209、211、212、218、253、254、367、370、373、374） | 同上，实验变更史 |
| `.claude/kb/experiments/43-扩展点字节上限.md` 的「## 历史版本」节（256 行起，含 258、264 两处 307） | 文档自身的「## 历史版本」小节，叙述 2026-09-14 那次重跑当时的做法与判红 |
| `.claude/kb/experiments/142-第一个事务的干跑.md:288` | 在「### 2026-09-14」历史子节里，叙述「277→307」这次更早的过渡，本身仍是真话 |
| `.claude/kb/experiments/23-journal几何.md:68-79`（field 表含「对齐填充 \| 1 \|」） | 表格上方原文写明「这张字段表是 jsn 占 8 字节那一版的实测算术，是 2026-08-29 那次运行真正跑出来的那一份，**原样保留**」——显式冻结 |
| `.claude/kb/layout/01-first-txn.md:435` | 唯一残留在「## 历史版本」节里，叙述「journal 记录头 277 → 307」那次过渡，本身仍是真话（该文件主体已被另一会话改完，见开头说明） |
| `research/prompts/*.md`（`m2-p6-header-implementer-report.md`、`_m2-treesplit-r1-{background,appendix}.md`、`e157/e158/e159-preregistration.md` 等约 60 个命中） | `research/prompts/` 冻结证据，只列不改；`27-format-constants.sh` 的扫描范围本身也不包含它（只扫 `.claude/kb/**/*.md` 与 `research/**/*.rs`+`crates/**/*.rs`，不含 `.md` 类 prompts） |
| `research/results/*.out`（e142、e155、e153、e160 等，含 628216162 与 3789 相关命中） | 产物是那一轮的原始输出，`27-format-constants.sh` 自己也显式排除 `research/results/`（理由同证据链不可事后改） |
| `.claude/gate.d/fixtures/55-qemu-first-transaction.sh/{red,green}/**`（含 628216162） | 这是门禁 55 号判别力自测用的预录 fixture 对；比对的是「程序侧」与「设备侧」两份历史真跑记录彼此一不一致，不依赖今天的头宽是 307 还是 311 |
| `records/2026-09-13-总审核.md:367,401`、`records/2026-09-22-panic面普查清单.md`（107,108,114,131,134）、`records/2026-09-22-panic面普查-走得到的那些.md:106`、`records/2026-09-24-里程碑二收尾调度.md:30,66,95` | 均为带日期的设计/调度记录，叙述那一天代码/评审的实际状态或那一天的排期决定，design-doc-discipline 下正常留历史 |
| `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:63,64,515` | 说明性注释，逐字写明「628216162 是 307 字节头下的值」「与 311 字节头下独立换算出的 back_chain 逐字相同」——是正确的今昔对照，不是残留 |

### 4. 不相干的 307（同一个数字，不同的量）

| 文件:行 | 说明 |
|---|---|
| `.claude/kb/experiments/137-映射key形态的性能差距.md:121`；`.claude/kb/experiments/56-消息缓冲的收益vsε.md:142,146`；`.claude/kb/experiments/58-校验和粒度的端到端代价.md:215` | 表格里的比率/计数值（0.307、1.7215 表里的 307 档位、1.300–1.307）与记录头无关 |
| `.claude/kb/experiments/99-writebuffer条目与seq的去重.md:99` | 「195 307 500 032」是带千分位空格的大数，"307" 只是其中一段数字 |
| `.claude/kb/experiments/79-根记录的容量.md:31` | 「索引节点类的现行头宽 68」，是另一种结构（索引节点），不是 journal 记录头 |
| `.claude/kb/checks-owed.md`（C307 多处，270、445、785、788 等） | C307 是决策编号（映射树两种 key 宽怎么装进一棵定宽 key 的树），不是字节数 |
| `records/2026-09-22-panic面普查清单.md:371,655,660,698`；`.claude/kb/invariants.md:33` 里的「C307」 | 同上，一处是行号（`allocator.rs:307`、`walk.rs:307`），两处是决策编号，均与本次改动无关 |
| `research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs:271,273,395,396`、`e56_epsilon.rs:1159`、`e68_inline_threshold.rs:40`、`e132_livelist_carrier_recount.rs:213`、`e145_self_describing_node_header.rs:1`、`e71_accounting_keys.rs:360,362` | implementer 报告第十节已逐条核过，均是各自实验自己的计数/扇出值，与 journal 记录头无关 |
| `.claude/kb/experiments/23-journal几何.md:73`、`.claude/kb/layout/01-first-txn.md:327`（已由并发会话改成「记录标志」，见开头） | 「对齐填充」/「记录标志」字段名问题，与「307」这个数字本身无关，已在第一类里单独列出 |

## 三、`format-const: JOURNAL_HEADER_BYTES` 可登记进 `stale=` 的候选（现扫 `.claude/gate.d/27-format-constants.sh` 实际扫描的文件集合：`.claude/kb/**/*.md` 排除 `*-history.md` 与 `decisions-history/`，每份截到「## 历史版本」之前、剥掉标记行本身；再加 `research/**/*.rs` + `crates/**/*.rs` 全文）

复现脚本：`/tmp/claude-1000/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/scratchpad/scan_scope.py`（原样复刻 27 号阶段第 3 步的 `scan` 构造），逐串命令与末行输出：

```
$ python3 scan_scope.py '头 **307 字节**' '登记值就是 307' '⌊3789 / 56⌋'
0	'头 **307 字节**'
0	'登记值就是 307'
0	'⌊3789 / 56⌋'
```

这三串**已经**登记在 `.claude/kb/decisions/23-journal的角色与格式.md:151` 的 `stale=` 里，现扫确认仍是 0 命中，登记有效、不用动。

```
$ python3 scan_scope.py '头 307 字节' '的 307 字节头' '现行头是 307' '现行头 307' 'JOURNAL_HEADER_BYTES: u64 = 307' '合计 **307 字节**' '4096 − 307' '4096 上余 3789' '512 上余 205' '占 512 扇区的 60%' '现行不含反向链的记录头是 303' '算法类型 1 + 填充 1' '// 对齐填充'
5	'头 307 字节'
5	'的 307 字节头'
1	'现行头是 307'
1	'现行头 307'
4	'JOURNAL_HEADER_BYTES: u64 = 307'
1	'合计 **307 字节**'
3	'4096 − 307'
2	'4096 上余 3789'
1	'512 上余 205'
1	'占 512 扇区的 60%'
1	'现行不含反向链的记录头是 303'
1	'算法类型 1 + 填充 1'
3	'// 对齐填充'
```

**这些串现在都命中 > 0，不给。** 它们全部命中的正是二.1、二.2 表里我列出的「要改」「research 装置」那些行——命中 0 次要等这些行改完并且 research 装置重跑之后才成立。`// 对齐填充` 即使清零也不建议登记：它是通用注释串，不排除以后某处别的对齐字节合法复用同一句注释（`format-evolution.md`「裸数字不许登记」同理的谨慎）。

## 四、没做什么

- 没改任何文件；只读、只分类。
- E43 的余量/上界、E49 的四档反向链代价、E61 的三档 hash 均匀性、E116 的回本比、E142 的判决产物这几类具体数值，我没有手算给出新值——它们是研究装置跑出来的产物，按 `evidence-discipline.md`「所有旧数据都只是参考」要重新跑才算数，不是我能在回扫里代替实验执行员算出来的；已在二.2 表里逐个列出文件:行交给实验执行员。
- `.claude/kb/checks-owed.md` 里除 C200、C307（不相干）外还有没有别的欠账条目引用旧头宽，我只做了「307」与「现行头/头宽」两类关键词的搜索，没有通读整份 `checks-owed.md`（1000+ 行）逐条核对；C200 已单独标「要人看」。
- `research/e7-index-bench/src/bin/e91_ring_admission.rs` 与 `e143_one_unit_per_transaction_journal.rs` 这两处「78」「95」到底算不算这次改动要跟的同一个量，我判不出来，列成「分不清，要人看」，没有替它们下结论。
- 没有验证 `crates/singlefs-core/src/journal.rs:150-151` 那个「记录标志」位（decisions/23 已定项 4 明文「位 0 = 本次发布末条」）在写入路径上是否已经实现——现查发现它目前是硬编码 `writer.put_u8(0); // 对齐填充`，读侧 `recovery.rs` 的 `record_ends_its_publish` 仍走内容启发式（点名了非码 1 单元即为末条），没有用这个位。这不是纯文本回扫能判定「改不改」的事（它是一项决策已定但代码未实现的欠账，`.claude/kb/checks-owed.md` 里没找到对应条目），如实报给主 agent，不在我的「要改」清单里当文本问题处理。
- 没有跑 `cargo test`/`gate.sh`，按定义不归我。
- 没有验证「填充 1」在 `research/prompts/_m2-treesplit-r1-{background,appendix}.md` 里两处命中之外还有没有别的冻结证据引用它——第一遍全仓搜到的 19 处「填充 1」已逐条列出（十九处见一.命令，去重后落点在二.1/二.3/上一条）。

## 附：本轮读到的关键权威文件（路径）

`.claude/kb/decisions/23-journal的角色与格式.md`（已定项 4，行号 139-164）、`crates/singlefs-format/src/lib.rs`（`JOURNAL_HEADER_BYTES = 311`，165 行）、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`（三者均已是 311，核对无误）、`research/prompts/m2-p6-header-implementer-report.md`（本轮派发指定的权威经过说明）。
