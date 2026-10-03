# 第 4 批定义改动的 diff（相对 HEAD，只留点名形式化验证一道的 hunk）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index 97a196cc..01773cb0 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -49,7 +49,7 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：checker 档包 `crates/singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而范围含它、直接执行它的测试二进制）与门禁 `checker-tier-layer0-replay`、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、全部实验复跑（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格）、E152 装置，只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build（逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 这四道，`gate-triage` 只跑 `gate.sh` 整轮与 experiment-replay 格，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：checker 档包 `crates/singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而范围含它、直接执行它的测试二进制）与门禁 `checker-tier-layer0-replay`、QEMU、herd7、crates 变异整表、形式化验证（门禁 checker-tier-formal-verification）、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、全部实验复跑（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格）、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算），只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build（逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 这五道，`gate-triage` 只跑 `gate.sh` 整轮与 experiment-replay 格，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，checker 档包 `singlefs-checker-tier` 的测试二进制也不许。
 - 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 checker-tier-crates-mutation-replay 在里面逐条套了，门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格与门禁 harness-model-differential-and-scenarios 在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：research-unit-tests 格 `GATE_RESEARCH_BUILD_MEMORY_MAX`，harness-model-differential-and-scenarios `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index 4c161255..251c4130 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -1,6 +1,6 @@
 ---
 name: crash-verifier
-description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
+description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表、Verus 形式化验证这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
 tools: Read, Bash
 model: sonnet
 effort: high
@@ -11,7 +11,7 @@ omitClaudeMd: true
 
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
-只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「checker 档只在提交时跑，harness 随时跑」里提交时跑的这四道门禁（文件是 `.claude/gate.d/<名>.sh`）：`checker-tier-layer0-replay`（默认快档，`--full` 只在用户要求或夜间）、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay。
+只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「checker 档只在提交时跑，harness 随时跑」里提交时跑的这五道门禁（文件是 `.claude/gate.d/<名>.sh`）：`checker-tier-layer0-replay`（默认快档，`--full` 只在用户要求或夜间）、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification。
 开工先读：`.claude/skills/crash-test/SKILL.md`「判读纪律」；`.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」「文件系统特有的反推缺口」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」。
 
 ## 输入（主 agent 必须给）
@@ -19,15 +19,16 @@ omitClaudeMd: true
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：`checker-tier-layer0-replay` 与 checker-tier-lkmm 的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；checker-tier-qemu-device-streams 的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/checker-tier-qemu-device-streams.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
+- checker-tier-formal-verification 的两个内存上限（第 1b 步用），各一个带单位的上限：每个 Verus 进程一个（空真与必红世界几处同时验，每处各占一份），E165 装置一个；各写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里 Verus 那条命令、装置那条命令经包装跑出的峰值）还是推的。
 - `checker-tier-layer0-replay` 跑哪一档：默认快档（`cargo test --release -p singlefs-checker-tier --lib --tests` 加逐条核全绿标记，`.claude/rules/verification.md`），派发提示什么都不写就是它；要跑全量的写明「`checker-tier-layer0-replay` 带 --full」与在哪棵 worktree 里跑（HEAD + 暂存区的 worktree 路径，或写明由你照 `checker-tier-layer0-replay` 快档出路里那三行建）、命令前缀 `SINGLEFS_HEAVY_TESTS=user-request`；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
 - 报告路径与草稿目录。
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 `checker-tier-layer0-replay`、或有 `checker-tier-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 `checker-tier-layer0-replay`，等它结束。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，checker-tier-lkmm 是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 查 `crates`，checker-tier-qemu-device-streams 按名字读的 `research/results/` 产物这里不查，由整轮门禁里 checker-tier-research-build-and-replay 的 experiment-replay 格兜；checker-tier-lkmm 查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 checker-tier-lkmm 只跑静态那几层；要没有输出，别处的未跟踪文件不挡；checker-tier-lkmm 读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
-   1b. `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；checker-tier-crates-mutation-replay 在里面逐条套了，照第 2 步直接跑，外面不再包一层。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 `checker-tier-layer0-replay`、或有 `checker-tier-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 `checker-tier-layer0-replay`，等它结束。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay、checker-tier-formal-verification 是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，checker-tier-lkmm 是 `litmus crates .claude/scripts/lkmm.sh`，四道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 查 `crates`，checker-tier-qemu-device-streams 按名字读的 `research/results/` 产物这里不查，由整轮门禁里 checker-tier-research-build-and-replay 的 experiment-replay 格兜；checker-tier-formal-verification 查 `crates` 与 `research/results/e165-*-red-examples.tsv`；checker-tier-lkmm 查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 checker-tier-lkmm 只跑静态那几层；要没有输出，别处的未跟踪文件不挡；checker-tier-lkmm 读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
+   1b. `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；checker-tier-crates-mutation-replay 在里面逐条套了，照第 2 步直接跑，外面不再包一层。checker-tier-formal-verification 也在里面套了（每次 Verus、E165 装置各经包装），照第 2 步直接跑、外面不再包一层，输入给的两个上限设进阶段读的变量，写在 `nice` 前面：`GATE_VERUS_MEMORY_MAX=<Verus 的上限> GATE_E165_PINNED_MEMORY_MAX=<装置的上限>`。
    1c. `checker-tier-layer0-replay` `--full` 开跑前在那棵 worktree 里跑 `bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments <worktree>` 与 `awk -F'\t' '$1 ~ /^crash-case:/ && $3 ~ /shard=across-machines/ { print $1 }' <worktree>/.claude/gate.d/stage-inputs.tsv`，两样原样抄进报告：前一条退 0 是双机分片开着，抄它打的 `PEER_SSH_HOST=` 与 `PEER_MEMORY_CAP=` 两行（第二台那一片的内存上限：驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 用它起那一片，本机那一片与 merge 由第 1b 步的包装管），退 1 是关着、照单机跑，抄它那一句原因；后一条列出的是开着时两台各跑一片的用例。驱动只由 `checker-tier-layer0-replay` `--full` 调（`--merged-log`），不单独跑它。
