# 审查报告 B：.claude/agents/ 十四份定义与 .claude/agent-def-review-exempt（分支 claude/exciting-bohr-4olowk 相对 73ba4a4，HEAD 71f0cbc）

时刻均为 UTC。行号用 `grep -n` 现取。钩子判定用合成 PreToolUse JSON 真喂（`AGENT_HOOK_DETECTIONS` 指到本草稿目录），辅助脚本 `review1/hk.sh`。

## 逐条

| # | 文件:行 | 类别 | 原句（整句抄） | 证据 | 建议改法 |
|---|---|---|---|---|---|
| 1 | .claude/agents/gate-triage.md:26 | 错误 | 「54、55、57、59 这几道重阶段在 `gate.sh` 里各走各的：54 号跑快档并核全绿标记，55、59 照 `.claude/gate.d/stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑；」 | 定义让分诊员跑的是 `bash .claude/scripts/gate.sh --staged`，不是 `research/scripts/gate-staged.sh`。复用判定 `research/scripts/stage-must-run.sh:51-52`：`staged_tree="${SINGLEFS_STAGED_TREE:-}"` / `[[ -n "$staged_tree" ]] \|\| say_run "这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用"`；全仓只有 `research/scripts/gate-staged.sh:27-28` 导出这个变量（`grep -rn SINGLEFS_STAGED_TREE .claude/ research/scripts/*.sh` 在 .claude/ 下零命中）。现跑 `env -u SINGLEFS_STAGED_TREE bash research/scripts/stage-must-run.sh "$PWD" 59-crates-mutation-replay.sh` 原样输出「这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用」rc=0（要跑）。另 `git rev-parse --verify -q refs/sop/staged-green` rc=1：这条 ref 在本仓不存在，而只有 gate-staged.sh 前移它（`gate-staged.sh:43`；共享 `gate.sh` 只写 `refs/sop/gate-ok`，`.claude/singlefs-ai-sop/scripts/gate.sh:535`）。结论：照定义的命令，55、59（还有 87）每次提交都现跑，崩溃验证员刚跑过的 55、59 在同一次提交里再跑一遍（59 号是小时级）。同一句也在 `.claude/main-agent.md:59` 与 `.claude/rules/implementation-workflow.md:55`（「55、59 靠「输入没变就复用上一次全绿判定」不重跑」），不在我负责的文件里，一并报 | 二选一：① 定义第 2 步的命令换成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（heavy-test-guard 对 gate-triage 放行，实测 rc=0；写范围一节补上它会 `update-ref refs/sop/staged-green`）；② 保留命令，把这句改成「直接跑 `gate.sh --staged` 时 55、59、87 不复用、照跑」，并让主 agent 表与 implementation-workflow.md 同步 |
| 2 | .claude/agents/crash-verifier.md:24、25 | 不一致 | 「54 号不归你：快档在 `gate.sh --staged` 的 HEAD + 暂存区树上跑才算得对全绿标记的键（在主工作区跑，工作区与暂存区不同就判红），全量由主 agent 在 worktree 里跑。」 | 这一句是 governance-defs-r1 判决 A1 的写回：理由是主工作区 ≠ 暂存区时判的不是要提交的那一批。同一理由对第 2 步仍让它在主工作区跑的 55、59 同样成立：`.claude/gate.d/stage-inputs.tsv` 里 55 读 `crates/ Cargo.toml Cargo.lock research/scripts/ research/results/`、59 读 `crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh`；`55-qemu-first-transaction.sh:27` 与 `59-crates-mutation-replay.sh` 的 `ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"` 即主工作区。别的会话在 `crates/` 有未暂存改动时，崩溃验证员的 55、59 绿判的是工作区那一份。第 6 步的指纹是 `git diff HEAD -- crates litmus`（工作区相对 HEAD），分不出工作区与暂存区不同，这一格没有任何东西报警。与第 1 条合起来：分诊员那一趟 `gate.sh --staged` 在暂存树上重跑了 55、59，所以真正判暂存内容的是分诊员那次，崩溃验证员这次既不被复用、又不对应要提交的树 | 二选一：① 崩溃验证员的 55、59 也放进 HEAD + 暂存区的 worktree 跑（照 54 号 `print_staged_worktree_full_commands` 的建法），或只在 `git diff --quiet -- crates Cargo.toml Cargo.lock research/scripts` 为真（工作区与暂存区在这几条路径上相同）时跑，不同就停下报；② 第 6 步的指纹加一项 `git diff -- crates litmus \| sha256sum`（工作区相对暂存区），非空就写明「这几个绿对应的不是暂存区那一份」 |
| 3 | .claude/agents/mutation-triage.md:28 | 错误 | 「`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；59 号照「计数：」行），超时、内存撞顶等其余几栏另列、不并进三个数，不为 0 的整轮已判失败；」 | `research/scripts/mutate.sh` 里 `record_verdict` 第 3 个参数是这一条判不判整轮失败：`:514` `record_verdict "$row_index" crashed 0 "💥 …"`（0＝不判失败），`:534` nameblind 1、`:536` notred 1、`:473` timeout 1、`:481` `memory_fails=1`；收束时 `:580` `((verdict_fails)) && fail=1`、末尾 `:669` `exit $fail`。所以 `💥` 不为 0 时整轮照样可以退 0、收尾照样有「已还原，基线仍全绿」，「其余几栏不为 0 的整轮已判失败」对 💥 不成立。59 号那一侧对（`59-crates-mutation-replay.sh:406-440` 每一栏非空都打 ✗）。同一句在 `.claude/rules/mutation-sampling.md:82`（不在我负责的文件里），`experiment-runner.md:43` 只说「内存撞顶与超时（不为 0 的整轮已判失败）」，那两栏是对的 | 改成「`⏱`、`🧱`、`⚠️` 不为 0 的整轮已判失败；`💥` 不判失败，照样单列、逐条交主 agent」，rule 那一句同步 |
| 4 | .claude/agents/three-way-verifier.md:21、27、31 | 不一致 | 21：「没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。」 27：「…两样都没有的记「分不清：文件在腿开工之后被改过」，不记 ✗；」 31：「「核不动」与「分不清：文件在腿交回之后被改过」单列，不算进 ✓ 也不算进 ✗；」 | 同一类（不记 ✓ 也不记 ✗）三处写成三个不同的标签：「可能在腿交回之后」「在腿开工之后」「在腿交回之后」。第 6 步（31 行）点名单列的只有「交回之后」那一个；照字面，第 2 步新加的「腿开工之后」不在第 6 步的单列名单里，末尾计数（41 行「核了几处、✓ 几处、✗ 几处」）落哪一栏没写 | 三处统一成一个标签（例「分不清：文件在腿开工之后被改过」），第 6 步与产出一节的计数写明这一栏单独计 |
| 5 | .claude/agents/three-way-verifier.md:27（输入一节 16-22） | 不清楚 | 「文件不在快照清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」。」 | 「输入（主 agent 必须给）」一节（16-22 行）没有「腿开工时刻」这一项；快照只是 `sha256sum` 清单，不带时刻。另外没给快照的设计轮同时落在两条规则里：21 行说「对不上记分不清、不记 ✗」，27 行「文件不在快照清单里的…照实写」读下来是没改过就记 ✗，两条给相反的判定 | 输入一节加一项「腿开工时刻（UTC）」；27 行这一支限定为「给了快照、而这个文件不在清单里」，设计轮只走 21 行那一条 |
| 6 | .claude/agents/implementation-writer.md:28、46 | 不一致 | 28：「全量 `cargo test --all`、层 0 各流的快档与全量、其余门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；」 46：「没走三方对抗；层 0、QEMU、herd7 与 crates 变异表归 `crash-verifier`；没提交。」 | 这条分支把 54 号从崩溃验证员手里拿走了：`crash-verifier.md:24`「54 号不归你」，`stage-owners.tsv` 里 `54-layer0-replay.sh	gate-triage`，`agent-common.md:46`「`crash-verifier` 只跑 55、57、59 号那几道（54 号快档在 `gate.sh --staged` 里，全量由主 agent 跑）」。46 行（不在 diff 里，但被这次改动变假）仍把层 0 归崩溃验证员；28 行把层 0 全量也算进「crash-verifier 与整轮门禁」，全量其实归主 agent | 46 改成「层 0 全量归主 agent、快档在整轮门禁里；QEMU、herd7 与 crates 变异表归 `crash-verifier`」；28 行括注改成「主 agent 的层 0 全量、`crash-verifier` 与整轮门禁」 |
| 7 | .claude/agents/experiment-runner.md:35 | 不清楚 | 「② 那条决策还在 `.claude/decision-links-pending` 里时（那份清单只减不增，一条都没有时跳过这一步）不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。」 | 「这一步」有两种读法：② 这一小步，或整个 7b。今天清单是空的：`grep -vc '^#' .claude/decision-links-pending` 输出 `0`。读成整个 7b 的执行员会连 ①（引没引依据才写支撑 / 推翻）与 ③（重跑后逐行回看）一起跳过，75 号的双向检查就漏掉那几行 | 改成「清单里一条都没有时，② 不适用，① ③ 照做」 |
| 8 | .claude/agents/kb-scribe.md:20 | 不清楚 | 「翻成未定的判两句（31 号两把尺都要，缺一句就红）：改不改第一个事务的字节、动不动格式，决策文件标题行的状态（已定 / 半定（N 项未定））与 `.claude/kb/decisions.md` 状态列的分项计数改成什么。」 | ① `.claude/kb/decisions.md` 状态列由生成器写、不许手改：该文件原文「这一列是各正文两个小节的投影，由 `.claude/scripts/gen-decision-items.py` 生成，**不许手改**」；书记员第 3 步本来就跑 `21-decision-items-sync.sh --write` 重生成它。这一项放在「缺了门禁必红而你不能自己补」的清单里，会让书记员把它当规格去手写。② 标题行状态只列了「已定 / 半定」，漏了全部分项都未定时的「待定（N 项未定）」（`.claude/rules/format-evolution.md`「半定 / 待定的决策例外」）；③ 现有标题用汉字数：`grep -h '^## D' .claude/kb/decisions/*.md` 里有「半定（一项未定）」2 条、「半定（三项未定）」1 条，与「N 项」写法没说哪个为准 | 状态列那半句改成「`decisions.md` 状态列由第 3 步的 `21 --write` 重生成，规格只给预期值供回读核对，不手写」；标题状态写成「已定 / 半定（N 项未定）/ 待定（N 项未定）」，并写明 N 用汉字还是数字 |
| 9 | .claude/agents/experiment-designer.md:22 | 不清楚 | 「实验简称（或由你按问题起一个，交主 agent 认）；另在登记里定一个英文名（小写字母与数字，词之间下划线），源文件、变异表与 `[[bin]]` 名用它，中文简称只用在 kb 页。」 | 这是设计员要做的事，却写在「输入（主 agent 必须给）」一节；「做什么」各步与「登记的固定节名」表（38-50 行，`## 一、问题` 到 `## 十三、读过的文件与跑过的命令`）都没有放英文名的位置。执行员 `experiment-runner.md:27` 要「一律用跑前登记里定的英文名」，登记里找不到固定位置时两边各猜一处 | 挪进「做什么」第 2 步（占号之后），并在固定节名表里指定落在哪一节（例 `## 一、问题` 首行「英文名：…」） |
| 10 | .claude/agents/experiment-runner.md:27 | 不清楚 | 「（源文件、变异表、`[[bin]]` 的 `name` 一律用跑前登记里定的英文名：源文件与变异表写蛇形 `e<号>_<英文名>`，`name` 写连字符 `e<号>-<英文名>`；中文简称只用在 kb 页）」 | 这一句不分支，而入库装置那一支没有 `[[bin]]`：`grep -c '\[\[bin\]\]' crates/singlefs-harness/Cargo.toml` 输出 `0`，那里的 bin 按文件名自动发现、名字带下划线（`ls crates/singlefs-harness/src/bin/`：`e156_allocation_basis_counts.rs`、`e158_root_choice_repair.rs`）；执行员也写不了那份清单（write-guard 对 experiment-runner 写 `crates/singlefs-harness/Cargo.toml` 实测 rc=2）。句末括注「（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）」跟在入库装置那一大段括注之后，读起来也罩入库装置 | 写明「连字符的 `name` 只对 `research/e7-index-bench` 那一支；入库装置不写 `[[bin]]`，bin 名就是文件名 `e<号>_<英文名>`（`cargo test -p singlefs-harness --bin e<号>_<英文名>`）」 |
| 11 | .claude/hooks/heavy-test-guard.sh:39（旁证，不在我负责的文件里） | 不一致 | 「#   crash-verifier：层 0、55 号与 qemu-system-*、herd7、crates 变异整表（vm-bench.sh、全量测试、整轮门禁、E152 拒）；」 | 定义与共用约束已改成崩溃验证员不跑 54 号（`crash-verifier.md:24`、`agent-common.md:46`），钩子照旧放行：合成 JSON `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`、agent_type=crash-verifier，退出码 0。判决 `governance-defs-r1-main-verification.md`「四、交用户的」第 1 行已登记这一条，这里只确认它今天仍成立 | 照判决交用户；定义那一侧不用改 |

## 核过没问题的

轻阶段现跑（本机，2026-09-26 UTC）：`62-stage-owners.sh`「✓ 阶段归属表与门禁目录一致（75 个阶段，归 9 个 agent）」rc=0；`63-agent-write-scope.sh` ✓ rc=0；`10-kb-rot.sh`「✓ 治理文档里的门禁号、路径与「小节」都指得到」rc=0；`rules-lint.sh`（`RULES_LINT_DIR=.claude/rules`，外加全部 agents、agent-common、main-agent）「✓ 规则只写怎么做（扫了 25 份文件 1611 行…）」；`GATE_BASE=73ba4a4 bash .claude/gate.d/72-agent-def-adversarial-review.sh`「✓ 改过的定义与共用约束 16 份都有去向（新写的判决文件 1 份，另有 1 份按 .claude/agent-def-review-exempt 豁免，基准 73ba4a4）」rc=0。

| 文件 | 核过的类别（条数） |
|---|---|
| crash-verifier.md | 钩子放行：55、59（后台 `{ …; echo "exit=$?"; }` 写法）、74 共 3 条命令 rc=0；阶段归属表里它是 55、57、59、74、77、94，没有 54（1）；「在主工作区跑，工作区与暂存区不同就判红」与 54 号文件头第 28-29 行一致（1）；与 main-agent.md:59、implementation-workflow.md:55、agent-common.md:46 的「只跑 55、57、59」一致（3）；E142 简称与登记一致（1） |
| experiment-designer.md | `--exclude=experiments.md --exclude=experiments-history.md` 两个都是 `.claude/kb/` 下的文件，`--exclude-dir` 排不掉的说法属实（1）；`claim-experiment.sh` 用法与文件头一致（1） |
| experiment-runner.md | `replay.sh` 不经包装 rc=2、经 `run-with-memory-cap.sh 4G` rc=0（2）；写范围闸对 bin、`crates/mutations.tsv`、`research/e7-index-bench/Cargo.toml`、跑前登记放行（4）；共享 gate.sh「转发计时」阶段在（`gate.sh:282`），65 号已不存在（1）；`vm-harness.md`「计时不在转发输出的循环里打时间戳」在（第 147 行）（1）；mutation-sampling.md 那一节标题在（第 78 行）（1）；`crates/mutations.tsv` 第 1 行是六段表头（1）；33 号只扫 `research/e7-index-bench/src/bin/`，入库装置不写 research 变异表不会让它红（1） |
| gate-triage.md | `gate.sh --staged`、87 号带前缀 rc=0（2）；`refs/sop/gate-ok` 只在不带 `--staged` 全绿时写（共享 gate.sh:100、535）（1）；stage-inputs.tsv 有 54、55、59、87 行、没有 57 行（1） |
| implementation-writer.md | fmt、clippy、build、33 号、74 号 5 条命令 rc=0（5）；`CODE_DISCIPLINE_LINTS` 在 `.claude/singlefs-ai-sop/scripts/check.sh:22`，check.sh 里有 `cargo test --all`（:44），「不跑 check.sh 本身，它是重型」属实（2）；写范围闸对 `crates/mutations.tsv`、`litmus/` rc=0（2）；删掉的 `mutations-append.tsv` 与 `git apply --check` 与第 13 行「在主工作区改」相符，没带走仍成立的判据（1） |
| kb-scribe.md | 预检命令带 `AGENT_HOOK_DETECTIONS`：kb 路径 rc=0、crates 路径 rc=2，检出只写进给的文件（2）；这条预检命令过 heavy-test-guard 与 bash-command-detector rc=0（2）；`relabel-item.py` 经 `lib-item-ref-status.py:74` 扫 `crates/**/*.rs`，新加的写范围一句属实（1）；写入后钩子文件头「红的那几道把 ✗ 与 → 两类行交回给书记官（JSON 的 additionalContext）」、followups 表里有 49、75（2） |
| mutation-triage.md | 59 号计数行格式（:407-408）、「没跑到」记进没红（:349）（2）；七个行首符号与 `mutate.sh` 的 :473、:483、:504、:514、:534、:536、:538 对得上（1）；`mutate.sh` 收尾「计数：内存撞顶 … 超时 …」行（:647）（1）；新输入「59 号输出路径」与第 3 步取法一致（1） |
| sweep.md | `rewrite-moved-paths.py` 已不存在（ls 报 No such file），改指 path-moves.md「怎么做」（第 8 行）（1）；组号用 ASCII `-` 与 `stale-candidates.py` 解析（:361-374）一致（1） |
| three-way-attack.md | 三个小节标题在 three-way-inference.md 第 19、115、129 行（1）；副本里经包装的 `cargo test -p … --test …` rc=0、不经包装的 `cargo run` rc=2，与「经内存包装」一致（2） |
| three-way-defense.md | 小节标题在（第 129 行）（1）；当前 sha256 与豁免第 10 行相同，豁免有效（1） |
| three-way-local-attack.md | `ASK_LOCAL_TIMEOUT` 默认 900（ask-local.sh:14）（1）；退出 5 / 6 时不打正文、作废副本由 `save_void` 写（:63-119）（2）；`oov-check.py` 第二个参数是提示文件、生词表截前 300 字符（:241、:249）（2）；`>` 与 `>|` 两种后台写法过 bash-command-detector rc=0（4） |
| three-way-local-defense.md | 点名的三节在 local-attack.md 里都在（1） |
| three-way-materials.md | `_m2-code-r1-diff.md` 在提交 3cff909 被删、按共用约束取法取得到（1）；47 号跑 `quote-rust-items.py --selftest`（47 号第 24 行）（1） |
| three-way-verifier.md | implementation-workflow.md「代码轮派腿之前记一份开工快照」小节在（第 24 行）（1） |

`.claude/agent-def-review-exempt` 新加 8 行（第 6-13 行），逐版核哈希（`git show <提交>:<文件> \| sha256sum`，提交取 73ba4a4 与分支上 9 个提交）：

| 行 | 文件 | 哈希对得上的版本 | 今天 |
|---|---|---|---|
| 6 | agent-common.md 19f44f94… | 2c4c744、3e99fcd | 失效（现为 204c8e4e…） |
| 7 | experiment-runner.md 3ad79487… | 2c4c744 至 78c9312 | 失效（现为 53b01fab…） |
| 8 | sweep.md c0d517fe… | 2c4c744 至 78c9312 | 失效（现为 f3a7952a…） |
| 9 | three-way-attack.md 0a4f4a56… | 2c4c744 至 78c9312 | 失效（现为 333f0e5c…） |
| 10 | three-way-defense.md 6023f802… | 2c4c744 至 71f0cbc | **有效** |
| 11 | three-way-materials.md 5abe013c… | 2c4c744 至 78c9312 | 失效（现为 2b6000de…） |
| 12 | main-agent.md fc197db3… | 3448172 至 78c9312 | 失效（现为 de1d9c40…） |
| 13 | agent-common.md beb5d43d… | 3448172 至 78c9312 | 失效（现为 204c8e4e…） |

每行的哈希都对得上它豁免的那一版（登记时成立）。失效的 7 行对应的文件在 71f0cbc 又改过，72 号靠 `research/prompts/governance-defs-r1-main-verification.md`（点名 15 份）放行：以 73ba4a4、2c4c744、3e99fcd、3448172、802fcc1 为基准各跑一次都绿，以 73ba4a4 为基准时 16 份里 1 份（three-way-defense）走豁免。第 4、5 行（分支之前加的）哈希在 73ba4a4..HEAD 的任何一版都对不上，是分支之前就失效的旧行，本分支没动它们。

## 没做什么

- 没跑重型测试，也没跑 gate.sh 整轮；第 1、2 条里「55、59 在同一次提交里跑两遍」是按脚本逻辑推的，没有真跑一趟提交流程量过。
- 没核 `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/*.md` 本身的改动（不在我负责的范围），只在与定义说同一件事时引用它们；第 1、3、11 条顺带点到的那几处留给负责那些文件的审查员。
- 12 号现跑红在 `research/prompts/m2-lastflag-implementer-report.md:130`（角标 P′），不在我负责的文件里，没判归属。
- 没核 `oov-check.py`、`corruption-check.py` 判得对不对，只核了参数与输出形态；没有真调本地模型（网关不在）。
- 没核 `write-guard.sh` 对相对路径以外写法的判定、`kb-scribe-followup.sh` 真跑一次写入后的回传（只读了文件头与表）。
- 本草稿目录里 `diffs/`、`probe/`、`tmp.*` 不是我建的，没碰。
