# 实验执行员「入库装置先编后换」三方第一轮判决（2026-09-27）

正文 `research/prompts/_runner-compile-first-r1-body.md`；背景材料 `research/prompts/_runner-compile-first-r1-background.md`；附录二 `research/prompts/_runner-compile-first-r1-diff.md`。被判的四份：`.claude/agents/experiment-runner.md`（第 2 步第 ④ 条）、`.claude/hooks/write-guard.sh`（「四、先编后换」）、`.claude/hooks/bash-command-detector.sh`（⑨）、`research/scripts/compile-then-swap.py`。开工快照 `research/prompts/runner-compile-first-r1-snapshot/sha256sums.txt`，腿跑着的时候主 agent 没改这四份。

## 一、各条腿

| 腿 | 格 | 交回 |
|---|---|---|
| 云端攻方（Opus），两次 | R1 | **缺席**：`research/prompts/runner-compile-first-r1-opus-output.md`、`runner-compile-first-r1-opus2-output.md` 两次都被安全分类器拦下一次回复、没有重写，R1 没判、没交用例。不能当「没打中」用。R1 由主 agent 现查代替（第二节），这一格算「没被攻过」 |
| 云端正推（Sonnet） | R2 | `research/prompts/runner-compile-first-r1-sonnet-output.md`，cite-check `✓ 核了 9 处引文，对上 9 处，没判 0 处` |
| 本地攻方 | R3 | 两份干净样本 `research/prompts/runner-compile-first-r1-local-attack-output-s1.md`、`-s2.md`；s2 前三次作废（void1–void3），命中的是提示事实表里的 `crates/**` 被损坏闸当成落单的强调标记，改写那三行之后第四次过闸。**s1 与 s2 用的提示在 F4、F5、F8 三行措辞不同**（改写记录 `research/prompts/runner-compile-first-r1-local-attack-translation-audit.md`） |

没派核查员：没有腿交模型、探针或产物；正推腿的引文 cite-check 已全对，本地腿交的是答复样本。

## 二、判定

| 格 | 判定 | 依据 |
|---|---|---|
| R1-a 经脚本文件间接写 | **打中（主 agent 现查，事实）**：执行员先在草稿目录写一个脚本、再执行它，由脚本写主工作区 `crates/` 下的 `.rs`，⑨ 看不见 | `.claude/hooks/bash-command-detector.sh:66` 原文「`python3 research/scripts/compile-then-swap.py …`（脚本文件里的写这里本来就判不到）」；同一天 E161 执行员用自己写的 `/tmp/claude-1000/e161-runner2/append-mutations.sh` 追加变异表，是这种写法的实例 |
| R1-b `git apply` / `patch` / `git checkout -- <文件>` / `git restore` | **打中（主 agent 现查，事实）**：不在 ⑨ 认的写法里 | `.claude/hooks/bash-command-detector.sh:61-63` 列的写法清单 |
| R1-c `crates/` 下的非 `.rs` | **打中（主 agent 现查，事实）**：`Cargo.toml` 这类改坏同样让全仓编不过，两道钩子都只认 `.rs` | `.claude/hooks/bash-command-detector.sh:65`「crates/ 下不是 .rs 的（crates/mutations.tsv）」放行；`is_main_workspace_crate_source` 判 `path.endswith(".rs")` |
| R1-d 编译标准不带 `-D warnings` | **不算缺口**：多一条警告不让别人编不过，第 51 行防的是编不过 | `research/scripts/compile-then-swap.py:142-143` 只跑 `cargo build` 与 `cargo test --no-run` |
| R1-e 竞态只核目标那一份 | **不算缺口**：别的会话同时改了库导致不兼容，是对方的改动，不归这一条 | `research/scripts/compile-then-swap.py:20` |
| R2-a 共用约束没跟上 | **打中（正推腿，主 agent 复核属实）**：`.claude/agent-common.md` 里 `compile-then-swap`、`先编后换` 零命中；「执行前拒绝的写法」Bash 检出一条列到 ⑧ 没有 ⑨；写闸一条只列三道判定，没有「四、先编后换」；脚本清单没收 `compile-then-swap.py` | `grep -c 'compile-then-swap\|COMPILE_FIRST\|先编后换' .claude/agent-common.md` → 0 |
| R2-b「交回前跑 `--clean`」没有代码核 | **打中（正推腿，主 agent 复核属实）**：两个钩子里 `--clean` 零命中；上游交回闸能不能兜底没核（推的） | `grep -c -- '--clean' .claude/hooks/write-guard.sh .claude/hooks/bash-command-detector.sh` → 0、0 |
| R2 其余 | 站住：第 ④ 条前四句在两个钩子与脚本里都有落点，拒绝信息与定义逐字一致 | 正推腿报告第四节 |
| R3 误拒 | **没打中**（两份样本十格都没判「拒」）；主要证据是钩子自检：`write-guard.sh --selftest` 51 种里含实现员与主 agent 写同一处放行、执行员写草稿目录与 `crates/mutations.tsv` 放行 | 两份样本的提示有三行措辞不同，按两次独立观测记，附这一条限定 |

## 三、改法（交工具实现员）

| 编号 | 改什么 | 该红的样本 |
|---|---|---|
| X1 | R1-c：两道钩子的路径判据从「`crates/` 下的 `.rs`」改成「`crates/` 下除 `crates/mutations.tsv` 之外的任何文件」 | 执行员 Edit `crates/singlefs-harness/Cargo.toml` 必须拒 |
| X2 | R1-b：⑨ 认 `git apply`、`patch`、`git checkout -- <路径>`、`git restore <路径>` 落到主工作区 `crates/` 的 | 执行员 `git apply /tmp/claude-1000/x.patch`（补丁动 `crates/`）必须拒 |
| X3 | R1-a：执行员 Bash 调用前后给主工作区 `crates/` 记指纹（PreToolUse 记、PostToolUse 比），变了而这条命令不是 `compile-then-swap.py` 的，就报出并把改动的文件名交给执行员、要它照先编后换重做；检出记进检出记录。形态由实现员定，要先查已有钩子能不能并 | 执行员 `bash /tmp/claude-1000/x.sh`（脚本往主工作区 `crates/` 下的 `.rs` 写一行）必须被检出 |
| X4 | R2-a：`.claude/agent-common.md` 三处补上（脚本清单收 `compile-then-swap.py`；Bash 检出一条补 ⑨；写闸一条补「四、先编后换」） | doc-lint 与门禁 62、63 号 |
| X5 | R2-b：先核上游交回闸（`handback-scratch-check`）认不认 `compile-then-swap.py` 建的编译目录；不认就让脚本的工作目录落在它认得的形态上，或在交回闸前补一条 | 执行员跑过脚本、没跑 `--clean` 就交回，必须被拦 |

X1–X5 都是主 agent 与正推腿这一轮给的，被攻过零轮；第二轮只攻这五条的改后代码。

## 回看决策

不涉及决策：改的是 agent 定义、钩子与研究脚本，不动任何 `.claude/kb/decisions/` 下的分项。