-2. 只在提交时（或主 agent 转达用户要求时）跑这四道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay，与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑别的门禁、`gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。checker-tier-qemu-device-streams、checker-tier-lkmm 与 `checker-tier-layer0-replay` 并行起（各自后台、各自内存上限，照第 1b 步），checker-tier-crates-mutation-replay 等 `checker-tier-layer0-replay` 跑完再起（整表复跑与层 0 全量争 CPU）；四道的命令都带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`；`checker-tier-layer0-replay` 默认跑快档（`bash .claude/gate.d/checker-tier-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「`checker-tier-layer0-replay` 带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑全量（`bash <worktree>/.claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑这五道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification，与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑别的门禁、`gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-layer0-replay` 并行起（各自后台、各自内存上限，照第 1b 步），checker-tier-crates-mutation-replay 等 `checker-tier-layer0-replay` 跑完再起（整表复跑与层 0 全量争 CPU）；五道的命令都带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`；`checker-tier-layer0-replay` 默认跑快档（`bash .claude/gate.d/checker-tier-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「`checker-tier-layer0-replay` 带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑全量（`bash <worktree>/.claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（新池新建文件的干跑） 产物或闭式比对的那几行；`checker-tier-layer0-replay` `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行，写着第二台那一片经 `run-with-memory-cap.sh <上限>` 起）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、`checker-tier-layer0-replay` 判 merge 那一趟的日志绿；第二台那一片退 250–254 是内存包装自己的结局，照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办，那一条用例这一趟的判定不算。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 03d6a569..5e904cb8 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
    1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（checker-tier-research-build-and-replay 的 experiment-replay 格经 `research/scripts/replay.sh` 逐条套了，同一道的 research-unit-tests 格与 harness-model-differential-and-scenarios 在阶段里面经包装）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、harness-model-differential-and-scenarios 与 experiment-replay 格才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 这几道重阶段在 `gate.sh` 里照各自的复用判定走（`checker-tier-layer0-replay` 跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀，不带的 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查一类）。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、harness-model-differential-and-scenarios 与 experiment-replay 格才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 这几道重阶段在 `gate.sh` 里照各自的复用判定走（`checker-tier-layer0-replay` 跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀，不带的 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查一类）。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（`checker-tier-layer0-replay` 的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 `checker-tier-layer0-replay` 脚本本身（`.claude/gate.d/checker-tier-layer0-replay.sh`；全绿标记的键还含它与 `cargo -V`、`rustc -V`） ⇒「这一轮」，下一步原样抄 `checker-tier-layer0-replay` 出路里的三行命令，一条都没碰 ⇒「不是这一轮」；✗ 行写着内存包装退 25x 的（research-unit-tests 格与 harness-model-differential-and-scenarios 的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）不按它点名的文件判：250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
diff --git a/.claude/agents/investigator.md b/.claude/agents/investigator.md
index e3eb3e8d..2eee8af3 100644
--- a/.claude/agents/investigator.md
+++ b/.claude/agents/investigator.md
@@ -29,7 +29,7 @@ required-inputs: 现象, 草稿目录, 报告
 2. 先原样复现：用输入给的命令在仓副本里（`rsync -a --exclude target --exclude .git`）跑一次，贴命令与原样输出。复现不出就写复现不出，列出与原现场差在哪（提交、环境变量、种子、线程数），不往下猜。
 3. 二分到文件:行：缩小输入、改坏或加打印都只在副本里做；每一步贴命令与输出，记下排除了哪几种解释、各凭哪条观测。
 4. 给最小复现：一条命令加它的原样输出；再给「什么现象会推翻这个定位」，并在副本里造一次那个现象看它确实会出现。
-5. 要跑的东西是重型测试（名字带 layer0 的测试二进制、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 的 experiment-replay 格、全量 cargo test）的，不跑，写进报告交主 agent 去问用户。
+5. 要跑的东西是重型测试（名字带 layer0 的测试二进制、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 的 experiment-replay 格、全量 cargo test）的，不跑，写进报告交主 agent 去问用户。
 6. 上下文过 400k：停在最近一个能交接的点，报告写查到哪一步、排除了什么、还差什么，交回。
 7. 派发提示写了「查到就修」的：定位到的改法在条款里写明了的，在主工作区只改「要动的 crates 文件」那一行列的文件，照 `.claude/agents/implementation-writer.md` 第 2、3、3a、4 步写实现、补测试、证红（`prove-red.sh`）、变异行追加进 `crates/mutations.tsv`、末尾一条命令跑 fmt、clippy、build 与动到的测试二进制；条款没写、要做设计判断的，停在那一处写进报告，不自己定。报告附 `git diff --stat -- crates`。
 
