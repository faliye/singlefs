# 验证两档拆分 第一轮：这一轮的 diff（git diff HEAD -M，只含这一轮的路径；kb 里别的改动与别的会话的文件不在内）

```diff
 research/scripts/admission.py                      | 197 +++++++++++++++++++--
 research/scripts/crash-case-check.py               |  54 +++---
 39 files changed, 524 insertions(+), 226 deletions(-)

diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index cba6d953..2b2449db 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -44,8 +44,8 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 `cargo test`，按轻阶段对待；逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
-- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
+- 重型测试就是 checker 档那一批（`.claude/rules/verification.md`）：`crates/singlefs-checker` 包里的测试（`cargo test -p singlefs-checker`、不挑包而范围含它、直接执行它的测试二进制）与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置，只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑 harness 档：自己动到的 `singlefs-harness` 与别的非 checker 包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 `cargo test`，按轻阶段对待；逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，`singlefs-checker` 包的测试二进制也不许。
 - 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index 99b87e94..958696f4 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -19,15 +19,15 @@ omitClaudeMd: true
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
-- 54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
+- 54 号跑哪一档：默认快档（`cargo test --release -p singlefs-checker` 加逐条核全绿标记，`.claude/rules/verification.md`），派发提示什么都不写就是它；要跑全量的写明「54 号带 --full」与在哪棵 worktree 里跑（HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建）、命令前缀 `SINGLEFS_HEAVY_TESTS=user-request`；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
 - 报告路径与草稿目录。
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 54 号，等它结束。55、57、59 在主工作区跑，判的是工作区那一份：每道开跑前取它自己的输入（55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，55 号按名字读的 `research/results/` 产物这里不查，由整轮门禁里的 87 号兜；57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层；要没有输出，别处的未跟踪文件不挡；57 号读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑、或有不带 `--selftest` 的 `research/scripts/layer0-shard-run.sh` 在跑时，不起 54 号，等它结束。55、57、59 在主工作区跑，判的是工作区那一份：每道开跑前先 `python3 research/scripts/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（55、59 号是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，57 号是 `litmus crates .claude/scripts/lkmm.sh`，三道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）与 `git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，55 号按名字读的 `research/results/` 产物这里不查，由整轮门禁里的 87 号兜；57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标，仓根有 `.lkmm-static-only` 时 57 号只跑静态那几层；要没有输出，别处的未跟踪文件不挡；57 号读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 在被 git 忽略的规范副本里，git 核不到，不在这一步里）。两样有一样不过就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」，原样贴两条命令的输出交主 agent；主 agent 照「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再派。
    1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
    1c. 54 号 `--full` 开跑前在那棵 worktree 里跑 `bash research/scripts/layer0-shard-configuration-check.sh --emit-assignments <worktree>` 与 `awk -F'\t' '$1 ~ /^crash-case:/ && $3 ~ /shard=across-machines/ { print $1 }' <worktree>/.claude/gate.d/stage-inputs.tsv`，两样原样抄进报告：前一条退 0 是双机分片开着，抄它打的 `PEER_SSH_HOST=` 与 `PEER_MEMORY_CAP=` 两行（第二台那一片的内存上限：驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 用它起那一片，本机那一片与 merge 由第 1b 步的包装管），退 1 是关着、照单机跑，抄它那一句原因；后一条列出的是开着时两台各跑一片的用例。驱动只由 54 号 `--full` 调（`--merged-log`），不单独跑它。
-2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`，经内存包装照第 1b 步）：它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段：55、57 号与 54 号并行起（各自后台、各自内存上限，照第 1b 步），59 号等 54 号跑完再起（整表复跑与层 0 全量争 CPU），其余轻阶段一次一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认跑快档（`bash .claude/gate.d/54-layer0-replay.sh`，经内存包装照第 1b 步：release 下跑 `singlefs-checker` 包不标 ignored 的用例，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，你原样抄进报告）；派发提示写明「54 号带 --full」时才在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑全量（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`）：它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（第一个事务的干跑） 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。双机分片跑的那几条另抄驱动打的 ①（两台工具链相同）、③（两台输入指纹相同）、④（两片开跑那一行，写着第二台那一片经 `run-with-memory-cap.sh <上限>` 起）、⑤（账本拷回）、⑥（三份发现日志）各一行；判绿要这几行都在、54 号判 merge 那一趟的日志绿；第二台那一片退 250–254 是内存包装自己的结局，照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条办，那一条用例这一趟的判定不算。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index 7223b380..2824448e 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
    1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 54、55、57、59、74、87 号才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，下一次 54、55、57、59、74、87 号才复用得上，判据在 `research/scripts/stage-must-run.sh` 文件头）；主 agent 明写全量的跑 `bash .claude/scripts/gate.sh` 不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 54 号脚本本身（`.claude/gate.d/54-layer0-replay.sh`；全绿标记的键还含它与 `cargo -V`、`rustc -V`） ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；✗ 行写着内存包装退 25x 的（15、74 号的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）不按它点名的文件判：250 ⇒「环境（上限）」，252 与 254 ⇒「并发」，251 与 253 ⇒「环境」，原样抄阶段的 → 与它上面包装自己打的 ✗、→；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 2190845a..633f66df 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -31,7 +31,7 @@ required-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红（经内存包装，共用约束「不做」一节），记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
    3a. 证红一律用 `bash research/scripts/prove-red.sh --copy <副本> [--memory <上限>] <crate> <变异名…>`：先把变异行写进 `crates/mutations.tsv`（参数带 `-p <crate>` 与 `--lib` / `--test <目标>` / `--bin <名>` 之一），它逐条施加、经内存包装跑、判红、还原；不挑目标的行它整次拒，目标带 layer0 的跳过并列出。不自己写证红脚本。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。checker 档不跑（`.claude/rules/verification.md`）：自己新加的崩溃枚举流写进 `crates/singlefs-checker/tests/`，不在交回前跑，随提交时的 54 号快档与之后的全量验；已有的一条都不跑；新测试落在 `singlefs-checker` 包里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。checker 档由提交时的崩溃验证员跑（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的 harness 档测试二进制（整个二进制，不按名字挑；`singlefs-checker` 包的不跑）、第 3 步的证红、`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 加 `.claude/singlefs-ai-sop/scripts/check.sh` 里 `CODE_DISCIPLINE_LINTS` 那几条 `-D`（照那份脚本现抄，不跑 `check.sh` 本身，它是重型）、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、checker 档的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
    4a. 改了 checker（`crates/singlefs-checker/src/`）某条不变量的判定集合的，报告单列一节「受影响的层 0 流与崩溃枚举用例」：哪几条流、哪几个用例的钉值会跟着变；层 0 快档是重型（整轮门禁的 54 号），集成时不跑，主 agent 把这一节交提交时的 `crash-verifier`。`research/scripts/apply-writer-patch.py` 见补丁动了 checker 而报告没这一节就拒绝打。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
@@ -39,7 +39,7 @@ required-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件
    - 在报告里写出「为什么走不到」（哪条构造保证、哪几个调用点）之后，才许写 `todo!` 或 `assert!`。
      条款没写的不是分支（trait 实现、derive、访问器），不加，逐项列进报告；其余照做。
 7. 新加层 0 流或崩溃点重放用例时，报告写明它比已有的流多罩了哪些崩溃状态、多跑了哪一步（重开、挂载、恢复）；与已有流的基线镜像、写表、段序列逐项相同的，不新开全量枚举，只加一条快用例钉住「相同」。
-   7a. 新写的崩溃枚举用例（测试函数直接调 `enumerate_layer0` 一族做全量枚举、不是 `quick_tier` 那几个的）不在名字带 layer0 的测试二进制里的，一律标 `#[ignore]`，报告里给出要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行原文，由主 agent 或 tooling-writer 登记；几步的合成流、单跑几秒的，在测试函数上面写一行注释 `// crash-case-check:not-a-crash-case <理由>`。`research/scripts/crash-case-check.py` 判这一条。
+   7a. 新写的崩溃枚举用例（测试函数直接调 `enumerate_layer0` 一族做全量枚举、不是 `quick_tier` 那几个的）一律写在 `crates/singlefs-checker/tests/` 里、标 `#[ignore]`，共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness（`.claude/rules/verification.md`「崩溃枚举用例住哪、怎么登记」），报告里给出要登记进 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行原文（`test=singlefs-checker:…`），由主 agent 或 tooling-writer 登记；几步的合成流、单跑几秒的，在测试函数上面写一行注释 `// crash-case-check:not-a-crash-case <理由>`。`research/scripts/crash-case-check.py` 判这一条。
 8. 上下文过 600k：停在最近一个编得过的点，报告写做完的件与没做的件，交回；不硬撑到被自动压缩。
 
 ## 写范围
@@ -54,4 +54,4 @@ required-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件
 
 ## 没做什么（固定会有的）
 
-- 没走三方对抗；层 0 全量、QEMU、herd7 与 crates 变异表归 `crash-verifier`（层 0 快档在整轮门禁里）；没提交。
+- 没走三方对抗；checker 档（54 号快档与全量、QEMU、herd7、crates 变异表）归 `crash-verifier`；没提交。
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index 7337b8ed..f8cd3150 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -27,7 +27,7 @@ required-inputs: 草稿目录, 报告, research/mutations/|crates/mutations.tsv
 
 1. 开跑前照共用约束「不做」一节看负载。
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
-3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
+3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句，替换在草稿目录里那份探针的副本上做、跑副本，不动仓里的探针（要 dm / loop 设备才跑得起来的，写明没跑、为什么，交主 agent）；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
 4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号没有这句，看它收尾有没有「计数：」行（中途退出时它先打一句 ✗ 就退，不打计数行），有就核「共 N 条」与表的条数对得上，编不过的、跑到点名测试之前进程就被杀的、点名的名字认不出的在「无效」一栏（第 5 步那一句分得开几种），「没跑到」算进没红；对不上就是中途退出，这一次的数不算，照实报。
 5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，按每条结果行开头的符号数（`[ ]` 里是变异名，不是结局）：`✅` 抓到、`⏭` 无效、`❌` 没红；`💥`（测试进程没跑完）、`⚠️`（报了失败却一个测试名都没抓到）、`⏱`（超时）、`🧱`（内存撞顶）另列，几栏加起来要等于表的条数；`mutate.sh` 收尾那行「计数：」只报内存撞顶与超时两栏；59 号照它自己收尾的「计数：」行，那一行没有 `💥`、`⚠️` 两栏，每一条按这个次序判：点名的测试那一行是 `FAILED` 记抓到；那一行出现了（跑了没红，或跑到它、跑完它之后进程被杀）记没红；那一行没出现、而输出里有一行 `error:` 开头或 `could not compile` 记无效——替换文编不过，或测试进程在跑到点名的测试之前就被杀（尾巴带 `process didn't exit successfully` 与信号），或点名的名字认不出（名字里带 `$` 这类字符，59 号按字面去找）；都没有记「没跑到」，算没红；无效、没红都让整道判红；另外三栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」不为 0 也让整道判红，照原样另列），其余几栏不并进三个数：`⚠️`、`⏱`、`🧱` 不为 0 的整轮已判失败，`mutate.sh` 的 `💥` 不判失败，逐条列出交主 agent；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
diff --git a/.claude/batch-scope b/.claude/batch-scope
index 5949d23a..fbdf3d2d 100644
--- a/.claude/batch-scope
+++ b/.claude/batch-scope
@@ -38,3 +38,19 @@ research/scripts/agent-watch.py	# 看门狗：管道尾不误报、每跨一个
 research/scripts/capped.sh	# 新立：线程上限包装脚本（用户 2026-09-24）
 research/scripts/watch.conf	# 看门狗新键 ask-every-minutes
 research/scripts/watch.sh	# 看门狗配置白名单加新键
+.claude/agent-common.md	# 验证两档拆分：重型测试那一条改成 checker 档按包判（records/2026-09-27-验证两档拆分.md）
+.claude/agents/crash-verifier.md	# 验证两档拆分：54 号默认快档、--full 只在用户要求或夜间
+.claude/agents/gate-triage.md	# 验证两档拆分：54 号快档核标记不作数报本次未跑
+.claude/agents/implementation-writer.md	# 验证两档拆分：崩溃枚举用例写进 checker 包、交回前只跑 harness 档
+.claude/gate.d/54-layer0-replay.sh	# 验证两档拆分：快档改跑 cargo test -p singlefs-checker，标记不作数报本次未跑不判红
+.claude/gate.d/94-checker-implementation-disjoint.sh	# 验证两档拆分：不读 [dev-dependencies]（checker 包 tests/ 经它依赖 core 与 harness）
+.claude/gate.d/stage-inputs.tsv	# 验证两档拆分：crash-case 行改包名 singlefs-checker，补登记位置寻址那条放量用例
+.claude/hooks/heavy-test-guard.sh	# 验证两档拆分：checker 档两个 kind 放行给崩溃验证员，规矩句改
+.claude/hooks/lib_heavy_tests.py	# 验证两档拆分：checker 档按包判，排在层 0 与崩溃枚举用例那两条之前
+.claude/rules/implementation-workflow.md	# 验证两档拆分：「重型测试只在提交时跑」一节改成两档、指向 verification.md
+.claude/rules/verification.md	# 验证两档拆分：新立，harness 与 checker 的定义、快档与全量、从上游收回的四节
+CLAUDE.md	# 验证两档拆分：规则清单加 verification.md
+crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	# 验证两档拆分：注释里的测试文件路径改到 checker 包（path-moves）
+crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs	# 验证两档拆分：文件头里的测试文件路径改到 checker 包（path-moves）
+research/scripts/admission.py	# 验证两档拆分：54 号自证的快档期望改成报本次未跑
+research/scripts/crash-case-check.py	# 验证两档拆分：判法改成崩溃枚举用例住 checker 包、标 ignore 的要登记
diff --git a/.claude/gate.d/54-layer0-replay.sh b/.claude/gate.d/54-layer0-replay.sh
index 6b02d5a5..be9bd5df 100755
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -1,12 +1,13 @@
 #!/usr/bin/env bash
 # admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
 # run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 research/scripts/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）
-# gate-stage: 层 0 崩溃点重放与登记的崩溃枚举用例（整轮门禁跑快档：两条流不标 ignored 的用例，再逐条核 stage-inputs.tsv 里 crash-case: 那几条用例各自那一格全绿标记与它这批输入的指纹相等；全量由主 agent 暂存之后在 HEAD + 暂存区的 worktree 里跑 --full，逐条用例照复用判定跑：那一格全绿标记在就复用，不在才在 release 下跑它、判绿写那一格；两条流的层 0 全量带断点续跑，第一个事务与 E142 产物逐字比对、里程碑「第二个事务」固定脚本到 E 与用例的闭式比对由用例自己断言；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
+# gate-stage: 层 0 崩溃点重放与登记的崩溃枚举用例（整轮门禁与提交时跑快档：checker 包（crates/singlefs-checker，D13 已定项 15）不标 ignored 的用例，再逐条核 stage-inputs.tsv 里 crash-case: 那几条用例各自那一格全绿标记与它这批输入的指纹相等，不作数的报「本次未跑」、不判红；全量由用户要求或夜间在 HEAD + 暂存区的 worktree 里跑 --full，逐条用例照复用判定跑：那一格全绿标记在就复用，不在才在 release 下跑它、判绿写那一格；两条流的层 0 全量带断点续跑，第一个事务与 E142 产物逐字比对、里程碑「第二个事务」固定脚本到 E 与用例的闭式比对由用例自己断言；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
 # gate-covers: 崩溃点重放
 #
 # 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问；
 # 用户 2026-09-26 定逐条用例复用，原话「下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」，
-# records/2026-09-24-里程碑二收尾调度.md 第三节「崩溃枚举的跑法」那一行）：
+# records/2026-09-24-里程碑二收尾调度.md 第三节「崩溃枚举的跑法」那一行；
+# 用户 2026-09-27 定验证代码分两档、提交时默认只跑快档，原话「checker可以自己跑，但是默认只有在提交时候才跑」，records/2026-09-27-验证两档拆分.md）：
 #
 #   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full [--start-over] <worktree>
 #     主 agent 暂存之后（提交时由崩溃验证员），在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
@@ -44,9 +45,11 @@
 #     （范围那一问不摘掉它）。
 #   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
 #     快档先核登记的每一条路径 git 至少列得出一个文件（git ls-files -co --exclude-standard -- <那一条>），有一条列不出判红，之后才问复用与改动范围。
-#     两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再逐条崩溃枚举用例算它这批输入的指纹、核那一格
+#     checker 包在 release 下只跑不标 ignored 的用例（cargo test --release -p singlefs-checker，崩溃枚举用例都住那个包的 tests/），一条都没通过判红；
+#     再逐条崩溃枚举用例算它这批输入的指纹、核那一格
 #     （admission.py crash-case-marker-check：在、记的指纹与用例相同、test result 是 1 passed、登记的计数行各恰好一行、要 exhaustive=true 的带着），
-#     全部作数才判绿，成功句逐条原样带出那一格的计数行与时刻；有一条不作数判红，逐条列原因（没有那一格时比最近写的一格与这一次的清单），出路是跑 --full。
+#     全部作数成功句逐条原样带出那一格的计数行与时刻；有不作数的逐条列原因（没有那一格时比最近写的一格与这一次的清单）、每条往 GATE_NOT_RUN_FILE 报一行「本次未跑」，
+#     不判红、退 0（全量默认不在提交时跑，D13 已定项 15；报过本次未跑的这一轮不算覆盖崩溃点重放），出路是跑 --full。
 #     按整批输入分格的旧标记（singlefs-layer0-full-green.*）不再认。
 #
 # 输入：整道阶段的复用判定与改动范围按 stage-inputs.tsv 里本阶段那一行（唯一登记位）；每条崩溃枚举用例的输入按它自己那一行，
@@ -217,11 +220,11 @@ delete_crash_case_marker() {
   return 0
 }
 
-# run_layer0_test_binary <测试二进制> <日志>：快档跑一条流不标 ignored 的用例；cargo 的整段输出进日志。
+# run_checker_package_tests <日志>：快档跑 checker 包不标 ignored 的全部用例（崩溃枚举用例住那个包的 tests/，D13 已定项 15）；cargo 的整段输出进日志。
 # 快档不写发现日志：调用方环境里的 SINGLEFS_LAYER0_FINDINGS_FILE 清掉，不漏给快用例。
-run_layer0_test_binary() {
-  env -u SINGLEFS_LAYER0_FINDINGS_FILE cargo test --release -p singlefs-harness --test "$1" -- --nocapture 2>&1 \
-    | tee "$2" \
+run_checker_package_tests() {
+  env -u SINGLEFS_LAYER0_FINDINGS_FILE cargo test --release -p singlefs-checker -- --nocapture 2>&1 \
+    | tee "$1" \
     | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
     | sed -u 's/^/    /'
   return "${PIPESTATUS[0]}"
@@ -381,34 +384,37 @@ print_staged_worktree_full_commands() {
   echo '                if [ "$layer0_tree_ready" = 1 ]; then SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "${layer0_full_base:?}/tree/.claude/gate.d/54-layer0-replay.sh" --full "${layer0_full_base:?}/tree"; layer0_full_rc=$?; else echo "worktree 没建好或暂存区的 diff 套不上，--full 没跑"; layer0_full_rc=1; fi; git worktree remove --force "${layer0_full_base:?}/tree" 2>/dev/null; rm -rf "${layer0_full_base:?}"; ( exit "$layer0_full_rc" )'
 }
 
-# run_quick_tier_of_stream <测试二进制> <流的名字>：快档跑一条流。判红打出路、返回 1；判绿把这条流的计数接到 quick_tier_report 后面。
-run_quick_tier_of_stream() {
-  local quick_log passed_and_ignored passed_count ignored_count
-  quick_log="$layer0_scratch_directory/quick-$1.log"
-  if ! run_layer0_test_binary "$1" "$quick_log"; then
+# run_checker_quick_tier：快档跑 checker 包不标 ignored 的用例。判红打出路、返回 1；判绿把每个测试二进制那一行 test result 的 passed / ignored 加总进 quick_tier_report。
+run_checker_quick_tier() {
+  local quick_log result_lines passed_count ignored_count passed_total ignored_total binary_count
+  quick_log="$layer0_scratch_directory/quick-singlefs-checker.log"
+  if ! run_checker_package_tests "$quick_log"; then
     tail -40 "$quick_log"
-    echo "  ✗ $2的快档用例判红（上面是 cargo test 的尾部）"
-    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $1 -- --nocapture"
+    echo "  ✗ checker 档快档判红（上面是 cargo test 的尾部）"
+    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-checker -- --nocapture（只看一个测试二进制加 --test <名>）；"
     echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
     return 1
   fi
-  passed_and_ignored="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log" | head -1)"
-  read -r passed_count ignored_count <<< "$passed_and_ignored"
-  if [[ -z "${passed_count:-}" || "$passed_count" == 0 ]]; then
-    echo "  ✗ $2的快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
-    echo "     → 怎么办：cargo test --release -p singlefs-harness --test $1 -- --list 看这个测试二进制里还剩几条不标 ignored 的用例；"
+  result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log")"
+  passed_total=0; ignored_total=0; binary_count=0
+  while read -r passed_count ignored_count; do
+    [[ -n "${passed_count:-}" ]] || continue
+    passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
+  done <<< "$result_lines"
+  if (( passed_total == 0 )); then
+    echo "  ✗ checker 档快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
+    echo "     → 怎么办：cargo test --release -p singlefs-checker -- --list 看这个包里还剩几条不标 ignored 的用例；"
     echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
     return 1
   fi
-  quick_tier_report+="${quick_tier_report:+；}$2 ${passed_count} 条通过、${ignored_count} 条 ignored"
+  quick_tier_report="checker 包 ${binary_count} 个测试二进制，${passed_total} 条通过、${ignored_total} 条 ignored（全量那几条留给 --full）"
   return 0
 }
 
-# ── 快档：两条流不标 ignored 的用例，再逐条核崩溃枚举用例这批输入那一格全绿标记 ─────────
+# ── 快档：checker 包不标 ignored 的用例，再逐条核崩溃枚举用例这批输入那一格全绿标记 ─────────
 if [[ "$layer0_tier" == quick ]]; then
   quick_tier_report=""
-  run_quick_tier_of_stream first_transaction_step_seven_layer0 "第一个事务那条流" || exit 1
-  run_quick_tier_of_stream second_transaction_step_zero_layer0 "两次发布那条流" || exit 1
+  run_checker_quick_tier || exit 1
   present_report="$layer0_scratch_directory/present-report"
   missing_report="$layer0_scratch_directory/missing-report"
   : > "$present_report"
@@ -436,20 +442,24 @@ if [[ "$layer0_tier" == quick ]]; then
       } >> "$missing_report"
     fi
   done
+  echo "  ✓ checker 档快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
   if (( ${#missing_cases[@]} > 0 )); then
-    echo "  ✗ 快档绿了（${quick_tier_report}），但 ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记：${missing_cases[*]}"
+    echo "  ! ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记，全量这一轮没跑，记「本次未跑」、不判红（全量默认不在提交时跑，D13 已定项 15）：${missing_cases[*]}"
     cat "$missing_report"
     legacy_marker_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green*' 2>/dev/null | grep -c .)"
     if [[ "$legacy_marker_total" -gt 0 ]]; then
       echo "       common-dir 里还有 ${legacy_marker_total} 格按整批输入分格的旧标记（singlefs-layer0-full-green*）：分成逐条用例之后不再认，可以删掉。"
     fi
-    echo "     → 怎么办：这几条的输入自上一次判绿以来变了（或从来没跑过）。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>"
+    echo "     → 要跑全量（用户要求或夜间）：暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>"
     echo "                （与 gate.sh --staged 同一建法；它只跑没有作数标记的那几条，别的复用）："
     print_staged_worktree_full_commands
-    exit 1
+    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
+      for missing_case in "${missing_cases[@]}"; do
+        echo "崩溃枚举用例 ${missing_case} 这批输入没有作数的全绿标记，全量没跑（要跑：54 号 --full）" >> "$GATE_NOT_RUN_FILE"
+      done
+    fi
   fi
-  echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
-  echo "  ✓ ${#crash_case_rows[@]} 条崩溃枚举用例的全绿标记都与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去用例读不到的文件），标记里的计数行原样："
+  echo "  ✓ 登记 ${#crash_case_rows[@]} 条崩溃枚举用例，$(( ${#crash_case_rows[@]} - ${#missing_cases[@]} )) 条的全绿标记与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去用例读不到的文件），标记里的计数行原样："
   cat "$present_report"
   exit 0
 fi
diff --git a/.claude/gate.d/94-checker-implementation-disjoint.sh b/.claude/gate.d/94-checker-implementation-disjoint.sh
index 2789be82..8eef4271 100755
--- a/.claude/gate.d/94-checker-implementation-disjoint.sh
+++ b/.claude/gate.d/94-checker-implementation-disjoint.sh
@@ -17,8 +17,9 @@
 #      （它后面紧跟的那一个 item：花括号配平到收尾，或一行以分号收尾），正文不许有分支或循环
 #      （`if` / `match` / `for` / `while` / `loop`）——它只许发射标量与纯算术。`#[cfg(test)]` 之后的别的 item 照扫。
 #
-# 依赖表怎么读（①②两条都靠它）：`[dependencies]`、`[dev-dependencies]`、`[build-dependencies]` 与
-# `[target.'…'.dependencies]` 这几种节头都认；`[dependencies.X]` 这种按 crate 开的表认成 X；
+# 依赖表怎么读（①②两条都靠它）：`[dependencies]`、`[build-dependencies]` 与 `[target.'…'.dependencies]` 这几种节头都认，
+# `[dev-dependencies]` 不认——它只编进 tests/ 与 examples/，库本身链不到；checker 档的崩溃枚举用例就住在 checker 包的 tests/ 里、
+# 经 dev-dependencies 依赖 core 与 harness 造镜像再判（D13（验证路线） 已定项 15），那不是库与实现共享代码；`[dependencies.X]` 这种按 crate 开的表认成 X；
 # `X.workspace = true` 的键认成 X；`别名 = { package = "真名", … }` 认成真名——闭包按真名求交，
 # 而 checker 源码里用那个别名（`别名::…`）引实现，与引 `singlefs_core` 同样判红。
 # 根目录 `Cargo.toml` 的 `[workspace.dependencies]` 里给某个键写了 `package =` 改名的，`X.workspace = true` 按那里的真名认。
@@ -59,8 +60,9 @@ def fail(message, steps):
         print(f"     → {step}")
     sys.exit(1)
 
-DEPENDENCY_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:dev-|build-)?dependencies\]$")
-DEPENDENCY_ITEM_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:dev-|build-)?dependencies\.([A-Za-z0-9_\-]+)\]$")
+# dev-dependencies 不进依赖图：只编进 tests/ 与 examples/，库链不到（D13（验证路线） 已定项 15）
+DEPENDENCY_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:build-)?dependencies\]$")
+DEPENDENCY_ITEM_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:build-)?dependencies\.([A-Za-z0-9_\-]+)\]$")
 PACKAGE_FIELD = re.compile(r"\bpackage\s*=\s*\"([^\"]+)\"")
 
 def workspace_renames():
