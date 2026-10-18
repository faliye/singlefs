# 代码欠账批 4-4 的代码轮：第一轮正文（2026-10-15）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 K8 -->

## 一、这一轮要判什么

代码欠账批 4-4 写段六个实现员（W1–W6）的补丁已打进主工作区并通过合入后验证（`merge-and-verify.sh`：213 个测试二进制 2241 条全绿，fmt、clippy、build 过）。这一轮攻这一批合入的产品代码：`crates/singlefs-core/src/` 下 24 份与 `crates/singlefs-format/src/lib.rs`，判「代码做的是不是条款说的」「哪一格会错」。

被判的改动全文：`research/prompts/_m3-b44-code-r1-diff.md`（`git diff HEAD -- crates/singlefs-core crates/singlefs-format`，HEAD c1e9b324）。六份实现员报告：`/home/fy5090/.local/share/singlefs/work/m3-b44-w1/report.md` 到 `m3-b44-w6/report.md`。调度记录：`records/2026-10-13-代码欠账批.md`「4-4 写段第一批」那一节与其后各条。

出口：每格判「代码对得上条款 / 对不上（给反例）」；对不上的写成改法（并进验证段的修红批）或交用户。

## 二、实现今天的样子（主 agent 的观测，2026-10-15 现查）

冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r1/repo/`：主工作区此刻的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus/`、`.claude/heavy-thread-default`。`crates/*/src/` 下 `.rs` 与 `.wgsl` 的 sha256 在 `research/prompts/m3-b44-code-r1-snapshot/crates-sha256.txt`。行号按冻结副本。方案按 `crates/` 今天的实现来谈。

主工作区里别的会话还有没提交的改动（`crates/singlefs-checker-tier/`、`crates/singlefs-unit-graph/`、`crates/singlefs-checker/src/walk.rs`），不在这一轮射程里；冻结副本照原样带着它们，编译要用。

## 三、格

| 格 | 改动 | 压着的条款 | 判什么 |
|---|---|---|---|
| K1 | `mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots` 那一判换成 `one_pointer_has_both_copies_overwritten_by_a_later_instance`（W1） | 用户 2026-10-15 定案（`research/prompts/c834-r3-main-verification.md`「## 四、用户定案（2026-10-15）」，岔路单 `research/prompts/c834-r3-forks.md`）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（第 392 行那一句还没写回） | 实现与定案逐句对不对得上：读哪几个槽、按什么长度读、比的是哪一个实例代号、空指针、读不出时走哪一支；有没有 R 走得完而它放过并写掉的历史；有没有入库 `urJ` 那一形它拒的 |
| K2 | A1、A2 两条用例与 A3 变异行（W1） | `research/prompts/code-backlog-b2-code-r1-main-verification.md`「## 三、采纳的改法」A1–A3 | 用例钉住的是不是判决要钉的那一道 |
| K3 | `system_configuration.rs` 的 `parse_slot` 判节点大小（W2） | 用户 2026-10-15 定 C840「当自证不过」（`.claude/kb/checks-owed.md` C840 那一行） | 不等时这一槽不可择、别的槽照择；拒因成员；会不会误拒合法的槽 |
| K4 | `pointer.rs` 写者断言改成「全零空指针照放、其余严格 `<`」（W2） | I-2.5（`crates/singlefs-checker/invariants/04-校验和.md` 第 10 行） | 有没有合法写路径会写出同盘两条位置条目的非空指针；全零那一例外认得对不对 |
| K5 | `recovery.rs` 的 `allocation_records_are_one_per_device` 与 `unit.rs` 的 `index_node_entry_capacity` 的前提断言（W2） | 外部审核 ①②（`records/2026-10-13-代码欠账批.md`「2026-10-14 用户贴来一份外部代码审查（六条）」那一条） | 产品调用方会不会走到新断言；「有记录、盘表空」改前交 `false`、改后 panic 这一处行为变化的射程 |
| K6 | `container_tree.rs` 开新容器 checked 加法，`PublishError::NextContainerNumberPastTheTopOfItsRange`，三棵树交 `Result`（W6） | 用户 2026-10-15 定 C841「写路径报类型化错误」 | 错误在发写之前交回、盘不变；三处穷举 `match` 的处置对不对 |
| K7 | 删死代码与只动注释的那几十处（W3、W4、W6；C842 删 `txg_is_inside_the_one_revolution_window`） | `.claude/kb/milestone/03-third-txn.md` 五·10 那一行；回扫报告 `/home/fy5090/.local/share/singlefs/work/m3-item5-10-sweep/report.md` | 删掉的是不是真的零调用、不改行为；改过的注释是不是说对了今天的代码 |
| K8 | `mounted_read.rs` 的 `ExtentTreeReadsAtOpen.leaves`、inode key 宽合成 `inode_tree.rs` 的 `INODE_KEY_WIDTH_IN_BYTES`（W6） | 同一份记录「W4 交回」「W6 交回」两条 | 计数改法对不对；常量合并有没有漏掉的字面量 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 逐格核「代码做的是不是条款说的」：每一格引条款原文与冻结副本的代码行，对得上与对不上逐句写 | K1–K8 |
| 云端攻方（Opus） | 攻「哪一格会错」：在冻结副本的拷贝上造镜像、历史或崩溃状态，找让改动判错（写掉还在用的单元、误拒合法态、panic、行为变了）的那一段 | K1、K3、K4、K5、K6，有余力再 K7、K8 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格两条腿都没给出反例 ⇒ 那一格代码对得上条款 | 两份报告那一格都写试过的形、没打中 |
| 一格被造出判错的段 ⇒ 那一处要改，改法并进修红批 | 一段可复现的镜像、历史或崩溃状态 |
| 一格的反例落在条款没写的地方 ⇒ 交用户定条款 | 反例成立，而条款对那一形没有一句话 |

⚠️ 打中之后，先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、交付要求

- 报告：
  - 先写进 `research/prompts/m3-b44-code-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`）。
  - 每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建。
  - 回复只写短句；早点开写。
- 引文：
  - 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
  - 引代码写冻结副本里的路径与行号。
- 草稿与模型：
  - 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-b44-code-r1-<腿名>/`。
  - 量法拷冻结副本，不在冻结副本里改。
  - 模型与驱动随报告拷进 `research/prompts/m3-b44-code-r1-<腿名>-model/`，含 `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh`。
  - 编 rocksdb 的那一步带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。
- 资源：
  - 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 24G 的上限。
  - 同一时刻只起一条 cargo；用自己的编译目录，不从别处拷编译产物。
  - 前台命令预计超过 4 分钟的拆成小段。
  - 一段历史的崩溃状态全量超过约 10⁶ 个的，不跑，写进报告「要主 agent 跑的命令」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