diff --git a/.claude/agents/tooling-writer.md b/.claude/agents/tooling-writer.md
index 1b0f7306..febf72de 100644
--- a/.claude/agents/tooling-writer.md
+++ b/.claude/agents/tooling-writer.md
@@ -46,4 +46,4 @@ required-inputs: 草稿目录, 报告, 出口, 要改的文件
 
 ## 没做什么（固定会有的）
 
-- 没跑重型测试（`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 本身、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
+- 没跑重型测试（`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index 82cdfd45..d83ece9f 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -39,6 +39,13 @@
 #   全部实验复跑  .claude/gate.d/checker-tier-research-build-and-replay.sh，只跑它的单测那一格（--check research-unit-tests）的除外：那一格落到下一类
 #   整轮门禁      gate.sh、research/scripts/gate-staged.sh
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
+#   广谱变异      research/scripts/broad-mutation.py（python 脚本不读正文，按名字与位置、解到的真实路径、文件开头的签名行认：`python3 -W / -X <值> 脚本`、
+#                 `python3 -m broad-mutation`、指到它的符号链接、带签名行的改名拷贝、`python3 -c` 的代码里点它的都认）。凡要起 cargo 或改库目录的都算：
+#                 整趟、--shard / --from / --package、--item 点名（点几条、带不带 --skip-stage-two 都算）、--stage-two-only、--sync-tree（换全局的版本指针）、
+#                 --import-results；只有带 --list-items / --summary / --help 的不算。子 agent 的放行表里没有这一类：算的那些只有主 agent 带前缀能跑，
+#                 不算的那几样谁都能跑。罩不住的写法在 lib_heavy_tests.py 文件头「广谱变异的逐条驱动」那一段
+#   Verus         .claude/gate.d/checker-tier-formal-verification.sh（逐格起 Verus、空真与必红世界逐处起、编装置重放钉住的红例）。子 agent 里只有 crash-verifier 的放行表有这一类：
+#                 提交时由它带前缀单独跑，主 agent 带前缀也能单独跑，gate-triage 起的整轮 gate.sh 里也有它；直接起的 verus 与 cargo verus 不算（写证明的实现员要用，判法在 lib_heavy_tests.py）
 #   门禁阶段按文件名认，不按编号认；带 --list、--list-items（只打格名表、逐项点名的清单）的哪一道都不算；带 --write（生成）的不算。
 # 提交时才跑的检查（不是重型，放行照同一张表；用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的 … 提示不执行 根本不靠谱」）：
 #   上面几类之外的 .claude/gate.d/ 阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、gate-overlap.py
@@ -50,15 +57,22 @@
 #   判定在 lib_heavy_tests.py 的 classify（COMMIT_CHECK_KINDS）；弄坏开关 HEAVY_TEST_GUARD_IGNORE_COMMIT_CHECKS=1 时这一类一律放行，自检必须判红。
 # 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
 #   主 agent（输入里没有 agent_type）：上面每一类，连同提交时才跑的检查；
-#   crash-verifier：checker 档（54 号与 checker 档包的测试）、QEMU 那一道与 qemu-system-*、herd7、crates 变异整表、提交时才跑的检查
+#   crash-verifier：checker 档（54 号与 checker 档包的测试）、QEMU 那一道与 qemu-system-*、herd7、crates 变异整表、Verus 那一道（形式化验证）、提交时才跑的检查
 #   （vm-bench.sh、全量测试、整轮门禁、全部实验复跑、E152 拒）；
-#   gate-triage：整轮门禁（gate.sh、gate-staged.sh）、全部实验复跑那一道、提交时才跑的检查（gate.sh 里 54 号与 QEMU、herd7、crates 变异表那三道
+#   gate-triage：整轮门禁（gate.sh、gate-staged.sh）、全部实验复跑那一道、提交时才跑的检查（gate.sh 里 54 号与 QEMU、herd7、crates 变异表、Verus 那四道
 #   靠「输入没变就复用上一次全绿判定」，直接调它们拒）；
 #   其余子 agent：一律拒，带不带前缀都拒。
 # 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
 # 往 `bash -c '…'`、`capped.sh N …`、`nice`、`timeout` 这类包装里面传。git 的 pre-commit hook 由 git 起，不经这道闸。
 # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、
 #   `perf stat|record|trace`（lib_heavy_tests.py 的 LAUNCHER_OPTIONS_WITH_VALUE）剥掉之后，里面那条当一段命令文本接着判：前缀、内存包装、脚本照认。
