# 代码欠账批 4-4 修红批的代码轮：第二轮正文（2026-10-17）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K4 K6 -->

## 一、这一轮要判什么

代码欠账批 4-4 第一轮代码轮（`research/prompts/m3-b44-code-r1-main-verification.md`）判下的修红批，由实现员 W8–W12 写完，补丁已打进主工作区，合入后验证测试全绿（`merge-and-verify.sh --skip-apply`：218 个测试二进制 2292 条，fmt、build 过）。W9、W12 只改测试代码，不在射程里。这一轮攻 W8、W10、W11 对四份产品代码的改动：`crates/singlefs-core/src/` 下 `mount.rs`、`recovery.rs`、`transaction.rs`、`mounted_session.rs`，判「代码做的是不是条款说的」「哪一格会错」。

被判的改动全文：`research/prompts/_m3-b44-code-r2-diff.md`（这四份文件在 W8 之前与三份补丁都打上之后的 diff；W8 之前的样子是在草稿里把 W11、W10、W8 三份补丁依次倒着撤掉还原的）。实现员报告：`/home/fy5090/.local/share/singlefs/work/m3-b44-w8/report.md`、`m3-b44-w10/report.md`、`m3-b44-w11/patch/report.md`。调度记录：`records/2026-10-13-代码欠账批.md` 里 2026-10-16「W8 交回」到「4-4 修红批写完」那几条。

出口：每格判「代码对得上条款 / 对不上（给反例）」；对不上的写成改法（并进验证段的下一批修红）或交用户。

## 二、实现今天的样子（主 agent 的观测，2026-10-17 现查）

冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r2/repo/`：主工作区此刻的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus/`、`.claude/heavy-thread-default`。`crates/*/src/` 下 `.rs` 与 `.wgsl` 的 sha256 在 `research/prompts/m3-b44-code-r2-snapshot/crates-sha256.txt`。行号按冻结副本。方案按 `crates/` 今天的实现来谈。

几个落点（冻结副本里现查的函数名，行号以冻结副本为准）：

- `recovery.rs`：`container_tree_pointers_read_from_disk`、`pointers_of_the_rebuilt_version_ascend_by_device`（W8 把墓碑树、谱系树、退役树的容器指针与根之下节点指针并进 I-2.5 那一判，W10 补根之下码 2 节点指针的用例）。
- `transaction.rs`：`rehearse_every_step_of_the_deletion_queue_walk`（W10：把删除队列走到空的每一步都只算不写地预演，每一步接在上一步预演出的那一版上）。
- `mount.rs`：`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`（取号之前那一拒）、`acquire_then_publish_until_the_continuation`（W11 抽出的取号到续做之前那一串，真做与预演共用）、`continue_the_deletion_queue_until_it_is_empty`；预演遇到空间不够时，在只进内存的盘与分配器拷贝上把取号之后那一串照真做走一遍，只拒 C841。
- `mounted_session.rs`：`DeviceKeepingWritesInMemory`（写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘），W11 把它的可见性放到 `pub(crate)`。

