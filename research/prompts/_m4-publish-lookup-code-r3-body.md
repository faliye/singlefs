# m4-publish-lookup-code-r3：里程碑四七·6 发布路径按身份查，代码轮第三轮正文（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

## 一、题面

这一轮攻七·6 第三批补丁，它照第二轮判决 `research/prompts/m4-publish-lookup-code-r2-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」两条与「五、交用户」用户定并进的两处写成，已合进主工作区。这是七·6 代码轮的第三轮、最终轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮的分工：前两轮判「成立」的撑不撑得住（第三批改了它们的前提）、要攻而没列进格的地方、这一批报出的数能不能核。前两轮已攻过的角度不重复，除非第三批改动了它们的前提；第三批新冒出来、只被攻过零轮的形态写进判决的交用户表，不为它再开一轮。

改动全文：`research/prompts/_m4-publish-lookup-code-r3-diff.md`（`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv` 原样）。实现员报告：`~/.local/share/singlefs/work/m4-publish-lookup-r3-writer/report.md`（sha256 `75ef78f0da4aa27524783eca01ed4a9cda0edfe13d83a9714962908e482fa479`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-21）

- 冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo/`（合入七·6 第三批之后的主工作区，去掉了私有配置 `multi-host.env`、`.env`、`.git` 与编译目录）。被判文件与引到的 kb 的 sha256 在 `research/prompts/m4-publish-lookup-code-r3-snapshot/sha256sums.txt`。
- 产品代码改动在 `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/Cargo.toml`；测试侧 `crates/singlefs-harness/Cargo.toml`（dev 依赖打开特性）与 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs`。要点（行号是主工作区合入后的）：
  - 按身份查的索引换成排好序的数组加折半查（`transaction.rs` 模块 `sorted_identity_index`，第 3440 行 `keeping_the_first_value_of_each_identity`：稳定排序之后 `dedup_by` 留每个身份最先那一项）。
  - 计数模块 `transaction::lookup_work`（特性 `lookup-work-counters`，默认关；计数存在线程局部变量 `LOOKUP_WORK_ON_THIS_THREAD`，第 3353 行起），数建索引次数、查的次数、看过索引几项、解中央映射叶几次，按几处分开记。原来读缓存表键数的 `leaves_decoded` 删了。
  - 用户 2026-10-21 定并进的两处：`relocation_checked_against_the_version` 改用按 key 的索引 `MappedUnitsOfAVersionReadByKey` 加批量读者；`mapping_locations_of_a_mapped_unit` 删掉，三个调用点改用 `MappedUnitsOfAVersionReadByRole`。
  - 增长慢档用例改读计数，两档用 `std::thread::scope` 并行。
- 合入后验证（量的）：第二台测试「✓ 测试全绿（267 个测试二进制，共 2547 条测试；跑了 2 个包…）」；fmt、clippy 过；本机 `cargo build --all-targets` 24G 上限下退 0、没有 warning（第二台默认 8G 那一趟撞上限被杀）；形式化轻型 3 格绿；debug 下 `publish_lookup_by_identity` 连慢档「test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 97.94s」；变异轻型（第二台，核心部分加点名 13 项）「计数：抓到 257 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条、锚点失效 0 行（共 258 条）」，另基线不绿 1 行是别的会话的 C656 那一行（与七·6 无关）；崩溃轻型结论另报，不当前提。
- 已知、报出来不算打中的：带不带 harness 测试的两种编法里 `singlefs-core` 的单元键不同（特性的 cfg 原子进键，实现员报、推的）——这一格放进 T4 去攻它的后果，不是把它当打中；`relocation_checked_against_the_version` 每批的代价没量过。
- 压着的条款：`.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行（用户 2026-10-21 定把两处并进来之后的样子）与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　前两轮判成立的，在新索引下还成立吗
第一轮 T1（索引与线性查找取同一项）、T3（查不到时的行为）与第二轮 T3（快照判定那一圈）都靠旧索引的性质判成立。核：稳定排序加 `dedup_by` 是不是对每种身份类型都留下原表里最先那一项（身份的 `Ord` 与 `==` 是不是一致）；折半查在空表、单项、全同身份、查不到时的行为与线性找逐项相同；查不到时的 panic 与错误文案、条件与改前逐字相同。

### T2　并进来的两处查法换了之后的行为
`relocation_checked_against_the_version` 按 key 找角色、`mapping_locations_of_a_mapped_unit` 的三个调用点。核：同一 key 对应几个角色、同一角色几把 key 时改前改后取到的是不是同一项；索引与批量读者在发布路径上建了之后表会不会被改；查不到时 `ReleaseNotInMapping` 等错误与 panic 是不是逐字相同；改前改后同一串合法操作写出的盘镜像是不是逐字节相同。

### T3　计数判的是不是产品调用点上的事
计数存在线程局部变量里。核：产品代码里有没有把查找放到别的线程（`std::thread::scope` 这类）上做、因而漏数；有没有不经 `lookup_work` 的查找路径（直接走 `sorted` 或另一份索引）；测试读计数的方式在用例并行（两档 `std::thread::scope`、libtest 的多线程）时会不会串数；门槛（200%）在登记的取样规模上会不会被别的因素推过线或压下线；改回线性、每次重建索引、每把 key 重解叶这几类改坏，计数是不是一定涨。

### T4　特性与变异
特性 `lookup-work-counters` 默认关、只由 harness 的 dev 依赖打开。核：发布构建与 checker 档构建里计数是不是真的编没了（零开销、行为不变）；工作区里一起编 harness 测试时整个 `singlefs-core` 带着计数编，这会不会改变 checker 档崩溃判定库的单元键、让判定白白现判或错误复用；新加与换点名的变异行（`crates/mutations.tsv` 末尾七·6 那几行）各自让点名的测试红的理由，有没有一处第三批的改法改回去而没有任何一行红。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 | 核「代码做的是不是第二轮判决第三节与用户定案要的那件事」：逐格对补丁原文与冻结副本的行，每条结论写文件:行号；T2、T3 要逐处列出两处查法与每个计数点的调用位置 |
| 云端攻方（Opus） | T1–T4 | 在冻结副本的拷贝上造反例：一串合法操作让新索引取到不同的项、或改前改后写出不同的盘镜像、或改后 panic 而改前不 panic、或产品的查找没被计数数到；T3、T4 造一个改坏查法或改坏判据而测试不红的变异，或一个让崩溃判定库单元键错误复用的编法 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷过去的源文件先 `find crates -type f -exec touch {} +` 再编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 16G 的上限，同一时刻只起一条 cargo；用自己的编译目录。本机排不上时照包装打出的那一行改去第二台（`scripts/hosts/multi-host-run.sh --peer`）。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