+#   research/scripts/detached-run.sh 的 `start … -- <命令>`（把命令挂到用户级 systemd 下脱离会话跑，调用方的 SINGLEFS_HEAVY_TESTS 带进去）同样剥掉，
+#   不读它的正文：`--` 之后那条照这张表判，外面写的前缀与 `--` 之后写的前缀都认，谁能跑什么不因为经它起而变（lib_heavy_tests.py 的 command_under_detached_run）。
+#   它的参数先照它开头的 preflight 摘掉每一个 `--force`（连 `--` 之后的），判的就是单元里真跑的那条（lib_shell_words.py 的 detached_run_arguments_after_preflight）。
+#   认它按内容，不只按名字（lib_shell_words.py 的 detached_run_script）：名字叫 detached-run.sh、指到它的符号链接、开头带签名行
+#   （行首 `# detached-run-script-signature:`）的改名拷贝，bash / sh / source / . 起的、直接执行的都认；同一条命令里前面 cp 拷来、heredoc 写出的，
+#   按写出的内容找签名行（written_detached_run）。认签名行时另读命令词那份文件的开头 4 KiB（不经 JudgingState、不记进读过的文件）。
+#   罩不住的写法在 research/scripts/detached-run.sh 文件头「谁能用」那一段。
 # 写进脚本文件再执行的，读脚本正文、逐条照同一张表判：命令位置上是 `bash|sh 文件`（不带 -c）、`./x.sh` 或 `路径/x.sh`、`source 文件` / `. 文件`，
 #   经 capped.sh N、nice、timeout、env、前缀变量包一层的同样认；脚本里再起脚本的递归读，读到第 MAXIMUM_SCRIPT_DEPTH（6）层为止，再往里判「看不全」。
 #   外层命令带的前缀与 export 往脚本里传，脚本里某一行自己写的前缀同样认；谁能跑什么照上面那张表，不因为写在脚本里而变。
@@ -109,7 +123,9 @@
 #   heavy-test-guard.sh             # 从 stdin 读 hook 的 JSON；拒绝退出 2，放行退出 0
 #   heavy-test-guard.sh --selftest  # 在临时工作区里走一遍拒绝与放行（连同 lib_heavy_tests.py 的自检）；HEAVY_TEST_GUARD_DISABLE_CHECK=1、
 #                                   # HEAVY_TEST_GUARD_IGNORE_WRITTEN_SCRIPTS=1（不认同一条命令里写出的脚本）、HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1（不判经没经内存包装）、
-#                                   # HEAVY_TEST_GUARD_IGNORE_COMMIT_CHECKS=1（提交时才跑的检查一律放行）时自检必须判红；
+#                                   # HEAVY_TEST_GUARD_IGNORE_COMMIT_CHECKS=1（提交时才跑的检查一律放行）、
+#                                   # HEAVY_TEST_GUARD_IGNORE_CRASH_VERIFIER_VERUS=1（crash-verifier 的放行集合里不带 Verus 那一道）、
+#                                   # HEAVY_TEST_GUARD_IGNORE_HEREDOC_DETACHED_RUN=1（同一条命令里 heredoc 写出的 detached-run.sh 拷贝不认）时自检必须判红；
 #                                   # 切词那一份的 LIB_SHELL_WORDS_BREAK=arguments-as-executed（`python3 -` 后面的词、`bash -n` 的目标当被执行的脚本）同样必须判红；
 #                                   # 判定那一半的弄坏开关（lib_heavy_tests.py 的 LIB_HEAVY_TESTS_BREAK、admission.py 的 ADMISSION_BREAK，写法在两份的文件头）设了哪一个也必须判红
 set -uo pipefail
@@ -156,9 +172,12 @@ SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有
 COMMIT_CHECK_KINDS = {"commit-check-stage", "commit-check-lint", "commit-check-selftest", "commit-check-structure"}
 AGENT_KINDS = {
     "crash-verifier": {"checker-tier-cargo", "checker-tier-binary", "layer0-stage", "qemu-stage", "qemu-system",
-                       "herd7-stage", "lkmm", "herd7", "crates-mutation-stage", "crates-mutation-mutate"} | COMMIT_CHECK_KINDS,
+                       "herd7-stage", "lkmm", "herd7", "crates-mutation-stage", "crates-mutation-mutate", "verus-stage"} | COMMIT_CHECK_KINDS,
     "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"} | COMMIT_CHECK_KINDS,
 }
+# 弄坏开关：crash-verifier 的放行集合里不带 Verus 那一道（只给自检证明「崩溃验证员带前缀跑 Verus 那一道」那一格会红）
+if os.environ.get("HEAVY_TEST_GUARD_IGNORE_CRASH_VERIFIER_VERUS") == "1":
+    AGENT_KINDS["crash-verifier"] = AGENT_KINDS["crash-verifier"] - {"verus-stage"}
 
 class HeavyUse(NamedTuple):
     test: object           # lib_heavy_tests.HeavyTest
@@ -556,12 +597,14 @@ def heavy_uses(text, directory, environment=None, frames_above=(), script=None,
     return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)
 
 POLICY = ("→ 规矩：重型测试就是 checker 档那一批（.claude/rules/verification.md）：checker 档包 singlefs-checker-tier 的测试（按包判，挑哪个目标都算）与 54 号、QEMU、herd7、crates 变异整表、"
-          "全量测试、整轮门禁、全部实验复跑、E152 装置，只在提交代码时跑一次、或用户要求时跑；"
+          "Verus 那一道门禁（checker-tier-formal-verification.sh；直接起 verus 验证明不算）、"
+          "全量测试、整轮门禁、全部实验复跑、E152 装置、广谱变异（research/scripts/broad-mutation.py 起 cargo 或改库目录的每一种用法，只有 --list-items / --summary 不算），"
+          "只在提交代码时跑一次、或用户要求时跑；"
           "子 agent 一律不跑，只跑 harness 档——自己动到的 singlefs-harness 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
           "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
-          "→ 各自那一份：crash-verifier 只跑 54 号与 checker-tier-qemu-device-streams.sh、checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh 三道，"
+          "→ 各自那一份：crash-verifier 只跑 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 五道，"
           "与它们底下的 checker 档包 singlefs-checker-tier 的测试（快档与登记的全量用例）、qemu-system、lkmm.sh / herd7、crates 变异整表；"
-          "gate-triage 只跑 `gate.sh` 整轮与 checker-tier-research-build-and-replay.sh（54 号与另外三道靠「输入没变就复用上一次全绿判定」）；"
+          "gate-triage 只跑 `gate.sh` 整轮与 checker-tier-research-build-and-replay.sh（54 号与另外四道靠「输入没变就复用上一次全绿判定」）；"
           "两个都要带那个前缀，都不跑全量 `cargo test`。\n"
           "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
           "子 agent 在交回里写明要跑什么、为什么，交主 agent 去问（派发提示里点名要你跑的也一样，写明被这道闸拒了）。")
