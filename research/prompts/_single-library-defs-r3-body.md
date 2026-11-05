# single-library-defs-r3：「判定库只有一份」「所有测试多机可用」第三轮（最后一轮）正文（2026-11-01）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

前两轮判决 `research/prompts/single-library-defs-r1-main-verification.md`、`research/prompts/single-library-defs-r2-main-verification.md` 都判打穿，打中的各自改回。这一轮是第三轮、最后一轮：只攻前两轮站住的形态，不为这一轮冒出的零轮形态再开一轮。

这一轮判的文字：

- `.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」「所有测试都要多机可用，不许依赖某一台」两节（第二轮判决第四节改后的第三版）。
- `.claude/agent-common.md`「写」一节「不新建判定库」「所有测试都要多机可用，不许依赖某一台」两条。
- `.claude/main-agent.md`「禁止」一节「判定库只有一份，不许各自建库」「所有测试都要多机可用，不许依赖某一台」两条。
- `.claude/kb/checks-owed.md` C948、C949 第三版的「怎么拦」与前置。

用户定案（原话）：「以后不允许各自测试简历自己库。 这是核心的策略，不允许有其他的」「所有测试都必须多机可用，不得依赖一台机器」「不加任何参数 删掉布局选择 只能用 rocksdb」「不得原地打开。 迁移完成后 数据无误，删掉原来的数据库」「把库合并到一起 用rustdb」；task-server 监控会话转的「所有的脚本必须多机跑。抛出的结果需要多机都能读取」「所有的服务必须要不受限制的多机跑」。

## 二、实现今天的样子（主 agent 的观测，2026-11-01）

方案按 `crates/`、`scripts/`、`prune-server/`、`task-server/` 今天的实现来谈（主工作区现状）：

- prune-server：单一布局（默认列族、键原样、选项照 `crates/singlefs-checker-tier/src/verdict_store.rs`），记账行 0xF0、0xF1 起头放同一份库，迁移子命令 `migrate-from`，在仓副本里 clippy 过、自证正在跑；本机常驻的还是上一版（多列族布局）服务，换二进制前要挪开现有库。
- 崩溃判定库：删兼容那一批已把两台共用库换成新目录（布局号 6、表标签 00–1B），旧的 layout-5 那一份（24 GB）与新读法不兼容、不迁。Rust 接服务的那一层已打进 `verdict_store.rs`、`crash_library.rs`（键原样），`cargo check -p singlefs-checker-tier --all-targets` 绿，测试没跑。
- Python 三家接服务的客户端自证全绿；verdict-library 一家 7200 条已在服务里、`--verify` 相同；形式化轻型带开关与不带开关两趟（2516、2517）在排。
- task-server：带开关的 Python 三家放行派第二台，崩溃三道不放；只派一台的几张表每条当欠账。

开工快照：`research/prompts/single-library-defs-r3-snapshot/defs-sha256.txt`、`defs-vs-head.diff`。

## 三、格

### K1　前两轮判决站不站得住（辩方）

- 第二轮判决第二节每一格的「主 agent 判」与第四节的改法对着此刻的文字核：落了没有、有没有改过头（把用户没说的写成了核心策略）。
- 第二轮判决替用户不定的那一条（「要让一条任务必须在第二台跑一趟得改『提交不挑台』」）这样处置对不对；第一轮判决翻掉攻方两条、第二轮判决把「没有例外」保留并把例外子句归成「不算测试的活」，站不站得住。

### K2　站住的形态哪一格会错（攻方）

- 「判定库的内容：复用要读的一切」这个宽定义：会不会把不该进库的东西（产物、日志、临时现场）也拉进去；对它的反例。
- 一族「接完」三件（缺省走服务且删本地路径、`--verify` 只判缺与异、旧库删）与登记表第三列写「prune-server」：三件的次序会不会出错（先删本地路径再 verify？）、「接完」之后这一族的任务不带开关怎么办、过渡期几族接完几族没接完时同一道门禁怎么办。
- 「第二台那一趟全部现判、结局逐项相同」这一判据：全部现判的代价（一趟多久）与它能不能真判出「依赖一台」；哪些测试全部现判也判不出（结局不取决于台）。
- 「许读的本机状态」那张清单与「只派一台的表每条都是欠账」：清单外的本机状态（例 `~/.config/singlefs/` 下的密钥、`.claude/heavy-thread-default`）算不算依赖一台。

### K3　C948、C949 第三版立得成立不成（攻方）

- C948 的前置清单（九样接进服务、文件头写键前缀、`CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 非 0 判红、认「目录存判定文件」与「旧目录被重建」）够不够把格立起来；判别力两份样本能不能分出两种实现。
- C949 动态格改成「第二台全部现判」：这一格怎么拿到第二台那一趟（谁交、怎么认是第二台）；静态格按新词新射程的存量怎么分合规与违例。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核前两轮判决：落了没有、改过头没有、替用户不定的那一条处置对不对 |
| 云端攻方（Opus） | K2、K3 | 只攻前两轮站住的形态；不重复前两轮攻过的角度（见两份判决第二节） |

## 五、被判的改动

`research/prompts/single-library-defs-r3-snapshot/defs-vs-head.diff` 里属于这一题的段：`verification.md` 两节、`agent-common.md`「写」两条、`main-agent.md`「禁止」两条；`checks-owed.md` 的 C948、C949 两行（快照 `defs-sha256.txt` 记了它的 sha256）。同一份 diff 里别的改动是别的会话的，不判。