主工作区里别的会话还有没提交的改动（`crates/singlefs-checker-tier/`、`crates/singlefs-unit-graph/`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/` 的几份），不在这一轮射程里；冻结副本照原样带着它们，编译要用。

## 三、格

| 格 | 改动 | 压着的条款 | 判什么 |
|---|---|---|---|
| J1 | 重建时 I-2.5 那一判并进三棵容器树（墓碑、谱系、退役）的容器指针与根之下节点指针（W8、W10） | I-2.5（`crates/singlefs-checker/invariants/04-校验和.md` 第 10 行）；第一轮判决 K4 那一行 | 并进清单的指针全没全（这一版里还有哪类节点指针没进这一判）；坏镜像在任何写之前交类型化错误、两块盘不变；会不会误拒合法写路径写出的镜像（全零空指针、单盘池、同一指针两条位置条目的合法形） |
| J2 | 取号之前把删除续做的每一步都预演一遍（W8、W10） | 岔路单 `research/prompts/m3-b44-code-r1-forks.md` 第一行（用户选「取号之前预演」）；用户 2026-10-16 定「预演整个续做」（调度记录「W8 交回」那一条末句） | 预演与真做读的是不是同一份输入（写行那次发布会不会改动续做看到的树、预演每一步接的那一版与真做那一步看到的是不是同一版）；只拒 C841（`PublishError::NextContainerNumberPastTheTopOfItsRange`），别的错照旧留给续做；预演不写盘、不动真分配器；拒了之后两块盘逐字节不变、再挂一次同样拒 |
| J3 | 预演遇到空间不够时，在只进内存的盘与分配器拷贝上把取号之后那一串（推抬 F、写行、续做）照真做走一遍（W11） | 用户 2026-10-16 定「抬 F 也预演进去」（调度记录「W10 交回」那一条末句）；`mounted_session.rs` 里 `DeviceKeepingWritesInMemory` 的文档与推环预演那一段 | 内存里走的那一串与真做是不是同一段代码、同一份输入（分配器拷贝、时刻、实例代号、读到的盘）；只进内存的盘的语义与真盘有没有分得开的地方（写零、读没写过的块、读故障、设备报错）；预演多出的读落在底下那块盘上，对带读故障注入的设备会不会改变真做那一趟看到的读结局；内存占用有没有上界（续做很长、盘很大时） |
| J4 | 取号之前那一拒复用 `MountError::Publish`、写账是空的（W8、W10、W11） | `mount.rs` 里 `MountError` 各成员的文档；C841 用户定案（调度记录「C841「写路径报类型化错误」」那一条） | 调用方（`mounted_session.rs`、harness 的模型对拍与随机历史）按 `MountError::Publish` 认的东西（写过没有、写账里有什么）在这一拒上还成不成立；有没有调用方因此把「没写」当成「写了一半」 |
| J5 | W8 新写、W10 删过日期与经过的注释 | `.claude/singlefs-ai-sop/rules/code-discipline.md`「注释只写为什么」 | 注释说的是不是今天代码做的事；删经过时有没有把「为什么」一起删掉 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 逐格核「代码做的是不是条款说的」：每一格引条款原文与冻结副本的代码行，对得上与对不上逐句写 | J1–J5 |
| 云端攻方（Opus） | 攻「哪一格会错」：在冻结副本的拷贝上造镜像、历史或崩溃状态，找让改动判错（写掉还在用的单元、误拒合法态、预演与真做结局不同、panic、盘变了）的那一段 | J1、J2、J3、J4，有余力再 J5 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格两条腿都没给出反例 ⇒ 那一格代码对得上条款 | 两份报告那一格都写试过的形、没打中 |
| 一格被造出判错的段 ⇒ 那一处要改，改法并进修红批 | 一段可复现的镜像、历史或崩溃状态 |
| 一格的反例落在条款没写的地方 ⇒ 交用户定条款 | 反例成立，而条款对那一形没有一句话 |

⚠️ 打中之后，先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告：
  - 先写进 `research/prompts/m3-b44-code-r2-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`）。
  - 每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建。
  - 回复只写短句；早点开写。
- 引文：
  - 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
  - 引代码写冻结副本里的路径与行号。
- 草稿与模型：
  - 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-b44-code-r2-<腿名>/`。
  - 量法拷冻结副本，不在冻结副本里改。
  - 模型与驱动随报告拷进 `research/prompts/m3-b44-code-r2-<腿名>-model/`，含 `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh`。
  - 编 rocksdb 的那一步带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。
- 资源：
  - 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 24G 的上限。
  - 同一时刻只起一条 cargo；用自己的编译目录，不从别处拷编译产物。
  - 前台命令预计超过 4 分钟的拆成小段。
  - 一段历史的崩溃状态全量超过约 10⁶ 个的，不跑，写进报告「要主 agent 跑的命令」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
