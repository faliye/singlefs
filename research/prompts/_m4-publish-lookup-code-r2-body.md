# m4-publish-lookup-code-r2：里程碑四七·6 发布路径按身份查，代码轮第二轮正文（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

这一轮攻七·6 第二批补丁，它照第一轮判决 `research/prompts/m4-publish-lookup-code-r1-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」三条改法写成，已合进主工作区。第二轮的攻击面照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮的分工：第一轮的修补本身、产生结论的方法、从未被看过的地方；不重复第一轮已攻过的 T1（索引与线性查找是不是同一项）、T2（整理读版本一遍走叶与逐 key 查）、T5（有没有改到「行为不变」之外）的角度，除非第二批补丁改动了它们的前提。

改动全文：`research/prompts/_m4-publish-lookup-code-r2-diff.md`（`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv` 原样）。实现员报告：`~/.local/share/singlefs/work/m4-publish-lookup-r2-writer/report.md`（sha256 `7d01435d02456e9564385427356551d945e29d4bb9f3db9da320a4cfb197ffb1`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-21）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo/`（合入七·6 第二批、别的会话批 11 两份只改测试的补丁、以及另一个会话的六份补丁（GPU 判器内核与派活、`walk.rs`、几份 checker 档与 harness 测试，不碰这一轮被判的三个文件）之后的主工作区，去掉了私有配置 `multi-host.env`）。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-code-r2-snapshot/sha256sums.txt`。
- 产品代码改动只在 `crates/singlefs-core/src/transaction.rs` 与 `crates/singlefs-core/src/mounted_session.rs`。主工作区里的落点：
  - `mounted_session.rs` 第 2795 行新的公开函数 `central_mapping_as_the_compaction_reads_it`（整理读一版时取映射条目那一半，原样抽自私有的 `version_as_the_compaction_reads_it`），第 2824 行私有函数改成调它；第 2782 行起的 `CentralMappingAsTheCompactionReadsIt` 带 `mapping_entries` 与 `mapping_leaves_decoded` 两个公开字段。
  - `transaction.rs` 第 4412 行 `leaves_decoded`（读批量读者已有的 `leaves_read.len()`）；第 5729 行快照判定那一圈改走 `mapped_units_by_role().mapping_key_of`，原来的私有函数 `mapping_key_carried_from` 删了；第 3305 行 `mapping_key_of` 的文档注释改了 panic 契约那一句。
- 测试 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs`（+252 −49）：等价用例拿整理实际取到的映射条目逐项比逐 key 走 `central_mapping_locations_in_the_version` 的参照；增长慢档用例改成数次数（解叶数、身份比较次数），改了名；另加一条快档合成表用例。变异 `crates/mutations.tsv` 末尾追加 5 行（「七·6 整理读版本」三行、「七·6 按身份查：索引从大到小逐项找」「七·6 按身份查：快照判定那一圈的映射 key 一律取 inode 根那一把」），「七·6 增长」两行整行换了测试名。
- 合入后验证（量的）：测试在第二台「✓ 测试全绿（264 个测试二进制，共 2533 条测试；跑了 2 个包…）」；fmt、clippy（带编码纪律七条）、`build --all-targets` 本机补跑，三样退 0。崩溃轻型在主工作区：`CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=35 collected=35 green=34 red=0 not_run=1 …`（没跑的是钉表）。集成副本上的变异轻型点名 37 行全部抓到，核心部分三处红经对照早于这一批（不在主工作区上的这一批补丁里）。主工作区上的形式化轻型三格全绿；主工作区上的变异轻型（第二台，核心部分加点名 28 项）「计数：抓到 264 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 1 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条、锚点失效 0 行（共 266 条）」，另基线不绿 1 行；红的 602 行（基线不绿）与 275 行（超时）早于这一批，不是七·6 的行，七·6 第二批的 14 行全部抓到。这两样都在另一个会话的六份补丁打进来之前跑的。
- 已知、报出来不算打中的：判决第三节点名之外 `mapping_locations_of_a_mapped_unit` 还在线性查找（记延后）；「整发布不再整版照抄」那一层在 `.claude/kb/milestone/04-fourth-txn.md`「优化项」表；实现员报告里写的「T4 ① 判得出只靠整理经这一个公开函数取映射，没有检查守着」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　公开出去的那一半是不是整理真正用的那一半
`central_mapping_as_the_compaction_reads_it` 从私有函数里抽出，用例拿它比参照。核：整理选段与搬单元读版本的每一条路是不是都经它取映射条目，有没有别的路径另取一份（另一条整理入口、恢复后第一次整理、快照下的整理）；抽出前后私有函数产出的 `VersionAsTheCompactionReadsIt` 是不是逐项相同（次序、留存条目接在后面那一段）；变成公开之后有没有别处能拿它绕开整理自己的不变量。

### T2　增长判据数的是不是增长本身
慢档用例改成数解叶数与身份比较次数。核：数的那两样在改坏查法时一定涨，在改成别的同样慢的写法（例如每次查都重建索引、索引按插入序线性找）时也涨不涨；身份比较次数用测试里包装身份去数，产品代码里有没有不经包装的比较（派生的 `Ord`、`HashMap` 的哈希）漏数；门槛（四倍表长下每次查平均比较次数增长低于 200%）在登记的命令与取样规模上会不会被别的因素推过线或压下线。

### T3　快照判定那一圈换了查法之后的行为
`deadlist_changes_of_this_publish` 那一圈改走 `mapped_units_by_role().mapping_key_of`。核：每轮现建索引还是复用，建的那一刻之后上一版的 `mapped_units` 会不会被改；同一角色在 `mapped_units` 里出现不止一次时，旧的 `iter().find` 与新索引取到的是不是同一项；查不到时 panic 的条件与文案是不是逐字相同；实现员报告说「索引键改错」那条变异红在产品自己的断言上、没走到测试自己的 deadlist 比较，这说明了什么（测试判别力是不是只靠产品断言）。

### T4　新加的五行变异与换名的两行
五行追加变异各自让点名的测试红的理由；「每把 key 现建一个批量读者」那一行改坏的是性能还是结果，点名用例凭什么红；换名之后两行增长变异仍点名到改名后的慢档用例；有没有一处第二批的改法（`leaves_decoded`、`central_mapping_as_the_compaction_reads_it` 的字段、`mapping_key_of` 那一圈）改回去而没有任何一行变异红。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 | 核「代码做的是不是第一轮判决第三节要的那件事」：逐格对补丁原文与冻结副本的行，每条结论写文件:行号；T1、T3 要逐处列出整理与快照判定每条路径上取映射条目、取映射 key 的地方 |
| 云端攻方（Opus） | T1–T4 | 在冻结副本的拷贝上造反例：一串合法操作让整理不经公开函数取映射、或改前改后写出不同的盘镜像、或一处查到不同的项、或改后 panic 而改前不 panic；T2、T4 造一个改坏查法或改坏判据而测试不红的变异 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；线程上限 8；用自己的编译目录。本机排不上时照包装打出的那一行改去第二台（`scripts/hosts/multi-host-run.sh --peer`）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
