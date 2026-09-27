# 实验执行员「入库装置先编后换」：定义与钩子三方，第一轮正文（2026-09-27）

<!-- doc-lint:not-numbers R1 R2 R3 -->

## 一、这一轮要判什么

用户 2026-09-27 弹窗选「改定义并加钩子」，关 `records/2026-09-16-subagent拆分提案.md` 第四十节第 51 行：实验执行员直接在主工作区里改 `crates/` 下的入库装置，改到一半编不过，harness 带 `--all-targets` 会连带编所有 bin，别的会话与实现员的编译一起卡住（同一天两次）。

工具实现员做了三样（报告 `/tmp/claude-1000/runner-install-hook/report.md`，sha256 `6396bac91d58d1d3…`）：

- `.claude/agents/experiment-runner.md` 第 2 步「只对入库装置生效」那一串加第 ④ 条：主工作区那份先拷进草稿目录，只在副本上改，改完经 `research/scripts/compile-then-swap.py` 编过再整份换进主工作区，交回前跑 `--clean`。
- `.claude/hooks/write-guard.sh` 加「四、先编后换」：experiment-runner 用 Write / Edit / MultiEdit 写主工作区 `crates/` 下的 `.rs` 一律拒，出路点名那条脚本；`crates/mutations.tsv`、草稿目录放行；implementation-writer 与主 agent 放行。
- `.claude/hooks/bash-command-detector.sh` 加 ⑨：experiment-runner 的 Bash 命令里 cp / mv / 重定向 / tee / sed -i / rm / python 写这类落到主工作区 `crates/` 下 `.rs` 的，拒；调用那条脚本放行。
- 新建 `research/scripts/compile-then-swap.py`：在草稿目录的仓副本上编那个 bin，编过、且主工作区那份在编译期间没被改过，才原子地换进去，换完回读逐字节比；编不过一个字节不写。

| 格 | 被攻的 | 问题 |
|---|---|---|
| R1 | **关不关得住第 51 行**：experiment-runner 还有没有别的路把编不过的代码落进主工作区 `crates/` | 钩子没拦到的写法（`git apply` / `git checkout` 一类、`cargo fix` / `cargo fmt` 这类改源码的工具、先写一个脚本再执行它、写到 `crates/` 下的非 `.rs` 但影响编译的文件如 `Cargo.toml` 与 `build.rs`、符号链接或 `..` 绕路、Bash 里没列到的写法如 `dd` / `install` / `perl -i` / `ln -sf`）；脚本放过的形态（装置不止一个文件、要改 `mod` 声明或 lib 的、带 `--features` 的、新增 bin 要改 Cargo.toml 的）；编过之后、换进之前主工作区那份被别的会话改了，竞态检查够不够 |
| R2 | **代码做的是不是定义第 ④ 条说的** | 正推：第 ④ 条每一句（拷出、只改副本、经脚本编、编过才换、交回前 `--clean`）在两个钩子与脚本里各落在哪一行、缺哪一句；定义与钩子的拒绝信息说的是不是同一件事；`.claude/agent-common.md` 里写范围、「执行前拒绝的写法」那一条要不要跟着改 |
| R3 | **误拒**：合法的操作被钩子拦下 | 逐格核一张写死的操作表（本地腿用）：执行员读 `crates/` 下文件、把主工作区那份 cp 进草稿目录、改草稿副本、追加 `crates/mutations.tsv`、写 `research/` 下的装置与 `research/e7-index-bench/Cargo.toml`、调用脚本、`--clean`；implementation-writer 与主 agent 写同一处；别的会话的子 agent 写 `crates/`。每格写钩子判拒还是放行、依据哪一行 |

**共用问句**：照今天的字面与代码，哪一步会放过、误拒或做错；给具体的工具调用 JSON、命令或文件改动；能在临时拷贝上量的量出来（钩子喂 JSON 看退出码，脚本在临时仓上跑）。

**不归这一轮的**：`experiment-runner.md` 里别的会话同一天没提交的改动（第 2 步前半段的英文名与 bin 名规矩、第 5 步复跑驱动那一句）；`mutate.sh` 不带 `--features` 的缺口（已交里程碑二收尾会话）。

## 二、实现今天的样子（主 agent 的观测，2026-09-27 JST 16:4x）

- 入库装置今天在 `crates/singlefs-harness/src/bin/` 下（`e158_root_choice_repair.rs`、`e161_crash_state_dedup_and_time_split.rs` 等），harness 带 `--all-targets` 连带编它们（singlefs-99 两次报的原样错误：`error[E0425]: cannot find value unit_check_fields`，`e161_crash_state_dedup_and_time_split.rs:5062` 等 5 处）。
- 自证现跑（原样末行）：
  - `bash .claude/hooks/write-guard.sh --selftest` → `✓ 自检通过（查了 51 种情形）：…experiment-runner 写主工作区 crates/ 下的 .rs（改、新建、.. 绕路）按先编后换拒绝并点名 compile-then-swap.py，implementation-writer 与主 agent 写同一处、执行员写草稿目录与 crates/mutations.tsv 放行…`
  - `bash .claude/hooks/bash-command-detector.sh --selftest` → `✓ 自检通过（查了 481 种）：…`
  - `python3 research/scripts/compile-then-swap.py --selftest` → `✓ compile-then-swap 自检通过（查了 11 种）：编不过的（bin 本身、#[cfg(test)]、新 bin）一个字节不写，编的时候主工作区那份被改了不盖，编过的整份换上且换的是新 inode、新 bin 建得出来，bin 以外的源文件、crates/ 以外的 .rs、放在主工作区里的草稿都不收，仓副本跑完就删、--clean 删得掉编译目录`
- `.claude/settings.json` 与 `.claude/hooks/agent-write-scope.tsv` 没改（`git diff --stat` 为空）；两个钩子的改动全是这一轮的：`git diff --stat HEAD` 为 `bash-command-detector.sh | 196`、`write-guard.sh | 68`。
- 开工快照 `research/prompts/runner-compile-first-r1-snapshot/sha256sums.txt`（5 个文件：上面四份加 `.claude/agent-common.md`）。

## 三、条款（材料员整段抄进附录）

- `.claude/agents/experiment-runner.md` 全文；
- `.claude/agent-common.md` 全文；
- `records/2026-09-16-subagent拆分提案.md` 第四十节第 51 行；
- `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」一节；
- `.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」一节。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | R1 | 在临时拷贝上造工具调用 JSON 与命令喂两个钩子、在临时仓上跑脚本，找放过；不碰主工作区 `crates/` |
| 云端正推（Sonnet） | R2 | 从定义第 ④ 条正推到三份代码的落点，缺哪一句 |
| 本地攻方 | R3 | 按主 agent 写死的操作表逐格填「拒 / 放行」与依据行号，每张表不超过 7 行 |

两条攻方腿不重叠：Opus 找「该拒没拒」，本地只逐格核「该放行没放行」。派攻方不派辩方：第一轮，没有被判出局要替之辩护的一方。

## 五、交付

- 腿的报告写 `research/prompts/runner-compile-first-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/runner-compile-first-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，去文件里现查，不从背景材料里数。
- 不跑重型测试；不改主工作区里任何文件；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；交回之前后台不许留着跑的东西。