diff --git a/.claude/gate.d/fixtures/94-checker-implementation-disjoint.sh/green/crates/singlefs-checker/Cargo.toml b/.claude/gate.d/fixtures/94-checker-implementation-disjoint.sh/green/crates/singlefs-checker/Cargo.toml
index 0445df9a..8578b1bb 100644
--- a/.claude/gate.d/fixtures/94-checker-implementation-disjoint.sh/green/crates/singlefs-checker/Cargo.toml
+++ b/.claude/gate.d/fixtures/94-checker-implementation-disjoint.sh/green/crates/singlefs-checker/Cargo.toml
@@ -3,3 +3,7 @@ name = "singlefs-checker"
 
 [dependencies]
 singlefs-format = { path = "../singlefs-format" }
+
+# 样本：dev-dependencies 只编进 tests/，不算共享（D13 已定项 15）
+[dev-dependencies]
+singlefs-core = { path = "../singlefs-core" }
diff --git a/.claude/gate.d/stage-inputs.tsv b/.claude/gate.d/stage-inputs.tsv
index a95c1d24..111b5823 100644
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -25,7 +25,7 @@
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
 # 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/admission.py research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh	command=cargo command=rustc	# 跑 crates 的崩溃点重放；research/scripts/admission.py 是它判日志、算每条崩溃枚举用例的输入指纹、读写全绿标记、交起用例命令的准入模块（进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份），改了它这一道不许复用上一次的判定；layer0-shard-run.sh 与 layer0-shard-configuration-check.sh 是 --full 在分片配置可用时跑登记了 shard=across-machines 的用例的驱动脚本与配置判法（按内容进那几条用例的指纹），改了它们快档同样要核标记；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/admission.py research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh	command=cargo command=rustc	# 跑 crates 的崩溃点重放；research/scripts/admission.py 是它判日志、算每条崩溃枚举用例的输入指纹、读写全绿标记、交起用例命令的准入模块（进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份），改了它这一道不许复用上一次的判定；layer0-shard-run.sh 与 layer0-shard-configuration-check.sh 是 --full 在分片配置可用时跑登记了 shard=across-machines 的用例的驱动脚本与配置判法（按内容进那几条用例的指纹），改了它们快档同样要核标记；崩溃枚举用例都在 crates/singlefs-checker/tests/（D13 已定项 15，.claude/rules/verification.md），快档是 cargo test --release -p singlefs-checker 不带 --ignored；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
 55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	command=qemu-system-x86_64 readwrite=/dev/kvm probe=research/scripts/vm-kernel.sh:--check	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/；这几条路径同时是改动范围（change-touches-crates.sh）的前缀。前提是 .claude/kb/vm-harness.md「三个前置」：缺 QEMU 装 qemu-system-x86；/dev/kvm 不可读写查 kvm 组成员身份，不许用 setfacl 补；找不到可读的内核镜像就跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL
 57-lkmm.sh	litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/	probe=.claude/scripts/lkmm.sh:--herd7-version environment=.claude/scripts/lkmm.sh:--herd7-version	# 判 litmus/ 下每条 Never 的对照组、代码绑定与 herd7 判定。lkmm.sh 是判法，它 source 的 lib.sh、找不到内核树时调的 fetch-deps.sh 一并登记；crates/ 整个取：litmus 的 singlefs-models 锚点今天指 crates/singlefs-core/src/transaction.rs 与 recovery.rs，读 litmus 文件名的测试在 crates/singlefs-harness/tests/publish_order_matches_litmus.rs，而 lkmm.sh 按文件名在全部 crates/**/*.rs 里找测试、新加一条 litmus 的锚点可以指到 crates/ 任何一处，只登记这三份会让下一条新锚点的改动被跳过（宁宽勿窄）。herd7 的版本不在 git 树里，经第三列 environment= 进复用判定（lkmm.sh --herd7-version 打的那一行，57 号判绿之后记进 git common-dir）；前提是找得到 herd7（probe= 同一个入口，PATH 里没有就试 opam 的环境；缺了 opam install herdtools7）。内核树的 tools/memory-model 不进判定，靠 stage-must-run.sh 的 24 小时复用上限兜
 59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	command=cargo	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据。前提：cargo（装 Rust 工具链，bash .claude/scripts/env.sh 会报缺什么）
@@ -33,10 +33,11 @@
 87-replay.sh	crates/ Cargo.toml Cargo.lock research/e7-index-bench/ research/scripts/ research/results/	# 复跑入库的实验产物，装置在 research/e7-index-bench/，登记表与脚本在 research/scripts/
 E142	research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs research/e7-index-bench/src/lib.rs research/e7-index-bench/Cargo.toml research/Cargo.toml research/Cargo.lock research/mutations/e142_first_transaction_dry_run.tsv research/results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out .claude/kb/decisions/08-核心索引结构.md .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/16-发布语义.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/layout/01-first-txn.md .claude/kb/decisions/22-单元原子性怎么合成.md .claude/kb/decisions/15-格式冻结政策.md .claude/kb/feature-bits.md research/e7-index-bench/src/bin/e142_region_diff_independent.rs research/mutations/e142_region_diff_independent.tsv crates/ Cargo.toml Cargo.lock research/prompts/e142-r19-prereg.md	# E142（第一个事务的干跑）默认调用（replay.sh 的 driver_e142 那一种）。装置源码、它 use 的 e7_index_bench 库、research 的 Cargo 清单与锁、变异表；driver_e142 传给装置的臂 N15 参照产物；replay.sh 不进（跑完把登记行指到新产物就是改它，进了指纹，新产物一存进来就对不上自己；驱动换了参数而这几条路径没变时要靠强制开关）；跑前登记 research/prompts/e142-r17-prereg.md 第二节被测条款所在的 D8（核心索引结构）、D13（验证路线）、D16（发布语义）、D18（块里携带什么信息）、D23（journal 的角色与格式）五份决策正文，加上宽度对账逐格抄的 layout/01（跑前登记第二节没列它，宁宽勿窄加上）；第十八次跑的跑前登记 research/prompts/e142-r18-prereg.md 第二节另加被测条款 D22（单元原子性怎么合成）、D15（格式冻结政策）与 feature-bits.md，那一次用独立比对 bin e142_region_diff_independent 判改前改后，它与它的变异表一并登记（2026-09-26 主 agent 按设计员报的漏列补上）；crates/ 整个取：driver_e142 编的 e142_first_transaction_write_dump 在 singlefs-harness，它依赖 checker、core、format 另外三个 crate，四个就是整个 crates/，多出来的只有 crates/mutations.tsv；按文件精确取要跟着模块图走，漏一个模块就是该跑的不跑
 E142/layer0	@E142	question-row=research/prompts/m2-keyspace-rerun-questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E142:verdict:control_violations_ok=true product-field=E142:verdict:positive_control_main_geometry_ok=true	# E142 第二段（设了 E142_LAYER0_MAIN 的调用：主臂层 0 整轮），输入与默认调用相同。前提照跑前登记 research/prompts/e142-r17-prereg.md 6.2、6.3：问题单第 6 行判成「对照是好的」之后才跑，且 Q142.26 caught = 12（最新产物判决行 positive_control_main_geometry_ok）；control_violations_ok 是第 6 行够判那一格的计数判定
-crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 shard=across-machines	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑、可双机分片（shard=across-machines：枚举经 Layer0Resume::from_environment 认 SINGLEFS_LAYER0_SHARD）；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
-crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B shard=across-machines	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑、可双机分片（同第一条流）；54 号 --full 原来整批跑的两条之二
-crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言。用例自己不打计数行，count-line= 登记的是它经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 打的那一行 LAYER0_PARALLEL_FINISHED（只调一次，恰好一行），threads= 按它判枚举那一段的工作线程（挂载那一遍自己起线程，不在判里）；没登记 exhaustive=：那一行不带 exhaustive=true
-crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）；计数行 C561_SIGMA_FULL 在评过的状态数等于闭式 2^18（crash::closed_form_state_count 只对 σ 算）时带 exhaustive=true；用例按掩码区间切片多线程跑、自己打一行与 LAYER0_PARALLEL_FINISHED 同形的线程行（不留进度文件：resumed_slices=0），threads= 按它判；不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）
-crash-case:layer0-parallel-line-one-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_parallel_line_one_layer0:full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean count-line=LAYER0_PARALLEL_LINE_ONE exhaustive=LAYER0_PARALLEL_LINE_ONE threads=LAYER0_PARALLEL_LINE_ONE shard=across-machines	# 并行线一那条流（取号 → 暖机两次 → A 一个数据单元 → B 顺序写两个数据单元、两条记录 → C 三个数据单元、三条记录）的层 0 全量，枚举域 12230590578 个状态；带断点续跑、可双机分片（shard=across-machines：枚举经 enumerate_layer0_in_state_slices_or_one_shard 与 Layer0Resume::from_environment 认 SINGLEFS_LAYER0_SHARD，与第一、第二条流同一个入口；一条用例一条流、两片各一份账本）。LAYER0_PARALLEL_LINE_ONE 是计数行、要 exhaustive=true，threads= 按它的 states= 找那一行 LAYER0_PARALLEL_FINISHED。这条全量不逐状态核发布边界（那一格由同文件不标 ignore 的快档核，崩在记录之间的 3 + 12 个状态）
-crash-case:layer0-tree-split-streams	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_supplement_two_tree_split_layer0:full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED count-line=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED threads=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT	# 记账树与中央映射树分裂七条流的层 0 全量（每条只录分裂那一次发布），合计 78905428 个状态；一条流一行计数行，前缀各不相同、每个恰好一行，都要 exhaustive=true（枚举到的状态数等于闭式 1 + (2^(2 × 单元数) − 1) + 3 + 1 + 8）；threads= 按各行的 states= 找 LAYER0_PARALLEL_FINISHED：中央映射根分裂、根降高、记账根分裂三条都是 1048588，两条叶分裂都是 4194316，状态数相同的几条判到的是其中第一行（同一个进程、同一套线程配置）。没登记 shard=across-machines：枚举经 enumerate_layer0_selecting_versions（Layer0Resume::NoProgressFile），不认分片开关；一条用例七条流，分片跑会写七份账本，而 research/scripts/layer0-shard-run.sh 要两片的账本各恰好一份。不留进度文件（没有断点续跑）
-crash-case:crash-injection-fast-tier	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED threads=CRASH_INJECTION_FINISHED threads-variable=SINGLEFS_CRASH_INJECTION_THREADS	# 随机崩溃注入快档（种子基起 24 段、每段 24 步、每段 4 个崩溃状态）：每个崩溃状态上只读恢复与判定、再起可写挂载发一次布跑 checker、挂载途中取号/写行/暖机各一个二次崩溃（代码审阅第 2 条）；debug 下单跑一百来秒，标了 ignore，release 跑；普通 cargo test 由同文件的小快档（头 4 段）守。计数行 CRASH_INJECTION_FINISHED 只调一次、恰好一行；它不打 LAYER0_PARALLEL_FINISHED，跑完那一行自己带 worker_threads= 与 slices=，登记 threads= 拿它判线程；用例读的线程变量是 SINGLEFS_CRASH_INJECTION_THREADS（threads-variable=，crash-case-command 把它设成配的线程数）；抽样，没登记 exhaustive=
+crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 shard=across-machines	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑、可双机分片（shard=across-machines：枚举经 Layer0Resume::from_environment 认 SINGLEFS_LAYER0_SHARD）；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
+crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B shard=across-machines	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑、可双机分片（同第一条流）；54 号 --full 原来整批跑的两条之二
+crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言。用例自己不打计数行，count-line= 登记的是它经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 打的那一行 LAYER0_PARALLEL_FINISHED（只调一次，恰好一行），threads= 按它判枚举那一段的工作线程（挂载那一遍自己起线程，不在判里）；没登记 exhaustive=：那一行不带 exhaustive=true
+crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）；计数行 C561_SIGMA_FULL 在评过的状态数等于闭式 2^18（crash::closed_form_state_count 只对 σ 算）时带 exhaustive=true；用例按掩码区间切片多线程跑、自己打一行与 LAYER0_PARALLEL_FINISHED 同形的线程行（不留进度文件：resumed_slices=0），threads= 按它判；不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）
+crash-case:layer0-parallel-line-one-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_parallel_line_one_layer0:full_enumeration_of_the_parallel_line_one_stream_is_exhaustive_and_clean count-line=LAYER0_PARALLEL_LINE_ONE exhaustive=LAYER0_PARALLEL_LINE_ONE threads=LAYER0_PARALLEL_LINE_ONE shard=across-machines	# 并行线一那条流（取号 → 暖机两次 → A 一个数据单元 → B 顺序写两个数据单元、两条记录 → C 三个数据单元、三条记录）的层 0 全量，枚举域 12230590578 个状态；带断点续跑、可双机分片（shard=across-machines：枚举经 enumerate_layer0_in_state_slices_or_one_shard 与 Layer0Resume::from_environment 认 SINGLEFS_LAYER0_SHARD，与第一、第二条流同一个入口；一条用例一条流、两片各一份账本）。LAYER0_PARALLEL_LINE_ONE 是计数行、要 exhaustive=true，threads= 按它的 states= 找那一行 LAYER0_PARALLEL_FINISHED。这条全量不逐状态核发布边界（那一格由同文件不标 ignore 的快档核，崩在记录之间的 3 + 12 个状态）
+crash-case:layer0-tree-split-streams	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_supplement_two_tree_split_layer0:full_enumeration_of_every_tree_split_stream_is_exhaustive_and_clean count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED count-line=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED count-line=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT count-line=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED exhaustive=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT exhaustive=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED threads=LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED threads=LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT threads=LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT	# 记账树与中央映射树分裂七条流的层 0 全量（每条只录分裂那一次发布），合计 78905428 个状态；一条流一行计数行，前缀各不相同、每个恰好一行，都要 exhaustive=true（枚举到的状态数等于闭式 1 + (2^(2 × 单元数) − 1) + 3 + 1 + 8）；threads= 按各行的 states= 找 LAYER0_PARALLEL_FINISHED：中央映射根分裂、根降高、记账根分裂三条都是 1048588，两条叶分裂都是 4194316，状态数相同的几条判到的是其中第一行（同一个进程、同一套线程配置）。没登记 shard=across-machines：枚举经 enumerate_layer0_selecting_versions（Layer0Resume::NoProgressFile），不认分片开关；一条用例七条流，分片跑会写七份账本，而 research/scripts/layer0-shard-run.sh 要两片的账本各恰好一份。不留进度文件（没有断点续跑）
+crash-case:layer0-position-addressed-tree-streams	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_position_addressed_trees_layer0:full_enumeration_of_every_enumerable_position_addressed_tree_stream_is_exhaustive_and_clean	# 位置寻址树（extent 树两层、分配记录树、没有文件的版本）那几条流的层 0 全量：单元写段全展开、每个两遍恢复 + checker（标了 ignore，release 跑）。用例每条流各打一行 LAYER0_POSITION_ADDRESSED stream=…（同一前缀不止一行），count-line= 登记不了，枚举到的状态数等于闭式、违例为 0 由用例自己断言；没登记 exhaustive= 与 threads=（那一行不带 exhaustive=true，也不打 LAYER0_PARALLEL_FINISHED：枚举经 enumerate_layer0_selecting_versions，Layer0Resume::NoProgressFile）；没登记 shard=across-machines（不认分片开关）。2026-09-27 之前这条放量用例没登记，谁都不跑；验证两档拆分那一轮补登记（records/2026-09-27-验证两档拆分.md）
+crash-case:crash-injection-fast-tier	crates/ Cargo.toml Cargo.lock	test=singlefs-checker:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED threads=CRASH_INJECTION_FINISHED threads-variable=SINGLEFS_CRASH_INJECTION_THREADS	# 随机崩溃注入快档（种子基起 24 段、每段 24 步、每段 4 个崩溃状态）：每个崩溃状态上只读恢复与判定、再起可写挂载发一次布跑 checker、挂载途中取号/写行/暖机各一个二次崩溃（代码审阅第 2 条）；debug 下单跑一百来秒，标了 ignore，release 跑；普通 cargo test 由同文件的小快档（头 4 段）守。计数行 CRASH_INJECTION_FINISHED 只调一次、恰好一行；它不打 LAYER0_PARALLEL_FINISHED，跑完那一行自己带 worker_threads= 与 slices=，登记 threads= 拿它判线程；用例读的线程变量是 SINGLEFS_CRASH_INJECTION_THREADS（threads-variable=，crash-case-command 把它设成配的线程数）；抽样，没登记 exhaustive=
diff --git a/.claude/hooks/heavy-test-guard.sh b/.claude/hooks/heavy-test-guard.sh
index 5156dec5..0a0d542d 100755
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -145,7 +145,7 @@ SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有
 
 # 子 agent 自己那一份（kind 见 lib_heavy_tests.KIND_CATEGORY）；不在表里的子 agent 一样也不许
 AGENT_KINDS = {
-    "crash-verifier": {"layer0-stage", "layer0-cargo", "layer0-binary", "crash-case-cargo", "crash-case-binary", "qemu-stage", "qemu-system",
+    "crash-verifier": {"checker-tier-cargo", "checker-tier-binary", "layer0-stage", "layer0-cargo", "layer0-binary", "crash-case-cargo", "crash-case-binary", "qemu-stage", "qemu-system",
                        "herd7-stage", "lkmm", "herd7", "crates-mutation-stage", "crates-mutation-mutate"},
     "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"},
 }
@@ -545,10 +545,11 @@ def heavy_uses(text, directory, environment=None, frames_above=(), script=None,
                                                  [run._replace(frames=run.frames[len(here):]) for run in inner.uncapped])
     return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)
 
