# m4-publish-lookup-code-r1：里程碑四七·6 发布路径按身份查，代码轮第一轮正文（2026-10-20）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

## 一、题面

这一轮攻里程碑四七·6 的一份补丁（`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行），它已合进主工作区。用户 2026-10-20 定「先修四处线性查找」：E185（GC一批搬迁的挂钟拆分） 第二段量出一批整理的线程 CPU 98% 以上不在区域扫描、CRC 与选区，调查员（报告 `~/.local/share/singlefs/work/gc-batch-cpu-by-pool-size/report.md`）把它定位到每次发布与整理读版本时对约 P 个元素各在长约 P 的 `Vec` 上 `iter().find`。补丁把五处改成按身份建一次的索引，要求行为一个字节不变、不改盘上格式、不改 `units` 的次序。

改动全文：`research/prompts/_m4-publish-lookup-code-r1-diff.md`（`crates.patch` 与 `mutations-append.tsv` 原样）。实现员报告：`~/.local/share/singlefs/work/m4-publish-lookup-by-identity/report.md`。

## 二、实现今天的样子（主 agent 的观测，2026-10-20）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo/`（合入七·6 之后的主工作区，去掉了私有配置 `multi-host.env`）。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-code-r1-snapshot/sha256sums.txt`。
- 产品代码改动只在 `crates/singlefs-core/src/transaction.rs` 与 `crates/singlefs-core/src/mounted_session.rs`。冻结副本里的落点：`transaction.rs` 第 3192 行 `FirstEntryOfEachIdentity`（第 3225 行它的 `unit`）、第 3626 行 `file_other_than_the_first`、第 3713 行 `TransactionOutput::unit`（仍是 `iter().find`，热路径改走索引）、第 4255 行 `CentralMappingOfAVersionReadLeafByLeaf`、第 7001 行 `mapping_key_carried_from`、第 11252 行 `carried_unit`；`mounted_session.rs` 第 2782 行 `version_as_the_compaction_reads_it`。测试 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs`（新），变异 `crates/mutations.tsv` 末尾「七·6 按身份查」七行、「七·6 增长」两行。
- 合入后验证：fmt、clippy（带 check.sh 的七条编码纪律 lint）本机过；编译与测试在第二台跑（本机内存排不上），`cargo test --offline --no-fail-fast -p singlefs-core -p singlefs-harness` 254 个测试二进制 2489 过、0 失败、78 忽略，`cargo build --offline --all-targets` 过（日志 `~/.local/share/singlefs/work/multi-host-run/merge-verify-m4-publish-lookup-2026-10-20/`）。变异轻型与崩溃轻型这一刻还在跑，结论交回时另报，不当前提。
- 实现员在副本里量的（报告「等价」「增长」两节，被攻过零轮）：改前改后两份二进制跑同一串操作，盘镜像 sha256 在五组上逐字节相同；一批整理 CPU（E185 配方，K = 8，release，同一会话轮流跑中位）P = 256 10.00→4.98、1024 28.98→15.50、4096 136.48→63.00 ms。
- 已知、记作欠账、报出来不算打中的：「整版照抄」那一层（版本装全部单元字节、整版克隆、分配记录每轮重新分组）不在这一轮，记在 `.claude/kb/milestone/04-fourth-txn.md`「优化项」表「发布不再整版照抄」一行；extent 树与多层码 2 树照抄节点时各调一次 `TransactionOutput::unit`（约 P² ÷ 扇出），不在点名的五处里，记延后；增长慢档用例的两行变异还没证红。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　索引与线性查找是不是同一项
`FirstEntryOfEachIdentity` 取同一身份最先那一项，声称与 `iter().find` 同。核：五处每一处原来查的那张表里同一身份会不会出现不止一次（实例表链的各片、照抄与新装的同一角色、`units` 末尾实例表各片的次序），出现时原来取到的是哪一项、现在取到的是哪一项；索引建的那一刻之后那张表还会不会被改（实现员说「索引活着时那一版改不了」，`units` 是公开字段、别处有就地改），有没有一处在建了索引之后改表再查。

### T2　整理读版本一遍走叶与逐 key 查是否逐项相同
`CentralMappingOfAVersionReadLeafByLeaf` 每片叶只解一次，声称三种情形（叶解不开、条目窄于字段表、同一把 key 有几条取最后一条）都与逐 key 走 `central_mapping_locations_in_the_version` 一样。核：逐 key 查时路由（`leaf_position_routed_to`）把一把 key 送到哪片叶，一遍走叶时这把 key 落在哪片叶，两者会不会不同（key 不在它路由到的那片叶里、几片叶都有同一把 key）；`MappingLookup` 的每一种结局在两种走法下产出的 `CentralMappingEntry` 集合与次序是否相同。

### T3　照抄路径（`carried_unit`、`mapping_key_carried_from`、`file_other_than_the_first`）
三处原来在上一版上线性找；改成索引后，查不到时的行为（原来 `expect` 的 panic 文案与条件）是否不变；`settle_the_allocation_record_tree` 每轮 `resolve` 里调 `mapping_key_carried_from`，索引是每轮现建还是跨轮复用，跨轮复用时中间那一轮改没改表。

### T4　测试与变异判得出它要判的
等价测试比的是不是改前改后真不同的路径（还是两边都走新索引、比了个寂寞）；七行「按身份查」变异各自让点名的测试红的理由；两行「增长」变异在门槛 400% 上分不分得出（实现员估「整理读版本退回逐 key 查映射」那一行可能不红）。

### T5　有没有改到「行为不变」之外
补丁里除了查法之外有没有别的行为改动（次序、错误分支、文案、公开 API 的签名），有没有改到盘上格式或写出的次序。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T5 | 核「代码做的是不是题面说的那件事」：逐格对补丁原文与冻结副本的行，每条结论写文件:行号；T1、T2 要逐处列出原查法与新查法在重复身份、跨叶 key 上各取哪一项 |
| 云端攻方（Opus） | T1–T5 | 在冻结副本的拷贝上造反例：一串合法操作让改前改后写出不同的盘镜像、或一处查到不同的项、或改后 panic 而改前不 panic；T4 造一个改坏查法而测试不红的变异 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；线程上限 8；用自己的编译目录。本机排不上时照包装打出的那一行改去第二台（`scripts/hosts/multi-host-run.sh --peer`）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
