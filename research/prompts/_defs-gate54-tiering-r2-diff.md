# 附录二：三处定义改动与门禁 54 号分档相关文件的工作区改动（`git diff HEAD -- <8 个路径>` 原样；基准 HEAD `3b60f098e97dc4c4f3ed9c6355422b607db1c34c`，生成于 2026-09-24T00:50:31Z）

## 一、diff（8 份文件相对 HEAD `3b60f098e97dc4c4f3ed9c6355422b607db1c34c` 的工作区改动，未裁剪）

命令：`git diff HEAD -- .claude/gate.d/54-layer0-replay.sh .claude/gate.d/stage-inputs.tsv .claude/main-agent.md .claude/agents/crash-verifier.md .claude/agents/experiment-runner.md .claude/hooks/agent-write-scope.tsv .claude/rules/implementation-workflow.md README.md`

```diff
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index a1d51cc..b74d381 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -20,8 +20,8 @@ omitClaudeMd: true
 
 ## 做什么
 
-1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号时，不起 54 号，等它结束。
-2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
+2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index 0bad931..e6305da 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -24,7 +24,7 @@ omitClaudeMd: true
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
-2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
+2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
@@ -36,7 +36,7 @@ omitClaudeMd: true
 
 ## 写范围
 
-- 上面列的 bin 源文件与 `research/e7-index-bench/Cargo.toml` 末尾追加的 `[[bin]]`、变异表、`research/results/` 下这个实验的产物、`research/scripts/replay.sh` 里这个实验的登记行、跑前登记与重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）的「修订」一段、实验页与它的索引行、`.claude/kb/experiments-history.md` 里这个实验的条目、只修锚点时点名的那张变异表、`/tmp/claude-1000/` 下的报告文件与草稿目录。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。
+- 上面列的 bin 源文件与 `research/e7-index-bench/Cargo.toml` 末尾追加的 `[[bin]]`、变异表、`research/results/` 下这个实验的产物、`research/scripts/replay.sh` 里这个实验的登记行、跑前登记与重跑登记（`research/prompts/e<号>-r<n>-prereg.md`）的「修订」一段、实验页与它的索引行、`.claude/kb/experiments-history.md` 里这个实验的条目、只修锚点时点名的那张变异表、入库装置的变异行（`crates/mutations.tsv` 末尾，只追加这个实验的行，不改别人的行）、`/tmp/claude-1000/` 下的报告文件与草稿目录。与 `.claude/hooks/agent-write-scope.tsv` 里你那几行一致。
 
 ## 产出
 
diff --git a/.claude/gate.d/54-layer0-replay.sh b/.claude/gate.d/54-layer0-replay.sh
index 0834ef1..539d945 100755
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -1,10 +1,30 @@
 #!/usr/bin/env bash
-# gate-stage: 层 0 崩溃点重放（两条流的全部崩溃状态在 release 下逐个跑恢复：第一个事务与 E142 产物逐字比对，里程碑「第二个事务」固定脚本到 E 与用例的闭式比对；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
+# gate-stage: 层 0 崩溃点重放（整轮门禁跑快档：两条流不标 ignored 的用例，再核对层 0 全量的全绿标记与这批输入的内容哈希相等；全量由主 agent 暂存之后在 HEAD + 暂存区的 worktree 里跑 --full，全绿标记按输入哈希分格：两条流的全部崩溃状态在 release 下逐个跑恢复，第一个事务与 E142 产物逐字比对，里程碑「第二个事务」固定脚本到 E 与用例的闭式比对；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
 # gate-covers: 崩溃点重放
 #
-# 里程碑「第一个事务」步 7：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
+# 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问）：
+#
+#   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>    主 agent 暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑一次
+#     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
+#     开跑与跑完各算一次输入的内容哈希，对不上（跑的过程中输入被改了）判红。全绿标记按输入哈希分格：
+#     `$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>`，不进工作树；放 common-dir，各 worktree 读写的是同一组。
+#     开跑一格都不删：同一批输入的结果是确定的，前一趟写下的那一格在这一趟跑的过程中照样算数。这一趟没写成标记就退出（判红、跑的过程中输入变了、
+#     被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动；全绿才写这一格。
+#     标记里有输入哈希、逐文件的「sha256  路径」、开跑与跑完的 UTC 时刻、工作线程数、两条流的计数行与 CHECKER 行原样。
+#   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
+#     快档：两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再按这批输入的哈希找那一格：
+#     有、里面记的哈希相同、计数行恰好三行、LAYER0 与 LAYER0B 两行都是 exhaustive=true，才判绿，成功句报快档计数、原样带出那一格的全量计数行与时刻。
+#     没有那一格判红（列出 common-dir 里最近写的一格与这一次不同的文件；不带哈希的旧名字 singlefs-layer0-full-green 不再认），
+#     哈希不同、计数行不对、缺 exhaustive=true 都判红。快档本身红照旧红。
+#
+# 输入：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的路径（唯一登记位，复用判定读的也是它）。文件集是 git 眼里这些路径下
+# 磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的）；逐个按内容算 sha256，「sha256  路径」按路径排序，整张再算一次 sha256。
+# 按内容算、不按 git 对象算：工作区跑的全量与 `--staged` 临时 worktree 里的同一份内容算出同一个数。
+# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动、工作区那一份 54 号），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
+#
+# 全量（里程碑「第一个事务」步 7）：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
 # 池级 checker（23 条不变量）、记录核对器（根在案而记录缺席、恢复自称新态而单元缺席）；三者的计数都由用例钉死。
-# 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；这里在 release 下跑它，
+# 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；--full 在 release 下带 --include-ignored 跑它，
 # 把用例打印的 `LAYER0 …` 计数行与逐条不变量的 `CHECKER …` 行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
 # 判别力：用例自己对着产物的十个计数断言（states / violations / root_persisted … 逐字），oracle 的判别力由同文件的靶向阳性对照证明
 # （根槽已持久而某个单元两份都没持久 ⇒ 8 个单元逐个都判红）。本阶段没有 fixtures 样本：判红要 cargo 真跑，装不进 fixtures 目录。
@@ -12,33 +32,79 @@
 # 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
 # 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、
 # 已跑完的状态数），这里边跑边转到本阶段的输出里，不删；成功行里报实际起了几个工作线程。
-# 没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
+# --full 里没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
+# --full 里 LAYER0 行的下一行不是 CHECKER 逐条不变量行：判红（不然成功句里「逐条不变量」打出来是空串，整道照样绿）。
+# 这两支判红都只拿合成日志核过：把判定段抽进临时脚本，喂一份缺那一行的日志。
+# 标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记、分格互不删、同一批输入两趟 --full 撞车不误红、同一批输入先绿后红删掉那一格、只改 Cargo.lock 判红、标记里 exhaustive=false 判红）拿临时仓加一个打合成日志的假 cargo 核过，同样不在 fixtures 里。
 set -uo pipefail
-ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
+# 参数：`--full` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
+layer0_tier="quick"
+root_argument=""
+for stage_argument in "$@"; do
+  case "$stage_argument" in
+    --full) layer0_tier="full" ;;
+    -*)
+      echo "  ✗ 认不出的参数：$stage_argument"
+      echo "     → 怎么办：只认 --full 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full] [项目根]"
+      exit 2 ;;
+    *) root_argument="$stage_argument" ;;
+  esac
+done
+ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
 cd "$ROOT" 2>/dev/null || exit 2
 
-# 这次改动没碰这道阶段判的东西就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
-# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。这是 C8（范围判定）的粗粒度前身：
-# 它只摘得掉「零行代码的改动」，摘不出别的，C8 照旧欠着。
-# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
+# 这一道读的路径：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的那几条（唯一登记位）。快档的改动范围与两档的输入哈希都按它算。
+layer0_stage_file_name="$(basename "$0")"
+layer0_input_table="$ROOT/.claude/gate.d/stage-inputs.tsv"
+layer0_registered_input_paths=()
+if [[ -f "$layer0_input_table" ]]; then
+  while IFS= read -r registered_row; do
+    read -r -a row_paths <<< "$registered_row"
+    layer0_registered_input_paths+=("${row_paths[@]}")
+  done < <(awk -F'\t' -v stage="$layer0_stage_file_name" '$1 == stage { print $2 }' "$layer0_input_table")
+fi
+if (( ${#layer0_registered_input_paths[@]} == 0 )); then
+  echo "  ✗ $layer0_input_table 里没有 $layer0_stage_file_name 这一行（或读不到这份表）：判不出这一道读哪些路径"
+  echo "     → 怎么办：在 .claude/gate.d/stage-inputs.tsv 里给本阶段登记它读的路径（制表符分隔，照别的行写）。"
+  exit 1
+fi
+layer0_input_paths_text="${layer0_registered_input_paths[*]}"
+
+# 快档先问两件事，任一答「可跳过」就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
+# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。--full 是收尾时点名要跑的，这两问都不问。
+# 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
 # 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
 # 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
-reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
-reuse_rc=$?
-if [[ "$reuse_rc" != 0 ]]; then
-  echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
-  echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
-  exit 77
-fi
-scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" crates/)"
-scope_rc=$?
-if [[ "$scope_rc" != 0 ]]; then
-  echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
-  echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。"
-  exit 77
+# 二问这次改动碰没碰登记给本阶段的那几条路径。这是 C8（范围判定）的粗粒度前身：它只摘得掉「零行输入的改动」，摘不出别的，C8 照旧欠着。
+if [[ "$layer0_tier" == quick ]]; then
+  reuse_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"
+  reuse_rc=$?
+  if [[ "$reuse_rc" != 0 ]]; then
+    echo "  ! 本阶段跳过（复用上一次整轮全绿的判定）：$reuse_reason"
+    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。"
+    exit 77
+  fi
+  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}")"
+  scope_rc=$?
+  if [[ "$scope_rc" != 0 ]]; then
+    echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
+    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，判据见 research/scripts/change-touches-crates.sh。"
+    exit 77
+  fi
 fi
 [[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 77; }
 
+# 全绿标记放 git 的 common-dir：不进工作树，主工作树与 `gate.sh --staged` 的临时 worktree 读写的是同一份。
+if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
+  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：层 0 全量的全绿标记没处放、也没处读"
+  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>。"
+  exit 1
+fi
+# 全绿标记按输入哈希分格：一格一个文件，文件名是这个前缀加「.<输入哈希>」。
+full_green_marker_prefix="$git_common_directory/singlefs-layer0-full-green"
+layer0_scratch_directory="$(mktemp -d)"
+trap 'rm -rf -- "${layer0_scratch_directory:?}"' EXIT
+
 machine_cores="$(nproc)"
 if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
   threads_origin="显式设的"
@@ -49,8 +115,11 @@ fi
 export SINGLEFS_LAYER0_THREADS
 
 # run_layer0_test_binary <测试二进制> <日志>：cargo 的整段输出进日志；`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
