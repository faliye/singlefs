# 定义三方第二轮被判的 diff（相对 HEAD，只留点名形式化验证一道、复用环境、包装结局的 hunk；新文件整份）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index 97a196cc..12ab2177 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -49,9 +49,9 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：checker 档包 `crates/singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而范围含它、直接执行它的测试二进制）与门禁 `checker-tier-layer0-replay`、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、全部实验复跑（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格）、E152 装置，只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build（逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 这四道，`gate-triage` 只跑 `gate.sh` 整轮与 experiment-replay 格，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：checker 档包 `crates/singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而范围含它、直接执行它的测试二进制）与门禁 `checker-tier-layer0-replay`、QEMU、herd7、crates 变异整表、形式化验证（门禁 checker-tier-formal-verification，连同直接起它的判法脚本 `research/scripts/verus-proof-check.py` 证明、空真、必红三种用法与 `research/scripts/e165-pin-red-examples.py --replay`）、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、全部实验复跑（门禁 checker-tier-research-build-and-replay 的 experiment-replay 格）、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算），只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build（逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 这五道，`gate-triage` 只跑 `gate.sh` 整轮与 experiment-replay 格，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，checker 档包 `singlefs-checker-tier` 的测试二进制也不许。
-- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 checker-tier-crates-mutation-replay 在里面逐条套了，门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格与门禁 harness-model-differential-and-scenarios 在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：research-unit-tests 格 `GATE_RESEARCH_BUILD_MEMORY_MAX`，harness-model-differential-and-scenarios `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 checker-tier-crates-mutation-replay 在里面逐条套了，门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格、门禁 harness-model-differential-and-scenarios 与门禁 checker-tier-formal-verification 在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：research-unit-tests 格 `GATE_RESEARCH_BUILD_MEMORY_MAX`，harness-model-differential-and-scenarios `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`，checker-tier-formal-verification `GATE_VERUS_MEMORY_MAX` 与 `GATE_E165_PINNED_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 读文件：5,000 行以内的整读一次（Read 或 `cat`），不切片；超过 5,000 行的先 `grep -n` 定位、再按行段读；同一份文件读过一次、之后没改过，就不再重读。几条只读命令（`ls`、`grep`、`wc`、`sha256sum` 这类）合成一次 Bash，不一条一次。每多一次调用就要把整份上下文重读一遍，切片读比整读贵。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。上下文到 400k 就停在一个能交接的点交回（定义另写了线的照定义）：报告写做完的、做到一半的（文件与差哪一步）、没开的。
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index 4c161255..c1433834 100644
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
@@ -19,19 +19,20 @@ omitClaudeMd: true
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：`checker-tier-layer0-replay` 与 checker-tier-lkmm 的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；checker-tier-qemu-device-streams 的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/checker-tier-qemu-device-streams.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
+- checker-tier-formal-verification 的两个内存上限（第 1b 步用），各一个带单位的上限：每个 Verus 进程一个（空真与必红世界几处同时验，每处各占一份），E165 装置一个；各写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里 Verus 那条命令、装置那条命令经包装跑出的峰值）还是推的。
 - `checker-tier-layer0-replay` 跑哪一档：默认快档（`cargo test --release -p singlefs-checker-tier --lib --tests` 加逐条核全绿标记，`.claude/rules/verification.md`），派发提示什么都不写就是它；要跑全量的写明「`checker-tier-layer0-replay` 带 --full」与在哪棵 worktree 里跑（HEAD + 暂存区的 worktree 路径，或写明由你照 `checker-tier-layer0-replay` 快档出路里那三行建）、命令前缀 `SINGLEFS_HEAVY_TESTS=user-request`；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
 - 报告路径与草稿目录。
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 `checker-tier-layer0-replay`、或有 `checker-tier-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 `checker-tier-layer0-replay`，等它结束。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，checker-tier-lkmm 是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 查 `crates`，checker-tier-qemu-device-streams 按名字读的 `research/results/` 产物这里不查，由整轮门禁里 checker-tier-research-build-and-replay 的 experiment-replay 格兜；checker-tier-lkmm 查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 checker-tier-lkmm 只跑静态那几层；要没有输出，别处的未跟踪文件不挡；checker-tier-lkmm 读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
-   1b. `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；checker-tier-crates-mutation-replay 在里面逐条套了，照第 2 步直接跑，外面不再包一层。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 `checker-tier-layer0-replay`、或有 `checker-tier-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 `checker-tier-layer0-replay`，等它结束。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay、checker-tier-formal-verification 是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，checker-tier-lkmm 是 `litmus crates .claude/scripts/lkmm.sh`，四道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay 查 `crates`，checker-tier-qemu-device-streams 按名字读的 `research/results/` 产物这里不查，由整轮门禁里 checker-tier-research-build-and-replay 的 experiment-replay 格兜；checker-tier-formal-verification 查 `crates` 与 `research/results`（这一道的指纹按整个 `research/results/` 算，别的会话在那里没暂存的改动与没跟踪的产物都算进去：那里有一份，这一道就不跑、也就不写全绿标记，停下报告交主 agent）；checker-tier-lkmm 查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 checker-tier-lkmm 只跑静态那几层；要没有输出，别处的未跟踪文件不挡；checker-tier-lkmm 读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
+   1b. `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；checker-tier-crates-mutation-replay 在里面逐条套了，照第 2 步直接跑，外面不再包一层。checker-tier-formal-verification 也在里面套了（每次 Verus、E165 装置各经包装），照第 2 步直接跑、外面不再包一层，输入给的两个上限设进阶段读的变量，写在 `nice` 前面：`GATE_VERUS_MEMORY_MAX=<Verus 的上限> GATE_E165_PINNED_MEMORY_MAX=<装置的上限>`。
    1c. `checker-tier-layer0-replay` `--full` 开跑前在那棵 worktree 里跑 `bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments <worktree>` 与 `awk -F'\t' '$1 ~ /^crash-case:/ && $3 ~ /shard=across-machines/ { print $1 }' <worktree>/.claude/gate.d/stage-inputs.tsv`，两样原样抄进报告：前一条退 0 是双机分片开着，抄它打的 `PEER_SSH_HOST=` 与 `PEER_MEMORY_CAP=` 两行（第二台那一片的内存上限：驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 用它起那一片，本机那一片与 merge 由第 1b 步的包装管），退 1 是关着、照单机跑，抄它那一句原因；后一条列出的是开着时两台各跑一片的用例。驱动只由 `checker-tier-layer0-replay` `--full` 调（`--merged-log`），不单独跑它。
-2. 只在提交时（或主 agent 转达用户要求时）跑这四道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay，与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑别的门禁、`gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。checker-tier-qemu-device-streams、checker-tier-lkmm 与 `checker-tier-layer0-replay` 并行起（各自后台、各自内存上限，照第 1b 步），checker-tier-crates-mutation-replay 等 `checker-tier-layer0-replay` 跑完再起（整表复跑与层 0 全量争 CPU）；四道的命令都带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`；`checker-tier-layer0-replay` 默认跑快档（`bash .claude/gate.d/checker-tier-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「`checker-tier-layer0-replay` 带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑全量（`bash <worktree>/.claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
-3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
+2. 只在提交时（或主 agent 转达用户要求时）跑这五道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification，与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑别的门禁、`gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-layer0-replay` 并行起（各自后台、各自内存上限，照第 1b 步），checker-tier-crates-mutation-replay 等 `checker-tier-layer0-replay` 跑完再起（整表复跑与层 0 全量争 CPU）；五道的命令都带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`；`checker-tier-layer0-replay` 默认跑快档（`bash .claude/gate.d/checker-tier-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 checker 档包 `singlefs-checker-tier` 不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「`checker-tier-layer0-replay` 带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑全量（`bash <worktree>/.claude/gate.d/checker-tier-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记耗时与退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`，checker-tier-formal-verification 那一条在 `nice` 前面照第 1b 步带两个上限：`{ SINGLEFS_HEAVY_TESTS=commit GATE_VERUS_MEMORY_MAX=<Verus 的上限> GATE_E165_PINNED_MEMORY_MAX=<装置的上限> nice -n 19 bash .claude/gate.d/checker-tier-formal-verification.sh; echo "exit=$?"; } > <草稿目录>/checker-tier-formal-verification.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。checker-tier-formal-verification 的「✗」行写着「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」的，那一格这一趟没判出结果：表里那一格记「没跑成」，不记成红在证明、表或实现上，原样抄它的「→」；照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办：命令与退出码写进报告，不绕开包装重跑。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（新池新建文件的干跑） 产物或闭式比对的那几行；`checker-tier-layer0-replay` `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行，写着第二台那一片经 `run-with-memory-cap.sh <上限>` 起）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、`checker-tier-layer0-replay` 判 merge 那一趟的日志绿；第二台那一片退 250–254 是内存包装自己的结局，照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办，那一条用例这一趟的判定不算。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
-6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus Cargo.toml Cargo.lock research/scripts | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。
+6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus Cargo.toml Cargo.lock research/scripts | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单）；跑 checker-tier-formal-verification 的那一趟，两条命令的路径都再加它在 `.claude/gate.d/stage-inputs.tsv` 那一行登记、这里还没有的几条（`.claude/gate.d/verus-pins.tsv .claude/gate.d/e165-pinned-red-examples.tsv .claude/gate.d/lib/stage-cells.sh research/results research/mutations/e165_replay_bounded_enumeration_custom.tsv`），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。
 
 ## 写范围
 
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 03d6a569..3b0cd9b5 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -24,9 +24,9 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-   1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（checker-tier-research-build-and-replay 的 experiment-replay 格经 `research/scripts/replay.sh` 逐条套了，同一道的 research-unit-tests 格与 harness-model-differential-and-scenarios 在阶段里面经包装）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、harness-model-differential-and-scenarios 与 experiment-replay 格才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 这几道重阶段在 `gate.sh` 里照各自的复用判定走（`checker-tier-layer0-replay` 跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀，不带的 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查一类）。超过 Bash 单次上限时后台跑、结束后读输出。
-3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（`checker-tier-layer0-replay` 的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 `checker-tier-layer0-replay` 脚本本身（`.claude/gate.d/checker-tier-layer0-replay.sh`；全绿标记的键还含它与 `cargo -V`、`rustc -V`） ⇒「这一轮」，下一步原样抄 `checker-tier-layer0-replay` 出路里的三行命令，一条都没碰 ⇒「不是这一轮」；✗ 行写着内存包装退 25x 的（research-unit-tests 格与 harness-model-differential-and-scenarios 的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）不按它点名的文件判：250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
+   1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（checker-tier-research-build-and-replay 的 experiment-replay 格经 `research/scripts/replay.sh` 逐条套了，同一道的 research-unit-tests 格与 harness-model-differential-and-scenarios 在阶段里面经包装，checker-tier-formal-verification 每次 Verus、E165 装置各经包装）。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、harness-model-differential-and-scenarios 与 experiment-replay 格才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 这几道重阶段在 `gate.sh` 里照各自的复用判定走（`checker-tier-layer0-replay` 跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀，不带的 `.claude/hooks/heavy-test-guard.sh` 拒（提交时才跑的检查一类）。超过 Bash 单次上限时后台跑、结束后读输出。
+3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（`checker-tier-layer0-replay` 的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 `checker-tier-layer0-replay` 脚本本身（`.claude/gate.d/checker-tier-layer0-replay.sh`；全绿标记的键还含它与 `cargo -V`、`rustc -V`） ⇒「这一轮」，下一步原样抄 `checker-tier-layer0-replay` 出路里的三行命令，一条都没碰 ⇒「不是这一轮」；✗ 行写着内存包装退 25x 的（research-unit-tests 格与 harness-model-differential-and-scenarios 的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」，checker-tier-formal-verification 的「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）不按它点名的文件判：250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
 ## 写范围
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 467c2c3b..63cd14c5 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -55,4 +55,4 @@ required-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件
 
 ## 没做什么（固定会有的）
 
-- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异表）归 `crash-verifier`；没提交。
+- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异表、Verus 形式化验证）归 `crash-verifier`；没提交。
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
diff --git a/.claude/gate.d/checker-tier-formal-verification.sh b/.claude/gate.d/checker-tier-formal-verification.sh
new file mode 100644
index 00000000..a9cd60c9
--- /dev/null
+++ b/.claude/gate.d/checker-tier-formal-verification.sh
@@ -0,0 +1,154 @@
+#!/usr/bin/env bash
+# admission: always 判的是此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
+# run-condition: none 要的 Verus、cargo 与装置由各格的判法自己判，缺了判红（缺工具直接红，不交给 gate.sh 预判）；核还没进仓时 Verus 那四格报本次未跑
+# gate-stage: 形式化验证（里程碑三七·5：Verus 证明恢复重放取前缀的核，版本钉住、空真与必红世界；E165 全量的红例钉进快速档）
+# gate-category: checker-tier 类
+# gate-similar: checker-tier-lkmm.sh 同是 checker-tier 类里起外部验证器的一道，接法照抄它（判法放脚本、阶段只接进门禁；按 stage-inputs.tsv 复用、判绿写全绿标记、environment= 记工具版本）；它起 herd7 判 litmus 的内存序声明，这里起 Verus 判 crates/ 里的证明，工具、输入、样本与失败方式都不同，并进去会让两件不相干的事共用一个复用判定
+# gate-similar: checker-tier-crates-mutation-replay.sh 它逐条改坏 crates/ 跑 cargo test 要求点名的测试红；这里的必红世界格逐条改坏核要求 Verus 报错，红的判据是证明失败不是测试失败，写不进它的 crates/mutations.tsv（调度记录 records/2026-10-01-里程碑三形式化接入.md 第五节「分法改动」）
+# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判实验二进制的变异表成形、锚点唯一；这里的两格判 E165 红例账与钉住的表、并逐行在今天的实现上重放，对象与判法都不同
+# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格整份复跑入库实验比产物；这里只逐行重放钉住的红例（快速档），是用户 2026-10-01 定的「全量红例加进快速档、跑完就进门禁」那一件
+# gate-similar: checker-tier-layer0-replay.sh 它判崩溃点重放与登记的崩溃枚举用例；这里判的是证明与有界穷举的红例，不起崩溃枚举
+#
+# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就六格按下面的次序全跑，格名写错退 2）：
+# gate-cell: verus-toolchain-pinned Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同
+# gate-cell: verus-proof Verus 带 --no-cheating 验整个 crate，限时内出结局、errors 0、verified 等于登记的数，核里没有作弊标记
+# gate-cell: verus-vacuity 核里每一处 requires 的函数体头插 assert(false) 都验不过（前置条件可满足，不空真）
+# gate-cell: verus-must-red-worlds 必红世界表每行改坏核之后 Verus 都报错且出现期望片段，原文恰好命中一次
+# gate-cell: pinned-red-examples-complete 登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里
+# gate-cell: pinned-red-examples-replay 钉住的每一行点名的变异还在它的表里，装置在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同
+# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh（写法与判据在它的文件头）。
+#
+# 判法都在脚本里，这一道只负责接进门禁、给样本开关：
+#   Verus 四格：research/scripts/verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根>（判据、默认值、弄坏开关写在它的文件头）；
+#     登记表 .claude/gate.d/verus-pins.tsv（发布 tag 与提交、zip 的 sha256、六个 crates.io 包、被验的 crate 与 verified 的登记数、核、必红世界表，同源的依据写在它的注释里）。
+#     核没进仓（登记的核不在、crate 不依赖 vstd、src 里没有 verus!）时四格都退 77；核不在而 crate 已经接了 Verus 判红。
+#     超时判红（C5：超时不当通过）：每次 Verus 经 research/scripts/run-with-memory-cap.sh 的限时（GATE_VERUS_TIMEOUT_SECONDS，默认 600 秒），
+#     单条查询撞 Verus 自己的 rlimit 同样按报错判红。上限 GATE_VERUS_MEMORY_MAX（默认 4G），几处同时验 GATE_VERUS_JOBS，线程取 capped.sh 设的 SINGLEFS_THREAD_CAP。
+#     必红世界格：改坏的副本输出里的报错除去 rlimit 用尽与 aborting due to 之外至少还有一条证明错才算红；只剩 rlimit 用尽的那一行不作数，判红。
+#     函数上的 verifier::rlimit 属性盖过命令行的 --rlimit：核里有它，verus-proof 格当作弊标记判红；必红世界表的替换文里有它，那一行判红。
+#     Verus 每次直接起、不经 cargo verus（它按内容缓存，内容不变时不出判定行）；被验的 crate（今天是 crates/singlefs-replay-prefix，核就是它的 src/lib.rs）
+#     只依赖 vstd，Verus 只编这一个 crate、用它自带的 vstd；crate 除 vstd 之外还有依赖判红。
+#     空真格一处一份副本、一次 Verus：几处一起插时，外层函数入口那一处失败之后里面闭包的那一处不单报。
+#   红例两格：research/scripts/e165-pin-red-examples.py --check / --replay（表的格式写在表头 .claude/gate.d/e165-pinned-red-examples.tsv）。
+#     装置 e165_replay_bounded_enumeration 还不认 --pinned-cases（第 4 批 3-6 才加）时，replay 格判完「变异还在」那一半就退 77。
+#
+# 复用（与 checker-tier-lkmm.sh 同一种接法，只在真仓与 --staged 的临时树里、不给 --check 时）：开跑前问 research/scripts/stage-must-run.sh，
+#   登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行的路径与 environment=（本阶段 --tool-versions 打的那一行：Verus 的版本，verus、rust_verify、z3、vstd 的 sha256
+#   与 VERUS_Z3_PATH）都没变就整道退 77；
+#   六格全绿（一格本次未跑都没有）之后记下 Verus 版本（admission.py gate-record-environment）、写这批输入的全绿标记（admission.py stage-marker-write；
+#   标记的指纹里也有 --tool-versions 那一行，换了 Verus 旧标记就不作数），写不成只报不红。
+#
+# 样本（fixtures/checker-tier-formal-verification.sh/<名>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）：
+#   verus-toolchain-pinned：-green、-commit-red、-fingerprint-red、-lock-red（假的 Verus 安装与假的 cargo 缓存放在样本的 .formal-sample/ 下）
+#   verus-proof：verus-proof-green、-counterexample-red（带反例）、-rlimit-red（单条查询撞 rlimit）、-timeout-red（rlimit 设成无穷、样本限时 20 秒，撞墙钟限时）、
+#     -cheating-red、-rlimit-attribute-red（函数上挂 verifier::rlimit 属性）、-count-red、-kernel-deleted-red、-extra-dependency-red（crate 除 vstd 之外还有依赖），
+#     与 verus-proof-kernel-absent（核没进仓，退 77）
+#   verus-vacuity：-green、-red（前置条件互相矛盾）、-closure-red（外层函数的前置条件可满足、里面闭包的不可满足）
+#   verus-must-red-worlds：-green、-survives-red、-anchor-red、-rlimit-red（改坏之后只剩 rlimit 用尽，不作数）、
+#     -rlimit-with-proof-error-green（rlimit 用尽之外另有一条真的证明错，红计入）、-rlimit-attribute-red（替换文里有 verifier::rlimit 属性）、
+#     -results-line-green（期望片段写成 Verus 普通输出判定行的「verified, 1 errors」，照 JSON 的 verified / errors 拼回那一行比，红计入）
+#   pinned-red-examples-complete：-green、-red；pinned-red-examples-replay：-green、-red（桩装置把一行打错）、-variant-red（点名的变异改了名）、
+#     pinned-red-examples-replay-device-absent（没有桩、装置源码不认 --pinned-cases，退 77）
+#   Verus 那几份样本起的是真的 Verus（SINGLEFS_VERUS_ROOT 或登记表 release 行的安装目录），没装就判红，与真仓一样。
+# 只供样本的开关（被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认，与共用库认 .gate-cells 同一种判法；真仓与 --staged 下放了也不认）：
+#   .formal-sample/verus-install/         版本一致格改读这里的 version.json（不起它）
+#   .formal-sample/cargo-home/            版本一致格改读这里的 registry/src/*/
+#   .formal-sample/verus-timeout-seconds  Verus 那几格的限时改成文件里的秒数
+#   .formal-sample/e165-stub.sh           replay 格改用这个桩当装置（经 bash 起，不经内存包装）
+# 弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>（逐项让哪份样本判错写在两份脚本的文件头）；
+#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；marker-ignored 让每份样本六格都跑，汇总那条「跑了 1 格」找不到）。
+#
+#   bash .claude/gate.d/checker-tier-formal-verification.sh [项目根]
+#   bash .claude/gate.d/checker-tier-formal-verification.sh --list        逐行打格名与判什么，不起 Verus、不编
+#   bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions   打 Verus 的版本与安装文件的 sha256（stage-inputs.tsv 的 environment= 用；不起 Verus，不算重型）
+set -uo pipefail
+source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
+preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
+STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
+VERUS_CHECK="$STAGE_REPOSITORY/research/scripts/verus-proof-check.py"
+PIN_CHECK="$STAGE_REPOSITORY/research/scripts/e165-pin-red-examples.py"
+ADMISSION_MODULE="$STAGE_REPOSITORY/research/scripts/admission.py"
+source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
+stage_cell verus-toolchain-pinned cell_verus_toolchain_pinned "Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同" \
+  "装登记的 Verus 发布或用 SINGLEFS_VERUS_ROOT 指到它；缓存不对就删掉那个包目录再 cargo fetch --locked；要换版本先照 .claude/gate.d/verus-pins.tsv 的注释复核同源再改登记"
+stage_cell verus-proof cell_verus_proof "Verus 带 --no-cheating 验整个 crate，限时内出结局、errors 0、verified 等于登记的数，核里没有作弊标记" \
+  "照 Verus 报错的位置修证明或实现；verified 数变了而确是加减了证明，就改 .claude/gate.d/verus-pins.tsv 的 crate 行并写明加减了哪几个；超时先查哪条查询慢，不许靠放宽限时了事；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改证明或实现"
+stage_cell verus-vacuity cell_verus_vacuity "核里每一处 requires 的函数体头插 assert(false) 都验不过（前置条件可满足，不空真）" \
+  "验得过的那一处前置条件不可满足：改前置条件让它可满足；插入点认错了就按 research/scripts/verus-proof-check.py 的 requires_insertion_points 改写法；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
+stage_cell verus-must-red-worlds cell_verus_must_red_worlds "必红世界表每行改坏核之后 Verus 都报错且出现期望片段，原文恰好命中一次" \
+  "锚点腐化就照今天的核改那一行的原文；改坏了还验得过就补证明或收紧 spec；表在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的路径；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改表或证明"
+stage_cell pinned-red-examples-complete cell_pinned_red_examples_complete "登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里" \
+  "补钉：python3 research/scripts/e165-pin-red-examples.py <没钉全的红例账>（已钉的不重复；这一格的输出里给了整条命令）"
+stage_cell pinned-red-examples-replay cell_pinned_red_examples_replay "钉住的每一行点名的变异还在它的表里，装置在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同" \
+  "变异行不在了就找回那一行；装置逐行对不上是今天的实现在钉住的输入上与原实现不同，按回归修实现；装置报指纹漂了先查它的枚举次序与状态编码；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改变异行或实现"
+stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
+root_argument=""
+tool_versions_requested=0
+while (($#)); do
+  case "$1" in
+    --tool-versions) tool_versions_requested=1; shift ;;
+    -*)
+      echo "  ✗ 认不出的选项 $1"
+      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--tool-versions 与一个项目根。"
+      exit 2 ;;
+    *) root_argument="$1"; shift ;;
+  esac
+done
+ROOT="${root_argument:-$STAGE_REPOSITORY}"
+cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
+ROOT="$(pwd)"
+if ((tool_versions_requested)); then
+  exec python3 "$VERUS_CHECK" tool-versions "$ROOT"
+fi
+
+# 被判的根是样本（不是这道阶段所在的仓、也不在 --staged 里）才认 .formal-sample/ 下的开关
+sample_mode=0
+if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
+toolchain_options=()
+verus_options=()
+replay_options=()
+if ((sample_mode)); then
+  [[ -d .formal-sample/verus-install ]] && toolchain_options+=(--verus-root "$ROOT/.formal-sample/verus-install")
+  [[ -d .formal-sample/cargo-home ]] && toolchain_options+=(--cargo-home "$ROOT/.formal-sample/cargo-home")
+  [[ -f .formal-sample/verus-timeout-seconds ]] && verus_options+=(--timeout "$(tr -dc '0-9' < .formal-sample/verus-timeout-seconds)")
+  [[ -f .formal-sample/e165-stub.sh ]] && replay_options+=(--binary "$ROOT/.formal-sample/e165-stub.sh" --memory-cap none)
+fi
+
+cell_verus_toolchain_pinned() { python3 "$VERUS_CHECK" toolchain "$ROOT" ${toolchain_options[@]+"${toolchain_options[@]}"}; }
+cell_verus_proof() { python3 "$VERUS_CHECK" proof "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
+cell_verus_vacuity() { python3 "$VERUS_CHECK" vacuity "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
+cell_verus_must_red_worlds() { python3 "$VERUS_CHECK" must-red-worlds "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
+cell_pinned_red_examples_complete() { python3 "$PIN_CHECK" --check --root "$ROOT"; }
+cell_pinned_red_examples_replay() { python3 "$PIN_CHECK" --replay --root "$ROOT" ${replay_options[@]+"${replay_options[@]}"}; }
+
+# 复用：真仓与 --staged 下、整道跑时先问要不要跑（判定只认 1 为可跳过，别的非 0 判红，照 stage-run-or-skip.sh）
+if ((sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)); then
+  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
+  if ( stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
+         "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径、Verus 的版本怎么进判定见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")" ); then
+    reuse_exit=0
+  else
+    reuse_exit=$?
+  fi
+  if ((reuse_exit == 77)); then
+    [[ -n "${GATE_NOT_RUN_FILE:-}" ]] && printf '%s 整道复用上一次整轮全绿的判定（输入与 Verus 版本都没变），本次未跑\n' "$(basename "$0")" >> "$GATE_NOT_RUN_FILE"
+    exit 77
+  fi
+  ((reuse_exit == 0)) || exit "$reuse_exit"
+fi
+
+# 逐格跑；本次未跑的格先收进自己的文件：有一格本次未跑就不记环境、不写全绿标记（没判全的一趟不算上次成功），再原样转给 $GATE_NOT_RUN_FILE
+not_run_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
+if ( export GATE_NOT_RUN_FILE="$not_run_record"; stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
+if [[ -n "${GATE_NOT_RUN_FILE:-}" && -s "$not_run_record" ]]; then cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"; fi
+if ((stage_exit == 0 && sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)) && [[ ! -s "$not_run_record" ]]; then
+  # 判绿之后记下 Verus 的版本、写这批输入的全绿标记：下一趟整轮输入与版本都没变才复用；写不成只报不红（方向是多跑）
+  if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0")"; then
+    echo "  ! 没记下这一次的 Verus 版本（admission.py gate-record-environment 没成）：下一趟整轮这一道照跑，不会复用"
+  fi
+  if ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0")" >/dev/null; then
+    echo "  ! 没写成这一道的全绿标记（admission.py stage-marker-write 没成）：下一趟这一道照跑，不会复用"
+  fi
+fi
+rm -f "${not_run_record:?}"
+exit "$stage_exit"
diff --git a/.claude/gate.d/code-tooling.sh b/.claude/gate.d/code-tooling.sh
index 845f28ff..ccdf074c 100755
--- a/.claude/gate.d/code-tooling.sh
+++ b/.claude/gate.d/code-tooling.sh
@@ -251,6 +255,9 @@ SELFTEST_RUNNERS=(
   "python3 research/scripts/migrate-changelog-format.py --selftest"
   "bash research/scripts/crash-amplification.sh --selftest"
   "bash research/scripts/crash-amplification-item.sh --selftest"
+  "python3 research/scripts/broad-mutation.py --selftest"
+  "python3 research/scripts/verus-proof-check.py --selftest" "python3 research/scripts/e165-pin-red-examples.py --selftest"
+  "bash research/scripts/e165-replay-mutation-kill-bounds.sh --selftest"
 )
 SELFTEST_RUNNERS_STARTED_ALONE=("bash research/scripts/layer0-shard-run.sh --selftest" "python3 research/scripts/agent-watch.py --selftest")
 
@@ -659,7 +666,7 @@ PY
 #     （按模式找进程在执行前拒绝；它的判别力由上游 selftest 的样本管，门禁「门禁自检」阶段跑它，这里只查注册着、文件在）；
 #   ⑩ SessionStart 里有一条 matcher 覆盖 compact、一条 matcher 同时覆盖 startup 与 resume（可以是同一条），命令都指向 `session-start.sh`
 #     （压缩上下文之后补回分支、未提交数、最近的记录与看门狗命令；会话启动或恢复时报近几小时内核因为内存不够杀过的进程，读不到内核日志报「没查成」）。
-#   ⑫ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `heavy-test-guard.sh` 的 hook（重型测试——层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置——子 agent 越出自己那一份就拒，主 agent 与 crash-verifier、gate-triage 不带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 也拒）。
+#   ⑫ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `heavy-test-guard.sh` 的 hook（重型测试——层 0、QEMU、herd7、crates 变异整表、Verus 形式化验证（连同直接起它的判法脚本）、全量测试、整轮门禁、全部实验复跑、E152 装置——子 agent 越出自己那一份就拒，主 agent 与 crash-verifier、gate-triage 不带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 也拒）。
 #   ⑬ 两份共用模块顶层定义的每个函数名，只许在它自己那一份里定义：`.claude/hooks/` 下别的文件里再 `def <名字>(` 一份就判红——
 #     切词模块 `.claude/hooks/lib_shell_words.py`（判 Bash 命令的 hook 手抄一份切词，一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出），
 #     重型测试判定模块 `.claude/hooks/lib_heavy_tests.py`（heavy-test-guard.sh 与看门狗 research/scripts/agent-watch.py 共用 classify、classify_process、
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index 82cdfd45..fcdc28b0 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -39,7 +39,16 @@
 #   全部实验复跑  .claude/gate.d/checker-tier-research-build-and-replay.sh，只跑它的单测那一格（--check research-unit-tests）的除外：那一格落到下一类
 #   整轮门禁      gate.sh、research/scripts/gate-staged.sh
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
-#   门禁阶段按文件名认，不按编号认；带 --list、--list-items（只打格名表、逐项点名的清单）的哪一道都不算；带 --write（生成）的不算。
+#   广谱变异      research/scripts/broad-mutation.py（python 脚本不读正文，按名字与位置、解到的真实路径、文件开头的签名行认：`python3 -W / -X <值> 脚本`、
+#                 `python3 -m broad-mutation`、指到它的符号链接、带签名行的改名拷贝、`python3 -c` 的代码里点它的都认）。凡要起 cargo 或改库目录的都算：
+#                 整趟、--shard / --from / --package、--item 点名（点几条、带不带 --skip-stage-two 都算）、--stage-two-only、--sync-tree（换全局的版本指针）、
+#                 --import-results；只有带 --list-items / --summary / --help 的不算。子 agent 的放行表里没有这一类：算的那些只有主 agent 带前缀能跑，
+#                 不算的那几样谁都能跑。罩不住的写法在 lib_heavy_tests.py 文件头「广谱变异的逐条驱动」那一段
+#   Verus         .claude/gate.d/checker-tier-formal-verification.sh（逐格起 Verus、空真与必红世界逐处起、编装置重放钉住的红例）。子 agent 里只有 crash-verifier 的放行表有这一类：
+#                 提交时由它带前缀单独跑，主 agent 带前缀也能单独跑，gate-triage 起的整轮 gate.sh 里也有它；直接起的 verus 与 cargo verus 不算（写证明的实现员要用，判法在 lib_heavy_tests.py）。
+#                 这一道的两份判法脚本直接起同算这一类：research/scripts/verus-proof-check.py 的 proof、vacuity、must-red-worlds，research/scripts/e165-pin-red-examples.py --replay；
+#                 只读的（verus-proof-check.py 的 toolchain、fingerprint、tool-versions，e165-pin-red-examples.py 的追加与 --check）不算（lib_heavy_tests.py 的 verus_judge_use）
+#   门禁阶段按文件名认，不按编号认；带 --list、--list-items、--tool-versions（只打格名表、逐项点名的清单、工具版本）的哪一道都不算；带 --write（生成）的不算。
 # 提交时才跑的检查（不是重型，放行照同一张表；用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的 … 提示不执行 根本不靠谱」）：
 #   上面几类之外的 .claude/gate.d/ 阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、gate-overlap.py
 #   （--list 查清单不算）、rules-lint.sh、hooks-registered.sh、stage-selftest.sh；research/scripts/gate-structure-check.py；research/scripts/、.claude/hooks/ 下
@@ -50,15 +59,22 @@
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
@@ -109,7 +125,9 @@
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
@@ -156,9 +174,12 @@ SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有
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
@@ -556,12 +599,16 @@ def heavy_uses(text, directory, environment=None, frames_above=(), script=None,
     return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)
 
 POLICY = ("→ 规矩：重型测试就是 checker 档那一批（.claude/rules/verification.md）：checker 档包 singlefs-checker-tier 的测试（按包判，挑哪个目标都算）与 54 号、QEMU、herd7、crates 变异整表、"
-          "全量测试、整轮门禁、全部实验复跑、E152 装置，只在提交代码时跑一次、或用户要求时跑；"
+          "Verus 那一道门禁（checker-tier-formal-verification.sh，连同直接起它的判法脚本 research/scripts/verus-proof-check.py 的 proof / vacuity / must-red-worlds "
+          "与 research/scripts/e165-pin-red-examples.py --replay；直接起 verus 验证明、只打 --tool-versions 不算）、"
+          "全量测试、整轮门禁、全部实验复跑、E152 装置、广谱变异（research/scripts/broad-mutation.py 起 cargo 或改库目录的每一种用法，只有 --list-items / --summary 不算），"
+          "只在提交代码时跑一次、或用户要求时跑；"
           "子 agent 一律不跑，只跑 harness 档——自己动到的 singlefs-harness 与别的非 checker 档包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
           "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
-          "→ 各自那一份：crash-verifier 只跑 54 号与 checker-tier-qemu-device-streams.sh、checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh 三道，"
-          "与它们底下的 checker 档包 singlefs-checker-tier 的测试（快档与登记的全量用例）、qemu-system、lkmm.sh / herd7、crates 变异整表；"
-          "gate-triage 只跑 `gate.sh` 整轮与 checker-tier-research-build-and-replay.sh（54 号与另外三道靠「输入没变就复用上一次全绿判定」）；"
+          "→ 各自那一份：crash-verifier 只跑 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 五道，"
+          "与它们底下的 checker 档包 singlefs-checker-tier 的测试（快档与登记的全量用例）、qemu-system、lkmm.sh / herd7、crates 变异整表、"
+          "Verus 那一道的两份判法脚本（起 Verus、编 E165 装置的那几种用法）；"
+          "gate-triage 只跑 `gate.sh` 整轮与 checker-tier-research-build-and-replay.sh（54 号与另外四道靠「输入没变就复用上一次全绿判定」）；"
           "两个都要带那个前缀，都不跑全量 `cargo test`。\n"
           "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
           "子 agent 在交回里写明要跑什么、为什么，交主 agent 去问（派发提示里点名要你跑的也一样，写明被这道闸拒了）。")
@@ -758,6 +805,25 @@ def selftest(hook_dir):
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
+            # Verus 那一道的两份判法脚本直接起：干活的用法照那一道判（lib_heavy_tests.py 的 verus-judges-by-stage-only 下前四格红）；
+            # 只打工具版本的谁都能跑（tool-versions-counted 下末两格红）
+            ("实现员直接起判法脚本跑必红世界", writer, "python3 research/scripts/verus-proof-check.py must-red-worlds .", 2),
+            ("主 agent 不带前缀直接起判法脚本跑必红世界", None, "python3 research/scripts/verus-proof-check.py must-red-worlds .", 2),
+            ("调查员经内存包装起红例重放", "investigator", "bash research/scripts/run-with-memory-cap.sh 4G python3 research/scripts/e165-pin-red-examples.py --replay --root .", 2),
+            ("门禁分诊带前缀直接起红例重放", triage, commit + "python3 research/scripts/e165-pin-red-examples.py --replay", 2),
+            ("崩溃验证员带前缀直接起判法脚本验证明", crash, commit + "python3 research/scripts/verus-proof-check.py proof .", 0),
+            ("实现员直接起判法脚本核版本一致（只读安装与缓存）", writer, "python3 research/scripts/verus-proof-check.py toolchain .", 0),
+            ("实现员只打 Verus 那一道的工具版本", writer, "bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions", 0),
+            ("主 agent 不带前缀只打 Verus 那一道的工具版本", None, "bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions", 0),
             ("实现员跑 gate-staged.sh", writer, "bash research/scripts/gate-staged.sh", 2),
             ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\ncargo test --all\nEOF", 2),
             ("实现员直接执行 checker 档的测试二进制", writer, tier_binary + " --test-threads 4", 2),
@@ -1201,7 +1297,7 @@ def selftest(hook_dir):
               "HEAVY_TEST_GUARD_DISABLE_CHECK 设着的话这里本来就该红")
         return 1
     refused_count = sum(1 for item in results if item[1] == 2)
-    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表拒绝，harness 档经内存包装放行，"
+    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表、Verus 那一道与它的判法脚本拒绝，harness 档经内存包装放行，"
           "崩溃验证员与门禁分诊带前缀跑各自那一份放行、越出那一份或不带前缀拒绝，主 agent 带 commit / user-request 放行、不带或带别的值拒绝；"
           "写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；"
           "同一条命令里 heredoc（cat / tee）写出或 cp 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法照旧记检出；"
diff --git a/.claude/hooks/lib_heavy_tests.py b/.claude/hooks/lib_heavy_tests.py
index 43d24db8..80e0e38a 100644
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -51,9 +56,16 @@ libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）
 --test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列；
 libtest 在 `--` 处停止认选项，`--` 之后的 --list 是过滤词，照样全跑，不算只列。
 门禁阶段按文件名认（.claude/gate.d/ 顶层的 *.sh，lib-*.sh 与 lib/ 底下的共用库不是阶段；STAGE_KIND）：重型的是 checker-tier-layer0-replay.sh 与去了编号的
-checker-tier-qemu-device-streams.sh、checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh、checker-tier-research-build-and-replay.sh；
-最后那一道（research 构建与单测、实验复跑两道合成）只有复跑那一格是重型：--check 只点名 research-unit-tests 的不算全部实验复跑。带 --list 或 --list-items 的（只打格名表、
-逐项点名的清单）哪一道都不算；带 --write 的（生成）不算。
+checker-tier-qemu-device-streams.sh、checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh、checker-tier-research-build-and-replay.sh、
+checker-tier-formal-verification.sh（起 Verus 这个外部验证器、编装置逐行重放钉住的红例，照 .claude/rules/verification.md 的归类判据归 checker 档；类名「Verus」）；
+直接起的 verus 与 cargo verus 不算重型：写证明的实现员改一处验一次，验一个 crate 是秒级（接回可行性那一趟量到 8–21 秒），与 cargo build / clippy 同一类，
+认成重型就验不了自己写的证明；只有门禁那一道（逐格验、空真与必红世界逐处起 Verus、编装置）算。
+这一道的两份判法脚本直接起，干活的用法同算这一类（verus_judge_use）：research/scripts/verus-proof-check.py 的 proof、vacuity、must-red-worlds（逐处起 Verus），
+research/scripts/e165-pin-red-examples.py 照它自己的参数切法落到 --replay 的（编 checker 档装置、逐行重放钉住的红例）；按名字与位置认（名字是它、又落在 research/scripts）。
+只读的照两份的文件头不算：verus-proof-check.py 的 toolchain（只读安装、Cargo.lock 与 cargo 缓存）、fingerprint、tool-versions 与认不出的子命令（它按用法错退 2），
+e165-pin-red-examples.py 的追加与 --check；两份的 --selftest 是提交时才跑的检查。
+最后那一道（research 构建与单测、实验复跑两道合成）只有复跑那一格是重型：--check 只点名 research-unit-tests 的不算全部实验复跑。带 --list、--list-items 或 --tool-versions 的
+（只打格名表、逐项点名的清单、工具版本）哪一道都不算；带 --write 的（生成）不算。
 另一类「提交时才跑的检查」（COMMIT_CHECK_KINDS，类名 COMMIT_CHECK_CATEGORY；用户 2026-09-28 定：扫仓的门禁与各道 lint、自证只在提交前由 gate-triage 跑的
 research/scripts/gate-staged.sh 跑）：不重型的门禁阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、
 gate-overlap.py（--list 查清单不算）、rules-lint.sh、hooks-registered.sh、stage-selftest.sh；research/scripts/gate-structure-check.py；
@@ -69,8 +81,27 @@ cargo mutants -d 指到别处的树。
   EnvironmentFile= 设的变量不带进里面那条命令）、list-past-separator（找 --list 时见到 `--` 不停）、configuration-before-subcommand-only（只收子命令之前的 --config）、mutants-configuration-ignored（cargo mutants 不看 --test-package、--test-workspace 与 .cargo/mutants.toml 的 test_workspace）、no-run-counted-as-running（`--no-run` 只编译也按跑了算）、
   shard-driver-by-name（双机分片的驱动脚本只按名字判、不读树根里的登记行，驱动自检起的那几趟也报）、
   stage-by-number（门禁阶段照旧按两位数编号认，去了编号的重型那几道认不出）、commit-checks-ignored（提交时才跑的检查这一类一样都不认）、
-  replay-cells-ignored（research 构建与复跑那一道不分格，只跑单测那一格也算全部实验复跑）。
-切词那一份的弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed（lib_shell_words.py：`python3 -` 后面的词、`bash -n` 的目标当被执行的脚本）下这里的自检同样必须判红。
+  verus-judges-by-stage-only（Verus 那一道的两份判法脚本直接起不认，干活的用法放过去）、tool-versions-counted（阶段带 --tool-versions 只打工具版本的照旧算那一道）、
+  replay-cells-ignored（research 构建与复跑那一道不分格，只跑单测那一格也算全部实验复跑）、
+  broad-mutation-ignored（广谱变异的逐条驱动不按名字认，整趟跑也放过去）、
+  broad-mutation-first-round-exemptions（「--item 点名又带 --skip-stage-two」与 --sync-tree 照第一版放过去）、
+  broad-mutation-by-name-only（驱动只按名字与位置认：指到它的符号链接、带签名行的改名拷贝、python -c 的代码里点它的都放过去）、
+  detached-run-opaque（research/scripts/detached-run.sh start … -- <命令> 不剥，里面那条重型命令放过去）、
+  detached-run-by-name-only（detached-run.sh 只按名字认：指到它的符号链接、带签名行的改名拷贝、同一条命令里先拷再起的都不剥，里面那条重型命令放过去）。
+广谱变异的逐条驱动 research/scripts/broad-mutation.py（类名「广谱变异」，kind broad-mutation）：它逐条改坏 core / checker 的源码，每条起 cargo，整趟要几天。
+  算的：凡要起 cargo 或改库目录的——整趟、--shard、--from、--package、--item 点名（点几条、带不带 --skip-stage-two 都算）、--stage-two-only、
+  --sync-tree（换全局的版本指针）、--import-results（往本机结果表里导入）；参数只认整词，argparse 认的缩写（--sum、--list）与贴着写的（--item=1）照算，接受的误拒。
+  不算的：带 --list-items、--summary、--help、-h 之一的（只读，驱动走查询那一支就返回）；--selftest 是提交时才跑的检查。
+  认驱动的三种办法（broad_mutation_driver）：名字是 broad-mutation.py 又落在 research/scripts（当前目录与命令文本里设的 PYTHONPATH 各试一遍，
+  `python3 -m broad-mutation` 由切词交成这个名字）；命令词解到的文件的真实路径落在 research/scripts/broad-mutation.py（符号链接）；
+  文件开头 BROAD_MUTATION_HEAD_BYTES 里有驱动的签名行（行首 `# broad-mutation-driver-signature:`，改名或拷走的一份）。
+  `python3 -c '…'` 的代码里出现 broad-mutation.py、或 run_module(…broad-mutation…) 的照驱动算（参数取代码之后的词）；代码里只是读它的也照算，接受的误拒。
+  它是 python 脚本，闸不读它的正文，只按这里的名字、位置、签名行与参数判。
+  罩不住的：同一条命令里先 cp / ln -s 再起（判的那一刻文件还不在）；删掉签名行的拷贝；python 代码里拼出来的路径、subprocess 起的；
+  `python3 - < 脚本` 与 heredoc 喂给 python 的代码；uv run、pypy 这类别的解释器；multi-host-run.sh --、xargs、find -exec、eval（对每一类重型都在的老口子）。
+切词那一份的弄坏开关（lib_shell_words.py）下这里的自检同样必须判红：LIB_SHELL_WORDS_BREAK=arguments-as-executed（`python3 -` 后面的词、`bash -n` 的目标
+当被执行的脚本）、LIB_SHELL_WORDS_BREAK=python-options-by-dash（python 带值选项的值被当成脚本名，`python3 -W ignore <驱动>`、`-X`、`-m`、`-c` 起的放过去）、
+LIB_SHELL_WORDS_BREAK=detached-run-force-kept（detached-run.sh 的参数不照它的 preflight 摘 --force，`-- --force <重型命令>` 放过去）。
 """
 import fnmatch, functools, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
 from typing import NamedTuple
@@ -102,6 +133,8 @@ KIND_CATEGORY = {
     "full-cargo": "全量测试", "check-sh": "全量测试",
     "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
     "e152": "E152 装置",
+    "broad-mutation": "广谱变异",
+    "verus-stage": "Verus",
 }
 # 提交时才跑的检查（用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的」）：
 # 扫仓的门禁阶段、上游的几道 lint、样本自检与阶段结构判法、研究脚本与钩子的 --selftest，都归提交前 gate-triage 跑的 research/scripts/gate-staged.sh
@@ -116,11 +149,13 @@ COMMIT_CHECK_SOP_SCRIPTS = frozenset({"doc-lint.sh", "gate-lint.sh", "shell-lint
 # 这几个目录下的脚本带 --selftest 跑，是提交时才跑的自证；.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，带不带 --selftest 都算
 COMMIT_CHECK_SELFTEST_DIRECTORIES = ("research/scripts", ".claude/hooks", ".claude/gate.d/lib")
 GATE_LIBRARY_DIRECTORY = ".claude/gate.d/lib"
-# 只查清单、不判的参数：阶段的 --list（格名表）与 --list-items（重型逐项点名的清单）、gate-overlap.py --list
-STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items"})
+# 只查清单、不判的参数：阶段的 --list（格名表）、--list-items（重型逐项点名的清单）与 --tool-versions（Verus 那一道只打工具版本）、gate-overlap.py --list
+STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--tool-versions"})
+TOOL_VERSIONS_ARGUMENT = "--tool-versions"
 # 重型的那几道阶段，按阶段文件名认（54 号名字没改，别的去了编号）；.claude/gate.d/ 下其余阶段是提交时才跑的检查
 STAGE_KIND = {"checker-tier-layer0-replay.sh": "layer0-stage", "checker-tier-qemu-device-streams.sh": "qemu-stage", "checker-tier-lkmm.sh": "herd7-stage",
-              "checker-tier-crates-mutation-replay.sh": "crates-mutation-stage", "checker-tier-research-build-and-replay.sh": "replay-all-stage"}
+              "checker-tier-crates-mutation-replay.sh": "crates-mutation-stage", "checker-tier-research-build-and-replay.sh": "replay-all-stage",
+              "checker-tier-formal-verification.sh": "verus-stage"}
 # research 构建与单测、实验复跑两道合成的那一道只有复跑那一格是重型：--check 只点名单测那一格的，算提交时才跑的检查；不给 --check、或点名里有复跑那一格的，算全部实验复跑
 RESEARCH_BUILD_AND_REPLAY_STAGE = "checker-tier-research-build-and-replay.sh"
 RESEARCH_UNIT_TESTS_CELL = "research-unit-tests"
@@ -133,7 +168,27 @@ KNOWN_SCRIPT_LOCATIONS = {
     "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
     "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
     "layer0-shard-run.sh": "research/scripts/layer0-shard-run.sh", "mutation-shard-run.sh": "research/scripts/mutation-shard-run.sh",
+    "broad-mutation.py": "research/scripts/broad-mutation.py",
 }
+# 广谱变异的逐条驱动（research/scripts/broad-mutation.py）：带这几个参数之一的只读（列项、汇总、帮助），不起 cargo、不改库目录
+BROAD_MUTATION_SCRIPT_NAME = "broad-mutation.py"
+BROAD_MUTATION_SCRIPT_DIRECTORY = "research/scripts"
+BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--help", "-h"})
+# 弄坏开关 broad-mutation-first-round-exemptions 下多放过去的（第一版的分类）：--sync-tree 当查询、「--item 加 --skip-stage-two」不算
+BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS = BROAD_MUTATION_QUERY_ARGUMENTS | {"--sync-tree"}
+# 驱动文件头的签名行（行首）：改名、拷走的一份按它认。读文件开头这么多字节找它；这份文件自己的开头里没有以它起头的行
+BROAD_MUTATION_SIGNATURE_LINE = re.compile(rb"^# broad-mutation-driver-signature:", re.MULTILINE)
+BROAD_MUTATION_HEAD_BYTES = 4096
+# python -c 的代码里点到驱动：文件名，或 runpy.run_module 按模块名起它
+BROAD_MUTATION_CODE_REFERENCE = re.compile(r"broad-mutation\.py|run_module\s*\([^)]*broad-mutation")
+# Verus 那一道（checker-tier-formal-verification.sh）的两份判法脚本：直接起时干活的用法算 verus-stage（verus_judge_use）
+VERUS_JUDGE_DIRECTORY = "research/scripts"
+VERUS_PROOF_CHECK_SCRIPT = "verus-proof-check.py"
+# 它文件头「用法」里起 Verus 的三个子命令（main 只认第一个参数）；toolchain 只读安装与缓存，fingerprint、tool-versions 只打印
+VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS = frozenset({"proof", "vacuity", "must-red-worlds"})
+E165_PIN_SCRIPT = "e165-pin-red-examples.py"
+# 它 main 里带一个值的选项（切参数时跳过那个值）；--check、--replay 里写在后面的那个定模式
+E165_PIN_OPTIONS_WITH_VALUE = frozenset({"--root", "--table", "--binary", "--memory-cap", "--threads", "--target", "--kill-bounds"})
 
 
 def heavy_test(kind, detail):
@@ -619,7 +674,8 @@ def classify_gate_stage(stage, arguments):
         match = GATE_STAGE_NUMBER_PATH.search(".claude/gate.d/" + stage)
         kind = STAGE_KIND_BY_NUMBER.get(match.group(1)) if match else None
         return heavy_test(kind, f"门禁 {stage}") if kind else None
-    if STAGE_QUERY_ARGUMENTS.intersection(arguments):
+    query_arguments = STAGE_QUERY_ARGUMENTS - {TOOL_VERSIONS_ARGUMENT} if break_is_set("tool-versions-counted") else STAGE_QUERY_ARGUMENTS
+    if query_arguments.intersection(arguments):
         return None
     kind = STAGE_KIND.get(stage)
     if (stage == RESEARCH_BUILD_AND_REPLAY_STAGE and not break_is_set("replay-cells-ignored")
@@ -881,6 +959,76 @@ def shard_driver_runs_checker_tier(arguments, directory):
     return "shard=across-machines" in tokens and package == CHECKER_PACKAGE_NAME
 
 
+def broad_mutation_driver(word, directory, environment=None):
+    """命令词是不是广谱变异的逐条驱动：是就交一句「凭什么认出来的」，不是交 None。三种办法（文件头「广谱变异的逐条驱动」那一段）：
+    名字与位置（当前目录与命令文本里设的 PYTHONPATH 各试一遍：`python3 -m broad-mutation` 交来的就是这个名字）、
+    解到的文件的真实路径（符号链接）、文件开头的签名行（改名或拷走的一份）。后两种读盘，文件不在就认不出。"""
+    expanded = os.path.expanduser(word)
+    bases = [directory]
+    for entry in ((environment or {}).get("PYTHONPATH") or "").split(os.pathsep):
+        if entry:
+            bases.append(shell_words.resolve_path(directory, os.path.expanduser(entry)))
+    location = f"{BROAD_MUTATION_SCRIPT_DIRECTORY}/{BROAD_MUTATION_SCRIPT_NAME}"
+    if os.path.basename(expanded) == BROAD_MUTATION_SCRIPT_NAME and any(located_in(expanded, base, BROAD_MUTATION_SCRIPT_DIRECTORY) for base in bases):
+        return location
+    if break_is_set("broad-mutation-by-name-only"):
+        return None
+    for base in bases:
+        resolved = shell_words.resolve_path(base, expanded)
+        if resolved is None or not os.path.isfile(resolved):
+            continue
+        real = os.path.realpath(resolved)
+        if real.endswith("/" + location):
+            return f"{word}（符号链接，指到 {location}）"
+        try:
+            with open(real, "rb") as handle:
+                head = handle.read(BROAD_MUTATION_HEAD_BYTES)
+        except OSError:
+            continue
+        if BROAD_MUTATION_SIGNATURE_LINE.search(head):
+            return f"{word}（开头带 {location} 的签名行：改名或拷走的一份）"
+    return None
+
+
+def broad_mutation_use(arguments, driver):
+    """认出是广谱变异的驱动之后按参数判：--selftest 是提交时才跑的检查；只读的（--list-items、--summary、--help）不算；其余一律算。"""
+    given = set(arguments)
+    if "--selftest" in given and not break_is_set("commit-checks-ignored"):
+        return heavy_test("commit-check-selftest", f"{driver} --selftest")
+    if break_is_set("broad-mutation-first-round-exemptions"):
+        if BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS & given or {"--item", "--skip-stage-two"} <= given:
+            return None
+    if BROAD_MUTATION_QUERY_ARGUMENTS & given:
+        return None
+    return heavy_test("broad-mutation", f"{driver}（逐条改坏 core / checker 的源码、每条起 cargo；--sync-tree 与 --import-results 改库目录里全局的版本指针与结果表；"
+                                        "只有 --list-items、--summary、--help 不算）")
+
+
+def e165_pin_mode(arguments):
+    """e165-pin-red-examples.py 的参数照它 main 的切法走一遍：--check、--replay 里写在后面的那个定模式，带值的选项跳过它的值；交 append、check 或 replay。"""
+    mode, position = "append", 0
+    while position < len(arguments):
+        argument = arguments[position]
+        if argument in ("--check", "--replay"):
+            mode = argument[2:]
+            position += 1
+        else:
+            position += 2 if argument in E165_PIN_OPTIONS_WITH_VALUE else 1
+    return mode
+
+
+def verus_judge_use(name, word, arguments, directory):
+    """Verus 那一道的两份判法脚本直接起（名字是它、又落在 research/scripts）：干活的用法交 verus-stage，只读的与别的脚本交 None。
+    --selftest 在这之前已由 classify_commit_check 认成提交时才跑的检查。弄坏开关 verus-judges-by-stage-only 下一律交 None。"""
+    if break_is_set("verus-judges-by-stage-only") or not located_in(word, directory, VERUS_JUDGE_DIRECTORY):
+        return None
+    if name == VERUS_PROOF_CHECK_SCRIPT and arguments[:1] and arguments[0] in VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS:
+        return heavy_test("verus-stage", f"research/scripts/{name} {arguments[0]}（门禁 checker-tier-formal-verification 那一格的判法，逐处起 Verus）")
+    if name == E165_PIN_SCRIPT and e165_pin_mode(arguments) == "replay":
+        return heavy_test("verus-stage", f"research/scripts/{name} --replay（门禁 checker-tier-formal-verification 的 pinned-red-examples-replay 格：编 checker 档装置、逐行重放）")
+    return None
+
+
 def classify(words, directory, environment=None):
     """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
     name, arguments = os.path.basename(words[0]), words[1:]
@@ -901,6 +1049,9 @@ def classify(words, directory, environment=None):
     commit_check = classify_commit_check(name, words[0], arguments, directory)
     if commit_check is not None:
         return commit_check
+    verus_judge = verus_judge_use(name, words[0], arguments, directory)
+    if verus_judge is not None:
+        return verus_judge
     if name == "lkmm.sh":
         return heavy_test("lkmm", ".claude/scripts/lkmm.sh")
     if name == "herd7":
@@ -925,6 +1076,14 @@ def classify(words, directory, environment=None):
     if name == "mutation-shard-run.sh":
         return None if "--selftest" in arguments else heavy_test(
             "crates-mutation-stage", "research/scripts/mutation-shard-run.sh（门禁 checker-tier-crates-mutation-replay.sh 的双机分片驱动：两台各跑一趟那一道、判 crates/mutations.tsv 的一部分）")
+    if not break_is_set("broad-mutation-ignored"):
+        driver = broad_mutation_driver(words[0], directory, environment)
+        if driver is not None:
+            return broad_mutation_use(arguments, driver)
+        if shell_words.PYTHON_NAME.match(name) and not break_is_set("broad-mutation-by-name-only"):
+            invocation = shell_words.python_invocation(arguments)
+            if invocation.code_string is not None and BROAD_MUTATION_CODE_REFERENCE.search(invocation.code_string):
+                return broad_mutation_use(arguments[invocation.script_index:], f"python -c 的代码里点了 {BROAD_MUTATION_SCRIPT_NAME}")
     if name in ("e152-file-system-benchmark", "e152-run.sh"):
         return heavy_test("e152", name)
     if name == "cargo":
@@ -1122,6 +1367,29 @@ def selftest():
             ("crates 变异表那一道（去了编号）按变异名点名", ["bash", ".claude/gate.d/checker-tier-crates-mutation-replay.sh", "--item", "加改成减"], work,
              "crates-mutation-stage"),
             ("research 构建与复跑那一道整道跑", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh"], work, "replay-all-stage"),
+            ("Verus 那一道（形式化验证）只跑一格", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--check", "verus-proof"], work, "verus-stage"),
+            ("Verus 那一道 --list 只打格名表", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--list"], work, None),
+            ("直接起 verus 验一个 crate 不算重型", ["verus", "--crate-type=lib", "--no-cheating", "src/lib.rs"], work, None),
+            ("cargo verus verify 不算重型", ["cargo", "verus", "verify", "-p", "singlefs-core"], work, None),
+            # Verus 那一道只打工具版本不算（开关 tool-versions-counted 下这一格红）
+            ("Verus 那一道 --tool-versions 只打工具版本", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--tool-versions"], work, None),
+            # 两份判法脚本直接起：干活的用法算那一道（开关 verus-judges-by-stage-only 下前五格红），只读的不算
+            ("直接起判法脚本跑必红世界", ["python3", "research/scripts/verus-proof-check.py", "must-red-worlds", "."], work, "verus-stage"),
+            ("直接起判法脚本跑空真、经 nice 与绝对路径", ["nice", "-n", "19", "python3", os.path.join(work, "research", "scripts", "verus-proof-check.py"), "vacuity", work], "/",
+             "verus-stage"),
+            ("直接起判法脚本验证明", ["python3", "research/scripts/verus-proof-check.py", "proof", "."], work, "verus-stage"),
+            ("直接起红例重放", ["python3", "research/scripts/e165-pin-red-examples.py", "--replay", "--root", "."], work, "verus-stage"),
+            ("经内存包装起红例重放、--check 写在 --replay 前面", ["bash", "research/scripts/run-with-memory-cap.sh", "4G", "python3", "research/scripts/e165-pin-red-examples.py",
+                                                            "--check", "--replay"], work, "verus-stage"),
+            ("判法脚本只打工具版本", ["python3", "research/scripts/verus-proof-check.py", "tool-versions", "."], work, None),
+            ("判法脚本核版本一致（只读安装与缓存）", ["python3", "research/scripts/verus-proof-check.py", "toolchain", "."], work, None),
+            ("判法脚本打包源码指纹", ["python3", "research/scripts/verus-proof-check.py", "fingerprint", "/tmp/claude-1000/vstd"], work, None),
+            ("红例钉表只核钉全", ["python3", "research/scripts/e165-pin-red-examples.py", "--check"], work, None),
+            ("红例钉表 --replay 写在 --check 前面、以 --check 为准", ["python3", "research/scripts/e165-pin-red-examples.py", "--replay", "--check"], work, None),
+            ("红例钉表 --root 的值碰巧叫 --replay", ["python3", "research/scripts/e165-pin-red-examples.py", "--check", "--root", "--replay"], work, None),
+            ("红例钉表追加", ["python3", "research/scripts/e165-pin-red-examples.py", "research/results/e165-sample-red-examples.tsv"], work, None),
+            ("判法脚本 --selftest 是提交时才跑的检查", ["python3", "research/scripts/verus-proof-check.py", "--selftest"], work, "commit-check-selftest"),
+            ("仓外同名的判法脚本不按名字认", ["python3", "/tmp/claude-1000/elsewhere/verus-proof-check.py", "must-red-worlds", "."], work, None),
             ("research 构建与复跑那一道只跑复跑那一格、按实验点名", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh", "--check", "experiment-replay",
                                                             "--item", "E142"], work, "replay-all-stage"),
             ("research 构建与复跑那一道两格一起点名", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh", "--check",
@@ -1313,7 +1611,8 @@ def selftest():
     if failures:
         print("    → 看 classify()、cargo_subcommand()、cargo_use()、package_specification_members()、alias_expansion()、test_invocation()、"
               "command_under_launcher()、launcher_option()、libtest_lists_only()、checker_test_targets()、test_binary_name()、"
-              "runs_compiled_code()、classify_process()、judged_by_name() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
+              "runs_compiled_code()、classify_process()、judged_by_name()、broad_mutation_driver()、broad_mutation_use()、verus_judge_use()、e165_pin_mode()、"
+              "command_under_detached_run()、lib_shell_words.py 的 detached_run_script()、detached_run_start_command()、python_invocation() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
               f"KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / LIBTEST_OPTIONS_WITH_VALUE 几张表；{BREAK_VARIABLE} 设着的话这里本来就该红")
         return 1
     print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
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
index 30b3ede6..558da609 100644
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
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑这五道：`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`checker-tier-layer0-replay` 跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），其余四道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-layer0-replay` 并行，checker-tier-crates-mutation-replay 与 `checker-tier-layer0-replay` 串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这五道在 `gate.sh` 里照各自的复用判定走：全绿标记或复用判定作数的不跑，不作数的——崩溃验证员没跑那一道、跑过之后那一道的输入或环境变了——整轮里真跑那一道，`checker-tier-layer0-replay` 照跑快档、逐条核全绿标记；它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（写回员：给它判决与条目，它自己起草规格、跑 `research/scripts/kb-spec-check.py` 到绿、再写；派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧；条目少、主 agent 自己写得清的，主 agent 自己写规格、跑 `research/scripts/replace-batch.py` 与 `bash .claude/gate.d/doc-decisions.sh --write`；按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个写回员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 doc-experiments 的 decision-links 格那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、门禁 code-source-discipline 的 mutation-tables 格红（变异表锚点腐化）→ `research/mutations/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
 | 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 那一格） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index b71e1501..fe216e76 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -29,7 +29,7 @@
 
 ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判
 
-改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 都登记着。
+改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 都登记着。
 
 ⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：
 
@@ -50,28 +50,27 @@
 | 场合 | 跑不跑 |
 |---|---|
 | 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；checker 档一样都不跑 |
-| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 `checker-tier-layer0-replay`（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 `checker-tier-layer0-replay`（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
 | 用户要求，或夜间 | checker 档全量：`checker-tier-layer0-replay` `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
 | 其余任何时候 | checker 档不跑；子 agent 只跑 harness |
 
-**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
+**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification（连同直接起它的判法脚本 `research/scripts/verus-proof-check.py` 证明、空真、必红三种用法与 `research/scripts/e165-pin-red-examples.py --replay`）与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算）。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
 
-herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：
+herd7 / LKMM、QEMU 与 Verus 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。三样都是本工程自己的阶段：
 
 | 装置 | 阶段 | 判什么 |
 |---|---|---|
 | herd7 / LKMM | `.claude/gate.d/checker-tier-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
 | QEMU 真设备 | `.claude/gate.d/checker-tier-qemu-device-streams.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |
+| Verus 形式化验证 | `.claude/gate.d/checker-tier-formal-verification.sh`（判法在 `research/scripts/verus-proof-check.py` 与 `research/scripts/e165-pin-red-examples.py`） | 逐格判什么在 `.claude/rules/verification.md`「门禁管哪一半」 |
 
-两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
+三道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
 `--staged` 那条路（几个会话共写一个仓时）同样跑它们。
 
 ## 测试与崩溃检测优先多线程
 
-- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
-- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
-- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
-- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
-- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。`checker-tier-layer0-replay` `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 `checker-tier-layer0-replay` 的那一层内存包装管。驱动只由 `checker-tier-layer0-replay` `--full` 调。门禁 `checker-tier-crates-mutation-replay.sh` 只在本地多机配置写了 `ENABLE_ACROSS_MACHINES=1` 时按行分两台（判法同一份：`research/scripts/layer0-shard-configuration-check.sh` 退 0 双机开、退 3 双机关、退 1 判不过），不认环境变量开，设了 `SINGLEFS_GATE_FULL=1` 或 `GATE_MUTATION_START_OVER=1` 时也不分：开着时先交 `research/scripts/mutation-shard-run.sh`，本机没有作数按条记录的行两堆均分、两台各判一堆，第二台的按条记录经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后本机整张判一遍（命中记录的复用、缺的现跑），全绿标记由本机写。判法退 3 时驱动报一行「双机：关」、什么都不分。这个驱动只由这一道门禁调；它与这道门禁的整套重设计归会话「里程碑3 放量 GPU加速」。
+- **写法**：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么，照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「编译与算法都要快」。本仓用 `std::thread::scope`，不为这个加依赖；每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）；线程数的环境变量没设就取 `std::thread::available_parallelism`。
+- **进度间隔**：5 分钟。单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度：每片报区间与已判的数。
+- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。`checker-tier-layer0-replay` `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 `checker-tier-layer0-replay` 的那一层内存包装管。驱动只由 `checker-tier-layer0-replay` `--full` 调。门禁 `checker-tier-crates-mutation-replay.sh` 只在本地多机配置写了 `ENABLE_ACROSS_MACHINES=1` 时按行分两台（判法同一份：`research/scripts/layer0-shard-configuration-check.sh` 退 0 双机开、退 3 双机关、退 1 判不过），不认环境变量开，设了 `SINGLEFS_GATE_FULL=1` 或 `GATE_MUTATION_START_OVER=1` 时也不分：开着时先交 `research/scripts/mutation-shard-run.sh`，本机没有作数按条记录的行两堆均分、两台各判一堆，第二台的按条记录经 `research/scripts/crates-mutation-rows.py import` 核过底座指纹与行键再导入；驱动退出之后本机整张判一遍（命中记录的复用、缺的现跑），全绿标记由本机写。判法退 3 时驱动报一行「双机：关」、什么都不分。这个驱动只由这一道门禁调；它与这道门禁的整套重设计归别的会话。
 
 **门禁管哪一半**：崩溃点重放由门禁 `checker-tier-layer0-replay` 判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑 `bash <它的根>/.claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
diff --git a/.claude/rules/verification.md b/.claude/rules/verification.md
index 3e97dc45..9120a627 100644
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
+| 崩溃放量全量 | `bash research/scripts/crash-amplification.sh`：不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），开跑先照多机配置清场、退出时复原；每项交 `crash-amplification-item.sh`：配置开了双机就两台跑、没开就一台跑，配置 `ENABLE_GPU=1` 时由 GPU 判，显卡照私有显卡配置（模板 `gpu-cards.env.example`）的先后与额度用，用不上时照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」报错退出，本仓认的用不上是第二台跑不成、点名的卡用不了、建不出表、有状态 GPU 判不了、内核展开超预算；一份库装全部分项，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library`，落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」，键与判法版本在 `crates/singlefs-checker-tier/src/crash_identity.rs`，存放布局在 `crates/singlefs-checker-tier/src/verdict_store.rs` | 用户要求或夜间 |
 
-checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。
+checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。
 
 ## 崩溃枚举用例住哪、怎么登记
 
@@ -95,6 +95,7 @@ checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutat
 | 判什么 | 谁判 |
 |---|---|
 | 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 门禁 `checker-tier-layer0-replay`；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
+| 形式化验证：恢复重放取前缀的核经 Verus 验过，E165 全量的红例都钉进快速档、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`），pinned-red-examples-complete、pinned-red-examples-replay 两格判红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` |
 | 模型对拍跑了、每段都判过 | 门禁 harness-model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
 | 崩溃枚举用例住在 checker 档、标了 `#[ignore]` 的登记了 `crash-case:`；checker 档测试文件声明模块 | 门禁 `checker-tier-layer0-replay` 开跑前跑 `research/scripts/crash-case-check.py --only placement,modules`，红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
 | 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读词表 `.claude/naming-vague-words`、还没改完的文件 `.claude/naming-vague-exclude`；test-file-names 格跑 `research/scripts/crash-case-check.py --only file-names` |
@@ -102,6 +103,6 @@ checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutat
 | harness 档一条用例一个场景 | 门禁 harness-model-differential-and-scenarios 的 one-scenario 格（跑 `research/scripts/crash-case-check.py --only one-scenario`） |
 | 变异行点名的测试跑得到：标了 `#[ignore]` 的带 `--include-ignored`、`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
 | 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
-| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |
+| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 checker 档包 `singlefs-checker-tier` 的测试（库单测、集成测试、装置二进制的内联测试）、门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification（连同直接起它的判法脚本 `research/scripts/verus-proof-check.py` 证明、空真、必红三种用法与 `research/scripts/e165-pin-red-examples.py --replay`）与 checker-tier-research-build-and-replay 的 experiment-replay 格、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置、广谱变异（`research/scripts/broad-mutation.py` 起 cargo 或改库目录的每一种用法；只有 `--list-items`、`--summary` 不算）算重型 |
 
 **它们管不到的**：harness 耗时用例标得对不对、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。
diff --git a/.claude/skills/crash-test/SKILL.md b/.claude/skills/crash-test/SKILL.md
index 84036b68..8e05dd00 100644
--- a/.claude/skills/crash-test/SKILL.md
+++ b/.claude/skills/crash-test/SKILL.md
@@ -1,6 +1,6 @@
 ---
 name: crash-test
-description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设备、崩溃点重放、模型对拍。判断写路径对不对、或要给并发改动补验证时用它。
+description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设备、崩溃点重放、模型对拍、Verus 形式化验证。判断写路径对不对、或要给并发改动补验证时用它。
 ---
 
 # 验证套件
@@ -12,9 +12,9 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设
 | 档 | 住哪 | 什么时候跑 | 怎么跑 |
 |---|---|---|---|
 | harness | `crates/singlefs-harness`：单元与集成测试 | 改了代码随时跑 | `cargo test -p singlefs-harness`（实现员经 `research/scripts/run-with-memory-cap.sh` 跑自己动到的 `--test <目标>`） |
-| checker 档 | `crates/singlefs-checker-tier`（崩溃枚举引擎、注入战役、装置二进制与它们的用例）与门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 的复跑那一格 | 默认只在提交时跑快档；全量由用户要求或夜间 | 命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）；没带，钩子 `.claude/hooks/heavy-test-guard.sh` 拒 |
+| checker 档 | `crates/singlefs-checker-tier`（崩溃枚举引擎、注入战役、装置二进制与它们的用例）与门禁 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 的复跑那一格 | 默认只在提交时跑快档；全量由用户要求或夜间 | 命令带 `SINGLEFS_HEAVY_TESTS=commit`（提交时）或 `=user-request`（用户要求）；没带，钩子 `.claude/hooks/heavy-test-guard.sh` 拒 |
 
-## 四样各落一道门禁
+## 五样各落一道门禁
 
 | 手段 | 验什么 | 阶段 | 覆盖声明的键（照 `.claude/gate-not-implemented.tsv` 抄） |
 |---|---|---|---|
@@ -22,6 +22,7 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设
 | 模型对拍 | 随机历史每一步与只住内存的理想模型比 | `.claude/gate.d/harness-model-differential-and-scenarios.sh`（harness 类，整轮门禁里跑） | `模型对拍` |
 | QEMU 真设备 | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 | `.claude/gate.d/checker-tier-qemu-device-streams.sh`（重型） | 不声明覆盖 |
 | LKMM 内存序 | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符 | `.claude/gate.d/checker-tier-lkmm.sh`，逻辑在 `.claude/scripts/lkmm.sh`（重型；缺 herd7 直接红） | 不声明覆盖 |
+| Verus 形式化验证 | 恢复重放取前缀的核经 Verus 验过（版本钉住、证明、空真、必红世界），E165 全量的红例钉进快速档逐行重放 | `.claude/gate.d/checker-tier-formal-verification.sh`，判法在 `research/scripts/verus-proof-check.py` 与 `research/scripts/e165-pin-red-examples.py`（重型） | 不声明覆盖 |
 
 阶段头部写 `# gate-covers: <键>`，键只能是 `.claude/gate-not-implemented.tsv` 里登记的（本项目登记了 `崩溃点重放`、`模型对拍`）与共享 `gate.sh` 自带的 `最终判据`、`命名纪律（shell）`，写错一个字判红。
 这个阶段这一轮跑了且通过，`gate.sh` 末尾的未实现清单才把那一项换到「由项目本地阶段覆盖」下面；报了「本次未跑」的这一轮不算覆盖。
@@ -37,4 +38,4 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 真设
 - **「没复现问题」不等于「没问题」。** 想说崩溃一致性成立，得先说清这一轮枚举了多少个崩溃点，是不是全部；快档只跑小流，全量那几条有没有作数的全绿标记看门禁 checker-tier-layer0-replay 的输出。
 - **checker 不报错，也可能是那条检查压根没实现。** 先看 `.claude/kb/invariants.md` 的状态列。
 - **判定结果读不到，这一轮就作废**，绝不能当成通过。装置读不到结束标记也一样。
-- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（门禁 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份。
+- 谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（门禁 checker-tier-layer0-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份。
diff --git a/README.md b/README.md
index 28eb30ee..019b7795 100644
--- a/README.md
+++ b/README.md
@@ -120,7 +123,7 @@ waaagh！
 提交前跑门禁：
 
 ```bash
-bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含 checker 档快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑；层 0 全量不在里面）
+bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含 checker 档快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑、Verus 形式化验证；层 0 全量不在里面）
 
 cargo test -p singlefs-harness            # harness 档：单元与集成测试，改了代码随时跑（.claude/rules/verification.md）
 SINGLEFS_HEAVY_TESTS=user-request cargo test --release -p singlefs-checker-tier --lib --tests   # checker 档快档：崩溃枚举用例住这个包的 tests/，全量那几条标 ignored；默认只在提交时由门禁 checker-tier-layer0-replay 跑
diff --git a/research/scripts/admission.py b/research/scripts/admission.py
index fa1325dd..29af7f87 100755
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -65,9 +66,10 @@
   ③ 跑的场合（重型测试前缀、内存包装）归 .claude/hooks/heavy-test-guard.sh 与 run-with-memory-cap.sh，这里不管。
 
 子命令（<根> 是被判仓的根）：
-  stage-fingerprint <根> <阶段文件名>    这一道登记输入在被判那棵树上的指纹：打「<sha256> <文件数>」
+  stage-fingerprint <根> <阶段文件名>    这一道登记输入在被判那棵树上的指纹（登记了 environment= 的连同此刻的环境）：打「<sha256> <文件数>」
   stage-marker-check <根> <阶段文件名>   这一道这批输入有没有作数的全绿标记：退 0 有（第一行 ok <路径> <时刻>），退 1 没有或过了 SINGLEFS_REUSE_HOURS
-  stage-marker-write <根> <阶段文件名>   判绿之后写这一格标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 自己调），打路径；gate-reuse 先看它再看暂存树
+  stage-marker-write <根> <阶段文件名>   判绿之后写这一格标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、
+                                         checker-tier-formal-verification 自己调），打路径；gate-reuse 先看它再看暂存树
   gate-reuse <根> <阶段文件名>   门禁的复用判定：退 0 要跑，退 10 可跳过（stage-must-run.sh 翻成它原来的 1）；
                                  模块自己出错一律退 0（按要跑处理），不许出错就跳过
   experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头）；
@@ -122,7 +124,7 @@
 强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。只越过 ①，不越过 ②；理由原样写进产物头。
 弄坏开关（只给自证用，证明那几格会红）：ADMISSION_BREAK=skip-unchanged（不比指纹）、skip-preconditions（不判前提）、
 raise-in-gate-reuse（门禁复用判定里抛异常，stage-must-run.sh 必须按要跑处理）、skip-gate-preconditions（门禁的前提不判）、
-ignore-gate-environment（门禁复用判定不比环境）、skip-build-environment（构建环境不进指纹）、ignore-linked-directories（指向目录的
+ignore-gate-environment（门禁复用判定不比环境）、environment-outside-stage-marker（environment= 不进门禁全绿标记的指纹）、skip-build-environment（构建环境不进指纹）、ignore-linked-directories（指向目录的
 符号链接照旧滤掉）、exclude-mentioned-test-targets（别的测试目标被代码点了名也减掉）、threads-by-worker-count（照旧只看
 worker_threads=1 判「只用了一个线程」，不看这一趟跑了几片）、first-definition-only（用例函数只看第一处定义）、
 ignore-attribute-without-space（`# [ignore]` 不认）、concat-include-only（只认不带路径前缀的 include!(concat!(…))）、
@@ -1121,7 +1123,8 @@ def gate_reuse(root, stage):
     return_code, _output = git_output(root, "rev-parse", "--is-inside-work-tree")
     if return_code != 0:
         return EXIT_GATE_MUST_RUN, f"判不出来：{root} 不是 git 工作树，按要跑处理"
-    # 先看这一道在被判那棵树上的全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时自己写，按登记输入的指纹分格，住 git common-dir）：
+    # 先看这一道在被判那棵树上的全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification
+    # 判绿时自己写，按登记输入的指纹分格——登记了 environment= 的连同此刻的环境——住 git common-dir）：
     # 在且没过复用上限就可跳过——崩溃验证员在工作区跑过的那一趟，整轮门禁不用再跑一遍。指纹算不出、没登记这一行的照旧往下按暂存树判。
     if not break_is_set("stage-marker-ignored"):
         try:
@@ -2206,7 +2209,8 @@ def crash_case_judging_digest_line():
 # ── 命令行 ────────────────────────────────────────────────────────────────────
 
 def stage_fingerprint(root, stage):
-    """一道门禁阶段的输入指纹：登记路径下逐文件清单 + 登记表 + 阶段脚本自己 + 登记行 + 工具链 + 构建环境。返回 (sha256, 文件数, 路径)。
+    """一道门禁阶段的输入指纹：登记路径下逐文件清单 + 登记表 + 阶段脚本自己 + 登记行 + 工具链 + 构建环境，登记了 environment= 的再加此刻环境的 sha256。
+    返回 (sha256, 文件数, 路径)。
     与 gate_reuse 比对的路径同一份（少一条输入就永远不重跑的那个理由同样成立）。"""
     stage_rows = rows_of_key(read_registration_rows(root), stage)
     if not any(row.input_paths for row in stage_rows):
@@ -2215,6 +2219,11 @@ def stage_fingerprint(root, stage):
     input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]
     registration_text = "".join(row.fingerprint_text() for row in stage_rows).encode("utf-8", "surrogateescape")
     named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
+    if not break_is_set("environment-outside-stage-marker"):
+        # 登记了 environment= 的：此刻的环境（逐条现跑，与 judge_gate_environment 同一份）进指纹，环境换了旧标记就对不上；取不出来抛 InputManifestError
+        _environment_lines, environment_digest = current_gate_environment(root, stage_rows)
+        if environment_digest is not None:
+            named_lines.append((f"<环境：{stage}>", environment_digest.encode()))
     _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
     return fingerprint, file_count, input_paths
 
@@ -2257,7 +2266,8 @@ def write_stage_marker(root, stage, fingerprint, file_count):
     marker_path = stage_marker_path(root, stage, fingerprint)
     if marker_path is None:
         raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处写")
-    text = ("# 门禁阶段的全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿之后经 research/scripts/admission.py stage-marker-write 写，按阶段与它登记输入的指纹分格；"
+    text = ("# 门禁阶段的全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification "
+            "判绿之后经 research/scripts/admission.py stage-marker-write 写，按阶段与它登记输入（登记了 environment= 的连同环境）的指纹分格；"
             "gate-reuse 按这一格判复用。不进工作树，别手改。\n"
             f"stage={stage}\ninput_hash={fingerprint}\ninput_file_count={file_count}\n"
             f"finished_utc={time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\njudged_root={os.path.abspath(root)}\n"
@@ -2948,7 +2958,7 @@ def run_selftest():
         print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
         return 1
     print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、"
-          "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、"
+          "ignore-gate-environment、environment-outside-stage-marker、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、"
           "first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、"
           "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、"
           "shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch、target-own-files-only、include-alias-subtracts、"
@@ -3085,7 +3095,17 @@ def run_gate_environment_cells(selftest, module):
         exit_code, output, _messages = reuse("57-demo.sh", base_tree)
         selftest.expect("输入与环境都没变时判可跳过，依据句带出 herd7 的版本", exit_code == 1 and "7.57-selftest-1" in output,
                         f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = run_quietly([sys.executable, module, "stage-marker-write", work, "57-demo.sh"], with_tools)
+        selftest.expect("登记了 environment= 的门禁判绿之后写全绿标记", exit_code == 0 and os.path.isfile(output.strip()), f"退 {exit_code}，stdout「{output.strip()}」")
         write_executable(fake_herd7, FAKE_HERD7_SCRIPT % "2")
+        # 环境进全绿标记的指纹：只换了 herd7、输入一字没改，判绿时写的标记不作数（开关 environment-outside-stage-marker 下这一格红）
+        exit_code, output, _messages = run_quietly([sys.executable, module, "stage-marker-check", work, "57-demo.sh"], with_tools)
+        selftest.expect("herd7 的版本变了，输入没变时写的全绿标记不作数", exit_code == 1 and "没有全绿标记" in output,
+                        f"退 {exit_code}，stdout「{output.strip()}」（弄坏开关 environment-outside-stage-marker 下就是这样：环境没进标记的指纹）")
+        exit_code, _output, _messages = run_quietly([sys.executable, module, "stage-marker-check", work, "57-demo.sh"],
+                                                    dict(with_tools, **{BREAK_VARIABLE: "environment-outside-stage-marker"}))
+        selftest.expect("弄坏开关 environment-outside-stage-marker 下「herd7 换了、旧标记不作数」那一格红（旧标记照样作数）", exit_code == 0,
+                        f"弄坏之后仍退 {exit_code}：这一格分不出环境进没进标记的指纹")
         exit_code, output, _messages = reuse("57-demo.sh", base_tree)
         selftest.expect("herd7 的版本变了判要跑，依据句带出记下的与现在的两个版本",
                         exit_code == 0 and "不同" in output and "7.57-selftest-1" in output and "7.57-selftest-2" in output,
diff --git a/research/scripts/e165-pin-red-examples.py b/research/scripts/e165-pin-red-examples.py
new file mode 100755
index 00000000..fc71a176
--- /dev/null
+++ b/research/scripts/e165-pin-red-examples.py
@@ -0,0 +1,607 @@
+#!/usr/bin/env python3
+# admission: always 每一次调都判此刻的红例账、钉住的表与变异表，上一次的结论不替这一次作保
+# run-condition: none 追加与 --check 只读写点名的文件；--replay 要的 cargo 与装置由它自己判，缺了判红或报本次未跑
+"""E165 红例钉进快速档：把全量那一档红例账里的每个红例按输入原样钉进 .claude/gate.d/e165-pinned-red-examples.tsv，并判钉得全不全、今天还绿不绿。
+
+用户 2026-10-01 定「全量形式化红了的例子 可以加到快速形式化中」「这个应该在每一次跑完之后就加入 门禁来控制比较好」
+（调度记录 records/2026-10-01-里程碑三形式化接入.md 第三节）。表的格式写在表自己的文件头。
+
+用法（<根> 默认取这份脚本往上两级的仓；--table 默认 <根>/.claude/gate.d/e165-pinned-red-examples.tsv）：
+  e165-pin-red-examples.py [--root 根] [--table 表] [--kill-bounds 产物]… <红例账…>
+      追加：账里还没钉的行按（① 输入指纹, ⑥ 变异）追加进表；同一个（①, ⑥）已在表里的不重复，同一行账（⑤ 出处相同、① 相同）已钉过的也不重复。
+      账里没有反例状态的行（序号 none，驱动报 no_single_state_differs）钉不了，只数不钉。
+      ⑥ 按这一刻的变异表换名：tsv-<行号> 取 crates/mutations.tsv 那一行的变异名，Z<n> 取 research/mutations/e165_replay_bounded_enumeration_custom.tsv
+      第 2 列那一行的变异名（要以 Z<n> 起头）；给了 --kill-bounds（第二段的产物）就再核一遍产物里那个变体那一行的 title= 与换出的名字相同（空格记成 _），
+      不同就一行都不钉（表在跑完之后被改过，行号已经对不上）。
+      同一个（①, ⑥）在表里的 ⑦ 与账里的不同：一行都不钉、判红。
+      写：读表时记 sha256，同目录临时文件写完、改名换上之前再算一次，变了就不写（别的会话刚写过，重跑）；写完回读，新钉的每一行都要在。
+  e165-pin-red-examples.py --check [--root 根] [--table 表] [<红例账…>]
+      门禁 pinned-red-examples-complete 格：不给账就取 <根>/research/results/e165-*-red-examples.tsv 全部。账里每一行（有反例状态的）都要钉着：
+      表里有一行 ① 等于这一行算出的指纹、⑦ 等于账的第 12 列，且 ⑤ 就是这一行账或 ⑥ 等于这一行按今天的变异表换出的名字。
+      表自己也判：每行八段、① 是 64 位十六进制、① 等于拿 ②③⑦ 重算的指纹、（①, ⑥）不重复。少一行判红，出路给补进去的命令。一份账都没有退 77。
+  e165-pin-red-examples.py --replay [--root 根] [--table 表] [--binary 可执行文件] [--memory-cap 上限|none] [--threads N] [--target 编译目录]
+      门禁 pinned-red-examples-replay 格：先判每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）还在它的表里（第 1 列等于那个名字的数据行），
+      再用装置 e165_replay_bounded_enumeration --pinned-cases <表> 在今天的实现上逐行判：stdout 里以「E7RESULT name=state 」起头的行
+      条数等于表的数据行数、第 i 行与第 i 个数据行的 ⑦ 逐字节相同、装置退 0（约定：每行一行结果，有行对不上或指纹漂了退非 0）。
+      不给 --binary 时：装置源码（crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs）里没有 --pinned-cases 就退 77
+      （变异那一半照判，红了照红）；有就 cargo build --release --locked -p singlefs-checker-tier --bin e165_replay_bounded_enumeration
+      （CARGO_TARGET_DIR 默认 ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-e165-pinned-target）再跑。
+      装置经 research/scripts/run-with-memory-cap.sh（上限默认 $GATE_E165_PINNED_MEMORY_MAX，再默认 2G）与 capped.sh N 起；--memory-cap none 直接起（样本的桩用）；
+      经包装起而包装自己退 250–254 的，判「没跑成」：退 1，✗ 行写「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」，
+      → 照包装文件头「退出码」一段，不指向变异行或实现；
+      --binary 以 .sh 结尾的经 bash 起（样本目录里的桩不带执行位）。
+  e165-pin-red-examples.py --selftest
+      在临时仓里造账、变异表、产物与桩装置，逐格判；弄坏开关逐个打开，每个都要让至少一格判错。
+退出码：0 绿（追加：钉好了或没有要钉的）；1 红；2 用法错；77 本次未跑（--check 没有账、--replay 装置不认 --pinned-cases）。
+
+弄坏开关 E165_PIN_RED_EXAMPLES_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）：
+  pin-twice              追加不去重                        pin-again 那一格判错
+  titles-unchecked       不核 --kill-bounds 的 title        titles-mismatch 那一格判错
+  no-recheck-before-write  改名换上之前不复核 sha256       race 那一格判错
+  check-ignores-missing  --check 不报没钉的行               check-missing 那一格（样本 pinned-red-examples-complete-red）判绿
+  fingerprint-unchecked  --check 不拿 ②③⑦ 重算 ①         check-tampered 那一格判绿
+  variant-unchecked      --replay 不判变异还在不在          replay-variant-missing 那一格（样本 pinned-red-examples-replay-variant-red）判绿
+  replay-ignores-output  --replay 只看装置退出码，不比 ⑦    replay-mismatch 那一格（样本 pinned-red-examples-replay-red）判绿
+  wrapper-exit-as-replay-red  --replay 不认包装自己的 250–254，照装置退非 0 报回归   replay-wrapper-exit 那一格（自证里的假包装退 252）判错
+"""
+import glob
+import hashlib
+import os
+import re
+import subprocess
+import sys
+import tempfile
+
+sys.dont_write_bytecode = True
+from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
+import os as preflight_os, sys as preflight_sys  # noqa: E402
+# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
+preflight_sys.dont_write_bytecode = True
+preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
+from project_preflight import preflight  # noqa: E402
+
+SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
+DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
+TABLE = ".claude/gate.d/e165-pinned-red-examples.tsv"
+LEDGER_GLOB = "research/results/e165-*-red-examples.tsv"
+CRATES_TABLE = "crates/mutations.tsv"
+CUSTOM_TABLE = "research/mutations/e165_replay_bounded_enumeration_custom.tsv"
+DEVICE_SOURCE = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
+DEVICE_BINARY = "e165_replay_bounded_enumeration"
+DEVICE_FLAG = "--pinned-cases"
+RESULT_PREFIX = "E7RESULT name=state "
+LEDGER_COLUMNS, TABLE_COLUMNS = 13, 8
+EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
+BREAK = {item for item in os.environ.get("E165_PIN_RED_EXAMPLES_BREAK", "").split(",") if item}
+MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
+# 内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」一段）：经它起时装置这一次没跑成，输出不算判定
+WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上", 253: "超过限时", 254: "被总上限挤掉"}
+
+
+class Red(Exception):
+    def __init__(self, message, howto):
+        super().__init__(message)
+        self.message, self.howto = message, howto
+
+
+class NotRun(Exception):
+    pass
+
+
+def report_red(message, howto):
+    print(f"  ✗ {message}")
+    print(f"     → 怎么办：{howto}")
+
+
+def file_digest(path):
+    return hashlib.sha256(open(path, "rb").read()).hexdigest()
+
+
+def state_before_outcome(line):
+    """逐态行里 state= 的值（整行最后一段），截到 ;first= 之前；认不出交 None。"""
+    match = re.search(r"(?:^| )state=(\S+)$", line)
+    return match.group(1).split(";first=")[0] if match else None
+
+
+def input_fingerprint(domain, size, original_line):
+    state = state_before_outcome(original_line)
+    if state is None:
+        return None
+    return hashlib.sha256(f"{domain}|{size}|{state}".encode("utf-8")).hexdigest()
+
+
+# ── 读账、读表、换变异名 ──
+
+def read_ledger(path):
+    rows = []
+    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
+        if not raw.strip() or raw.startswith("#"):
+            continue
+        fields = raw.split("\t")
+        if len(fields) != LEDGER_COLUMNS:
+            raise Red(f"红例账 {path} 第 {number} 行是 {len(fields)} 段，要 {LEDGER_COLUMNS} 段", "账由 E165 第二段驱动写，格式在它的表头；手改坏了从 git 找回")
+        rows.append({"number": number, "fields": fields})
+    return rows
+
+
+def mutation_names(root, relative):
+    """变异表的 {物理行号: 变异名}（数据行）。"""
+    path = os.path.join(root, relative)
+    if not os.path.isfile(path):
+        raise Red(f"变异表 {relative} 不在", "它随仓走；被删了从 git 找回")
+    names = {}
+    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
+        if raw.strip() and not raw.startswith("#"):
+            names[number] = raw.split("\t")[0]
+    return names
+
+
+def kill_bound_titles(paths):
+    """第二段产物里每个 (变体, 表行) 的 title=（空格记成 _）。"""
+    titles = {}
+    for path in paths:
+        for line in open(path, encoding="utf-8").read().split("\n"):
+            if not line.startswith("E7RESULT name=kill_bound "):
+                continue
+            item = re.search(r" item=(\S+)", line)
+            row = re.search(r" table_row=(\d+)", line)
+            title = re.search(r" title=(\S+)", line)
+            if item and row and title:
+                titles[(item.group(1), int(row.group(1)))] = title.group(1)
+    return titles
+
+
+class VariantNames:
+    def __init__(self, root):
+        self.root, self.cache = root, {}
+
+    def names_of(self, relative):
+        if relative not in self.cache:
+            self.cache[relative] = mutation_names(self.root, relative)
+        return self.cache[relative]
+
+    def variant(self, ledger_path, row, titles=None):
+        """账的第 1、2 列换成「<表>#<名>」。"""
+        item, rows_text = row["fields"][0], row["fields"][1]
+        first_row = rows_text.split("+")[0]
+        where = f"{os.path.basename(ledger_path)}:{row['number']}"
+        if not first_row.isdigit():
+            raise Red(f"{where} 第 2 列「{rows_text}」认不出表行", "账的第 2 列是表行号（几行同一个改法用 + 连）")
+        line_number = int(first_row)
+        match = re.fullmatch(r"tsv-(\d+)", item)
+        if match:
+            table = CRATES_TABLE
+            if int(match.group(1)) != line_number:
+                raise Red(f"{where}：变体 {item} 与第 2 列的表行 {rows_text} 对不上", "账写坏了；从 git 找回原样")
+        elif re.fullmatch(r"Z\d+", item):
+            table = CUSTOM_TABLE
+        else:
+            raise Red(f"{where}：变体「{item}」既不是 tsv-<行号> 也不是 Z<n>", "只认 E165 第二段驱动写的两种变体名")
+        name = self.names_of(table).get(line_number)
+        if name is None or (table == CUSTOM_TABLE and not name.startswith(item + " ")):
+            raise Red(f"{where}：{table} 第 {line_number} 行不是变体 {item} 的变异（读到 {name!r}）",
+                      "表在跑完之后被改过、行号对不上：拿跑那一趟时的表（git show <那次提交>:<表>）核出变异名，或重跑那一趟再钉")
+        if titles is not None and "titles-unchecked" not in BREAK:
+            title = titles.get((item, line_number))
+            if title is None or title != name.replace(" ", "_"):
+                raise Red(f"{where}：变体 {item} 在第二段产物里的 title 是 {title!r}，今天的 {table} 第 {line_number} 行是 {name!r}",
+                          "表在跑完之后被改过：拿跑那一趟时的表核出变异名，或重跑那一趟再钉")
+        return f"{table}#{name}"
+
+
+def pinned_fields(ledger_path, row, variant):
+    fields = row["fields"]
+    domain, size, ordinal, original, mutant = fields[2], fields[3], fields[4], fields[11], fields[12]
+    fingerprint = input_fingerprint(domain, size, original)
+    if fingerprint is None:
+        raise Red(f"{os.path.basename(ledger_path)}:{row['number']} 第 12 列认不出 state=", "账的第 12 列是原实现的逐态行（E7RESULT name=state …），手改坏了从 git 找回")
+    return [fingerprint, domain, size, ordinal, f"{os.path.basename(ledger_path)}:{row['number']}", variant, original, mutant]
+
+
+def has_example(row):
+    return row["fields"][4] != "none" and row["fields"][11] != ""
+
+
+def read_table(path):
+    if not os.path.isfile(path):
+        raise Red(f"钉住的表 {path} 不在", "它随仓走，表头写着格式；被删了从 git 找回")
+    text = open(path, encoding="utf-8").read()
+    rows = []
+    for number, raw in enumerate(text.split("\n"), 1):
+        if not raw.strip() or raw.startswith("#"):
+            continue
+        rows.append({"number": number, "fields": raw.split("\t")})
+    return text, rows
+
+
+def table_problems(rows):
+    problems, seen = [], set()
+    for row in rows:
+        fields = row["fields"]
+        if len(fields) != TABLE_COLUMNS:
+            problems.append(f"第 {row['number']} 行是 {len(fields)} 段，要 {TABLE_COLUMNS} 段")
+            continue
+        if not re.fullmatch(r"[0-9a-f]{64}", fields[0]):
+            problems.append(f"第 {row['number']} 行的 ① 不是 64 位小写十六进制")
+        elif input_fingerprint(fields[1], fields[2], fields[6]) != fields[0] and "fingerprint-unchecked" not in BREAK:
+            problems.append(f"第 {row['number']} 行的 ① 与拿 ②③⑦ 重算的指纹不同（手改过这一行）")
+        key = (fields[0], fields[5])
+        if key in seen:
+            problems.append(f"第 {row['number']} 行的（①, ⑥）与前面一行重复")
+        seen.add(key)
+    return problems
+
+
+# ── 追加 ──
+
+def append(root, table, ledgers, kill_bounds):
+    titles = kill_bound_titles(kill_bounds) if kill_bounds else None
+    text, existing = read_table(table)
+    problems = table_problems(existing)
+    if problems:
+        raise Red(f"钉住的表 {table} 本身不对：" + "；".join(problems[:5]), "表只由这份脚本追加，不手改；手改坏了从 git 找回再钉")
+    digest_at_read = file_digest(table)
+    by_key = {(row["fields"][0], row["fields"][5]): row["fields"] for row in existing}
+    by_origin = {(row["fields"][4], row["fields"][0]) for row in existing}
+    names = VariantNames(root)
+    new_rows, already, without_example, conflicts, total = [], 0, 0, [], 0
+    for ledger in ledgers:
+        for row in read_ledger(ledger):
+            total += 1
+            if not has_example(row):
+                without_example += 1
+                continue
+            fields = pinned_fields(ledger, row, names.variant(ledger, row, titles))
+            key = (fields[0], fields[5])
+            if "pin-twice" not in BREAK:
+                if key in by_key:
+                    if by_key[key][6] != fields[6]:
+                        conflicts.append(f"{fields[4]}（{fields[5]}）：表里 ⑦ 是 {by_key[key][6][:120]}…，账里是 {fields[6][:120]}…")
+                    else:
+                        already += 1
+                    continue
+                if (fields[4], fields[0]) in by_origin:
+                    already += 1
+                    continue
+            new_rows.append(fields)
+            by_key[key] = fields
+            by_origin.add((fields[4], fields[0]))
+    if conflicts:
+        for conflict in conflicts[:5]:
+            print(f"     {conflict}")  # gate-lint:detail
+        raise Red(f"{len(conflicts)} 行账与表里同一个（输入, 变异）的原实现逐态行不同，一行都没钉",
+                  "原实现在这个输入上的行为变了：先弄清是实现改了行为（要三方认）还是账跑在别的树上，再决定改表还是重跑")
+    if not new_rows:
+        print(f"  ✓ {len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态，没有要新钉的；表里 {len(existing)} 行")
+        return EXIT_GREEN
+    new_text = (text if text.endswith("\n") or not text else text + "\n") + "".join("\t".join(fields) + "\n" for fields in new_rows)
+
+    def unchanged_since_read():
+        if "no-recheck-before-write" in BREAK:
+            return
+        if current_digest(table) != digest_at_read:
+            raise ChangedSinceRead()
+
+    try:
+        replace_file_contents_by_rename(table, new_text, check_before_rename=unchanged_since_read)
+    except ChangedSinceRead:
+        raise Red(f"读到写之间 {table} 被别人改过，一行都没写", "别的会话刚写过这张表：重跑同一条命令")
+    except ReplaceRefused as refused:
+        raise Red(f"没换上：{str(refused).splitlines()[0]}", str(refused).splitlines()[-1] if len(str(refused).splitlines()) > 1 else "看上面的原因")
+    _text, after = read_table(table)
+    present = {"\t".join(row["fields"]) for row in after}
+    lost = [fields for fields in new_rows if "\t".join(fields) not in present]
+    if lost:
+        raise Red(f"写完回读：新钉的 {len(new_rows)} 行里 {len(lost)} 行不在表里", "别的会话在改名换上之后又写过；重跑同一条命令")
+    print(f"  ✓ 钉进 {len(new_rows)} 行（{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态），回读都在；表里现有 {len(after)} 行")
+    return EXIT_GREEN
+
+
+class ChangedSinceRead(Exception):
+    pass
+
+
+def current_digest(path):
+    return file_digest(path)
+
+
+# ── 格一：账都钉着 ──
+
+def check(root, table, ledgers):
+    if not ledgers:
+        ledgers = sorted(glob.glob(os.path.join(root, LEDGER_GLOB)))
+    if not ledgers:
+        raise NotRun(f"{LEDGER_GLOB} 一份红例账都没有：这一格无对象可判")
+    _text, rows = read_table(table)
+    problems = table_problems(rows)
+    if problems:
+        raise Red(f"钉住的表本身不对：" + "；".join(problems[:5]), "表只由 research/scripts/e165-pin-red-examples.py 追加；手改坏了从 git 找回再钉")
+    by_fingerprint = {}
+    for row in rows:
+        by_fingerprint.setdefault(row["fields"][0], []).append(row["fields"])
+    names = VariantNames(root)
+    missing, counted, without_example = [], 0, 0
+    for ledger in ledgers:
+        for row in read_ledger(ledger):
+            if not has_example(row):
+                without_example += 1
+                continue
+            counted += 1
+            origin = f"{os.path.basename(ledger)}:{row['number']}"
+            fingerprint = input_fingerprint(row["fields"][2], row["fields"][3], row["fields"][11])
+            candidates = [fields for fields in by_fingerprint.get(fingerprint, []) if fields[6] == row["fields"][11]]
+            pinned = any(fields[4] == origin for fields in candidates)
+            if not pinned and candidates:
+                try:
+                    variant = names.variant(ledger, row)
+                except Red:
+                    variant = None
+                pinned = any(fields[5] == variant for fields in candidates)
+            if not pinned:
+                missing.append((ledger, origin))
+    if missing and "check-ignores-missing" not in BREAK:
+        for _ledger, origin in missing[:10]:
+            print(f"     没钉：{origin}")  # gate-lint:detail
+        ledgers_to_pin = sorted({os.path.relpath(ledger, root) for ledger, _origin in missing})
+        raise Red(f"{len(ledgers)} 份红例账 {counted} 行里 {len(missing)} 行没钉进 {os.path.relpath(table, root)}",
+                  f"补进去：python3 research/scripts/e165-pin-red-examples.py {' '.join(ledgers_to_pin)}（已钉的不重复）")
+    print(f"  ✓ {len(ledgers)} 份红例账 {counted} 行都钉在 {os.path.relpath(table, root)}（{without_example} 行没有反例状态、不钉）；表里 {len(rows)} 行，① 都与 ②③⑦ 重算的相同")
+    return EXIT_GREEN
+
+
+# ── 格二：变异还在、今天的实现上绿 ──
+
+def replay(root, table, options):
+    _text, rows = read_table(table)
+    problems = table_problems(rows)
+    if problems:
+        raise Red("钉住的表本身不对：" + "；".join(problems[:5]), "表只由 research/scripts/e165-pin-red-examples.py 追加；手改坏了从 git 找回再钉")
+    if not rows:
+        raise NotRun(f"{os.path.relpath(table, root)} 一行都没有：这一格无对象可判")
+    missing_variants = []
+    if "variant-unchecked" not in BREAK:
+        tables = {}
+        for row in rows:
+            for part in row["fields"][5].split("+"):
+                relative, _hash, name = part.partition("#")
+                if relative not in tables:
+                    path = os.path.join(root, relative)
+                    tables[relative] = set(mutation_names(root, relative).values()) if os.path.isfile(path) else None
+                if tables[relative] is None or name not in tables[relative]:
+                    missing_variants.append(f"第 {row['number']} 行：{part}")
+    if missing_variants:
+        for item in missing_variants[:10]:
+            print(f"     不在了：{item}")  # gate-lint:detail
+        raise Red(f"{len(rows)} 行里 {len(missing_variants)} 处点名的「改回去就红」的变异不在它的表里了",
+                  "变异行被删或改了名：找回那一行（改法被悄悄撤回时它要先响），确是改名就照新名改这张表（要写明为什么）")
+    binary = options["binary"]
+    if binary is None:
+        source = os.path.join(root, DEVICE_SOURCE)
+        if not os.path.isfile(source) or DEVICE_FLAG not in open(source, encoding="utf-8").read():
+            raise NotRun(f"{len(rows)} 行点名的变异都还在；装置 {DEVICE_SOURCE} 还不认 {DEVICE_FLAG}（第 4 批 3-6 才加），逐行重放这一半本次未跑")
+        environment = dict(os.environ, CARGO_TARGET_DIR=options["target"])
+        command = ["nice", "-n", "19", "cargo", "build", "--release", "--locked", "-p", "singlefs-checker-tier", "--bin", DEVICE_BINARY]
+        completed = subprocess.run(command, cwd=root, capture_output=True, text=True, env=environment)
+        if completed.returncode != 0:
+            raise Red(f"编装置没成（{' '.join(command[3:])} 退 {completed.returncode}）：{' | '.join(completed.stderr.strip().split(chr(10))[-4:])}",
+                      "先让它在平时的工具链上编得过；缓存缺包 cargo fetch --locked")
+        binary = os.path.join(options["target"], "release", DEVICE_BINARY)
+    # 以 .sh 结尾的（样本的桩）经 bash 起：样本目录里的文件不带执行位（script-modes.sh 不看 fixtures/）
+    command = ["bash", binary, DEVICE_FLAG, os.path.abspath(table)] if binary.endswith(".sh") else [binary, DEVICE_FLAG, os.path.abspath(table)]
+    wrapped = options["memory_cap"] != "none"
+    if wrapped:
+        command = ["bash", MEMORY_CAP_RUNNER, options["memory_cap"],
+                   "bash", os.path.join(SCRIPT_DIRECTORY, "capped.sh"), str(options["threads"]), *command]
+    environment = dict(os.environ, RUN_WITH_MEMORY_CAP_PARALLEL_REASON="门禁里逐行重放钉住的红例，与同一趟别的格各用各的上限",
+                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="门禁的上限按钉住的表最大时给，表只涨不缩")
+    completed = subprocess.run(command, cwd=root, capture_output=True, text=True, env=environment)
+    if wrapped and completed.returncode in WRAPPER_OWN_EXITS and "wrapper-exit-as-replay-red" not in BREAK:
+        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
+        raise Red(f"没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 {completed.returncode}（{WRAPPER_OWN_EXITS[completed.returncode]}，上限 {options['memory_cap']}），"
+                  f"钉住的 {len(rows)} 行这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
+                  "照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：250 是装置要的比上限多，把 GATE_E165_PINNED_MEMORY_MAX（或 --memory-cap）设大再跑；"
+                  "251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；"
+                  "253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑；不拿掉包装跑，不改钉住的表、变异行或实现")
+    results = [line for line in completed.stdout.split("\n") if line.startswith(RESULT_PREFIX)]
+    mismatches = [] if "replay-ignores-output" in BREAK else \
+        [f"第 {row['number']} 行（{row['fields'][4]}）：要 {row['fields'][6][:160]}，装置打的是 {(results[index] if index < len(results) else '（没打）')[:160]}"
+         for index, row in enumerate(rows) if index >= len(results) or results[index] != row["fields"][6]]
+    if completed.returncode != 0 or mismatches or (len(results) != len(rows) and "replay-ignores-output" not in BREAK):
+        for mismatch in mismatches[:5]:
+            print(f"     {mismatch}")  # gate-lint:detail
+        raise Red(f"钉住的 {len(rows)} 行在今天的实现上：装置退 {completed.returncode}、打了 {len(results)} 行结果、{len(mismatches)} 行与 ⑦ 不同"
+                  f"{'（' + completed.stderr.strip().split(chr(10))[-1][:200] + '）' if completed.stderr.strip() else ''}",
+                  "今天的实现在钉住的输入上与原实现不同：是改动引入的回归就修实现；装置报指纹漂了就先查装置的枚举次序与状态编码变没变")
+    print(f"  ✓ 钉住的 {len(rows)} 行：点名的变异都还在它们的表里；装置 {DEVICE_FLAG} 逐行打的结果都与 ⑦ 逐字节相同")
+    return EXIT_GREEN
+
+
+# ── 命令行 ──
+
+def main(arguments):
+    if arguments[:1] == ["--selftest"]:
+        return selftest()
+    options = {"root": DEFAULT_ROOT, "table": None, "kill_bounds": [], "binary": None, "threads": int(os.environ.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1),
+               "memory_cap": os.environ.get("GATE_E165_PINNED_MEMORY_MAX") or "2G",
+               "target": os.path.join(os.environ.get("GATE_CROSS_RUN_TMPDIR") or os.environ.get("TMPDIR") or "/tmp", "singlefs-e165-pinned-target")}
+    mode, ledgers, position = "append", [], 0
+    valued = {"--root": "root", "--table": "table", "--binary": "binary", "--memory-cap": "memory_cap", "--threads": "threads", "--target": "target"}
+    while position < len(arguments):
+        argument = arguments[position]
+        if argument in ("--check", "--replay"):
+            mode = argument[2:]
+            position += 1
+        elif argument in valued or argument == "--kill-bounds":
+            if position + 1 >= len(arguments):
+                report_red(f"{argument} 后面缺值", "看文件头的用法一节")
+                return EXIT_USAGE
+            if argument == "--kill-bounds":
+                options["kill_bounds"].append(arguments[position + 1])
+            else:
+                options[valued[argument]] = int(arguments[position + 1]) if argument == "--threads" else arguments[position + 1]
+            position += 2
+        elif argument.startswith("--"):
+            report_red(f"认不出的选项 {argument}", "看文件头的用法一节")
+            return EXIT_USAGE
+        else:
+            ledgers.append(arguments[position])
+            position += 1
+    root = os.path.abspath(options["root"])
+    table = os.path.abspath(options["table"] or os.path.join(root, TABLE))
+    if mode == "append" and not ledgers:
+        report_red("追加要至少一份红例账", "例：python3 research/scripts/e165-pin-red-examples.py research/results/<某一趟>-red-examples.tsv")
+        return EXIT_USAGE
+    missing = [ledger for ledger in ledgers + options["kill_bounds"] if not os.path.isfile(ledger)]
+    if missing:
+        report_red(f"点名的文件不在：{' '.join(missing)}", "路径相对当前目录；红例账在 research/results/ 下")
+        return EXIT_USAGE
+    try:
+        if mode == "append":
+            return append(root, table, [os.path.abspath(ledger) for ledger in ledgers], options["kill_bounds"])
+        if mode == "check":
+            return check(root, table, [os.path.abspath(ledger) for ledger in ledgers])
+        return replay(root, table, options)
+    except Red as red:
+        report_red(red.message, red.howto)
+        return EXIT_RED
+    except NotRun as not_run:
+        print(f"  ! {not_run}")
+        return EXIT_NOT_RUN
+
+
+# ── 自证 ──
+
+def selftest():
+    global current_digest, MEMORY_CAP_RUNNER
+    cases, wrong = 0, []
+    import contextlib
+    import io
+    with tempfile.TemporaryDirectory(prefix="e165-pin-selftest-") as work:
+        root = os.path.join(work, "root")
+        for directory in (".claude/gate.d", "crates", "research/mutations", "research/results"):
+            os.makedirs(os.path.join(root, directory))
+
+        def write(relative, text):
+            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
+                handle.write(text)
+
+        crates_table = "# 表头\n红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n红的两条\tcrates/x.rs\tc\td\t-p x\tt\n"
+        write(CRATES_TABLE, crates_table)
+        write(CUSTOM_TABLE, "# 表头\nZ1 自造的\tcrates/x.rs\te\tf\n")
+        header = "# 自证用的钉住的表\n"
+        write(TABLE, header)
+
+        def state_line(domain, ordinal, state, first):
+            return f"E7RESULT name=state domain={domain} ring_slots=64 n=2 ordinal={ordinal} counters=states state={state};first={first}"
+
+        ledger_rows = [
+            ["tsv-2", "2", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:b")],
+            ["tsv-3", "3+9", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:c")],
+            ["Z1", "2", "X", "2", "5", "outcome", "outcome", "true", "true", "true", "cmd", state_line("X", 5, "shape=2c0", "returned:d"), state_line("X", 5, "shape=2c0", "returned:e")],
+            ["tsv-3", "3+9", "X", "2", "none", "no_single_state_differs", "none", "false", "false", "false", "", "", ""],
+        ]
+        ledger = os.path.join(root, "research/results/e165-selftest-red-examples.tsv")
+        write(os.path.relpath(ledger, root), "# 账头\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
+        kill_bounds = os.path.join(root, "research/results/e165-selftest-kill-bounds.out")
+        good_titles = ("E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red title=红的一条 complete=yes\n"
+                       "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red title=红的两条 complete=yes\n"
+                       f"E7RESULT name=kill_bound item=Z1 table={CUSTOM_TABLE} table_row=2 outcome=red title=Z1_自造的 complete=yes\n")
+        write(os.path.relpath(kill_bounds, root), good_titles)
+        stub = os.path.join(work, "stub-device.sh")
+        with open(stub, "w", encoding="utf-8") as handle:
+            handle.write("#!/usr/bin/env bash\n# 桩装置：--pinned-cases <表> 逐个数据行打 ⑦；STUB_MODE=wrong 把最后一行打错，=fail 退 1\n"
+                         "[[ \"$1\" == --pinned-cases ]] || exit 2\n"
+                         "rows=$(grep -v '^#' \"$2\" | grep -c .)\n"
+                         "grep -v '^#' \"$2\" | grep . | cut -f7 | { if [[ \"${STUB_MODE:-}\" == wrong ]]; then sed '$ s/returned:/returned:WRONG/'; else cat; fi; }\n"
+                         "echo \"E7RESULT name=pinned_cases rows=$rows\"\n"
+                         "[[ \"${STUB_MODE:-}\" == fail ]] && exit 1\nexit 0\n")
+        os.chmod(stub, 0o755)
+
+        def expect(label, arguments, want_exit, want_text, environment=None):
+            nonlocal cases
+            cases += 1
+            saved = dict(os.environ)
+            os.environ.update(environment or {})
+            buffer = io.StringIO()
+            try:
+                with contextlib.redirect_stdout(buffer):
+                    got = main(["--root", root, *arguments])
+            finally:
+                os.environ.clear()
+                os.environ.update(saved)
+            output = buffer.getvalue()
+            if got != want_exit or want_text not in output:
+                wrong.append(label)
+                print(f"    {label} 这一格判错：期望退 {want_exit}、输出含「{want_text}」，实测退 {got}：{output.strip()[-300:]}")  # gate-lint:detail
+            return output
+
+        table_path = os.path.join(root, TABLE)
+        expect("check-empty-table", ["--check", ledger], 1, "3 行里 3 行没钉进")
+        write(os.path.relpath(kill_bounds, root) + ".wrong", good_titles.replace("title=红的两条", "title=别的变异"))
+        expect("titles-mismatch", ["--kill-bounds", kill_bounds + ".wrong", ledger], 1, "title 是")
+        # 读到写之间被改过：让复核那一次读到不同的 sha256
+        original_digest = current_digest
+        current_digest = lambda path: "0" * 64
+        expect("race", ["--kill-bounds", kill_bounds, ledger], 1, "被别人改过")
+        current_digest = original_digest
+        if open(table_path, encoding="utf-8").read() != header:
+            wrong.append("race-wrote")
+            print("    race-wrote 这一格判错：复核判红之后表还是被写了")  # gate-lint:detail
+        cases += 1
+        expect("pin", ["--kill-bounds", kill_bounds, ledger], 0, "钉进 3 行")
+        expect("pin-again", [ledger], 0, "3 行已钉")
+        expect("check-green", ["--check", ledger], 0, "3 行都钉在")
+        expect("check-default-ledgers", ["--check"], 0, "1 份红例账 3 行都钉在")
+        pinned_text = open(table_path, encoding="utf-8").read()
+        write(TABLE, "\n".join(line for line in pinned_text.split("\n") if "\tZ1 " not in line and "#Z1" not in line) + "\n")
+        expect("check-missing", ["--check", ledger], 1, "3 行里 1 行没钉进")
+        write(TABLE, pinned_text.replace("shape=2c0;first=returned:d\tE7", "shape=2c1;first=returned:d\tE7", 1))
+        expect("check-tampered", ["--check", ledger], 1, "重算的指纹不同")
+        write(TABLE, pinned_text)
+        conflicting = [list(fields) for fields in ledger_rows]
+        conflicting[0][11] = state_line("L", 3, "shape=1c0", "returned:z")
+        other_ledger = os.path.join(root, "research/results/e165-selftest-conflict-red-examples.tsv")
+        write(os.path.relpath(other_ledger, root), "# 账头\n" + "\t".join(conflicting[0]) + "\n")
+        expect("conflict", [other_ledger], 1, "原实现逐态行不同")
+        os.remove(other_ledger)
+        expect("replay-green", ["--replay", "--binary", stub, "--memory-cap", "none"], 0, "都与 ⑦ 逐字节相同")
+        expect("replay-mismatch", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "行与 ⑦ 不同", {"STUB_MODE": "wrong"})
+        expect("replay-fail", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "装置退 1", {"STUB_MODE": "fail"})
+        expect("replay-device-without-flag", ["--replay"], 77, "还不认 --pinned-cases")
+        # 经内存包装起、包装自己退 252（排不上）：判「没跑成」，出路照包装文件头，不报回归（开关 wrapper-exit-as-replay-red 下这一格判错）
+        fake_wrapper =os.path.join(work, "fake-memory-cap-wrapper.sh")
+        with open(fake_wrapper, "w", encoding="utf-8") as handle:
+            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 252（内存不够排不上）\nexit 252\n")
+        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, fake_wrapper
+        try:
+            expect("replay-wrapper-exit", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1, "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 252")
+        finally:
+            MEMORY_CAP_RUNNER = saved_runner
+        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的"))
+        expect("replay-variant-missing", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "不在它的表里了")
+        write(CRATES_TABLE, crates_table)
+
+    breaks_missed, breaks_proven = [], 0
+    if not wrong and not BREAK:
+        for item in ("pin-twice", "titles-unchecked", "no-recheck-before-write", "check-ignores-missing", "fingerprint-unchecked", "variant-unchecked", "replay-ignores-output",
+                     "wrapper-exit-as-replay-red"):
+            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
+                                       env=dict(os.environ, E165_PIN_RED_EXAMPLES_BREAK=item))
+            if completed.returncode == 0:
+                breaks_missed.append(item)
+            else:
+                breaks_proven += 1
+    if wrong:
+        report_red(f"e165-pin-red-examples 自证 {cases} 格里 {len(wrong)} 格判错：{' '.join(wrong)}", "按判错那几格的标签找 append / check / replay 里对应的一段修")
+        return EXIT_RED
+    if breaks_missed:
+        report_red(f"弄坏开关打开之后自证照样全对：{' '.join(breaks_missed)}", "那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
+        return EXIT_RED
+    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not BREAK else "（开着弄坏开关，没重跑开关）"
+    print(f"  ✓ e165-pin-red-examples 自证 {cases} 格都判对{suffix}")
+    return EXIT_GREEN
+
+
+if __name__ == "__main__":
+    preflight(__file__)
+    sys.exit(main(sys.argv[1:]))
diff --git a/research/scripts/stage-must-run.sh b/research/scripts/stage-must-run.sh
index a36d1014..5e8a6799 100755
--- a/research/scripts/stage-must-run.sh
+++ b/research/scripts/stage-must-run.sh
@@ -14,7 +14,8 @@
 # 以及**模块自己出错一律按要跑处理**——python 起不来、抛异常退 1，都不许被读成「可跳过」，那等于把这道门禁关掉。
 #
 # 判据只有一句：**这一道读的那几条路径，在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间，git 说变没变。**
-# 在这之前先看一眼全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时按登记输入在被判那棵树上的指纹写 git common-dir 里的
+# 在这之前先看一眼全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification
+# 判绿时按登记输入（登记了 environment= 的连同此刻的环境）在被判那棵树上的指纹写 git common-dir 里的
 # `singlefs-stage-green.<阶段文件名>.<指纹>`（admission.py stage-marker-write），在且没过 SINGLEFS_REUSE_HOURS 就可跳过——
 # 崩溃验证员在工作区跑过的那一趟，整轮门禁不用再跑第二遍（用户 2026-09-27 定：照 54 号写标记、只跑变了的）。
 # 路径清单在 `.claude/gate.d/stage-inputs.tsv`，那是唯一登记位；清单自己也进比对（清单变了必定重跑）。
diff --git a/research/scripts/verus-proof-check.py b/research/scripts/verus-proof-check.py
new file mode 100755
index 00000000..bc6b2f88
--- /dev/null
+++ b/research/scripts/verus-proof-check.py
@@ -0,0 +1,1100 @@
+#!/usr/bin/env python3
+# admission: always 每一次调都判此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
+# run-condition: none 要的 Verus、cargo 与内存包装由各子命令自己判，缺了判红并给出路（缺工具直接红，不交给预判退 78）
+"""门禁 checker-tier-formal-verification 的 Verus 那四格的判法（里程碑三七·5；阶段只负责把它接进门禁、给样本开关）。
+
+被验的：登记表 .claude/gate.d/verus-pins.tsv 点名的 crate（今天是只依赖 vstd 的 crates/singlefs-replay-prefix）与它里面的核（kernel 行，今天就是它的 src/lib.rs）。
+Verus 每次都直接起（不经 cargo verus：它按内容缓存，内容不变时不出判定行），对 crate 的入口文件 --crate-type=lib，带 --no-cheating、--output-json 与 --rlimit <登记的数>
+（登记表 rlimit 行；没有这一行时取 DEFAULT_RLIMIT，就是 Verus 0.2026.09.20 自己的默认 10。proof、vacuity、must-red-worlds 三格同一个数），
+判定取 stdout 的 JSON（verification-results 的 verified / errors / encountered-vir-error，verus 的 commit），报错位置取 stderr。
+直接起的 Verus 只带它自带的 vstd：crate 的 Cargo.toml 除 vstd 之外还有依赖，proof、vacuity、must-red-worlds 三格开跑之前判红。
+
+用法（<根> 是被判的仓根；四个判的子命令都退 0 绿、1 红、2 用法错、77 本次未跑）：
+  verus-proof-check.py toolchain <根> [--verus-root 目录] [--cargo-home 目录]
+      版本一致：Verus 安装里 version.json 的 version 与 commit、安装目录旁 verus.zip 的 sha256（zip 在就核，不在报一句）、
+      Cargo.lock 里登记的六个包各恰好一个版本且等于登记、本机 cargo 缓存里每个包的 .cargo_vcs_info.json（sha1、path_in_vcs）
+      与包源码的内容指纹（fingerprint 子命令的算法），都与登记相同。
+  verus-proof-check.py proof <根> [通用选项]
+      核里（去掉 // 注释之后）不许出现 external_body、assume…（含 assume_specification）、admit、verifier::external / verifier(external)、
+      verifier::rlimit / verifier(rlimit …)（函数上的 rlimit 属性盖过命令行的 --rlimit，那个函数就不在登记的额度上验）；
+      Verus 验整个 crate：限时内出结局、没有 vir 错（作弊标记在这一层也被 --no-cheating 拒）、errors 0、
+      verified 等于登记的数、跑的 Verus 的 commit 等于登记。超时判红（C5：超时不当通过）。
+  verus-proof-check.py vacuity <根> [通用选项]
+      空真：核里每一处 requires（整行以 requires 起头，fn 与闭包都算）找到它的函数体的 {（其后第一行以 { 起头、缩进不深于 requires 那一行的），
+      在临时副本里 { 后面插 assert(false);，一处一份副本、一次 Verus；每一处都要报 assertion failed 且位置就是插的那一行。
+      验得过 ⇒ 这一处的前置条件不可满足，判红。一次只插一处：插在外层函数入口的 assert(false) 失败之后，Verus 把其后全部当 false 已知，
+      里面闭包的那一处就验得过了（2026-10-01 在草稿里实测：七处一起插只报六处）。
+  verus-proof-check.py must-red-worlds <根> [通用选项]
+      必红世界：登记表 must-red-worlds 行点名的表（五段制表符分隔：变异名、文件、原文、替换文、期望片段；\\n 表换行），
+      每行原文在文件里恰好命中一次，改坏的副本上 Verus 要出 JSON、errors ≥ 1，输出里出现期望片段。一行都没有判红。
+      期望片段在原始输出连同照 JSON 的 verified 与 errors 拼出的「verification results:: <V> verified, <E> errors」那一行里找
+      （Verus 不带 --output-json 时打这一行，带了不打；拼出来的与它逐字相同）；JSON 里没有这两个数、拼不出的那一行判红。
+      输出里 error 起头的报错除去 Resource limit (rlimit) exceeded 与 aborting due to 之外至少还有一条（证明错）才算红；只剩 rlimit 用尽的那一行不作数、判红：
+      红在算力额度上，不是证明失败（改替换文，或调高登记的 rlimit）。替换文里（去掉 // 注释之后）有 verifier::rlimit / verifier(rlimit …) 的那一行判红：它盖过命令行的 --rlimit。
+  通用选项：--verus-root 目录（默认 $SINGLEFS_VERUS_ROOT，再默认 ${XDG_DATA_HOME:-~/.local/share}/<登记表 release 行第五列>）
+            --timeout 秒（默认 $GATE_VERUS_TIMEOUT_SECONDS，再默认 600）  --threads N（默认 $SINGLEFS_THREAD_CAP，再默认 nproc）
+            --jobs J（空真与必红世界几处同时验，默认 $GATE_VERUS_JOBS，再默认 min(处数, max(1, N / 4))；每一处的 Verus 取 N / J 条线程）
+            --memory-cap 上限（每次 Verus 经 research/scripts/run-with-memory-cap.sh，默认 $GATE_VERUS_MEMORY_MAX，再默认 4G；
+              限时交给它的 RUN_WITH_MEMORY_CAP_TIME_LIMIT，退 253 就是超时；退 250、251、252、254 是包装自己的结局，那一处判「没跑成」：
+              这一格退 1，✗ 行写「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」，→ 照包装文件头「退出码」一段，不指向证明、核或必红世界表）
+  verus-proof-check.py fingerprint <包目录>      打印包源码的内容指纹与文件数（填登记表用）
+  verus-proof-check.py tool-versions [<根>] [--verus-root 目录]   打印一行：version.json 的 version、commit，安装目录里 verus、rust_verify、z3、vstd.vir、
+      libvstd.rlib 各自的 sha256（不在的写「没有」），与 VERUS_Z3_PATH（没设写「未设」，设了写它的值与它指的文件的 sha256）
+      （门禁准入登记 stage-inputs.tsv 的 environment= 用，进复用判定与全绿标记的指纹；Verus 没装也打一行、退 0）
+  verus-proof-check.py --selftest                拿假的 Verus（一段按源码里的标记出判定的 python）、假的 cargo 缓存逐格判；弄坏开关逐个打开，每个都要让至少一格判错
+
+核没进仓（登记的核一份都不在、crate 的 Cargo.toml 不依赖 vstd、crate 的 src 里一处 verus! 都没有）时四格都退 77（本次未跑）；
+核不在而 crate 已经接了 Verus（依赖 vstd 或有 verus!）判红：核被删了不许悄悄变成本次未跑。
+
+包源码的内容指纹：包目录下全部普通文件，去掉顶层的 Cargo.toml、Cargo.toml.orig、.cargo_vcs_info.json（crates.io 打包时改写或生成的）
+与 .cargo-ok（cargo 解包时写的本机标记），逐个「sha256  相对路径」按路径字节序排好、每行一个换行，整段再取 sha256。
+
+弄坏开关 VERUS_PROOF_CHECK_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）：
+  commit-unchecked       toolchain 不比 version.json 的 commit          toolchain-commit 那一格（样本 verus-toolchain-pinned-commit-red）判绿
+  fingerprint-unchecked  toolchain 不比包源码的内容指纹                 toolchain-fingerprint 那一格（样本 verus-toolchain-pinned-fingerprint-red）判绿
+  absent-kernel-not-run  核不在一律退 77                                kernel-deleted 那一格（样本 verus-proof-kernel-deleted-red）退 77
+  cheating-unchecked     不 grep 作弊标记、不带 --no-cheating            proof-cheating 那一格（样本 verus-proof-cheating-red）判绿
+  count-unchecked        不比 verified 与登记的数                        proof-count 那一格（样本 verus-proof-count-red）判绿
+  timeout-as-pass        超时当绿                                        proof-timeout 那一格（样本 verus-proof-timeout-red）判绿
+  vacuity-accepts-verified  插了 assert(false) 还验得过也当没事          vacuity-vacuous 那一格（样本 verus-vacuity-red）判绿
+  vacuity-all-at-once       一份核的全部插入点插进同一份副本只跑一遍，闭包那一处没单报就当它随外层验过了
+                                                                         vacuity-closure-vacuous 那一格（样本 verus-vacuity-closure-red）判绿
+  mutation-accepts-green    改坏的副本上验得过也当红了                   must-red-survives 那一格（样本 verus-must-red-worlds-survives-red）判绿
+  rlimit-counts-as-red      必红世界里 rlimit 用尽的报错也当证明失败     must-red-rlimit 那一格（样本 verus-must-red-worlds-rlimit-red）判绿
+  rlimit-any-line-voids     必红世界里输出夹着 rlimit 用尽的一律不作数（另有证明错也不算）
+                                                                         must-red-rlimit-with-proof-error 那一格（样本 verus-must-red-worlds-rlimit-with-proof-error-green）判红
+  rlimit-attribute-unchecked  核里不认 verifier::rlimit 属性              proof-rlimit-attribute 那一格（样本 verus-proof-rlimit-attribute-red）判绿
+  mutation-rlimit-attribute-accepted  必红世界不拒替换文里的 verifier::rlimit 属性
+                                                                         must-red-rlimit-attribute 那一格（样本 verus-must-red-worlds-rlimit-attribute-red）判绿
+  rlimit-unpinned           Verus 命令行不带 --rlimit                    proof-green 那一格判错（假 Verus 没见到 --rlimit 就报 vir 错）
+  anchor-unchecked          不核原文恰好命中一次                         must-red-anchor 那一格（样本 verus-must-red-worlds-anchor-red）报不出「恰好命中一次」
+  expect-raw-json-only      必红世界的期望片段只在原始输出里找，不拼 verification results 那一行
+                                                                         must-red-results-line 那一格（样本 verus-must-red-worlds-results-line-green）判红
+  wrapper-exit-as-proof-red 包装自己退 250、251、252、254 照旧报成这一格的问题、给这一格自己的出路
+                                                                         proof-wrapper-exit、must-red-wrapper-exit 两格（自证里的假包装退 250）判错
+  tool-versions-launcher-only  tool-versions 只打 version.json 与 verus 的 sha256，不看 rust_verify、z3、vstd 与 VERUS_Z3_PATH
+                                                                         tool-versions-z3 那一格判错
+管不到的：spec 本身写得对不对；核与调它的 singlefs-core 之间（recovery.rs 调核那一段）没被证明的部分；插入点只认「requires 整行起头、函数体的 { 单独起行或是闭包那一行的头一个 {」
+  这种写法（verusfmt 与本仓的写法），别的写法认不出就判红要求改写法。
+"""
+import concurrent.futures
+import glob
+import hashlib
+import json
+import os
+import re
+import shutil
+import subprocess
+import sys
+import tempfile
+import tomllib
+
+import os as preflight_os, sys as preflight_sys  # noqa: E402
+# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
+preflight_sys.dont_write_bytecode = True
+preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
+from project_preflight import preflight  # noqa: E402
+
+SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
+REGISTRY = ".claude/gate.d/verus-pins.tsv"
+EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
+WRAPPER_TIMEOUT_EXIT = 253
+# 登记表没有 rlimit 行时 Verus 命令行上写的数：Verus 0.2026.09.20 自己的默认值（登记表 rlimit 行的依据写在那一行上面的注释里）
+DEFAULT_RLIMIT = 10
+RLIMIT_EXHAUSTED = "Resource limit (rlimit) exceeded"
+# Verus 不带 --output-json 时末尾打的判定行（0.2026.09.20 实测，1 也写 errors）；判法带 --output-json 不打它，必红世界照 JSON 的两个数拼回来再比期望片段
+RESULTS_LINE = "verification results:: {verified} verified, {errors} errors"
+WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上", 254: "被总上限挤掉"}
+MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
+# 包装自己的结局（WRAPPER_OWN_EXITS）那一处没跑成时给的出路，照 run-with-memory-cap.sh 文件头「退出码」一段
+WRAPPER_HOWTO = ("照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：250 是这一处要的比上限多，把 GATE_VERUS_MEMORY_MAX（或 --memory-cap）设大再跑；"
+                 "251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑，不改证明、核或必红世界表")
+# tool-versions 除 verus 外另哈希的安装文件：verus 只是起 rust_verify 的外壳，判定还取决于 rust_verify、z3 与 vstd
+TOOL_VERSION_FILES = ("rust_verify", "z3", "vstd.vir", "libvstd.rlib")
+EXCLUDED_PACKAGE_FILES = {"Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json", ".cargo-ok"}
+CHEATING_MARKER = re.compile(r"\b(external_body|external_fn_specification|assume\w*|admit)\b|verifier\s*::\s*external\b|verifier\s*\(\s*external\s*\)")
+# 函数（或模块）上的 rlimit 属性：#[verifier::rlimit(N)] 与旧写法 #[verifier(rlimit(N))] 都盖过命令行的 --rlimit（Verus 0.2026.09.20 实测两种写法都认、--no-cheating 不拒）
+RLIMIT_ATTRIBUTE = re.compile(r"verifier\s*::\s*rlimit\b|verifier\s*\(\s*rlimit\b")
+FUNCTION_LINE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(open\s+|closed\s+)?(proof\s+|spec\s+|exec\s+)?fn\s+(\w+)")
+DIAGNOSTIC_LOCATION = re.compile(r"^\s*-->\s+(\S+?):(\d+):(\d+)")
+BREAK = {item for item in os.environ.get("VERUS_PROOF_CHECK_BREAK", "").split(",") if item}
+
+
+class Red(Exception):
+    """这一格判红：message 是卡在哪，howto 是下一步。"""
+
+    def __init__(self, message, howto):
+        super().__init__(message)
+        self.message, self.howto = message, howto
+
+
+class NotRun(Exception):
+    """这一格本次未跑（核还没进仓）。"""
+
+
+def report_red(message, howto):
+    print(f"  ✗ {message}")
+    print(f"     → 怎么办：{howto}")
+
+
+def strip_line_comment(line):
+    position = line.find("//")
+    return line if position < 0 else line[:position]
+
+
+# ── 登记表 ──
+
+class Registry:
+    def __init__(self):
+        self.tag = self.commit = self.zip_sha256 = self.install_relative = None
+        self.packages = []
+        self.crate_directory = None
+        self.verified_count = None
+        self.kernels = []
+        self.must_red_table = None
+        self.rlimit = None
+
+
+def read_registry(root):
+    path = os.path.join(root, REGISTRY)
+    if not os.path.isfile(path):
+        raise Red(f"登记表 {REGISTRY} 不在 {root} 里", "它随仓走；被删了从 git 找回，格式写在它自己的文件头")
+    registry = Registry()
+    problems = []
+    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
+        if not raw.strip() or raw.startswith("#"):
+            continue
+        fields = raw.split("\t")
+        kind, width = fields[0], {"release": 5, "package": 6, "crate": 3, "kernel": 2, "must-red-worlds": 2, "rlimit": 2}.get(fields[0])
+        if width is None or len(fields) != width:
+            problems.append(f"第 {number} 行认不出（第一段 {kind!r}、{len(fields)} 段）")
+            continue
+        if kind == "release":
+            if registry.tag is not None:
+                problems.append(f"第 {number} 行：release 行不止一行")
+            registry.tag, registry.commit, registry.zip_sha256, registry.install_relative = fields[1:]
+            if not re.fullmatch(r"[0-9a-f]{40}", registry.commit) or not re.fullmatch(r"[0-9a-f]{64}", registry.zip_sha256):
+                problems.append(f"第 {number} 行：commit 要 40 位、zip 的 sha256 要 64 位小写十六进制")
+        elif kind == "package":
+            name, version, sha1, path_in_vcs, fingerprint = fields[1:]
+            if not re.fullmatch(r"[0-9a-f]{40}", sha1) or not re.fullmatch(r"[0-9a-f]{64}", fingerprint):
+                problems.append(f"第 {number} 行：sha1 要 40 位、内容指纹要 64 位小写十六进制")
+            registry.packages.append({"name": name, "version": version, "sha1": sha1, "path_in_vcs": path_in_vcs, "fingerprint": fingerprint})
+        elif kind == "crate":
+            if registry.crate_directory is not None:
+                problems.append(f"第 {number} 行：crate 行不止一行")
+            registry.crate_directory = fields[1].rstrip("/")
+            if not fields[2].isdigit() or int(fields[2]) == 0:
+                problems.append(f"第 {number} 行：verified 的登记数要是正整数，读到 {fields[2]!r}")
+            else:
+                registry.verified_count = int(fields[2])
+        elif kind == "kernel":
+            registry.kernels.append(fields[1])
+        elif kind == "rlimit":
+            if registry.rlimit is not None:
+                problems.append(f"第 {number} 行：rlimit 行不止一行")
+            if not fields[1].isdigit() or int(fields[1]) == 0:
+                problems.append(f"第 {number} 行：rlimit 要是正整数，读到 {fields[1]!r}")
+            else:
+                registry.rlimit = int(fields[1])
+        else:
+            if registry.must_red_table is not None:
+                problems.append(f"第 {number} 行：must-red-worlds 行不止一行")
+            registry.must_red_table = fields[1]
+    if registry.tag is None or registry.crate_directory is None or not registry.kernels or registry.must_red_table is None or not registry.packages:
+        problems.append("release、crate、must-red-worlds 各要恰好一行，package 与 kernel 至少一行")
+    for kernel in registry.kernels:
+        if registry.crate_directory and not kernel.startswith(registry.crate_directory + "/"):
+            problems.append(f"核 {kernel} 不在登记的 crate {registry.crate_directory} 里")
+    if problems:
+        raise Red(f"登记表 {REGISTRY} 写坏了：" + "；".join(problems), "照它文件头的格式改；一行一条、制表符分隔")
+    if registry.rlimit is None:
+        registry.rlimit = DEFAULT_RLIMIT
+    return registry
+
+
+def kernel_files(root, registry):
+    """登记的核都在就交回它们；核没进仓 NotRun；核不在而 crate 已经接了 Verus 判红。"""
+    missing = [kernel for kernel in registry.kernels if not os.path.isfile(os.path.join(root, kernel))]
+    if not missing:
+        return [os.path.join(root, kernel) for kernel in registry.kernels]
+    crate = os.path.join(root, registry.crate_directory)
+    evidence = []
+    manifest = os.path.join(crate, "Cargo.toml")
+    if os.path.isfile(manifest) and re.search(r"^\s*vstd\s*=", open(manifest, encoding="utf-8").read(), re.M):
+        evidence.append(f"{registry.crate_directory}/Cargo.toml 依赖 vstd")
+    for directory, _subdirectories, names in os.walk(os.path.join(crate, "src")):
+        for name in sorted(names):
+            if name.endswith(".rs") and "verus!" in open(os.path.join(directory, name), encoding="utf-8", errors="replace").read():
+                evidence.append(os.path.relpath(os.path.join(directory, name), root) + " 里有 verus!")
+    if (not evidence and len(missing) == len(registry.kernels)) or "absent-kernel-not-run" in BREAK:
+        raise NotRun(f"登记的核 {' '.join(missing)} 还没进仓（crate 不依赖 vstd、src 里没有 verus!）：Verus 这一格本次无对象可判")
+    raise Red(f"登记的核 {' '.join(missing)} 不在，而 crate 已经接了 Verus（{'；'.join(evidence) or '另一份登记的核在'}）",
+              f"核被删了或改了名：找回它，或者照新路径改 {REGISTRY} 的 kernel 行；核的删除不许让这一格变成本次未跑")
+
+
+# ── Verus 安装、crate、外部依赖 ──
+
+def verus_directory(registry, override):
+    if override:
+        return os.path.abspath(override)
+    if os.environ.get("SINGLEFS_VERUS_ROOT"):
+        return os.path.abspath(os.environ["SINGLEFS_VERUS_ROOT"])
+    data_home = os.environ.get("XDG_DATA_HOME") or os.path.join(os.path.expanduser("~"), ".local", "share")
+    return os.path.join(data_home, registry.install_relative)
+
+
+def read_version_json(directory):
+    path = os.path.join(directory, "version.json")
+    if not os.path.isfile(path):
+        return None
+    try:
+        return json.load(open(path, encoding="utf-8")).get("verus", {})
+    except (ValueError, OSError):
+        return None
+
+
+def crate_manifest(root, registry):
+    path = os.path.join(root, registry.crate_directory, "Cargo.toml")
+    if not os.path.isfile(path):
+        raise Red(f"{registry.crate_directory}/Cargo.toml 不在", f"crate 目录改了就照新路径改 {REGISTRY} 的 crate 行")
+    try:
+        return tomllib.load(open(path, "rb"))
+    except tomllib.TOMLDecodeError as error:
+        raise Red(f"{registry.crate_directory}/Cargo.toml 读不出：{error}", "先让 cargo build 认得这份清单")
+
+
+def check_dependencies(registry, manifest):
+    """直接起的 Verus 只带它自带的 vstd：被验的 crate 除 vstd 之外还有依赖就编不过，开跑之前判红、说清下一步。"""
+    others = sorted(name for name in manifest.get("dependencies", {}) if name != "vstd")
+    if others:
+        raise Red(f"{registry.crate_directory}/Cargo.toml 除 vstd 之外还依赖 {' '.join(others)}：直接起的 Verus 只带它自带的 vstd，编不了别的 crate",
+                  "核的 crate 只依赖 vstd：要用的东西搬进核，或留在调用核的那一侧；确要加依赖，先给 research/scripts/verus-proof-check.py "
+                  "补上用 Verus 钉的工具链编依赖、交给 --extern 的一步（旧写法 git log -S prepare_externs 找得到）")
+
+
+def copy_crate(root, registry, work):
+    source = os.path.join(root, registry.crate_directory)
+    destination = os.path.join(work, registry.crate_directory)
+    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target"), symlinks=True)
+    return destination
+
+
+def verus_arguments(registry, manifest, threads):
+    package = manifest.get("package", {})
+    entry = manifest.get("lib", {}).get("path", "src/lib.rs")
+    arguments = ["--crate-type=lib", f"--edition={package.get('edition', '2021')}", "--crate-name", package.get("name", "crate").replace("-", "_"),
+                 "--num-threads", str(threads), "--output-json"]
+    if "rlimit-unpinned" not in BREAK:
+        arguments += ["--rlimit", str(registry.rlimit)]
+    if "cheating-unchecked" not in BREAK:
+        arguments.insert(0, "--no-cheating")
+    return arguments + [entry]
+
+
+def run_verus(verus, crate_copy, arguments, options, key):
+    """起一次 Verus；交回 {exit, timed_out, wrapper, json, diagnostics}。"""
+    command = [verus, *arguments]
+    environment = dict(os.environ)
+    timeout = options["timeout"]
+    if options.get("launcher") is None:
+        command = ["bash", MEMORY_CAP_RUNNER, options["memory_cap"],
+                   "bash", os.path.join(SCRIPT_DIRECTORY, "capped.sh"), str(options["threads"]), "nice", "-n", "19", *command]
+        environment.update(RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout), RUN_WITH_MEMORY_CAP_KEY=f"verus-proof-check {key}",
+                           RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="门禁的上限按整个 crate 最坏的一处给，变异副本与插了断言的副本比平常大",
+                           RUN_WITH_MEMORY_CAP_PARALLEL_REASON="门禁里空真与必红世界几处同时验，每处各一份副本与各自的上限")
+        subprocess_timeout = None
+    else:
+        command = [*options["launcher"], *command]
+        subprocess_timeout = timeout
+    try:
+        completed = subprocess.run(command, cwd=crate_copy, capture_output=True, text=True, env=environment, timeout=subprocess_timeout)
+        exit_code, stdout, stderr = completed.returncode, completed.stdout, completed.stderr
+    except subprocess.TimeoutExpired as expired:
+        exit_code, stdout, stderr = WRAPPER_TIMEOUT_EXIT, "", (expired.stderr or b"").decode("utf-8", "replace") if isinstance(expired.stderr, bytes) else (expired.stderr or "")
+    outcome = {"exit": exit_code, "timed_out": exit_code == WRAPPER_TIMEOUT_EXIT, "wrapper": WRAPPER_OWN_EXITS.get(exit_code) if options.get("launcher") is None else None,
+               "json": None, "diagnostics": stderr + "\n" + stdout}
+    try:
+        outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None
+    except ValueError:
+        outcome["json"] = None
+    return outcome
+
+
+def diagnostic_blocks(text):
+    """stderr 里的「error: <说什么>」与它下面第一处「--> 文件:行:列」配成一对。"""
+    blocks, pending = [], None
+    for line in text.split("\n"):
+        if line.startswith("error"):
+            pending = line.split(":", 1)[1].strip() if ":" in line else line
+            if pending.startswith("aborting due to"):
+                pending = None
+            continue
+        match = DIAGNOSTIC_LOCATION.match(line)
+        if match and pending is not None:
+            blocks.append((pending, match.group(1), int(match.group(2))))
+            pending = None
+    return blocks
+
+
+def proof_error_messages(text):
+    """输出里 error 起头的报错，除去 rlimit 用尽与「aborting due to」那一句之后剩下的（证明错）；必红世界只剩 rlimit 用尽的那一行不作数。"""
+    messages = []
+    for line in text.split("\n"):
+        if not line.startswith("error"):
+            continue
+        message = line.split(":", 1)[1].strip() if ":" in line else line
+        if RLIMIT_EXHAUSTED in message or message.startswith("aborting due to"):
+            continue
+        messages.append(message)
+    return messages
+
+
+def results_line(results):
+    """照 JSON 的 verified 与 errors 拼出 Verus 普通输出里那一行判定行（与不带 --output-json 时逐字相同）；两个数缺一个、不是整数就交 None。"""
+    verified, errors = results.get("verified"), results.get("errors")
+    if not all(isinstance(value, int) and not isinstance(value, bool) for value in (verified, errors)):
+        return None
+    return RESULTS_LINE.format(verified=verified, errors=errors)
+
+
+def outcome_problem(outcome, options):
+    """这一次 Verus 没得出可判的结局时交回一句（超时、包装结局、没出 JSON），得出了交 None。"""
+    if outcome["timed_out"]:
+        if "timeout-as-pass" in BREAK:
+            return None
+        return f"超过 {options['timeout']} 秒没出结局，按红记（C5：超时不当通过）"
+    if outcome["wrapper"]:
+        if "wrapper-exit-as-proof-red" in BREAK:
+            return f"内存包装自己的结局（退 {outcome['exit']}，{outcome['wrapper']}），这一次不算结果"
+        return (f"没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 {outcome['exit']}（{outcome['wrapper']}，上限 {options['memory_cap']}），"
+                "这一次的输出不算判定")
+    if outcome["json"] is None or "verification-results" not in outcome["json"]:
+        tail = " | ".join(line for line in outcome["diagnostics"].strip().split("\n")[-6:] if line.strip())
+        return f"Verus 没出判定（退 {outcome['exit']}，编不过或起不来）：{tail}"
+    return None
+
+
+def problem_howto(outcome, howto):
+    """outcome_problem 报了问题时给的出路：包装自己的结局给包装文件头那一段（WRAPPER_HOWTO），别的给这一格自己的 howto。"""
+    if outcome["wrapper"] and "wrapper-exit-as-proof-red" not in BREAK:
+        return WRAPPER_HOWTO
+    return howto
+
+
+# ── 四格 ──
+
+def check_toolchain(root, options):
+    registry = read_registry(root)
+    kernel_files(root, registry)
+    problems = []
+    directory = verus_directory(registry, options.get("verus_root"))
+    version = read_version_json(directory)
+    if version is None:
+        problems.append((f"Verus 安装 {directory} 里没有读得出的 version.json", f"装登记的发布 {registry.tag}，或用 SINGLEFS_VERUS_ROOT 指到它的安装目录"))
+    else:
+        if version.get("version") != registry.tag:
+            problems.append((f"Verus 安装的 version 是 {version.get('version')}，登记的是 {registry.tag}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
+        if version.get("commit") != registry.commit and "commit-unchecked" not in BREAK:
+            problems.append((f"Verus 安装的 commit 是 {version.get('commit')}，登记的是 {registry.commit}", f"换装 {registry.tag}；要升级先改 {REGISTRY} 并复核同源"))
+    zip_path = os.path.join(os.path.dirname(directory), "verus.zip")
+    zip_note = "安装目录旁没有 verus.zip，没核它的 sha256"
+    if os.path.isfile(zip_path):
+        digest = hashlib.sha256(open(zip_path, "rb").read()).hexdigest()
+        zip_note = f"verus.zip 的 sha256 与登记相同"
+        if digest != registry.zip_sha256:
+            problems.append((f"{zip_path} 的 sha256 是 {digest}，登记的是 {registry.zip_sha256}", "重新下载登记的发布包，或把改过的那份挪走"))
+    lock_path = os.path.join(root, "Cargo.lock")
+    locked = {}
+    if not os.path.isfile(lock_path):
+        problems.append(("仓根没有 Cargo.lock", "cargo generate-lockfile 之后照登记钉版本"))
+    else:
+        for package in tomllib.load(open(lock_path, "rb")).get("package", []):
+            locked.setdefault(package.get("name"), []).append(package.get("version"))
+    cargo_home = options.get("cargo_home") or os.environ.get("CARGO_HOME") or os.path.join(os.path.expanduser("~"), ".cargo")
+    for package in registry.packages:
+        name, version_text = package["name"], package["version"]
+        if os.path.isfile(lock_path) and locked.get(name) != [version_text]:
+            problems.append((f"Cargo.lock 里 {name} 的版本是 {locked.get(name) or '没有'}，登记的是 [{version_text}]", f"依赖写成 {name} = \"={version_text}\"（或经 vstd 带进来）再 cargo update -p {name} --precise {version_text}"))
+        directories = sorted(glob.glob(os.path.join(cargo_home, "registry", "src", "*", f"{name}-{version_text}")))
+        if not directories:
+            problems.append((f"本机 cargo 缓存（{cargo_home}/registry/src/*/）里没有 {name}-{version_text}", "在仓根跑 cargo fetch --locked"))
+        for package_directory in directories:
+            vcs_path = os.path.join(package_directory, ".cargo_vcs_info.json")
+            try:
+                vcs = json.load(open(vcs_path, encoding="utf-8"))
+            except (OSError, ValueError):
+                vcs = {}
+            if vcs.get("git", {}).get("sha1") != package["sha1"] or vcs.get("path_in_vcs") != package["path_in_vcs"]:
+                problems.append((f"{package_directory} 的 .cargo_vcs_info.json 是 sha1={vcs.get('git', {}).get('sha1')} path_in_vcs={vcs.get('path_in_vcs')}，"
+                                 f"登记的是 {package['sha1']} {package['path_in_vcs']}", "缓存被别的来源顶掉了：删掉这个目录再 cargo fetch --locked"))
+            fingerprint, _count = package_source_fingerprint(package_directory)
+            if fingerprint != package["fingerprint"] and "fingerprint-unchecked" not in BREAK:
+                problems.append((f"{package_directory} 的源码内容指纹是 {fingerprint}，登记的是 {package['fingerprint']}",
+                                 "缓存里的源码被改过或换了来源：删掉这个目录再 cargo fetch --locked；确要换包就先改登记并复核同源"))
+    if problems:
+        for message, howto in problems:
+            report_red(message, howto)
+        return EXIT_RED
+    print(f"  ✓ Verus {registry.tag}（commit {registry.commit[:12]}…）与登记相同，{zip_note}；Cargo.lock 与本机缓存里 {len(registry.packages)} 个包的版本、出处与源码指纹都与登记相同")
+    return EXIT_GREEN
+
+
+def cheating_markers(root, kernels):
+    found = []
+    for kernel in kernels:
+        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
+            match = CHEATING_MARKER.search(strip_line_comment(line))
+            if match:
+                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
+    return found
+
+
+def rlimit_attributes(root, kernels):
+    found = []
+    for kernel in kernels:
+        for number, line in enumerate(open(kernel, encoding="utf-8").read().split("\n"), 1):
+            match = RLIMIT_ATTRIBUTE.search(strip_line_comment(line))
+            if match:
+                found.append(f"{os.path.relpath(kernel, root)}:{number}：{match.group(0)}")
+    return found
+
+
+def verus_executable(registry, options):
+    directory = verus_directory(registry, options.get("verus_root"))
+    executable = os.path.join(directory, "verus")
+    version = read_version_json(directory)
+    if not os.access(executable, os.X_OK) or version is None:
+        raise Red(f"找不到 Verus（{executable} 不可执行或旁边没有 version.json）",
+                  f"装登记的发布 {registry.tag}（{REGISTRY} 的 release 行），或用 SINGLEFS_VERUS_ROOT 指到它的安装目录")
+    return executable, version
+
+
+def prepared(root, options):
+    registry = read_registry(root)
+    kernels = kernel_files(root, registry)
+    manifest = crate_manifest(root, registry)
+    check_dependencies(registry, manifest)
+    executable, _version = verus_executable(registry, options)
+    return registry, kernels, manifest, executable
+
+
+def check_proof(root, options):
+    registry, kernels, manifest, executable = prepared(root, options)
+    problems = []
+    if "cheating-unchecked" not in BREAK:
+        for marker in cheating_markers(root, kernels):
+            problems.append((f"核里有不许的标记 {marker}", "证明里不许 assume / admit / external_body / verifier::external：把它换成真证明，证不出就照实报，不绕过"))
+        if "rlimit-attribute-unchecked" not in BREAK:
+            for marker in rlimit_attributes(root, kernels):
+                problems.append((f"核里有不许的标记 {marker}（函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，这个函数不在登记的额度上验）",
+                                 f"删掉这个属性；证明确要更多额度，照 {REGISTRY} rlimit 行上面的注释在新数上把核与必红世界表整张重验一遍，再调高登记的数"))
+    with tempfile.TemporaryDirectory(prefix="verus-proof-check-") as work:
+        crate_copy = copy_crate(root, registry, work)
+        outcome = run_verus(executable, crate_copy, verus_arguments(registry, manifest, options["threads"]), options, "proof")
+    problem = outcome_problem(outcome, options)
+    if problem:
+        problems.append((problem, problem_howto(outcome, "看上面 Verus 的报错；超时先看是哪条查询慢（--profile），确要更长的时限设 GATE_VERUS_TIMEOUT_SECONDS 并写明为什么")))
+    elif not outcome["timed_out"]:
+        results = outcome["json"]["verification-results"]
+        commit = outcome["json"].get("verus", {}).get("commit")
+        if results.get("encountered-vir-error") or not results.get("success") or results.get("errors", 1) != 0:
+            blocks = diagnostic_blocks(outcome["diagnostics"])
+            listed = "；".join(f"{path}:{line} {message}" for message, path, line in blocks[:8]) or outcome["diagnostics"].strip()[-400:]
+            problems.append((f"Verus 没验过：verified {results.get('verified')}、errors {results.get('errors')}、vir 错 {results.get('encountered-vir-error')}：{listed}",
+                             "证明或实现有一处对不上：照报错的位置修；反例是真的就先修实现"))
+        elif results.get("verified") != registry.verified_count and "count-unchecked" not in BREAK:
+            problems.append((f"verified {results.get('verified')} 与登记的 {registry.verified_count} 不同（少了是有证明被删掉或没被验到）",
+                             f"确是加减了证明就照这一次的数改 {REGISTRY} 的 crate 行，并在改动说明里写加减了哪几个"))
+        elif commit != registry.commit:
+            problems.append((f"跑的 Verus 报的 commit 是 {commit}，登记的是 {registry.commit}", "SINGLEFS_VERUS_ROOT 指到登记的那份安装"))
+        elif not results.get("is-verifying-entire-crate", False):
+            problems.append(("Verus 报这一次没验整个 crate", "别给 --verify-module 这类只验一部分的选项"))
+    if problems:
+        for message, howto in problems:
+            report_red(message, howto)
+        return EXIT_RED
+    verified = outcome["json"]["verification-results"].get("verified") if outcome["json"] else "?"
+    print(f"  ✓ Verus 验 {registry.crate_directory}：{verified} verified、0 errors（登记 {registry.verified_count}），--no-cheating，--rlimit {registry.rlimit}，"
+          f"{len(kernels)} 份核里 0 处作弊标记，限时 {options['timeout']} 秒内出结局")
+    return EXIT_GREEN
+
+
+def requires_insertion_points(text):
+    """每一处 requires 的函数体头：[(requires 行号, { 所在行的下标, 说明)]；认不出函数体的抛 Red。"""
+    lines = text.split("\n")
+    points = []
+    for index, line in enumerate(lines):
+        stripped = strip_line_comment(line).strip()
+        if stripped != "requires" and not stripped.startswith("requires "):
+            continue
+        indent = len(line) - len(line.lstrip())
+        brace = None
+        for later in range(index + 1, len(lines)):
+            code = strip_line_comment(lines[later])
+            if FUNCTION_LINE.match(code):
+                break
+            if code.strip().startswith("{") and len(code) - len(code.lstrip()) <= indent:
+                brace = later
+                break
+        if brace is None:
+            raise Red(f"第 {index + 1} 行的 requires 找不到函数体的 {{（没有函数体，或 {{ 没单独起行）",
+                      "把函数体的 { 单独起一行（verusfmt 的写法）；确是没有函数体的声明，把判法补进 research/scripts/verus-proof-check.py 的 requires_insertion_points")
+        owner = next((FUNCTION_LINE.match(strip_line_comment(lines[back])).group(5) for back in range(index, -1, -1)
+                      if FUNCTION_LINE.match(strip_line_comment(lines[back]))), "?")
+        inline = lines[brace].split("{", 1)[1].strip()
+        label = f"{owner} 里第 {index + 1} 行起的闭包" if inline else f"fn {owner}"
+        points.append((index + 1, brace, label))
+    return points
+
+
+def verify_in_parallel(root, registry, manifest, executable, options, edits, key):
+    """edits：[(说明, {相对仓根的文件: 新内容})]；每一处一份副本、一次 Verus。交回与 edits 同序的 outcome。"""
+    jobs = options["jobs"] or min(len(edits), max(1, options["threads"] // 4))
+    threads = max(1, options["threads"] // jobs)
+
+    def one(edit):
+        with tempfile.TemporaryDirectory(prefix="verus-proof-check-") as work:
+            crate_copy = copy_crate(root, registry, work)
+            for relative, content in edit[1].items():
+                with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
+                    handle.write(content)
+            return run_verus(executable, crate_copy, verus_arguments(registry, manifest, threads), options, key)
+
+    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
+        futures = [pool.submit(one, edit) for edit in edits]
+        return [future.result() for future in futures]
+
+
+def with_assert_false(line):
+    head, tail = line.split("{", 1)
+    return head + "{ assert(false);" + tail
+
+
+def check_vacuity(root, options):
+    registry, kernels, manifest, executable = prepared(root, options)
+    edits, expected, all_at_once = [], [], []
+    for kernel in kernels:
+        text = open(kernel, encoding="utf-8").read()
+        relative = os.path.relpath(kernel, root)
+        lines = text.split("\n")
+        every_point = list(lines)
+        for requires_line, brace, label in requires_insertion_points(text):
+            changed = list(lines)
+            changed[brace] = with_assert_false(changed[brace])
+            every_point[brace] = with_assert_false(every_point[brace])
+            edits.append((f"{relative} {label}", {relative: "\n".join(changed)}))
+            expected.append((os.path.relpath(kernel, os.path.join(root, registry.crate_directory)), brace + 1, requires_line))
+        all_at_once.append((relative, {relative: "\n".join(every_point)}))
+    if not edits:
+        raise NotRun(f"{len(kernels)} 份核里一处 requires 都没有：没有前置条件，空真这一格无对象可判")
+    if "vacuity-all-at-once" in BREAK:
+        # 弄坏：一份核的全部插入点插进同一份副本只跑一遍。外层函数入口的 assert(false) 一失败，里面闭包那一处就验得过、不单报
+        shared = dict(zip((relative for relative, _change in all_at_once),
+                          verify_in_parallel(root, registry, manifest, executable, options, all_at_once, "vacuity")))
+        outcomes = [shared[label.split(" ", 1)[0]] for label, _change in edits]
+    else:
+        outcomes = verify_in_parallel(root, registry, manifest, executable, options, edits, "vacuity")
+    problems = []
+    for (label, _change), (path, line, requires_line), outcome in zip(edits, expected, outcomes):
+        problem = outcome_problem(outcome, options)
+        if problem:
+            problems.append((f"{label}（第 {requires_line} 行的 requires）：{problem}",
+                             problem_howto(outcome, "插断言的副本判不出来：看报错，修好再跑；超时见 GATE_VERUS_TIMEOUT_SECONDS")))
+            continue
+        if outcome["timed_out"]:
+            continue
+        results = outcome["json"]["verification-results"]
+        if results.get("success") and results.get("errors", 0) == 0:
+            if "vacuity-accepts-verified" not in BREAK:
+                problems.append((f"{label}：函数体头插 assert(false) 照样验得过——第 {requires_line} 行起的前置条件不可满足（空真），这个函数的 ensures 什么也没证",
+                                 "改前置条件让它可满足（常见：两条互相矛盾、范围写反）；改完这一格插的 assert(false) 要在那一行报 assertion failed"))
+            continue
+        blocks = diagnostic_blocks(outcome["diagnostics"])
+        if not any("assertion failed" in message and path == block_path and line == block_line for message, block_path, block_line in blocks):
+            if "vacuity-all-at-once" in BREAK and "闭包" in label:
+                continue
+            listed = "；".join(f"{block_path}:{block_line} {message}" for message, block_path, block_line in blocks[:6])
+            problems.append((f"{label}：插的 assert(false) 在 {path}:{line}，Verus 报的错不在那一行（{listed or '没有配上位置的报错'}）",
+                             "插入点认错了或副本编不过：看 requires_insertion_points 认的 {{ 是不是函数体的那一个"))
+    if problems:
+        for message, howto in problems:
+            report_red(message, howto)
+        return EXIT_RED
+    print(f"  ✓ 空真：{len(kernels)} 份核里 {len(edits)} 处 requires，每一处在函数体头插 assert(false) 都在插的那一行报 assertion failed（前置条件可满足）")
+    return EXIT_GREEN
+
+
+def must_red_rows(root, registry):
+    path = os.path.join(root, registry.must_red_table)
+    if not os.path.isfile(path):
+        raise Red(f"必红世界表 {registry.must_red_table} 不在", "核进了仓就要带它（五段制表符分隔：变异名、文件、原文、替换文、期望片段）；路径改了就改登记表的 must-red-worlds 行")
+    rows, problems = [], []
+    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
+        if not raw.strip() or raw.startswith("#"):
+            continue
+        fields = raw.split("\t")
+        if len(fields) != 5 or not all(fields[index] for index in (0, 1, 2, 4)):
+            problems.append(f"第 {number} 行不是五段（或变异名、文件、原文、期望片段有空的）")
+            continue
+        rows.append({"number": number, "name": fields[0], "file": fields[1], "original": fields[2].replace("\\n", "\n"),
+                     "replacement": fields[3].replace("\\n", "\n"), "expected": fields[4]})
+    if problems:
+        raise Red(f"{registry.must_red_table} 写坏了：" + "；".join(problems), "照表头的格式改")
+    if not rows:
+        raise Red(f"{registry.must_red_table} 一行都没有：核没有一个必须报红的世界", "每一层证明至少一条改回去 Verus 就红的变异（show-me-test.md「设计论证同样照办」）")
+    return rows
+
+
+def check_must_red_worlds(root, options):
+    registry, _kernels, manifest, executable = prepared(root, options)
+    rows = must_red_rows(root, registry)
+    problems, edits, judged = [], [], []
+    rlimit_with_proof_error = matched_in_results_line = 0
+    for row in rows:
+        target = os.path.join(root, row["file"])
+        if not row["file"].startswith(registry.crate_directory + "/") or not os.path.isfile(target):
+            problems.append((f"第 {row['number']} 行「{row['name']}」的文件 {row['file']} 不在登记的 crate 里或不存在", "变异只许改被验的那个 crate 里的文件"))
+            continue
+        attributes = [match.group(0) for match in (RLIMIT_ATTRIBUTE.search(strip_line_comment(line)) for line in row["replacement"].split("\n")) if match]
+        if attributes and "mutation-rlimit-attribute-accepted" not in BREAK:
+            problems.append((f"第 {row['number']} 行「{row['name']}」的替换文里有 {attributes[0]}：函数上的 rlimit 属性盖过命令行的 --rlimit {registry.rlimit}，改坏的副本不在登记的额度上验",
+                             "删掉替换文里的 rlimit 属性；这一行要更多额度才红得出来，就换一处改法（必红世界要在登记的 rlimit 上红）"))
+            continue
+        text = open(target, encoding="utf-8").read()
+        hits = text.count(row["original"])
+        if hits != 1 and "anchor-unchecked" not in BREAK:
+            problems.append((f"第 {row['number']} 行「{row['name']}」的原文在 {row['file']} 里命中 {hits} 次，要恰好命中一次（锚点腐化）",
+                             "照今天的代码改这一行的原文，让它只命中要改坏的那一处"))
+            continue
+        edits.append((row["name"], {row["file"]: text.replace(row["original"], row["replacement"], 1)}))
+        judged.append(row)
+    outcomes = verify_in_parallel(root, registry, manifest, executable, options, edits, "must-red-worlds") if edits else []
+    for row, outcome in zip(judged, outcomes):
+        problem = outcome_problem(outcome, options)
+        if problem:
+            problems.append((f"第 {row['number']} 行「{row['name']}」：{problem}",
+                             problem_howto(outcome, "必红世界要红在证明上：改坏之后编不过的变异不算，换一处只改判定的改法")))
+            continue
+        if outcome["timed_out"]:
+            continue
+        results = outcome["json"]["verification-results"]
+        summary_line = results_line(results)
+        if summary_line is None:
+            problems.append((f"第 {row['number']} 行「{row['name']}」：Verus 的 JSON 里没有整数的 verified 与 errors（读到 verified {results.get('verified')!r}、"
+                             f"errors {results.get('errors')!r}、vir 错 {results.get('encountered-vir-error')}），拼不出「verification results:: <V> verified, <E> errors」那一行，期望片段没法比",
+                             "vir 错是改坏的副本没走到验证（类型或模式错）：换一处只改判定的改法；没有 vir 错就是 Verus 换了 JSON 的写法："
+                             "照新写法改 research/scripts/verus-proof-check.py 的 results_line"))
+            continue
+        searchable = outcome["diagnostics"] if "expect-raw-json-only" in BREAK else outcome["diagnostics"] + "\n" + summary_line
+        rlimit_exhausted = RLIMIT_EXHAUSTED in outcome["diagnostics"]
+        proof_errors = proof_error_messages(outcome["diagnostics"])
+        if rlimit_exhausted and (not proof_errors or "rlimit-any-line-voids" in BREAK) and "rlimit-counts-as-red" not in BREAK:
+            listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
+            problems.append((f"第 {row['number']} 行「{row['name']}」：Verus 输出里有 {RLIMIT_EXHAUSTED}，除它与 aborting due to 之外没有别的报错（--rlimit {registry.rlimit}；"
+                             f"verified {results.get('verified')}、errors {results.get('errors')}：{listed}），这一行的红不作数",
+                             f"这一行红在算力额度上，不是证明失败：改替换文或调高登记的 rlimit（{REGISTRY} 的 rlimit 行）"))
+            continue
+        if rlimit_exhausted:
+            rlimit_with_proof_error += 1
+        if results.get("errors", 0) < 1 and "mutation-accepts-green" not in BREAK:
+            problems.append((f"第 {row['number']} 行「{row['name']}」改坏之后 Verus 照样验过（verified {results.get('verified')}、errors 0）：证明没盯住这一处",
+                             "补证明或收紧 spec，让这一处改坏就验不过；确是等价变异就从表里删掉并写明为什么"))
+        elif row["expected"] not in searchable and "mutation-accepts-green" not in BREAK:
+            listed = "；".join(f"{path}:{line} {message}" for message, path, line in diagnostic_blocks(outcome["diagnostics"])[:6])
+            problems.append((f"第 {row['number']} 行「{row['name']}」红了，但输出里没有期望的「{row['expected']}」（报的是：{listed}；照 JSON 拼出的判定行：{summary_line}）",
+                             "红的地方不是这条变异要盯的那一处：查证明是不是换了一条路红的，再改期望片段或改证明"))
+        elif row["expected"] not in outcome["diagnostics"]:
+            matched_in_results_line += 1
+    if problems:
+        for message, howto in problems:
+            report_red(message, howto)
+        return EXIT_RED
+    print(f"  ✓ 必红世界：{registry.must_red_table} 的 {len(rows)} 行每行改坏之后 Verus 都报错，且出现期望的片段；原文都恰好命中一次；"
+          f"没有一行只红在 rlimit 用尽上（--rlimit {registry.rlimit}；{rlimit_with_proof_error} 行输出里夹着 rlimit 用尽、另有证明错，红计入）；"
+          f"{matched_in_results_line} 行的期望片段对上的是照 JSON 拼出的 verification results 那一行")
+    return EXIT_GREEN
+
+
+# ── 指纹与工具版本 ──
+
+def package_source_fingerprint(directory):
+    entries = []
+    for current, subdirectories, names in os.walk(directory):
+        subdirectories.sort()
+        for name in names:
+            full = os.path.join(current, name)
+            relative = os.path.relpath(full, directory)
+            if relative in EXCLUDED_PACKAGE_FILES:
+                continue
+            content = ("symlink:" + os.readlink(full)).encode() if os.path.islink(full) else open(full, "rb").read()
+            entries.append((relative.encode("utf-8", "surrogateescape"), hashlib.sha256(content).hexdigest()))
+    entries.sort()
+    text = b"".join(digest.encode() + b"  " + relative + b"\n" for relative, digest in entries)
+    return hashlib.sha256(text).hexdigest(), len(entries)
+
+
+def tool_versions(root, override):
+    try:
+        registry = read_registry(root)
+    except Red:
+        registry = Registry()
+        registry.install_relative = "singlefs/formal-tools/verus"
+    directory = verus_directory(registry, override)
+    version = read_version_json(directory)
+
+    def file_sha256(path):
+        return hashlib.sha256(open(path, "rb").read()).hexdigest() if os.path.isfile(path) else "没有"
+
+    digest = file_sha256(os.path.join(directory, "verus"))
+    extra = ""
+    if "tool-versions-launcher-only" not in BREAK:
+        z3_override = os.environ.get("VERUS_Z3_PATH", "")
+        extra = "".join(f" {name}_sha256={file_sha256(os.path.join(directory, name))}" for name in TOOL_VERSION_FILES)
+        extra += f" VERUS_Z3_PATH={z3_override + ':' + file_sha256(z3_override) if z3_override else '未设'}"
+    if version is None:
+        print(f"verus 没装（{directory} 里没有 version.json） verus_sha256={digest}{extra}")
+    else:
+        print(f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest}{extra}")
+    return EXIT_GREEN
+
+
+# ── 命令行 ──
+
+def parse_options(arguments):
+    options = {"verus_root": None, "cargo_home": None, "launcher": None,
+               "timeout": int(os.environ.get("GATE_VERUS_TIMEOUT_SECONDS") or 600),
+               "threads": int(os.environ.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1),
+               "jobs": int(os.environ.get("GATE_VERUS_JOBS") or 0),
+               "memory_cap": os.environ.get("GATE_VERUS_MEMORY_MAX") or "4G"}
+    positional = []
+    flags = {"--verus-root": "verus_root", "--cargo-home": "cargo_home", "--timeout": "timeout", "--threads": "threads",
+             "--jobs": "jobs", "--memory-cap": "memory_cap"}
+    position = 0
+    while position < len(arguments):
+        argument = arguments[position]
+        if argument in flags:
+            if position + 1 >= len(arguments):
+                raise ValueError(f"{argument} 后面缺值")
+            value = arguments[position + 1]
+            options[flags[argument]] = int(value) if flags[argument] in ("timeout", "threads", "jobs") else value
+            position += 2
+        elif argument.startswith("--"):
+            raise ValueError(f"认不出的选项 {argument}")
+        else:
+            positional.append(argument)
+            position += 1
+    if options["threads"] < 1 or options["timeout"] < 1 or options["jobs"] < 0:
+        raise ValueError("--threads、--timeout 要正整数，--jobs 不许负")
+    return positional, options
+
+
+CHECKS = {"toolchain": check_toolchain, "proof": check_proof, "vacuity": check_vacuity, "must-red-worlds": check_must_red_worlds}
+
+
+def run_check(name, root, options):
+    try:
+        return CHECKS[name](os.path.abspath(root), options)
+    except Red as red:
+        report_red(red.message, red.howto)
+        return EXIT_RED
+    except NotRun as not_run:
+        print(f"  ! {not_run}")
+        return EXIT_NOT_RUN
+
+
+def main(arguments):
+    if arguments[:1] == ["--selftest"]:
+        return selftest()
+    if not arguments or arguments[0] not in (*CHECKS, "fingerprint", "tool-versions"):
+        report_red("用法：verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根> [选项] | fingerprint <包目录> | tool-versions [<根>] | --selftest",
+                   "看文件头的用法一节")
+        return EXIT_USAGE
+    try:
+        positional, options = parse_options(arguments[1:])
+    except ValueError as error:
+        report_red(f"参数不对：{error}", "看文件头的用法一节")
+        return EXIT_USAGE
+    if arguments[0] == "fingerprint":
+        if len(positional) != 1 or not os.path.isdir(positional[0]):
+            report_red("fingerprint 要一个包目录", "例：verus-proof-check.py fingerprint ~/.cargo/registry/src/<索引>/vstd-<版本>")
+            return EXIT_USAGE
+        fingerprint, count = package_source_fingerprint(positional[0])
+        print(f"{fingerprint}\t{count}")
+        return EXIT_GREEN
+    if arguments[0] == "tool-versions":
+        return tool_versions(os.path.abspath(positional[0]) if positional else os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY)), options["verus_root"])
+    if len(positional) != 1 or not os.path.isdir(positional[0]):
+        report_red(f"{arguments[0]} 要恰好一个仓根", "例：verus-proof-check.py proof /path/to/singlefs")
+        return EXIT_USAGE
+    return run_check(arguments[0], positional[0], options)
+
+
+# ── 自证 ──
+
+FAKE_VERUS = r'''#!/usr/bin/env python3
+# 假的 Verus：按源码里的标记出判定。只给 verus-proof-check.py --selftest 用。
+import json, os, re, sys, time
+here = os.path.dirname(os.path.realpath(__file__))
+commit = json.load(open(os.path.join(here, "version.json")))["verus"]["commit"]
+no_cheating = "--no-cheating" in sys.argv
+rlimit = float(sys.argv[sys.argv.index("--rlimit") + 1]) if "--rlimit" in sys.argv[:-1] else None
+files = []
+for directory, _s, names in os.walk("src"):
+    for name in sorted(names):
+        if name.endswith(".rs"):
+            files.append(os.path.join(directory, name))
+functions, errors, vir = 0, [], []
+for path in sorted(files):
+    lines = open(path).read().split("\n")
+    owner_vacuous = owner_failed = closure_vacuous = False
+    for number, line in enumerate(lines, 1):
+        if "SELFTEST_SLEEP" in line:
+            time.sleep(30)
+        if re.match(r"^\s*(pub\s+)?(proof\s+)?fn\s", line):
+            functions += 1
+            owner_vacuous = "SELFTEST_VACUOUS" in line
+            owner_failed = False
+        if "SELFTEST_CLOSURE_VACUOUS" in line:
+            closure_vacuous = True
+        if "assert(false);" in line:
+            if closure_vacuous:
+                closure_vacuous = False
+            elif not owner_vacuous and not owner_failed:
+                errors.append(("assertion failed", path, number, line.index("assert(false)") + 1))
+                owner_failed = True
+        if "SELFTEST_BROKEN" in line:
+            errors.append(("postcondition not satisfied", path, number, 1))
+        needs = re.search(r"SELFTEST_NEEDS_RLIMIT_(\d+)", line)
+        if needs and rlimit is not None and rlimit < int(needs.group(1)):
+            errors.append(("while loop: Resource limit (rlimit) exceeded; consider rerunning with --profile for more details", path, number, 1))
+        if no_cheating and re.search(r"\bassume\(", line.split("//")[0]):
+            vir.append(("assume/admit not allowed with --no-cheating", path, number, 1))
+if rlimit is None:
+    vir.append(("SELFTEST: --rlimit not on the command line", "src/lib.rs", 1, 1))
+for message, path, number, column in errors + vir:
+    sys.stderr.write(f"error: {message}\n   --> {path}:{number}:{column}\n    |\n\n")
+if vir:
+    print(json.dumps({"verification-results": {"encountered-error": True, "encountered-vir-error": True, "success": False}, "verus": {"commit": commit}}))
+    sys.exit(1)
+failed = len({(p, n) for _m, p, n, _c in errors})
+results = {"encountered-error": bool(errors), "encountered-vir-error": False, "success": not errors,
+           "verified": functions - failed, "errors": failed, "is-verifying-entire-crate": True}
+if any("SELFTEST_NO_COUNTS" in open(path).read() for path in files):
+    del results["verified"], results["errors"]
+print(json.dumps({"verification-results": results, "verus": {"commit": commit}}))
+sys.exit(1 if errors else 0)
+'''
+
+SELFTEST_KERNEL = """use vstd::prelude::*;
+verus! {
+// 这里的 assume 只在注释里，不算作弊标记
+pub fn clamp_to_limit(value: u64, limit: u64) -> (result: u64)
+    requires
+        limit > 0,
+    ensures
+        result <= limit,
+{
+    if value > limit { limit } else { value }
+}
+
+pub fn keep_small(value: u64) -> (result: u64)
+    ensures
+        result <= 10,
+{
+    let pick = |candidate: u64| -> (picked: u64)
+        requires
+            candidate <= 10,
+        { candidate };
+    if value > 10 { pick(10) } else { pick(value) }
+}
+} // verus!
+"""
+
+
+def selftest():
+    global MEMORY_CAP_RUNNER
+    cases, wrong = 0, []
+    with tempfile.TemporaryDirectory(prefix="verus-proof-check-selftest-") as work:
+        commit, other_commit = "a" * 40, "b" * 40
+        install = os.path.join(work, "tools", "verus-x86-linux")
+        os.makedirs(install)
+        with open(os.path.join(install, "verus"), "w") as handle:
+            handle.write(FAKE_VERUS)
+        os.chmod(os.path.join(install, "verus"), 0o755)
+        zip_path = os.path.join(work, "tools", "verus.zip")
+        open(zip_path, "wb").write(b"zip")
+        zip_digest = hashlib.sha256(b"zip").hexdigest()
+        cargo_home = os.path.join(work, "cargo-home")
+        package_directory = os.path.join(cargo_home, "registry", "src", "index-x", "vstd-0.0.1")
+        os.makedirs(os.path.join(package_directory, "src"))
+        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
+        open(os.path.join(package_directory, "Cargo.toml"), "w").write("[package]\n")
+        open(os.path.join(package_directory, ".cargo-ok"), "w").write('{"v":1}')
+        json.dump({"git": {"sha1": "c" * 40}, "path_in_vcs": "source/vstd"}, open(os.path.join(package_directory, ".cargo_vcs_info.json"), "w"))
+        fingerprint, _count = package_source_fingerprint(package_directory)
+
+        def make_root(name, kernel=SELFTEST_KERNEL, count=2, must_red=None, version_commit=commit, lock_version="0.0.1", with_kernel=True, wired=True, extra_dependency="",
+                      rlimit=7):
+            root = os.path.join(work, name)
+            os.makedirs(os.path.join(root, ".claude", "gate.d"))
+            os.makedirs(os.path.join(root, "crates", "core", "src"))
+            json.dump({"verus": {"version": "0.9.9.tag", "commit": version_commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
+            open(os.path.join(root, REGISTRY), "w").write(
+                "# 自证用\n"
+                f"release\t0.9.9.tag\t{commit}\t{zip_digest}\tunused\n"
+                f"package\tvstd\t0.0.1\t{'c' * 40}\tsource/vstd\t{fingerprint}\n"
+                "crate\tcrates/core\t%d\n" % count +
+                "kernel\tcrates/core/src/kernel.rs\n"
+                "must-red-worlds\tcrates/core/verus-mutations.tsv\n" + (f"rlimit\t{rlimit}\n" if rlimit is not None else ""))
+            open(os.path.join(root, "Cargo.lock"), "w").write(f'version = 4\n\n[[package]]\nname = "vstd"\nversion = "{lock_version}"\n')
+            open(os.path.join(root, "crates", "core", "Cargo.toml"), "w").write(
+                '[package]\nname = "core-sample"\nedition = "2021"\n\n[dependencies]\n' + ('vstd = "=0.0.1"\n' if wired else "") + extra_dependency)
+            open(os.path.join(root, "crates", "core", "src", "lib.rs"), "w").write("pub mod kernel;\n" if with_kernel else "")
+            if with_kernel:
+                open(os.path.join(root, "crates", "core", "src", "kernel.rs"), "w").write(kernel)
+            rows = must_red if must_red is not None else ["收紧改松\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tpostcondition not satisfied"]
+            open(os.path.join(root, "crates", "core", "verus-mutations.tsv"), "w").write("# 自证用\n" + "".join(row + "\n" for row in rows))
+            return root
+
+        options = {"verus_root": install, "cargo_home": cargo_home, "launcher": [], "timeout": 5, "threads": 2, "jobs": 2,
+                   "memory_cap": "1G"}
+
+        def expect(label, check, root, want_exit, want_text):
+            nonlocal cases
+            cases += 1
+            import io
+            import contextlib
+            buffer = io.StringIO()
+            with contextlib.redirect_stdout(buffer):
+                got = run_check(check, root, options)
+            output = buffer.getvalue()
+            if got != want_exit or want_text not in output:
+                wrong.append(label)
+                print(f"    {label} 这一格判错：期望退 {want_exit}、输出含「{want_text}」，实测退 {got}：{output.strip()[-300:]}")  # gate-lint:detail
+
+        green = make_root("green")
+        expect("toolchain-green", "toolchain", green, 0, "个包的版本、出处与源码指纹都与登记相同")
+        expect("proof-green", "proof", green, 0, "2 verified、0 errors")
+        expect("vacuity-green", "vacuity", green, 0, "2 处 requires")
+        expect("must-red-green", "must-red-worlds", green, 0, "1 行每行改坏之后")
+        expect("toolchain-commit", "toolchain", make_root("commit", version_commit=other_commit), 1, "Verus 安装的 commit 是")
+        expect("toolchain-lock", "toolchain", make_root("lock", lock_version="0.0.2"), 1, "Cargo.lock 里 vstd 的版本")
+        json.dump({"verus": {"version": "0.9.9.tag", "commit": commit, "toolchain": "stable"}}, open(os.path.join(install, "version.json"), "w"))
+        tampered = make_root("tampered")
+        open(os.path.join(package_directory, "src", "lib.rs"), "a").write("// 改过\n")
+        expect("toolchain-fingerprint", "toolchain", tampered, 1, "源码内容指纹是")
+        open(os.path.join(package_directory, "src", "lib.rs"), "w").write("// vstd\n")
+        expect("kernel-not-wired", "proof", make_root("absent", with_kernel=False, wired=False), 77, "还没进仓")
+        expect("kernel-deleted", "proof", make_root("deleted", with_kernel=False, wired=True), 1, "而 crate 已经接了 Verus")
+        expect("proof-broken", "proof", make_root("broken", kernel=SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_BROKEN")), 1, "Verus 没验过")
+        expect("proof-count", "proof", make_root("count", count=3), 1, "与登记的 3 不同")
+        expect("proof-cheating", "proof", make_root("cheating", kernel=SELFTEST_KERNEL.replace("if value > limit { limit }", "assume(false);\n    if value > limit { limit }")), 1, "不许的标记")
+        expect("proof-timeout", "proof", make_root("timeout", kernel=SELFTEST_KERNEL + "// SELFTEST_SLEEP\n"), 1, "没出结局，按红记")
+        vacuous = SELFTEST_KERNEL.replace("pub fn clamp_to_limit(", "pub fn clamp_to_limit( /* SELFTEST_VACUOUS */")
+        expect("vacuity-vacuous", "vacuity", make_root("vacuous", kernel=vacuous), 1, "空真")
+        expect("vacuity-closure-found", "vacuity", green, 0, "2 处 requires")
+        closure_vacuous = SELFTEST_KERNEL.replace("pub fn keep_small(value: u64) -> (result: u64)\n",
+                                                  "pub fn keep_small(value: u64) -> (result: u64)\n    requires\n        value < 100,\n").replace(
+            "            candidate <= 10,", "            candidate <= 10, // SELFTEST_CLOSURE_VACUOUS")
+        expect("vacuity-closure-vacuous", "vacuity", make_root("closurevacuous", kernel=closure_vacuous), 1, "keep_small 里第 20 行起的闭包：函数体头插 assert(false) 照样验得过")
+        expect("crate-extra-dependency", "proof", make_root("extradependency", extra_dependency='singlefs-format = { path = "../format" }\n'), 1, "除 vstd 之外还依赖 singlefs-format")
+        expect("must-red-survives", "must-red-worlds", make_root("survives", must_red=["改注释\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // 只是注释\tpostcondition"]), 1, "照样验过")
+        expect("must-red-anchor", "must-red-worlds", make_root("anchor", must_red=["不在的原文\tcrates/core/src/kernel.rs\tnot present anywhere\tx\tpostcondition"]), 1, "恰好命中一次")
+        expect("must-red-fragment", "must-red-worlds", make_root("fragment", must_red=["片段不对\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\tif value > limit { limit } else { value } // SELFTEST_BROKEN\tassertion failed"]), 1, "没有期望的")
+        expect("must-red-empty", "must-red-worlds", make_root("empty", must_red=[]), 1, "一行都没有")
+        # rlimit：登记的数写上 Verus 命令行（假 Verus 见到的 --rlimit 小于标记要的数就报 rlimit 用尽）；没有 rlimit 行取 DEFAULT_RLIMIT；
+        # 必红世界里只因 rlimit 用尽才报错的那一行不作数、判红（期望片段写成 rlimit 那一句也不算）
+        needs_seven = SELFTEST_KERNEL.replace("result <= limit,", "result <= limit, // SELFTEST_NEEDS_RLIMIT_7")
+        expect("proof-rlimit-registered", "proof", make_root("rlimitregistered", kernel=needs_seven, rlimit=7), 0, "--rlimit 7")
+        expect("proof-rlimit-too-low", "proof", make_root("rlimitlow", kernel=needs_seven, rlimit=3), 1, "Resource limit (rlimit) exceeded")
+        expect("proof-rlimit-default", "proof", make_root("rlimitdefault", kernel=SELFTEST_KERNEL.replace("result <= limit,", f"result <= limit, // SELFTEST_NEEDS_RLIMIT_{DEFAULT_RLIMIT}"),
+                                                          rlimit=None), 0, f"--rlimit {DEFAULT_RLIMIT}")
+        expect("registry-rlimit-zero", "proof", make_root("rlimitzero", rlimit=0), 1, "rlimit 要是正整数")
+        expect("must-red-rlimit", "must-red-worlds", make_root("mustredrlimit", must_red=["只撞 rlimit\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
+                                                                                        "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99\tResource limit"]),
+               1, "这一行红在算力额度上，不是证明失败")
+        # 输出里夹着 rlimit 用尽、另有一条真的证明错：红计入（只剩 rlimit 用尽的才不作数）
+        expect("must-red-rlimit-with-proof-error", "must-red-worlds", make_root("mustredrlimitproof", must_red=[
+            "撞 rlimit 又有证明错\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
+            "if value > limit { limit } else { value } // SELFTEST_NEEDS_RLIMIT_99 SELFTEST_BROKEN\tpostcondition not satisfied"]),
+               0, "1 行输出里夹着 rlimit 用尽、另有证明错，红计入")
+        # 函数上的 rlimit 属性盖过命令行的 --rlimit：核里有它，proof 判红；必红世界的替换文里有它，那一行判红
+        expect("proof-rlimit-attribute", "proof", make_root("rlimitattribute", kernel=SELFTEST_KERNEL.replace(
+            "pub fn clamp_to_limit(", "#[verifier::rlimit(20)]\npub fn clamp_to_limit(")), 1, "verifier::rlimit（函数上的 rlimit 属性盖过命令行的 --rlimit 7")
+        expect("must-red-rlimit-attribute", "must-red-worlds", make_root("mustredrlimitattribute", must_red=[
+            "加 rlimit 属性\tcrates/core/src/kernel.rs\tpub fn clamp_to_limit(\t#[verifier::rlimit(20)] // SELFTEST_BROKEN\\npub fn clamp_to_limit(\tpostcondition not satisfied"]),
+               1, "的替换文里有 verifier::rlimit")
+        # 期望片段写成 Verus 普通输出的判定行：判法带 --output-json 时 Verus 不打它，照 JSON 的 verified / errors 拼回来比；JSON 里缺这两个数的那一行判红
+        expect("must-red-results-line", "must-red-worlds", make_root("mustredresultsline", must_red=[
+            "片段写成判定行\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
+            "if value > limit { limit } else { value } // SELFTEST_BROKEN\tverification results:: 1 verified, 1 errors"]),
+               0, "1 行的期望片段对上的是照 JSON 拼出的 verification results 那一行")
+        expect("must-red-results-line-missing", "must-red-worlds", make_root("mustrednocounts", must_red=[
+            "JSON 缺数\tcrates/core/src/kernel.rs\tif value > limit { limit } else { value }\t"
+            "if value > limit { limit } else { value } // SELFTEST_BROKEN SELFTEST_NO_COUNTS\tpostcondition not satisfied"]),
+               1, "拼不出「verification results:: <V> verified, <E> errors」那一行")
+        # 内存包装自己退 250（撞了自己的上限）：那一处判「没跑成」，出路照包装文件头，不指向证明或必红世界表（开关 wrapper-exit-as-proof-red 下这两格判错）
+        fake_wrapper = os.path.join(work, "fake-memory-cap-wrapper.sh")
+        with open(fake_wrapper, "w", encoding="utf-8") as handle:
+            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 250（撞了这一条自己的上限）\nexit 250\n")
+        saved_runner, saved_launcher = MEMORY_CAP_RUNNER, options["launcher"]
+        MEMORY_CAP_RUNNER, options["launcher"] = fake_wrapper, None
+        try:
+            expect("proof-wrapper-exit", "proof", green, 1, "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 250")
+            expect("must-red-wrapper-exit", "must-red-worlds", green, 1, "→ 怎么办：照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办")
+        finally:
+            MEMORY_CAP_RUNNER, options["launcher"] = saved_runner, saved_launcher
+        # tool-versions 换了 z3、设了 VERUS_Z3_PATH 打的那一行都要变（开关 tool-versions-launcher-only 下这一格判错）
+        import io
+        import contextlib
+
+        def tool_versions_line():
+            buffer = io.StringIO()
+            with contextlib.redirect_stdout(buffer):
+                tool_versions(green, install)
+            return buffer.getvalue()
+
+        cases += 1
+        saved_z3_path = os.environ.pop("VERUS_Z3_PATH", None)
+        try:
+            without_z3 = tool_versions_line()
+            with open(os.path.join(install, "z3"), "w", encoding="utf-8") as handle:
+                handle.write("# 自证用的假 z3\n")
+            with_z3 = tool_versions_line()
+            os.environ["VERUS_Z3_PATH"] = zip_path
+            with_override = tool_versions_line()
+        finally:
+            os.environ.pop("VERUS_Z3_PATH", None)
+            if saved_z3_path is not None:
+                os.environ["VERUS_Z3_PATH"] = saved_z3_path
+        if len({without_z3, with_z3, with_override}) != 3:
+            wrong.append("tool-versions-z3")
+            print(f"    tool-versions-z3 这一格判错：换 z3、设 VERUS_Z3_PATH 之后打的行应当各不相同，实测：{without_z3.strip()} | {with_z3.strip()} | {with_override.strip()}")  # gate-lint:detail
+
+    breaks_missed, breaks_proven = [], 0
+    if not wrong and not BREAK:
+        for item in ("commit-unchecked", "fingerprint-unchecked", "absent-kernel-not-run", "cheating-unchecked", "count-unchecked", "timeout-as-pass",
+                     "vacuity-accepts-verified", "vacuity-all-at-once", "mutation-accepts-green", "anchor-unchecked", "rlimit-counts-as-red", "rlimit-unpinned",
+                     "rlimit-any-line-voids", "rlimit-attribute-unchecked", "mutation-rlimit-attribute-accepted", "expect-raw-json-only",
+                     "wrapper-exit-as-proof-red", "tool-versions-launcher-only"):
+            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
+                                       env=dict(os.environ, VERUS_PROOF_CHECK_BREAK=item))
+            if completed.returncode == 0:
+                breaks_missed.append(item)
+            else:
+                breaks_proven += 1
+    if wrong:
+        report_red(f"verus-proof-check 自证 {cases} 格里 {len(wrong)} 格判错：{' '.join(wrong)}", "按判错那几格的标签找对应的 check_* 函数修；判据写在文件头")
+        return EXIT_RED
+    if breaks_missed:
+        report_red(f"弄坏开关打开之后自证照样全对：{' '.join(breaks_missed)}", "那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
+        return EXIT_RED
+    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not BREAK else "（开着弄坏开关，没重跑开关）"
+    print(f"  ✓ verus-proof-check 自证 {cases} 格都判对{suffix}")
+    return EXIT_GREEN
+
+
+if __name__ == "__main__":
+    preflight(__file__)
+    sys.exit(main(sys.argv[1:]))
```