-POLICY = ("→ 规矩：重型测试（层 0、崩溃枚举用例、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；"
-          "子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
+POLICY = ("→ 规矩：重型测试就是 checker 档那一批（.claude/rules/verification.md）：singlefs-checker 包的测试、层 0、崩溃枚举用例、QEMU、herd7、crates 变异整表、"
+          "全量测试、整轮门禁、全部实验复跑、E152 装置，只在提交代码时跑一次、或用户要求时跑；"
+          "子 agent 一律不跑，只跑 harness 档——自己动到的 singlefs-harness 等非 checker 包的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
           "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
-          "→ 各自那一份：crash-verifier 只跑 54、55、57、59 号与它们底下的层 0 测试、登记的崩溃枚举用例、qemu-system、lkmm.sh / herd7、crates 变异整表；"
+          "→ 各自那一份：crash-verifier 只跑 54、55、57、59 号与它们底下的 checker 包测试（快档与登记的崩溃枚举用例）、qemu-system、lkmm.sh / herd7、crates 变异整表；"
           "gate-triage 只跑 `gate.sh` 整轮与 87 号（54、55、57、59 靠「输入没变就复用上一次全绿判定」）；两个都要带那个前缀，都不跑全量 `cargo test`。"
           "`.claude/gate.d/` 下其余阶段不是重型，谁都能跑。\n"
           "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
@@ -706,6 +707,12 @@ def selftest(hook_dir):
         # (说明, agent_type（None 是主 agent）, 命令, 该拒 2 / 该放 0[, 该记几条检出, 该读进去几份脚本]；后两列不写是 0、0)
         cases = [
             ("实现员跑 cargo test --all", writer, "cargo test --all", 2),
+            ("实现员跑 checker 包的测试（checker 档按包判）", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker", 2),
+            ("实现员跑 checker 包点名一个快档目标", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker --test checker_quick_stream", 2),
+            ("实现员跑 checker 库自己的单测（--lib）不算", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker --lib", 0),
+            ("崩溃验证员带前缀经包装跑 checker 包快档", crash, commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker", 0),
+            ("主 agent 不带前缀跑 checker 包", None, "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker", 2),
+            ("主 agent 带 =user-request 跑 checker 包", None, request + "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-checker", 0),
             ("实现员跑层 0 测试目标", writer, "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
             ("实现员跑 check.sh", writer, "bash .claude/scripts/check.sh", 2),
             ("实现员经 capped.sh 跑 --workspace", writer, "bash research/scripts/capped.sh 4 cargo test --workspace", 2),
diff --git a/.claude/hooks/lib_heavy_tests.py b/.claude/hooks/lib_heavy_tests.py
index 315b6556..937f4dca 100644
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -33,7 +33,11 @@ heavy-test-guard.sh（执行前拒绝）与看门狗（research/scripts/agent-wa
 
 认的输入：cargo 命令行（test / t、run / r，以及起测试的 nextest run、miri、llvm-cov、hack、mutants）、按名字认的脚本与门禁阶段、
 qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，
-名字含 layer0 的算层 0。libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
+名字含 layer0 的算层 0。
+**checker 档按包判**（D13 已定项 15，.claude/rules/verification.md）：cargo test 跑到 singlefs-checker 包的集成测试就是重型——-p 点名它、不挑包而范围含它、
+当前目录在它里面裸跑，且不挑目标、或 --tests / --all-targets、或 --test 点名它的测试目标；只跑它的 --lib / --doc / --bins 不算（库自己的单测，harness 档随时跑）；
+直接执行它 tests/ 下测试目标的二进制（名字是 tests/<名>.rs 或 tests/<名>/main.rs 的 <名>）也算。这一条排在层 0 与崩溃枚举用例那两条之前：包对了别的都不用问。
+旧的那几条（名字含 layer0、登记的崩溃枚举用例带 --ignored）只管 harness 这类别的包里的脚本。libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
 --test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列；
 libtest 在 `--` 处停止认选项，`--` 之后的 --list 是过滤词，照样全跑，不算只列。
 崩溃枚举用例（门禁 54 号逐条跑的那几条，登记在 .claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>）：
@@ -85,6 +89,7 @@ class HeavyTest(NamedTuple):
 
 # 每一种重型用法：kind → 它属于哪一类
 KIND_CATEGORY = {
+    "checker-tier-cargo": "checker 档", "checker-tier-binary": "checker 档",
     "layer0-stage": "层 0", "layer0-cargo": "层 0", "layer0-binary": "层 0",
     "qemu-stage": "QEMU", "qemu-system": "QEMU", "vm-bench": "QEMU",
     "herd7-stage": "herd7", "lkmm": "herd7", "herd7": "herd7",
@@ -132,6 +137,7 @@ def libtest_lists_only(libtest_arguments):
 
 # 崩溃枚举用例的登记表（与 research/scripts/admission.py 读的是同一份）：键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>
 CRASH_CASE_REGISTRY = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
+CHECKER_PACKAGE_NAME = "singlefs-checker"   # checker 档住的包（D13 已定项 15）；它的 tests/ 整体算重型
 CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
 HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
 IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}
@@ -288,8 +294,8 @@ def workspace_packages(root_manifest, root_sections):
     return packages
 
 
-def layer0_test_targets(package_directory):
-    """包里名字含 layer0 的集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
+def integration_test_targets(package_directory):
+    """包里全部集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
     names = set()
     for path in glob.glob(os.path.join(package_directory, "tests", "*.rs")):
         names.add(os.path.basename(path)[:-3])
@@ -298,7 +304,34 @@ def layer0_test_targets(package_directory):
     for target in (manifest_sections(os.path.join(package_directory, "Cargo.toml")) or {}).get("test") or []:
         if isinstance(target, dict) and target.get("name"):
             names.add(target["name"])
-    return sorted(name for name in names if "layer0" in name)
+    return sorted(names)
+
+
+def layer0_test_targets(package_directory):
+    """包里名字含 layer0 的集成测试目标。"""
+    return [name for name in integration_test_targets(package_directory) if "layer0" in name]
+
+
+def checker_package_directories(directory):
+    """directory 往上找到的第一个含 crates/<checker 包> 的根里那个包目录；一个都找不到（仓外执行）才取本 hook 所在仓的那一个。"""
+    while directory:
+        candidate = os.path.join(directory, "crates", CHECKER_PACKAGE_NAME)
+        if os.path.isdir(candidate):
+            return [candidate]
+        parent = os.path.dirname(directory)
+        if parent == directory:
+            break
+        directory = parent
+    own = os.path.join(HOOK_REPOSITORY, "crates", CHECKER_PACKAGE_NAME)
+    return [own] if os.path.isdir(own) else []
+
+
+def checker_test_targets(directory):
+    """checker 包 tests/ 下的测试目标名（两个仓的并）：直接执行的测试二进制按它判 checker 档。"""
+    names = set()
+    for package_directory in checker_package_directories(directory):
+        names.update(integration_test_targets(package_directory))
+    return names
 
 
 CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
@@ -515,6 +548,32 @@ def cargo_use(arguments, directory, environment=None):
         return None
     if whole_workspace:
         return heavy_test("full-cargo", f"{described} 带 --workspace / --all")
+    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
+    crash_cases = registered_crash_cases(os.path.dirname(manifest_path) if manifest_path else directory)
+    sections = manifest_sections(manifest_path) if manifest_path else None
+    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
+    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
+    narrowed = bool(selectors & NARROWING_SELECTORS)
+    scope, is_virtual_workspace_root = [], False
+    if sections is not None:
+        is_virtual_workspace_root = "workspace" in sections and "package" not in sections
+        if packages:
+            scope = [members[name] for name in packages if name in members]
+        elif is_virtual_workspace_root:
+            scope = list(members.values())
+        else:
+            scope = [os.path.normpath(os.path.dirname(manifest_path))]
+    if sections is not None and not packages and is_virtual_workspace_root and not narrowed:
+        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
+    if sections is not None and not narrowed and members and set(scope) == set(members.values()):
+        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
+                                         f"{len(members)} 个成员），等于全量"))
+    # checker 档按包判（D13 已定项 15）：跑到 singlefs-checker 包的集成测试就是重型；只跑它的 --lib / --doc / --bins 不算。排在层 0 与崩溃枚举用例那两条之前
+    checker_directory = members.get(CHECKER_PACKAGE_NAME)
+    if checker_directory and os.path.normpath(checker_directory) in {os.path.normpath(path) for path in scope} \
+            and (not selectors or selectors & {"--tests", "--all-targets", "--test"}):
+        return heavy_test("checker-tier-cargo", f"{described} 跑到 {CHECKER_PACKAGE_NAME} 包的测试（checker 档，默认只在提交时跑；"
+                                                f"{'--test ' + '、'.join(test_names) if test_names else '不挑目标'}）")
     named_layer0 = [name for name in test_names if "layer0" in name]
     if named_layer0:
         return heavy_test("layer0-cargo", f"{described} --test {named_layer0[0]}")
@@ -523,11 +582,6 @@ def cargo_use(arguments, directory, environment=None):
         ignored_reasons.append(f"--run-ignored {run_ignored_value}")
     if unseen_reason:
         ignored_reasons.append(unseen_reason)
-    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
-    crash_cases = registered_crash_cases(os.path.dirname(manifest_path) if manifest_path else directory)
-    sections = manifest_sections(manifest_path) if manifest_path else None
-    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
-    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
     crash_case_targets = sorted({case.target for case in crash_cases})
     for pattern in test_names:
         matched = fnmatch.filter(crash_case_targets, pattern) if GLOB_CHARACTERS & set(pattern) else [name for name in crash_case_targets if name == pattern]
@@ -537,19 +591,6 @@ def cargo_use(arguments, directory, environment=None):
             return found_heavy
     if sections is None:
         return None
-    is_virtual_workspace_root = "workspace" in sections and "package" not in sections
-    narrowed = bool(selectors & NARROWING_SELECTORS)
-    if not packages and is_virtual_workspace_root and not narrowed:
-        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
-    if packages:
-        scope = [members[name] for name in packages if name in members]
-    elif is_virtual_workspace_root:
-        scope = list(members.values())
-    else:
-        scope = [os.path.normpath(os.path.dirname(manifest_path))]
-    if not narrowed and members and set(scope) == set(members.values()):
-        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
-                                         f"{len(members)} 个成员），等于全量"))
     layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
     unselected = not selectors or bool(selectors & {"--tests", "--all-targets"})
     crash_cases_in_scope = [case for case in crash_cases if members.get(case.package) in scope]
@@ -809,6 +850,8 @@ def classify(words, directory, environment=None):
     if binary is not None:
         if libtest_lists_only(arguments):
             return None
+        if binary in checker_test_targets(directory or HOOK_REPOSITORY):
+            return heavy_test("checker-tier-binary", f"直接执行 {CHECKER_PACKAGE_NAME} 包的测试二进制（{binary}）：checker 档，默认只在提交时跑")
         if "layer0" in binary:
             return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）")
         cases = [case for case in registered_crash_cases(directory) if case.target == binary]
@@ -932,6 +975,8 @@ def build_sample_workspace(work):
     write("crates/singlefs-harness/tests/common/marked_case.rs", "#[test]\n#[ignore]\nfn the_marked_included_case() {}\n")
     write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
     write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", marked_case)
+    write("crates/singlefs-checker/tests/checker_quick_stream.rs", "#[test]\nfn a_quick_case() {}\n")
+    write("crates/singlefs-checker/src/lib.rs", "pub fn judge() {}\n")
     write(CRASH_CASE_REGISTRY, "# 样本登记表\n54-layer0-replay.sh\tcrates/\t# 样本\n"
                                "crash-case:sample\tcrates/ Cargo.toml\ttest=singlefs-harness:sample_crash_enumeration:the_full_case\t# 样本\n"
                                "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n"
@@ -983,11 +1028,18 @@ def selftest():
              "crash-case-cargo"),
             ("--test 通配命中崩溃枚举用例、带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_*", "--", "--ignored"], work,
              "crash-case-cargo"),
-            ("只有崩溃枚举用例的包不挑目标、带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--", "--ignored"], work, "crash-case-cargo"),
+            ("checker 包不挑目标、带 --ignored：按包判 checker 档", ["cargo", "test", "-p", "singlefs-checker", "--", "--ignored"], work, "checker-tier-cargo"),
+            ("checker 包不挑目标、不带 --ignored：快档也是 checker 档", ["cargo", "test", "-p", "singlefs-checker"], work, "checker-tier-cargo"),
+            ("checker 包 --tests", ["cargo", "test", "-p", "singlefs-checker", "--tests"], work, "checker-tier-cargo"),
+            ("checker 包点名一个快档目标", ["cargo", "test", "-p", "singlefs-checker", "--test", "checker_quick_stream"], work, "checker-tier-cargo"),
+            ("在 checker 包目录里裸跑", ["cargo", "test", "--release"], os.path.join(work, "crates", "singlefs-checker"), "checker-tier-cargo"),
+            ("checker 包只跑 --lib 不算", ["cargo", "test", "-p", "singlefs-checker", "--lib"], work, None),
+            ("checker 包只跑 --doc 不算", ["cargo", "test", "-p", "singlefs-checker", "--doc"], work, None),
+            ("-p harness 加 -p checker 一起", ["cargo", "test", "-p", "singlefs-core", "-p", "singlefs-checker"], work, "checker-tier-cargo"),
             ("点名崩溃枚举用例、不带 --ignored（只跑它的快用例）", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"], work, None),
             ("点名崩溃枚举用例、libtest 参数只有 --nocapture",
              ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--nocapture"], work, None),
-            ("只有崩溃枚举用例的包 --lib 带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--lib", "--", "--ignored"], work, None),
+            ("checker 包 --lib 带 --ignored：库的单测，不算", ["cargo", "test", "-p", "singlefs-checker", "--lib", "--", "--ignored"], work, None),
             ("别的测试目标带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "second_transaction_step_one_overwrite", "--", "--ignored"],
              work, None),
             # 用例函数没标 #[ignore]：不带 --ignored 也跑到全量（读源码判，与 admission.py crash-cases 同一套）
@@ -1001,14 +1053,14 @@ def selftest():
             ("cargo nextest run --run-ignored all 点名崩溃枚举用例",
              ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "all"], work, "crash-case-cargo"),
             ("cargo nextest run --run-ignored=only -E 过滤式、只有崩溃枚举用例的包",
-             ["cargo", "nextest", "run", "-p", "singlefs-checker", "--run-ignored=only", "-E", "test(=the_full_case)"], work, "crash-case-cargo"),
+             ["cargo", "nextest", "run", "-p", "singlefs-checker", "--run-ignored=only", "-E", "test(=the_full_case)"], work, "checker-tier-cargo"),
             ("cargo nextest run --run-ignored default 只跑快用例",
              ["cargo", "nextest", "run", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--run-ignored", "default"], work, None),
             ("cargo nextest run 在工作区根裸跑", ["cargo", "nextest", "run"], work, "full-cargo"),
             ("cargo nextest list 不起测试", ["cargo", "nextest", "list", "--workspace"], work, None),
             ("cargo mutants 整包（反复跑 cargo test，包里有层 0 目标）", ["cargo", "mutants", "-p", "singlefs-harness"], work, "layer0-cargo"),
             ("cargo mutants 尾参交给 cargo test、带 --include-ignored",
-             ["cargo", "mutants", "-p", "singlefs-checker", "--", "--test", "checker_crash_enumeration", "--", "--include-ignored"], work, "crash-case-cargo"),
+             ["cargo", "mutants", "-p", "singlefs-checker", "--", "--test", "checker_crash_enumeration", "--", "--include-ignored"], work, "checker-tier-cargo"),
             ("cargo miri test 点名崩溃枚举用例、带 --ignored",
              ["cargo", "miri", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work, "crash-case-cargo"),
             ("cargo llvm-cov 不带子命令、点名崩溃枚举用例带 --ignored",
@@ -1218,9 +1270,9 @@ def selftest():
         # 名字不含 layer0 的那几条：含 layer0 的按层 0 那一类先认出来，判不出登记表取没取到
         own_repository_targets = sorted(target for _package, target in registered_crash_case_targets(HOOK_REPOSITORY) if "layer0" not in target)
         if own_repository_targets:
-            binary_cases.append(("仓外执行、登记表取这份文件所在仓的那一份：那里登记的崩溃枚举用例带 --ignored",
+            binary_cases.append(("仓外执行、取这份文件所在仓：那里登记的崩溃枚举用例住在 checker 包，直接执行它的二进制按包判 checker 档",
                                  [f"/tmp/target-elsewhere/release/deps/{own_repository_targets[0]}-0123456789abcdef", "--ignored"], "/",
-                                 "crash-case-binary"))
+                                 "checker-tier-binary"))
         else:
             results.append((f"这份文件所在仓的登记表（{os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)}）里有名字不含 layer0 的崩溃枚举用例",
                             True, False))
diff --git "a/.claude/kb/decisions/13-\351\252\214\350\257\201\350\267\257\347\272\277.md" "b/.claude/kb/decisions/13-\351\252\214\350\257\201\350\267\257\347\272\277.md"
index 7e5394e7..6155d3ea 100644
--- "a/.claude/kb/decisions/13-\351\252\214\350\257\201\350\267\257\347\272\277.md"
+++ "b/.claude/kb/decisions/13-\351\252\214\350\257\201\350\267\257\347\272\277.md"
@@ -20,6 +20,7 @@ D13（验证路线） 管本工程拿什么验正确性：三类现成办法全
 | 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** |
 | 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** |
 | 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** |
+| 15 | **验证代码分两档** | harness 是单元与集成测试、随时跑；checker 档（checker 包的测试、QEMU、herd7、变异整表、实验复跑）默认只在提交时跑快档，全量由用户要求或夜间 **状态：已定。** |
 
 #### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」
 
@@ -89,7 +90,7 @@ D13（验证路线） 管本工程拿什么验正确性：三类现成办法全
 - **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
 - **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。
 
-**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 27 号按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。
+**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 27 号按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。集成测试不算共享：`crates/singlefs-checker/tests/` 下的崩溃枚举用例经 `[dev-dependencies]` 依赖 `singlefs-core` 与 `singlefs-harness` 造镜像再交给 checker 判，门禁 94 号只读 `[dependencies]` 与 `[build-dependencies]`（已定项 15）。
 
 **依据**：
 - 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
@@ -171,7 +172,7 @@ D13（验证路线） 管本工程拿什么验正确性：三类现成办法全
 
 | 层 | 范围 | 抽样 | 触发 |
 |---|---|---|---|
-| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | 任何触碰写路径的提交 |
+| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |
 | 层 1 常规 | 中等规模负载（写请求数上千） | 项目级固定种子 + **按崩溃点所在子阶段分桶抽样**（意图写入 / 分批推进 / 意图删除 / 纯数据写） | 每次提交 |
 | 层 2 全量 | 层 1 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮 | 不抽样 | 夜间 / 发布前 |
 
@@ -187,7 +188,7 @@ D13（验证路线） 管本工程拿什么验正确性：三类现成办法全
 **夜间档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个夜间档 bug 的挂钟 ≈ `log2(N) × 一天`，一个不可复现的夜间失败价值接近 0，
 甚至为负，它训练人忽略红灯。
 
-**射程**：今天只有层 0（门禁 54 号）；层 0 分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；提交时全量、带断点续跑（门禁 54 号 `--full`）；层 1、层 2 在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；层 1 抽样乘的 N 与调度归已定项 10、11。
+**射程**：今天只有层 0（门禁 54 号）；层 0 分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 54 号 `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求或夜间跑；层 1、层 2 在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；层 1 抽样乘的 N 与调度归已定项 10、11。
 
 **依据**：
 - 无实验：分层是门禁挂钟的设计约束，层 1、层 2 的负载与挂钟都还没有，没有可量的量；分桶防的自欺情形来自一个还没跑的实验的作废条款，不是量到的数。
@@ -312,6 +313,18 @@ spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移
 
 **欠**：无。
 
+#### 已定项 15：验证代码分两档——harness 随时跑，checker 档默认只在提交时跑
+
+**定案**：`crates/` 下的验证代码分两档，按「什么时候跑」分，不按机制分。**harness 档**是 `crates/singlefs-harness`：单元测试与集成测试那一层，改了代码随时跑，实现员交回前跑自己动到的测试二进制；包内再分轻重，重的（随机历史长档、注入战役的大档、release 下要跑几分钟的）标 `#[ignore]`，要跑随时跑，一律经内存包装。**checker 档**是 `crates/singlefs-checker` 包里的测试（崩溃枚举用例、注入战役的快档与全量）连同 QEMU 真设备（门禁 55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档自己分两档：**快档**是 `cargo test --release -p singlefs-checker` 不带 `--ignored`（门禁 54 号在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（54 号 `--full`），提交时不默认跑，由用户要求或夜间跑，跑法按已定项 9 分层；GPU 只接校验和那一截（E163（GPU多卡算单元校验和））、要不要接归 D24（后台重活能不能卸给 GPU）。崩溃枚举用例一律写在 `crates/singlefs-checker/tests/` 里（`research/scripts/crash-case-check.py` 判），不写进 harness；重型测试闸按包判：跑到 `singlefs-checker` 包的测试就是 checker 档。
+
+**射程**：管 `crates/` 下测试住哪个包、什么时候跑、闸按什么认；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管 checker 库与实现共享什么（已定项 5：checker 包的 `[dev-dependencies]` 依赖 core 与 harness 造镜像再判，不算共享）。「checker 档」与「池级 checker」是两个名字：前者是这一档的测试，后者是 `check_pool_image` 那个一元谓词（已定项 7）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`）。实验二进制与 QEMU 二进制留在 harness 包：它们不是测试，不决定什么时候跑。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。
+
+**依据**：
+- 无实验：什么时候跑是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
+- 用户定案（2026-09-27），原话在变更史；改动由主 agent 自己做完再走三方（用户同日定「这个任务你不要排subagent了 全部你来做」，随后补「改完后可以走三方腿验证」），判决出来后补进这里。
+
+**欠**：无。
+
 ## 历史版本
 
 D13（验证路线）的历史条目集中在 [decisions-history.md](../decisions-history.md)。
diff --git a/.claude/kb/tooling.md b/.claude/kb/tooling.md
index 208267a3..533dd1da 100644
--- a/.claude/kb/tooling.md
+++ b/.claude/kb/tooling.md
@@ -645,7 +645,7 @@ rustup 追加在文件末尾的 PATH 那句因此从不执行；
 | 27 号（格式常量） | kb 标记改值后，`research/**/*.rs` 里每一份同名 `const` 都要跟（JOURNAL_HEADER_BYTES 95 → 277 时 E43（扩展点字节上限）、E116（打包容器的账·补元数据写与整理策略） 两份装置都重跑换产物）；`stale=` 列的旧值串不许再出现在 kb 正文 | 别把仍在别处合法出现的串（`base = 95`）加进 `stale=` |
 | clippy `shadow_unrelated` | 同一个测试函数里第二次 `let reader = …` / `let segments = …` 判红，rustfmt 重排之后用正则替换还会漏改后半段（2026-09-14 三次返工） | 每个读者 / 段 / 报告起不同的名字（`extent_reader`、`warm_up_segments`、`control_report`） |
 | naming-lint | 「字母 + 数字」当单字母：`read_u48` 判红，测试名以 `a_` 开头判红；它扫全仓的 .rs，连 `research/prompts` 里云端腿写的模型源码也扫 | 写 `read_six_byte_unsigned`；`research/prompts` 2026-09-14 起在 `.claude/naming-lint-exclude`（证据目录，不回收），腿的模型写到别的目录照查 |
-| 54 号（层 0 全量） | 用例标 `#[ignore]`，平时 `cargo test --workspace` 报 1 ignored 是正常的 | 门禁在 release 下跑它，每个状态加池级 checker 与记录核对器后约 171 s；全量在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
+| 54 号（层 0 全量） | 用例标 `#[ignore]`、住 `crates/singlefs-checker/tests/`（D13（验证路线） 已定项 15），`cargo test -p singlefs-checker` 报几条 ignored 是正常的；harness 档 `cargo test -p singlefs-harness` 跑不到它们 | 门禁在 release 下跑它，每个状态加池级 checker 与记录核对器后约 171 s；全量在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
 | 变异证明 | rustfmt 之后多行调用的整串匹配「没命中」 | 按「找到含关键串的行、删到以 `);` 收尾的那一行」做；改完复原并 `assert` 文件逐字节等于原文 |
 | checker 新判定 | 只在对象已经过了同一关之后才判的判定永远判不红：2026-09-14 checker「根是码 2」那条第一稿只在根按码 2 读通之后才判，坏镜像语料当场让它报「不适用」 | 每条新判定配一份坏镜像（`checker_known_bad_images.rs` 的形态），不许只看干净镜像全绿 |
 | 批量定点替换 | 逐条写盘会在一处锚点不中时留下半套改动（2026-09-14 写回 54 处，一处原文「没被」锚点写成「没有被」） | 用 `research/scripts/replace-batch.py`：全部替换先在内存里核「恰好命中 1 次」，全过了才写盘 |
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index 8ae82c92..9e8a309b 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -7,17 +7,17 @@
 主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派；一批实现员交回后的合入与验证派一个 `implementation-writer` 做「合入后验证」（派发表那一行）；探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写（派发表「改 `crates/`：探索性的」与「一个阶段任务结束」两行），自己写的 `crates/` 改动并进同一批的代码轮三方。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活照旧临时写提示派 general-purpose。
 
 ## 禁止
-- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。
+- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。这一条管枚举域，不管跑的时机：什么时候跑按 `.claude/rules/verification.md`，checker 档默认只在提交时跑快档，全量由用户要求或夜间。
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务（派出去的一件活）一个出口，达成出口即终止任务，扩大任务必须弹窗；一件活可以关多条问题（「一轮怎么开、怎么收」第 4 条的一簇），每条各有验收标准，出口是全部达成。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
 
-- **禁止在subagent中跑重型测试**（例外只有 `crash-verifier` 与 `gate-triage` 各跑自己那一部分、命令带 `SINGLEFS_HEAVY_TESTS` 前缀），完整清单以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准。
+- **禁止在subagent中跑重型测试**（例外只有 `crash-verifier` 与 `gate-triage` 各跑自己那一部分、命令带 `SINGLEFS_HEAVY_TESTS` 前缀），完整清单以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准。
 
 ## 一轮怎么开、怎么收
 
 1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。把这一轮的任务列全，逐个标它依赖谁、输入齐没齐：没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派——调查各派各的调查员，实验各派各的执行员（分开实验），改 `crates/` 的按第 4 条聚簇（统一实现），写回 kb 的照第 10 条一批一份规格；类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类），改 `crates/` 与改门禁不合给一个 agent（写范围不同）。同时派受派发闸 ⑦ 的 opus 并发上限约束，超出的排下一波。有依赖的只串依赖那一环；散的活不排成一串，合成大块。课题、出口与这张依赖表写进这一轮的调度记录（`records/`），第 7、8 条回头对着它判。派之前按这张表问一遍「哪几件能现在一起做」，答不出就还没规划完。
 2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在定（判断与定案现在做，改动并进第 4 条最近的一批）；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
-3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
+3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
 4. **派实现员之前先聚簇，不为一条发现单派**：把这一轮已抓到、条款已定、要改 `crates/` 的改动全部列出来，按要动的 `crates/` 文件聚簇——文件有交集的并进同一个实现员一次改完（派发提示逐条列问题与各自的验收标准，一个实现员可以关多条，出口是全部达成），互不相交的在同一条消息里同时派、各自的「不碰」清单写上兄弟实现员的文件；同时派几个时一律交补丁（各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `research/scripts/apply-writer-patch.py` 打），`crates/mutations.tsv` 只追加、不算撞文件；份数不设上限。批内冒出的新发现进下一批清单，凑批再派；照第 2 条判为挡着本轮课题的，也并进下一批，不单派；与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件。下一批在上一批合入、取过代码轮快照之后就派，要改快照里文件的等那一轮判决（派发闸 ⑥）。
 5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
 6. **派一个 agent 之前说清它关的是哪几条已经抓到的问题**（可以不止一条，逐条列）；说不出来的，那条该进记录、不该进本轮。调查、调研类的派发点名的是要回答的问题，不是已抓到的问题。
@@ -55,12 +55,12 @@
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
 | 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在有前一轮判决可复核时派）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，照 `.claude/rules/three-way-inference.md`「核查员按轮派」派 `three-way-verifier`（有腿交了模型、产物或复跑命令就派，输入里给派腿时主 agent 记下的 `date -u`，代码轮另给开工快照；不派的在判决里写明为什么）→ 主 agent 写判决；本地腿派攻方还是辩方由主 agent 定，在正文分工表里写明理由；云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿；三轮之后停 |
-| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑层 0 全量、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
+| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一批交回到齐派「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：派 `crash-verifier` 跑 checker 档：54 号快档、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；层 0 全量只在用户要求或夜间跑，平时不跑） |
 | 一批实现员交回到齐、合入之前 | `implementation-writer` 做「合入后验证」（输入给这一批的补丁目录与各份报告；照「一轮怎么开、怎么收」第 10 条：打全部补丁、编一次、非层 0 测试二进制逐个 `--test` 跑一遍、红的照报告改钉值、只证合入时改过的变异行）→ 主 agent 审 diff、合入 → 上面那一行的三方 |
-| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑层 0 全量、QEMU、herd7、crates 变异表 |
+| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：`crash-verifier` 跑 checker 档（54 号快档、QEMU、herd7、crates 变异表） |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
 | 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（55、57 与 54 号并行，59 号与 54 号串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（54、55、57、59 在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：54 号跑快档（`cargo test --release -p singlefs-checker`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求或夜间才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；判绿按输入哈希写全绿标记），55、57、59 同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（55、57 与 54 号并行，59 号与 54 号串行，内存上限照派发提示各给）→ `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（54、55、57、59 在 `gate.sh` 里只核标记与复用判定，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-spec-drafter` 起草规格（条目少、主 agent 自己写得清的可以自己写）→ 主 agent 判 → `kb-scribe`（按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个书记员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `research/mutations/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
 | 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 72 号） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 3026692c..b936db80 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -1,4 +1,4 @@
-# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑
+# 实现改动的流程：写代码 → 三方对抗 → checker，checker 档只在提交时跑
 
 **这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
 本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。
@@ -9,7 +9,7 @@
 |---|---|---|
 | 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
 | 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
-| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |
+| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（checker 档快档，全量按 `.claude/rules/verification.md`）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |
 
 **次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。
 
@@ -43,31 +43,18 @@
 
 ⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。
 
-## 重型测试只在提交时跑
+## checker 档只在提交时跑，harness 随时跑
 
-**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：
-
-- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
-- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
-- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
-- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
-- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
-- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
-- 全部实验复跑：87 号。
-- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
-- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv`、`strace -E` / `--env` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。
-
-只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算（短选项合写的 `strace -fo <文件>`、`flock -xw 10` 同样剥）。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算，跟在 `--skip`、`--logfile` 这类带值的选项后面的 `--list` 是那个选项的值，照算。命令位置上执行的脚本（`bash <脚本>`、`./<脚本>.sh`、`source <脚本>`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+两档各是什么、谁跑、带什么前缀，在 `.claude/rules/verification.md`；这里只写实现改动流程里的落点：
 
 | 场合 | 跑不跑 |
 |---|---|
-| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量（HEAD + 暂存区的 worktree 里 `--full`，连同登记的崩溃枚举用例）、QEMU、herd7、crates 变异整表由 `crash-verifier` 跑，主 agent 后台派、看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
-| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
-| 其余任何时候 | 不跑 |
-| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员在 HEAD + 暂存区的 worktree 里跑 54 号 `--full`（连同登记的崩溃枚举用例），另跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，55、57、59 号同样只核各自的全绿标记 |
-| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |
+| 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；checker 档一样都不跑 |
+| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 54 号（快档：`cargo test --release -p singlefs-checker`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 55、57、59 号；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
+| 用户要求，或夜间 | checker 档全量：54 号 `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
+| 其余任何时候 | checker 档不跑；子 agent 只跑 harness |
 
-由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
+**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 `singlefs-checker` 包的测试（`cargo test -p singlefs-checker`、不挑包而包的范围含它、直接执行它的测试二进制）、54 / 55 / 57 / 59 / 87 号、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
 
 herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：
 
@@ -87,4 +74,4 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
 - **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。54 号 `--full` 在本地配置（仓根 `layer0-shard.env`，模板 `layer0-shard.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 54 号的那一层内存包装管。驱动只由 54 号 `--full` 调。
 
-**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求或夜间，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
diff --git a/.claude/skills/crash-test/SKILL.md b/.claude/skills/crash-test/SKILL.md
index eb208959..09c03f69 100644
--- a/.claude/skills/crash-test/SKILL.md
+++ b/.claude/skills/crash-test/SKILL.md
@@ -9,4 +9,4 @@ description: 跑 singlefs 的验证套件——LKMM 内存序、QEMU/KVM 压测
 
 ## 在本项目里
 
-共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（55、57、59 号）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
+共享正文不管内存序与虚机（它写明那两样由项目自己定）。本项目的四样各落在一道门禁：崩溃点重放是 `.claude/gate.d/54-layer0-replay.sh`，QEMU 真设备是 `55-qemu-first-transaction.sh`，LKMM 内存序是 `57-lkmm.sh`，模型对拍是 `74-model-differential.sh`。前三样是重型测试，谁在什么时候跑、带什么前缀，照 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行；子 agent 里只有崩溃验证员（55、57、59 号）与门禁分诊员（整轮门禁 `research/scripts/gate-staged.sh`）各跑自己那一份，别的子 agent 不跑，钩子 `.claude/hooks/heavy-test-guard.sh` 会拒。
diff --git a/CLAUDE.md b/CLAUDE.md
index 437a900c..8cc144a0 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -56,6 +56,7 @@ subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案
 @.claude/rules/three-way-inference.md
 @.claude/rules/mutation-sampling.md
 @.claude/rules/implementation-workflow.md
+@.claude/rules/verification.md
 @.claude/rules/implementation-first.md
 @.claude/rules/path-moves.md
 
@@ -106,4 +107,4 @@ subagent 与协作工具的欠账记在 `records/2026-09-16-subagent拆分提案
 
 - 先定决策，再写代码——未定项还开着就写下去的实现多半要返工。
 - 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」（`.claude/rules/fs-design.md`「从事务开始，不从功能开始」）。
-- 门禁全绿**只构成第一个事务在模型层的崩溃一致性证据**；里程碑「第二个事务」步 0 那条固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）在管理员回退改成挂着时的向前发布之后，层 0 用例只改到编得过、下游钉的值没有重核，重写与重跑归层 0 规模那一轮与实六，这之前它的层 0 结果不作数（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行）——层 0 崩溃点重放（门禁 54 号）的负载是两条流：第一个事务，以及固定脚本到 E；checker 判 46 条不变量（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`；第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变）、I-5.4（分配记录罩住的槽互不相交）、I-1.8（归并后版本全序）、I-7.3（环健康性） 与 I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读）、I-8.9（一次发布的记录序号连续且只有末条带标志） 、I-7.12（系统配置 F 不低于同盘根上的 F） 与 I-1.11（映射 key 与单元头相符）；I-3.1（已分配统计对得上）、I-2.1（校验和与内容匹配） 与后加的六条里除 I-3.8（实例表行唯一且低于挂载根） 之外的五条按回退候选集判，候选集的下界取 F_生效（各幸存盘最新持久有效根带的 F 与系统配置里的 F 取大）；I-3.1（已分配统计对得上） 在最新根带的 F 低于 F_生效 时报不适用）。
+- 门禁全绿**只构成第一个事务在模型层的崩溃一致性证据**；里程碑「第二个事务」步 0 那条固定脚本（覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次）在管理员回退改成挂着时的向前发布之后，层 0 用例只改到编得过、下游钉的值没有重核，重写与重跑归层 0 规模那一轮与实六，这之前它的层 0 结果不作数（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实三交回」那一行）——层 0 崩溃点重放（门禁 54 号）的负载是两条流：第一个事务，以及固定脚本到 E；checker 判 48 条不变量（数它的命令：`grep -c '^| I-.*已实现' .claude/kb/invariants.md`；第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变）、I-5.4（分配记录罩住的槽互不相交）、I-1.8（归并后版本全序）、I-7.3（环健康性） 与 I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读）、I-8.9（一次发布的记录序号连续且只有末条带标志） 、I-7.12（系统配置 F 不低于同盘根上的 F） 、I-1.11（映射 key 与单元头相符）、I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序）；I-3.1（已分配统计对得上）、I-2.1（校验和与内容匹配） 与后加的六条里除 I-3.8（实例表行唯一且低于挂载根） 之外的五条按回退候选集判，候选集的下界取 F_生效（各幸存盘最新持久有效根带的 F 与系统配置里的 F 取大）；I-3.1（已分配统计对得上） 在最新根带的 F 低于 F_生效 时报不适用）。
diff --git a/Cargo.lock b/Cargo.lock
index 1fbeabdc..ff5b1e3b 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -6,7 +6,9 @@ version = 4
 name = "singlefs-checker"
 version = "0.1.0"
 dependencies = [
+ "singlefs-core",
  "singlefs-format",
+ "singlefs-harness",
 ]
 
 [[package]]
diff --git a/README.md b/README.md
index 008ddd48..fedacde5 100644
--- a/README.md
+++ b/README.md
@@ -119,10 +119,11 @@ waaagh！
 提交前跑门禁：
 
 ```bash
-bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑；层 0 全量不在里面）
+bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含 checker 档快档与逐条全绿标记核对、QEMU 真设备、herd7、crates 变异表复跑；层 0 全量不在里面）
 
-cargo test --workspace                    # 平时的单测；登记的崩溃枚举用例（层 0 全量等）都标 ignored，这里只跑快档与缩小版
-bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 快档（两条流里不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记
+cargo test -p singlefs-harness            # harness 档：单元与集成测试，改了代码随时跑（.claude/rules/verification.md）
+SINGLEFS_HEAVY_TESTS=user-request cargo test --release -p singlefs-checker   # checker 档快档：崩溃枚举用例住这个包的 tests/，全量那几条标 ignored；默认只在提交时由门禁 54 号跑
+bash .claude/gate.d/54-layer0-replay.sh   # 单跑 54 号快档（checker 包不标 ignored 的用例），再逐条核登记的崩溃枚举用例各自那一格全绿标记，不作数的报「本次未跑」
 bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>  # 层 0 全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（建法见快档判红时的出路句），逐条崩溃枚举用例按它自己的输入指纹复用或重跑，判绿写那一条的全绿标记
 bash .claude/gate.d/55-qemu-first-transaction.sh  # 单跑 QEMU 两块 virtio 盘上的第一个事务、发布 B、第二个实例、发布 D 与抬 F
 bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 257d9025..bfeffcbd 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -14,13 +14,13 @@ K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它
 oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
 oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
 oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
-Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
+Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-checker --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
 步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.highest_transaction_number_in_this_instance + 1,	            transaction: previous.highest_transaction_number_in_this_instance,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	        instance_table: InstanceTablePlan::Rewrite(instance_table),	        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
-步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-checker --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
 步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 2));	-p singlefs-harness --test second_transaction_step_three_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
 步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test second_transaction_step_three_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
 步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
@@ -54,15 +54,15 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs
 步 5：「非空」只比 inode 树的根指针、不比 extent 树的	crates/singlefs-core/src/mount.rs	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree	-p singlefs-core --lib -- root_is_non_empty_when_either	root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root
 步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写在单元区里的两个单元	crates/singlefs-core/src/mount.rs	    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);	    let _ = (instance_table_placement, tree_table_placement);	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0	crates/singlefs-core/src/mount.rs	        back_chain: back_chain_of(&current_version_without_file.record_bytes),	        back_chain: 0,	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
-步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
-步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;	        placement_of(&root.instance_table, TransactionUnit::TreeTable)?;	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
+步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	-p singlefs-checker --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
+步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;	        placement_of(&root.instance_table, TransactionUnit::TreeTable)?;	-p singlefs-checker --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
 步 3：写行时这次要写的行没接进重写出去的那条实例表链（带文件的一版与树表 0 条的一版共用这一处拼法）	crates/singlefs-core/src/mount.rs	    rows.extend_from_slice(rows_written);	    rows.extend_from_slice(&[]);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝	crates/singlefs-core/src/mount.rs	        Ok(pointers) => Ok(pointers),\n        Err(failure) => Err(	        Ok(pointers) => Ok(pointers),\n        Err(_failure) if true => Ok(UserVisibleTreeRootPointers::ABSENT),\n        Err(failure) => Err(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_is_refused_when	raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable
 步 3：取号不核判定时算出的号（瞬时读错让判定与取号算出不同号，写进新号之后才发现）	crates/singlefs-core/src/transaction.rs	    if recomputed != expected {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- transient_system_configuration_read_errors_between	transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write
 第一个事务 步 1：mkfs 不核根环区域归属是不是第一版写死的 0 / 1 / 0	crates/singlefs-core/src/make_filesystem.rs	    if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {	    if false {	-p singlefs-harness --test first_transaction_step_one_mkfs -- region_layout_other_than	region_layout_other_than_zero_one_zero_is_refused_before_any_write
 步 3：第一个文件版本的 txg 写死 3，不从它要建在上面的那一版接着算（2026-09-23 用户定案：`FIRST_TRANSACTION_TXG` 只管 mkfs 那条流）	crates/singlefs-core/src/transaction.rs	    let first_file_version_txg =\n        checkpoint_txg_of_the_publish_after(version_to_build_on.checkpoint_txg)?;	    let first_file_version_txg = CheckpointTxg(3);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3：publish_first_file 不核要建在上面的那条根与上一条记录说的是不是同一版	crates/singlefs-core/src/transaction.rs	    if !follows_directly {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_whose_root_and_previous_record_disagree	first_file_version_whose_root_and_previous_record_disagree_is_refused_before_any_write
-步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同）	crates/singlefs-core/src/mount.rs	        rollback_floor: current_version_without_file.root.rollback_floor,	        rollback_floor: CheckpointTxg(1),	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- formatted_pool_mount_stream_has_the_same	formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream
+步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同）	crates/singlefs-core/src/mount.rs	        rollback_floor: current_version_without_file.root.rollback_floor,	        rollback_floor: CheckpointTxg(1),	-p singlefs-checker --test second_transaction_step_three_formatted_pool_layer0 -- formatted_pool_mount_stream_has_the_same	formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream
 增补 1：发布路径漏计数据单元的写（第一个事务按种类的合计对不上录制器在设备一层记下的写）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::Data { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
 增补 1：发布路径漏计实例表单元的写（写行发布按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::InstanceTable { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- writable_remount_row_publish	writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes
 增补 1：写入口漏计系统配置槽的写（暖机按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	        self.writes_by_structure_kind\n            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);	        drop(slot_bytes);	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
@@ -81,11 +81,11 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs
 增补 2：提交内生块段耗尽不回落（抬 F 的空发布在唯一全空段被扣住时又报 NoSpaceFor）	crates/singlefs-core/src/allocator.rs	            None => self\n                .lowest_commit_generated_fallback_slot(footprint)\n                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),	            None => None,	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_when_the_only	raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold
 增补 2：回落只看已分配位（发出影子账隔离的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- fallback_skips_a_slot_isolated	fallback_skips_a_slot_isolated_by_the_shadow_ledger
 增补 2：回落只看已分配位（抬 F 的空发布发出扣住的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot	raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise
-步 0：崩溃镜像的记录提示漏掉基镜像里的记录（种进基镜像的残留记录恢复扫不到）	crates/singlefs-harness/src/crash.rs	        let mut offsets = self\n            .base\n            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;	        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
-步 6：恢复从系统配置 tail 起逐条验证记录的点名单元、失配即中止（E78 的自我中止形态）	crates/singlefs-core/src/recovery.rs	            let records = scan_journal(reader, &system_configuration);	            let records = scan_journal(reader, &system_configuration);\n            let trusted_tail_mismatch = records\n                .values()\n                .filter(|record| record.counter > system_configuration.quantities.journal_tail)\n                .any(|record| {\n                    replay_journal(\n                        reader,\n                        &RootRecord {\n                            instance: record.instance,\n                            checkpoint_txg: CheckpointTxg(record.checkpoint_txg.0 - 1),\n                            ..root\n                        },\n                        system_configuration.immutable.sizes.journal_ring_bytes,\n                        &BTreeMap::from([((record.instance, record.counter), record.clone())]),\n                        true,\n                        None,\n                    )\n                    .0\n                    .verification_failed\n                        > 0\n                });\n            if trusted_tail_mismatch {\n                return RecoveryReport {\n                    outcome: RecoveryOutcome::Failed {\n                        root: Some(root_key),\n                        failure: RecoveryFailure::NoValidRoot,\n                    },\n                    effective_root: None,\n                    journal: JournalScanReport::default(),\n                    mapping_fallbacks,\n                };\n            }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
-步 6：I-3.8 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/walk.rs	        self.judgements.judge(\n            "I-3.8",	        let _ = (\n            "I-3.8",	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
-步 6：I-2.1 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/image.rs	        judgements.judge("I-2.1", matches, || {	        let _ = ("I-2.1", matches, || {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
-步 6：层 0 计数不记「不适用」（评估过 + 不适用 ≠ 状态数）	crates/singlefs-harness/src/crash.rs	            InvariantVerdict::NotApplicable(_) => {\n                *tally\n                    .checker_not_applicable_states\n                    .entry(invariant)\n                    .or_insert(0) += 1;\n            }	            InvariantVerdict::NotApplicable(_) => {}	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+步 0：崩溃镜像的记录提示漏掉基镜像里的记录（种进基镜像的残留记录恢复扫不到）	crates/singlefs-harness/src/crash.rs	        let mut offsets = self\n            .base\n            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;	        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();	-p singlefs-checker --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
+步 6：恢复从系统配置 tail 起逐条验证记录的点名单元、失配即中止（E78 的自我中止形态）	crates/singlefs-core/src/recovery.rs	            let records = scan_journal(reader, &system_configuration);	            let records = scan_journal(reader, &system_configuration);\n            let trusted_tail_mismatch = records\n                .values()\n                .filter(|record| record.counter > system_configuration.quantities.journal_tail)\n                .any(|record| {\n                    replay_journal(\n                        reader,\n                        &RootRecord {\n                            instance: record.instance,\n                            checkpoint_txg: CheckpointTxg(record.checkpoint_txg.0 - 1),\n                            ..root\n                        },\n                        system_configuration.immutable.sizes.journal_ring_bytes,\n                        &BTreeMap::from([((record.instance, record.counter), record.clone())]),\n                        true,\n                        None,\n                    )\n                    .0\n                    .verification_failed\n                        > 0\n                });\n            if trusted_tail_mismatch {\n                return RecoveryReport {\n                    outcome: RecoveryOutcome::Failed {\n                        root: Some(root_key),\n                        failure: RecoveryFailure::NoValidRoot,\n                    },\n                    effective_root: None,\n                    journal: JournalScanReport::default(),\n                    mapping_fallbacks,\n                };\n            }	-p singlefs-checker --test second_transaction_step_zero_layer0 -- stale_tail	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
+步 6：I-3.8 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/walk.rs	        self.judgements.judge(\n            "I-3.8",	        let _ = (\n            "I-3.8",	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+步 6：I-2.1 的判定没跑到（层 0 报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/image.rs	        judgements.judge("I-2.1", matches, || {	        let _ = ("I-2.1", matches, || {	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+步 6：层 0 计数不记「不适用」（评估过 + 不适用 ≠ 状态数）	crates/singlefs-harness/src/crash.rs	            InvariantVerdict::NotApplicable(_) => {\n                *tally\n                    .checker_not_applicable_states\n                    .entry(invariant)\n                    .or_insert(0) += 1;\n            }	            InvariantVerdict::NotApplicable(_) => {}	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 增补 2 第 21 行：暖机把 jsn 计数器写成 txg（C366；接在 jsn 40 之后时记录落进 jsn 1、2 那两格、tail 写 2）	crates/singlefs-core/src/transaction.rs	txg: CheckpointTxg(txg_number),\n                counter: previous_counter + 1,	txg: CheckpointTxg(txg_number),\n                counter: txg_number,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- warm_up_after_a_journal_counter	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
 增补 2 第 19 行：出生序号发号器挪回每装一个文件对象重建一次（同一个 checkpoint 的第二个对象从 0 重数）	crates/singlefs-core/src/transaction.rs	) -> FileVersionUnits {\n    let txg = checkpoint.txg;	) -> FileVersionUnits {\n    let mut sequences_rebuilt_on_every_call = BirthSequenceAllocator::default();\n    let sequences = &mut sequences_rebuilt_on_every_call;\n    let txg = checkpoint.txg;	-p singlefs-core --lib -- transaction::tests::second_file_object	second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero
 增补 2 第 30 行：真设备二进制的挂载窗口从重开那一刻算起（取号的两次系统配置槽写进了设备一层的数、不在按种类的账里）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        pool_writes_between(&counts_after_acquisition, &counts_after_mount),	        pool_writes_between(&vec![DeviceCallCounts { write_calls: 0, written_bytes: 0, force_unit_access_writes: 0, barrier_calls: 0 }; counts_after_mount.len()], &counts_after_mount),	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
@@ -109,11 +109,11 @@ I-5.4：checker 判不出分配记录重叠	crates/singlefs-checker/src/walk.rs
 I-5.4：只判最新根那棵账	crates/singlefs-checker/src/walk.rs	    judge_allocation_records_disjoint(\n        reader,\n        &roots,\n        &candidate_indexes,	    judge_allocation_records_disjoint(\n        reader,\n        &roots,\n        &[newest_index],	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_whose_span	an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant
 增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
 增补 3 第 1 件：随机历史偏向抬 F 之后复用的取样点判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）	crates/singlefs-core/src/allocator.rs	            records.retain(|record| !(record.device == device && record.slot == record_slot));	            // 变异：罩住的已回收记录不删	-p singlefs-harness --test second_transaction_supplement_three_random_history -- reuse_heavy_random_histories	reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms
-增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	    (0..state_count.div_ceil(states_per_slice))	    (1..state_count.div_ceil(states_per_slice))	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
-增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	                .saturating_add(states_per_slice)\n                .min(state_count);	                .saturating_add(states_per_slice)\n                .saturating_add(1)\n                .min(state_count);	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	    (0..state_count.div_ceil(states_per_slice))	    (1..state_count.div_ceil(states_per_slice))	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）	crates/singlefs-harness/src/crash.rs	                .saturating_add(states_per_slice)\n                .min(state_count);	                .saturating_add(states_per_slice)\n                .saturating_add(1)\n                .min(state_count);	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 增补 2 第 41 行 层 0 并行：相邻两片重叠（切片单测）	crates/singlefs-harness/src/crash.rs	                .saturating_add(states_per_slice)\n                .min(state_count);	                .saturating_add(states_per_slice)\n                .saturating_add(1)\n                .min(state_count);	-p singlefs-harness --lib -- state_slices_cover_every_state	state_slices_cover_every_state_exactly_once_in_ordinal_order
 增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反	crates/singlefs-harness/src/crash.rs	            let ordinal_within_the_segment =\n                ordinal - self.state_ranges_by_segment[segment_of_state].start;	            let ordinal_within_the_segment =\n                !(ordinal - self.state_ranges_by_segment[segment_of_state].start);	-p singlefs-harness --lib -- the_state_plan_hands_out	the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment
-增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的	crates/singlefs-harness/src/crash.rs	        if self.first_violation.is_none() {\n            self.first_violation = first_violation;\n        }	        if first_violation.is_some() {\n            self.first_violation = first_violation;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- one_state_slices_on_eight_threads	one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread
+增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的	crates/singlefs-harness/src/crash.rs	        if self.first_violation.is_none() {\n            self.first_violation = first_violation;\n        }	        if first_violation.is_some() {\n            self.first_violation = first_violation;\n        }	-p singlefs-checker --test second_transaction_step_zero_layer0 -- one_state_slices_on_eight_threads	one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread
 增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的	crates/singlefs-harness/src/crash.rs	            self.checker_first_violation\n                .entry(invariant)\n                .or_insert(detail);	            self.checker_first_violation.insert(invariant, detail);	-p singlefs-harness --lib -- absorbing_a_following_slice	absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation
 增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS	crates/singlefs-harness/src/crash.rs	        let (worker_threads, worker_threads_source) = match environment_value {	        let (worker_threads, worker_threads_source) = match environment_value.and(Err::<String, _>(std::env::VarError::NotPresent)) {	-p singlefs-harness --lib -- worker_threads_come_from_the_environment_variable	worker_threads_come_from_the_environment_variable_before_available_parallelism
 增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程	crates/singlefs-harness/src/crash.rs	                text.parse::<NonZeroUsize>().unwrap_or_else(|error| {	                text.parse::<NonZeroUsize>().or(Ok::<NonZeroUsize, std::num::ParseIntError>(NonZeroUsize::MIN)).unwrap_or_else(|error| {	-p singlefs-harness --lib -- zero_worker_threads_in_the_environment_variable	zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back
@@ -147,16 +147,16 @@ K1 × 模型：分配记录树一次至多重写几个节点的上界不往下
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {\n                return Err(\n                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },\n                );\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	            DeviceAgreement::SomeDevicesWithoutAnswer(_full_devices) => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_sampling	allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
-增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
-增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
-增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/segments.rs	            RecordedOperationKind::Barrier => {\n                self.devices_with_unreleased_writes\n                    .remove(&operation.device);\n                self.close_once_every_device_is_released()	            RecordedOperationKind::Barrier => {\n                self.devices_with_unreleased_writes\n                    .remove(&operation.device);\n                SegmentAfterOperation::StaysOpen	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
+增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
+增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
+增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/segments.rs	            RecordedOperationKind::Barrier => {\n                self.devices_with_unreleased_writes\n                    .remove(&operation.device);\n                self.close_once_every_device_is_released()	            RecordedOperationKind::Barrier => {\n                self.devices_with_unreleased_writes\n                    .remove(&operation.device);\n                SegmentAfterOperation::StaysOpen	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
 增补 3 第 3 件（用户 2026-09-20 定案第 2 条）：段内只截前缀，不摆任意真子集（「后发的写先持久」整类零覆盖）	crates/singlefs-harness/src/crash_injection.rs	    if segment_length <= WRITES_A_SUBSET_MASK_HOLDS {\n        let mask = source.below((1u64 << segment_length) - 1);\n        return bits_of(mask, segment_length);\n    }\n    let mut persisted: Vec<bool> = (0..segment_length).map(|_| source.below(2) == 1).collect();\n    if persisted.iter().all(|is_persisted| *is_persisted) {\n        let withheld = usize::try_from(source.below(u64::try_from(segment_length).expect("段长")))\n            .expect("下标装得进 usize");\n        persisted[withheld] = false;\n    }\n    persisted	    let persisted_prefix = source.below(u64::try_from(segment_length).expect("段长"));\n    (0..segment_length)\n        .map(|write| u64::try_from(write).expect("下标") < persisted_prefix)\n        .collect()	-p singlefs-harness --lib -- crash_injection::tests::crash_points_withhold_every_kind_of_write	crash_points_withhold_every_kind_of_write_and_leave_holes_inside_a_segment
 增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器	crates/singlefs-harness/src/crash_injection.rs	        let record_check = check_records_against(\n            &image,\n            &image,\n            &writes,\n            &persisted,\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,\n        );\n        tally.record_checks += 1;	        let record_check = RecordCheck::default();	-p singlefs-harness --lib -- crash_injection::tests::crash_points_are_reproducible	crash_points_are_reproducible_proper_subsets_sorted_by_segment
 增补 3 第 3 件（用户 2026-09-20 定案第 4 条）：崩溃状态又只摆在起点跑完之后（起点那一段的写永远抽不到）	crates/singlefs-harness/src/crash_injection.rs	                (marks.after_make_filesystem..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	                (marks\n                    .after_each_step\n                    .first()\n                    .map_or(0, |(_, after)| *after)\n                    ..marks.after_the_last_finished_step)\n                    .contains(&stream_indexes[*write])	-p singlefs-harness --lib -- crash_injection::tests::crash_points_fall_inside_the_starting_point	crash_points_fall_inside_the_starting_point_too
 增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：已知红第 1 条不读机理（不看 F 有没有真把环里读得到的根挡掉）	crates/singlefs-harness/src/history.rs	        && only_allocated_statistic_above_walked(observation)\n        && allocation_statistic_mechanism_of(observation)\n            .is_some_and(|mechanism| mechanism.the_floor_dropped_readable_roots())	        && only_allocated_statistic_above_walked(observation)	-p singlefs-harness --lib -- history::tests::known_red_forms_are_matched_by_mechanism	known_red_forms_are_matched_by_mechanism_not_by_signature
-增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：这个测试周期的种子基换成别的数（这一周期量过的判红整批作废，两个二进制也不再同基）	crates/singlefs-harness/src/crash_injection.rs	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 12345;	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- the_test_cycle_seed_base	the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle
+增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：这个测试周期的种子基换成别的数（这一周期量过的判红整批作废，两个二进制也不再同基）	crates/singlefs-harness/src/crash_injection.rs	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;	pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 12345;	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- the_test_cycle_seed_base	the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle
 增补 3 第 3 件（用户 2026-09-20 定案第 9 条）：镜像文件不排他创建，撞上同名接着用旧镜像	crates/singlefs-core/src/block_device.rs	            .create_new(true)\n            .open(path)	            .create(true)\n            .truncate(false)\n            .open(path)	-p singlefs-core --lib -- block_device::tests::creating_an_image_that_already_exists	creating_an_image_that_already_exists_is_refused_and_leaves_the_old_bytes_alone
-增补 3 第 3 件（用户 2026-09-20 定案第 3 条的配套口径）：记录核对器第二条判据不认「被流里更晚的写盖过」，合法复用被判成单元缺席	crates/singlefs-harness/src/crash.rs	        let explained_by_a_persisted_legal_later_write =\n            (copy + 1..writes.len()).any(|later_index| {	        let explained_by_a_persisted_legal_later_write =\n            false && (copy + 1..writes.len()).any(|later_index| {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
+增补 3 第 3 件（用户 2026-09-20 定案第 3 条的配套口径）：记录核对器第二条判据不认「被流里更晚的写盖过」，合法复用被判成单元缺席	crates/singlefs-harness/src/crash.rs	        let explained_by_a_persisted_legal_later_write =\n            (copy + 1..writes.len()).any(|later_index| {	        let explained_by_a_persisted_legal_later_write =\n            false && (copy + 1..writes.len()).any(|later_index| {	-p singlefs-checker --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 增补 3 第 3 件（用户 2026-09-20 定案第 7 条，同日定的范围）：随机历史快档的种子基改回写死的别的数（与崩溃注入那个二进制不再同基，跑的是另一批历史）	crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs	const FAST_TIER_FIRST_SEED: u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;	const FAST_TIER_FIRST_SEED: u64 = 0;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- the_five_sampling_tiers	the_five_sampling_tiers_start_from_the_test_cycle_seed_base
 系统配置四类：运行配置那一档的字节预算从 36 改成 35（D22 已定项 26 的 389 / 4 / 36 / 60 里动一个数）	crates/singlefs-core/src/system_configuration.rs	    pub const FIELD_TABLE_BYTES: u64 = 36;	    pub const FIELD_TABLE_BYTES: u64 = 35;	-p singlefs-core --lib -- the_four_mutability_classes_budget	the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total
 系统配置四类：分档记账把系统运行量记进系统运行配置那个计数器（四档的和仍是 489，分法错了）	crates/singlefs-core/src/system_configuration.rs	            SlotFieldMutability::RuntimeQuantity => &mut self.runtime_quantity_bytes,	            SlotFieldMutability::RuntimeQuantity => &mut self.runtime_configuration_bytes,	-p singlefs-core --lib -- to_slot_writes_exactly_the_budgeted_bytes	to_slot_writes_exactly_the_budgeted_bytes_into_each_mutability_class
@@ -191,7 +191,7 @@ C461：一块盘的系统配置全废时 checker 整片报不适用，而恢复
 I-8.7（实例内事务号不重号）判定恒真：同一实例两条记录事务号重号也判绿	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-8.7", record.transaction > previous_transaction, || {	            judgements.judge("I-8.7", true, || {	-p singlefs-harness --test checker_known_bad_images -- repeated_transaction_number	repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant
 I-8.7 不排除事务号 0：空发布记录也进序列（射程「0 不进序列」的阳性对照必红）	crates/singlefs-checker/src/walk.rs	            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {	            if false {	-p singlefs-harness --test checker_known_bad_images -- repeated_transaction_number	repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant
 I-8.7 不按实例分组：跨实例的事务号也拿来比（射程「跨实例不判」必红）	crates/singlefs-checker/src/walk.rs	                .insert(record.instance, (*counter, record.transaction));	                .insert(0, (*counter, record.transaction));	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
-I-8.7 一条记录都不判（层 0 整片报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/walk.rs	            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {	            if true {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+I-8.7 一条记录都不判（层 0 整片报「不适用」，阴性结果与没跑到混在一起）	crates/singlefs-checker/src/walk.rs	            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {	            if true {	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 I-8.7 判别力：事务号取上一条记录（2026-09-21 修掉的那个 bug）在抬 F 之后那份镜像上要被池级 checker 判红	crates/singlefs-core/src/transaction.rs	            transaction: previous.highest_transaction_number_in_this_instance + 1,	            transaction: previous.record.transaction + 1,	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 E156 referenced_slots 丢掉实例表的兜底加回	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	    if !has_instance_table {\n        total += INSTANCE_TABLE_SPAN_SLOTS;\n    }\n	    if false {\n        total += INSTANCE_TABLE_SPAN_SLOTS;\n    }\n	-p singlefs-harness --bin e156_allocation_basis_counts	referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite
 并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号	crates/singlefs-core/src/inode_tree.rs	                                container: inode.0,	                                container: 0,	-p singlefs-core --lib -- inode_tree	the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything
@@ -264,9 +264,9 @@ C504（树表条目宽在走读里无守卫）：走读树表条目之前不判
 步 3（三处写死的前提之二）：第一个文件版本照抄的实例表指针取错成树表指针	crates/singlefs-core/src/transaction.rs	            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),	            instance_table: InstanceTablePlan::Carry(version_to_build_on.tree_table),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3（R8）：树表 0 条、而根指着的实例表或树表不是 mkfs 写的那一版时照样重建账（被换下的那一片成了空闲槽）	crates/singlefs-core/src/mount.rs	    if root.instance_table.head.birth_txg != CheckpointTxg(0)\n        || root.tree_table.head.birth_txg != CheckpointTxg(0)\n    {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_third_writable_mount	a_third_writable_mount_on_a_version_whose_rows_were_written_is_refused_before_touching_the_allocator
 步 3：publish_first_file 不核要建在上面的那一版有没有过文件版本（再走一次会重新建树、重新发对象出生代）；C511 第 3 步起判的是树表条数	crates/singlefs-core/src/transaction.rs	    if tree_table_entries != 0 {	    if false {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
-C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑）：复用豁免不看更晚那次写持没持久（回到落地版，更晚那次写没落盘的真洞上失声）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                true\n                    && later.device == copy_write.device	-p singlefs-harness --test second_transaction_step_zero_layer0 -- a_unit_whose_only_later_write	a_unit_whose_only_later_write_to_the_same_slot_never_landed_is_reported_missing_by_the_record_checker
-C507：复用豁免把「更晚那次写已持久」判反（合法复用被判成单元缺席，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                !persisted[later_index]\n                    && later.device == copy_write.device	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
-C507：复用豁免拿被判那份自己的持久判定当更晚那次写的（索引取错，合法复用照样被判成缺席）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                persisted[copy]\n                    && later.device == copy_write.device	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
+C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑）：复用豁免不看更晚那次写持没持久（回到落地版，更晚那次写没落盘的真洞上失声）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                true\n                    && later.device == copy_write.device	-p singlefs-checker --test second_transaction_step_zero_layer0 -- a_unit_whose_only_later_write	a_unit_whose_only_later_write_to_the_same_slot_never_landed_is_reported_missing_by_the_record_checker
+C507：复用豁免把「更晚那次写已持久」判反（合法复用被判成单元缺席，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                !persisted[later_index]\n                    && later.device == copy_write.device	-p singlefs-checker --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
+C507：复用豁免拿被判那份自己的持久判定当更晚那次写的（索引取错，合法复用照样被判成缺席）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                persisted[copy]\n                    && later.device == copy_write.device	-p singlefs-checker --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 C506：挂载不判每区槽数 S 的区间（盘上自述 3 或 17 照样挂）	crates/singlefs-core/src/root_ring.rs	if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)\n            .contains(&declared_slots_per_region)\n        {	if false\n            && !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)\n                .contains(&declared_slots_per_region)\n        {	-p singlefs-harness --test system_configuration_slots_per_region -- mounting_refuses_a_slots_per_region	mounting_refuses_a_slots_per_region_outside_the_interval_and_accepts_four_eight_and_sixteen
 C506：挂载把 S 读成编译期的 8，不读系统配置槽偏移 362 那一字节	crates/singlefs-core/src/system_configuration.rs	u64::from(bytes[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")]),	8,	-p singlefs-harness --test system_configuration_slots_per_region -- the_ring_geometry_follows	the_ring_geometry_follows_the_slots_per_region_on_disk_not_a_compile_time_constant
 C506：一个区域按编译期的 8 个槽算，不按传进来的 S 算（mkfs 清根环清错长度）	crates/singlefs-core/src/root_ring.rs	slots_per_region.count() * u64::from(fixed_structure_slot_spacing)	8 * u64::from(fixed_structure_slot_spacing)	-p singlefs-harness --test system_configuration_slots_per_region -- the_ring_geometry_follows	the_ring_geometry_follows_the_slots_per_region_on_disk_not_a_compile_time_constant
@@ -285,7 +285,7 @@ I-8.8 ④ 删掉：没提交的事务后面这个实例还在写也判绿（后
 I-8.8 提交标记字节 2..=255 静默读成「不带」（提交标记字节写成 2 那份坏镜像不红）	crates/singlefs-checker/src/walk.rs	            unrecognized => Self::Unrecognized(unrecognized),	            _unrecognized => Self::Absent,	-p singlefs-harness --test checker_known_bad_images -- each_transaction_boundary_bad_image	each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria
 I-8.8 有一事务两条记录的镜像上也报不适用（成立永远报不出来，阴性结果与没对象混在一起）	crates/singlefs-checker/src/walk.rs	    } else if judged_transactions > 0 {	    } else if false {	-p singlefs-harness --test checker_known_bad_images -- one_transaction_over_two_records	one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants
 I-8.8 拿事务号 0 成组：写行与暖机两条空发布各带提交标记，干净镜像上 ③ 就红（射程「事务号 0 不成组」的阳性对照必红）	crates/singlefs-checker/src/walk.rs	            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)	            .filter(|_| true)	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
-I-8.8 拿事务号 0 成组：层 0 上两条空发布都落了盘的崩溃状态判红（层 0 每个崩溃状态都跑 I-8.8 的那一格）	crates/singlefs-checker/src/walk.rs	            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)	            .filter(|_| true)	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+I-8.8 拿事务号 0 成组：层 0 上两条空发布都落了盘的崩溃状态判红（层 0 每个崩溃状态都跑 I-8.8 的那一格）	crates/singlefs-checker/src/walk.rs	            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)	            .filter(|_| true)	-p singlefs-checker --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
 I-8.8 ④ 只看非 0 的号：没提交的事务后面跟一条空发布判绿（2026-09-23 主 agent 定案：失败之后这个实例不再写任何记录，空发布也算）	crates/singlefs-checker/src/walk.rs	                .find(|(_, later)| later.instance == *instance)	                .find(|(_, later)| later.instance == *instance && later.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)	-p singlefs-harness --test checker_known_bad_images -- each_transaction_boundary_bad_image	each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria
 E156 self_release_slots_of_this_publish 数分配不数释放（M24 的反面）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	.filter(|record| record.device == device && record.is_released && record.generation == txg)	.filter(|record| record.device == device && !record.is_released && record.generation == txg)	-p singlefs-harness --bin e156_allocation_basis_counts	first_transaction_self_release_is_one_slot
 E156 G27 退回读内存分配器的行，不读镜像（R3 的反面，M21；2026-09-24 `rustfmt` 把这一行折成四行，原文跟着改）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	let mirror_red = allocated_minus_deferred_mismatches_referenced(\n            &corrupted_pool,\n            accounting_slot,\n            referenced,\n        );	let mirror_red = !allocated_minus_deferred_matches_referenced(\n            &allocator,\n            DeviceIdentity(0),\n            referenced,\n        );	-p singlefs-harness --bin e156_allocation_basis_counts	red_check_reads_the_corrupted_mirror_not_the_live_allocator
@@ -294,8 +294,8 @@ E158 root_choice_repair 系统配置槽余量算术改成 4096-480（第七节 7
 E158 root_choice_repair 活回退事件计数只数最近一次（PC-Nw 该数 2 不数 2）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    for event in rollback_events {	    for event in rollback_events.iter().rev().take(1) {	-p singlefs-harness --bin e158_root_choice_repair	pc_nw_counts_two_live_rollback_events_from_a_hand_written_ledger
 步 1 验收第 4 条：inode 记录的写入时间留成 A 的（覆盖写照抄上一版的写入时间）	crates/singlefs-core/src/transaction.rs	                write_time_seconds: file.write_time_seconds,\n                inode_object_birth: previous.inode_record.object_birth,	                write_time_seconds: previous.inode_record.write_time_seconds,\n                inode_object_birth: previous.inode_record.object_birth,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 步 1 验收第 4 条：反向链算成 A 之前那条（覆盖写的反向链取上一版记录自己的反向链）	crates/singlefs-core/src/transaction.rs	            back_chain: back_chain_of(&previous.record_bytes),\n            file: Some(FileVersionPlan {	            back_chain: previous.record.back_chain,\n            file: Some(FileVersionPlan {	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
-步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform(CommitStep::Barrier) {\n        return Err(pool.roll_back_acquisition(\n            &written,\n            previous_instance,\n            witnessed_journal_tail,\n            cause,\n        ));\n    }\n    Ok(instance)	    Ok(instance)	-p singlefs-harness --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
-步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）	crates/singlefs-harness/src/crash.rs	            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n        }	            publish_of_segment[segment_index] =\n                next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);\n            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n            continue;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform(CommitStep::Barrier) {\n        return Err(pool.roll_back_acquisition(\n            &written,\n            previous_instance,\n            witnessed_journal_tail,\n            cause,\n        ));\n    }\n    Ok(instance)	    Ok(instance)	-p singlefs-checker --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
+步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）	crates/singlefs-harness/src/crash.rs	            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n        }	            publish_of_segment[segment_index] =\n                next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);\n            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n            continue;\n        }	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）	crates/singlefs-harness/src/bad_disk_input.rs	            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),	            BaseImageTier::TreeTableWithoutEntries => false,	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- bad_disk_inputs_never_read_back	bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites
 C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n            },\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        &parameters_of_the_pool,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger,\n            system_configuration,\n            space_admission,\n            selected_version_journal_position,\n            rereads: RereadsOfThisMount {\n                read_stage: read_stage_settled,\n                instance_table_of_the_newest_root,\n            },\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- write_error_after_the_acquisition	write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
 增补 2 收口第 44 行：I-3.10 的比较恒成立（未释放记录的分配代与单元头里的诞生代号不比）	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.10", record.generation == birth_txg, || {	            judgements.judge("I-3.10", true || record.generation == birth_txg, || {	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
@@ -303,7 +303,7 @@ C378（认了）：取号之后写行或暖机报块设备错时把系统配置
 增补 2 收口第 44 行：I-3.10 射程 ② 改读末槽（跨两槽的记录不再取起点槽那个单元头）	crates/singlefs-checker/src/walk.rs	                    record.slot * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	                    (record.slot + record.span_slots - 1) * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
 增补 2 收口第 44 行：I-3.10 射程 ③ 放宽（起点槽的头校验和不过也照样取诞生代号来比）	crates/singlefs-checker/src/walk.rs	    checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET)\n        .then(|| read_u64(header, birth_txg_offset))	    (true || checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET))\n        .then(|| read_u64(header, birth_txg_offset))	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_whose_start_slot_header	an_allocation_record_whose_start_slot_header_does_not_verify_is_not_judged_by_the_allocation_generation_invariant
 增补 2 收口第 54 行（候选 b）：由记录施加出来、根槽从没落盘的那一版不并进遍历（合法镜像上 I-3.1 红回来）	crates/singlefs-checker/src/walk.rs	    for version in &versions_applied_only_by_records {	    for version in versions_applied_only_by_records.iter().take(0) {	-p singlefs-harness --test checker_known_bad_images -- version_applied_only_by_its_journal_record_is_walked	version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_the_leak_on_top_still_reddens_it
-增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版不并进遍历（层 0 残留记录那条流 12 个状态在 I-3.1 上红回来）	crates/singlefs-checker/src/walk.rs	    for version in &versions_applied_only_by_records {	    for version in versions_applied_only_by_records.iter().take(0) {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
+增补 2 收口第 54 行（候选 b）：由记录施加出来的那一版不并进遍历（层 0 残留记录那条流 12 个状态在 I-3.1 上红回来）	crates/singlefs-checker/src/walk.rs	    for version in &versions_applied_only_by_records {	    for version in versions_applied_only_by_records.iter().take(0) {	-p singlefs-checker --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
 增补 2 收口第 54 行（候选 b）第 ① 条放宽：不看实例表里有没有这个实例的行，环里带提交标记的记录都算施加过	crates/singlefs-checker/src/walk.rs	            )\n            && instance_table_rows.iter().any(|row| {	            )\n            || instance_table_rows.iter().any(|row| {	-p singlefs-harness --test checker_known_bad_images -- journal_record_that_no_mount_has_applied_yet	journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds
 增补 2 收口第 54 行（候选 b）第 ③ 条去掉：低于回退下界 F 的那一版也并进遍历	crates/singlefs-checker/src/walk.rs	        let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;	        let at_or_above_the_floor = true || record.checkpoint_txg >= rollback_floor;	-p singlefs-harness --test checker_known_bad_images -- version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor	version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it
 增补 2 收口第 54 行（候选 b）第 ④ 条去掉：环里最旧的有效根已比那一版新（它换下的单元已可回收）也并进遍历	crates/singlefs-checker/src/walk.rs	        let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;	        let its_units_are_not_reclaimable_yet = true || oldest_valid_root_txg < record.checkpoint_txg;	-p singlefs-harness --test checker_known_bad_images -- version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring	version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_the_mount_reclaimed_its_units
@@ -400,7 +400,7 @@ C394 N3：分配器释放时不看「留在已分配」那几块盘（对不上
 C394 N3（用户 2026-09-24 定：对不上那一份的分配记录留在已分配）：核出对不上之后照常释放那一份（记录改写成已释放）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- after_rebuilding_the_previous_version	after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy
 C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）	crates/singlefs-core/src/transaction.rs	                        CopyCheck::Unreadable => QuarantinedCopyReading::UnreadableAfterOneReread,	                        CopyCheck::Unreadable => QuarantinedCopyReading::IntactButAnotherCopyFailed,	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- release_checksum_read_that_keeps_failing	release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_the_mismatch
 C513（复用豁免不判那次复用合不合法）：回收谓词那一判拿掉（证得出过不了也开脱，回到只看更晚那次写落没落盘）	crates/singlefs-harness/src/crash.rs	    release_generation_at_least <= reclaim_threshold_at_most.0	    release_generation_at_least <= reclaim_threshold_at_most.0 || true	-p singlefs-harness --test second_transaction_supplement_two_record_checker_reuse_legality -- an_illegal_reuse	an_illegal_reuse_whose_later_write_landed_no_longer_excuses_the_missing_unit_in_the_record_checker
-C513：回收谓词判得过严（释放代下界取无穷大，抬 F 之后的合法复用也不开脱，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);	    let release_generation_at_least = u64::MAX;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
+C513：回收谓词判得过严（释放代下界取无穷大，抬 F 之后的合法复用也不开脱，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);	    let release_generation_at_least = u64::MAX;	-p singlefs-checker --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/transaction.rs	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.payload_start.0,	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
 并行线一：C491：共享的提交内生块挪到一次发布的第一条记录里点名（末条只点名自己那个数据单元）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
 并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file	crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file
@@ -410,8 +410,8 @@ K2：extent 树下段根的层级取「罩的单元数大于单元数」的最
 并行线一：挂载态打开文件不核 extent key 的 offset 段是不是那个单元的文件字节偏移	crates/singlefs-core/src/mounted_read.rs	            if record.file_offset_in_bytes() != expected_file_offset {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset	mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset
 并行线一：记录读者不判点名项区越没越过 4096（并行线一验收第 4 条第三个变异：点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）	crates/singlefs-core/src/journal.rs	        if payload_end > record_bytes\n            || crc32_castagnoli	        if crc32_castagnoli	-p singlefs-core --lib -- journal::tests::record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser	record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser
 并行线一：一个数据单元的发布也切成两条记录（末条只点名共享内生块）	crates/singlefs-core/src/transaction.rs	    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];	    let data_roles_named_before_the_last_record = &data_roles[..];	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path	sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path
-并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
-并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
+并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-checker --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
+并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-checker --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
 步 1 验收第 4 条 × K2：extent 上段叶条目内联的数据指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *data_pointers\n                            .first()	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *previous\n                            .map_or(data_pointers, |previous_version| {\n                                previous_version.data_pointers.as_slice()\n                            })\n                            .first()	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start_reads_the_second_content_and_the_pool_checker_stays_green	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 并行线一：数据单元头里的锚点偏移写成单元序号（与 extent key 的 offset 段不符：冷启动顺序读在第 1 个单元当场拒，I-1.1）	crates/singlefs-core/src/transaction.rs	            anchor_offset: transaction.payload_start.0,	            anchor_offset: transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
 并行线一：数据单元的净荷字节倒序写（长度、锚点、写序都对，只有内容错：冷启动顺序读回与写入逐字节比对判红）	crates/singlefs-core/src/transaction.rs	            transaction.payload_of(file.content),	            &transaction.payload_of(file.content).iter().rev().copied().collect::<Vec<u8>>(),	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
@@ -470,7 +470,7 @@ P6 后一半：树表 0 条那一版上写行那次发布的本次发布内序
 增补 2 收口表第 58 行：真设备二进制可写挂载失败时不用挂载交回的已落盘账	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	                        &failed.writes_of_persisted_publishes,\n                        &failed.writes_of_failed_publishes,	                        &[],\n                        &failed.writes_of_failed_publishes,	-p singlefs-harness --bin first_transaction_on_device -- failed_writable_mount	failed_writable_mount_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
 增补 2 第 58 行：第一个事务失败时不交写入口的失败账（接替原第 381 行：用例改名）	crates/singlefs-harness/src/scenario.rs	        failed_step: FirstTransactionPathStep::FirstTransaction,\n        cause: format!("{error:?}"),\n        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        failed_step: FirstTransactionPathStep::FirstTransaction,\n        cause: format!("{error:?}"),\n        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- first_transaction_path_failures	first_transaction_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2j）：分配器提交内生块那一处把「每块盘上都没有」报成「小盘写满」；抬 F 的空发布拿不到固定点那条写死用例判出（接替原第 166 行：单元区墙取样点换到 256 槽小盘之后只走得到用户数据那一处）	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot_outside_the_hold	raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise
-代码三方 m2-wave3-code-r1 第四节第 2 条（改法 E）连带：I-3.10 不读下一次挂载会先施加的那一版，第一个事务那条流的快档里 txg 3 记录已落、根槽没落的状态不再评估（逐状态现算的期望红在 I-3.10 评估过的状态数上）	crates/singlefs-checker/src/walk.rs	    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);	    let _ = tree_table_pointer_of_the_version_the_next_mount_applies_first;	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+代码三方 m2-wave3-code-r1 第四节第 2 条（改法 E）连带：I-3.10 不读下一次挂载会先施加的那一版，第一个事务那条流的快档里 txg 3 记录已落、根槽没落的状态不再评估（逐状态现算的期望红在 I-3.10 评估过的状态数上）	crates/singlefs-checker/src/walk.rs	    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);	    let _ = tree_table_pointer_of_the_version_the_next_mount_applies_first;	-p singlefs-checker --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
 E158 root_choice_repair PC2 的 MultiOffsetReadFailingBlockDevice 读判定取反（该在点名的偏移里才报错，改成不在点名的偏移里才报错——这正是根因一原来那个 bug 的镜像）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if self.failing_offsets.contains(&offset.0) {\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)	if !self.failing_offsets.contains(&offset.0) {\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)	-p singlefs-harness --bin e158_root_choice_repair	multi_offset_read_failing_block_device_fails_every_named_offset
 E158 root_choice_repair PC2 的本地系统配置字段表合计默认值退回 481（该在没设环境变量时取今天的 489，D22 已定项 9 字段表合计；实四甲改）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	Err(env::VarError::NotPresent) => 489,	Err(env::VarError::NotPresent) => 481,	-p singlefs-harness --bin e158_root_choice_repair	parse_local_system_configuration_bytes_defaults_to_489_when_unset
 E158 root_choice_repair PC2 甲的 root_ring_slot_targets_for 过滤条件恒真（该只挑点名的 (实例, txg)，改成什么根槽都收）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	.filter(|(_, _, view)| wanted.contains(&(view.instance, view.checkpoint_txg)))	.filter(|(_, _, _view)| true)	-p singlefs-harness --bin e158_root_choice_repair	root_ring_slot_targets_for_finds_exactly_the_named_roots
@@ -488,7 +488,7 @@ E158 root_choice_repair 候选 (b) 分配记录树指称「从内容树反推占
 收口表第 26 行 I-7.9：上限算法改宽（第 4 新的非空有效根改成第 3 新的）	crates/singlefs-checker/src/walk.rs	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(3)	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(2)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：有效根按抬之后的 F 取（上限的「有效」带 txg ≥ 新 F，检查恒真的那一种读法）	crates/singlefs-checker/src/walk.rs	            raising_root,\n            floor_before_the_raise,\n            cache,	            raising_root,\n            raising_root.rollback_floor,\n            cache,	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：抬 F 的根不认同一实例的前一条，按它自己的实例表认 txg 比它小的有效根（回退那条根被当成抬 F，合法历史上误红）	crates/singlefs-checker/src/walk.rs	                root.instance == raising_root.instance\n                    && root.checkpoint_txg < raising_root.checkpoint_txg	                !abandoned_by_instance_table_rows(\n                    &instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)\n                        .unwrap_or_default(),\n                    root,\n                ) && root.checkpoint_txg < raising_root.checkpoint_txg	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_root_at_the_effective_floor	rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version
-收口表第 26 行 I-7.9：一条抬 F 的根都不认（层 0 那条抬过 F 的流上 I-7.9 一个状态都评估不到）	crates/singlefs-checker/src/walk.rs	        if raising_root.rollback_floor <= floor_before_the_raise {	        if raising_root.rollback_floor <= floor_before_the_raise || true {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+收口表第 26 行 I-7.9：一条抬 F 的根都不认（层 0 那条抬过 F 的流上 I-7.9 一个状态都评估不到）	crates/singlefs-checker/src/walk.rs	        if raising_root.rollback_floor <= floor_before_the_raise {	        if raising_root.rollback_floor <= floor_before_the_raise || true {	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 收口表第 46 行 C480：I-9.15 判定恒真（blocks 与 ⌈size ÷ 512⌉ 不等也判成立）	crates/singlefs-checker/src/walk.rs	                        .judge("I-9.15", blocks == logical_length_in_blocks, || {	                        .judge("I-9.15", true, || {	-p singlefs-harness --test checker_known_bad_images -- inode_record_blocks_other_than	inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant
 收口表第 46 行 C480：I-9.15 按 ⌊size ÷ 512⌋ 比（向下取整，写路径写的 6 在 3000 字节上被判红）	crates/singlefs-checker/src/walk.rs	                    size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);	                    size_in_bytes / INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES;	-p singlefs-harness --test checker_known_bad_images -- inode_record_blocks_other_than	inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant
 收口表第 46 行 C480：写路径把 blocks 写回按分到一个 32 KiB 单元算的 64，checker 的 I-9.15 在真镜像上红	crates/singlefs-core/src/records.rs	        writer.put_u64(self.size.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)); // blocks = ⌈size ÷ 512⌉	        writer.put_u64(64); // blocks = ⌈size ÷ 512⌉	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
@@ -514,8 +514,8 @@ E158 root_choice_repair Φ2 系统配置槽这一类证据：newest_self_certifi
 实二十：一次发布之内跳号不断链	crates/singlefs-core/src/recovery.rs	            if u64::from(record.ordinal_within_publish.0)\n                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1\n            {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
 实二十：一个事务的提交标记没出现、下一条换了事务号也照接	crates/singlefs-core/src/recovery.rs	            if !previous_record_of_the_open_publish.is_commit\n                && record.transaction != previous_record_of_the_open_publish.transaction\n            {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- transaction_whose_commit_marker_never_appears	transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied
 实二十：末条不带提交标记也整次施加	crates/singlefs-core/src/recovery.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- last_record_of_the_publish_without_the_commit_marker	last_record_of_the_publish_without_the_commit_marker_keeps_its_publish_unapplied
-实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-core/src/recovery.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if !record.is_commit {	-p singlefs-harness --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
-实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => true,	-p singlefs-harness --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-core/src/recovery.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if !record.is_commit {	-p singlefs-checker --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => true,	-p singlefs-checker --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
 实二十：I-8.9 不判记录标志其余位	crates/singlefs-checker/src/walk.rs	            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
 实二十：I-8.9 不判序号 0	crates/singlefs-checker/src/walk.rs	            if record.ordinal_within_publish == 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
 实二十：I-8.9 不判一次发布之内跳号	crates/singlefs-checker/src/walk.rs	                if ordinal_distance != i128::from(counter_distance) {	                if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
@@ -571,8 +571,8 @@ K4：打开文件时按需读 extent 树的节点数多记一倍（实现自报
 树分裂 读树时不判内部条目宽（55 字节的映射条目当 113 字节的内部条目切）	crates/singlefs-core/src/code_two_tree.rs	        if header.entry_width < internal_width {	        if false && header.entry_width < internal_width {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries	central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries
 树分裂 读树时孩子该有的层级写成父层级（不减一）	crates/singlefs-core/src/code_two_tree.rs	                Some(header.level - 1),	                Some(header.level),	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back	central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back
 树分裂 读树时不核内部节点的子树覆盖区间	crates/singlefs-core/src/code_two_tree.rs	            if first_child.header.smallest_key != header.smallest_key\n                || last_child.header.largest_key != header.largest_key\n            {	            if false {	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers	two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers
-树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败）	crates/singlefs-core/src/recovery.rs	            accounting_tree_nodes: roots.accounting.version.node_count(),	            accounting_tree_nodes: 1,	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
-树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）	crates/singlefs-core/src/transaction.rs	        u64::from(root_header.level) + 1\n    }	        u64::from(root_header.level)\n    }	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- the_accounting_streams_read_the_tree_height_from_the_root_node_header	the_accounting_streams_read_the_tree_height_from_the_root_node_header
+树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败）	crates/singlefs-core/src/recovery.rs	            accounting_tree_nodes: roots.accounting.version.node_count(),	            accounting_tree_nodes: 1,	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）	crates/singlefs-core/src/transaction.rs	        u64::from(root_header.level) + 1\n    }	        u64::from(root_header.level)\n    }	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- the_accounting_streams_read_the_tree_height_from_the_root_node_header	the_accounting_streams_read_the_tree_height_from_the_root_node_header
 释放退回按提示（树分裂换锚点：释放判定路径按「这次换下的上一版角色」查）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => previous_version.placements(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
 步 1 变异：不释放旧落点（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => Vec::<Placement>::new(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：defer 行写 0（树分裂换锚点：记账行按行种类取值）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::DeferQueueBytes => 0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
@@ -772,13 +772,13 @@ E158 root_choice_repair PC2 的环境变量不按给的值解、一律解默认
 实五 445：算上限时读坏的槽一律当没有根、不重读（重读仍坏那一格照抬、F 抬过条款上限）	crates/singlefs-core/src/mount.rs	        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	        if true || !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	-p singlefs-harness --test second_transaction_admission_raises_the_floor_before_refusing -- known_root_slot_still_bad_after_one_reread	known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any_write
 实五 445：算上限时读坏的槽一律当没有根、不重读（实四丙的构造：种子基 + 339 第 1 步，F 抬过条款上限、I-7.9 红）	crates/singlefs-core/src/mount.rs	        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	        if true || !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	-p singlefs-harness --test second_transaction_admission_raises_the_floor_before_refusing -- the_seeded_history_whose_ceiling_read	the_seeded_history_whose_ceiling_read_of_the_genesis_root_slot_fails_once_refuses_the_raise_above_the_ceiling
 实五 445：挂载那一刻就读不出的槽也重读、仍坏就拒（挡住了条款说当没有根的那一格）	crates/singlefs-core/src/mount.rs	        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	        if false && !ring_slots_known_to_hold_a_root.contains(&ring_slot) {	-p singlefs-harness --test second_transaction_admission_raises_the_floor_before_refusing -- root_slot_already_unreadable_at_mount	root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise
-实五 崩在准入推的那一串空发布中间：抬 F 那一串不先写系统配置（带新 F 的根落盘而系统配置里还是旧 F，I-7.12 判红）	crates/singlefs-core/src/transaction.rs	    for index in 0..pool.devices.len() {\n        if let Err(cause) = pool.write_system_configuration_slot(\n            index,\n            journal_tail,	    for index in 0..0 {\n        if let Err(cause) = pool.write_system_configuration_slot(\n            index,\n            journal_tail,	-p singlefs-harness --test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session -- crash_states_inside_the_floor_raise	crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green
-实六 C561：解释扇区的后写不看持久集合（后写落没落都算解释）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                true\n                    && later.device == copy_write.device	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
-实六 C561：换成只看盘上字节的按扇区判（后写在那个扇区上的字节与盘上相同就算解释，SecDisk）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                on_disk.as_ref().is_some_and(|bytes| later_start <= sector_offset && sector_offset + SECTOR_BYTES <= later_end && later.contents.range_still_on_disk(usize::try_from(sector_offset - later_start).expect("扇区偏移"), &bytes[sector_start..sector_start + sector_bytes]))\n                    && later.device == copy_write.device	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
-实六 C561：换回今天的判法（解释扇区的后写要整份还在盘上，C561 的假红回来）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                reader.read(later.device, later.offset, usize::try_from(later.length_in_bytes()).expect("写长")).is_some_and(|later_bytes| later.contents.still_on_disk(&later_bytes))\n                    && later.device == copy_write.device	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- partial_landing_of_the_two_later_nodes	partial_landing_of_the_two_later_nodes_never_reports_a_unit_missing
-实六 C561：解释扇区的后写不过回收谓词（C513 那一道撤回）	crates/singlefs-harness/src/crash.rs	                        .or_insert_with(|| {\n                            reuse_is_not_proven_illegal_by_the_reclaim_predicate(\n                                writes,\n                                earlier_publish_txg,\n                                later_index,\n                            )\n                        })	                        .or_insert_with(|| true)	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- an_illegal_reuse_does_not_explain	an_illegal_reuse_does_not_explain_the_overwritten_unit_in_any_of_the_three_states
-实六 C561：只看副本的头一个扇区（整份开脱的假绿回来：y 盖住 l 的头、l 的后一半没人看）	crates/singlefs-harness/src/crash.rs	    for sector_start in (0..length).step_by(sector_bytes) {	    for sector_start in (0..sector_bytes).step_by(sector_bytes) {	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- a_data_unit_only_half_covered	a_data_unit_only_half_covered_by_a_landed_later_node_is_missing
-实六 C561：check_records 不把崩溃状态的持久集合交给核对器（当成全落）	crates/singlefs-harness/src/crash.rs	        image.writes,\n        &image.persisted,\n        RecordStreamContinuity::OneRecording,	        image.writes,\n        &vec![true; image.writes.len()],\n        RecordStreamContinuity::OneRecording,	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
+实五 崩在准入推的那一串空发布中间：抬 F 那一串不先写系统配置（带新 F 的根落盘而系统配置里还是旧 F，I-7.12 判红）	crates/singlefs-core/src/transaction.rs	    for index in 0..pool.devices.len() {\n        if let Err(cause) = pool.write_system_configuration_slot(\n            index,\n            journal_tail,	    for index in 0..0 {\n        if let Err(cause) = pool.write_system_configuration_slot(\n            index,\n            journal_tail,	-p singlefs-checker --test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session -- crash_states_inside_the_floor_raise	crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green
+实六 C561：解释扇区的后写不看持久集合（后写落没落都算解释）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                true\n                    && later.device == copy_write.device	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
+实六 C561：换成只看盘上字节的按扇区判（后写在那个扇区上的字节与盘上相同就算解释，SecDisk）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                on_disk.as_ref().is_some_and(|bytes| later_start <= sector_offset && sector_offset + SECTOR_BYTES <= later_end && later.contents.range_still_on_disk(usize::try_from(sector_offset - later_start).expect("扇区偏移"), &bytes[sector_start..sector_start + sector_bytes]))\n                    && later.device == copy_write.device	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
+实六 C561：换回今天的判法（解释扇区的后写要整份还在盘上，C561 的假红回来）	crates/singlefs-harness/src/crash.rs	                persisted[later_index]\n                    && later.device == copy_write.device	                reader.read(later.device, later.offset, usize::try_from(later.length_in_bytes()).expect("写长")).is_some_and(|later_bytes| later.contents.still_on_disk(&later_bytes))\n                    && later.device == copy_write.device	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- partial_landing_of_the_two_later_nodes	partial_landing_of_the_two_later_nodes_never_reports_a_unit_missing
+实六 C561：解释扇区的后写不过回收谓词（C513 那一道撤回）	crates/singlefs-harness/src/crash.rs	                        .or_insert_with(|| {\n                            reuse_is_not_proven_illegal_by_the_reclaim_predicate(\n                                writes,\n                                earlier_publish_txg,\n                                later_index,\n                            )\n                        })	                        .or_insert_with(|| true)	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- an_illegal_reuse_does_not_explain	an_illegal_reuse_does_not_explain_the_overwritten_unit_in_any_of_the_three_states
+实六 C561：只看副本的头一个扇区（整份开脱的假绿回来：y 盖住 l 的头、l 的后一半没人看）	crates/singlefs-harness/src/crash.rs	    for sector_start in (0..length).step_by(sector_bytes) {	    for sector_start in (0..sector_bytes).step_by(sector_bytes) {	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- a_data_unit_only_half_covered	a_data_unit_only_half_covered_by_a_landed_later_node_is_missing
+实六 C561：check_records 不把崩溃状态的持久集合交给核对器（当成全落）	crates/singlefs-harness/src/crash.rs	        image.writes,\n        &image.persisted,\n        RecordStreamContinuity::OneRecording,	        image.writes,\n        &vec![true; image.writes.len()],\n        RecordStreamContinuity::OneRecording,	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_false_red_and_the_misaligned_hole	the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing
 实六 甲二：原地写的子集掩码不右移（单元写全落那一位串进原地写）	crates/singlefs-harness/src/crash.rs	                            ordinal_within_the_segment / 2,	                            ordinal_within_the_segment,	-p singlefs-harness --lib -- the_quick_tier_plan_takes_every_in_place_subset	the_quick_tier_plan_takes_every_in_place_subset_with_the_copy_on_write_writes_none_or_all
 实六 甲二：有单元写的段少算单元写全落那一半状态	crates/singlefs-harness/src/crash.rs	                let copy_on_write_choices: u64 = if copy_on_write_writes.is_empty() {\n                    1\n                } else {\n                    2\n                };	                let copy_on_write_choices: u64 = if copy_on_write_writes.is_empty() {\n                    1\n                } else {\n                    1\n                };	-p singlefs-harness --lib -- the_quick_tier_plan_takes_every_in_place_subset	the_quick_tier_plan_takes_every_in_place_subset_with_the_copy_on_write_writes_none_or_all
 实六 甲二：单元写也算原地写（甲二退成全量）	crates/singlefs-harness/src/crash.rs	            StepKind::UnitWrite => false,	            StepKind::UnitWrite => true,	-p singlefs-harness --lib -- the_quick_tier_plan_takes_every_in_place_subset	the_quick_tier_plan_takes_every_in_place_subset_with_the_copy_on_write_writes_none_or_all
@@ -795,7 +795,7 @@ E158 root_choice_repair PC2 的环境变量不按给的值解、一律解默认
 实六 续跑：强制从头跑的开关不起作用（R5）	crates/singlefs-harness/src/layer0_progress.rs	            match (settings.start, existing) {	            match (Layer0ResumeStart::ResumeFromTheProgressFile, existing) {	-p singlefs-harness --test crash_enumeration_resumes_from_its_progress_file -- starting_over_reruns_every_slice	starting_over_reruns_every_slice_even_with_a_valid_progress_file
 实六 续跑：同一片出现两次一律整份作废（计数相同也不去重，R3）	crates/singlefs-harness/src/layer0_progress.rs	            Some(earlier) if *earlier != tally => {	            Some(_earlier) => {	-p singlefs-harness --lib -- the_same_slice_twice	the_same_slice_twice_is_deduplicated_only_when_the_counts_are_identical
 实六 续跑：没设进度目录也留进度文件	crates/singlefs-harness/src/layer0_progress.rs	            Err(std::env::VarError::NotPresent) => return Self::NoProgressFile,	            Err(std::env::VarError::NotPresent) => PathBuf::from("/tmp"),	-p singlefs-harness --lib -- the_environment_decides	the_environment_decides_whether_and_how_the_progress_file_is_kept
-实六 固定脚本：层 0 切段不带卸载入口段（C557 那一道在带卸载的流上当场断言停）	crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs	        &entry_spans,\n        &geometry(),	        &[],\n        &geometry(),	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_of_the_quick_tier	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
+实六 固定脚本：层 0 切段不带卸载入口段（C557 那一道在带卸载的流上当场断言停）	crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs	        &entry_spans,\n        &geometry(),	        &[],\n        &geometry(),	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_of_the_quick_tier	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
 实七 生成器：抽到崩溃恢复抛弃根时生成的是一次可写挂载（快档里这一种操作一次都没有）	crates/singlefs-harness/src/history.rs	        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {\n            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot\n        }	        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {\n            HistoryOperation::CloseAndMountWritable\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier	random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
 实七 执行器：会话推的那几串抬 F 不交模型比（小盘准入判着的取样点判出）	crates/singlefs-harness/src/history.rs	        judge_floor_raises_pushed_by_the_session(model, floor_raises);	        judge_floor_raises_pushed_by_the_session(model, &[]);	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged	unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
 实七 执行器：经会话推了抬 F 再发成的覆盖写不记推了几串（发布那一处的推跑没跑到看不出）	crates/singlefs-harness/src/history.rs	                    floor_raises_pushed_by_the_session: floor_raises.len(),	                    floor_raises_pushed_by_the_session: 0,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged	unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
@@ -817,8 +817,8 @@ E158 root_choice_repair PC2 的环境变量不按给的值解、一律解默认
 实八 (d) 按注入点认：实现写的行与重算的行相同反而不认	crates/singlefs-harness/src/fault_injection.rs	                (Some(written), Some(recomputed)) if written == recomputed	                (Some(written), Some(recomputed)) if written != recomputed	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
 实八 (d) 模型：「写的实例表行」对不上时不带实现写的行	crates/singlefs-harness/src/model.rs	                    disagreement.implementation_rows_written = Some(rows_written.clone());	                    disagreement.implementation_rows_written = None;	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
 实八 (d) 按注入点认：有被吞的根槽写也不重算可写挂载写的行	crates/singlefs-harness/src/fault_injection.rs	        let instance_rows_with_the_swallowed_roots_rebuilt_from_their_records =\n            if swallowed_roots.is_empty() {	        let instance_rows_with_the_swallowed_roots_rebuilt_from_their_records =\n            if true {	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- the_rows_written_after	the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied
-实八 (c) 记录核对器：不按恢复落到那一版的实例表豁免被抛弃的发布（被抛弃那次发布的单元被合法复用后判缺席）	crates/singlefs-harness/src/crash.rs	        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)\n            && !abandoned_by_the_landed_version(&publish)	        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- units_of_a_publish_the_landed_version	units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red
-实八 (c) checker：按实例表判抛弃时 T = Ti 也算被抛弃（行 (i, Ti) 那一版自己的单元缺席不再判红）	crates/singlefs-checker/src/walk.rs	        self.rows\n            .iter()\n            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)	        self.rows\n            .iter()\n            .any(|row| row.instance == instance && checkpoint_txg >= row.published_checkpoint_txg)	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- units_of_a_publish_the_landed_version	units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red
+实八 (c) 记录核对器：不按恢复落到那一版的实例表豁免被抛弃的发布（被抛弃那次发布的单元被合法复用后判缺席）	crates/singlefs-harness/src/crash.rs	        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)\n            && !abandoned_by_the_landed_version(&publish)	        if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- units_of_a_publish_the_landed_version	units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red
+实八 (c) checker：按实例表判抛弃时 T = Ti 也算被抛弃（行 (i, Ti) 那一版自己的单元缺席不再判红）	crates/singlefs-checker/src/walk.rs	        self.rows\n            .iter()\n            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)	        self.rows\n            .iter()\n            .any(|row| row.instance == instance && checkpoint_txg >= row.published_checkpoint_txg)	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- units_of_a_publish_the_landed_version	units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red
 E156 M35（第 4 次重跑登记第九节）：岔路 1 的族的分配器退回不装根环表（挂载内不回收）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	allocator_after_make_filesystem(parameters, devices, genesis)	allocator_without_the_root_ring(devices, genesis)	-p singlefs-harness --bin e156_allocation_basis_counts	hh_without_holes_at_four_slots_per_region_reclaims_each_tree_table_one_ring_length_after_release
 E156 M36（第 4 次重跑登记第九节）：门槛不从镜像的根环最旧有效根读，改成「最新有效根 − 环里根数 + 1」（漏看洞槽里的旧根）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	    valid_root_txgs\n        .iter()\n        .min()\n        .copied()\n        .unwrap_or(0)\n        .max(effective_floor)	    (valid_root_txgs.iter().max().copied().unwrap_or(0) + 1)\n        .saturating_sub(u64::try_from(valid_root_txgs.len()).unwrap_or(0))\n        .max(effective_floor)	-p singlefs-harness --bin e156_allocation_basis_counts	hh_back_hole_at_four_slots_per_region_pins_the_threshold_to_txg_four_until_txg_twenty_eight
 E156 M37（第 4 次重跑登记第九节）：W6 的切段点包进根槽那一次写（c2 造不出洞）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	CrashCutPoint::BeforeTheRootSlotWrite => root_slot_write_index,	CrashCutPoint::BeforeTheRootSlotWrite => root_slot_write_index + 1,	-p singlefs-harness --bin e156_allocation_basis_counts	hh_back_hole_at_four_slots_per_region_pins_the_threshold_to_txg_four_until_txg_twenty_eight
@@ -906,9 +906,9 @@ E158 root_choice_repair 第 2 次跑：V2 的「没拦到」取反（拦到了
 实审 B3b 第 2 条（崩溃注入补可写挂载）：崩溃后的可写挂载起在空盘上，不是那份崩溃后镜像	crates/singlefs-harness/src/crash_injection.rs	                device.image = sectors.clone();	                device.image = crate::crash::SparseDevice::default();	-p singlefs-harness --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
 实审 B3b 第 2 条（二次崩溃）：挂载那一段的取号写一律归进写行，二次崩溃摆不到取号那一段	crates/singlefs-harness/src/crash_injection.rs	                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {\n                    WritableMountPhase::Acquisition\n                }	                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => {\n                    WritableMountPhase::RowPublish\n                }	-p singlefs-harness --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
 实审 B3b 第 2 条（二次崩溃）：二次崩溃状态上允许的版本不并进这次挂载写出的根（恢复落到挂载写的根上就判「模型没提交过」）	crates/singlefs-harness/src/crash_injection.rs	            versions_allowed.insert(model_root_key(instance, checkpoint_txg), content.clone());	            let _not_allowed = (model_root_key(instance, checkpoint_txg), content);	-p singlefs-harness --test crash_injection_writable_mount_after_the_crash	every_crash_state_is_followed_by_a_writable_mount_one_publish_the_checker_and_second_crashes_in_each_phase
-实审 B3b c561 计数行：评过的状态数少于闭式也打 exhaustive=true	crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs	        enumerated.states == enumerated.closed_form_states,	        enumerated.states <= enumerated.closed_form_states,	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
-实审 B3b c561 线程行：这一趟跑的片报得比总片数多一片（读回 + 跑的 ≠ 总片数）	crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs	        enumerated.slices,\n        enumerated.elapsed_seconds	        enumerated.slices + 1,\n        enumerated.elapsed_seconds	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
-实审 B3b c561 计数行：闭式按 σ 两遍算（评满了也打不出 exhaustive=true）	crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs	        closed_form_states: closed_form_state_count(&[segment.to_vec()]),	        closed_form_states: closed_form_state_count(&[segment.to_vec(), segment.to_vec()]),	-p singlefs-harness --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
+实审 B3b c561 计数行：评过的状态数少于闭式也打 exhaustive=true	crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs	        enumerated.states == enumerated.closed_form_states,	        enumerated.states <= enumerated.closed_form_states,	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
+实审 B3b c561 线程行：这一趟跑的片报得比总片数多一片（读回 + 跑的 ≠ 总片数）	crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs	        enumerated.slices,\n        enumerated.elapsed_seconds	        enumerated.slices + 1,\n        enumerated.elapsed_seconds	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
+实审 B3b c561 计数行：闭式按 σ 两遍算（评满了也打不出 exhaustive=true）	crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs	        closed_form_states: closed_form_state_count(&[segment.to_vec()]),	        closed_form_states: closed_form_state_count(&[segment.to_vec(), segment.to_vec()]),	-p singlefs-checker --test record_checker_judges_absence_by_the_persisted_set -- the_sigma_enumeration_prints	the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape
 实审 A2a 第 15 条：短于一条记录的环不拒	crates/singlefs-core/src/make_filesystem.rs	    if ring_bytes < JOURNAL_RECORD_BYTES {	    if ring_bytes < JOURNAL_RECORD_BYTES / 4 {	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_ring_shorter_than_one_record_is_refused_before_any_write_and_one_record_long_passes_on_to_the_safety_factor_check
 实审 A2a 第 16 条：根槽装不下根记录不在开头拒（走到 to_slot 的断言）	crates/singlefs-core/src/make_filesystem.rs	    if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES {	    if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES / 2 {	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_root_slot_narrower_than_the_root_record_is_refused_before_any_write
 实审 A2a 第 16 条：根槽宽过槽距不拒	crates/singlefs-core/src/make_filesystem.rs	    if geometry.physical_block_size > geometry.fixed_structure_slot_spacing {	    if geometry.physical_block_size > geometry.fixed_structure_slot_spacing * 2 {	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_root_slot_wider_than_the_slot_spacing_is_refused_and_one_as_wide_is_not
@@ -943,8 +943,8 @@ E158 root_choice_repair 第 3 次跑 M10：K3 / K4 按错误成员分、不按
 E158 root_choice_repair 第 3 次跑 M12：本地常量每区槽数 mkfs 默认写成 9（V3 该停不停）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	            local_value: 8,	            local_value: 9,	-p singlefs-harness --bin e158_root_choice_repair	third_run_constants_and_anchors_all_pass
 E158 root_choice_repair 第 3 次跑 M13：V2 不查点了名的落点有没有被拦到读（从没读到的落点该报不报）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	            .filter(|(_, intercepted)| **intercepted == 0)\n            .map(|(fault, _)| fault.label.clone())	            .filter(|(_, intercepted)| **intercepted == u64::MAX)\n            .map(|(fault, _)| fault.label.clone())	-p singlefs-harness --bin e158_root_choice_repair	named_target_never_read_is_reported_as_not_intercepted
 E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数读坏的（读坏的一次该数 1 数 0）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        if fails {\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)?;\n        for (range_offset, range_length) in zeroed {\n            let start = range_offset.saturating_sub(offset.0);\n            let end = (range_offset + range_length)\n                .saturating_sub(offset.0)\n                .min(length);	        if fails {\n            let mut guard = self.state.borrow_mut();\n            guard.read_calls -= 1;\n            guard.read_bytes -= length;\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)?;\n        for (range_offset, range_length) in zeroed {\n            let start = range_offset.saturating_sub(offset.0);\n            let end = (range_offset + range_length)\n                .saturating_sub(offset.0)\n                .min(length);	-p singlefs-harness --bin e158_root_choice_repair	failed_reads_are_counted_as_reads
-实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（恢复落到由记录重建的一版时找不到它的根槽写、读不出它的实例表，被它抛弃的发布的豁免落空）	crates/singlefs-harness/src/crash_injection.rs	            &writes,\n            &persisted,\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,	            &writes[..first_write_of_the_segment + writes_in_the_segment],\n            &persisted[..first_write_of_the_segment + writes_in_the_segment],\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_state_landing_on_a_version_rebuilt_from_records	crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons
-实审 B3c-1：记录核对器在实例表读得出时把每次发布都当被抛弃（整条流让由记录重建的那一版的实例表读得出之后，没被抛弃的前一版缺单元不再判红）	crates/singlefs-harness/src/crash.rs	            .is_some_and(|table| table.abandons(publish.instance, publish.checkpoint_txg))	            .is_some()	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- on_the_whole_stream_a_kept_publish_missing_a_unit	on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red
+实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（恢复落到由记录重建的一版时找不到它的根槽写、读不出它的实例表，被它抛弃的发布的豁免落空）	crates/singlefs-harness/src/crash_injection.rs	            &writes,\n            &persisted,\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,	            &writes[..first_write_of_the_segment + writes_in_the_segment],\n            &persisted[..first_write_of_the_segment + writes_in_the_segment],\n            RecordStreamContinuity::OneRecording,\n            report.effective_root,	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- crash_state_landing_on_a_version_rebuilt_from_records	crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons
+实审 B3c-1：记录核对器在实例表读得出时把每次发布都当被抛弃（整条流让由记录重建的那一版的实例表读得出之后，没被抛弃的前一版缺单元不再判红）	crates/singlefs-harness/src/crash.rs	            .is_some_and(|table| table.abandons(publish.instance, publish.checkpoint_txg))	            .is_some()	-p singlefs-checker --test second_transaction_supplement_three_crash_injection -- on_the_whole_stream_a_kept_publish_missing_a_unit	on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red
 实审 B3a-2 第 1 条：录制器把连着的屏障不分设备并成一道（盘 1 漏发屏障的流与每盘都发的逐步相同）	crates/singlefs-harness/src/lib.rs	            .any(|previous| previous.device == operation.device);	            .any(|previous| previous.kind == RecordedOperationKind::Barrier);	-p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites -- a_device_that_misses_its_barrier	a_device_that_misses_its_barrier_records_a_different_stream_than_every_device_barriering
 实审 B3a-2 第 1 条：录制器把连着的屏障不分设备并成一道（第一个事务各路径的操作数与种类串退回按池数）	crates/singlefs-harness/src/lib.rs	            .any(|previous| previous.device == operation.device);	            .any(|previous| previous.kind == RecordedOperationKind::Barrier);	-p singlefs-harness --test first_transaction_step_five_publish -- recorded_paths_match	recorded_paths_match_the_registered_segment_sequences
 实审 B3a-2 第 1 条：录制器把连着的屏障不分设备并成一道（mkfs 的操作数退回 21）	crates/singlefs-harness/src/lib.rs	            .any(|previous| previous.device == operation.device);	            .any(|previous| previous.kind == RecordedOperationKind::Barrier);	-p singlefs-harness --test first_transaction_step_one_mkfs -- recorded_stream_matches	recorded_stream_matches_the_registered_mkfs_segment_sequence
@@ -956,8 +956,8 @@ E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数
 实审 B3a-2 第 1 条：设备日志的期望把别的盘的屏障也投成这块盘的 FLUSH	crates/singlefs-harness/src/device_log.rs	            RecordedOperationKind::Barrier => {\n                if operation.device != device {\n                    continue;\n                }	            RecordedOperationKind::Barrier => {	-p singlefs-harness --lib -- missing_flush_is_reported	missing_flush_is_reported_as_the_first_divergence
 实审 B3a-2 第 4 条：原地覆写的写也只取两态（第三态没补，第一条流全量退回 67108885）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites -- the_first_transaction_stream_keeps_its_segments	the_first_transaction_stream_keeps_its_segments_and_takes_the_third_state_on_its_system_configuration_writes
 实审 B3a-2 第 4 条：原地覆写的写也只取两态（手摆的写表上两式退回两态的闭式）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites -- in_place_overwrites_take_three_states	in_place_overwrites_take_three_states_and_every_other_write_two
-实审 B3a-2 第 4 条：原地覆写的写也只取两态（第一条流甲二快档退回 29）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
-实审 B3a-2 第 4 条：原地覆写的写也只取两态（第二条流甲二快档退回 205）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_of_the_quick_tier	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
+实审 B3a-2 第 4 条：原地覆写的写也只取两态（第一条流甲二快档退回 29）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+实审 B3a-2 第 4 条：原地覆写的写也只取两态（第二条流甲二快档退回 205）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_step_zero_layer0 -- every_crash_state_of_the_quick_tier	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
 实审 B3a-2 第 4 条：认原地覆写时不看基镜像（基镜像上有旧内容的那一槽被当成空的）	crates/singlefs-harness/src/crash.rs	    base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps	    an_earlier_write_with_bytes_overlaps	-p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites -- in_place_overwrites_take_three_states	in_place_overwrites_take_three_states_and_every_other_write_two
 实审 B3a-2 第 4 条：撕裂那一态不叠撕裂镜像（与没持久是同一个镜像，第三态没真生成）	crates/singlefs-harness/src/crash.rs	                    persisted[torn_image.torn_image_index] = true;	                    persisted[torn_image.torn_image_index] = false;	-p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites -- the_torn_state_of_a_system_configuration_rotation	the_torn_state_of_a_system_configuration_rotation_is_generated_and_fed_to_the_checker_and_the_record_checker
 实审 B3a-2 第 4 条：撕裂镜像取整次新写（新旧不同那一截全是新的，读得出新的）	crates/singlefs-harness/src/crash.rs	    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;	    let first_old_byte = first_differing_byte + differing_span_in_bytes;	-p singlefs-harness --lib -- the_torn_image_keeps	the_torn_image_keeps_the_first_half_of_the_differing_bytes_new_and_the_rest_old
@@ -1023,13 +1023,13 @@ E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数
 实审 A4b：中央映射树叶层一次都不按会切算	crates/singlefs-core/src/admission.rs	                if entries_at_most <= leaf_entries {	                if true {	-p singlefs-core --lib -- admission::tests	central_mapping_term_counts_changed_paths_and_splits_on_every_level
 实审 A4b：中央映射树内部层一次都不按会切算	crates/singlefs-core/src/admission.rs	                if nodes_below_at_most <= internal_entries {	                if true {	-p singlefs-core --lib -- admission::tests	central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities
 实审 A4b：压小容量下量 ckpt_cost 照旧按节点格式的容量算	crates/singlefs-core/src/admission.rs	                    node_capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),	                    CodeTwoTreeNodeCapacities::FromTheNodeFormat.of_tree(MultiLevelCodeTwoTree::CentralMapping),	-p singlefs-harness --test admission_checkpoint_cost_per_device_paths -- every_empty_publish_under_capped_code_two_tree_capacities	every_empty_publish_under_capped_code_two_tree_capacities_fits_in_the_worst_case_checkpoint_cost_counted_with_them
-实审 B3a-3b：原地覆写的写也只取两态（只做过 mkfs 的池那条流快档退回 22）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
-实审 B3a-3b：原地覆写的写也只取两态（取号屏障那条流展开的两段退回 1 + 3 + 262143）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
-实审 B3a-3b：层 0 状态数函数不认原地覆写（按位置寻址的树那几条流枚举出来的比它多）	crates/singlefs-harness/src/crash.rs	        expansion,\n        &TearableInPlaceOverwrites::of(base, writes),\n    )	        expansion,\n        &TearableInPlaceOverwrites::none(writes.len()),\n    )	-p singlefs-harness --test second_transaction_position_addressed_trees_layer0 -- every_position_addressed_tree_stream_recovers_cleanly	every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments
-实审 B3a-3b：原地覆写的写也只取两态（并行线一快档退回 102）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
-实审 B3a-3b：原地覆写的写也只取两态（L8 退回 20 个状态、根槽已落的 4 个）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
-实审 B3a-3b：原地覆写的写也只取两态（树分裂快档每条流退回 8）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
-实审 B3a-3b：checker 在没有文件的一版上也判一格映射 key 与单元头相符（第一条流快档里它不再只在 txg 3 的根已持久时评估）	crates/singlefs-checker/src/walk.rs	        self.walk_central_mapping_entries(&leaf_entries);	        self.judgements\n            .judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, true, String::new);\n        self.walk_central_mapping_entries(&leaf_entries);	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+实审 B3a-3b：原地覆写的写也只取两态（只做过 mkfs 的池那条流快档退回 22）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
+实审 B3a-3b：原地覆写的写也只取两态（取号屏障那条流展开的两段退回 1 + 3 + 262143）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
+实审 B3a-3b：层 0 状态数函数不认原地覆写（按位置寻址的树那几条流枚举出来的比它多）	crates/singlefs-harness/src/crash.rs	        expansion,\n        &TearableInPlaceOverwrites::of(base, writes),\n    )	        expansion,\n        &TearableInPlaceOverwrites::none(writes.len()),\n    )	-p singlefs-checker --test second_transaction_position_addressed_trees_layer0 -- every_position_addressed_tree_stream_recovers_cleanly	every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments
+实审 B3a-3b：原地覆写的写也只取两态（并行线一快档退回 102）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
+实审 B3a-3b：原地覆写的写也只取两态（L8 退回 20 个状态、根槽已落的 4 个）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实审 B3a-3b：原地覆写的写也只取两态（树分裂快档每条流退回 8）	crates/singlefs-harness/src/crash.rs	            Self::NotPersistedTornOrPersisted => 3,	            Self::NotPersistedTornOrPersisted => 2,	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+实审 B3a-3b：checker 在没有文件的一版上也判一格映射 key 与单元头相符（第一条流快档里它不再只在 txg 3 的根已持久时评估）	crates/singlefs-checker/src/walk.rs	        self.walk_central_mapping_entries(&leaf_entries);	        self.judgements\n            .judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, true, String::new);\n        self.walk_central_mapping_entries(&leaf_entries);	-p singlefs-checker --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
 实审 C11 三方各算头宽：core 那份 key 区间只算一个 key	crates/singlefs-core/src/unit.rs	        .expect("key 区间两个 key")\n        * key_width;	        .expect("key 区间两个 key")\n        * key_width\n        / 2;	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：core 那份漏了预留位 29	crates/singlefs-core/src/unit.rs	    index_node_plain_header_end(key_width) + reserved_bytes()	    index_node_plain_header_end(key_width)	-p singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width -- core_checker_and_model	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte
 实审 C11 三方各算头宽：core 建码 2 节点时明文头末尾错取含预留位的头宽	crates/singlefs-core/src/unit.rs	    assert_eq!(largest_key.len(), key_width);\n    let header_end = index_node_plain_header_end(key_width);	    assert_eq!(largest_key.len(), key_width);\n    let header_end = index_node_header_bytes(key_width);	-p singlefs-core --lib -- unit::tests	empty_tree_table_node_has_key_width_at_51_and_a_sealed_header
@@ -1076,11 +1076,11 @@ E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数
 实审 A2c Q5：对拍不认装在里面的抬 F 的错（理由报成说不清）	crates/singlefs-harness/src/model_comparison.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            refusal_reason_of_mount_error(&failed.cause)\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --lib -- model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside	model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside
 实审 A2c Q5：对拍不认装在里面的抬 F 报的上限	crates/singlefs-harness/src/model_comparison.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            reported_ceiling_of_mount_error(&failed.cause)\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => None,	-p singlefs-harness --lib -- model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside	model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside
 实审 A2c Q5：对拍不认装在里面的抬 F 点名的读坏根环槽	crates/singlefs-harness/src/model_comparison.rs	        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {\n            root_ring_slot_still_bad_after_one_reread_of_mount_error(&failed.cause)\n        }	        MountError::FloorRaiseFailedAfterTheMountsPublishes(_failed) => None,	-p singlefs-harness --lib -- model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside	model_comparison::tests::a_floor_raise_failure_after_the_mounts_publishes_is_judged_by_the_floor_raise_error_inside
-实审 B3a-3c：按位置寻址的树计数行又直接算 1u64 << |段|（第三条流 322 写的单元写段移位溢出，快用例 panic）	crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs	        let proper_subsets = 1u64.checked_shl(segment_length)?.checked_sub(1)?;	        let proper_subsets = (1u64 << segment_length).checked_sub(1)?;	-p singlefs-harness --test second_transaction_position_addressed_trees_layer0 -- every_position_addressed_tree_stream_recovers_cleanly	every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments
-实审 B3a-3c：L8 叶容器数又按写死的 5 个角色取（63 片叶容器，点名 74 项、末条 7 项）	crates/singlefs-harness/tests/second_transaction_parallel_line_three_spill_over_layer0.rs	    let new_inode_count = new_inode_count_filling(leaf_containers);	    let new_inode_count = new_inode_count_filling(JOURNAL_NAMED_ENTRIES_PER_RECORD - 5 + 1);	-p singlefs-harness --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
-实审 B3a-3c：树分裂 层 0：叶切在末尾（根分裂那条流录之后是 L9 L1 I2，不是 L5 L5 I2）	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() - 1);	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
-实审 B3a-3c：树分裂 层 0：内部节点装不下也不切（两层连着分裂那条流录之后还是 L3 L3 L4 I3，不长第三层）	crates/singlefs-core/src/code_two_tree.rs	            if children.len() > capacity.internal_entries {	            if false && children.len() > capacity.internal_entries {	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
-实审 B3a-3c：树分裂 层 0：删空的叶不从父节点摘掉（摘空那条流在发布里就停在「规划与读回都不交出空节点」）	crates/singlefs-core/src/code_two_tree.rs	            if children[child_index].node.entry_count() == 0 {	            if false && children[child_index].node.entry_count() == 0 {	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+实审 B3a-3c：按位置寻址的树计数行又直接算 1u64 << |段|（第三条流 322 写的单元写段移位溢出，快用例 panic）	crates/singlefs-checker/tests/second_transaction_position_addressed_trees_layer0.rs	        let proper_subsets = 1u64.checked_shl(segment_length)?.checked_sub(1)?;	        let proper_subsets = (1u64 << segment_length).checked_sub(1)?;	-p singlefs-checker --test second_transaction_position_addressed_trees_layer0 -- every_position_addressed_tree_stream_recovers_cleanly	every_position_addressed_tree_stream_recovers_cleanly_in_every_state_of_its_small_segments
+实审 B3a-3c：L8 叶容器数又按写死的 5 个角色取（63 片叶容器，点名 74 项、末条 7 项）	crates/singlefs-checker/tests/second_transaction_parallel_line_three_spill_over_layer0.rs	    let new_inode_count = new_inode_count_filling(leaf_containers);	    let new_inode_count = new_inode_count_filling(JOURNAL_NAMED_ENTRIES_PER_RECORD - 5 + 1);	-p singlefs-checker --test second_transaction_parallel_line_three_spill_over_layer0 -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实审 B3a-3c：树分裂 层 0：叶切在末尾（根分裂那条流录之后是 L9 L1 I2，不是 L5 L5 I2）	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() - 1);	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+实审 B3a-3c：树分裂 层 0：内部节点装不下也不切（两层连着分裂那条流录之后还是 L3 L3 L4 I3，不长第三层）	crates/singlefs-core/src/code_two_tree.rs	            if children.len() > capacity.internal_entries {	            if false && children.len() > capacity.internal_entries {	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+实审 B3a-3c：树分裂 层 0：删空的叶不从父节点摘掉（摘空那条流在发布里就停在「规划与读回都不交出空节点」）	crates/singlefs-core/src/code_two_tree.rs	            if children[child_index].node.entry_count() == 0 {	            if false && children[child_index].node.entry_count() == 0 {	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
 实审 C11b 算式改字面量：码 1 载荷起点写成 135（不等于类身份段 105 + 预留位 29）	crates/singlefs-format/src/lib.rs	pub const DATA_UNIT_PAYLOAD_OFFSET: u64 = 134;	pub const DATA_UNIT_PAYLOAD_OFFSET: u64 = 135;	-p singlefs-format --lib -- arithmetic_initializer_literals	arithmetic_initializer_literals_equal_the_expressions_they_replaced
 实审 C11b 算式改字面量：码 3 记录区起点写成 137（不等于类身份段 107 + 预留位 29）	crates/singlefs-format/src/lib.rs	pub const PACKED_UNIT_RECORDS_OFFSET: u64 = 136;	pub const PACKED_UNIT_RECORDS_OFFSET: u64 = 137;	-p singlefs-format --lib -- arithmetic_initializer_literals	arithmetic_initializer_literals_equal_the_expressions_they_replaced
 实审 C11b 算式改字面量：journal 新根段写成 189（不等于 86 + 86 + 8 + 8）	crates/singlefs-format/src/lib.rs	pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 188;	pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 189;	-p singlefs-format --lib -- arithmetic_initializer_literals	arithmetic_initializer_literals_equal_the_expressions_they_replaced
@@ -1108,7 +1108,7 @@ E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数
 实审 B3c-2 第 2 件：第二截把崩溃态镜像当成恢复后的镜像交（挂载之后的池上丢了单元看不见）	crates/singlefs-harness/src/crash_injection.rs	            crash_image,\n            pool_after_the_mount,\n	            crash_image,\n            crash_image,\n	-p singlefs-harness --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage	units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage
 实审 B3c-2 第 2 件：第二截的写表不带挂载之后那一次发布的写（它合法复用的历史槽解释不了，挂载之后的池上假红）	crates/singlefs-harness/src/crash_injection.rs	                .chain(writes_of_the_publish_after_the_mount)\n	                .chain(writes_of_the_publish_after_the_mount.iter().take(0))\n	-p singlefs-harness --test crash_injection_record_checker_sees_every_publish_across_a_second_crash -- units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage	units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage
 实审 A4c C545 那一格（D3 已定项 8 第 2 条 / 已定项 10 ②）：用户数据的候选落点不排除这次挂载开过的聚簇段（段外没有成对空槽时第 7 个单元落进段里、写做成）	crates/singlefs-core/src/allocator.rs	            if !inside_cluster_segment\n                && self.is_free(SlotNumber(candidate))	            let _ = inside_cluster_segment;\n            if self.is_free(SlotNumber(candidate))	-p singlefs-core --lib -- allocator::tests::user_data_does_not_land	user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed
-实审 B3a-3d：树分裂 层 0：根只剩一个孩子时不降高（根降高那条流录之后是 L10 I1，不是 L10）	crates/singlefs-core/src/code_two_tree.rs	            PlanningNode::Internal { children, .. } if children.len() == 1 => {	            PlanningNode::Internal { children, .. } if false && children.len() == 1 => {	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+实审 B3a-3d：树分裂 层 0：根只剩一个孩子时不降高（根降高那条流录之后是 L10 I1，不是 L10）	crates/singlefs-core/src/code_two_tree.rs	            PlanningNode::Internal { children, .. } if children.len() == 1 => {	            PlanningNode::Internal { children, .. } if false && children.len() == 1 => {	-p singlefs-checker --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
 E158 root_choice_repair 第 4 次跑 M14：标记前后的读只数读成的读、不数读坏的（三次读坏两次读成该数 5 数 2）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        marker.read_calls += 1;\n        if marker.reads_before_the_first_wait.is_some()	        if result.is_ok() {\n            marker.read_calls += 1;\n        }\n        if marker.reads_before_the_first_wait.is_some()	-p singlefs-harness --bin e158_root_choice_repair	reads_before_and_after_the_first_wait_count_failed_reads_too
 E158 root_choice_repair 第 4 次跑 M16：Q0 去向把「s_h 里已不是 K_h」一律判成环在且被抛弃（根槽被盖那一格该出 root_slot_overwritten 出 in_ring_and_abandoned）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    if slot_holds_the_hidden_root && hidden_abandoned {\n        return HiddenRootDestination::InRingAndAbandoned;	    if (slot_holds_the_hidden_root && hidden_abandoned) || at_the_hidden_slot.is_some_and(|key| key != position.key) {\n        return HiddenRootDestination::InRingAndAbandoned;	-p singlefs-harness --bin e158_root_choice_repair	the_hidden_root_destination_tells_an_overwritten_slot_from_an_untouched_one
 E158 root_choice_repair 第 4 次跑 M18：Q5-同成 把臂交回 Err 的格也算进去（手写三格该收 2 格收 3 格）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    if !(same_start && both_ok) {\n        return None;\n    }	    if !same_start {\n        return None;\n    }	-p singlefs-harness --bin e158_root_choice_repair	both_mounted_extra_reads_leave_out_the_refused_cells
diff --git a/crates/singlefs-checker/Cargo.toml b/crates/singlefs-checker/Cargo.toml
index c9f2e64c..ae216fbc 100644
--- a/crates/singlefs-checker/Cargo.toml
+++ b/crates/singlefs-checker/Cargo.toml
@@ -3,7 +3,13 @@ name = "singlefs-checker"
 version = "0.1.0"
 publish = false
 edition = "2021"
-description = "checker：与实现只共享格式常量模块（D13 已定项 5），解析、校验、遍历各写一份"
+description = "checker：库是池级 checker，与实现只共享格式常量模块（D13 已定项 5），解析、校验、遍历各写一份；tests/ 是 checker 档——崩溃枚举用例与注入战役，默认只在提交时跑（D13 已定项 15）"
 
 [dependencies]
 singlefs-format = { path = "../singlefs-format" }
+
+# checker 档的测试住在这个包的 tests/ 里（D13 已定项 15）：崩溃枚举用例要驱动实现造镜像、录写流，再交给上面的库判。
+# 只有测试编进这两个依赖，库本身仍只依赖 singlefs-format（D13 已定项 5，门禁 94 号只读 [dependencies] 与 [build-dependencies]）。
+[dev-dependencies]
+singlefs-core = { path = "../singlefs-core" }
+singlefs-harness = { path = "../singlefs-harness" }
diff --git a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs b/crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
rename to crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs
index cb3bbf99..4ba4b001 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
+++ b/crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs
@@ -2,6 +2,7 @@
 //! 步 6 的恢复与 oracle、池级 checker（一元判决）、记录核对器（二元判决）。oracle 那一半的计数与 E142（第一个事务的干跑）
 //! 第八次跑产物的 `name=layer0` / `name=journal_effect` 行逐字对；再加一组靶向的阳性对照。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{build_pool, file_content, geometry};
diff --git a/crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs b/crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs
similarity index 99%
rename from crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs
rename to crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs
index 765d2de8..c7189888 100644
--- a/crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs
+++ b/crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs
@@ -16,6 +16,7 @@
 //!
 //! σ 那一段全部 2^18 个状态的记录核对器判定另有一条标了 ignore 的全量（崩溃枚举，提交时由崩溃验证员跑）。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{build_pool, geometry, parameters, publish_overwrite_in_process, BuiltPool};
diff --git a/crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs b/crates/singlefs-checker/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs
rename to crates/singlefs-checker/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs
index f3defbe1..af9ff02f 100644
--- a/crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs
+++ b/crates/singlefs-checker/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs
@@ -4,6 +4,7 @@
 //! 恢复之后与挂载之后两份镜像的池级 checker 都判；挂载那一遍照枚举用的写表叠，撕裂那一态的镜像也交给挂载。
 //! 名字里不带 layer0：它不是层 0 的全量流，是一条历史上的一小段，平时跑得起。
 
+#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
 mod common_admission;
 
 use common_admission::{
diff --git a/crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs b/crates/singlefs-checker/tests/second_transaction_parallel_line_one_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_parallel_line_one_layer0.rs
index be941d1c..eb1dabc4 100644
--- a/crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_parallel_line_one_layer0.rs
@@ -21,6 +21,7 @@
 //! 两段占 5368709118），登记成崩溃枚举用例，门禁 54 号 --full 在 release 下跑，带断点续跑、可双机分片（与第一、第二条流同一个入口）。
 //! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
diff --git a/crates/singlefs-harness/tests/second_transaction_parallel_line_three_spill_over_layer0.rs b/crates/singlefs-checker/tests/second_transaction_parallel_line_three_spill_over_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_parallel_line_three_spill_over_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_parallel_line_three_spill_over_layer0.rs
index 5679bf19..010ad700 100644
--- a/crates/singlefs-harness/tests/second_transaction_parallel_line_three_spill_over_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_parallel_line_three_spill_over_layer0.rs
@@ -20,6 +20,7 @@
 //! 末条落了而第一条没落是断号）。多跑的一步只有恢复本身（这条流上没有重开、挂载）。每个状态照常跑两遍恢复 + 多版本 oracle、
 //! 池级 checker（I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志） 在内）、记录核对器。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{
diff --git a/crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs b/crates/singlefs-checker/tests/second_transaction_position_addressed_trees_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_position_addressed_trees_layer0.rs
index b3d539dd..8a833059 100644
--- a/crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_position_addressed_trees_layer0.rs
@@ -22,7 +22,9 @@
 //!
 //! 按派活的约束这一份只要求编译通过，一条都没跑；快档与全量交 crash-verifier 跑。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
+#[path = "../../singlefs-harness/tests/common_tree_split/mod.rs"]
 mod common_tree_split;
 
 use common::{
diff --git a/crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs b/crates/singlefs-checker/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
index 75680d99..321423b2 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_step_three_acquisition_barrier_layer0.rs
@@ -14,6 +14,7 @@
 //! 全量（门禁 54 号 `--full`）已经罩着；这一条不多罩崩溃状态，多的是在平时的 `cargo test` 里展开它们、按 I-7.7 判——
 //! 第二条流的快用例按甲二展开，写行那一段（18 个单元写）只取全不落或全落。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{
diff --git a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs b/crates/singlefs-checker/tests/second_transaction_step_three_formatted_pool_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_step_three_formatted_pool_layer0.rs
index 3f1a2690..cccae7a4 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_step_three_formatted_pool_layer0.rs
@@ -3,6 +3,7 @@
 //! 取崩溃状态，快的那条（26 写的那一段不展开）跑恢复 + 多版本 oracle、池级 checker、记录核对器。
 //! 这条流的基线、写表（带内容）、段序列与 mkfs 同一个进程里跑第一个事务那条流逐项相同（用例钉住），全量枚举就是第一个事务那条流的全量，不另跑。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use common::{
diff --git a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs b/crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs
index 24684804..b0ea58a6 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs
@@ -6,6 +6,7 @@
 //! 全量那条标 ignored，54 号门禁在 release 下跑它（带断点续跑）。再加四组靶向的阳性对照。
 //! 另有两条只展开小段的流：基镜像里预置一条残留记录（步 0 预想的细节第三条的正例），与到 E 之后再复用两次、改坏 tail（步 6 必红「陈旧 tail + 已复用的块」）。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
 use std::num::{NonZeroU64, NonZeroUsize};
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs b/crates/singlefs-checker/tests/second_transaction_supplement_three_crash_injection.rs
similarity index 100%
rename from crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs
rename to crates/singlefs-checker/tests/second_transaction_supplement_three_crash_injection.rs
diff --git a/crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs b/crates/singlefs-checker/tests/second_transaction_supplement_two_tree_split_layer0.rs
similarity index 99%
rename from crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs
rename to crates/singlefs-checker/tests/second_transaction_supplement_two_tree_split_layer0.rs
index 3569d9ad..38b708a4 100644
--- a/crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split_layer0.rs
+++ b/crates/singlefs-checker/tests/second_transaction_supplement_two_tree_split_layer0.rs
@@ -20,7 +20,9 @@
 //! 每次发布还带着分配记录树的五个节点（D8（核心索引结构） 已定项 14：两盘各叶 61、两盘各第 1 层、根），被录那一次写 9–13 个单元。
 //! 多跑的一步只有恢复本身（这几条流上没有重开、挂载）。与已有流的基线镜像、写表、段序列都不相同，各自全量枚举。
 
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
+#[path = "../../singlefs-harness/tests/common_tree_split/mod.rs"]
 mod common_tree_split;
 
 use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS};
diff --git a/crates/singlefs-harness/Cargo.toml b/crates/singlefs-harness/Cargo.toml
index 2bdcbf09..6570c2df 100644
--- a/crates/singlefs-harness/Cargo.toml
+++ b/crates/singlefs-harness/Cargo.toml
@@ -3,7 +3,7 @@ name = "singlefs-harness"
 version = "0.1.0"
 publish = false
 edition = "2021"
-description = "验证装置：包在块设备外面的录制器，把每个写请求与屏障记成一条流，给崩溃点重放用"
+description = "harness 档：单元测试与集成测试那一层，改了代码随时跑；连同录制器、理想模型、崩溃态枚举引擎等脚手架库。放量用例不住这里（D13 已定项 15，.claude/rules/verification.md）"
 
 [dependencies]
 singlefs-checker = { path = "../singlefs-checker" }
diff --git a/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs b/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
index 50c8323b..a61ecb1e 100644
--- a/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
@@ -5061,7 +5061,7 @@ const SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET: u64 = 110;
 const SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS: usize = 30;
 const SEVENTH_BATCH_FAULT_REPRODUCTION_FAULTS: usize = 6;
 /// 崩溃注入快档那一段：种子基 + 16、每段 24 步、每段抽 4 个崩溃状态（快档规模，
-/// `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` 的 `FAST_TIER_*`）。
+/// `crates/singlefs-checker/tests/second_transaction_supplement_three_crash_injection.rs` 的 `FAST_TIER_*`）。
 const SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 16;
 const SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
 const SEVENTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;
@@ -17511,7 +17511,7 @@ mod fourth_run_segment_two {
     const RANDOM_HISTORY_SEGMENTS_REGISTERED: usize = 30;
 
     /// 实八：崩溃注入种子基 + 2、24 步、4 个崩溃状态（r3 登记第 373–391 行那张表「实八」一行；
-    /// 步数与抽法同崩溃注入快档 `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` 第 39、41 行）。
+    /// 步数与抽法同崩溃注入快档 `crates/singlefs-checker/tests/second_transaction_supplement_three_crash_injection.rs` 第 39、41 行）。
     const EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 2;
     const EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
     const EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;
@@ -17755,7 +17755,7 @@ mod fourth_run_segment_two {
     const RANDOM_HISTORY_TEST_SOURCE: &str =
         "crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs";
     const CRASH_INJECTION_TEST_SOURCE: &str =
-        "crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs";
+        "crates/singlefs-checker/tests/second_transaction_supplement_three_crash_injection.rs";
 
     fn investigator_report_text() -> Result<String, String> {
         let path = env::var(INVESTIGATOR_REPORT_ENVIRONMENT_VARIABLE)
diff --git a/crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs b/crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs
index 6a1d041b..63fc0ec1 100644
--- a/crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs
+++ b/crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs
@@ -9,7 +9,7 @@
 //!
 //! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 已有的文件。
 //! 第一条流经 `#[path]` 引 `tests/common/mod.rs` 的 `build_pool`；第二条流的脚本照抄
-//! `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:203-474`（到 E 再正常卸载那一支）。
+//! `crates/singlefs-checker/tests/second_transaction_step_zero_layer0.rs:203-474`（到 E 再正常卸载那一支）。
 //! 决定几何与门槛的数都写成本地常量（值抄自登记第十一节 S1 与第七节），`main` 开头逐条与 crates 的同名常量回比（S1）。
 //! 枚举计划与撕裂镜像表是本文件自己的一份，与 crates 私有的那一份逐状态对拍（S3）。
 //!
diff --git a/crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs b/crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs
index 6c81ceab..c2db452f 100644
--- a/crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs
+++ b/crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs
@@ -379,6 +379,7 @@ fn starting_over_reruns_every_slice_even_with_a_valid_progress_file() {
 
 /// 判红的那一趟删进度文件（R4、R5）：观察者在第 12 个状态上 panic，这一趟没跑完；进度文件不留，下一趟从头跑。
 /// panic 那一片的片行没写（片行在观察者看完之后才写），删文件由 panic 展开时做。
+// crash-case-check:not-a-crash-case 测的是断点续跑装置在判红时删进度文件，枚举的是几步的合成流、单跑几秒；不是要登记的放量用例
 #[test]
 fn a_run_that_goes_red_leaves_no_progress_file_behind() {
     let stream = first_stream("resume-red-run");
diff --git a/research/scripts/admission.py b/research/scripts/admission.py
old mode 100755
new mode 100644
index b5ccb81b..181ded08
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -65,6 +65,9 @@
   ③ 跑的场合（重型测试前缀、内存包装）归 .claude/hooks/heavy-test-guard.sh 与 run-with-memory-cap.sh，这里不管。
 
 子命令（<根> 是被判仓的根）：
+  stage-fingerprint <根> <阶段文件名>    这一道登记输入在被判那棵树上的指纹：打「<sha256> <文件数>」
+  stage-marker-check <根> <阶段文件名>   这一道这批输入有没有作数的全绿标记：退 0 有（第一行 ok <路径> <时刻>），退 1 没有或过了 SINGLEFS_REUSE_HOURS
+  stage-marker-write <根> <阶段文件名>   判绿之后写这一格标记（55、57、59 号自己调），打路径；gate-reuse 先看它再看暂存树
   gate-reuse <根> <阶段文件名>   门禁的复用判定：退 0 要跑，退 10 可跳过（stage-must-run.sh 翻成它原来的 1）；
                                  模块自己出错一律退 0（按要跑处理），不许出错就跳过
   experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头）；
@@ -220,6 +223,7 @@ LAYER0_SHARD_VARIABLE = "SINGLEFS_LAYER0_SHARD"
 CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
 COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
 CRASH_CASE_MARKER_PREFIX = "singlefs-crash-case-green."
+STAGE_MARKER_PREFIX = "singlefs-stage-green."          # 55、57、59 号判绿之后按输入指纹写的全绿标记（用户 2026-09-27 定：照 54 号写标记、只跑变了的）
 LAYER0_PARALLEL_FINISHED_PREFIX = "LAYER0_PARALLEL_FINISHED "
 PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
 MARKER_DIFFERENCES_LISTED_AT_MOST = 20
@@ -1117,6 +1121,17 @@ def gate_reuse(root, stage):
     return_code, _output = git_output(root, "rev-parse", "--is-inside-work-tree")
     if return_code != 0:
         return EXIT_GATE_MUST_RUN, f"判不出来：{root} 不是 git 工作树，按要跑处理"
+    # 先看这一道在被判那棵树上的全绿标记（55、57、59 号判绿时自己写，按登记输入的指纹分格，住 git common-dir）：
+    # 在且没过复用上限就可跳过——崩溃验证员在工作区跑过的那一趟，整轮门禁不用再跑一遍。指纹算不出、没登记这一行的照旧往下按暂存树判。
+    if not break_is_set("stage-marker-ignored"):
+        try:
+            fingerprint, file_count, _paths = stage_fingerprint(root, stage)
+        except (RegistrationError, InputManifestError):
+            fingerprint = None
+        if fingerprint is not None:
+            valid, lines = check_stage_marker(root, stage, fingerprint)
+            if valid:
+                return EXIT_GATE_MAY_SKIP, f"这一道这批输入（指纹 {fingerprint[:16]}…，{file_count} 个文件）有全绿标记：{lines[0]}"
     staged_tree = os.environ.get("SINGLEFS_STAGED_TREE", "")
     if not staged_tree:
         return EXIT_GATE_MUST_RUN, "这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用"
@@ -2190,6 +2205,122 @@ def crash_case_judging_digest_line():
 
 # ── 命令行 ────────────────────────────────────────────────────────────────────
 
+def stage_fingerprint(root, stage):
+    """一道门禁阶段的输入指纹：登记路径下逐文件清单 + 登记表 + 阶段脚本自己 + 登记行 + 工具链 + 构建环境。返回 (sha256, 文件数, 路径)。
+    与 gate_reuse 比对的路径同一份（少一条输入就永远不重跑的那个理由同样成立）。"""
+    stage_rows = rows_of_key(read_registration_rows(root), stage)
+    if not any(row.input_paths for row in stage_rows):
+        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {stage} 这一行")
+    input_paths = [input_path for row in stage_rows for input_path in row.input_paths]
+    input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]
+    registration_text = "".join(row.fingerprint_text() for row in stage_rows).encode("utf-8", "surrogateescape")
+    named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
+    _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
+    return fingerprint, file_count, input_paths
+
+
+def stage_marker_path(root, stage, fingerprint):
+    """这一道这批输入那一格全绿标记：git common-dir 里「前缀 + 阶段文件名 + . + 指纹」；取不到 common-dir 返回 None。"""
+    common_directory = git_common_directory(root)
+    if common_directory is None:
+        return None
+    return os.path.join(common_directory, f"{STAGE_MARKER_PREFIX}{stage}.{fingerprint}")
+
+
+def check_stage_marker(root, stage, fingerprint):
+    """这一道这批输入的全绿标记作不作数：返回 (作数?, 说明行)。作数时第一行是「ok <路径> <跑完的时刻>」；
+    过了复用上限（SINGLEFS_REUSE_HOURS，默认 24 小时；内容没变不代表环境没变）不作数。"""
+    marker_path = stage_marker_path(root, stage, fingerprint)
+    if marker_path is None:
+        return False, [f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处读"]
+    marker = read_crash_case_marker(marker_path)
+    if marker is None:
+        return False, [f"这批输入（指纹 {fingerprint[:16]}…）没有全绿标记：{marker_path}"]
+    finished_text = marker["fields"].get("finished_utc", "")
+    try:
+        finished_epoch = int(time.mktime(time.strptime(finished_text, "%Y-%m-%dT%H:%M:%SZ"))) - (time.mktime(time.localtime(0)) - time.mktime(time.gmtime(0)))
+    except (ValueError, OverflowError):
+        return False, [f"全绿标记 {marker_path} 的 finished_utc 读不出（{finished_text!r}），不作数"]
+    hours_text = os.environ.get("SINGLEFS_REUSE_HOURS", "") or "24"
+    try:
+        reuse_limit_hours = int(hours_text)
+    except ValueError:
+        reuse_limit_hours = 0
+    age_hours = max(0, int(time.time()) - int(finished_epoch)) // 3600
+    if age_hours >= reuse_limit_hours:
+        return False, [f"全绿标记 {marker_path} 跑完于 {finished_text}、{age_hours} 小时前，到了复用上限 {hours_text} 小时，不作数"]
+    return True, [f"ok {marker_path} {finished_text}"]
+
+
+def write_stage_marker(root, stage, fingerprint, file_count):
+    """判绿之后写这一道这批输入的全绿标记：同目录排他建临时文件、写完改名换上。返回标记路径；取不到 common-dir、写不了抛 InputManifestError。"""
+    marker_path = stage_marker_path(root, stage, fingerprint)
+    if marker_path is None:
+        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处写")
+    text = ("# 门禁阶段的全绿标记：55、57、59 号判绿之后经 research/scripts/admission.py stage-marker-write 写，按阶段与它登记输入的指纹分格；"
+            "gate-reuse 按这一格判复用。不进工作树，别手改。\n"
+            f"stage={stage}\ninput_hash={fingerprint}\ninput_file_count={file_count}\n"
+            f"finished_utc={time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\njudged_root={os.path.abspath(root)}\n"
+            f"heavy_prefix={os.environ.get('SINGLEFS_HEAVY_TESTS', '')}\n")
+    try:
+        handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(marker_path), prefix=os.path.basename(marker_path) + ".partial.")
+        try:
+            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
+                handle.write(text)
+            os.replace(temporary_path, marker_path)
+        except OSError:
+            os.unlink(temporary_path)
+            raise
+    except OSError as error:
+        raise InputManifestError(f"写不了 {marker_path}：{error}") from error
+    return marker_path
+
+
+def command_stage_fingerprint(arguments):
+    if len(arguments) != 2:
+        print("  ✗ 用法：admission.py stage-fingerprint <项目根> <阶段文件名>")
+        print("     → 怎么办：阶段文件名是 .claude/gate.d/ 下那一份的文件名，要在 stage-inputs.tsv 里登记过")
+        return EXIT_REGISTRATION_ERROR
+    try:
+        fingerprint, file_count, _paths = stage_fingerprint(*arguments)
+    except (RegistrationError, InputManifestError) as error:
+        print(f"算不出 {arguments[1]} 的输入指纹：{error}")
+        return EXIT_REGISTRATION_ERROR
+    print(f"{fingerprint} {file_count}")
+    return 0
+
+
+def command_stage_marker_check(arguments):
+    if len(arguments) != 2:
+        print("  ✗ 用法：admission.py stage-marker-check <项目根> <阶段文件名>")
+        print("     → 怎么办：退 0 有作数的全绿标记（第一行 ok <路径> <时刻>），退 1 没有或不作数")
+        return EXIT_REGISTRATION_ERROR
+    try:
+        fingerprint, _file_count, _paths = stage_fingerprint(*arguments)
+    except (RegistrationError, InputManifestError) as error:
+        print(f"判不了 {arguments[1]} 的全绿标记：{error}")
+        return EXIT_REGISTRATION_ERROR
+    valid, lines = check_stage_marker(arguments[0], arguments[1], fingerprint)
+    for line in lines:
+        print(line)
+    return 0 if valid else 1
+
+
+def command_stage_marker_write(arguments):
+    if len(arguments) != 2:
+        print("  ✗ 用法：admission.py stage-marker-write <项目根> <阶段文件名>")
+        print("     → 怎么办：只在这一道判绿之后调，它按此刻的输入指纹写标记、打路径")
+        return EXIT_REGISTRATION_ERROR
+    try:
+        fingerprint, file_count, _paths = stage_fingerprint(*arguments)
+        marker_path = write_stage_marker(arguments[0], arguments[1], fingerprint, file_count)
+    except (RegistrationError, InputManifestError) as error:
+        print(f"{arguments[1]} 的全绿标记没写成：{error}")
+        return 1
+    print(marker_path)
+    return 0
+
+
 def command_gate_reuse(arguments):
     if len(arguments) < 2 or not arguments[0] or not arguments[1]:
         print("  ✗ 用法：stage-must-run.sh <项目根> <阶段文件名>")
@@ -2618,6 +2749,44 @@ def point_replay_row_at(work, product_name):
     write_text(os.path.join(work, REPLAY_SCRIPT), f"TABLE=$(cat <<'TSV'\nE900|@driver_e900||{product_name}|exact\nTSV\n)\n")
 
 
+def run_stage_marker_cells(selftest, module):
+    """55、57、59 号的全绿标记：没有时判不了、写了之后 gate-reuse 不看暂存树也可跳过、弄坏开关下转红、过上限不作数、输入改了就没有它的格。"""
+    work = tempfile.mkdtemp(prefix="admission-stage-marker-")
+    try:
+        build_selftest_repository(work)
+        def call(*arguments, environment_changes=None):
+            return run_quietly([sys.executable, module, *arguments], environment_changes)
+        exit_code, output, _messages = call("stage-fingerprint", work, "59-demo.sh")
+        selftest.expect("stage-fingerprint 打「<64 位指纹> <文件数>」", exit_code == 0 and re.fullmatch(r"[0-9a-f]{64} [0-9]+", output.strip()) is not None,
+                        f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("stage-marker-check", work, "59-demo.sh")
+        selftest.expect("没有全绿标记时 stage-marker-check 退 1、说没有", exit_code == 1 and "没有全绿标记" in output, f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
+        selftest.expect("没有标记、也不是 gate-staged.sh 起的：gate-reuse 判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("stage-marker-write", work, "59-demo.sh")
+        marker_path = output.strip()
+        selftest.expect("stage-marker-write 把标记写进 git common-dir、打它的路径",
+                        exit_code == 0 and os.path.isfile(marker_path) and os.path.basename(marker_path).startswith(STAGE_MARKER_PREFIX + "59-demo.sh."),
+                        f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("stage-marker-check", work, "59-demo.sh")
+        selftest.expect("写过之后 stage-marker-check 退 0、第一行 ok", exit_code == 0 and output.startswith("ok "), f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
+        selftest.expect("有全绿标记时 gate-reuse 不看暂存树也判可跳过（退 10）", exit_code == EXIT_GATE_MAY_SKIP and "全绿标记" in output,
+                        f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh", environment_changes={BREAK_VARIABLE: "stage-marker-ignored"})
+        selftest.expect("弄坏开关 stage-marker-ignored 下「有标记可跳过」那一格转红（判要跑）", exit_code == EXIT_GATE_MUST_RUN,
+                        f"弄坏之后仍退 {exit_code}：这一格分不出标记看没看")
+        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh", environment_changes={"SINGLEFS_REUSE_HOURS": "0"})
+        selftest.expect("复用上限 0 小时：标记再新也不作数、判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
+        write_text(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn one() -> u32 { 2 }\n")
+        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
+        selftest.expect("登记的输入改了一个字节就没有它那一格标记、判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
+        exit_code, output, _messages = call("stage-marker-check", work, "not-registered.sh")
+        selftest.expect("没登记的阶段 stage-marker-check 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}，stdout「{output.strip()}」")
+    finally:
+        shutil.rmtree(work, ignore_errors=True)
+
+
 def run_selftest():
     selftest = Selftest()
     work = tempfile.mkdtemp(prefix="admission-selftest-")
@@ -2747,6 +2916,8 @@ def run_selftest():
                                                    {BREAK_VARIABLE: "raise-in-gate-reuse", "SINGLEFS_STAGED_TREE": "0" * 40})
         selftest.expect("门禁复用判定出异常时 stage-must-run.sh 判要跑", exit_code == 0 and "按要跑处理" in output,
                         f"退 {exit_code}，stdout「{output.strip()}」")
+        # ⑩b 55、57、59 号的全绿标记（用户 2026-09-27 定：照 54 号写标记、只跑变了的）
+        run_stage_marker_cells(selftest, module)
 
         # ⑪ 门禁行的第三列：前提（command= / readwrite= / probe=）、环境进复用判定（environment=，拿假 herd7 当桩）、中文路径
         run_gate_precondition_cells(selftest, module)
@@ -2782,7 +2953,7 @@ def run_selftest():
           "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、"
           "shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch、target-own-files-only、include-alias-subtracts、"
           "runner-arguments-from-configuration-directory-only、digest-skips-other-statements、digest-allows-non-name-dispatch、digest-allows-imported-names、"
-          "judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line "
+          "judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line、stage-marker-ignored "
           "下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
     return 0
 
@@ -4018,8 +4189,8 @@ def stream_log(prefix, worker_threads=32, resumed_slices=0, freshly_run_slices=6
 def run_layer0_stage_cells(selftest, module):
     """门禁 54 号这一份的流程：拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下，名字照旧是 54-layer0-replay.sh），
     假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量与线程数（由 crash-case-command 交出）、判绿写标记（线程分记配的与起的）、
-    快档核标记、第二趟全复用、只重跑输入变了的那一条、--start-over、显式设线程数、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、
-    cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、快档不带 SINGLEFS_GATE_FULL=1 时只改登记表判红、只改 54 号与只改准入模块判法之外的部分
+    快档跑一遍 checker 包并核标记、缺一格报本次未跑不判红、第二趟全复用、只重跑输入变了的那一条、--start-over、显式设线程数、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、
+    cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、快档不带 SINGLEFS_GATE_FULL=1 时只改登记表报本次未跑、只改 54 号与只改准入模块判法之外的部分
     照样核标记而判绿（54 号不进指纹）、改准入模块的判法判红（都不退 77）。"""
     repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
     real_stage = os.path.join(repository_root, ".claude/gate.d/54-layer0-replay.sh")
@@ -4129,9 +4300,10 @@ def run_layer0_stage_cells(selftest, module):
                         and not any(line.startswith("worker_threads=") for line in (stream_a_marker + case_c_marker).split("\n")),
                         f"stream-a 那一格：{stream_a_marker[:600]}；case-c 那一格：{case_c_marker[:400]}")
         exit_code, output, calls = stage()
-        selftest.expect("54 号快档：两条流跑不标 ignored 的用例，三条用例的标记都作数，判绿",
+        selftest.expect("54 号快档：跑一遍 checker 包不标 ignored 的用例（一次 cargo test -p singlefs-checker），三条用例的标记都作数，判绿、不报本次未跑",
                         exit_code == 0 and all(key in output for key, _t, _f, _c in LAYER0_STAGE_CASES) and not full_runs(calls)
-                        and len(calls) == 2, f"退 {exit_code}，cargo 调了 {calls}，输出尾部：{output.strip()[-600:]}")
+                        and len(calls) == 1 and calls[0][0] == "" and "本次未跑" not in output,
+                        f"退 {exit_code}，cargo 调了 {calls}，输出尾部：{output.strip()[-600:]}")
         exit_code, output, calls = stage("--full")
         selftest.expect("54 号 --full 第二趟：输入没变，三条全复用、一条都不跑", exit_code == 0 and not full_runs(calls) and output.count(" 复用：") == 3,
                         f"退 {exit_code}，跑了 {full_runs(calls)}，输出尾部：{output.strip()[-600:]}")
@@ -4143,8 +4315,8 @@ def run_layer0_stage_cells(selftest, module):
         for name in stream_b_marker:
             os.remove(os.path.join(common_directory, name))
         exit_code, output, calls = stage()
-        selftest.expect("54 号快档：删掉 stream-b 那一格标记就判红，点名 crash-case:stream-b，出路是 --full",
-                        exit_code == 1 and "✗" in output and "crash-case:stream-b" in output and "--full" in output,
+        selftest.expect("54 号快档：删掉 stream-b 那一格标记，快档照样绿、退 0，那一条报「本次未跑」并点名 crash-case:stream-b，出路是 --full（D13 已定项 15：全量默认不在提交时跑）",
+                        exit_code == 0 and "✗" not in output and "本次未跑" in output and "crash-case:stream-b" in output and "--full" in output,
                         f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
         set_logs(stream_b=stream_log("LAYER0B", worker_threads=1, resumed_slices=63, freshly_run_slices=1))
         exit_code, output, calls = stage("--full", "--start-over")
@@ -4250,12 +4422,12 @@ def run_layer0_stage_cells(selftest, module):
             exit_code, output = quick_tier_without_forcing()
             write_text(path, original_text)
             if named_cases:
-                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，判红并点名 {'、'.join(named_cases)}，不退 77",
-                                exit_code == 1 and "没有作数的全绿标记" in output and all(key in output for key in named_cases),
+                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，退 0 而报「本次未跑」并点名 {'、'.join(named_cases)}，不退 77",
+                                exit_code == 0 and "没有作数的全绿标记" in output and "本次未跑" in output and all(key in output for key in named_cases),
                                 f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
             else:
-                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记（不退 77），三条标记都作数，判绿",
-                                exit_code == 0 and all(key in output for key in every_case) and "没有作数的全绿标记" not in output,
+                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记（不退 77），三条标记都作数，判绿、不报本次未跑",
+                                exit_code == 0 and all(key in output for key in every_case) and "没有作数的全绿标记" not in output and "本次未跑" not in output,
                                 f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
     except (OSError, shutil.Error) as error:
         selftest.expect("54 号这一份拷得进临时仓、跑得起来", False, f"{error}（真仓的 54 号在 {real_stage}）")
@@ -4421,6 +4593,9 @@ printf '%s  %s\n' "$toolchain_versions_hash" "<工具链：${toolchain_versions/
 
 COMMANDS = {
     "gate-reuse": command_gate_reuse,
+    "stage-fingerprint": command_stage_fingerprint,
+    "stage-marker-check": command_stage_marker_check,
+    "stage-marker-write": command_stage_marker_write,
     "experiment": command_experiment,
     "paths": command_paths,
     "manifest": command_manifest,
diff --git a/research/scripts/crash-case-check.py b/research/scripts/crash-case-check.py
old mode 100755
new mode 100644
index afbbb3b3..24e489c7
--- a/research/scripts/crash-case-check.py
+++ b/research/scripts/crash-case-check.py
@@ -1,7 +1,7 @@
 #!/usr/bin/env python3
 # admission: always 判的是此刻 crates/*/tests/ 的源码与登记表，几秒跑完；每次改测试都现判
 # run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/stage-inputs.tsv
-"""崩溃枚举用例必须标 #[ignore] 并登记成 crash-case：直接调全量崩溃枚举函数的测试函数，漏一样就判红。
+"""崩溃枚举用例必须住在 checker 包、标 #[ignore] 的要登记成 crash-case：直接调全量崩溃枚举函数的测试函数，写在别的包里、或标了 ignore 却没登记，判红。
 
 用法：
     crash-case-check.py [仓根]
@@ -10,10 +10,12 @@
 判法：
   ① 扫 crates/*/tests/*.rs（不进子目录：子目录是共用模块，不是测试目标）里每个带 #[test] 的函数，函数体里直接调了
      enumerate_layer0 一族（名字以 enumerate_layer0 起头的函数），而不是快档（名字里带 quick_tier）的，算崩溃枚举用例；
-  ② 它要带 #[ignore]（不带 --ignored 的 cargo test 不跑它）；
-  ③ .claude/gate.d/stage-inputs.tsv 里要有一行键是 crash-case:<名>、第三列 test=<包>:<测试目标>:<这个函数名>，测试目标是文件名去掉 .rs。
-  测试目标名字里带 layer0 的整个二进制归门禁 54 号的层 0 流（重型测试闸按名字认它、54 号整个二进制跑），②③ 都不判——标了 #[ignore] 反而让 54 号跑不到它。
-  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：枚举的是几步的合成流、单跑几秒的，写明为什么。
+  ② 它要住在 crates/singlefs-checker/tests/（checker 档，D13 已定项 15，.claude/rules/verification.md「崩溃枚举用例住哪、怎么登记」）：
+     写在别的包（harness 这类日常测试包）里判红，那会让 cargo test 那个包的人跑到全量枚举；
+  ③ checker 包里标了 #[ignore] 的（全量那条），.claude/gate.d/stage-inputs.tsv 里要有一行键是 crash-case:<名>、第三列
+     test=singlefs-checker:<测试目标>:<这个函数名>（测试目标是文件名去掉 .rs）——标了 ignore 又没登记的谁都不跑；
+     不标 #[ignore] 的是 checker 档的快档（小流、几步的合成流），随 cargo test -p singlefs-checker 跑，不要求登记。
+  测试函数上面的注释里写了「crash-case-check:not-a-crash-case <理由>」（理由至少 4 个字）的不判：写明为什么不算用例。
 判不了的：经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（报告里写明是按直接调用认的）。
 退出码：0 都齐；1 有缺的（逐个列出）；2 用法错或读不了登记表。
 """
@@ -30,6 +32,7 @@ sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "
 from preflight import preflight  # noqa: E402
 BROKEN = os.environ.get("CRASH_CASE_CHECK_BREAK", "")
 
+CHECKER_PACKAGE = "singlefs-checker"
 ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)\s*(?:::<[^>]*>)?\s*\(")
 TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")
 CRASH_CASE_ROW = re.compile(r"^crash-case:[^\t]+\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
@@ -91,12 +94,11 @@ def problems_in(root):
     tests = crash_enumeration_tests(root)
     problems = []
     for package_directory, target, function, ignored, where in tests:
-        if "layer0" in target and BROKEN != "layer0-checked":
+        if package_directory != CHECKER_PACKAGE and BROKEN != "package-unchecked":
+            problems.append(f"{where} {function}：直接调全量崩溃枚举，却写在 {package_directory} 包里（崩溃枚举用例要住在 crates/{CHECKER_PACKAGE}/tests/）")
             continue
-        if not ignored:
-            problems.append(f"{where} {function}：直接调全量崩溃枚举，却没标 #[ignore]")
-        if (target, function) not in registered_targets and BROKEN != "rows-ignored":
-            problems.append(f"{where} {function}：没有登记成 crash-case（.claude/gate.d/stage-inputs.tsv 里没有 test=<包>:{target}:{function}）")
+        if ignored and (target, function) not in registered_targets and BROKEN != "rows-ignored":
+            problems.append(f"{where} {function}：标了 #[ignore] 却没有登记成 crash-case（.claude/gate.d/stage-inputs.tsv 里没有 test={CHECKER_PACKAGE}:{target}:{function}），谁都不跑它")
     return tests, problems
 
 
@@ -109,44 +111,50 @@ def run(root):
     for problem in problems:
         print(f"  ✗ {problem}")  # gate-lint:detail
     if problems:
-        print(f"  ✗ 崩溃枚举用例 {len(tests)} 条里有 {len(problems)} 处缺 #[ignore] 或缺登记（逐处列在上面）")  # gate-lint:summary
-        print("  → 怎么办：给那个测试函数加 #[ignore]，把 crash-case:<名> 那一行（第三列 test=<包>:<测试目标>:<函数名> 与 count-line 等）登记进 "
-              ".claude/gate.d/stage-inputs.tsv；层 0 流的快档用 quick_tier 那一族，不算崩溃枚举用例")
+        print(f"  ✗ 崩溃枚举用例 {len(tests)} 条里有 {len(problems)} 处写在 checker 包之外、或标了 ignore 没登记（逐处列在上面）")  # gate-lint:summary
+        print(f"  → 怎么办：把那个测试文件挪进 crates/{CHECKER_PACKAGE}/tests/（共用模块经 #[path = \"../../singlefs-harness/tests/common/mod.rs\"] mod common; 指回去），"
+              "全量那条标 #[ignore] 并把 crash-case:<名> 那一行（第三列 test=singlefs-checker:<测试目标>:<函数名> 与 count-line 等）登记进 "
+              ".claude/gate.d/stage-inputs.tsv；小流的快档不标 ignore、不用登记（.claude/rules/verification.md）")
         return 1
-    print(f"  ✓ 崩溃枚举用例都标了 #[ignore]、不在层 0 二进制里的都登记了（查了 {len(tests)} 条；按测试函数直接调 enumerate_layer0 一族认，经共用函数间接调的认不出）")
+    print(f"  ✓ 崩溃枚举用例都住在 {CHECKER_PACKAGE} 包里、标了 #[ignore] 的都登记了（查了 {len(tests)} 条；按测试函数直接调 enumerate_layer0 一族认，经共用函数间接调的认不出）")
     return 0
 
 
 def selftest():
     work = tempfile.mkdtemp(prefix="crash-case-check-selftest-")
     try:
-        tests_directory = os.path.join(work, "crates", "demo", "tests")
+        tests_directory = os.path.join(work, "crates", CHECKER_PACKAGE, "tests")
+        harness_tests_directory = os.path.join(work, "crates", "singlefs-harness", "tests")
         os.makedirs(tests_directory)
+        os.makedirs(harness_tests_directory)
         os.makedirs(os.path.join(work, ".claude", "gate.d"))
         open(os.path.join(tests_directory, "plain.rs"), "w").write(
             "#[test]\n#[ignore]\nfn registered_case() {\n    let report = enumerate_layer0_versions(&stream);\n}\n\n"
             "#[test]\nfn quick_case() {\n    enumerate_layer0_quick_tier_versions(&stream);\n}\n\n"
+            "#[test]\nfn small_stream_in_the_quick_tier() {\n    enumerate_layer0_selecting(&small, 1);\n}\n\n"
             "#[test]\nfn unrelated() {\n    assert_eq!(1, 1);\n}\n")
-        open(os.path.join(tests_directory, "other_layer0.rs"), "w").write("#[test]\nfn stream_full() {\n    enumerate_layer0(&s);\n}\n")
         open(os.path.join(tests_directory, "synthetic.rs"), "w").write(
             "// crash-case-check:not-a-crash-case 三步的合成流，单跑不到一秒\n#[test]\nfn tiny_stream() {\n    enumerate_layer0(&tiny);\n}\n")
+        open(os.path.join(harness_tests_directory, "daily.rs"), "w").write("#[test]\nfn daily_quick() {\n    enumerate_layer0_quick_tier_versions(&s);\n}\n")
         open(os.path.join(work, ".claude", "gate.d", "stage-inputs.tsv"), "w").write(
-            "# 样本\ncrash-case:demo\tcrates/\ttest=demo:plain:registered_case count-line=x\t#样本\n")
+            f"# 样本\ncrash-case:demo\tcrates/\ttest={CHECKER_PACKAGE}:plain:registered_case count-line=x\t#样本\n")
         failures = []
         tests, problems = problems_in(work)
         if len(tests) != 2 or problems:
-            failures.append(f"干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题（层 0 二进制里的不判），实际 {len(tests)} 条、{problems}")
-        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\nfn forgot_everything() {\n    enumerate_layer0_selecting(&s, 1);\n}\n")
+            failures.append(f"干净的样本应当认出 2 条（注明不是用例的那条不算）、0 处问题（checker 包里不标 ignore 的快档放行），实际 {len(tests)} 条、{problems}")
+        open(os.path.join(tests_directory, "plain.rs"), "a").write("\n#[test]\n#[ignore]\nfn forgot_to_register() {\n    enumerate_layer0_selecting(&s, 1);\n}\n")
+        open(os.path.join(harness_tests_directory, "daily.rs"), "a").write("\n#[test]\nfn full_in_harness() {\n    enumerate_layer0(&s);\n}\n")
         _, problems = problems_in(work)
-        if not any("forgot_everything" in problem and "#[ignore]" in problem for problem in problems) or \
-                not any("forgot_everything" in problem and "登记" in problem for problem in problems):
-            failures.append(f"没标 ignore、没登记的用例应当两处都报，实际 {problems}")
+        if not any("forgot_to_register" in problem and "没有登记" in problem for problem in problems):
+            failures.append(f"checker 包里标了 ignore 没登记的用例应当报缺登记，实际 {problems}")
+        if not any("full_in_harness" in problem and "singlefs-harness 包里" in problem for problem in problems):
+            failures.append(f"写在 harness 包里的崩溃枚举用例应当报住错了包，实际 {problems}")
         for failure in failures:
             print(f"  ✗ 自检：{failure}")  # gate-lint:detail
         if failures:
             print("  → 看 crash_enumeration_tests() / problems_in() 的判法；CRASH_CASE_CHECK_BREAK 设着的话这里本来就该红")
             return 1
-        print("  ✓ crash-case-check 自检通过：登记了且标了 ignore 的放行，快档、无关测试、注明不是用例的不算，层 0 二进制不判，没标没登记的两处都报（查了 4 种）")
+        print("  ✓ crash-case-check 自检通过：checker 包里登记了的放行、不标 ignore 的快档放行，注明不是用例的不算，标了 ignore 没登记的报缺登记，写在 harness 包里的报住错了包（查了 5 种）")
         return 0
     finally:
         import shutil
```