+# --full 带 --include-ignored（全量用例在平时 cargo test 里标 ignored）；快档不带，只跑不标 ignored 的那几条。
 run_layer0_test_binary() {
-  cargo test --release -p singlefs-harness --test "$1" -- --include-ignored --nocapture 2>&1 \
+  local -a libtest_selection=()
+  if [[ "$layer0_tier" == full ]]; then libtest_selection=(--include-ignored); fi
+  cargo test --release -p singlefs-harness --test "$1" -- "${libtest_selection[@]}" --nocapture 2>&1 \
     | tee "$2" \
     | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
     | sed -u 's/^/    /'
@@ -81,6 +150,159 @@ worker_threads_are_acceptable() { # <流的名字> <实际起的工作线程数>
   return 0
 }
 
+# write_layer0_input_manifest <清单文件>：这一道输入的逐文件清单（「sha256  路径」，按路径排序）写进清单文件，
+# 整张清单的 sha256 与文件数放进 layer0_input_hash / layer0_input_file_count。路径取 layer0_registered_input_paths。
+# 返回非 0：git 列不出文件、一个文件都没有、或读文件出错——三种都判不出哈希。
+write_layer0_input_manifest() {
+  local manifest_file="$1" listed_file
+  local -a existing_files=()
+  git -C "$ROOT" ls-files -z --cached --others --exclude-standard -- "${layer0_registered_input_paths[@]}" > "$manifest_file.listing" || return 1
+  # 工作区里删了、删除还没暂存的文件不算：跑的是磁盘上这一份，--staged 的临时 worktree 里它还在，两边照样对不上
+  while IFS= read -r -d '' listed_file; do
+    if [[ -f "$ROOT/$listed_file" ]]; then existing_files+=("$listed_file"); fi
+  done < <(LC_ALL=C sort -zu "$manifest_file.listing")
+  (( ${#existing_files[@]} > 0 )) || return 1
+  ( cd "$ROOT" && sha256sum -- "${existing_files[@]}" ) > "$manifest_file" || return 1
+  layer0_input_hash="$(sha256sum < "$manifest_file" | cut -d' ' -f1)"
+  layer0_input_file_count="${#existing_files[@]}"
+  return 0
+}
+
+fail_without_input_manifest() {
+  echo "  ✗ 算不出这一道输入的内容哈希：git 在登记的路径（${layer0_input_paths_text}）下一个文件都列不出来，或读文件出错"
+  echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text}，看列不列得出文件；"
+  echo "                列得出就逐个 sha256sum 一遍，找读不了的那一个。登记的路径写错了，改 .claude/gate.d/stage-inputs.tsv。"
+  exit 1
+}
+
+# report_manifest_differences <前一份清单> <后一份清单> <前一份的叫法> <后一份的叫法>：逐个列出两份清单里不同的文件，最多 20 个，另报总数。
+report_manifest_differences() {
+  local difference_lines difference_count
+  difference_lines="$(awk -v earlier_name="$3" -v later_name="$4" '
+    FNR == NR { earlier_hash[substr($0, 67)] = substr($0, 1, 64); next }
+    {
+      later_path = substr($0, 67)
+      if (!(later_path in earlier_hash)) print "只在" later_name "里：" later_path
+      else if (earlier_hash[later_path] != substr($0, 1, 64)) print "内容不同：" later_path
+      delete earlier_hash[later_path]
+    }
+    END { for (earlier_path in earlier_hash) print "只在" earlier_name "里：" earlier_path }
+  ' "$1" "$2" | LC_ALL=C sort)"
+  difference_count="$(grep -c . <<< "$difference_lines")"
+  echo "       不同的文件共 ${difference_count} 个（最多列 20 个）："
+  sed -n '1,20p' <<< "$difference_lines" | sed 's/^/         /'
+}
+
+# print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree、在里面用那棵树里的 54 号跑 --full 的命令。
+# 建法与共享 gate.sh --staged 相同：worktree add --detach HEAD，再 apply --index 暂存区的 diff（diff 为空就不套）。
+print_staged_worktree_full_commands() {
+  echo '                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
+  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }'
+  echo '                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'
+}
+
+# report_newest_other_marker <这一次的清单>：这批输入没有自己那一格时，拿 common-dir 里最近写的一格与这一次比，列出不同的文件；
+# 不带哈希的旧名字标记是分格之前写的，不再认，有就点名。
+report_newest_other_marker() {
+  local newest_marker newest_marker_manifest
+  newest_marker="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' -printf '%T@\t%p\n' 2>/dev/null | sort -rn | head -1 | cut -f2-)"
+  if [[ -n "$newest_marker" ]]; then
+    echo "       common-dir 里最近写的一格是 $(basename "$newest_marker")（跑完于 $(sed -n 's/^finished_utc=//p' "$newest_marker" | head -1)），与这一次的输入比："
+    newest_marker_manifest="$layer0_scratch_directory/newest-marker-manifest"
+    sed -n 's/^input_file //p' "$newest_marker" > "$newest_marker_manifest"
+    report_manifest_differences "$newest_marker_manifest" "$1" "那一格" "这一次"
+  else
+    echo "       common-dir 里一格全绿标记都没有。"
+  fi
+  if [[ -f "$full_green_marker_prefix" ]]; then
+    echo "       不带哈希的旧名字标记（$full_green_marker_prefix）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。"
+  fi
+}
+
+# run_quick_tier_of_stream <测试二进制> <流的名字>：快档跑一条流。判红打出路、返回 1；判绿把这条流的计数接到 quick_tier_report 后面。
+run_quick_tier_of_stream() {
+  local quick_log passed_and_ignored passed_count ignored_count
+  quick_log="$layer0_scratch_directory/quick-$1.log"
+  if ! run_layer0_test_binary "$1" "$quick_log"; then
+    tail -40 "$quick_log"
+    echo "  ✗ $2的快档用例判红（上面是 cargo test 的尾部）"
+    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $1 -- --nocapture"
+    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
+    return 1
+  fi
+  passed_and_ignored="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log" | head -1)"
+  read -r passed_count ignored_count <<< "$passed_and_ignored"
+  if [[ -z "${passed_count:-}" || "$passed_count" == 0 ]]; then
+    echo "  ✗ $2的快档跑过了，却读不到 cargo 的 test result 行，或一条用例都没通过：扫到 0 条不是通过"
+    echo "     → 怎么办：cargo test --release -p singlefs-harness --test $1 -- --list 看这个测试二进制里还剩几条不标 ignored 的用例；"
+    echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
+    return 1
+  fi
+  quick_tier_report+="${quick_tier_report:+；}$2 ${passed_count} 条通过、${ignored_count} 条 ignored"
+  return 0
+}
+
+# ── 快档：两条流不标 ignored 的用例，再核对全绿标记 ─────────
+if [[ "$layer0_tier" == quick ]]; then
+  quick_manifest="$layer0_scratch_directory/quick-manifest"
+  write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest
+  full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"
+  quick_tier_report=""
+  run_quick_tier_of_stream first_transaction_step_seven_layer0 "第一个事务那条流" || exit 1
+  run_quick_tier_of_stream second_transaction_step_zero_layer0 "两次发布那条流" || exit 1
+  if [[ ! -f "$full_green_marker_path" ]]; then
+    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入（哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件）没有层 0 全量的全绿标记（$full_green_marker_path）"
+    report_newest_other_marker "$quick_manifest"
+    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+    print_staged_worktree_full_commands
+    exit 1
+  fi
+  marker_input_hash="$(sed -n 's/^input_hash=//p' "$full_green_marker_path" | head -1)"
+  if [[ "$marker_input_hash" != "$layer0_input_hash" ]]; then
+    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入那一格全绿标记里记的输入哈希（${marker_input_hash:-标记里读不到}）与这一次的（${layer0_input_hash}，${layer0_input_file_count} 个文件）不同：那一格被改过或拷错了"
+    marker_manifest="$layer0_scratch_directory/marker-manifest"
+    sed -n 's/^input_file //p' "$full_green_marker_path" > "$marker_manifest"
+    report_manifest_differences "$marker_manifest" "$quick_manifest" "那一格" "这一次"
+    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+    print_staged_worktree_full_commands
+    exit 1
+  fi
+  marker_count_lines="$(grep -E '^(LAYER0|CHECKER|LAYER0B) ' "$full_green_marker_path")"
+  marker_count_line_total="$(grep -c . <<< "$marker_count_lines")"
+  if [[ "$marker_count_line_total" != 3 ]]; then
+    echo "  ✗ 这批输入那一格全绿标记的哈希对得上，计数行却不是 LAYER0 / CHECKER / LAYER0B 恰好三行（数到 ${marker_count_line_total} 行）：标记不是 --full 写的，或被改过"
+    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+    print_staged_worktree_full_commands
+    exit 1
+  fi
+  # 写这一格的 --full 本该把「不是全量」判红、不写标记；标记里仍有 exhaustive 不是 true 的，说明跑的那一份 54 号的判定被改过
+  marker_exhaustive_total="$(grep -cE '^LAYER0B? (.* )?exhaustive=true( |$)' <<< "$marker_count_lines")"
+  if [[ "$marker_exhaustive_total" != 2 ]]; then
+    echo "  ✗ 这批输入那一格全绿标记里，LAYER0 与 LAYER0B 两行不都是 exhaustive=true（是的只有 ${marker_exhaustive_total} 行）：写它的那一趟 --full 没把「不是全量」判红"
+    grep -E '^LAYER0B? ' <<< "$marker_count_lines" | sed 's/^/         /'
+    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+    print_staged_worktree_full_commands
+    echo "                那一趟判「层 0 不是全量」就是这一批让枚举退化了，去 crash.rs 的 enumerate_layer0 看。"
+    exit 1
+  fi
+  marker_finished_utc="$(sed -n 's/^finished_utc=//p' "$full_green_marker_path" | head -1)"
+  echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：${quick_tier_report}"
+  echo "  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}），两条流都是 exhaustive=true：层 0 全量跑完于 ${marker_finished_utc}，标记里的计数行原样："
+  sed 's/^/      /' <<< "$marker_count_lines"
+  exit 0
+fi
+
+# ── --full：记下开跑时的输入清单，跑完对一遍；开跑一格都不删，这一趟没写成标记就退出时才删这批输入那一格 ─────────
+full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
+manifest_at_start="$layer0_scratch_directory/manifest-at-start"
+write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
+input_hash_at_start="$layer0_input_hash"
+full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
+# 这一趟判红（任何一处 exit）或被打断：同一批输入先绿后红，前一趟那一格不再作数，退出前删掉它；写成了就留着
+full_marker_written=0
+trap 'if [[ "$full_marker_written" != 1 ]]; then rm -f -- "${full_green_marker_path:?}"; fi; rm -rf -- "${layer0_scratch_directory:?}"' EXIT
+echo "  · --full 开跑（${full_started_utc}）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 ${input_hash_at_start:0:16}…（${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}）"
+
 log="$(mktemp)"
 if ! run_layer0_test_binary first_transaction_step_seven_layer0 "$log"; then
   tail -40 "$log"
@@ -99,6 +321,12 @@ if [[ -z "$line" ]]; then
   echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例要 println! 一行以 LAYER0 开头的计数（字段见该用例的文档注释）。"
   exit 1
 fi
+if [[ -z "$checker_line" ]]; then
+  echo "  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话"
+  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；"
+  echo "                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。"
+  exit 1
+fi
 if [[ "$line" != *"exhaustive=true"* ]]; then
   echo "  ✗ 层 0 不是全量：$line"
   echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0 看。"
@@ -132,3 +360,38 @@ if [[ "$line_b" != *"exhaustive=true"* ]]; then
 fi
 worker_threads_are_acceptable "两次发布那条流" "$worker_threads_b" || exit 1
 echo "  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；${worker_threads_b} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line_b#LAYER0B }"
+
+# ── --full 判绿：跑完再算一次输入，与开跑时相同才写全绿标记 ─────────
+manifest_at_finish="$layer0_scratch_directory/manifest-at-finish"
+write_layer0_input_manifest "$manifest_at_finish" || fail_without_input_manifest
+if [[ "$layer0_input_hash" != "$input_hash_at_start" ]]; then
+  echo "  ✗ 全量跑的过程中这一道的输入变了（开跑 ${input_hash_at_start:0:16}…，跑完 ${layer0_input_hash:0:16}…）：两条流读到的不一定是同一版，不写全绿标记"
+  report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
+  echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+  print_staged_worktree_full_commands
+  exit 1
+fi
+full_finished_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
+marker_being_written="$full_green_marker_path.partial.$$"
+if ! {
+  echo "# 层 0 全量全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后写，按输入哈希分格，整轮门禁的快档按这批输入的哈希读这一格。不进工作树，别手改。"
+  echo "input_hash=$layer0_input_hash"
+  echo "input_file_count=$layer0_input_file_count"
+  echo "input_paths=$layer0_input_paths_text"
+  echo "started_utc=$full_started_utc"
+  echo "finished_utc=$full_finished_utc"
+  echo "judged_root=$ROOT"
+  echo "worker_threads=第一个事务那条流 ${worker_threads}、两次发布那条流 ${worker_threads_b}；SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核"
+  echo "$line"
+  echo "$checker_line"
+  echo "$line_b"
+  sed 's/^/input_file /' "$manifest_at_finish"
+} > "$marker_being_written" || ! mv -f -- "$marker_being_written" "$full_green_marker_path"; then
+  rm -f -- "${marker_being_written:?}"
+  echo "  ✗ 全量全绿，全绿标记却没写成（$full_green_marker_path）"
+  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
+  exit 1
+fi
+full_marker_written=1
+marker_slot_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' | grep -c .)"
+echo "  ✓ 全绿标记写进 $full_green_marker_path（输入哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件；开跑 ${full_started_utc}，跑完 ${full_finished_utc}；common-dir 里现有 ${marker_slot_total} 格）"
diff --git a/.claude/gate.d/stage-inputs.tsv b/.claude/gate.d/stage-inputs.tsv
index feeefac..d77c15f 100644
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -8,7 +8,7 @@
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
 # 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 的路径限定就按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock .claude/kb/layout/ research/results/	# 跑 crates 的崩溃点重放，与字节布局表、入库产物逐字比对
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）
 55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/
 59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试
 74-model-differential.sh	crates/ Cargo.toml Cargo.lock	# 拿 crates 里的实现与只住内存的理想模型对拍
diff --git a/.claude/hooks/agent-write-scope.tsv b/.claude/hooks/agent-write-scope.tsv
index 2a2e087..daa39b3 100644
--- a/.claude/hooks/agent-write-scope.tsv
+++ b/.claude/hooks/agent-write-scope.tsv
@@ -1,5 +1,7 @@
 # 项目 subagent 用 Write / Edit 时的写范围：一个 agent 一个或几个路径模式，一行一个。由 agent-write-scope.sh 在 PreToolUse 里判。
 # 三列用制表符分隔：agent 名、路径模式、出处。路径模式相对仓根，以 / 开头的是绝对路径；** 跨目录，* 不跨目录。
+# ⚠️ 模式里只有 ** 与 * 两个元字符：write-guard.sh 的 glob_to_regex 把别的字符一律 re.escape，
+#    写 e[0-9]*.rs 会被当成字面的「e[0-9]」、永远匹配不上，而闸只报「越出写范围」、不会说模式没写对（2026-09-22 踩过）。
 # 只登记 tools 里有 Write 或 Edit 的定义；表与定义逐项一致由 63-agent-write-scope.sh 判。Bash 里的写不经过这道闸。
 implementation-writer	crates/**	implementation-writer.md「写范围」
 implementation-writer	litmus/**	implementation-writer.md「写范围」
@@ -7,6 +9,8 @@ implementation-writer	/tmp/claude-1000/**	报告文件与草稿目录（定义
 kb-scribe	.claude/kb/**	kb-scribe.md「写范围」；relabel-item.py 改写的别处文件只许那个脚本改，不经 Edit
 kb-scribe	/tmp/claude-1000/**	报告文件与草稿目录
 experiment-runner	research/e7-index-bench/src/bin/**	experiment-runner.md「写范围」：bin 源文件
+experiment-runner	crates/singlefs-harness/src/bin/e*.rs	实验装置要驱动真实 crates/ 代码时的只读 bin（2026-09-22 E156 撞上：岔路问的是「今天这份代码的性质」，另写一份独立模型答不了）。限定 e 开头加数字，碰不到 first_transaction_* 那几个生产装置；这个 bin 不许改 core/checker，由代码三方与门禁 56 号管
+experiment-runner	crates/mutations.tsv	入库装置的变异行只追加进末尾（experiment-runner.md 第 2 步与「写范围」；2026-09-23 E158 撞上：闸拒了那次追加，三行落进 research/mutations/ 下、门禁 59 号复跑不到）
 experiment-runner	research/mutations/**	变异表
 experiment-runner	research/results/**	这个实验的产物
 experiment-runner	research/scripts/replay.sh	复跑登记行
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index af40d1a..1129ba8 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -16,11 +16,11 @@
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `python3 research/scripts/agent-watch.py watch --agents <这次派出的 agent id，逗号分隔>`：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
 子 agent 等长活时不续提示缓存，主 agent 也不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
 
-叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程。
+叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。
 
 ## 交回怎么读
 
@@ -41,7 +41,7 @@
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记）→ `gate-triage` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 9f8c1d5..bf1a7fe 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -15,10 +15,12 @@
 
 ## 改 agent 定义与共用约束，走同一条三步
 
-**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。门禁 71 号判这一条。
+**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。
 
 **改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。
 
+**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。
+
 ## 代码轮派腿之前记一份开工快照
 
 派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。
@@ -27,7 +29,7 @@
 
 ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判
 
-改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：层 0 崩溃点重放还读字节布局表与它比对的入库产物，herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制。
+改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。
 
 ⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：
 
@@ -60,4 +62,4 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 - **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
 
-**门禁管哪一半**：崩溃点重放由门禁 54 号判：没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
diff --git a/README.md b/README.md
index 008ef73..72390c4 100644
--- a/README.md
+++ b/README.md
@@ -101,10 +101,11 @@ waaagh！
 提交前跑门禁：
 
 ```bash
-bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 全量与 QEMU 真设备，要十几分钟）
+bash .claude/scripts/gate.sh              # 共享阶段 + .claude/gate.d/ 的项目阶段（含层 0 快档与全绿标记核对、QEMU 真设备，要十几分钟；层 0 全量不在里面）
 
 cargo test --workspace                    # 平时的单测；层 0 全量标 ignored，这里只跑缩小版
-bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放全量（release）
+bash .claude/gate.d/54-layer0-replay.sh   # 单跑层 0 崩溃点重放快档，并核这批输入有没有全量的全绿标记
+bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>  # 层 0 全量（release）：暂存之后在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（建法见快档判红时的出路句），判绿按输入哈希写一格全绿标记
 bash .claude/gate.d/55-qemu-first-transaction.sh  # 单跑 QEMU 两块 virtio 盘上的第一个事务
 bash .claude/scripts/lkmm.sh              # 单跑 LKMM，需要 herd7 与一棵内核树
 bash research/scripts/vm-bench.sh --selftest  # 单跑虚机装置自检（装置归项目）
```

## 二、按三份实现员报告、第一轮 diff（`_defs-gate54-tiering-r1-diff.md`）与 `sha256sums.txt` 核对：哪些 hunk 属于这一轮（gate54-tiering 三轮），哪些不属于

派发⚠️只点名 54 号、`stage-inputs.tsv`、README.md 三份「混着别的会话没提交的改动」；下表逐 hunk 核过全部 8 份，另发现 `.claude/hooks/agent-write-scope.tsv`、`.claude/main-agent.md`、`.claude/rules/implementation-workflow.md` 三份也混着与这一轮无关的未提交内容——这三份不在派发点名的三份之内，按同一条道理一并列出，供主 agent过目。

| 文件 | hunk（`@@` 标记） | 判定 | 依据 |
|---|---|---|---|
| `.claude/gate.d/54-layer0-replay.sh` | 全部 6 个 hunk | 属于这一轮（r1+r2+r3 累积） | 当前文件 sha256 `232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7` 与 r2 正文二「54 号真文件 397 行，sha256 …（第三轮报告）」逐字相同——工作区这份文件从 r3 之后未再被任何会话改动；r2-report.md 只报这一处改动的摘要（diff 在 `r2/r2.diff`，+102/−47），未逐字节附 r2 阶段的 diff 正文，**这一段本身裁不开、按「截不开的整段给出」处理**，但整份文件的最终字节已用 sha256 核对为 r3 收尾时的样子，不含 r1/r2/r3 之外的改动 |
| `.claude/gate.d/stage-inputs.tsv` | 1 个 hunk（`@@ -8,7 +8,7 @@`） | 属于这一轮（r1，之后未再变） | 与 `_defs-gate54-tiering-r1-diff.md:346-358` 逐字节相同（`index feeefac..d77c15f` 一致） |
| `README.md` | 1 个 hunk（`@@ -101,10 +101,11 @@`） | 属于这一轮（r1 起始改法 + r3 精修） | 与 `r3-report.md` 附录 `r3/README.diff`（第 684-698 行）逐字比对：r3 的「改前」状态正是本 hunk「单跑层 0 崩溃点重放全量（release）」那一句已经被 r1 改成快档措辞之后的样子，r3 只把 `--full` 那一行精修成 worktree 写法；未见 r1/r3 之外的改动 |
| `.claude/agents/crash-verifier.md` | 1 个 hunk（`@@ -20,8 +20,8 @@`） | 属于这一轮（r1 起始改法 + r2 精修） | 与 `_defs-gate54-tiering-r1-diff.md:8-20` 比对：r1 版本写「54 号带 `--full`（不带只跑快档）」，今天是「54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带」——正是 K2 从 r1 判决「站不住，改」到 r2 落地的措辞，同一 hunk 位置、同一段落 |
| `.claude/agents/experiment-runner.md` | hunk 1（`@@ -24,7 +24,7 @@`，「做什么」第 2 步） | **不属于这一轮** | 与 `_defs-gate54-tiering-r1-diff.md:25-31` 逐字节相同——r1 开工前就已存在于工作区，主题是「入库装置」bin 路径与常量引用规则（2026-09-22 E156），跟 K1（`crates/mutations.tsv` 写范围）无关；三份实现员报告都没有改这一步 |
| `.claude/agents/experiment-runner.md` | hunk 2（`@@ -36,7 +36,7 @@`，「写范围」） | 属于这一轮（r1，之后未再变） | 与 `_defs-gate54-tiering-r1-diff.md:34-39` 逐字节相同，新增的正是 K1「入库装置的变异行（`crates/mutations.tsv` 末尾，只追加这个实验的行，不改别人的行）」 |
| `.claude/hooks/agent-write-scope.tsv` | hunk 1（`@@ -1,5 +1,7 @@`）新增两行注释 | **不属于这一轮** | 与 `_defs-gate54-tiering-r1-diff.md:363-369` 逐字节相同，r1 开工前已存在，内容是 2026-09-22 E156 撞上的 glob 元字符提醒，跟 gate54 分档无关 |
| `.claude/hooks/agent-write-scope.tsv` | hunk 2（`@@ -7,6 +9,8 @@`）新增两行 | **一行属于这一轮、一行不属于——同一 hunk 内裁不开** | 与 `_defs-gate54-tiering-r1-diff.md:371-379` 逐字节相同。`experiment-runner\tcrates/singlefs-harness/src/bin/e*.rs\t…`（E156，2026-09-22）不属于这一轮；`experiment-runner\tcrates/mutations.tsv\t…`（K1，2026-09-23 E158 撞上）属于这一轮。两行相邻在同一个 `@@` 段落里，git 不会把它们拆成两个 hunk，故整段保留，不单独抽出这一行 |
| `.claude/main-agent.md` | hunk 1（`@@ -16,11 +16,11 @@`，「派出去之后」看门狗） | **不属于这一轮** | 与 `_defs-gate54-tiering-r1-diff.md:384-393` 逐字节相同（`agent-watch.py` → `watch.sh` 那处改法），r1 开工前已存在，与 gate54 分档无关的看门狗脚本改名 |
| `.claude/main-agent.md` | hunk 2（`@@ -41,7 +41,7 @` "暂存之后、提交之前跑门禁"） | 属于这一轮 | r1 的原始改法落在「一个阶段任务结束」那一行末尾（`_defs-gate54-tiering-r1-diff.md:398-406`，追加「这一批碰了 crates/ 就后台跑 …--full」一句）；K3 被 r1 判决「站不住，改」后，落地版把改动从那一行**撤回**、改到「暂存之后、提交之前跑门禁」那一行——今天这一 hunk 正是撤回加新写的结果，径直体现 K3 从判决到落地 |
| `.claude/rules/implementation-workflow.md` | hunk 1（`@@ -15,10 +15,12 @@`，「改 agent 定义与共用约束」） | **不属于这一轮** | 门禁 71 号 → 「规则纪律（项目本地）」与新增「三步在定义上各取什么」一段，正文没有点名这一节（正文只点「复用上一次全量门禁的判定」与「测试与崩溃检测优先多线程」两节）；`research/prompts/gate-fix-forks-r3-snapshot/changed-during-legs.md` 记录这处改动是「主 agent 在 2026-09-24 02:20 JST 改了 1 处」，属于另一个并行会话（gate-fix-forks 轮），不是 gate54-tiering |
| `.claude/rules/implementation-workflow.md` | hunk 2（`@@ -27,7 +29,7 @@`，「复用上一次全量门禁的判定」） | 属于这一轮 | 正文一节名点名「复用上一次全量门禁的判定」；改法把「层 0 崩溃点重放还读字节布局表…」换成「每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准」，正是 V4 的落地措辞，虽不见于三份实现员报告（均未提及自己改了这一处），但与正文点名的范围一致 |
| `.claude/rules/implementation-workflow.md` | hunk 3（`@@ -60,4 +62,4 @@`，「门禁管哪一半」末段） | 属于这一轮（r3） | 与 `r3-report.md` 附录 `r3/implementation-workflow.diff`（第 705-717 行）逐字节相同 |

## 三、裁剪：只保留确认属于这一轮的 hunk（`.claude/main-agent.md`、`.claude/rules/implementation-workflow.md` 可按 hunk 边界干净裁开；`.claude/agents/experiment-runner.md` 与 `.claude/hooks/agent-write-scope.tsv` 的混杂发生在同一 hunk 内部，裁不开，按上表标注保留整段，不在此重复）

```diff
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index af40d1a..1129ba8 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -41,7 +41,7 @@
 | 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记）→ `gate-triage` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
```

```diff
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index 9f8c1d5..bf1a7fe 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -27,7 +29,7 @@
 
 ## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判
 
-改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：层 0 崩溃点重放还读字节布局表与它比对的入库产物，herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制。
+改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。
 
 ⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：
 
@@ -60,4 +62,4 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 - **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
 
-**门禁管哪一半**：崩溃点重放由门禁 54 号判：没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
```

本轮没有新建文件，故不附「新文件全文」一节。