@@ -758,6 +801,15 @@ def selftest(hook_dir):
             ("实现员 vm-bench.sh --selftest", writer, "bash research/scripts/vm-bench.sh --selftest", 2),
             ("实现员跑 herd7", writer, "herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
             ("实现员跑 lkmm.sh", writer, "bash .claude/scripts/lkmm.sh", 2),
+            ("实现员跑 Verus 那一道门禁", writer, "bash .claude/gate.d/checker-tier-formal-verification.sh --check verus-proof", 2),
+            ("门禁分诊带前缀单独调 Verus 那一道", triage, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
+            ("主 agent 不带前缀跑 Verus 那一道", None, "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
+            ("主 agent 带前缀跑 Verus 那一道", None, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 0),
+            # crash-verifier 提交时跑 Verus 那一道（HEAVY_TEST_GUARD_IGNORE_CRASH_VERIFIER_VERUS=1 下第一格红）
+            ("崩溃验证员带前缀跑 Verus 那一道", crash, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 0),
+            ("崩溃验证员不带前缀跑 Verus 那一道", crash, "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
+            ("实现员带前缀跑 Verus 那一道照拒", writer, commit + "bash .claude/gate.d/checker-tier-formal-verification.sh", 2),
+            ("实现员直接起 verus 验自己写的证明：不算重型", writer, "verus --crate-type=lib --no-cheating crates/singlefs-core/src/lib.rs", 0),
             ("实现员跑 gate-staged.sh", writer, "bash research/scripts/gate-staged.sh", 2),
             ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\ncargo test --all\nEOF", 2),
             ("实现员直接执行 checker 档的测试二进制", writer, tier_binary + " --test-threads 4", 2),
diff --git a/.claude/hooks/runner-dispatch-guard.sh b/.claude/hooks/runner-dispatch-guard.sh
index 658c28de..3a1fa88b 100755
--- a/.claude/hooks/runner-dispatch-guard.sh
+++ b/.claude/hooks/runner-dispatch-guard.sh
@@ -575,8 +575,8 @@ def decide(hook_input, project_root, hook_dir=None):
     if (subagent_type not in HEAVY_TEST_OWNERS and os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_HEAVY_TEST") != "1"
             and not HEAVY_TEST_LINE.search(prompt)):
         return 2, (f"✗ 派 {subagent_type} 的提示里没有「重型测试：不跑」一行。\n"
-                   "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑，"
-                   "只由 crash-verifier（checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay）与 gate-triage（gate.sh 整轮与 checker-tier-research-build-and-replay）跑；执行时 .claude/hooks/heavy-test-guard.sh 也拒。\n"
+                   "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、形式化验证（Verus）、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时、或用户要求时跑，"
+                   "只由 crash-verifier（checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification）与 gate-triage（gate.sh 整轮与 checker-tier-research-build-and-replay）跑；执行时 .claude/hooks/heavy-test-guard.sh 也拒。\n"
                    "→ 怎么办：提示里单独写一行「重型测试：不跑」；提交时的重阶段派 crash-verifier、整轮门禁派 gate-triage；提交之外确实要跑，先弹窗问用户，"
                    "用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。")
     if (subagent_type in GENERAL_PURPOSE_TYPES and valve_reason(prompt, "通用已判") is None
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index 30b3ede6..bbefa450 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -49,7 +49,7 @@
 
 同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数 = 此刻在跑、任务里带重活的 agent 数（三方腿编模型、主 agent 自己起的长命令各算一个，别的会话的不算）；重活数变了，给在跑的 agent 发消息改 N。
 
-派崩溃验证员时给 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 各自的内存上限，每道一个带单位的上限（例 16G），照 `.claude/agents/crash-verifier.md`「输入」一节给：`checker-tier-layer0-replay` 与 checker-tier-lkmm 写明是量过的还是推的，checker-tier-qemu-device-streams 的不小于同时起的虚机数乘每台的内存。门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格与门禁 harness-model-differential-and-scenarios 在阶段里面经内存包装，要换它们的上限就在派发提示里给一个带单位的上限（例 16G），没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。
+派崩溃验证员时给 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 各自的内存上限，每道一个带单位的上限（例 16G），照 `.claude/agents/crash-verifier.md`「输入」一节给：`checker-tier-layer0-replay` 与 checker-tier-lkmm 写明是量过的还是推的，checker-tier-qemu-device-streams 的不小于同时起的虚机数乘每台的内存；checker-tier-formal-verification 给两个（每个 Verus 进程一个、E165 装置一个），各写明是量过的还是推的。门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格与门禁 harness-model-differential-and-scenarios 在阶段里面经内存包装，要换它们的上限就在派发提示里给一个带单位的上限（例 16G），没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。
 
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
@@ -58,12 +58,12 @@
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
 | 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `python3 research/scripts/three-way-materials.py <轮>`（清单默认全抄、索引表与历史节不抄，主 agent 要标例外就改清单再跑 `checklist-specs.py`；正文引到 30 份以上文件、清单要人挑的轮才派 `three-way-materials`）→ 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在有前一轮判决可复核时派）、`three-way-attack`（不派本地模型的腿）→ 全部交齐后，主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」：逐条核腿报告里的原文引用、产物行与复跑命令，代码轮对着开工快照核腿引的行；不派核查员）；如果第一轮结论不一致无法收敛，开第二轮。如果前两轮结论不一致无法收敛，开第三轮。第三轮为最终轮，论证前两轮的结论和判定过程。第三轮结束后不再新开，交给主agent判断 |
-| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑 checker 档：`checker-tier-layer0-replay` 快档、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；层 0 全量只在用户要求或夜间跑，平时不跑） |
+| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑 checker 档：`checker-tier-layer0-replay` 快档、QEMU、herd7、crates 变异表、Verus 形式化验证（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；层 0 全量只在用户要求或夜间跑，平时不跑） |
 | 一批实现员交回到齐、合入之前 | 主 agent 跑 `bash research/scripts/merge-and-verify.sh [--memory 上限] [--threads N] <补丁目录…>`：打全部补丁（`apply-writer-patch.py`，核不过整批停）、编一次、跑 harness 档四个包的全部测试二进制、fmt、clippy，红的逐条列出并给只重跑那几条的命令；红了才派一个 `implementation-writer` 照各份报告的算法改钉值（只证合入时改过的变异行）→ 主 agent 审 diff → 上面那一行的三方 |
-| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑 checker 档（`checker-tier-layer0-replay` 快档、QEMU、herd7、crates 变异表） |
+| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑 checker 档（`checker-tier-layer0-replay` 快档、QEMU、herd7、crates 变异表、Verus 形式化验证） |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
 | 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再按判定种类抽查：每一种判定各抽不少于 5 条（不足 5 条的全看），对着载体今天的原文核；抽中判错的那一段整段重派，重派交回再抽一遍，两次都干净才收 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见 `.claude/gate.d/doc-process-records.sh` 文件头 knowledge-sync 格那一段，那一格判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑这四道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`checker-tier-layer0-replay` 跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），其余三道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm 与 `checker-tier-layer0-replay` 并行，checker-tier-crates-mutation-replay 与 `checker-tier-layer0-replay` 串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这四道在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑这五道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`checker-tier-layer0-replay` 跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），其余四道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-layer0-replay` 并行，checker-tier-crates-mutation-replay 与 `checker-tier-layer0-replay` 串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这五道在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（写回员：给它判决与条目，它自己起草规格、跑 `research/scripts/kb-spec-check.py` 到绿、再写；派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧；条目少、主 agent 自己写得清的，主 agent 自己写规格、跑 `research/scripts/replace-batch.py` 与 `bash .claude/gate.d/doc-decisions.sh --write`；按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个写回员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 doc-experiments 的 decision-links 格那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、门禁 code-source-discipline 的 mutation-tables 格红（变异表锚点腐化）→ `research/mutations/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
 | 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 那一格） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index b71e1501..ba881bdd 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -29,7 +29,7 @@
 
 ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判
 
-改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 都登记着。
+改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 都登记着。
 
 ⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：
 
@@ -50,11 +50,11 @@
 | 场合 | 跑不跑 |
 |---|---|
 | 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；checker 档一样都不跑 |
-| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 `checker-tier-layer0-replay`（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 `checker-tier-layer0-replay`（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
 | 用户要求，或夜间 | checker 档全量：`checker-tier-layer0-replay` `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
 | 其余任何时候 | checker 档不跑；子 agent 只跑 harness |
 
-**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
+**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算）。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
 
 herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：
 
@@ -68,10 +68,7 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 
 ## 测试与崩溃检测优先多线程
 
-- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
-- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
-- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
-- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
-- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。`checker-tier-layer0-replay` `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 `checker-tier-layer0-replay` 的那一层内存包装管。驱动只由 `checker-tier-layer0-replay` `--full` 调。门禁 `checker-tier-crates-mutation-replay.sh` 只在本地多机配置写了 `ENABLE_ACROSS_MACHINES=1` 时按行分两台（判法同一份：`research/scripts/layer0-shard-configuration-check.sh` 退 0 双机开、退 3 双机关、退 1 判不过），不认环境变量开，设了 `SINGLEFS_GATE_FULL=1` 或 `GATE_MUTATION_START_OVER=1` 时也不分：开着时先交 `research/scripts/mutation-shard-run.sh`，本机没有作数按条记录的行两堆均分、两台各判一堆，第二台的按条记录经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后本机整张判一遍（命中记录的复用、缺的现跑），全绿标记由本机写。判法退 3 时驱动报一行「双机：关」、什么都不分。这个驱动只由这一道门禁调；它与这道门禁的整套重设计归会话「里程碑3 放量 GPU加速」。
+- **写法**：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么，照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「编译与算法都要快」。本仓用 `std::thread::scope`，不为这个加依赖；每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）；线程数的环境变量没设就取 `std::thread::available_parallelism`。
+- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。`checker-tier-layer0-replay` `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 `checker-tier-layer0-replay` 的那一层内存包装管。驱动只由 `checker-tier-layer0-replay` `--full` 调。门禁 `checker-tier-crates-mutation-replay.sh` 只在本地多机配置写了 `ENABLE_ACROSS_MACHINES=1` 时按行分两台（判法同一份：`research/scripts/layer0-shard-configuration-check.sh` 退 0 双机开、退 3 双机关、退 1 判不过），不认环境变量开，设了 `SINGLEFS_GATE_FULL=1` 或 `GATE_MUTATION_START_OVER=1` 时也不分：开着时先交 `research/scripts/mutation-shard-run.sh`，本机没有作数按条记录的行两堆均分、两台各判一堆，第二台的按条记录经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后本机整张判一遍（命中记录的复用、缺的现跑），全绿标记由本机写。判法退 3 时驱动报一行「双机：关」、什么都不分。这个驱动只由这一道门禁调；它与这道门禁的整套重设计归别的会话。
 
-**门禁管哪一半**：崩溃点重放由门禁 `checker-tier-layer0-replay` 判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑 `bash <它的根>/.claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 `checker-tier-layer0-replay` 判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑 `bash <它的根>/.claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。形式化验证由门禁 checker-tier-formal-verification 判：四格用 Verus 验 `crates/singlefs-replay-prefix`（版本钉住、证明、空真、必红世界），两格判 E165 红例钉表（钉全、在今天的实现上逐行重放）；提交时由 `crash-verifier` 跑，按 `.claude/gate.d/stage-inputs.tsv` 它那一行的输入复用。
diff --git a/.claude/rules/verification.md b/.claude/rules/verification.md
index 3e97dc45..fa703248 100644
--- a/.claude/rules/verification.md
+++ b/.claude/rules/verification.md
@@ -10,21 +10,21 @@
 | 名字 | 包 | 装什么 | 什么时候跑 |
 |---|---|---|---|
 | **harness 档** | `crates/singlefs-harness` | 单元测试与集成测试，加上它们用的脚手架库：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景 | 改了代码随时跑 |
-| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`；`verdict_store` 挂特性 `verdict-store`，`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例；连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
+| **checker 档** | `crates/singlefs-checker-tier` | 崩溃态枚举引擎与记录核对器（`crash`）、断点续跑与双机分片（`layer0_progress`）、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对与运行模式、崩溃放量（崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`；`verdict_store` 挂特性 `verdict-store`，`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`）、实验与真设备装置二进制（`src/bin/`），以及它们的全部用例；连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）、形式化验证（checker-tier-formal-verification：Verus 验 `crates/singlefs-replay-prefix`，E165 红例钉表两格） | 默认只在提交时跑；想单独跑带 `SINGLEFS_HEAVY_TESTS=user-request` |
 | **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 它是库，被两档调用；它自己的库单测归 harness 档，随时跑 |
 
 归类判据：一条测试或一段代码要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，归 checker 档；否则归 harness 档。拿不准的放 checker 档，再由代码三方判要不要挪回。
 依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（它的依赖闭包里没有 `singlefs-checker-tier`，dev-dependencies 也算；源码里零处引它；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判）。两档都要用的一小段代码，宁可各留一份（例：`crates/singlefs-harness/tests/common_corrupted_allocation_record/mod.rs` 抄自 checker 档的坏盘输入），也不让 harness 依赖 checker 档。
-测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；不带里程碑、步号、增补号、并行线号、欠账号（以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`），来历写进文件头的文档注释。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
+测试文件按它测什么起名：领域在前、场景在后（`rollback_by_a_forward_publish`、`crash_enumeration_fixed_script_stream`），checker 档的以模块起头（`crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_`）；名字不带来历照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」，本仓判的形态是以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_c519_*`。`research/scripts/crash-case-check.py` 的 file-names 那一样判，门禁 code-source-discipline 的 test-file-names 格在真仓上跑它。
 不许用的叫法：「checker 包」（分不清是池级 checker 还是 checker 档）、「放量用例」「验证档」（说 checker 档）。
 
 ## harness 档里再分轻用例与耗时用例
 
 - 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
-- 红了只重跑红的那几条，不整份重跑，轻用例与耗时用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`，红的是耗时用例也跑得到），经 Bash 起、加内存包装；修完代码也只重跑红的那几条；整份重跑只在上一趟的日志不全时（脚本判红、一条命令都不打），完整的一遍归提交时的整轮门禁。
+- 红了只重跑红的那几条（`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」），轻用例与耗时用例一样：`python3 research/scripts/rerun-failed-tests.py <上一趟的日志> -p <包>` 按测试目标打印只跑那几条的命令（一律带 `--include-ignored`），经 Bash 起、加内存包装；它判红、一条命令都不打时才整份重跑，完整的一遍归提交时的整轮门禁。
 - 耗时用例：debug 下单条跑到 60 秒及以上的用例，标 `#[ignore = "harness 耗时用例：…"]`；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。单条用时看平常跑的时候量到的，不另外单独量。
 - 调全量崩溃枚举函数（`enumerate_layer0` 一族，快档 `quick_tier` 那几个除外）或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`、或造 `CrashImage`），直接这样做或经同一文件里的函数这样做的测试不是 harness 档的重，它是 checker 档，写进 `crates/singlefs-checker-tier/tests/`；`research/scripts/crash-case-check.py` 的 placement 那一样判这一条，写在别的包里判红，门禁 `checker-tier-layer0-replay` 开跑前在真仓上跑它。
-- 一条用例一个场景：按参数循环、每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的，拆成一个带参数的函数加每个取值一条 `#[test]`，红了只重跑那一条；确是一个场景的（同一个池上按次序做几轮）在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。
+- 一条用例一个场景照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」：本仓按「每一轮新建一个池」认参数循环（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`），确是一个场景的在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。`research/scripts/crash-case-check.py` 的 one-scenario 那一样判，门禁 harness-model-differential-and-scenarios 的 one-scenario 格在真仓上跑它。
 
 ## checker 档自己分快档与全量
 
@@ -32,9 +32,9 @@
 |---|---|---|
 | 快档 | `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（库与集成测试；装置二进制 `src/bin/` 的内联单测不在快档里，归它们的变异表与实验复跑）；门禁 `checker-tier-layer0-replay` 跑它，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
 | 全量 | `checker-tier-layer0-replay` `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求或夜间 |
-| 崩溃放量全量 | `bash research/scripts/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），开跑先照多机配置清场、退出时复原；每项交 `crash-amplification-item.sh`：配置开了双机就两台跑、没开就一台跑，配置 `ENABLE_GPU=1` 时由 GPU 判，显卡照私有显卡配置（模板 `gpu-cards.env.example`）的先后与额度用，做不到（第二台跑不成、点名的卡用不了、建不出表、有状态 GPU 判不了、内核展开超预算）报错退出，不退回少一台或 CPU；一份库装全部分项，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library`，落在开机会清空的目录里被拒；重跑同一条命令就是续跑（同一判法版本下判过的块复用，判法版本变了旧行留着、这一版从头判） | 用户要求或夜间 |
+| 崩溃放量全量 | `bash research/scripts/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），开跑先照多机配置清场、退出时复原；每项交 `crash-amplification-item.sh`：配置开了双机就两台跑、没开就一台跑，配置 `ENABLE_GPU=1` 时由 GPU 判，显卡照私有显卡配置（模板 `gpu-cards.env.example`）的先后与额度用，用不上时照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」报错退出，本仓认的用不上是第二台跑不成、点名的卡用不了、建不出表、有状态 GPU 判不了、内核展开超预算；一份库装全部分项，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library`，落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」，键与判法版本在 `crates/singlefs-checker-tier/src/verdict_store.rs` | 用户要求或夜间 |
 
-checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。
+checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。
 
 ## 崩溃枚举用例住哪、怎么登记
 
@@ -95,6 +95,7 @@ checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutat
 | 判什么 | 谁判 |
 |---|---|
 | 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 门禁 `checker-tier-layer0-replay`；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
+| 形式化验证：恢复重放取前缀的核经 Verus 验过，E165 全量的红例都钉进快速档、在今天的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`），pinned-red-examples-complete、pinned-red-examples-replay 两格判红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` |
 | 模型对拍跑了、每段都判过 | 门禁 harness-model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
 | 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 门禁 `checker-tier-layer0-replay` 开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
 | 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
@@ -102,6 +103,6 @@ checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutat
 | harness 档一条用例一个场景 | 门禁 harness-model-differential-and-scenarios 的 one-scenario 格（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
 | 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
 | 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
-| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |
+| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算）算重型 |
 
 **它们管不到的**：harness 耗时用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
diff --git a/.claude/skills/crash-test/SKILL.md b/.claude/skills/crash-test/SKILL.md
index 84036b68..66663b0f 100644
--- a/.claude/skills/crash-test/SKILL.md
+++ b/.claude/skills/crash-test/SKILL.md
@@ -12,7 +12,7 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设
 | 档 | 住哪 | 什么时候跑 | 怎么跑 |
 |---|---|---|---|
 | harness | `crates/singlefs-harness`：单元与集成测试 | 改了代码随时跑 | `cargo test -p singlefs-harness`（实现员经 `research/scripts/run-with-memory-cap.sh` 跑自己动到的 `--test <目标>`） |
-| checker 档 | `crates/singlefs-checker-tier`（崩溃枚举引擎、注入战役、装置二进制与它们的用例）与门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 的复跑那一格 | 默认只在提交时跑快档；全量由用户要求或夜间 | 命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）；没带，钩子 `.claude/hooks/heavy-test-guard.sh` 拒 |
+| checker 档 | `crates/singlefs-checker-tier`（崩溃枚举引擎、注入战役、装置二进制与它们的用例）与门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 的复跑那一格 | 默认只在提交时跑快档；全量由用户要求或夜间 | 命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）；没带，钩子 `.claude/hooks/heavy-test-guard.sh` 拒 |
 
 ## 四样各落一道门禁
 
@@ -37,4 +37,4 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设
 - **「没复现问题」不等于「没问题」。** 想说崩溃一致性成立，得先说清这一轮枚举了多少个崩溃点，是不是全部；快档只跑小流，全量那几条有没有作数的全绿标记看门禁 checker-tier-layer0-replay 的输出。
 - **checker 不报错，也可能是那条检查压根没实现。** 先看 `.claude/kb/invariants.md` 的状态列。
 - **判定结果读不到，这一轮就作废**，绝不能当成通过。装置读不到结束标记也一样。
-- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（门禁 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份。
+- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（门禁 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份。
```
