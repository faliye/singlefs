# m4-publish-lookup-followup-code-r1：里程碑四七·6 后续批，代码轮第一轮正文（2026-10-22）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

## 一、题面

七·6（发布路径按身份查）代码轮第三轮、最终轮判决 `research/prompts/m4-publish-lookup-code-r3-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」第 1–4 条与「五、交用户」第一行（用户定「修键算法，认并上来的特性」）各交了一个实现员，补丁都已合进主工作区。同一份判决「五、交用户」第二行用户定「另立一题开新代码轮」，这一轮就是那一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮的分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。七·6 前三轮攻过的角度（索引与线性找取同一项、整理读版本一遍走叶、快照判定那一圈、计数存在线程局部变量）不重复，除非这一批改动了它们的前提。

改动全文：`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md`（七·6 后续批的 `crates.patch` 与 `mutations-append.tsv` 原样；判法单元键那份补丁只抄 `crates/singlefs-core/` 下的两份）。实现员报告：七·6 后续批 `~/.local/share/singlefs/work/m4-publish-lookup-followup-writer/patch/report.md`（sha256 前缀 `c4a25ddb`），判法单元键 `~/.local/share/singlefs/work/m4-unit-graph-unified-features-writer/report.md`（sha256 前缀 `9db0f035`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-22）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo/`（合入这两批与第 7 批之后的主工作区，去掉了私有配置 `multi-host.env`、`.env`、`.git` 与编译目录）。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-followup-code-r1-snapshot/sha256sums.txt`。
- 产品代码改动：`crates/singlefs-core/src/transaction.rs`（七·6 后续批，+181 −41）；`crates/singlefs-core/Cargo.toml` 加 `links = "singlefs-core"`、新文件 `crates/singlefs-core/build.rs`（判法单元键那一批，把这一次编译里 cargo 给本包开的特性经 `links` 元数据交给直接依赖它的构建脚本）。测试侧：`crates/singlefs-harness/tests/publish_lookup_by_identity.rs`（+702 −10）与 `crates/mutations.tsv` 追加的 19 行（都以「七·6 第三轮」起头）。`crates/singlefs-core/src/mounted_session.rs` 这一批没动。
- 七·6 后续批的要点（实现员报告「第 1 条」那张表，行号是改前的 `transaction.rs`）：整理发批核源落点第 9917 行（找留存的映射条目）改走新加的 `TransactionOutput::mapping_entries_kept_for_the_snapshots_by_key` 索引、第 9986 行（取计划项的字节）改走批量读者已有的单元索引；`deadlist_changes_of_this_publish` 第 6116 行改走同一份留存条目索引；释放三处与快照判定那一圈按 inode 号找文件改走新的 `FilesOtherThanTheFirstOfAVersionReadByInode`；`build_extent_tree` 第 11854、11985 行与 `build_multi_level_tree` 第 14786 行照抄节点改走 `publish_admitted` 里一次发布建一次的上一版单元索引（原来第 14049 行才建，挪到装 extent 树之前）。`relocation_checked_against_the_version` 删掉了参数 `previous`。计数点分开：释放三处拆成三个 `LookupSite`，另加五个新的。
- 没改的 8 处单次调用（主工作区行号 4103、4119、6452、6481、7115、7780、10146、12839）与 `mount.rs` 第 5529 行：主 agent 判每次发布只调常数次、每次 O(P)，不是七·6 要去掉的随表长平方，不并进七·6，记进里程碑四「优化项」表「发布不再整版照抄」那一行（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 2026-10-22 那一条）。这是主 agent 的判断，没量过。
- 合入后验证（量的）：七·6 后续批第二台 fmt、clippy、build 过，测试唯一一条红是第二台盘满（`No space left on device`），本机单跑「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 1959.19s」；判法单元键那一段「✓ 测试全绿（281 个测试二进制，共 2736 条测试；跑了 3 个包…）」；形式化轻型本机「✓ checker-tier-formal-verification.sh：3 格里 3 格绿」。实现员在第二台：两档等价池 `images_sha256` 与改前逐字相同（16 文件 `66357a80…f56fa49`，48 文件 `7b3bb85d…cf90708e`）；19 行变异 `prove-red.sh` 各自点名的用例红；增长慢档 `growth_in_percent=124 index_builds_growth_in_percent=100`。崩溃轻型与变异轻型这一段还在跑，结论另报，不当前提。
- 已知、报出来不算打中的：证红之后实现员又改了两处（`transaction.rs` 一个类型别名、测试文件三处 clippy 写法），19 行原文仍各恰好命中一次，没再重跑证红（实现员报告「没做什么」）；释放三处的新用例直接调三个公开的释放入口判，不经发布路径（实现员报告：发布算定点时释放判定一次发布走 2 遍、37 次查不成整数遍）。这两件放进 T3 去攻它们的后果。
- 压着的条款：`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行，与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　七处改法的行为
核：每处新索引与改前的 `iter().find` 是不是取同一项（同一 key 有几条留存条目、同一 inode 有几个文件时取哪一条）；上一版单元索引挪到装 extent 树之前建，从建到最后一次用之间上一版的单元表会不会被改、索引会不会过期；`relocation_checked_against_the_version` 删掉参数 `previous` 之后取到的版本是不是同一份；查不到时的 panic 文案与 `Err`（`DeadlistEntryNotReleasable { KeptMappingEntryMissing }`、`PlannedUnitNotInTheVersion`）条件与改前逐字相同；改前改后同一串合法操作写出的盘镜像逐字节相同。

