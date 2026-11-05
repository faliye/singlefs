# single-library-defs-r1：「判定库只有一份」「所有测试多机可用」两条写进定义的第一轮正文（2026-11-01）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

用户 2026-11-01 定了两条核心策略，原话：「以后不允许各自测试简历自己库。 这是核心的策略，不允许有其他的。写进 测试agent或者skill或者主agent中。」「再加一条 所有测试都必须多机可用，不得依赖一台机器」。主 agent 照原话写进三处，用户要求不豁免、走三方：

- `.claude/rules/verification.md` 新加两节：「判定库只有一份：全仓统一写进 prune-server 原地打开的那份 RocksDB」「所有测试都要多机可用，不许依赖某一台」。
- `.claude/agent-common.md`「写」一节新加两条：「不新建判定库」「所有测试都要多机可用，不许依赖某一台」。
- `.claude/main-agent.md`「禁止」一节新加两条：「判定库只有一份，不许各自建库」「所有测试都要多机可用，不许依赖某一台」。

会红的检查还没立，欠账 C948（判定另存别处的库）、C949（测试只在一台机器上跑得起）记在 `.claude/kb/checks-owed.md`。

被判的只有上面六处新加的文字。三份文件里别的未提交改动是别的会话的，不在这一轮里（`research/prompts/single-library-defs-r1-snapshot/defs-vs-head.diff` 是整份对 HEAD 的 diff，腿只看六处新加的段）。

## 二、实现今天的样子（主 agent 的观测，2026-11-01）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）：

- 判定库的现状：崩溃判定库是 RocksDB（`crates/singlefs-checker-tier/src/verdict_store.rs`，全部表在默认列族、键首字节分表，`crash_library.rs` 的 `crash_library_directory` 指到家目录下那份 24 GB 的库）；形式化、lkmm、QEMU 共用 `scripts/crash/lib_verdict_library.py` 的 JSON 文件库，崩溃点重放快档几格与模型对拍用 `scripts/gate/admission.py` 自己的 JSON 文件库，变异表用 `scripts/mutation/lib_mutation_units.py` 的 key=value 文本库，replay.sh 用目录，广谱变异用 TSV。
- 服务：`prune-server/`（仓根下独立 Cargo 工作区）已在本机常驻；Python 三家经 `scripts/prune-server/prune_server_client.py` 接入（开关 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`），Rust 崩溃库经 `prune_server::reliable_client` 接入（补丁已打进 `crates/singlefs-checker-tier/src/verdict_store.rs`、`crash_library.rs`，`PRUNE_SERVER_TABLE = Table::Conclusions`，原键原值）。服务原地打开那份崩溃 RocksDB 的改法正在写（调度记录 `records/2026-11-01-判定库服务化.md` 批次 7）。
- 多机：task-server 把 `scripts/hosts/`、`task-server/`、`.claude/hooks/`、`scripts/agents/` 下的脚本只在本机起（`task-server/task_server.py` 的 `LOCAL_ONLY_DIRECTORIES`），别的照 `peer_eligible` 派第二台；派第二台时把本机 `multi-host.env` 原样带进那棵树的树根（`scripts/hosts/peer-host-lib.sh` 文件头）。

开工快照：`research/prompts/single-library-defs-r1-snapshot/defs-sha256.txt`（三份被判文件与 checks-owed.md）、`research/prompts/single-library-defs-r1-snapshot/defs-vs-head.diff`。

## 三、格

### K1　六处新加的文字做的是不是用户那两句说的事（正推）

- 逐处对用户原话：「不允许各自测试建立自己库」有没有被写成可执行的做法（什么算「自己的库」、要存判定该怎么存、旧文件库怎么办）；「所有测试多机可用、不依赖一台机器」有没有写成判得出来的判据（读什么算依赖一台、什么算「多机可用」、新测试什么时候算写完）。
- 三处之间（规则、共用约束、主 agent）有没有一处写得比另一处宽或窄；规则只写怎么做（`.claude/singlefs-ai-sop/rules/rules-discipline.md`）这一条守没守：有没有日期、来历、论证混进正文。

### K2　哪一格会错（攻方）

- 「判定库只有一份」与现有做法的冲突点：轻型三条在副本里跑时写哪一份库（`.claude/rules/verification.md`「两档：轻型与重型」说轻型与重型同一份库）；实验装置（E174、E178 各有自己的 RoundLibrary / SegmentFourLibrary）、replay.sh、广谱变异的 TSV——这几家按新条款算不算「各自建库」，条款写了没有、要怎么办；崩溃放量双机分片把库快照拷到第二台再并回（`scripts/crash/crash-amplification-two-hosts.sh`）与「只有一份」矛不矛盾。
- 「所有测试多机可用」的边角：真块设备与 QEMU 的测试、带 GPU 的测试、读本机 `~/.local/share/singlefs/memory-peaks.tsv` 这类运行状态的脚本、门禁 checker-tier-crates-mutation-replay 的双机分片——条款说「只能在一台上跑的不算测试、登进只在本机的表」能不能罩住；「新写的先在第二台跑过一趟才算写完」在第二台掉线时怎么办，条款写了没有。
- 两条都写「核心策略，没有例外」：有没有一处现有规则与它冲突而没改（例 `.claude/rules/verification.md` 别处还写着某家判定库的目录与取法，`.claude/agent-common.md` 别处许子 agent 在副本里起库）。

### K3　欠账够不够立成会红的检查（攻方）

- C948、C949 写的「怎么拦」能不能真做成检查：静态扫写判定的调用认得全不全（Rust 侧 `VerdictStore` 之外的写法怎么认）、「三家旧文件库目录在 HEAD 之后有新写」怎么判、「每条新测试在第二台跑过一趟」靠什么登记；判别力那一句给的样本能不能真让格转红。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1 | 核六处新加的文字做的是不是用户原话说的事、三处之间一致不一致、守不守规则只写怎么做 |
| 云端攻方（Opus） | K2、K3 | 找条款与现有做法的冲突点、边角、没罩住的家；判欠账能不能立成会红的检查 |

## 五、被判的改动

`research/prompts/single-library-defs-r1-snapshot/defs-vs-head.diff` 里六处新加的段：`.claude/rules/verification.md` 的「## 判定库只有一份」「## 所有测试都要多机可用」两节，`.claude/agent-common.md`「## 写」下以「- **不新建判定库**」「- **所有测试都要多机可用」起头的两条，`.claude/main-agent.md`「## 禁止」下以「- **判定库只有一份，不许各自建库**」「- **所有测试都要多机可用」起头的两条。同一份 diff 里别的改动是别的会话的，不判。