### T2　射程判得对不对
核：没改的 8 处单次调用是不是真的每次发布只调常数次（调它的函数自己在不在按项的循环里、会不会被每个单元或每个文件调一次）；发布与整理路径上还有没有按项调、随表长平方涨的线性查找漏在七处与 8 处之外（`iter().find`、`.position`、`.contains` 一类在 `units`、`mapped_units`、文件表、留存条目表上）；`mount.rs` 第 5529 行判成挂载路径对不对。

### T3　计数与新用例判的是不是产品调用点上的事
核：五条新快档用例里「恰好等于」的断言数的是不是产品调用点；释放三处直接调公开入口判，发布路径上那三处改回线性或每项重建时有没有一条用例红；按批大小取样（2 项、8 项）分不分得出每项重建、逐 key 查、改回线性三类；整数遍判法（快照判定那一圈、照抄三处）会不会让一个改坏的写法凑成整数遍而不红；新加的 19 行变异各自红的理由；证红之后改的类型别名与三处 clippy 写法有没有让哪一行变成改回去也不红。

### T4　`singlefs-core` 加 `links` 与构建脚本
核：`links = "singlefs-core"` 加上之后产品构建与发布构建行为是不是不变（不链接任何库，只交元数据）；`build.rs` 交出的特性名是不是 cargo 并过之后的那一份（harness 档的 dev 依赖打开的 `lookup-work-counters` 在里面）；它的 `rerun-if-changed=build.rs` 会不会让特性变了而元数据没重算；直接依赖 `singlefs-core` 的别的包同名 `links` 撞不撞。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 | 核「代码做的是不是七·6 第三轮判决第三节 1–4 条与用户定案要的那件事」：逐格对补丁原文与冻结副本的行，每条结论写文件:行号；T1 逐处列出七处的调用位置与改前改后各取哪一项；T2 逐处列出 8 处单次调用的调用者与它们被调的次数 |
| 云端攻方（Opus） | T1–T4 | 在冻结副本的拷贝上造反例：一串合法操作让新索引取到不同的项、或改前改后写出不同的盘镜像、或改后 panic 而改前不 panic；T2 找一处发布或整理路径上按项调而没改的线性查找，量它随表长怎么涨；T3 造一个改坏查法而测试不红的变异 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；用自己的编译目录。本机资源紧张就尝试其他可用的机器（`scripts/hosts/multi-host-run.sh --peer`）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
