# 附录二：门禁批改动的 diff（原样放，另附主 agent 对 check-segment-registry.py 的两处修补）

**生成时刻**：2026-09-26。

**基准**：门禁批实现员在改动前用 `cp -p` 备份的文件快照（记在 `/tmp/claude-1000/gate-batch-m2/spec.md`「怎么改」一节），**不是**某个 git 提交号。经核对（`git diff HEAD -- <文件>`，HEAD = `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`，2026-09-25 +0000），当前工作区里这 7 个 git 追踪文件相对 HEAD 的差异比下面这份 diff 更大：例如 `.claude/agents/crash-verifier.md` frontmatter 的 `model: sonnet` → `model: opus`、新增 `effort: high`，`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」一节的整段改写，这两处在下面这份 diff 里都是没有 `+`/`-` 的上下文行（说明改动前的那份备份本身已经带着比 HEAD 更多、这次任务之外的改动，大概率是同一批「里程碑二收尾」里更早、还没提交的另一层改动）；`research/scripts/admission.py` 在 git 里完全未跟踪（`git status --porcelain` 显示 `??`），diff 的「改动前」一侧只对应那份 `cp -p` 备份，不对应任何 git 版本。下面「一」原样是 `/tmp/claude-1000/gate-batch-m2/my-changes.diff` 的内容（`wc -l` 2543 行、8 个文件，用 `diff -u`／等价手段生成，不带 `diff --git` 头），不是 `git diff HEAD` 的内容；「二」是主 agent 自己对 `research/scripts/check-segment-registry.py` 的两处修补（数字间下划线、认不出第二条流登记句就判红），改前备份 `check-segment-registry.py.before`（2026-09-22），与仓里今天这份 `diff -u` 原样。

## 一、门禁批交回的 diff（`/tmp/claude-1000/gate-batch-m2/my-changes.diff` 原样，2543 行，8 个文件）

```diff
--- a/.claude/gate.d/54-layer0-replay.sh
+++ b/.claude/gate.d/54-layer0-replay.sh
@@ -1,73 +1,76 @@
 #!/usr/bin/env bash
-# gate-stage: 层 0 崩溃点重放（整轮门禁跑快档：两条流不标 ignored 的用例，再核对层 0 全量的全绿标记与这批输入的内容哈希相等；全量由主 agent 暂存之后在 HEAD + 暂存区的 worktree 里跑 --full，全绿标记按输入哈希分格：两条流的全部崩溃状态在 release 下逐个跑恢复，第一个事务与 E142 产物逐字比对，里程碑「第二个事务」固定脚本到 E 与用例的闭式比对；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
+# gate-stage: 层 0 崩溃点重放与登记的崩溃枚举用例（整轮门禁跑快档：两条流不标 ignored 的用例，再逐条核 stage-inputs.tsv 里 crash-case: 那几条用例各自那一格全绿标记与它这批输入的指纹相等；全量由主 agent 暂存之后在 HEAD + 暂存区的 worktree 里跑 --full，逐条用例照复用判定跑：那一格全绿标记在就复用，不在才在 release 下跑它、判绿写那一格；两条流的层 0 全量带断点续跑，第一个事务与 E142 产物逐字比对、里程碑「第二个事务」固定脚本到 E 与用例的闭式比对由用例自己断言；只做过 mkfs 的池可写挂载再发第一个文件版本那条流与第一个事务逐项相同，由 cargo test 里的快用例钉住，不另枚举）
 # gate-covers: 崩溃点重放
 #
-# 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问）：
+# 分两档（用户 2026-09-19 定，原话「每次主 agent 执行完任务后统一执行」，records/2026-09-19-里程碑二遗留收拢.md「五之二」第 8 问；
+# 用户 2026-09-26 定逐条用例复用，原话「下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」，
+# records/2026-09-24-里程碑二收尾调度.md 第三节「崩溃枚举的跑法」那一行）：
 #
-#   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>    主 agent 暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑一次
-#     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
-#     开跑与跑完各算一次输入的内容哈希，对不上（跑的过程中输入被改了）判红。全绿标记按输入哈希分格：
-#     `$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>`，不进工作树；放 common-dir，各 worktree 读写的是同一组。
-#     开跑一格都不删：同一批输入（连同判它的 54 号与工具链，见「输入」一段）的结果是确定的，前一趟写下的那一格在这一趟跑的过程中照样算数。
-#     这一趟没写成标记就退出（判红、被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；「跑的过程中输入变了」那一支判红不删：
-#     两条流读到的不一定是开跑那一批，它说不出那一批的好坏。被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动；全绿才写这一格。
-#     标记里有输入哈希、逐文件的「sha256  路径」、开跑与跑完的时刻、工作线程数、两条流的计数行与 CHECKER 行原样。
+#   bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full [--start-over] <worktree>
+#     主 agent 暂存之后（提交时由崩溃验证员），在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
+#     逐条崩溃枚举用例（.claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几行，照登记表的次序）：先算这条用例这批输入的指纹
+#     （research/scripts/admission.py crash-case-manifest：登记路径下的文件减去别的测试目标独占的测试文件，加判它的 54 号、工具链、构建环境
+#     与这条用例的登记行）；那一格全绿标记在、作数就复用，不跑；不在才跑 `cargo test --release -p <包> --test <测试目标> -- --include-ignored
+#     --exact <用例函数> --nocapture`，按登记行第三列判日志（crash-case-judge），开跑与跑完各算一次指纹，相同才写那一格（crash-case-record）。
+#     全绿标记在 git common-dir：`singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 读写同一组；别的格不动。
+#     一条判红删它这批输入那一格（先绿后红，前一趟那一格不再作数），接着跑下一条；「跑的过程中输入变了」那一支判红不删：它说不出开跑那一批的好坏。
+#     断点续跑：跑用例时设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=<common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>（不随 worktree 删掉）、
+#     SINGLEFS_LAYER0_INPUT_FINGERPRINT=<这条用例的输入指纹>；--start-over 设 SINGLEFS_LAYER0_START_OVER=1（丢掉进度文件、从头跑），
+#     不带它时从调用方的环境里清掉这个变量。续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-harness/src/layer0_progress.rs。
 #   bash .claude/gate.d/54-layer0-replay.sh [项目根]           整轮门禁的默认（gate.sh 只传项目根）
 #     快档先核登记的每一条路径 git 至少列得出一个文件（git ls-files -co --exclude-standard -- <那一条>），有一条列不出判红，之后才问复用与改动范围。
-#     两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再按这批输入的哈希找那一格：
-#     有、里面记的哈希相同、LAYER0 / CHECKER / LAYER0B 三种计数行各恰好一行、LAYER0 与 LAYER0B 两行各自带 exhaustive=true，才判绿，
-#     成功句报快档计数、原样带出那一格的全量计数行与时刻。
-#     没有那一格判红（列出 common-dir 里最近写的一格与这一次不同的文件；不带哈希的旧名字 singlefs-layer0-full-green 不再认），
-#     哈希不同、计数行不对、缺 exhaustive=true 都判红。快档本身红照旧红。
+#     两条流的测试二进制在 release 下只跑不标 ignored 的用例，一条都没通过判红；再逐条崩溃枚举用例算它这批输入的指纹、核那一格
+#     （admission.py crash-case-marker-check：在、记的指纹与用例相同、test result 是 1 passed、登记的计数行各恰好一行、要 exhaustive=true 的带着），
+#     全部作数才判绿，成功句逐条原样带出那一格的计数行与时刻；有一条不作数判红，逐条列原因（没有那一格时比最近写的一格与这一次的清单），出路是跑 --full。
+#     按整批输入分格的旧标记（singlefs-layer0-full-green.*）不再认。
 #
-# 输入：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的路径（唯一登记位，复用判定读的也是它），外加判这一格的 54 号与工具链。
-# 文件集是 git 眼里这些路径下磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的）；逐个按内容算 sha256，「sha256  路径」按路径排序；
-# 清单末尾再加两行：跑的这一份 54 号的 sha256（名字 `<判它的 54 号：54-layer0-replay.sh>`）、`cargo -V` 与 `rustc -V` 原样输出的 sha256
-# （名字 `<工具链：…>`）；整张再算一次 sha256。报的文件数只数登记路径下的文件。这两行只进哈希，不进复用与改动范围那两问：
-# 同一份 crates/ 换了判它的 54 号或工具链，前一趟那一格就不再作数。
-# 按内容算、不按 git 对象算：工作区跑的全量与 `--staged` 临时 worktree 里的同一份内容算出同一个数。
-# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动、工作区那一份 54 号），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
+# 输入：整道阶段的复用判定与改动范围按 stage-inputs.tsv 里本阶段那一行（唯一登记位）；每条崩溃枚举用例的输入按它自己那一行，
+# 由 admission.py 按内容算（主工作区跑的与 `--staged` 临时 worktree 里的同一份内容算出同一个数）。
+# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
 #
-# 全量（里程碑「第一个事务」步 7）：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
-# 池级 checker（23 条不变量）、记录核对器（根在案而记录缺席、恢复自称新态而单元缺席）；三者的计数都由用例钉死。
-# 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；--full 在 release 下带 --include-ignored 跑它，
-# 把用例打印的 `LAYER0 …` 计数行与逐条不变量的 `CHECKER …` 行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
-# 判别力：用例自己对着产物的十个计数断言（states / violations / root_persisted … 逐字），oracle 的判别力由同文件的靶向阳性对照证明
-# （根槽已持久而某个单元两份都没持久 ⇒ 8 个单元逐个都判红）。本阶段没有 fixtures 样本：判红要 cargo 真跑，装不进 fixtures 目录。
+# 全量（里程碑「第一个事务」步 7）：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑恢复与 oracle、池级 checker、记录核对器，
+# 计数由用例钉死。全量要跑很久，平时 `cargo test` 里那几条标 ignored；--full 在 release 下带 --include-ignored --exact 逐条跑。
+# 用例的判别力在用例自己：对着产物与闭式的计数断言，oracle 的判别力由同文件的靶向阳性对照证明。
 #
 # 多线程（增补 2 收口表第 41 行；2026-09-18 用户定：测试与崩溃检测优先多线程）：crash.rs 按状态序号区间切片、多线程跑，
-# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`（片号、序号区间、段号、
-# 已跑完的状态数），这里边跑边转到本阶段的输出里，不删；成功行里报实际起了几个工作线程。
-# --full 里没显式把 SINGLEFS_LAYER0_THREADS 设成 1、而本机多于 1 核却只起了 1 个工作线程：判红（多半是线程数没传进去）。
-# --full 里 LAYER0 行的下一行不是 CHECKER 逐条不变量行：判红（不然成功句里「逐条不变量」打出来是空串，整道照样绿）。
-# 这两支判红都只拿合成日志核过：把判定段抽进临时脚本，喂一份缺那一行的日志。
-# 标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记、分格互不删、同一批输入两趟 --full 撞车不误红、同一批输入先绿后红删掉那一格、只改 Cargo.lock 判红、标记里 exhaustive=false 判红、标记里 LAYER0 抄两遍而没有 LAYER0B 判红、
-# 登记的路径列不出文件判红）拿临时仓加一个打合成日志的假 cargo 核过，同样不在 fixtures 里；判它的 54 号换了、工具链换了判红，
-# 跑的过程中输入变了不删开跑那一格，拿 research/prompts/defs-gate54-tiering-r2-opus-model/fix-arms.sh 的那几段历史核过。
+# 线程数由 SINGLEFS_LAYER0_THREADS 传进去——没设就取本机核数（nproc）。每跑完一片，用例打一行 `LAYER0_PROGRESS`，这里边跑边转到本阶段的输出里。
+# 登记了 threads=<前缀> 的用例：按那一行计数的状态数找 `LAYER0_PARALLEL_FINISHED`，这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、
+# SINGLEFS_LAYER0_THREADS 没显式设成 1，判红（多半是线程数没传进去）；全部片从进度文件读回（起 0 个线程）、只剩 1 片要跑（最多起 1 个）都不判红。
+#
+# 判别力：日志与标记怎么判、输入指纹怎么算，由 research/scripts/admission.py --selftest 的「崩溃枚举用例」那几格拿合成日志与临时仓核；
+# 这一份脚本的流程（逐条复用、只重跑输入变了的那一条、续跑的三个环境变量、--start-over、快档缺一格判红、跑的过程中输入变了不写标记）
+# 由同一份自证的「54 号」那几格核：把这一份拷进临时仓（不放在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑。本阶段没有 fixtures 样本。
 set -uo pipefail
-# 参数：`--full` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
+# 参数：`--full`、`--start-over` 与项目根，顺序不限；gate.sh 只传项目根，于是整轮门禁走快档。
 layer0_tier="quick"
+layer0_start_over=0
 root_argument=""
 for stage_argument in "$@"; do
   case "$stage_argument" in
     --full) layer0_tier="full" ;;
+    --start-over) layer0_start_over=1 ;;
     -*)
       echo "  ✗ 认不出的参数：$stage_argument"
-      echo "     → 怎么办：只认 --full 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full] [项目根]"
+      echo "     → 怎么办：只认 --full、--start-over 与项目根，顺序不限：bash .claude/gate.d/54-layer0-replay.sh [--full [--start-over]] [项目根]"
       exit 2 ;;
     *) root_argument="$stage_argument" ;;
   esac
 done
+if [[ "$layer0_start_over" == 1 && "$layer0_tier" != full ]]; then
+  echo "  ✗ --start-over 只跟 --full 一起用：快档不跑全量，没有进度文件可丢"
+  echo "     → 怎么办：要丢掉进度文件、从头跑全量，写成 bash .claude/gate.d/54-layer0-replay.sh --full --start-over <根>。"
+  exit 2
+fi
 ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
-# 跑的这一份 54 号进输入清单（write_layer0_input_manifest）；相对的 $0 在 cd 之后会指错，所以在 cd 之前取成绝对路径
+# 跑的这一份 54 号进每条用例的输入清单；相对的 $0 在 cd 之后会指错，所以在 cd 之前取成绝对路径
 layer0_stage_script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
-# 门禁与实验共用的准入模块：读登记表、算输入清单、判前提都经它（research/scripts/admission.py，文件头写全了各子命令）
+# 门禁与实验共用的准入模块：读登记表、算输入清单、判前提、判日志与读写全绿标记都经它（research/scripts/admission.py，文件头写全了各子命令）
 layer0_stage_repository="$(cd "$(dirname "$0")/../.." && pwd)"
 layer0_admission_module="$layer0_stage_repository/research/scripts/admission.py"
 cd "$ROOT" 2>/dev/null || exit 2
 
 # 这一道读的路径：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的那几条（唯一登记位），经准入模块的 paths 读。
-# 快档的改动范围与两档的输入哈希都按它算。
+# 快档的复用判定与改动范围按它算；每条崩溃枚举用例的输入另按它自己那一行算。
 layer0_stage_file_name="$(basename "$0")"
 layer0_input_table="$ROOT/.claude/gate.d/stage-inputs.tsv"
 layer0_registered_input_paths=()
@@ -83,7 +86,7 @@
 layer0_input_paths_text="${layer0_registered_input_paths[*]}"
 
 # 快档先问两件事，任一答「可跳过」就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
-# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。--full 是收尾时点名要跑的，这两问都不问。
+# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。--full 是收尾时点名要跑的，这两问都不问（逐条用例的复用另判）。
 # 一问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
 # 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
 # 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
@@ -120,84 +123,89 @@
 # 前提（cargo、rustc）登记在 stage-inputs.tsv 本阶段那一行第三列，经准入模块判，没齐判红（不退 77）
 python3 "$layer0_admission_module" gate-preconditions "$layer0_stage_repository" "$layer0_stage_file_name" || exit 1
 
-# 全绿标记放 git 的 common-dir：不进工作树，主工作树与 `gate.sh --staged` 的临时 worktree 读写的是同一份。
+# 全绿标记与续跑的进度文件放 git 的 common-dir：不进工作树，主工作树与 `gate.sh --staged` 的临时 worktree 读写的是同一份。
 if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
-  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：层 0 全量的全绿标记没处放、也没处读"
-  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-layer0-full-green.<输入哈希>。"
+  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：崩溃枚举用例的全绿标记与续跑的进度文件没处放、也没处读"
+  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；标记在 \$(git rev-parse --git-common-dir)/singlefs-crash-case-green.<用例名>.<输入指纹>。"
   exit 1
 fi
-# 全绿标记按输入哈希分格：一格一个文件，文件名是这个前缀加「.<输入哈希>」。
-full_green_marker_prefix="$git_common_directory/singlefs-layer0-full-green"
+layer0_progress_root="$git_common_directory/singlefs-layer0-progress"
 layer0_scratch_directory="$(mktemp -d)"
 trap 'rm -rf -- "${layer0_scratch_directory:?}"' EXIT
 
 machine_cores="$(nproc)"
 if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
   threads_origin="显式设的"
+  threads_origin_word="explicit"
 else
   SINGLEFS_LAYER0_THREADS="$machine_cores"
   threads_origin="没设，取本机核数"
+  threads_origin_word="default"
 fi
 export SINGLEFS_LAYER0_THREADS
 
-# run_layer0_test_binary <测试二进制> <日志>：cargo 的整段输出进日志；`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
-# --full 带 --include-ignored（全量用例在平时 cargo test 里标 ignored）；快档不带，只跑不标 ignored 的那几条。
-run_layer0_test_binary() {
-  local -a libtest_selection=()
-  if [[ "$layer0_tier" == full ]]; then libtest_selection=(--include-ignored); fi
-  cargo test --release -p singlefs-harness --test "$1" -- "${libtest_selection[@]}" --nocapture 2>&1 \
-    | tee "$2" \
-    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
-    | sed -u 's/^/    /'
-  return "${PIPESTATUS[0]}"
-}
-
-# worker_threads_of_full_run <日志> <计数行>：计数行里的状态数对上的那一行 `LAYER0_PARALLEL_FINISHED`，取实际起的工作线程数。
-worker_threads_of_full_run() {
-  local states
-  states="$(sed -n 's/^[A-Z0-9]* states=\([0-9]*\) .*/\1/p' <<<"$2")"
-  grep "^LAYER0_PARALLEL_FINISHED states=$states " "$1" | head -1 | sed -n 's/.* worker_threads=\([0-9]*\) .*/\1/p'
-}
+# 登记的崩溃枚举用例，一条一行「键、包、测试目标、用例函数」（制表符分隔），照登记表的次序；登记有错判红。
+if ! crash_case_listing="$(python3 "$layer0_admission_module" crash-cases "$ROOT")"; then
+  printf '%s\n' "$crash_case_listing" | sed 's/^/       /'
+  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里 crash-case: 那几行登记有错（上面逐条列出）：判不出要跑、要核哪几条崩溃枚举用例"
+  echo "     → 怎么办：照 research/scripts/admission.py 文件头「崩溃枚举用例行」改那几行（test=<包>:<测试目标>:<用例函数> 恰好一条，用例函数在测试目标里找得到）；"
+  echo "                单跑 python3 research/scripts/admission.py crash-cases <项目根>，改到它退 0 为止。"
+  exit 1
+fi
+crash_case_rows=()
+if [[ -n "$crash_case_listing" ]]; then mapfile -t crash_case_rows <<< "$crash_case_listing"; fi
+if (( ${#crash_case_rows[@]} == 0 )); then
+  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里一条崩溃枚举用例（键是 crash-case: 的行）都没登记：层 0 全量没有东西可跑、可核"
+  echo "     → 怎么办：两条流的层 0 全量至少各登记一行，写法见 research/scripts/admission.py 文件头「崩溃枚举用例行」。"
+  exit 1
+fi
 
-# 没显式设成 1、本机多于 1 核却只起了 1 个工作线程（或根本没打收尾行）：判红。返回 0 表示线程数没问题。
-worker_threads_are_acceptable() { # <流的名字> <实际起的工作线程数>
-  if [[ -z "$2" ]]; then
-    echo "  ✗ $1：全量用例跑过了，却没打印状态数对得上的 LAYER0_PARALLEL_FINISHED 行，判不出起了几个工作线程"
-    echo "     → 怎么办：全量那条用例要经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 跑（它开跑与跑完各打一行 LAYER0_PARALLEL_*）；"
-    echo "                绕开它自己逐个跑状态的写法退回了单线程，改回去。"
+# write_crash_case_manifest <键> <清单文件>：这条用例这批输入的逐文件清单写进清单文件，指纹、文件数、减去的文件数放进
+# case_fingerprint / case_file_count / case_excluded_count。算不出返回 1，原因放进 case_manifest_problem。
+write_crash_case_manifest() {
+  local manifest_summary
+  if ! manifest_summary="$(python3 "$layer0_admission_module" crash-case-manifest "$ROOT" "$1" "$2" \
+      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" --toolchain --build-environment)"; then
+    case_manifest_problem="$manifest_summary"
     return 1
   fi
-  if [[ "$2" == 1 && "$machine_cores" -gt 1 && ! ( "$threads_origin" == "显式设的" && "$SINGLEFS_LAYER0_THREADS" == 1 ) ]]; then
-    echo "  ✗ $1：本机 $machine_cores 核、SINGLEFS_LAYER0_THREADS=$SINGLEFS_LAYER0_THREADS（$threads_origin），全量枚举却只起了 1 个工作线程"
-    echo "     → 怎么办：看 crash.rs 的 Layer0Parallelism::from_environment 读没读到 SINGLEFS_LAYER0_THREADS、state_slices 切出来的片数够不够分给每个线程；"
-    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh。"
+  read -r case_fingerprint case_file_count case_excluded_count <<< "$manifest_summary"
+  if [[ ! "$case_fingerprint" =~ ^[0-9a-f]{64}$ || ! "$case_file_count" =~ ^[1-9][0-9]*$ || ! "$case_excluded_count" =~ ^[0-9]+$ ]]; then
+    case_manifest_problem="准入模块打的不是「<指纹> <文件数> <减去的文件数>」：$manifest_summary"
     return 1
   fi
   return 0
 }
 
-# write_layer0_input_manifest <清单文件>：这一道输入的逐文件清单（「sha256  路径」，按路径排序）写进清单文件，末尾再加两行：
-# 跑的这一份 54 号的 sha256、`cargo -V` 与 `rustc -V` 原样输出的 sha256（名字用尖括号括起，与路径分开）。
-# 整张清单的 sha256 与文件数（只数登记路径下的文件）放进 layer0_input_hash / layer0_input_file_count。
-# 清单由准入模块的 manifest 写（路径取登记表本阶段那一行，与 layer0_registered_input_paths 同一份）：文件集是 git 眼里这些路径下
-# 磁盘上真有的文件（工作区里删了、删除还没暂存的不算：跑的是磁盘上这一份，--staged 的临时 worktree 里它还在），按字节序排；
-# 判这一格的 54 号与工具链也进键：同一份 crates/ 换了判据或编译器，前一趟的结论就不再作数。
-# 返回非 0：git 列不出文件、一个文件都没有、读文件出错、或 cargo -V / rustc -V 跑不出来——四种都判不出哈希（模块退 2，原因打在 stderr）。
-write_layer0_input_manifest() {
-  local manifest_file="$1" manifest_summary
-  manifest_summary="$(python3 "$layer0_admission_module" manifest "$ROOT" "$layer0_stage_file_name" "$manifest_file" \
-    --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" --toolchain)" || return 1
-  read -r layer0_input_hash layer0_input_file_count <<< "$manifest_summary"
-  [[ "$layer0_input_hash" =~ ^[0-9a-f]{64}$ && "$layer0_input_file_count" =~ ^[1-9][0-9]*$ ]] || return 1
+# delete_crash_case_marker <键> <指纹>：这条用例这批输入那一格全绿标记删掉（判红时；前一趟那一格不再作数）。
+delete_crash_case_marker() {
+  local marker_path
+  marker_path="$(python3 "$layer0_admission_module" crash-case-marker-path "$ROOT" "$1" "$2")" || return 0
+  if [[ -n "$marker_path" ]]; then rm -f -- "${marker_path:?}"; fi
   return 0
 }
 
-fail_without_input_manifest() {
-  echo "  ✗ 算不出这一道输入的内容哈希：git 在登记的路径（${layer0_input_paths_text}）下一个文件都列不出来、读文件出错，或 cargo -V / rustc -V 跑不出来"
-  echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text}，看列不列得出文件；"
-  echo "                列得出就逐个 sha256sum 一遍，找读不了的那一个。登记的路径写错了，改 .claude/gate.d/stage-inputs.tsv。"
-  echo "                文件都读得了，就在项目根跑 cargo -V && rustc -V：跑不出来是工具链没装好，bash .claude/scripts/env.sh 看缺什么。"
-  exit 1
+# run_layer0_test_binary <测试二进制> <日志>：快档跑一条流不标 ignored 的用例；cargo 的整段输出进日志。
+run_layer0_test_binary() {
+  cargo test --release -p singlefs-harness --test "$1" -- --nocapture 2>&1 \
+    | tee "$2" \
+    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
+    | sed -u 's/^/    /'
+  return "${PIPESTATUS[0]}"
+}
+
+# run_crash_case <包> <测试目标> <用例函数> <日志> <输入指纹>：--full 跑一条崩溃枚举用例，带续跑的三个环境变量；
+# cargo 的整段输出进日志，`LAYER0_PROGRESS` 行边跑边转到本阶段的输出（跑全量时这里一直在涨）。
+run_crash_case() {
+  local -a progress_settings=(SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$layer0_progress_root/$5" SINGLEFS_LAYER0_INPUT_FINGERPRINT="$5")
+  local -a start_over_setting=(-u SINGLEFS_LAYER0_START_OVER)
+  if [[ "$layer0_start_over" == 1 ]]; then start_over_setting=(SINGLEFS_LAYER0_START_OVER=1); fi
+  env "${start_over_setting[@]}" "${progress_settings[@]}" \
+    cargo test --release -p "$1" --test "$2" -- --include-ignored --exact "$3" --nocapture 2>&1 \
+    | tee "$4" \
+    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } \
+    | sed -u 's/^/    /'
+  return "${PIPESTATUS[0]}"
 }
 
 # report_manifest_differences <前一份清单> <后一份清单> <前一份的叫法> <后一份的叫法>：逐个列出两份清单里不同的文件，最多 20 个，另报总数。
@@ -226,24 +234,6 @@
   echo '                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'
 }
 
-# report_newest_other_marker <这一次的清单>：这批输入没有自己那一格时，拿 common-dir 里最近写的一格与这一次比，列出不同的文件；
-# 不带哈希的旧名字标记是分格之前写的，不再认，有就点名。
-report_newest_other_marker() {
-  local newest_marker newest_marker_manifest
-  newest_marker="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' -printf '%T@\t%p\n' 2>/dev/null | sort -rn | head -1 | cut -f2-)"
-  if [[ -n "$newest_marker" ]]; then
-    echo "       common-dir 里最近写的一格是 $(basename "$newest_marker")（跑完于 $(sed -n 's/^finished_utc=//p' "$newest_marker" | head -1)），与这一次的输入比："
-    newest_marker_manifest="$layer0_scratch_directory/newest-marker-manifest"
-    sed -n 's/^input_file //p' "$newest_marker" > "$newest_marker_manifest"
-    report_manifest_differences "$newest_marker_manifest" "$1" "那一格" "这一次"
-  else
-    echo "       common-dir 里一格全绿标记都没有。"
-  fi
-  if [[ -f "$full_green_marker_prefix" ]]; then
-    echo "       不带哈希的旧名字标记（$full_green_marker_prefix）是按输入哈希分格之前写的，不再认：它罩不到任何一批，可以删掉。"
-  fi
-}
-
 # run_quick_tier_of_stream <测试二进制> <流的名字>：快档跑一条流。判红打出路、返回 1；判绿把这条流的计数接到 quick_tier_report 后面。
 run_quick_tier_of_stream() {
   local quick_log passed_and_ignored passed_count ignored_count
@@ -267,166 +257,141 @@
   return 0
 }
 
-# ── 快档：两条流不标 ignored 的用例，再核对全绿标记 ─────────
+# ── 快档：两条流不标 ignored 的用例，再逐条核崩溃枚举用例这批输入那一格全绿标记 ─────────
 if [[ "$layer0_tier" == quick ]]; then
-  quick_manifest="$layer0_scratch_directory/quick-manifest"
-  write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest
-  full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"
   quick_tier_report=""
   run_quick_tier_of_stream first_transaction_step_seven_layer0 "第一个事务那条流" || exit 1
   run_quick_tier_of_stream second_transaction_step_zero_layer0 "两次发布那条流" || exit 1
-  if [[ ! -f "$full_green_marker_path" ]]; then
-    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入（哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件）没有层 0 全量的全绿标记（$full_green_marker_path）"
-    report_newest_other_marker "$quick_manifest"
-    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
-    print_staged_worktree_full_commands
-    exit 1
-  fi
-  marker_input_hash="$(sed -n 's/^input_hash=//p' "$full_green_marker_path" | head -1)"
-  if [[ "$marker_input_hash" != "$layer0_input_hash" ]]; then
-    echo "  ✗ 快档绿了（${quick_tier_report}），但这批输入那一格全绿标记里记的输入哈希（${marker_input_hash:-标记里读不到}）与这一次的（${layer0_input_hash}，${layer0_input_file_count} 个文件）不同：那一格被改过或拷错了"
-    marker_manifest="$layer0_scratch_directory/marker-manifest"
-    sed -n 's/^input_file //p' "$full_green_marker_path" > "$marker_manifest"
-    report_manifest_differences "$marker_manifest" "$quick_manifest" "那一格" "这一次"
-    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
-    print_staged_worktree_full_commands
-    exit 1
-  fi
-  marker_count_lines="$(grep -E '^(LAYER0|CHECKER|LAYER0B) ' "$full_green_marker_path")"
-  # 三种开头各数各的：只数总数时，LAYER0 那一行抄两遍、LAYER0B 一行都没有也凑得出三行
-  marker_layer0_line_total="$(grep -c '^LAYER0 ' <<< "$marker_count_lines")"
-  marker_checker_line_total="$(grep -c '^CHECKER ' <<< "$marker_count_lines")"
-  marker_layer0b_line_total="$(grep -c '^LAYER0B ' <<< "$marker_count_lines")"
-  if [[ "$marker_layer0_line_total" != 1 || "$marker_checker_line_total" != 1 || "$marker_layer0b_line_total" != 1 ]]; then
-    echo "  ✗ 这批输入那一格全绿标记的哈希对得上，计数行却不是 LAYER0 / CHECKER / LAYER0B 各恰好一行（LAYER0 ${marker_layer0_line_total} 行、CHECKER ${marker_checker_line_total} 行、LAYER0B ${marker_layer0b_line_total} 行）：标记不是 --full 写的，或被改过"
-    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
-    print_staged_worktree_full_commands
-    exit 1
-  fi
-  # 写这一格的 --full 本该把「不是全量」判红、不写标记；标记里仍有 exhaustive 不是 true 的，说明跑的那一份 54 号的判定被改过
-  marker_layer0_line="$(grep '^LAYER0 ' <<< "$marker_count_lines")"
-  marker_layer0b_line="$(grep '^LAYER0B ' <<< "$marker_count_lines")"
-  marker_streams_not_exhaustive=()
-  if ! grep -qE '^LAYER0 (.* )?exhaustive=true( |$)' <<< "$marker_layer0_line"; then marker_streams_not_exhaustive+=("LAYER0（第一个事务那条流）"); fi
-  if ! grep -qE '^LAYER0B (.* )?exhaustive=true( |$)' <<< "$marker_layer0b_line"; then marker_streams_not_exhaustive+=("LAYER0B（两次发布那条流）"); fi
-  if (( ${#marker_streams_not_exhaustive[@]} > 0 )); then
-    echo "  ✗ 这批输入那一格全绿标记里，${marker_streams_not_exhaustive[*]} 那一行不带 exhaustive=true：写它的那一趟 --full 没把「不是全量」判红"
-    printf '%s\n%s\n' "$marker_layer0_line" "$marker_layer0b_line" | sed 's/^/         /'
-    echo "     → 怎么办：这批输入的层 0 全量没在收尾跑过。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+  present_report="$layer0_scratch_directory/present-report"
+  missing_report="$layer0_scratch_directory/missing-report"
+  : > "$present_report"
+  : > "$missing_report"
+  missing_cases=()
+  for crash_case_row in "${crash_case_rows[@]}"; do
+    IFS=$'\t' read -r case_key _case_package _case_target _case_function <<< "$crash_case_row"
+    case_manifest="$layer0_scratch_directory/quick-manifest.${case_key#crash-case:}"
+    if ! write_crash_case_manifest "$case_key" "$case_manifest"; then
+      missing_cases+=("$case_key")
+      echo "       $case_key：算不出这批输入的指纹：$case_manifest_problem" >> "$missing_report"
+      continue
+    fi
+    if marker_check_output="$(python3 "$layer0_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$case_fingerprint" "$case_manifest")"; then
+      read -r _ok_word _marker_path marker_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
+      {
+        echo "    $case_key：那一格跑完于 ${marker_finished_utc}（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去别的测试目标独占的 ${case_excluded_count} 个）"
+        tail -n +2 <<< "$marker_check_output" | sed 's/^/      /'
+      } >> "$present_report"
+    else
+      missing_cases+=("$case_key")
+      {
+        echo "       $case_key（这批输入的指纹 ${case_fingerprint:0:16}…，${case_file_count} 个文件，减去别的测试目标独占的 ${case_excluded_count} 个）："
+        sed 's/^/         /' <<< "$marker_check_output"
+      } >> "$missing_report"
+    fi
+  done
+  if (( ${#missing_cases[@]} > 0 )); then
+    echo "  ✗ 快档绿了（${quick_tier_report}），但 ${#missing_cases[@]} 条崩溃枚举用例没有作数的全绿标记：${missing_cases[*]}"
+    cat "$missing_report"
+    legacy_marker_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green*' 2>/dev/null | grep -c .)"
+    if [[ "$legacy_marker_total" -gt 0 ]]; then
+      echo "       common-dir 里还有 ${legacy_marker_total} 格按整批输入分格的旧标记（singlefs-layer0-full-green*）：分成逐条用例之后不再认，可以删掉。"
+    fi
+    echo "     → 怎么办：这几条的输入自上一次判绿以来变了（或从来没跑过）。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>"
+    echo "                （与 gate.sh --staged 同一建法；它只跑没有作数标记的那几条，别的复用）："
     print_staged_worktree_full_commands
-    echo "                那一趟判「层 0 不是全量」就是这一批让枚举退化了，去 crash.rs 的 enumerate_layer0 看。"
     exit 1
   fi
-  marker_finished_utc="$(sed -n 's/^finished_utc=//p' "$full_green_marker_path" | head -1)"
-  echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：${quick_tier_report}"
-  echo "  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}），两条流都是 exhaustive=true：层 0 全量跑完于 ${marker_finished_utc}，标记里的计数行原样："
-  sed 's/^/      /' <<< "$marker_count_lines"
+  echo "  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
+  echo "  ✓ ${#crash_case_rows[@]} 条崩溃枚举用例的全绿标记都与各自这批输入的指纹相同（登记路径 ${layer0_input_paths_text}，逐条减去别的测试目标独占的测试文件），标记里的计数行原样："
+  cat "$present_report"
   exit 0
 fi
 
-# ── --full：记下开跑时的输入清单，跑完对一遍；开跑一格都不删，这一趟没写成标记就退出时才删这批输入那一格 ─────────
+# ── --full：逐条崩溃枚举用例照复用判定跑；那一格在就复用，不在才跑，判绿写那一格 ─────────
 full_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
-manifest_at_start="$layer0_scratch_directory/manifest-at-start"
-write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
-input_hash_at_start="$layer0_input_hash"
-full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
-# 这一趟判红（任何一处 exit）或被打断：同一批输入先绿后红，前一趟那一格不再作数，退出前删掉它；写成了就留着。
-# 例外是「跑的过程中输入变了」那一支：两条流读到的不一定是开跑那一批，这一趟说不出开跑那一批的好坏，不删那一格
-full_marker_written=0
-full_input_changed_during_run=0
-trap 'if [[ "$full_marker_written" != 1 && "$full_input_changed_during_run" != 1 ]]; then rm -f -- "${full_green_marker_path:?}"; fi; rm -rf -- "${layer0_scratch_directory:?}"' EXIT
-echo "  · --full 开跑（${full_started_utc}）：不删任何一格，这一趟判红才删这批输入那一格（跑的过程中输入变了的那一种红不删）；这一道的输入哈希 ${input_hash_at_start:0:16}…（${layer0_input_file_count} 个文件，登记路径 ${layer0_input_paths_text}，连同判它的 54 号与工具链）"
-
-log="$(mktemp)"
-if ! run_layer0_test_binary first_transaction_step_seven_layer0 "$log"; then
-  tail -40 "$log"
-  echo "  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）"
-  echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture"
-  echo "                oracle 报的第一条违例在断言消息里。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
-  rm -f "$log"
-  exit 1
-fi
-line="$(grep '^LAYER0 ' "$log" | head -1)"
-checker_line="$(grep -A1 '^LAYER0 ' "$log" | grep '^CHECKER ' | head -1)"
-worker_threads="$(worker_threads_of_full_run "$log" "$line")"
-rm -f "$log"
-if [[ -z "$line" ]]; then
-  echo "  ✗ 用例跑过了，却没打印 LAYER0 计数行"
-  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例要 println! 一行以 LAYER0 开头的计数（字段见该用例的文档注释）。"
-  exit 1
-fi
-if [[ -z "$checker_line" ]]; then
-  echo "  ✗ 用例跑过了，LAYER0 计数行的下一行却不是以 CHECKER 开头的逐条不变量行：成功句里「逐条不变量」那一句会是空话"
-  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例打完 LAYER0 行要紧跟着 println! checker_line(&tally) 那一行；"
-  echo "                两行之间插进了别的输出，就把 CHECKER 那一行挪回 LAYER0 行的正下方。"
-  exit 1
-fi
-if [[ "$line" != *"exhaustive=true"* ]]; then
-  echo "  ✗ 层 0 不是全量：$line"
-  echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0 看。"
-  exit 1
-fi
-worker_threads_are_acceptable "第一个事务那条流" "$worker_threads" || exit 1
-echo "  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；${worker_threads} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line#LAYER0 }"
-echo "  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：${checker_line#CHECKER }"
-# 第二个事务（发布 B）：取号 → 暖机 → A → B 整条流，多版本 oracle（里程碑「第二个事务」步 0 / 步 6 在 B 上的那一半）。
-log_b="$(mktemp)"
-if ! run_layer0_test_binary second_transaction_step_zero_layer0 "$log_b"; then
-  tail -40 "$log_b"
-  echo "  ✗ 两次发布那条流的层 0 用例判红（上面是 cargo test 的尾部）"
-  echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture"
-  echo "                多版本 oracle 报的第一条违例在断言消息里：实际走的根是哪一代就得读出那一代的内容。"
-  rm -f "$log_b"
-  exit 1
-fi
-line_b="$(grep '^LAYER0B ' "$log_b" | head -1)"
-worker_threads_b="$(worker_threads_of_full_run "$log_b" "$line_b")"
-rm -f "$log_b"
-if [[ -z "$line_b" ]]; then
-  echo "  ✗ 两次发布那条流的用例跑过了，却没打印 LAYER0B 计数行"
-  echo "     → 怎么办：second_transaction_step_zero_layer0.rs 里全量那条用例要 println! 一行以 LAYER0B 开头的计数。"
-  exit 1
-fi
-if [[ "$line_b" != *"exhaustive=true"* ]]; then
-  echo "  ✗ 两次发布那条流的层 0 不是全量：$line_b"
-  echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0_selecting_versions 看。"
-  exit 1
-fi
-worker_threads_are_acceptable "两次发布那条流" "$worker_threads_b" || exit 1
-echo "  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；${worker_threads_b} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line_b#LAYER0B }"
-
-# ── --full 判绿：跑完再算一次输入，与开跑时相同才写全绿标记 ─────────
-manifest_at_finish="$layer0_scratch_directory/manifest-at-finish"
-write_layer0_input_manifest "$manifest_at_finish" || fail_without_input_manifest
-if [[ "$layer0_input_hash" != "$input_hash_at_start" ]]; then
-  full_input_changed_during_run=1
-  echo "  ✗ 全量跑的过程中这一道的输入变了（开跑 ${input_hash_at_start:0:16}…，跑完 ${layer0_input_hash:0:16}…）：两条流读到的不一定是同一版，不写全绿标记；开跑那一批已有的那一格不删"
-  report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
-  echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
-  print_staged_worktree_full_commands
-  exit 1
-fi
+start_over_note="不带 --start-over：有进度文件就接着跑"
+if [[ "$layer0_start_over" == 1 ]]; then start_over_note="带 --start-over：进度文件整份丢掉、从头跑"; fi
+echo "  · --full 开跑（${full_started_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例逐条照复用判定跑（这批输入那一格全绿标记在就复用）；续跑的进度文件在 ${layer0_progress_root}/<输入指纹>/，${start_over_note}"
+red_cases=()
+green_cases=()
+reused_cases=()
+for crash_case_row in "${crash_case_rows[@]}"; do
+  IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
+  case_label="${case_key#crash-case:}"
+  manifest_at_start="$layer0_scratch_directory/manifest-at-start.$case_label"
+  if ! write_crash_case_manifest "$case_key" "$manifest_at_start"; then
+    echo "  ✗ $case_key：算不出这批输入的指纹：$case_manifest_problem"
+    echo "     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- ${layer0_input_paths_text} 看列不列得出文件，再跑 cargo -V && rustc -V；"
+    echo "                单跑 python3 research/scripts/admission.py crash-case-manifest <根> $case_key <清单文件> --toolchain --build-environment 看它报什么。"
+    red_cases+=("$case_key")
+    continue
+  fi
+  fingerprint_at_start="$case_fingerprint"
+  file_count_at_start="$case_file_count"
+  excluded_count_at_start="$case_excluded_count"
+  if marker_check_output="$(python3 "$layer0_admission_module" crash-case-marker-check "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start")"; then
+    read -r _ok_word reused_marker_path reused_finished_utc <<< "$(head -1 <<< "$marker_check_output")"
+    echo "  · $case_key 复用：这批输入（指纹 ${fingerprint_at_start:0:16}…）那一格全绿标记跑完于 ${reused_finished_utc}，这一趟不跑；要重跑就删掉 ${reused_marker_path}"
+    reused_cases+=("$case_key")
+    continue
+  fi
+  case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
+  case_log="$layer0_scratch_directory/log.$case_label"
+  echo "  · $case_key 开跑（${case_started_utc}；cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function；这批输入的指纹 ${fingerprint_at_start:0:16}…，${file_count_at_start} 个文件，减去别的测试目标独占的 ${excluded_count_at_start} 个）："
+  sed 's/^/      /' <<< "$marker_check_output"
+  if ! run_crash_case "$case_package" "$case_target" "$case_function" "$case_log" "$fingerprint_at_start"; then
+    tail -40 "$case_log"
+    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
+    echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
+    echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function --nocapture"
+    echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
+    red_cases+=("$case_key")
+    continue
+  fi
+  judged_lines_file="$layer0_scratch_directory/judged.$case_label"
+  if ! judge_output="$(python3 "$layer0_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file" \
+      --machine-cores "$machine_cores" --threads "$SINGLEFS_LAYER0_THREADS" --threads-origin "$threads_origin_word")"; then
+    delete_crash_case_marker "$case_key" "$fingerprint_at_start"
+    printf '%s\n' "$judge_output" | sed 's/^/       /'
+    echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
+    echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
+    echo "                不是全量去 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0 看；只起了 1 个线程看 Layer0Parallelism::from_environment 读没读到 SINGLEFS_LAYER0_THREADS，"
+    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh --full <根>。"
+    red_cases+=("$case_key")
+    continue
+  fi
+  manifest_at_finish="$layer0_scratch_directory/manifest-at-finish.$case_label"
+  if ! write_crash_case_manifest "$case_key" "$manifest_at_finish"; then
+    echo "  ✗ $case_key 跑完之后算不出这批输入的指纹：$case_manifest_problem；不写全绿标记"
+    echo "     → 怎么办：多半是跑的过程中有人删了登记路径下的文件或工具链坏了；等改动停下，在 HEAD + 暂存区的 worktree 里重跑 --full。"
+    red_cases+=("$case_key")
+    continue
+  fi
+  if [[ "$case_fingerprint" != "$fingerprint_at_start" ]]; then
+    echo "  ✗ $case_key 跑的过程中它的输入变了（开跑 ${fingerprint_at_start:0:16}…，跑完 ${case_fingerprint:0:16}…）：读到的不一定是同一版，不写全绿标记；开跑那一批已有的那一格不删"
+    report_manifest_differences "$manifest_at_start" "$manifest_at_finish" "开跑时" "跑完时"
+    echo "     → 怎么办：别在有人改这些路径的树里跑。暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full <它的根>（与 gate.sh --staged 同一建法）："
+    print_staged_worktree_full_commands
+    red_cases+=("$case_key")
+    continue
+  fi
+  threads_text="${judge_output//$'\n'/；}${judge_output:+；}SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核"
+  if ! record_output="$(python3 "$layer0_admission_module" crash-case-record "$ROOT" "$case_key" "$fingerprint_at_start" "$manifest_at_start" "$judged_lines_file" \
+      --files "$file_count_at_start" --excluded "$excluded_count_at_start" --started "$case_started_utc" --judged-root "$ROOT" --threads-text "$threads_text")"; then
+    echo "  ✗ $case_key 判绿，全绿标记却没写成：$record_output"
+    echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
+    red_cases+=("$case_key")
+    continue
+  fi
+  rmdir -- "$layer0_progress_root/$fingerprint_at_start" 2>/dev/null
+  echo "  ✓ $case_key 判绿（${threads_text}）：全绿标记写进 ${record_output}，记下的行原样："
+  sed 's/^/      /' "$judged_lines_file"
+  green_cases+=("$case_key")
+done
 full_finished_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
-marker_being_written="$full_green_marker_path.partial.$$"
-if ! {
-  echo "# 层 0 全量全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后写，按输入哈希分格，整轮门禁的快档按这批输入的哈希读这一格。不进工作树，别手改。"
-  echo "input_hash=$layer0_input_hash"
-  echo "input_file_count=$layer0_input_file_count"
-  echo "input_paths=$layer0_input_paths_text"
-  echo "started_utc=$full_started_utc"
-  echo "finished_utc=$full_finished_utc"
-  echo "judged_root=$ROOT"
-  echo "worker_threads=第一个事务那条流 ${worker_threads}、两次发布那条流 ${worker_threads_b}；SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核"
-  echo "$line"
-  echo "$checker_line"
-  echo "$line_b"
-  sed 's/^/input_file /' "$manifest_at_finish"
-} > "$marker_being_written" || ! mv -f -- "$marker_being_written" "$full_green_marker_path"; then
-  rm -f -- "${marker_being_written:?}"
-  echo "  ✗ 全量全绿，全绿标记却没写成（$full_green_marker_path）"
-  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full（没有标记，整轮门禁的快档会一直红）。"
+if (( ${#red_cases[@]} > 0 )); then
+  echo "  ✗ --full 有 ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条、复用 ${#reused_cases[@]} 条；开跑 ${full_started_utc}，跑完 ${full_finished_utc}）"
+  echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完暂存，再在 HEAD + 暂存区的 worktree 里跑 --full（这一趟判绿的与复用的那几条下一趟照样复用）。"
   exit 1
 fi
-full_marker_written=1
-marker_slot_total="$(find "$git_common_directory" -maxdepth 1 -type f -name 'singlefs-layer0-full-green.*' ! -name '*.partial.*' | grep -c .)"
-echo "  ✓ 全绿标记写进 $full_green_marker_path（输入哈希 ${layer0_input_hash:0:16}…，${layer0_input_file_count} 个文件；开跑 ${full_started_utc}，跑完 ${full_finished_utc}；common-dir 里现有 ${marker_slot_total} 格）"
+echo "  ✓ --full 跑完（开跑 ${full_started_utc}，跑完 ${full_finished_utc}）：${#crash_case_rows[@]} 条崩溃枚举用例，这一趟跑了判绿 ${#green_cases[@]} 条（${green_cases[*]:-无}），复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）"
--- a/research/scripts/admission.py
+++ b/research/scripts/admission.py
@@ -17,6 +17,19 @@
       environment=<仓内脚本>[:<参数>…]    不是前提，是门禁复用判定的一项输入：bash <根>/<脚本> <参数…> 打出来的东西
                                           （例 herd7 的版本）。不在 git 树里的东西靠它进复用判定，见 ①
       正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
+  崩溃枚举用例行（键是 crash-case:<名>，门禁 54 号逐条按复用判定跑、逐条记全绿标记）：路径写整个 crates/ 与 Cargo 清单、锁，
+    不按用例手列；算输入时自动减去别的测试目标独占的测试文件（见「崩溃枚举用例」一节）。第三列认四种：
+      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>
+      count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行，原样记进全绿标记
+      exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
+      threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）
+
+输入指纹里的构建环境（build_environment_lines；实验的指纹与 manifest / crash-case-manifest 带 --build-environment 时进清单）：
+  仓根与它往上每一层目录的 .cargo/config、.cargo/config.toml、rust-toolchain、rust-toolchain.toml，CARGO_HOME（没设取 ~/.cargo）下的
+  config、config.toml——在的才进，按内容进，名字里不带绝对路径（同一批内容在主工作区与临时 worktree 里算出同一个数）；
+  环境变量 RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、CARGO_PROFILE_*、CARGO_BUILD_*（CARGO_BUILD_JOBS 除外：只定编译并行度，
+  research/scripts/capped.sh 与 mutate.sh 会设它）、CARGO_TARGET_*_RUSTFLAGS、CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、
+  RUSTC_WORKSPACE_WRAPPER，设了的才进；RUSTC 设了的进它的值与 `$RUSTC -V`。登记路径下指向目录的符号链接按链接指向的目录里的文件计入。
 
 准入条件分三类，这里管前两类：
   ① 输入没变 ⇒ 不跑，沿用上次结论。门禁：登记的路径在 refs/sop/staged-green 那棵树与这一次的暂存树之间 git 说没变
@@ -36,11 +49,24 @@
   experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头）；
                                  退 77 输入自上次产物以来没变；退 3 前提没齐；退 2 登记或调用有错。说明一律打到 stderr
   paths <根> <键>                列出登记给这个键的路径（@ 引用已展开），一行一条
-  manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain]
+  manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain] [--build-environment]
                                  把这个键的输入逐文件清单（「sha256  路径」，按路径字节序）写进清单文件，末尾按参数次序
-                                 加「sha256  <名字>」行；stdout 打「<整张清单的 sha256> <文件数>」。与门禁 54 号
-                                 write_layer0_input_manifest 逐字节相同的写法见 --selftest 那一格
+                                 加「sha256  <名字>」行；stdout 打「<整张清单的 sha256> <文件数>」。另算一遍逐字节相同的写法见 --selftest 那一格
   keys <根>                      列出登记表里的实验键，一行一个
+  crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在），每条打一行
+                                 「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
+  crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--toolchain] [--build-environment]
+                                 这条用例的输入清单：登记路径下的文件减去别的测试目标独占的测试文件，末尾按参数次序加名字行，
+                                 最后一行是这条用例的登记行（键、路径与第三列）；stdout 打「<指纹> <文件数> <减去的文件数>」
+  crash-case-marker-check <根> <键> <指纹> <清单文件>
+                                 这批输入那一格全绿标记在不在、作不作数：退 0 作数（stdout 第一行「ok <标记路径> <跑完的时刻>」，
+                                 其后是标记里的计数行）；退 1 不作数（stdout 是原因，没有那一格时再比最近写的一格与这一次的清单）
+  crash-case-judge <根> <键> <日志> <记录行文件> --machine-cores <核数> --threads <线程数> --threads-origin explicit|default
+                                 判 --full 跑那一条的日志：退 0 判绿（记进标记的行写进记录行文件，stdout 是线程那一句）；退 1 判红（stdout 逐条原因）
+  crash-case-record <根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> --started <时刻> --judged-root <路径> --threads-text <一句>
+                                 判绿之后写那一格全绿标记（同目录临时文件写完再改名换上）；stdout 打标记路径
+  crash-case-marker-path <根> <键> <指纹>
+                                 那一格全绿标记的路径（判红时阶段删它）
   gate-preconditions <阶段所在的仓根> <阶段文件名>
                                  门禁开跑前判第三列的前提（command=、readwrite=、probe=）：退 0 齐了（stdout 一个字不打）；
                                  退 1 没齐、退 2 登记有错，两种都在 stdout 打 ✗、明细与出路，阶段照判红。
@@ -54,12 +80,17 @@
 强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。只越过 ①，不越过 ②；理由原样写进产物头。
 弄坏开关（只给自证用，证明那几格会红）：ADMISSION_BREAK=skip-unchanged（不比指纹）、skip-preconditions（不判前提）、
 raise-in-gate-reuse（门禁复用判定里抛异常，stage-must-run.sh 必须按要跑处理）、skip-gate-preconditions（门禁的前提不判）、
-ignore-gate-environment（门禁复用判定不比环境）。
+ignore-gate-environment（门禁复用判定不比环境）、skip-build-environment（构建环境不进指纹）、ignore-linked-directories（指向目录的
+符号链接照旧滤掉）、exclude-mentioned-test-targets（别的测试目标被代码点了名也减掉）、threads-by-worker-count（照旧只看
+worker_threads=1 判「只用了一个线程」，不看这一趟跑了几片）。
 
 管不到的：跑的是不是按今天的源码编出来的二进制（指纹按源码算，跑的是 target/ 里的旧二进制时两边对不上，
 replay.sh 与 cargo run 开跑前都会重编）；登记的路径少写了一条（那条输入变了不会放行，要靠强制开关，登记行本身进指纹，
 补登记之后自然放行）；产物头的指纹行是不是真由那一趟写的（由装置入口调本模块写，拷来的产物照样带着）。
+崩溃枚举用例减去的「别的测试目标独占的文件」认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、
+build.rs 之外的构建期代码按目录读 tests/；这两种会让那份文件被减掉而它其实被读了。
 """
+import glob
 import hashlib
 import os
 import re
@@ -68,6 +99,7 @@
 import sys
 import tempfile
 import time
+import tomllib
 
 REGISTRATION_TABLE = ".claude/gate.d/stage-inputs.tsv"
 REPLAY_SCRIPT = "research/scripts/replay.sh"
@@ -95,6 +127,26 @@
 REPLAY_ROW_FORM = re.compile(r"^(E[0-9]+[A-Z0-9]*)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$")
 GATE_CONDITION_FORM = re.compile(r"^(?P<kind>command|readwrite|probe|environment)=(?P<value>\S+)$")
 
+# 崩溃枚举用例（门禁 54 号逐条跑、逐条记全绿标记）
+CRASH_CASE_KEY_PREFIX = "crash-case:"
+CRASH_CASE_KEY_FORM = re.compile(r"^crash-case:[a-z0-9][a-z0-9-]*$")
+CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads)=(?P<value>\S+)$")
+CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
+COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
+CRASH_CASE_MARKER_PREFIX = "singlefs-crash-case-green."
+LAYER0_PARALLEL_FINISHED_PREFIX = "LAYER0_PARALLEL_FINISHED "
+PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
+MARKER_DIFFERENCES_LISTED_AT_MOST = 20
+
+# 构建环境（build_environment_lines）：进指纹的配置文件名与环境变量
+CARGO_CONFIGURATION_FILE_NAMES = ("config", "config.toml")
+TOOLCHAIN_FILE_NAMES = ("rust-toolchain", "rust-toolchain.toml")
+BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
+BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_PROFILE_"), re.compile(r"^CARGO_BUILD_"),
+                                    re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))
+# 名字对得上而不进指纹的：它们不改编出来的东西，而本仓的包装会设它们，进了指纹同一批输入在包装里外算出两个数、全绿标记永远对不上
+BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT = {"CARGO_BUILD_JOBS": "只定编译并行度；research/scripts/capped.sh 与 mutate.sh 会设它"}
+
 
 class RegistrationError(Exception):
     """登记表或调用写错了：退 2，出路是改登记表那一行或改调用。"""
@@ -116,6 +168,10 @@
         """进指纹的只有键与路径：准入条件判的是「能不能跑」，不是「算什么」，改它不该让没变的输入放行。"""
         return f"{self.key}\t{' '.join(self.input_paths)}\n"
 
+    def text_with_conditions(self):
+        """崩溃枚举用例的登记行进指纹时连第三列一起进：第三列定的是跑哪一条用例、日志怎么判，改了前一趟的结论就不再作数。"""
+        return f"{self.key}\t{' '.join(self.input_paths)}\t{' '.join(self.conditions)}\n"
+
 
 def break_is_set(switch_name):
     return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")
@@ -221,12 +277,22 @@
     problems = []
     for row in rows:
         is_experiment = bool(EXPERIMENT_KEY_FORM.match(row.key))
-        if not is_experiment and not row.key.endswith(".sh"):
-            problems.append(f"第 {row.line_number} 行：键 {row.key} 既不是门禁阶段文件名（*.sh）也不是实验键（E<号>[/<方式>]）")
+        is_crash_case = bool(CRASH_CASE_KEY_FORM.match(row.key))
+        if not is_experiment and not is_crash_case and not row.key.endswith(".sh"):
+            problems.append(f"第 {row.line_number} 行：键 {row.key} 既不是门禁阶段文件名（*.sh）、实验键（E<号>[/<方式>]），"
+                            "也不是崩溃枚举用例（crash-case:<小写字母、数字、->）")
         if not row.input_paths:
             problems.append(f"第 {row.line_number} 行：{row.key} 没有登记路径")
         if not is_experiment and any(path.startswith("@") for path in row.input_paths):
-            problems.append(f"第 {row.line_number} 行：门禁行 {row.key} 用了 @ 引用，门禁的复用判定还不展开它")
+            problems.append(f"第 {row.line_number} 行：{row.key} 用了 @ 引用，门禁与崩溃枚举用例的判定还不展开它")
+        if is_crash_case:
+            try:
+                crash_case = parse_crash_case(row)
+                if root is not None:
+                    crash_case_test_files(root, crash_case)
+            except RegistrationError as error:
+                problems.append(f"第 {row.line_number} 行：{error}")
+            continue
         for token in row.conditions:
             try:
                 if is_experiment:
@@ -244,6 +310,13 @@
                 problems.append(f"第 {row.line_number} 行：{error}")
             for input_path in self_invalidating_input_paths(row.input_paths):
                 problems.append(f"第 {row.line_number} 行：实验 {row.key} 登记了 {input_path}，它罩住存产物那一步要改的文件，新产物一存进来指纹就对不上自己")
+    crash_case_lines = {}
+    for row in rows:
+        if CRASH_CASE_KEY_FORM.match(row.key):
+            crash_case_lines.setdefault(row.key, []).append(row.line_number)
+    for key, line_numbers in crash_case_lines.items():
+        if len(line_numbers) > 1:
+            problems.append(f"第 {'、'.join(str(number) for number in line_numbers)} 行：崩溃枚举用例 {key} 登记了 {len(line_numbers)} 行，一条用例只许一行")
     return problems
 
 
@@ -266,14 +339,43 @@
 
 
 def listed_input_files(root, input_paths):
-    """git 眼里这些路径下磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的），按字节序去重排好。"""
+    """git 眼里这些路径下磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的），按字节序去重排好。
+    git 把指向目录的符号链接当一个文件列出来：它指向的目录里的文件按「链接路径/目录里的相对路径」逐个计入（绕回自己的只走一遍）；
+    指不到东西的链接照旧不计。"""
     completed = subprocess.run(
         ["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],
         capture_output=True)
     if completed.returncode != 0:
         raise InputManifestError(f"git ls-files 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()}")
     names = sorted({name for name in completed.stdout.split(b"\0") if name})
-    return [name for name in names if os.path.isfile(os.path.join(os.fsencode(root), name))]
+    root_bytes = os.fsencode(root)
+    files = []
+    for name in names:
+        full_path = os.path.join(root_bytes, name)
+        if os.path.isfile(full_path):
+            files.append(name)
+        elif os.path.islink(full_path) and os.path.isdir(full_path) and not break_is_set("ignore-linked-directories"):
+            files.extend(files_under_linked_directory(root_bytes, name))
+    return sorted(set(files))
+
+
+def files_under_linked_directory(root_bytes, link_name):
+    """指向目录的符号链接 link_name（仓根起）底下的普通文件，名字写成「链接路径/相对路径」；目录里再有绕回去的链接，同一个目录只走一遍。"""
+    link_path = os.path.join(root_bytes, link_name)
+    walked_directories = set()
+    files = []
+    for directory, subdirectories, file_names in os.walk(link_path, followlinks=True):
+        real_directory = os.path.realpath(directory)
+        if real_directory in walked_directories:
+            subdirectories[:] = []
+            continue
+        walked_directories.add(real_directory)
+        subdirectories.sort()
+        for file_name in sorted(file_names):
+            full_path = os.path.join(directory, file_name)
+            if os.path.isfile(full_path):
+                files.append(link_name + b"/" + os.path.relpath(full_path, link_path))
+    return files
 
 
 def unlisted_input_paths(root, input_paths):
@@ -282,7 +384,7 @@
 
 
 def toolchain_line(root):
-    """与门禁 54 号同一种写法：`cargo -V && rustc -V` 原样输出去掉末尾换行、再补一个换行的 sha256，名字 <工具链：…>。"""
+    """`cargo -V && rustc -V` 原样输出去掉末尾换行、再补一个换行的 sha256，名字 <工具链：…>（门禁 54 号的输入指纹也是这一行）。"""
     outputs = []
     for command in (["cargo", "-V"], ["rustc", "-V"]):
         try:
@@ -299,11 +401,69 @@
     return (name, versions + b"\n")
 
 
+def read_configuration_file(path):
+    try:
+        with open(path, "rb") as handle:
+            return handle.read()
+    except OSError as error:
+        raise InputManifestError(f"读不了构建环境里的 {path}：{error}") from error
+
+
+def build_environment_lines(root, environment=None):
+    """进指纹的构建环境（写法见文件头「输入指纹里的构建环境」）：[(名字, 内容)]，一样都没有时是空表（清单与不带它时逐字节相同）。
+    名字里只写是哪一类、不写绝对路径：同一批内容在主工作区与临时 worktree 里要算出同一个数。
+    cargo 按当前目录往上一层层读配置、最后读 CARGO_HOME 的；CARGO_HOME 那一份同时是某一层的，只按 CARGO_HOME 算一次。"""
+    if break_is_set("skip-build-environment"):
+        return []
+    environment = os.environ if environment is None else environment
+    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
+    cargo_home_files = [os.path.join(cargo_home, name) for name in CARGO_CONFIGURATION_FILE_NAMES]
+    cargo_home_real_paths = {os.path.realpath(path) for path in cargo_home_files}
+    lines = []
+    directory = os.path.realpath(root)
+    while True:
+        for name in CARGO_CONFIGURATION_FILE_NAMES:
+            path = os.path.join(directory, ".cargo", name)
+            if os.path.isfile(path) and os.path.realpath(path) not in cargo_home_real_paths:
+                lines.append((f"<构建环境：目录层级里的 .cargo/{name}>", read_configuration_file(path)))
+        for name in TOOLCHAIN_FILE_NAMES:
+            path = os.path.join(directory, name)
+            if os.path.isfile(path):
+                lines.append((f"<构建环境：目录层级里的 {name}>", read_configuration_file(path)))
+        parent = os.path.dirname(directory)
+        if parent == directory:
+            break
+        directory = parent
+    for path in cargo_home_files:
+        if os.path.isfile(path):
+            lines.append((f"<构建环境：CARGO_HOME 下的 {os.path.basename(path)}>", read_configuration_file(path)))
+    for variable in sorted(environment):
+        if variable in BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT:
+            continue
+        if variable in BUILD_ENVIRONMENT_EXACT_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS):
+            lines.append((f"<构建环境：环境变量 {variable}>", environment[variable].encode("utf-8", "surrogateescape")))
+    if "RUSTC" in environment:
+        rustc = environment["RUSTC"]
+        try:
+            completed = subprocess.run([rustc, "-V"], cwd=root, capture_output=True, env=dict(environment))
+        except OSError as error:
+            raise InputManifestError(f"RUSTC={rustc} 起不来：{error}") from error
+        if completed.returncode != 0 or not completed.stdout.strip():
+            raise InputManifestError(f"RUSTC={rustc} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
+        lines.append(("<构建环境：RUSTC 的值与它的 -V>", rustc.encode("utf-8", "surrogateescape") + b"\n" + completed.stdout))
+    return lines
+
+
 def input_manifest(root, input_paths, named_lines):
     """逐文件清单加末尾几行「sha256  <名字>」；返回 (清单字节, 整张的 sha256, 文件数)。"""
     listed_files = listed_input_files(root, input_paths)
     if not listed_files:
         raise InputManifestError(f"登记的路径（{' '.join(input_paths)}）下 git 一个文件都列不出来")
+    return manifest_of_files(root, listed_files, named_lines)
+
+
+def manifest_of_files(root, listed_files, named_lines):
+    """给定的文件（仓根起、按字节序排好）逐个「sha256  路径」，再加末尾几行「sha256  <名字>」；返回 (清单字节, 整张的 sha256, 文件数)。"""
     manifest = bytearray()
     for name in listed_files:
         try:
@@ -317,10 +477,10 @@
 
 
 def experiment_fingerprint(root, rows, key):
-    """实验的输入指纹：登记路径下的逐文件清单 + 登记行本身 + 工具链。返回 (sha256, 文件数, 路径)。"""
+    """实验的输入指纹：登记路径下的逐文件清单 + 登记行本身 + 工具链 + 构建环境。返回 (sha256, 文件数, 路径)。"""
     input_paths, contributing_rows = resolve_input_paths(rows, key)
     registration_text = "".join(row.fingerprint_text() for row in contributing_rows).encode("utf-8", "surrogateescape")
-    named_lines = [(f"<登记行：{key}>", registration_text), toolchain_line(root)]
+    named_lines = [(f"<登记行：{key}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
     _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
     return fingerprint, file_count, input_paths
 
@@ -790,6 +950,461 @@
     return EXIT_GATE_MUST_RUN, f"这几条路径与上次整轮全绿那棵树不同：{changed}"
 
 
+# ── 崩溃枚举用例（门禁 54 号：逐条按复用判定跑、逐条记全绿标记）────────────────
+#
+# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去别的测试目标独占的测试文件，
+# 再加判它的 54 号、工具链、构建环境与这条用例的登记行。别的测试目标 X 独占的文件：它所在的包没有 build.rs、没写 [[test]]、
+# package.autotests 与 package.build，X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录），而 X 的名字不以整词
+# 出现在别处任何 .rs（注释去掉之后）与 Cargo.toml 里——`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、
+# 运行期按字面路径读它，都让它留在输入里。登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、
+# build.rs、新拆出的 crate）默认就在输入里。
+
+class CrashCase:
+    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines):
+        self.row = row
+        self.key = row.key
+        self.name = row.key[len(CRASH_CASE_KEY_PREFIX):]
+        self.package = package
+        self.target = target
+        self.function = function
+        self.count_lines = count_lines
+        self.exhaustive_lines = exhaustive_lines
+        self.thread_lines = thread_lines
+
+    def test_text(self):
+        return f"{self.package}:{self.target}:{self.function}"
+
+
+def parse_crash_case(row):
+    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=，都抛 RegistrationError。"""
+    tests, count_lines, exhaustive_lines, thread_lines = [], [], [], []
+    for token in row.conditions:
+        match = CRASH_CASE_CONDITION_FORM.match(token)
+        if not match:
+            raise RegistrationError(f"崩溃枚举用例 {row.key} 的第三列认不出 {token}（只认 test=<包>:<测试目标>:<用例函数>、"
+                                    "count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>）")
+        kind, value = match.group("kind"), match.group("value")
+        if kind == "test":
+            test = CRASH_CASE_TEST_FORM.match(value)
+            if not test:
+                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token} 要写成 test=<包>:<测试目标>:<用例函数>（字母、数字、_，包名另许 -）")
+            tests.append(test)
+            continue
+        if not COUNT_LINE_PREFIX_FORM.match(value):
+            raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：前缀只许大写字母、数字、_，以字母开头")
+        {"count-line": count_lines, "exhaustive": exhaustive_lines, "threads": thread_lines}[kind].append(value)
+    if len(tests) != 1:
+        raise RegistrationError(f"崩溃枚举用例 {row.key} 要恰好一条 test=，实际 {len(tests)} 条")
+    if len(set(count_lines)) != len(count_lines):
+        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 count-line= 有重复的前缀")
+    unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]
+    if unregistered:
+        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 exhaustive= / threads= 点名的 {' '.join(unregistered)} 没有登记成 count-line=")
+    test = tests[0]
+    return CrashCase(row, test.group("package"), test.group("target"), test.group("function"),
+                     count_lines, exhaustive_lines, thread_lines)
+
+
+def crash_cases_of(rows):
+    """登记表里全部崩溃枚举用例，照登记表的次序；一行写错抛 RegistrationError。"""
+    return [parse_crash_case(row) for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]
+
+
+def crash_case_of_key(rows, key):
+    own_rows = rows_of_key(rows, key)
+    if not CRASH_CASE_KEY_FORM.match(key) or len(own_rows) != 1:
+        raise RegistrationError(f"{REGISTRATION_TABLE} 里崩溃枚举用例 {key} 要恰好一行，实际 {len(own_rows)} 行")
+    return parse_crash_case(own_rows[0])
+
+
+def manifest_sections(path):
+    try:
+        with open(path, "rb") as handle:
+            return tomllib.load(handle)
+    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
+        return None
+
+
+def workspace_package_directories(root):
+    """仓根 Cargo.toml 的工作区成员（members 按 glob 展开；仓根自己是包也算）：包名 → 仓根起的目录。"""
+    root_sections = manifest_sections(os.path.join(root, "Cargo.toml")) or {}
+    directories = []
+    if "package" in root_sections:
+        directories.append(".")
+    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
+        for directory in sorted(glob.glob(os.path.join(root, pattern))):
+            directories.append(os.path.relpath(directory, root))
+    packages = {}
+    for directory in directories:
+        name = ((manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}).get("package") or {}).get("name")
+        if name:
+            packages[name] = os.path.normpath(directory)
+    return packages
+
+
+def crash_case_test_files(root, case):
+    """这条用例的测试目标的源文件（仓根起）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs。
+    包不是工作区成员、目标不在、用例函数在目标里找不到，都抛 RegistrationError。"""
+    package_directory = workspace_package_directories(root).get(case.package)
+    if package_directory is None:
+        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
+    single_file = os.path.join(package_directory, "tests", case.target + ".rs")
+    directory_target = os.path.join(package_directory, "tests", case.target)
+    if os.path.isfile(os.path.join(root, single_file)):
+        files = [single_file]
+    elif os.path.isfile(os.path.join(root, directory_target, "main.rs")):
+        files = sorted(os.path.relpath(path, root)
+                       for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True))
+    else:
+        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
+    function_form = re.compile(r"\bfn\s+" + re.escape(case.function) + r"\s*\(")
+    for relative in files:
+        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
+            if function_form.search(rust_code_without_comments(handle.read())):
+                return files
+    raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到（改了名？）")
+
+
+def rust_code_without_comments(text):
+    """去掉 Rust 源码里的注释（行注释、可嵌套的块注释），字符串、原始字符串、字符字面量原样留着：判「别处有没有代码点名一个测试目标」时，
+    注释里提到它不算，字符串里写它的路径（include_str!、#[path]、运行期按路径读）算。"""
+    pieces = []
+    position = 0
+    length = len(text)
+    while position < length:
+        if text.startswith("//", position):
+            line_end = text.find("\n", position)
+            position = length if line_end < 0 else line_end
+            continue
+        if text.startswith("/*", position):
+            depth = 0
+            while position < length:
+                if text.startswith("/*", position):
+                    depth += 1
+                    position += 2
+                elif text.startswith("*/", position):
+                    depth -= 1
+                    position += 2
+                    if depth == 0:
+                        break
+                else:
+                    position += 1
+            pieces.append(" ")
+            continue
+        character = text[position]
+        previous_is_identifier = position > 0 and (text[position - 1].isalnum() or text[position - 1] == "_")
+        raw_string = RAW_STRING_START.match(text, position)
+        if raw_string and not previous_is_identifier:
+            closing = '"' + raw_string.group("hashes")
+            end = text.find(closing, raw_string.end())
+            end = length if end < 0 else end + len(closing)
+            pieces.append(text[position:end])
+            position = end
+            continue
+        if character == '"':
+            end = position + 1
+            while end < length and text[end] != '"':
+                end += 2 if text[end] == "\\" else 1
+            end = min(end + 1, length)
+            pieces.append(text[position:end])
+            position = end
+            continue
+        if character == "'":
+            if text.startswith("\\", position + 1):
+                closing = text.find("'", position + 3)
+                end = length if closing < 0 else closing + 1
+                pieces.append(text[position:end])
+                position = end
+                continue
+            if position + 2 < length and text[position + 2] == "'":
+                pieces.append(text[position:position + 3])
+                position += 3
+                continue
+        pieces.append(character)
+        position += 1
+    return "".join(pieces)
+
+
+RAW_STRING_START = re.compile(r'b?r(?P<hashes>#*)"')
+
+
+def whole_word_form(word):
+    return re.compile(r"(?<![A-Za-z0-9_])" + re.escape(word) + r"(?![A-Za-z0-9_])")
+
+
+def test_targets_of_package(package_directory, listed_names):
+    """包目录下 tests/ 里 cargo 自动认的集成测试目标：目标名 → 它的文件（tests/<名>.rs；tests/<名>/main.rs 那种目标是 tests/<名>/ 下全部文件）。
+    package_directory 与 listed_names 都是仓根起的 str 路径。"""
+    tests_prefix = ("" if package_directory == "." else package_directory + "/") + "tests/"
+    listed_set = set(listed_names)
+    targets = {}
+    for name in listed_names:
+        if not name.startswith(tests_prefix):
+            continue
+        rest = name[len(tests_prefix):]
+        parts = rest.split("/")
+        if len(parts) == 1 and rest.endswith(".rs"):
+            targets.setdefault(rest[:-3], []).append(name)
+        elif len(parts) >= 2 and tests_prefix + parts[0] + "/main.rs" in listed_set:
+            targets.setdefault(parts[0], []).append(name)
+    return targets
+
+
+def package_keeps_every_test_file(root, package_directory, listed_set):
+    """包里有 build.rs、写了 [[test]]、package.autotests 或 package.build：测试目标怎么编、读什么由它们另定，这一个包的测试文件一份都不减。"""
+    prefix = "" if package_directory == "." else package_directory + "/"
+    sections = manifest_sections(os.path.join(root, package_directory, "Cargo.toml")) or {}
+    package = sections.get("package") or {}
+    return (prefix + "build.rs" in listed_set or "test" in sections
+            or "autotests" in package or "build" in package)
+
+
+def files_exclusive_to_other_test_targets(root, listed_files, own_package_directory, own_target):
+    """listed_files（bytes，仓根起）里别的测试目标独占的文件（bytes 的集合）：判法见本节开头的说明。"""
+    listed_names = [os.fsdecode(name) for name in listed_files]
+    listed_set = set(listed_names)
+    package_directories = sorted({os.path.dirname(name) or "." for name in listed_names
+                                  if os.path.basename(name) == "Cargo.toml"
+                                  and "package" in (manifest_sections(os.path.join(root, name)) or {})})
+    candidates = {}
+    for package_directory in package_directories:
+        if package_keeps_every_test_file(root, package_directory, listed_set):
+            continue
+        for target, files in test_targets_of_package(package_directory, listed_names).items():
+            if (os.path.normpath(package_directory), target) != (os.path.normpath(own_package_directory), own_target):
+                candidates[(package_directory, target)] = files
+    code_texts = {}
+    for name in listed_names:
+        if name.endswith(".rs") or os.path.basename(name) == "Cargo.toml":
+            try:
+                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as handle:
+                    text = handle.read()
+            except OSError as error:
+                raise InputManifestError(f"读不了 {name}：{error}") from error
+            code_texts[name] = rust_code_without_comments(text) if name.endswith(".rs") else text
+    exclusive = set()
+    for (_package_directory, target), files in candidates.items():
+        own_files = set(files)
+        named_elsewhere = any(whole_word_form(target).search(text) for name, text in code_texts.items() if name not in own_files)
+        if named_elsewhere and not break_is_set("exclude-mentioned-test-targets"):
+            continue
+        exclusive.update(os.fsencode(name) for name in files)
+    return exclusive
+
+
+def crash_case_manifest(root, case, named_lines):
+    """这条用例的输入清单：返回 (清单字节, 指纹, 文件数, 减去的文件 [bytes])。末尾在 named_lines 之后加这条用例的登记行（连第三列）。"""
+    listed_files = listed_input_files(root, case.row.input_paths)
+    if not listed_files:
+        raise InputManifestError(f"{case.key} 登记的路径（{' '.join(case.row.input_paths)}）下 git 一个文件都列不出来")
+    package_directory = workspace_package_directories(root).get(case.package)
+    if package_directory is None:
+        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
+    excluded = files_exclusive_to_other_test_targets(root, listed_files, package_directory, case.target)
+    kept = [name for name in listed_files if name not in excluded]
+    registration = (f"<登记行：{case.key}>", case.row.text_with_conditions().encode("utf-8", "surrogateescape"))
+    manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + [registration])
+    return manifest, fingerprint, file_count, sorted(excluded)
+
+
+def git_common_directory(root):
+    return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
+    common_directory = output.strip() if return_code == 0 else ""
+    return common_directory or None
+
+
+def crash_case_marker_path(root, case, fingerprint):
+    """那一格全绿标记：git common-dir 里「前缀 + 用例名 + . + 指纹」，不进工作树，各 worktree 共用一份；取不到 common-dir 返回 None。"""
+    common_directory = git_common_directory(root)
+    if common_directory is None:
+        return None
+    return os.path.join(common_directory, f"{CRASH_CASE_MARKER_PREFIX}{case.name}.{fingerprint}")
+
+
+def fields_of_line(line):
+    return dict(token.split("=", 1) for token in line.split() if "=" in token)
+
+
+def judge_worker_threads(prefix, count_line, log_lines, machine_cores, threads_explicitly_one):
+    """按计数行的 states= 找那一行 LAYER0_PARALLEL_FINISHED，判工作线程：返回 (原因或 None, 一句说明)。
+    判红只在这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、线程数没显式设成 1 时：全部片从进度文件读回时起 0 个线程，
+    只剩 1 片要跑时最多起 1 个，这两种都不是「线程数没传进去」。"""
+    states = fields_of_line(count_line).get("states")
+    finished = [line for line in log_lines if line.startswith(f"{LAYER0_PARALLEL_FINISHED_PREFIX}states={states} ")] if states else []
+    if not finished:
+        return (f"{prefix} 那一行（states={states or '读不到'}）找不到状态数对得上的 LAYER0_PARALLEL_FINISHED 行，判不出起了几个工作线程"
+                "（全量要经 crates/singlefs-harness/src/crash.rs 的 enumerate_layer0_in_state_slices 跑）"), ""
+    fields = fields_of_line(finished[0])
+    try:
+        worker_threads = int(fields["worker_threads"])
+        slices = int(fields["slices"])
+        resumed_slices = int(fields["resumed_slices"])
+        freshly_run_slices = int(fields["freshly_run_slices"])
+    except (KeyError, ValueError):
+        return (f"LAYER0_PARALLEL_FINISHED 那一行缺 worker_threads= / slices= / resumed_slices= / freshly_run_slices=，或不是整数："
+                f"{finished[0]}"), ""
+    if resumed_slices + freshly_run_slices != slices:
+        return (f"LAYER0_PARALLEL_FINISHED 那一行读回的片 {resumed_slices} + 这一趟跑的片 {freshly_run_slices} ≠ 总片数 {slices}："
+                f"{finished[0]}"), ""
+    if freshly_run_slices > 0 and worker_threads == 0:
+        return f"这一趟跑了 {freshly_run_slices} 片，却报起了 0 个工作线程：{finished[0]}", ""
+    if break_is_set("threads-by-worker-count"):
+        judged_on_one_thread = worker_threads == 1
+    else:
+        judged_on_one_thread = freshly_run_slices >= 2 and worker_threads == 1
+    if judged_on_one_thread and machine_cores > 1 and not threads_explicitly_one:
+        return (f"本机 {machine_cores} 核、线程数没显式设成 1，这一趟跑了 {freshly_run_slices} 片却只起了 1 个工作线程"
+                f"（多半是线程数没传进去）：{finished[0]}"), ""
+    if freshly_run_slices == 0:
+        note = f"{prefix}：全部 {slices} 片从进度文件读回，这一趟没起工作线程"
+    else:
+        note = f"{prefix}：{worker_threads} 个工作线程跑了 {freshly_run_slices} 片、从进度文件读回 {resumed_slices} 片（共 {slices} 片）"
+    return None, note
+
+
+def judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one):
+    """判 --full 跑一条用例的日志：返回 (原因的清单, 要记进全绿标记的行, 线程那几句)。原因的清单为空才算判绿。"""
+    log_lines = log_text.split("\n")
+    problems, recorded_lines, thread_notes = [], [], []
+    results = [line for line in log_lines if line.startswith("test result: ")]
+    if len(results) != 1 or not PASSED_ONE_TEST_FORM.match(results[0]):
+        problems.append(f"读不到恰好一行「test result: ok. 1 passed; 0 failed; …」：过滤到 {case.function} 应当恰好跑一条用例，"
+                        f"读到 {len(results)} 行 test result（{'｜'.join(results[:3]) or '一行都没有'}）")
+    else:
+        recorded_lines.append("test_result=" + results[0])
+    count_line_of_prefix = {}
+    for prefix in case.count_lines:
+        found = [line for line in log_lines if line.startswith(prefix + " ")]
+        if len(found) != 1:
+            problems.append(f"以「{prefix} 」开头的计数行要恰好一行，读到 {len(found)} 行")
+            continue
+        count_line_of_prefix[prefix] = found[0]
+        recorded_lines.append(found[0])
+    for prefix in case.exhaustive_lines:
+        line = count_line_of_prefix.get(prefix)
+        if line is not None and fields_of_line(line).get("exhaustive") != "true":
+            problems.append(f"{prefix} 那一行不带 exhaustive=true，不是全量（枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)）：{line}")
+    for prefix in case.thread_lines:
+        line = count_line_of_prefix.get(prefix)
+        if line is None:
+            continue
+        problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one)
+        if problem:
+            problems.append(problem)
+        else:
+            thread_notes.append(note)
+    recorded_lines += ["parallel_finished=" + line for line in log_lines if line.startswith(LAYER0_PARALLEL_FINISHED_PREFIX)]
+    return problems, recorded_lines, thread_notes
+
+
+def read_crash_case_marker(path):
+    """全绿标记的 key=value 行、原样的计数行与 input_file 行；读不了返回 None。"""
+    try:
+        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
+            text = handle.read()
+    except OSError:
+        return None
+    marker = {"fields": {}, "raw_lines": [], "input_files": []}
+    for line in text.split("\n"):
+        if not line or line.startswith("#"):
+            continue
+        if line.startswith("input_file "):
+            marker["input_files"].append(line[len("input_file "):])
+        elif re.match(r"^[a-z_]+=", line):
+            name, value = line.split("=", 1)
+            marker["fields"].setdefault(name, value)
+        else:
+            marker["raw_lines"].append(line)
+    return marker
+
+
+def crash_case_marker_problems(case, fingerprint, marker):
+    """标记内容作不作数：返回原因的清单（空表是作数）。判的与 --full 写它之前判日志的是同一组：换了判法、被手改过、拷错了都在这里现形。"""
+    problems = []
+    fields = marker["fields"]
+    if fields.get("input_hash") != fingerprint:
+        problems.append(f"标记里记的输入指纹（{fields.get('input_hash', '读不到')}）与这一次的（{fingerprint}）不同：那一格被改过或拷错了")
+    if fields.get("case") != case.key:
+        problems.append(f"标记里记的用例（{fields.get('case', '读不到')}）不是 {case.key}")
+    if not PASSED_ONE_TEST_FORM.match(fields.get("test_result", "")):
+        problems.append(f"标记里没有「test result: ok. 1 passed; 0 failed; …」那一行（{fields.get('test_result', '读不到')}）")
+    for prefix in case.count_lines:
+        found = [line for line in marker["raw_lines"] if line.startswith(prefix + " ")]
+        if len(found) != 1:
+            problems.append(f"标记里以「{prefix} 」开头的计数行不是恰好一行（{len(found)} 行）：标记不是 --full 写的，或被改过")
+        elif prefix in case.exhaustive_lines and fields_of_line(found[0]).get("exhaustive") != "true":
+            problems.append(f"标记里 {prefix} 那一行不带 exhaustive=true：写它的那一趟 --full 没把「不是全量」判红（{found[0]}）")
+    return problems
+
+
+def manifest_differences(earlier_lines, later_lines, earlier_name, later_name):
+    """两份「sha256  名字」清单里不同的名字，按字节序排；返回说明句的清单。"""
+    earlier = {line[66:]: line[:64] for line in earlier_lines if len(line) > 66}
+    later = {line[66:]: line[:64] for line in later_lines if len(line) > 66}
+    differences = []
+    for name in sorted(set(earlier) | set(later)):
+        if name not in earlier:
+            differences.append(f"只在{later_name}里：{name}")
+        elif name not in later:
+            differences.append(f"只在{earlier_name}里：{name}")
+        elif earlier[name] != later[name]:
+            differences.append(f"内容不同：{name}")
+    return differences
+
+
+def check_crash_case_marker(root, case, fingerprint, manifest_text):
+    """这批输入那一格全绿标记作不作数：返回 (作数?, 说明行)。作数时说明行第一行是「ok <路径> <跑完的时刻>」，其后是标记里的计数行；
+    不作数时是原因，没有那一格时再拿这条用例最近写的一格与这一次的清单比。"""
+    marker_path = crash_case_marker_path(root, case, fingerprint)
+    if marker_path is None:
+        return False, [f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处读"]
+    marker = read_crash_case_marker(marker_path)
+    if marker is None:
+        lines = [f"这批输入（指纹 {fingerprint[:16]}…）没有全绿标记：{marker_path}"]
+        others = sorted(glob.glob(os.path.join(os.path.dirname(marker_path), f"{CRASH_CASE_MARKER_PREFIX}{case.name}.*")),
+                        key=os.path.getmtime, reverse=True)
+        others = [path for path in others if ".partial." not in os.path.basename(path)]
+        if not others:
+            lines.append(f"common-dir 里 {case.key} 一格全绿标记都没有。")
+            return False, lines
+        newest = read_crash_case_marker(others[0]) or {"fields": {}, "input_files": []}
+        differences = manifest_differences(newest["input_files"], manifest_text.rstrip("\n").split("\n"), "那一格", "这一次")
+        lines.append(f"最近写的一格是 {os.path.basename(others[0])}（跑完于 {newest['fields'].get('finished_utc', '没记')}），"
+                     f"与这一次的输入比，不同的共 {len(differences)} 处（最多列 {MARKER_DIFFERENCES_LISTED_AT_MOST} 处）：")
+        lines += ["  " + difference for difference in differences[:MARKER_DIFFERENCES_LISTED_AT_MOST]]
+        return False, lines
+    problems = crash_case_marker_problems(case, fingerprint, marker)
+    if problems:
+        return False, [f"这批输入那一格全绿标记（{marker_path}）不作数："] + ["  " + problem for problem in problems]
+    shown = [line for line in marker["raw_lines"] if any(line.startswith(prefix + " ") for prefix in case.count_lines)]
+    return True, [f"ok {marker_path} {marker['fields'].get('finished_utc', '没记')}"] + shown
+
+
+def write_crash_case_marker(root, case, fingerprint, manifest_text, recorded_lines, details):
+    """判绿之后写那一格全绿标记：同目录排他建临时文件、写完改名换上。返回标记路径；取不到 common-dir、写不了抛 InputManifestError。"""
+    marker_path = crash_case_marker_path(root, case, fingerprint)
+    if marker_path is None:
+        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处写")
+    text = ("# 崩溃枚举用例的全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后经 research/scripts/admission.py crash-case-record 写，"
+            "按用例与它的输入指纹分格；整轮门禁的快档与下一趟 --full 按这一格判复用。不进工作树，别手改。\n"
+            f"case={case.key}\ntest={case.test_text()}\ninput_hash={fingerprint}\n"
+            + "".join(f"{name}={value}\n" for name, value in details)
+            + "".join(line + "\n" for line in recorded_lines)
+            + "".join(f"input_file {line}\n" for line in manifest_text.rstrip("\n").split("\n") if line))
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
 # ── 命令行 ────────────────────────────────────────────────────────────────────
 
 def command_gate_reuse(arguments):
@@ -836,38 +1451,51 @@
     return 0
 
 
-def command_manifest(arguments):
-    usage_ok = len(arguments) >= 3
-    extra_lines_requested = []
-    index = 3
-    while usage_ok and index < len(arguments):
-        option = arguments[index]
-        if option == "--extra-file" and index + 2 < len(arguments):
-            extra_lines_requested.append(("file", arguments[index + 1], arguments[index + 2]))
+def named_line_requests(options, allowed_flags):
+    """清单末尾那几行的参数：[(种类, 名字, 文件)]，种类是 file 或 flag 本身；写法不对返回 None。"""
+    requested = []
+    index = 0
+    while index < len(options):
+        option = options[index]
+        if option == "--extra-file" and index + 2 < len(options):
+            requested.append(("file", options[index + 1], options[index + 2]))
             index += 3
-        elif option in ("--toolchain", "--registration-row"):
-            extra_lines_requested.append((option, None, None))
+        elif option in allowed_flags:
+            requested.append((option, None, None))
             index += 1
         else:
-            usage_ok = False
-    if not usage_ok:
-        say("  ✗ 用法：admission.py manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain]")
-        say("     → 怎么办：末尾那几行按参数次序加；门禁 54 号的写法是 --extra-file \"<判它的 54 号：54-layer0-replay.sh>\" <54 号的路径> --toolchain")
+            return None
+    return requested
+
+
+def named_lines_of(root, requested, registration_line=None):
+    """按参数次序算末尾那几行；--registration-row 取 registration_line。读不了 --extra-file 的文件抛 OSError。"""
+    named_lines = []
+    for kind, name, path in requested:
+        if kind == "file":
+            with open(path, "rb") as handle:
+                named_lines.append((name, handle.read()))
+        elif kind == "--toolchain":
+            named_lines.append(toolchain_line(root))
+        elif kind == "--build-environment":
+            named_lines.extend(build_environment_lines(root))
+        else:
+            named_lines.append(registration_line)
+    return named_lines
+
+
+def command_manifest(arguments):
+    requested = named_line_requests(arguments[3:], ("--toolchain", "--registration-row", "--build-environment")) if len(arguments) >= 3 else None
+    if requested is None:
+        say("  ✗ 用法：admission.py manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain] [--build-environment]")
+        say("     → 怎么办：末尾那几行按参数次序加，例 --extra-file \"<判它的 54 号：54-layer0-replay.sh>\" <54 号的路径> --toolchain --build-environment")
         return EXIT_REGISTRATION_ERROR
     root, key, manifest_file = arguments[0], arguments[1], arguments[2]
     try:
         rows = read_registration_rows(root)
         input_paths, contributing_rows = resolve_input_paths(rows, key)
-        named_lines = []
-        for kind, name, path in extra_lines_requested:
-            if kind == "file":
-                with open(path, "rb") as handle:
-                    named_lines.append((name, handle.read()))
-            elif kind == "--toolchain":
-                named_lines.append(toolchain_line(root))
-            else:
-                registration_text = "".join(row.fingerprint_text() for row in contributing_rows)
-                named_lines.append((f"<登记行：{key}>", registration_text.encode("utf-8", "surrogateescape")))
+        registration_text = "".join(row.fingerprint_text() for row in contributing_rows)
+        named_lines = named_lines_of(root, requested, (f"<登记行：{key}>", registration_text.encode("utf-8", "surrogateescape")))
         manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
     except (RegistrationError, InputManifestError, OSError) as error:
         say(f"  ✗ 写不出 {key} 的输入清单：{error}")
@@ -935,6 +1563,149 @@
     return 0
 
 
+def command_crash_cases(arguments):
+    """核登记的崩溃枚举用例，逐条打「键、包、测试目标、用例函数」；登记有错退 2，原因不带 ✗（阶段自己打 ✗ 与出路）。"""
+    if len(arguments) != 1:
+        print("  ✗ 用法：admission.py crash-cases <项目根>")
+        print("     → 怎么办：只给项目根一个参数")
+        return EXIT_REGISTRATION_ERROR
+    root = arguments[0]
+    rows = read_registration_rows(root)
+    crash_case_rows = [row for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]
+    problems = [problem for problem in lint_registration_rows(crash_case_rows, root)]
+    if problems:
+        for problem in problems:
+            print(f"{REGISTRATION_TABLE} {problem}")
+        return EXIT_REGISTRATION_ERROR
+    for case in crash_cases_of(crash_case_rows):
+        print(f"{case.key}\t{case.package}\t{case.target}\t{case.function}")
+    return 0
+
+
+def command_crash_case_manifest(arguments):
+    requested = named_line_requests(arguments[3:], ("--toolchain", "--build-environment")) if len(arguments) >= 3 else None
+    if requested is None:
+        print("  ✗ 用法：admission.py crash-case-manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--toolchain] [--build-environment]")
+        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写；末尾那几行按参数次序加")
+        return EXIT_REGISTRATION_ERROR
+    root, key, manifest_file = arguments[0], arguments[1], arguments[2]
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+        manifest, fingerprint, file_count, excluded = crash_case_manifest(root, case, named_lines_of(root, requested))
+    except (RegistrationError, InputManifestError, OSError) as error:
+        print(f"写不出 {key} 的输入清单：{error}")
+        return EXIT_REGISTRATION_ERROR
+    with open(manifest_file, "wb") as handle:
+        handle.write(manifest)
+    print(f"{fingerprint} {file_count} {len(excluded)}")
+    return 0
+
+
+def command_crash_case_marker_check(arguments):
+    if len(arguments) != 4:
+        print("  ✗ 用法：admission.py crash-case-marker-check <项目根> <键> <指纹> <清单文件>")
+        print("     → 怎么办：指纹与清单文件取 crash-case-manifest 那一趟的")
+        return EXIT_REGISTRATION_ERROR
+    root, key, fingerprint, manifest_file = arguments
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+        with open(manifest_file, encoding="utf-8", errors="surrogateescape") as handle:
+            manifest_text = handle.read()
+    except (RegistrationError, OSError) as error:
+        print(f"判不了 {key} 的全绿标记：{error}")
+        return EXIT_REGISTRATION_ERROR
+    valid, lines = check_crash_case_marker(root, case, fingerprint, manifest_text)
+    for line in lines:
+        print(line)
+    return 0 if valid else 1
+
+
+def option_values(options, names):
+    """--名字 值 成对的参数：返回 {名字: 值}；有不认得的、缺值的返回 None。"""
+    values = {}
+    index = 0
+    while index < len(options):
+        if options[index] not in names or index + 1 >= len(options):
+            return None
+        values[options[index]] = options[index + 1]
+        index += 2
+    return values
+
+
+def command_crash_case_judge(arguments):
+    values = option_values(arguments[4:], ("--machine-cores", "--threads", "--threads-origin")) if len(arguments) >= 4 else None
+    usable = (values is not None and set(values) == {"--machine-cores", "--threads", "--threads-origin"}
+              and values["--machine-cores"].isdigit() and values["--threads"].isdigit() and values["--threads-origin"] in ("explicit", "default"))
+    if not usable:
+        print("  ✗ 用法：admission.py crash-case-judge <项目根> <键> <日志> <记录行文件> --machine-cores <核数> --threads <线程数> --threads-origin explicit|default")
+        print("     → 怎么办：核数取 nproc，线程数取传给用例的 SINGLEFS_LAYER0_THREADS；显式设的写 explicit，没设、取本机核数的写 default")
+        return EXIT_REGISTRATION_ERROR
+    root, key, log_file, recorded_file = arguments[:4]
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
+            log_text = handle.read()
+    except (RegistrationError, OSError) as error:
+        print(f"判不了 {key} 的日志：{error}")
+        return EXIT_REGISTRATION_ERROR
+    threads_explicitly_one = values["--threads-origin"] == "explicit" and values["--threads"] == "1"
+    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, int(values["--machine-cores"]), threads_explicitly_one)
+    if problems:
+        for problem in problems:
+            print(problem)
+        return 1
+    with open(recorded_file, "w", encoding="utf-8", errors="surrogateescape") as handle:
+        handle.write("".join(line + "\n" for line in recorded_lines))
+    for note in thread_notes:
+        print(note)
+    return 0
+
+
+def command_crash_case_record(arguments):
+    names = ("--files", "--excluded", "--started", "--judged-root", "--threads-text")
+    values = option_values(arguments[5:], names) if len(arguments) >= 5 else None
+    if values is None or set(values) != set(names):
+        print("  ✗ 用法：admission.py crash-case-record <项目根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> "
+              "--started <时刻> --judged-root <路径> --threads-text <一句>")
+        print("     → 怎么办：指纹、清单文件、文件数与减去的文件数取开跑时 crash-case-manifest 那一趟的，记录行文件取 crash-case-judge 写的")
+        return EXIT_REGISTRATION_ERROR
+    root, key, fingerprint, manifest_file, recorded_file = arguments[:5]
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+        with open(manifest_file, encoding="utf-8", errors="surrogateescape") as handle:
+            manifest_text = handle.read()
+        with open(recorded_file, encoding="utf-8", errors="surrogateescape") as handle:
+            recorded_lines = [line for line in handle.read().split("\n") if line]
+        details = [("input_file_count", values["--files"]), ("excluded_file_count", values["--excluded"]),
+                   ("started_utc", values["--started"]), ("finished_utc", time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())),
+                   ("judged_root", values["--judged-root"]), ("worker_threads", values["--threads-text"])]
+        marker_path = write_crash_case_marker(root, case, fingerprint, manifest_text, recorded_lines, details)
+    except (RegistrationError, InputManifestError, OSError) as error:
+        print(f"{key} 的全绿标记没写成：{error}")
+        return 1
+    print(marker_path)
+    return 0
+
+
+def command_crash_case_marker_path(arguments):
+    if len(arguments) != 3:
+        print("  ✗ 用法：admission.py crash-case-marker-path <项目根> <键> <指纹>")
+        print("     → 怎么办：指纹取 crash-case-manifest 那一趟的")
+        return EXIT_REGISTRATION_ERROR
+    root, key, fingerprint = arguments
+    try:
+        case = crash_case_of_key(read_registration_rows(root), key)
+    except RegistrationError as error:
+        print(f"找不到 {key}：{error}")
+        return EXIT_REGISTRATION_ERROR
+    marker_path = crash_case_marker_path(root, case, fingerprint)
+    if marker_path is None:
+        print(f"{root} 不是 git 工作树（取不到 git common-dir）")
+        return EXIT_REGISTRATION_ERROR
+    print(marker_path)
+    return 0
+
+
 # ── 自证 ──────────────────────────────────────────────────────────────────────
 
 SELFTEST_HERE = os.path.dirname(os.path.abspath(__file__))
@@ -1115,7 +1886,7 @@
         status, _detail = stored_product_status(work, "E904")
         selftest.expect("没登记的键状态是 unregistered", status == "unregistered", f"实际 {status}")
 
-        # ⑨ 清单与门禁 54 号的 write_layer0_input_manifest 同一种写法：拿 sha256sum、sort -zu 与 cargo -V 另算一遍，逐字节比
+        # ⑨ manifest 的清单写法：拿 sha256sum、sort -zu 与 cargo -V 另算一遍，逐字节比（crash-case-manifest 写清单用的是同一个 manifest_of_files）
         independent = subprocess.run(["bash", "-c", INDEPENDENT_MANIFEST_SCRIPT, "_", work, os.path.join(work, REGISTRATION_TABLE)],
                                      capture_output=True)
         manifest_file = os.path.join(work, "manifest.out")
@@ -1123,7 +1894,7 @@
                                                    "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", table_path, "--toolchain"])
         with open(manifest_file, "rb") as handle:
             module_manifest = handle.read()
-        selftest.expect("清单与 54 号的写法（sha256sum、sort -zu、cargo -V && rustc -V）逐字节相同",
+        selftest.expect("清单与另算的一遍（sha256sum、sort -zu、cargo -V && rustc -V）逐字节相同",
                         independent.returncode == 0 and exit_code == 0 and module_manifest == independent.stdout
                         and output.split()[0] == hashlib.sha256(independent.stdout).hexdigest(),
                         f"另算退 {independent.returncode}，模块退 {exit_code}，{messages.strip()}")
@@ -1140,6 +1911,14 @@
         run_gate_environment_cells(selftest, module)
         # ⑫ replay.sh 接准入的那几处：输入没变不跑、前提没齐拒、比对前删产物头、装置没打指纹行
         run_replay_cells(selftest, module)
+        # ⑭ 构建环境与指向目录的符号链接进指纹（层 0 规模第三轮判决 U4）：每一样改了，指纹必须变
+        run_build_environment_cells(selftest, module)
+        # ⑮ 崩溃枚举用例：去注释找点名、减去别的测试目标独占的文件、登记行自查、日志怎么判、全绿标记读写
+        run_rust_comment_cells(selftest)
+        run_crash_case_input_cells(selftest, module)
+        run_crash_case_log_cells(selftest)
+        # ⑯ 门禁 54 号这一份的流程：拷进临时仓（不在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑
+        run_layer0_stage_cells(selftest, module)
 
         # ⑬ 真仓的登记表自己写对没有（语法：实验键、条件的种类分门禁与实验、@ 引用、probe= / environment= 指的脚本在不在）
         repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
@@ -1153,7 +1932,8 @@
         print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
         return 1
     print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、"
-          "ignore-gate-environment 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
+          "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count "
+          "下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
     return 0
 
 
@@ -1423,7 +2203,567 @@
         shutil.rmtree(tools, ignore_errors=True)
 
 
-# 与门禁 54 号 write_layer0_input_manifest 同一套步骤的另一份实现，只给自证当对照：它不 import 本模块，用的是 sha256sum 与 sort。
+def environment_without_build_settings(changes=None):
+    """子进程的环境：去掉会进构建环境那一格的变量（CARGO_HOME、RUSTC 与名字对得上的那几类）与握手用的几个，再套上 changes。"""
+    environment = {name: value for name, value in os.environ.items()
+                   if not (name in BUILD_ENVIRONMENT_EXACT_VARIABLES or name in ("RUSTC", "CARGO_HOME")
+                           or any(form.match(name) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS))}
+    for variable in (FORCE_RERUN_VARIABLE, "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE", "SINGLEFS_REUSE_HOURS"):
+        environment.pop(variable, None)
+    environment.update(changes or {})
+    return environment
+
+
+def run_in_environment(command, environment, working_directory=None):
+    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
+    return completed.returncode, completed.stdout, completed.stderr
+
+
+class BreakSwitch:
+    """在本进程里临时设 ADMISSION_BREAK（给纯函数那几格证明会红），退出时照原样还回去。"""
+    def __init__(self, switch_name):
+        self.switch_name = switch_name
+        self.saved = None
+
+    def __enter__(self):
+        self.saved = os.environ.get(BREAK_VARIABLE)
+        os.environ[BREAK_VARIABLE] = self.switch_name
+        return self
+
+    def __exit__(self, *_exception):
+        if self.saved is None:
+            os.environ.pop(BREAK_VARIABLE, None)
+        else:
+            os.environ[BREAK_VARIABLE] = self.saved
+        return False
+
+
+FAKE_TOOLCHAIN_SCRIPTS = {
+    "cargo": '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "cargo 0.0.0-selftest"\nexit 0\n',
+    "rustc": '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "rustc 0.0.0-selftest"\nexit 0\n',
+}
+
+
+def run_build_environment_cells(selftest, module):
+    """构建环境与指向目录的符号链接进指纹：仓根往上一层与仓根自己的 cargo 配置、CARGO_HOME 下的配置、rust-toolchain 两种、
+    环境变量那几类、RUSTC 的 -V、登记路径下指向目录的符号链接，每一样改了指纹必须变、改回去又相同；CARGO_BUILD_JOBS 改了指纹不变。"""
+    base = tempfile.mkdtemp(prefix="admission-selftest-build-")
+    try:
+        outer = os.path.join(base, "outer")
+        work = os.path.join(outer, "repo")
+        cargo_home = os.path.join(base, "cargo-home")
+        tools = os.path.join(base, "tools")
+        outside = os.path.join(base, "outside")
+        os.makedirs(cargo_home)
+        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
+        write_text(os.path.join(work, REGISTRATION_TABLE), "59-demo.sh\tcrates/ Cargo.toml\t# 样本：门禁行\nE900\tcrates/\t# 样本：实验\n")
+        write_text(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
+        write_text(os.path.join(work, "Cargo.toml"), "[workspace]\n")
+        write_text(os.path.join(outside, "shared/shared.rs"), "pub fn shared() {}\n")
+        os.symlink(os.path.join(outside, "shared"), os.path.join(work, "crates/linked"))
+        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
+            write_executable(os.path.join(tools, name), script)
+        version_file = os.path.join(base, "switchable-version")
+        write_text(version_file, "rustc 9.9.9-a\n")
+        switchable_rustc = os.path.join(tools, "switchable-rustc")
+        write_executable(switchable_rustc, f'#!/usr/bin/env bash\ncat "{version_file}"\n')
+        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": cargo_home})
+        manifest_file = os.path.join(base, "manifest.out")
+
+        def fingerprint(changes=None):
+            exit_code, output, messages = run_in_environment(
+                [sys.executable, module, "manifest", work, "59-demo.sh", manifest_file, "--toolchain", "--build-environment"],
+                dict(environment, **(changes or {})))
+            return output.split()[0] if exit_code == 0 and output.split() else f"退 {exit_code}：{messages.strip()}"
+
+        baseline = fingerprint()
+        file_cells = [
+            ("仓根往上一层的 .cargo/config.toml", os.path.join(outer, ".cargo/config.toml")),
+            ("仓根往上一层的 .cargo/config", os.path.join(outer, ".cargo/config")),
+            ("仓根自己的 .cargo/config.toml", os.path.join(work, ".cargo/config.toml")),
+            ("CARGO_HOME 下的 config.toml", os.path.join(cargo_home, "config.toml")),
+            ("CARGO_HOME 下的 config", os.path.join(cargo_home, "config")),
+            ("仓根往上一层的 rust-toolchain", os.path.join(outer, "rust-toolchain")),
+            ("仓根往上一层的 rust-toolchain.toml", os.path.join(outer, "rust-toolchain.toml")),
+        ]
+        for label, path in file_cells:
+            write_text(path, '[build]\nrustflags = ["-C", "overflow-checks=off"]\n')
+            changed = fingerprint()
+            os.remove(path)
+            restored = fingerprint()
+            selftest.expect(f"构建环境：加一份{label}，指纹变；删掉又回到原样", changed != baseline and restored == baseline,
+                            f"原样 {baseline[:16]}，加了之后 {changed[:16]}，删掉之后 {restored[:16]}")
+        variable_cells = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS", "CARGO_BUILD_RUSTFLAGS",
+                          "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
+                          "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]
+        for variable in variable_cells:
+            changed = fingerprint({variable: "selftest-value"})
+            selftest.expect(f"构建环境：设了环境变量 {variable}，指纹变", changed != baseline, f"原样与设了之后都是 {baseline[:16]}")
+        left_out = fingerprint({"CARGO_BUILD_JOBS": "7"})
+        selftest.expect("构建环境：只设 CARGO_BUILD_JOBS（编译并行度，包装会设它），指纹不变", left_out == baseline,
+                        f"原样 {baseline[:16]}，设了之后 {left_out[:16]}")
+        with_rustc_a = fingerprint({"RUSTC": switchable_rustc})
+        write_text(version_file, "rustc 9.9.9-b\n")
+        with_rustc_b = fingerprint({"RUSTC": switchable_rustc})
+        selftest.expect("构建环境：RUSTC 设了指纹变；同一个 RUSTC 的 -V 变了，指纹也变",
+                        with_rustc_a != baseline and with_rustc_b != with_rustc_a,
+                        f"原样 {baseline[:16]}，RUSTC 报 a {with_rustc_a[:16]}，报 b {with_rustc_b[:16]}")
+        linked_file = os.path.join(outside, "shared/shared.rs")
+        write_text(linked_file, "pub fn shared() { let changed = 1; }\n")
+        through_link = fingerprint()
+        before_break = fingerprint({BREAK_VARIABLE: "ignore-linked-directories"})
+        write_text(linked_file, "pub fn shared() {}\n")
+        after_break = fingerprint({BREAK_VARIABLE: "ignore-linked-directories"})
+        selftest.expect("登记路径下指向目录的符号链接：链接指向的目录里的文件改了，指纹变", through_link != baseline and fingerprint() == baseline,
+                        f"原样 {baseline[:16]}，改了链接那头之后 {through_link[:16]}")
+        selftest.expect("弄坏开关 ignore-linked-directories 下「链接那头改了」那一格红（指纹不变）", before_break == after_break,
+                        f"弄坏之后仍然一变一不变：{before_break[:16]} → {after_break[:16]}")
+        config_path = os.path.join(outer, ".cargo/config.toml")
+        broken_before = fingerprint({BREAK_VARIABLE: "skip-build-environment"})
+        write_text(config_path, '[build]\nrustflags = ["-C", "overflow-checks=off"]\n')
+        broken_after = fingerprint({BREAK_VARIABLE: "skip-build-environment"})
+        os.remove(config_path)
+        selftest.expect("弄坏开关 skip-build-environment 下「上一层的 .cargo/config.toml」那一格红（指纹不变）", broken_before == broken_after,
+                        f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
+
+        def experiment_fingerprint_line(changes=None):
+            _exit_code, output, _messages = run_in_environment([sys.executable, module, "experiment", work, "E900"],
+                                                               dict(environment, **(changes or {})))
+            return output.strip().split("\n")[0]
+        plain = experiment_fingerprint_line()
+        with_flags = experiment_fingerprint_line({"RUSTFLAGS": "-C overflow-checks=off"})
+        selftest.expect("实验的输入指纹同样带构建环境：设了 RUSTFLAGS，产物头的指纹变",
+                        plain.startswith("E7INPUT name=input_fingerprint key=E900 ") and with_flags.startswith("E7INPUT ") and plain != with_flags,
+                        f"没设「{plain}」，设了「{with_flags}」")
+    finally:
+        shutil.rmtree(base, ignore_errors=True)
+
+
+RUST_COMMENT_CELLS = [
+    ("行注释里提到不算", "// helper_x\nfn a() {}\n", False),
+    ("可嵌套的块注释里提到不算", "/* outer /* inner helper_x */ still helper_x */ fn a() {}", False),
+    ("文档注释里提到不算", "/// 见 helper_x\n//! helper_x\nfn a() {}", False),
+    ("mod 声明算", "mod helper_x;\n", True),
+    ("字符串里的路径算", 'const P: &str = "tests/helper_x.rs";', True),
+    ("字符串里带 // 之后的代码照样算", 'let u = "a//b"; mod helper_x;', True),
+    ("原始字符串里带引号之后的代码照样算", 'let r = r#"a"b"#; mod helper_x;', True),
+    ("字符字面量是双引号之后的代码照样算", "let q = '\"'; mod helper_x;", True),
+    ("生命周期之后的代码照样算", "fn f<'a>(x: &'a str) -> &'a str { x } mod helper_x;", True),
+    ("转义的单引号字符之后的代码照样算", "let c = '\\''; mod helper_x;", True),
+    ("更长的名字里含它不算（按整词）", "mod helper_x_more;", False),
+]
+
+
+def run_rust_comment_cells(selftest):
+    """判「别处有没有代码点名一个测试目标」的那一步：注释里提到不算，代码与字符串里算，字符串、字符字面量、生命周期不把后面的代码吞掉。"""
+    for label, code, expected in RUST_COMMENT_CELLS:
+        found = bool(whole_word_form("helper_x").search(rust_code_without_comments(code)))
+        selftest.expect(f"去注释找点名：{label}", found == expected, f"应当 {'找到' if expected else '找不到'}，实际 {'找到' if found else '找不到'}")
+
+
+def build_crash_case_repository(work, registration_rows):
+    """一个临时小仓：包 pkg 里有用例 own_case（mod common、mod helper_target）、被注释提到的 other_target、被字符串点名的 string_named。"""
+    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
+    write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
+    write_text(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\n')
+    write_text(os.path.join(work, "crates/pkg/src/lib.rs"),
+               '//! 注释里提到 other_target 不算\npub const PATH: &str = "tests/string_named.rs";\npub fn one() -> u32 { 1 }\n')
+    write_text(os.path.join(work, "crates/pkg/tests/own_case.rs"),
+               'mod common;\nmod helper_target;\n#[test]\n#[ignore = "崩溃枚举（样本）"]\nfn the_case() {}\n')
+    write_text(os.path.join(work, "crates/pkg/tests/common/mod.rs"), "pub fn shared() {}\n")
+    write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
+    write_text(os.path.join(work, "crates/pkg/tests/other_target.rs"), "#[test]\nfn other() {}\n")
+    write_text(os.path.join(work, "crates/pkg/tests/string_named.rs"), "#[test]\nfn named() {}\n")
+    write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in registration_rows))
+
+
+def run_crash_case_input_cells(selftest, module):
+    """崩溃枚举用例的输入：减去的恰好是别的测试目标独占、没被代码点名的文件；改它指纹不变，改被点名的、共用的、登记行、新加的共用文件指纹变。
+    另核登记行自查与全绿标记的读写。"""
+    work = tempfile.mkdtemp(prefix="admission-selftest-crash-case-")
+    try:
+        own_row = "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 exhaustive=LAYER0\t# 样本"
+        build_crash_case_repository(work, [own_row])
+        tools = os.path.join(work, ".tools")
+        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
+            write_executable(os.path.join(tools, name), script)
+        write_text(os.path.join(work, ".gitignore"), ".tools/\n*.out\n")
+        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
+                                                          "CARGO_HOME": os.path.join(work, ".tools/cargo-home")})
+        manifest_file = os.path.join(work, "manifest.out")
+
+        def admission(*arguments, changes=None):
+            return run_in_environment([sys.executable, module, *arguments], dict(environment, **(changes or {})))
+
+        def fingerprint(changes=None):
+            exit_code, output, _messages = admission("crash-case-manifest", work, "crash-case:own", manifest_file, "--toolchain",
+                                                     "--build-environment", changes=changes)
+            parts = output.split()
+            return (parts[0], int(parts[2])) if exit_code == 0 and len(parts) == 3 else (f"退 {exit_code}：{output.strip()}", -1)
+
+        exit_code, output, _messages = admission("crash-cases", work)
+        selftest.expect("crash-cases 逐条打「键、包、测试目标、用例函数」", exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\n",
+                        f"退 {exit_code}，stdout「{output.strip()}」")
+        baseline, excluded_count = fingerprint()
+        with open(manifest_file, encoding="utf-8") as handle:
+            manifest_text = handle.read()
+        kept_names = sorted(line[66:] for line in manifest_text.split("\n") if len(line) > 66 and not line[66:].startswith("<"))
+        selftest.expect("减去的恰好是没被代码点名的 other_target.rs（注释里提到不算）；被 mod 点名的、被字符串点名的、共用的 common/ 与用例自己都留着",
+                        excluded_count == 1 and "crates/pkg/tests/other_target.rs" not in kept_names
+                        and all(name in kept_names for name in ("crates/pkg/tests/own_case.rs", "crates/pkg/tests/helper_target.rs",
+                                                                "crates/pkg/tests/string_named.rs", "crates/pkg/tests/common/mod.rs")),
+                        f"减去 {excluded_count} 个，留下的是 {kept_names}")
+        changes = [
+            ("改别的测试目标独占的 other_target.rs", "crates/pkg/tests/other_target.rs", "#[test]\nfn other() { assert!(true); }\n", False),
+            ("新加一个没人点名的测试目标 new_unrelated.rs", "crates/pkg/tests/new_unrelated.rs", "#[test]\nfn new() {}\n", False),
+            ("改被用例 mod 点名的 helper_target.rs", "crates/pkg/tests/helper_target.rs", "pub fn helper() { let changed = 1; }\n", True),
+            ("改被 lib.rs 里的字符串点名的 string_named.rs", "crates/pkg/tests/string_named.rs", "#[test]\nfn named() { let changed = 1; }\n", True),
+            ("改共用的 tests/common/mod.rs", "crates/pkg/tests/common/mod.rs", "pub fn shared() { let changed = 1; }\n", True),
+            ("新加一份与 common/mod.rs 并存的 tests/common.rs（用例写着 mod common）", "crates/pkg/tests/common.rs", "pub fn clash() {}\n", True),
+            ("改 src/lib.rs", "crates/pkg/src/lib.rs", "//! 注释里提到 other_target 不算\npub const PATH: &str = \"tests/string_named.rs\";\npub fn one() -> u32 { 2 }\n", True),
+        ]
+        for label, relative, text, should_change in changes:
+            path = os.path.join(work, relative)
+            original = open(path, encoding="utf-8").read() if os.path.exists(path) else None
+            write_text(path, text)
+            changed, _count = fingerprint()
+            if original is None:
+                os.remove(path)
+            else:
+                write_text(path, original)
+            restored, _count = fingerprint()
+            selftest.expect(f"崩溃枚举用例的输入：{label}，指纹{'变' if should_change else '不变'}",
+                            (changed != baseline) == should_change and restored == baseline,
+                            f"原样 {baseline[:16]}，改了之后 {changed[:16]}，改回之后 {restored[:16]}")
+        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() { let changed = 1; }\n")
+        broken_changed, _count = fingerprint({BREAK_VARIABLE: "exclude-mentioned-test-targets"})
+        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
+        broken_restored, _count = fingerprint({BREAK_VARIABLE: "exclude-mentioned-test-targets"})
+        selftest.expect("弄坏开关 exclude-mentioned-test-targets 下「改被 mod 点名的 helper_target.rs」那一格红（指纹不变）",
+                        broken_changed == broken_restored, f"弄坏之后仍然一变一不变：{broken_changed[:16]} → {broken_restored[:16]}")
+        build_script = os.path.join(work, "crates/pkg/build.rs")
+        write_text(build_script, "fn main() {}\n")
+        with_build_script, excluded_with_build_script = fingerprint()
+        os.remove(build_script)
+        selftest.expect("包里有 build.rs 时这个包的测试文件一份都不减（指纹变、减去 0 个）",
+                        with_build_script != baseline and excluded_with_build_script == 0, f"减去 {excluded_with_build_script} 个")
+        table_path = os.path.join(work, REGISTRATION_TABLE)
+        write_text(table_path, own_row.replace("exhaustive=LAYER0", "exhaustive=LAYER0 threads=LAYER0") + "\n")
+        with_threads, _count = fingerprint()
+        write_text(table_path, own_row + "\n")
+        selftest.expect("改了这条用例登记行的第三列，指纹变", with_threads != baseline, f"原样与改了之后都是 {baseline[:16]}")
+
+        lint_rows = [
+            ("用例函数在测试目标里找不到", "crash-case:own\tcrates/\ttest=pkg:own_case:no_such_function\t# 样本", "找不到"),
+            ("测试目标不在", "crash-case:own\tcrates/\ttest=pkg:no_such_target:the_case\t# 样本", "不在"),
+            ("包不是工作区成员", "crash-case:own\tcrates/\ttest=nowhere:own_case:the_case\t# 样本", "不是仓根 Cargo.toml 的工作区成员"),
+            ("test= 写了两条", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case test=pkg:own_case:the_case\t# 样本", "恰好一条 test="),
+            ("exhaustive= 点名的前缀没登记成 count-line=", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case exhaustive=LAYER0\t# 样本", "没有登记成 count-line="),
+            ("认不出的条件", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case command=cargo\t# 样本", "认不出"),
+            ("同一个键两行", own_row + "\n" + own_row, "一条用例只许一行"),
+        ]
+        for label, rows_text, expected_phrase in lint_rows:
+            write_text(table_path, rows_text + "\n")
+            exit_code, output, _messages = admission("crash-cases", work)
+            selftest.expect(f"crash-cases 自查：{label}，退 2 并说出来", exit_code == EXIT_REGISTRATION_ERROR and expected_phrase in output,
+                            f"退 {exit_code}，stdout「{output.strip()}」")
+        write_text(table_path, own_row + "\n")
+
+        # 全绿标记：写了再核作数；换一个指纹不作数且比出不同；标记被改（指纹、exhaustive）不作数
+        judged_file = os.path.join(work, "judged.out")
+        write_text(judged_file, "test_result=test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s\n"
+                                "LAYER0 states=5 closed_form=5 exhaustive=true\nparallel_finished=LAYER0_PARALLEL_FINISHED states=5 slices=1\n")
+        fingerprint_now, _count = fingerprint()
+        exit_code, marker_path, _messages = admission("crash-case-record", work, "crash-case:own", fingerprint_now, manifest_file, judged_file,
+                                                      "--files", "6", "--excluded", "1", "--started", "2026-09-26",
+                                                      "--judged-root", work, "--threads-text", "样本")
+        marker_path = marker_path.strip()
+        exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", fingerprint_now, manifest_file)
+        selftest.expect("crash-case-record 写的那一格，crash-case-marker-check 判作数并带出计数行",
+                        exit_code == 0 and os.path.isfile(marker_path) and exit_code_check == 0 and output.startswith(f"ok {marker_path} ")
+                        and "LAYER0 states=5 closed_form=5 exhaustive=true" in output, f"写退 {exit_code}，核退 {exit_code_check}，stdout「{output.strip()}」")
+        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() { let changed = 1; }\n")
+        other_fingerprint, _count = fingerprint()
+        exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", other_fingerprint, manifest_file)
+        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
+        selftest.expect("输入变了的那一批没有自己那一格：不作数，并比出与最近写的一格不同的是哪个文件",
+                        exit_code_check == 1 and "没有全绿标记" in output and "内容不同：crates/pkg/tests/helper_target.rs" in output,
+                        f"退 {exit_code_check}，stdout「{output.strip()}」")
+        with open(marker_path, encoding="utf-8") as handle:
+            good_marker = handle.read()
+        for label, old, new, phrase in (("记的指纹被改了", f"input_hash={fingerprint_now}", "input_hash=" + "0" * 64, "输入指纹"),
+                                        ("计数行不带 exhaustive=true", "exhaustive=true", "exhaustive=false", "exhaustive=true"),
+                                        ("计数行抄了两遍", "LAYER0 states=5 closed_form=5 exhaustive=true\n",
+                                         "LAYER0 states=5 closed_form=5 exhaustive=true\n" * 2, "恰好一行"),
+                                        ("test result 不是 1 passed", "1 passed;", "0 passed;", "1 passed")):
+            write_text(marker_path, good_marker.replace(old, new, 1))
+            exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", fingerprint_now, manifest_file)
+            selftest.expect(f"全绿标记{label}：不作数并说出来", exit_code_check == 1 and phrase in output, f"退 {exit_code_check}，stdout「{output.strip()}」")
+        write_text(marker_path, good_marker)
+    finally:
+        shutil.rmtree(work, ignore_errors=True)
+
+
+def synthetic_layer0_log(states=100, slices=64, worker_threads=32, resumed_slices=0, freshly_run_slices=64, exhaustive="true",
+                         result="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s",
+                         layer0_lines=1, with_checker=True, with_finished=True, finished_fields=None):
+    """合成的一趟 --full 日志：LAYER0_PARALLEL_FINISHED、LAYER0 计数行、CHECKER 行、test result 各几行由参数定。"""
+    lines = [f"LAYER0_PARALLEL_START states={states} slices={slices}"]
+    if with_finished:
+        fields = finished_fields or (f"worker_threads={worker_threads} configured_worker_threads=32 worker_threads_source=environment_variable "
+                                     f"resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices}")
+        lines.append(f"LAYER0_PARALLEL_FINISHED states={states} slices={slices} {fields} progress_file_after_completion=deleted elapsed_seconds=1.0")
+    lines += [f"LAYER0 states={states} closed_form={states} violations=0 exhaustive={exhaustive}"] * layer0_lines
+    if with_checker:
+        lines.append("CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0")
+    if result is not None:
+        lines.append(result)
+    return "\n".join(lines) + "\n"
+
+
+def run_crash_case_log_cells(selftest):
+    """--full 跑一条用例的日志怎么判（纯函数，合成日志）：test result 恰好 1 passed、计数行恰好一行、exhaustive=true、工作线程按这一趟跑了几片判。"""
+    case = parse_crash_case(RegistrationRow("crash-case:demo", ["crates/"], ["test=pkg:target:function", "count-line=LAYER0",
+                                                                             "count-line=CHECKER", "exhaustive=LAYER0", "threads=LAYER0"], 1, ""))
+    cells = [
+        ("全量、32 个线程跑了 64 片", synthetic_layer0_log(), 32, False, True),
+        ("全部 64 片从进度文件读回、起 0 个线程", synthetic_layer0_log(worker_threads=0, resumed_slices=64, freshly_run_slices=0), 32, False, True),
+        ("读回 63 片、只剩 1 片要跑、起 1 个线程", synthetic_layer0_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), 32, False, True),
+        ("跑了 64 片却只起 1 个线程（本机 32 核、线程数没显式设）", synthetic_layer0_log(worker_threads=1), 32, False, False),
+        ("跑了 64 片只起 1 个线程，线程数显式设成 1", synthetic_layer0_log(worker_threads=1), 32, True, True),
+        ("跑了 64 片只起 1 个线程，本机只有 1 核", synthetic_layer0_log(worker_threads=1), 1, False, True),
+        ("跑了 5 片却报起了 0 个线程", synthetic_layer0_log(worker_threads=0, resumed_slices=59, freshly_run_slices=5), 32, False, False),
+        ("读回的片 + 这一趟跑的片 ≠ 总片数", synthetic_layer0_log(resumed_slices=10, freshly_run_slices=10), 32, False, False),
+        ("LAYER0_PARALLEL_FINISHED 缺续跑片数（旧格式）", synthetic_layer0_log(finished_fields="worker_threads=32"), 32, False, False),
+        ("没有状态数对得上的 LAYER0_PARALLEL_FINISHED", synthetic_layer0_log(with_finished=False), 32, False, False),
+        ("过滤之后一条用例都没跑（0 passed）",
+         synthetic_layer0_log(result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s"), 32, False, False),
+        ("没有 test result 行", synthetic_layer0_log(result=None), 32, False, False),
+        ("不是全量（exhaustive=false）", synthetic_layer0_log(exhaustive="false"), 32, False, False),
+        ("LAYER0 计数行抄了两遍", synthetic_layer0_log(layer0_lines=2), 32, False, False),
+        ("没有 CHECKER 行", synthetic_layer0_log(with_checker=False), 32, False, False),
+    ]
+    for label, log_text, machine_cores, explicitly_one, expected_green in cells:
+        problems, recorded_lines, _notes = judge_crash_case_log(case, log_text, machine_cores, explicitly_one)
+        green = not problems
+        recorded_ok = not green or (recorded_lines[0].startswith("test_result=test result: ok. 1 passed")
+                                    and any(line.startswith("LAYER0 ") for line in recorded_lines)
+                                    and any(line.startswith("parallel_finished=LAYER0_PARALLEL_FINISHED ") for line in recorded_lines))
+        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green and recorded_ok,
+                        f"判成{'绿' if green else '红'}：{problems}；记下的行 {recorded_lines}")
+    with BreakSwitch("threads-by-worker-count"):
+        problems, _recorded, _notes = judge_crash_case_log(
+            case, synthetic_layer0_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), 32, False)
+    selftest.expect("弄坏开关 threads-by-worker-count 下「只剩 1 片要跑、起 1 个线程」那一格红（照旧只看 worker_threads=1）", bool(problems),
+                    "弄坏之后仍判绿：这一格分不出「这一趟跑了几片」")
+
+
+# 假 cargo：-V 打版本；test 带 --include-ignored 的记下目标、过滤、续跑的三个环境变量，照控制目录里那条目标的日志与退出码打；
+# 不带的（快档）打一行 2 passed。控制目录里有 touch-during-run 时，跑的过程中往它写的那个文件追加一行（造「跑的过程中输入变了」）。
+FAKE_CARGO_FOR_STAGE = r'''#!/usr/bin/env bash
+if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-selftest"; exit 0; fi
+target=""; filter=""; include_ignored=0; exact=0; release=0; after_separator=0
+arguments=("$@")
+for ((position = 0; position < ${#arguments[@]}; position++)); do
+  word="${arguments[position]}"
+  if [[ "$word" == -- ]]; then after_separator=1; continue; fi
+  if (( after_separator )); then
+    case "$word" in --include-ignored) include_ignored=1 ;; --exact) exact=1 ;; --nocapture) ;; *) filter="$word" ;; esac
+  elif [[ "$word" == --test ]]; then target="${arguments[position + 1]}"
+  elif [[ "$word" == --release ]]; then release=1; fi
+done
+printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$target" "$filter" "$include_ignored" "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY-unset}" \
+  "${SINGLEFS_LAYER0_INPUT_FINGERPRINT-unset}" "${SINGLEFS_LAYER0_START_OVER-unset}" "$exact" "$release" >> "$FAKE_CARGO_CONTROL/invocations"
+if (( ! include_ignored )); then echo "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s"; exit 0; fi
+if [[ -f "$FAKE_CARGO_CONTROL/touch-during-run" ]]; then
+  echo "// 跑的过程中改的" >> "$(cat "$FAKE_CARGO_CONTROL/touch-during-run")"
+  rm -f "$FAKE_CARGO_CONTROL/touch-during-run"
+fi
+cat "$FAKE_CARGO_CONTROL/log.$target"
+exit "$(cat "$FAKE_CARGO_CONTROL/exit.$target" 2>/dev/null || echo 0)"
+'''
+LAYER0_STAGE_CASES = [
+    ("crash-case:stream-a", "first_transaction_step_seven_layer0", "stream_a_full", "count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0"),
+    ("crash-case:stream-b", "second_transaction_step_zero_layer0", "stream_b_full", "count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B"),
+    ("crash-case:case-c", "case_c", "case_c_full", ""),
+]
+PASSED_ONE_LINE = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s"
+
+
+def stream_log(prefix, worker_threads=32, resumed_slices=0, freshly_run_slices=64, result=PASSED_ONE_LINE):
+    return (f"LAYER0_PROGRESS slice=1/64\nLAYER0_PARALLEL_FINISHED states=100 slices=64 worker_threads={worker_threads} configured_worker_threads=32 "
+            f"worker_threads_source=environment_variable resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices} "
+            f"progress_file_after_completion=deleted elapsed_seconds=1.0\n{prefix} states=100 closed_form=100 violations=0 exhaustive=true\n{result}\n")
+
+
+def run_layer0_stage_cells(selftest, module):
+    """门禁 54 号这一份的流程：拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下，名字照旧是 54-layer0-replay.sh），
+    假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量、判绿写标记、快档核标记、第二趟全复用、只重跑输入变了的那一条、
+    --start-over、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记。"""
+    repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
+    real_stage = os.path.join(repository_root, ".claude/gate.d/54-layer0-replay.sh")
+    work = tempfile.mkdtemp(prefix="admission-selftest-stage54-")
+    try:
+        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
+        write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/singlefs-harness"]\n')
+        write_text(os.path.join(work, "crates/singlefs-harness/Cargo.toml"), '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
+        write_text(os.path.join(work, "crates/singlefs-harness/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
+        for _key, target, function, _conditions in LAYER0_STAGE_CASES:
+            write_text(os.path.join(work, f"crates/singlefs-harness/tests/{target}.rs"), f"#[test]\n#[ignore]\nfn {function}() {{}}\n")
+        rows = ["54-layer0-replay.sh\tcrates/ Cargo.toml\tcommand=cargo command=rustc\t# 样本"]
+        rows += [f"{key}\tcrates/ Cargo.toml\ttest=singlefs-harness:{target}:{function}{' ' + conditions if conditions else ''}\t# 样本"
+                 for key, target, function, conditions in LAYER0_STAGE_CASES]
+        write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in rows))
+        os.makedirs(os.path.join(work, "research/scripts"))
+        shutil.copy(module, os.path.join(work, "research/scripts/admission.py"))
+        for helper in ("stage-must-run.sh", "change-touches-crates.sh"):
+            shutil.copy(os.path.join(SELFTEST_HERE, helper), os.path.join(work, "research/scripts", helper))
+        stage_copy = os.path.join(work, ".claude/stage-under-test/54-layer0-replay.sh")
+        os.makedirs(os.path.dirname(stage_copy))
+        shutil.copy(real_stage, stage_copy)
+        write_text(os.path.join(work, ".gitignore"), ".control/\n.tools/\n")
+        tools = os.path.join(work, ".tools")
+        control = os.path.join(work, ".control")
+        os.makedirs(control)
+        write_executable(os.path.join(tools, "cargo"), FAKE_CARGO_FOR_STAGE)
+        write_executable(os.path.join(tools, "rustc"), FAKE_TOOLCHAIN_SCRIPTS["rustc"])
+        write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 32\n")
+        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
+                                                          "CARGO_HOME": os.path.join(tools, "cargo-home"), "FAKE_CARGO_CONTROL": control,
+                                                          "SINGLEFS_GATE_FULL": "1", "SINGLEFS_LAYER0_START_OVER": "1"})
+        environment.pop("SINGLEFS_LAYER0_THREADS", None)
+        common_directory = os.path.join(work, ".git")
+
+        def set_logs(stream_a=None, stream_b=None, case_c=None, exits=None):
+            write_text(os.path.join(control, "log.first_transaction_step_seven_layer0"), stream_a or stream_log("LAYER0") + "CHECKER x=0\n")
+            write_text(os.path.join(control, "log.second_transaction_step_zero_layer0"), stream_b or stream_log("LAYER0B"))
+            write_text(os.path.join(control, "log.case_c"), case_c or PASSED_ONE_LINE + "\n")
+            for target in ("first_transaction_step_seven_layer0", "second_transaction_step_zero_layer0", "case_c"):
+                exit_path = os.path.join(control, f"exit.{target}")
+                if os.path.exists(exit_path):
+                    os.remove(exit_path)
+            for target, code in (exits or {}).items():
+                write_text(os.path.join(control, f"exit.{target}"), f"{code}\n")
+
+        def invocations():
+            path = os.path.join(control, "invocations")
+            if not os.path.exists(path):
+                return []
+            with open(path, encoding="utf-8") as handle:
+                lines = [line.split("\t") for line in handle.read().split("\n") if line]
+            os.remove(path)
+            return lines
+
+        def stage(*arguments):
+            exit_code, output, messages = run_in_environment(["bash", stage_copy, *arguments, work], environment)
+            return exit_code, output + messages, invocations()
+
+        def expected_fingerprint(key):
+            _exit_code, output, _messages = run_in_environment(
+                [sys.executable, os.path.join(work, "research/scripts/admission.py"), "crash-case-manifest", work, key,
+                 os.path.join(control, "expected-manifest"), "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", stage_copy,
+                 "--toolchain", "--build-environment"], environment)
+            return output.split()[0] if output.split() else ""
+
+        def markers():
+            return sorted(name for name in os.listdir(common_directory) if name.startswith(CRASH_CASE_MARKER_PREFIX) and ".partial." not in name)
+
+        def full_runs(calls):
+            return [call for call in calls if call[2] == "1"]
+
+        set_logs()
+        exit_code, output, calls = stage("--full")
+        runs = full_runs(calls)
+        fingerprints = {key: expected_fingerprint(key) for key, _target, _function, _conditions in LAYER0_STAGE_CASES}
+        progress_settings_ok = all(
+            call[3] == os.path.join(common_directory, "singlefs-layer0-progress", fingerprints[key]) and call[4] == fingerprints[key] and call[5] == "unset"
+            for call, (key, _target, _function, _conditions) in zip(runs, LAYER0_STAGE_CASES))
+        selftest.expect("54 号 --full 头一趟：三条用例在 release 下逐条跑（--include-ignored --exact 各自的用例函数），各写一格标记",
+                        exit_code == 0 and [(call[0], call[1], call[6], call[7]) for call in runs]
+                        == [(target, function, "1", "1") for _key, target, function, _c in LAYER0_STAGE_CASES]
+                        and len(markers()) == 3, f"退 {exit_code}，跑了 {runs}，标记 {markers()}，输出尾部：{output.strip()[-800:]}")
+        selftest.expect("54 号 --full 设续跑的环境变量：进度目录 <common-dir>/singlefs-layer0-progress/<这条用例的指纹>、输入指纹是这条用例的；"
+                        "调用方环境里的 SINGLEFS_LAYER0_START_OVER=1 在不带 --start-over 时被清掉",
+                        progress_settings_ok and len(set(fingerprints.values())) == 3, f"跑的时候看到 {runs}，这几条用例的指纹 {fingerprints}")
+        exit_code, output, calls = stage()
+        selftest.expect("54 号快档：两条流跑不标 ignored 的用例，三条用例的标记都作数，判绿",
+                        exit_code == 0 and all(key in output for key, _t, _f, _c in LAYER0_STAGE_CASES) and not full_runs(calls)
+                        and len(calls) == 2, f"退 {exit_code}，cargo 调了 {calls}，输出尾部：{output.strip()[-600:]}")
+        exit_code, output, calls = stage("--full")
+        selftest.expect("54 号 --full 第二趟：输入没变，三条全复用、一条都不跑", exit_code == 0 and not full_runs(calls) and output.count(" 复用：") == 3,
+                        f"退 {exit_code}，跑了 {full_runs(calls)}，输出尾部：{output.strip()[-600:]}")
+        write_text(os.path.join(work, "crates/singlefs-harness/tests/case_c.rs"), "#[test]\n#[ignore]\nfn case_c_full() { let changed = 1; }\n")
+        exit_code, output, calls = stage("--full")
+        selftest.expect("54 号 --full：只改了 case_c 独占的测试文件，只重跑 case_c，另两条复用",
+                        exit_code == 0 and [call[0] for call in full_runs(calls)] == ["case_c"], f"退 {exit_code}，跑了 {full_runs(calls)}")
+        stream_b_marker = [name for name in markers() if ".stream-b." in name]
+        for name in stream_b_marker:
+            os.remove(os.path.join(common_directory, name))
+        exit_code, output, calls = stage()
+        selftest.expect("54 号快档：删掉 stream-b 那一格标记就判红，点名 crash-case:stream-b，出路是 --full",
+                        exit_code == 1 and "✗" in output and "crash-case:stream-b" in output and "--full" in output,
+                        f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
+        set_logs(stream_b=stream_log("LAYER0B", worker_threads=1, resumed_slices=63, freshly_run_slices=1))
+        exit_code, output, calls = stage("--full", "--start-over")
+        runs = full_runs(calls)
+        selftest.expect("54 号 --full --start-over：设 SINGLEFS_LAYER0_START_OVER=1；只剩 1 片要跑、起 1 个线程不误红，写回那一格",
+                        exit_code == 0 and [(call[0], call[5]) for call in runs] == [("second_transaction_step_zero_layer0", "1")]
+                        and any(".stream-b." in name for name in markers()), f"退 {exit_code}，跑了 {runs}，输出尾部：{output.strip()[-600:]}")
+        exit_code, output, _calls = run_in_environment(["bash", stage_copy, "--start-over", work], environment)
+        selftest.expect("54 号 --start-over 不带 --full：退 2，说只跟 --full 一起用", exit_code == 2 and "只跟 --full 一起用" in output,
+                        f"退 {exit_code}，输出「{output.strip()}」")
+        stream_b_markers = [name for name in markers() if ".stream-b." in name]
+        stream_b_marker_path = os.path.join(common_directory, stream_b_markers[0]) if stream_b_markers else ""
+        if stream_b_marker_path:
+            with open(stream_b_marker_path, encoding="utf-8") as handle:
+                tampered_marker = handle.read().replace("exhaustive=true", "exhaustive=false")
+            write_text(stream_b_marker_path, tampered_marker)
+        set_logs(stream_b=stream_log("LAYER0B", worker_threads=1))
+        exit_code, output, calls = stage("--full")
+        selftest.expect("54 号 --full：这批输入那一格在而不作数就重跑，重跑判红时删掉那一格（先绿后红，前一趟的不再作数）",
+                        bool(stream_b_marker_path) and exit_code == 1 and [call[0] for call in full_runs(calls)] == ["second_transaction_step_zero_layer0"]
+                        and not os.path.exists(stream_b_marker_path),
+                        f"改之前 stream-b 那一格{'在' if stream_b_marker_path else '不在（上一格没写成）'}；退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}")
+        red_cells = [
+            ("跑了 64 片却只起 1 个线程", {"stream_b": stream_log("LAYER0B", worker_threads=1)}, None, "只起了 1 个工作线程"),
+            ("过滤之后一条用例都没跑（0 passed）",
+             {"stream_b": stream_log("LAYER0B", result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s")},
+             None, "1 passed"),
+            ("cargo test 退非 0", {}, {"second_transaction_step_zero_layer0": 101}, "cargo test 退非 0"),
+        ]
+        for label, logs, exits, phrase in red_cells:
+            for name in [name for name in markers() if ".stream-b." in name]:
+                os.remove(os.path.join(common_directory, name))
+            set_logs(stream_b=logs.get("stream_b"), exits=exits)
+            exit_code, output, calls = stage("--full")
+            selftest.expect(f"54 号 --full：stream-b {label} ⇒ 这一条判红、不写标记，退 1",
+                            exit_code == 1 and phrase in output and not any(".stream-b." in name for name in markers())
+                            and [call[0] for call in full_runs(calls)] == ["second_transaction_step_zero_layer0"],
+                            f"退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
+        for name in [name for name in markers() if ".case-c." in name]:
+            os.remove(os.path.join(common_directory, name))
+        set_logs(exits={"first_transaction_step_seven_layer0": 101})
+        for name in [name for name in markers() if ".stream-a." in name]:
+            os.remove(os.path.join(common_directory, name))
+        exit_code, output, calls = stage("--full")
+        selftest.expect("54 号 --full：stream-a 判红之后接着跑下一条，stream-b 与 case-c 照样判绿、写标记",
+                        exit_code == 1 and [call[0] for call in full_runs(calls)] == [target for _k, target, _f, _c in LAYER0_STAGE_CASES]
+                        and not any(".stream-a." in name for name in markers())
+                        and any(".stream-b." in name for name in markers()) and any(".case-c." in name for name in markers()),
+                        f"退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}")
+        set_logs()
+        write_text(os.path.join(control, "touch-during-run"), os.path.join(work, "crates/singlefs-harness/src/lib.rs"))
+        exit_code, output, calls = stage("--full")
+        selftest.expect("54 号 --full：跑的过程中共用的 src/lib.rs 被改了 ⇒ 那一条判红、不写标记，出路里列出变了的文件",
+                        exit_code == 1 and "跑的过程中它的输入变了" in output and "crates/singlefs-harness/src/lib.rs" in output
+                        and not any(".stream-a." in name for name in markers()), f"退 {exit_code}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
+    except (OSError, shutil.Error) as error:
+        selftest.expect("54 号这一份拷得进临时仓、跑得起来", False, f"{error}（真仓的 54 号在 {real_stage}）")
+    finally:
+        shutil.rmtree(work, ignore_errors=True)
+
+
+# manifest 子命令（不带 --build-environment）同一套步骤的另一份实现，只给自证当对照：它不 import 本模块，用的是 sha256sum 与 sort。
 INDEPENDENT_MANIFEST_SCRIPT = r'''
 set -uo pipefail
 root="$1"; stage_script="$2"
@@ -1447,6 +2787,12 @@
     "keys": command_keys,
     "gate-preconditions": command_gate_preconditions,
     "gate-record-environment": command_gate_record_environment,
+    "crash-cases": command_crash_cases,
+    "crash-case-manifest": command_crash_case_manifest,
+    "crash-case-marker-check": command_crash_case_marker_check,
+    "crash-case-judge": command_crash_case_judge,
+    "crash-case-record": command_crash_case_record,
+    "crash-case-marker-path": command_crash_case_marker_path,
 }
 
 
@@ -1454,7 +2800,8 @@
     if arguments[:1] == ["--selftest"]:
         return run_selftest()
     if not arguments or arguments[0] not in COMMANDS:
-        say("  ✗ 用法：admission.py {gate-reuse|gate-preconditions|gate-record-environment|experiment|paths|manifest|keys} … 或 --selftest")
+        say("  ✗ 用法：admission.py {gate-reuse|gate-preconditions|gate-record-environment|experiment|paths|manifest|keys|crash-cases|"
+            "crash-case-manifest|crash-case-marker-check|crash-case-judge|crash-case-record|crash-case-marker-path} … 或 --selftest")
         say("     → 怎么办：各子命令的参数见文件头")
         return EXIT_REGISTRATION_ERROR
     return COMMANDS[arguments[0]](arguments[1:])
--- a/.claude/gate.d/stage-inputs.tsv
+++ b/.claude/gate.d/stage-inputs.tsv
@@ -10,6 +10,10 @@
 #   实验（键是实验号；同一个实验另有一种调用方式的写 E<号>/<方式>）：装置开跑前调 admission.py experiment：
 #     这几条路径下每个文件的 sha256、登记行本身与工具链汇成输入指纹，research/results/ 里有一份产物头上记着同一个键、同一个指纹 ⇒ 拒绝重跑；
 #     第三列的准入条件没齐 ⇒ 拒绝开跑。路径写成 @<别的键> 就是那个键登记的全部路径。
+#   崩溃枚举用例（键是 crash-case:<名>）：门禁 54 号逐条按复用判定跑、逐条记全绿标记（在 git common-dir，按用例与它的输入指纹分格），
+#     输入没变复用、变了才重跑。路径用排除法写：只写整个 crates/ 与 Cargo 清单、锁，不按用例手列它读哪些文件——admission.py 算输入时
+#     自动减去别的测试目标独占的测试文件（判法见它的「崩溃枚举用例」一节），再加判它的 54 号、工具链、构建环境与这一行本身。
+#     第三列：test=<包>:<测试目标>:<用例函数> 恰好一条，count-line= / exhaustive= / threads= 定日志怎么判（写法见 admission.py 文件头）。
 #
 # ⚠️ 这份清单**自己也进门禁的比对**（stage-must-run.sh 无条件把它加进路径列表）。少写一条输入，那条输入就永远
 # 不会让这道阶段重跑——而清单本身变了必定重跑，改清单的代价因此是「下一趟全跑一次」，不是零；加一个实验行也一样。
@@ -17,7 +21,7 @@
 #
 # 宁宽勿窄：多写一条路径只会多跑几趟，少写一条会让一次真的改动被跳过。拿不准就写上。
 # 只写从仓库根起的路径；目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）。
-54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	command=cargo command=rustc	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入哈希，快档与 --full 都起 cargo test）
+54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	command=cargo command=rustc	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）。这一行管整道阶段的复用判定与改动范围；它逐条跑的崩溃枚举用例各自的输入登记在本表 crash-case: 开头的那几行。前提：cargo 与 rustc（两档都要 cargo -V && rustc -V 算输入指纹，快档与 --full 都起 cargo test）
 55-qemu-first-transaction.sh	crates/ Cargo.toml Cargo.lock research/scripts/ research/results/	command=qemu-system-x86_64 readwrite=/dev/kvm probe=research/scripts/vm-kernel.sh:--check	# 起虚机跑 crates 的装置，虚机与复跑脚本都在 research/scripts/；这几条路径同时是改动范围（change-touches-crates.sh）的前缀。前提是 .claude/kb/vm-harness.md「三个前置」：缺 QEMU 装 qemu-system-x86；/dev/kvm 不可读写查 kvm 组成员身份，不许用 setfacl 补；找不到可读的内核镜像就跑 bash research/scripts/vm-kernel.sh，按它打印的路径设 SINGLEFS_KERNEL
 57-lkmm.sh	litmus/ .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh .claude/singlefs-ai-sop/scripts/lib.sh crates/	probe=.claude/scripts/lkmm.sh:--herd7-version environment=.claude/scripts/lkmm.sh:--herd7-version	# 判 litmus/ 下每条 Never 的对照组、代码绑定与 herd7 判定。lkmm.sh 是判法，它 source 的 lib.sh、找不到内核树时调的 fetch-deps.sh 一并登记；crates/ 整个取：litmus 的 singlefs-models 锚点今天指 crates/singlefs-core/src/transaction.rs 与 recovery.rs，读 litmus 文件名的测试在 crates/singlefs-harness/tests/publish_order_matches_litmus.rs，而 lkmm.sh 按文件名在全部 crates/**/*.rs 里找测试、新加一条 litmus 的锚点可以指到 crates/ 任何一处，只登记这三份会让下一条新锚点的改动被跳过（宁宽勿窄）。herd7 的版本不在 git 树里，经第三列 environment= 进复用判定（lkmm.sh --herd7-version 打的那一行，57 号判绿之后记进 git common-dir）；前提是找得到 herd7（probe= 同一个入口，PATH 里没有就试 opam 的环境；缺了 opam install herdtools7）。内核树的 tools/memory-model 不进判定，靠 stage-must-run.sh 的 24 小时复用上限兜
 59-crates-mutation-replay.sh	crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh	command=cargo	# 按 crates/mutations.tsv 逐条改坏 crates/ 里的源码再跑点名的测试；每条经 run-with-memory-cap.sh 带内存上限跑，它判不判得出撞顶也是这一道的判据。前提：cargo（装 Rust 工具链，bash .claude/scripts/env.sh 会报缺什么）
@@ -25,3 +29,7 @@
 87-replay.sh	crates/ Cargo.toml Cargo.lock research/e7-index-bench/ research/scripts/ research/results/	# 复跑入库的实验产物，装置在 research/e7-index-bench/，登记表与脚本在 research/scripts/
 E142	research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs research/e7-index-bench/src/lib.rs research/e7-index-bench/Cargo.toml research/Cargo.toml research/Cargo.lock research/mutations/e142_first_transaction_dry_run.tsv research/results/e142-first-txn-dry-run-2026-09-25-r16-arm-n15.out .claude/kb/decisions/08-核心索引结构.md .claude/kb/decisions/13-验证路线.md .claude/kb/decisions/16-发布语义.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/layout/01-first-txn.md .claude/kb/decisions/22-单元原子性怎么合成.md .claude/kb/decisions/15-格式冻结政策.md .claude/kb/feature-bits.md research/e7-index-bench/src/bin/e142_region_diff_independent.rs research/mutations/e142_region_diff_independent.tsv crates/ Cargo.toml Cargo.lock	# E142（第一个事务的干跑）默认调用（replay.sh 的 driver_e142 那一种）。装置源码、它 use 的 e7_index_bench 库、research 的 Cargo 清单与锁、变异表；driver_e142 传给装置的臂 N15 参照产物；replay.sh 不进（跑完把登记行指到新产物就是改它，进了指纹，新产物一存进来就对不上自己；驱动换了参数而这几条路径没变时要靠强制开关）；跑前登记 research/prompts/e142-r17-prereg.md 第二节被测条款所在的 D8（核心索引结构）、D13（验证路线）、D16（发布语义）、D18（块里携带什么信息）、D23（journal 的角色与格式）五份决策正文，加上宽度对账逐格抄的 layout/01（跑前登记第二节没列它，宁宽勿窄加上）；第十八次跑的跑前登记 research/prompts/e142-r18-prereg.md 第二节另加被测条款 D22（单元原子性怎么合成）、D15（格式冻结政策）与 feature-bits.md，那一次用独立比对 bin e142_region_diff_independent 判改前改后，它与它的变异表一并登记（2026-09-26 主 agent 按设计员报的漏列补上）；crates/ 整个取：driver_e142 编的 e142_first_transaction_write_dump 在 singlefs-harness，它依赖 checker、core、format 另外三个 crate，四个就是整个 crates/，多出来的只有 crates/mutations.tsv；按文件精确取要跟着模块图走，漏一个模块就是该跑的不跑
 E142/layer0	@E142	question-row=research/prompts/m2-keyspace-rerun-questions.md#6:够判[：:][^（(|]*对照(本身)?是好的 product-field=E142:verdict:control_violations_ok=true product-field=E142:verdict:positive_control_main_geometry_ok=true	# E142 第二段（设了 E142_LAYER0_MAIN 的调用：主臂层 0 整轮），输入与默认调用相同。前提照跑前登记 research/prompts/e142-r17-prereg.md 6.2、6.3：问题单第 6 行判成「对照是好的」之后才跑，且 Q142.26 caught = 12（最新产物判决行 positive_control_main_geometry_ok）；control_violations_ok 是第 6 行够判那一格的计数判定
+crash-case:layer0-first-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:first_transaction_step_seven_layer0:layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0	# 第一条流（mkfs → 取号 → 暖机 → A）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之一。LAYER0 是计数行、要 exhaustive=true，CHECKER 是逐条不变量行
+crash-case:layer0-second-stream	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_step_zero_layer0:full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B	# 第二条流（里程碑「第二个事务」固定脚本到 E 再正常卸载）的层 0 全量，带断点续跑；54 号 --full 原来整批跑的两条之二
+crash-case:floor-raise-pushed-by-the-session	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_crash_inside_the_floor_raise_pushed_by_the_session:crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green	# 崩在会话推的那一串抬 F 中间：按层 0 的枚举域枚举那一段、恢复之后与再挂载之后各判池级 checker（标了 ignore，release 跑）；不留进度文件，计数由用例自己断言，不打计数行
+crash-case:c561-sigma-full	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:record_checker_judges_absence_by_the_persisted_set:every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present count-line=C561_SIGMA_FULL	# C561（记录核对器的复用豁免在复用只落一半时假红） 的 σ 全量：σ 那一段 2^18 个状态记录核对器一个都不判缺席（标了 ignore，release 跑）
--- a/.claude/gate.d/stage-owners.tsv
+++ b/.claude/gate.d/stage-owners.tsv
@@ -34,7 +34,7 @@
 51-admission-terms-covered.sh	kb-scribe	准入不等式各项有人维护
 52-segment-registry.sh	kb-scribe,experiment-runner	字节表段序列登记与 E142 产物两边
 53-format-const-placeholders.sh	implementation-writer,kb-scribe	crates 常量文件的占位与 kb 分项两边
-54-layer0-replay.sh	crash-verifier	层 0 崩溃点重放
+54-layer0-replay.sh	crash-verifier	层 0 崩溃点重放与 stage-inputs.tsv 里登记的崩溃枚举用例（crash-case: 行，逐条按输入复用、只重跑输入变了的）：提交时在 HEAD + 暂存区的 worktree 里跑 --full
 55-qemu-first-transaction.sh	crash-verifier	QEMU 真设备
 56-crates-adversarial-review.sh	gate-triage	判决文件由主 agent 写，提交前核点名
 57-lkmm.sh	crash-verifier	herd7 内存序
--- a/.claude/hooks/lib_heavy_tests.py
+++ b/.claude/hooks/lib_heavy_tests.py
@@ -21,6 +21,9 @@
 
 认的输入：cargo 命令行（test 与 run）、按名字认的脚本与门禁阶段、qemu-system-*、herd7，
 以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`：按 <名字> 判，名字含 layer0 的算层 0。
+崩溃枚举用例（门禁 54 号逐条跑的那几条，登记在 .claude/gate.d/stage-inputs.tsv 键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>）：
+跑到它们的测试目标、libtest 参数带 --ignored 或 --include-ignored 的算重型（cargo test 点名它、通配命中它、或不挑目标而包里有它；直接执行它的测试二进制）。
+登记表取两份的并：从命令的当前目录（有 --manifest-path 时取它所在的目录）往上找到的第一份，与这份文件所在仓的那一份。
 """
 import fnmatch, glob, importlib.util, os, re, shlex, shutil, sys, tempfile, tomllib
 from typing import NamedTuple
@@ -51,6 +54,7 @@
     "full-cargo": "全量测试", "check-sh": "全量测试",
     "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
     "e152": "E152 装置",
+    "crash-case-cargo": "崩溃枚举用例", "crash-case-binary": "崩溃枚举用例",
 }
 # 只有这几道阶段是重型；.claude/gate.d/ 下其余阶段谁都能跑
 STAGE_KIND = {"54": "layer0-stage", "55": "qemu-stage", "57": "herd7-stage", "59": "crates-mutation-stage", "87": "replay-all-stage"}
@@ -67,6 +71,51 @@
     return HeavyTest(kind, KIND_CATEGORY[kind], detail)
 
 
+# 崩溃枚举用例的登记表（与 research/scripts/admission.py 读的是同一份）：键是 crash-case: 的行，第三列 test=<包>:<测试目标>:<用例函数>
+CRASH_CASE_REGISTRY = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
+CRASH_CASE_TEST_CONDITION = re.compile(r"^test=(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):[A-Za-z_][A-Za-z0-9_]*$")
+HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
+IGNORED_TEST_ARGUMENTS = {"--ignored", "--include-ignored"}
+
+
+def crash_case_registries(directory):
+    """要读的登记表：从 directory 往上找到的第一份，加这份文件所在仓的那一份（两份是同一个文件时只算一次）。"""
+    found = []
+    while directory:
+        candidate = os.path.join(directory, CRASH_CASE_REGISTRY)
+        if os.path.isfile(candidate):
+            found.append(candidate)
+            break
+        parent = os.path.dirname(directory)
+        if parent == directory:
+            break
+        directory = parent
+    own = os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)
+    if os.path.isfile(own) and all(os.path.realpath(own) != os.path.realpath(path) for path in found):
+        found.append(own)
+    return found
+
+
+def registered_crash_case_targets(directory):
+    """登记的崩溃枚举用例：{(包名, 测试目标)}。读不了的登记表当它没有。"""
+    targets = set()
+    for path in crash_case_registries(directory):
+        try:
+            with open(path, encoding="utf-8", errors="replace") as handle:
+                lines = handle.read().split("\n")
+        except OSError:
+            continue
+        for line in lines:
+            columns = line.split("\t")
+            if not columns[0].startswith("crash-case:") or len(columns) < 3 or columns[2].lstrip().startswith("#"):
+                continue
+            for token in columns[2].split():
+                match = CRASH_CASE_TEST_CONDITION.match(token)
+                if match:
+                    targets.add((match.group("package"), match.group("target")))
+    return targets
+
+
 def manifest_sections(path):
     try:
         with open(path, "rb") as handle:
@@ -207,7 +256,15 @@
     named_layer0 = [name for name in test_names if "layer0" in name]
     if named_layer0:
         return heavy_test("layer0-cargo", f"cargo test --test {named_layer0[0]}")
+    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
+    runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(libtest_arguments))
     manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
+    crash_cases = registered_crash_case_targets(os.path.dirname(manifest_path) if manifest_path else directory) if runs_ignored else set()
+    for pattern in test_names:
+        crash_case_names = sorted({target for _package, target in crash_cases})
+        matched = fnmatch.filter(crash_case_names, pattern) if GLOB_CHARACTERS & set(pattern) else [name for name in crash_case_names if name == pattern]
+        if matched:
+            return heavy_test("crash-case-cargo", f"cargo test --test {pattern} 带 {runs_ignored[0]}，跑到登记的崩溃枚举用例（{'、'.join(matched)}）")
     sections = manifest_sections(manifest_path) if manifest_path else None
     if sections is None:
         return None
@@ -227,6 +284,10 @@
         return heavy_test("full-cargo", (f"cargo test 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                          f"{len(members)} 个成员），等于全量"))
     layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
+    unselected = not selectors or bool(selectors & {"--tests", "--all-targets"})
+    crash_cases_in_scope = sorted(target for package, target in crash_cases if members.get(package) in scope)
+    if not layer0_targets and unselected and crash_cases_in_scope:
+        return heavy_test("crash-case-cargo", f"cargo test 不挑目标、带 {runs_ignored[0]}，会跑到登记的崩溃枚举用例（{'、'.join(crash_cases_in_scope)}）")
     if not layer0_targets:
         return None
     if not selectors or selectors & {"--tests", "--all-targets"}:
@@ -289,7 +350,12 @@
         return heavy_test(STAGE_KIND[stage], f"门禁 {stage} 号（{name}）") if stage in STAGE_KIND else None
     binary = test_binary_name(words[0])
     if binary is not None:
-        return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）") if "layer0" in binary else None
+        if "layer0" in binary:
+            return heavy_test("layer0-binary", f"直接执行名字含 layer0 的测试二进制（{binary}）")
+        runs_ignored = sorted(IGNORED_TEST_ARGUMENTS & set(arguments))
+        if runs_ignored and binary in {target for _package, target in registered_crash_case_targets(directory)}:
+            return heavy_test("crash-case-binary", f"直接执行登记的崩溃枚举用例的测试二进制（{binary}）、带 {runs_ignored[0]}")
+        return None
     if name.startswith("qemu-system"):
         return heavy_test("qemu-system", name)
     if name == "vm-bench.sh":
@@ -341,18 +407,26 @@
 
 
 def build_sample_workspace(work):
-    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标。"""
+    """仓根与 research/ 两个虚工作区，harness 里有一个名字含 layer0 的测试目标；登记表登记两条崩溃枚举用例：
+    harness 里的 sample_crash_enumeration，与只有它一个测试目标的包 singlefs-checker 里的 checker_crash_enumeration。"""
     def write(relative, text):
         path = os.path.join(work, relative)
         os.makedirs(os.path.dirname(path), exist_ok=True)
         with open(path, "w", encoding="utf-8") as handle:
             handle.write(text)
-    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness"]\nexclude = ["research"]\n')
+    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness", "crates/singlefs-checker"]\n'
+                        'exclude = ["research"]\n')
     write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
     write("crates/singlefs-core/tests/core_contract.rs", "")
     write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
     write("crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs", "")
     write("crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs", "")
+    write("crates/singlefs-harness/tests/sample_crash_enumeration.rs", "")
+    write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
+    write("crates/singlefs-checker/tests/checker_crash_enumeration.rs", "")
+    write(CRASH_CASE_REGISTRY, "# 样本登记表\n54-layer0-replay.sh\tcrates/\t# 样本\n"
+                               "crash-case:sample\tcrates/ Cargo.toml\ttest=singlefs-harness:sample_crash_enumeration:the_full_case\t# 样本\n"
+                               "crash-case:checker\tcrates/ Cargo.toml\ttest=singlefs-checker:checker_crash_enumeration:the_full_case\t# 样本\n")
     write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
     write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')
 
@@ -379,6 +453,20 @@
             ("一个不是层 0 的测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
             ("clippy --all-targets", ["cargo", "clippy", "--all-targets"], work, None),
             ("bash 起轻阶段 12 号", ["bash", ".claude/gate.d/12-no-prime-marks.sh"], work, None),
+            ("点名崩溃枚举用例、带 --include-ignored --exact",
+             ["cargo", "test", "--release", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--include-ignored", "--exact", "the_full_case"],
+             work, "crash-case-cargo"),
+            ("点名崩溃枚举用例、带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--ignored"], work,
+             "crash-case-cargo"),
+            ("--test 通配命中崩溃枚举用例、带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_*", "--", "--ignored"], work,
+             "crash-case-cargo"),
+            ("只有崩溃枚举用例的包不挑目标、带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--", "--ignored"], work, "crash-case-cargo"),
+            ("点名崩溃枚举用例、不带 --ignored（只跑它的快用例）", ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration"], work, None),
+            ("点名崩溃枚举用例、libtest 参数只有 --nocapture",
+             ["cargo", "test", "-p", "singlefs-harness", "--test", "sample_crash_enumeration", "--", "--nocapture"], work, None),
+            ("只有崩溃枚举用例的包 --lib 带 --ignored", ["cargo", "test", "-p", "singlefs-checker", "--lib", "--", "--ignored"], work, None),
+            ("别的测试目标带 --ignored", ["cargo", "test", "-p", "singlefs-harness", "--test", "second_transaction_step_one_overwrite", "--", "--ignored"],
+             work, None),
         ]
         binary_cases = [
             ("绝对路径的层 0 测试二进制", [os.path.join(work, "target", "release", "deps", layer0_binary), "--test-threads", "4"], work, "layer0-binary"),
@@ -389,7 +477,20 @@
             ("deps 下的 .d 依赖文件不是二进制", [os.path.join(work, "target", "release", "deps", layer0_binary + ".d")], work, None),
             ("哈希不是 16 位", [os.path.join(work, "target", "release", "deps", "first_transaction_step_seven_layer0-0123abcd")], work, None),
             ("名字只当参数", ["grep", "-c", "x", os.path.join("target", "release", "deps", layer0_binary)], work, None),
+            ("崩溃枚举用例的测试二进制带 --include-ignored",
+             [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef"), "--include-ignored", "--exact", "the_full_case"],
+             work, "crash-case-binary"),
+            ("崩溃枚举用例的测试二进制不带 --ignored", [os.path.join(work, "target", "release", "deps", "sample_crash_enumeration-0123456789abcdef")], work, None),
         ]
+        # 名字不含 layer0 的那几条：含 layer0 的按层 0 那一类先认出来，判不出登记表取没取到
+        own_repository_targets = sorted(target for _package, target in registered_crash_case_targets(HOOK_REPOSITORY) if "layer0" not in target)
+        if own_repository_targets:
+            binary_cases.append(("仓外执行、登记表取这份文件所在仓的那一份：那里登记的崩溃枚举用例带 --ignored",
+                                 [f"/tmp/target-elsewhere/release/deps/{own_repository_targets[0]}-0123456789abcdef", "--ignored"], "/",
+                                 "crash-case-binary"))
+        else:
+            results.append((f"这份文件所在仓的登记表（{os.path.join(HOOK_REPOSITORY, CRASH_CASE_REGISTRY)}）里有名字不含 layer0 的崩溃枚举用例",
+                            True, False))
         for label, argv, cwd, want in cargo_cases + binary_cases:
             found = classify_process(argv, cwd)
             results.append((label, want, found[0].kind if found else None))
--- a/.claude/hooks/heavy-test-guard.sh
+++ b/.claude/hooks/heavy-test-guard.sh
@@ -33,10 +33,12 @@
 #   全部实验复跑  .claude/gate.d/87-*
 #   整轮门禁      gate.sh、research/scripts/gate-staged.sh（--selftest 不算）
 #   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
+#   崩溃枚举用例  .claude/gate.d/stage-inputs.tsv 里键是 crash-case: 的那几条用例（门禁 54 号逐条跑）的测试目标，libtest 参数带 --ignored 或
+#                 --include-ignored：cargo test 点名它、--test 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制（判定与登记表取法在 lib_heavy_tests.py）
 #   .claude/gate.d/ 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑、不用带前缀。
 # 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
 #   主 agent（输入里没有 agent_type）：上面每一类；
-#   crash-verifier：层 0、55 号与 qemu-system-*、herd7、crates 变异整表（vm-bench.sh、全量测试、整轮门禁、E152 拒）；
+#   crash-verifier：层 0、崩溃枚举用例、55 号与 qemu-system-*、herd7、crates 变异整表（vm-bench.sh、全量测试、整轮门禁、E152 拒）；
 #   gate-triage：整轮门禁（gate.sh、gate-staged.sh）与 87 号（gate.sh 里 54 / 55 / 57 / 59 靠「输入没变就复用上一次全绿判定」，直接调它们拒）；
 #   其余子 agent：一律拒，带不带前缀都拒。
 # 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
@@ -126,8 +128,8 @@
 
 # 子 agent 自己那一份（kind 见 lib_heavy_tests.KIND_CATEGORY）；不在表里的子 agent 一样也不许
 AGENT_KINDS = {
-    "crash-verifier": {"layer0-stage", "layer0-cargo", "layer0-binary", "qemu-stage", "qemu-system", "herd7-stage", "lkmm", "herd7",
-                       "crates-mutation-stage", "crates-mutation-mutate"},
+    "crash-verifier": {"layer0-stage", "layer0-cargo", "layer0-binary", "crash-case-cargo", "crash-case-binary", "qemu-stage", "qemu-system",
+                       "herd7-stage", "lkmm", "herd7", "crates-mutation-stage", "crates-mutation-mutate"},
     "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"},
 }
 
@@ -513,10 +515,10 @@
                                                  [run._replace(frames=run.frames[len(here):]) for run in inner.uncapped])
     return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)
 
-POLICY = ("→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；"
+POLICY = ("→ 规矩：重型测试（层 0、崩溃枚举用例、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；"
           "子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
           "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
-          "→ 各自那一份：crash-verifier 只跑 54、55、57、59 号与它们底下的层 0 测试、qemu-system、lkmm.sh / herd7、crates 变异整表；"
+          "→ 各自那一份：crash-verifier 只跑 54、55、57、59 号与它们底下的层 0 测试、登记的崩溃枚举用例、qemu-system、lkmm.sh / herd7、crates 变异整表；"
           "gate-triage 只跑 `gate.sh` 整轮与 87 号（54、55、57、59 靠「输入没变就复用上一次全绿判定」）；两个都要带那个前缀，都不跑全量 `cargo test`。"
           "`.claude/gate.d/` 下其余阶段不是重型，谁都能跑。\n"
           "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
@@ -720,6 +722,16 @@
             ("崩溃验证员带前缀跑层 0 测试目标", crash,
              commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 0),
             ("崩溃验证员带前缀直接执行层 0 测试二进制", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G " + layer0_binary, 0),
+            ("实现员跑登记的崩溃枚举用例（带 --include-ignored）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test --release -p singlefs-harness --test sample_crash_enumeration -- --include-ignored --exact the_full_case", 2),
+            ("实现员跑崩溃枚举用例那个测试目标的快用例（不带 --ignored）", writer,
+             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test sample_crash_enumeration", 0),
+            ("崩溃验证员不带前缀跑崩溃枚举用例", crash,
+             "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test sample_crash_enumeration -- --ignored", 2),
+            ("崩溃验证员带前缀跑崩溃枚举用例", crash,
+             commit + "bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test sample_crash_enumeration -- --include-ignored --exact the_full_case", 0),
+            ("主 agent 不带前缀直接执行崩溃枚举用例的测试二进制", None, "./target/release/deps/sample_crash_enumeration-0123456789abcdef --ignored", 2),
+            ("主 agent 带前缀直接执行崩溃枚举用例的测试二进制", None, commit + "./target/release/deps/sample_crash_enumeration-0123456789abcdef --ignored", 0),
             ("崩溃验证员带 =user-request 跑 55 号", crash, request + "bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
             ("崩溃验证员带前缀跑 59 号", crash, "GATE_MUTATION_TARGET_DIR=/tmp/t " + commit + "nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh", 0),
             ("门禁分诊带前缀跑 gate.sh --staged", triage, commit + "nice -n 19 bash .claude/scripts/gate.sh --staged", 0),
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -1,6 +1,6 @@
 ---
 name: crash-verifier
-description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
+description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
 tools: Read, Bash
 model: opus
 effort: high
@@ -19,21 +19,22 @@
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 54、55、57 号各自的内存上限（第 1b 步用），每道一个带单位的上限（例 16G）：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值，且那一行「线程数」一列与这一次跑的线程数相同）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
+- 54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
    1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
-2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑（`bash <worktree>/.claude/gate.d/54-layer0-replay.sh --full <worktree>`，经内存包装照第 1b 步）：它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它；自己建的 worktree 跑完用 `git worktree remove --force` 删；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
-4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
+4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行；54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
 6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容，不只按名单），前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。
 
 ## 写范围
 
-- 报告文件、草稿目录。阶段自己用的临时目录与编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）是阶段本身的行为；你不改仓里任何文件。
+- 报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物（59 号的 `GATE_MUTATION_TARGET_DIR` 等）与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。
 
 ## 产出
 
@@ -42,4 +43,4 @@
 ## 没做什么（固定会有的）
 
 - 没修任何一处红，也不判红是不是这一轮的改动造成的（交主 agent 或 `gate-triage`）。
-- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、哪些没进来，看 54 号头部，不由你外推。
+- 全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -55,6 +55,7 @@
 - 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
 - 全部实验复跑：87 号。
 - E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
+- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标，libtest 参数带 `--ignored` 或 `--include-ignored` 的——`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制。不带这两个参数、只跑那个测试目标里快用例的不算。
 
 只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
@@ -85,4 +86,4 @@
 - **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
 - **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
 
-**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`.claude/gate.d/54-layer0-replay.sh`；红的那一支只拿合成日志核过）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
+**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
```

## 二、主 agent 对 `research/scripts/check-segment-registry.py` 的两处修补（`diff -u` 改前备份 vs 仓里今天这份）

改前备份：`/tmp/claude-1000/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/scratchpad/check-segment-registry.py.before`（2026-09-22，改前）。今天这份：`research/scripts/check-segment-registry.py`（2026-09-26，改后）。

```diff
--- /tmp/claude-1000/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/scratchpad/check-segment-registry.py.before	2026-09-23
+++ research/scripts/check-segment-registry.py	2026-09-26
@@ -357,9 +357,14 @@
         array_count = len(array_pattern.findall(normalized_test_text))
         if array_count == 0:
             mismatches.append(f'{label}：用例文件 {test_file_relative_path} 里没有钉这个数组 `{array_text}`')
-        if str(claimed_crash_state_count) not in normalized_test_text:
+        # Rust 的长整数字面量带下划线分组（clippy unreadable_literal 要求，例 `6_649_413_746`），比状态数之前把数字之间的下划线去掉
+        digits_joined_test_text = re.sub(r'(?<=\d)_(?=\d)', '', normalized_test_text)
+        if str(claimed_crash_state_count) not in digits_joined_test_text:
             mismatches.append(f'{label}：用例文件 {test_file_relative_path} 里没有钉状态数 {claimed_crash_state_count}')
         checked_descriptions.append(f'{label}（{claimed_operation_count} 次写、{claimed_crash_state_count} 个状态，数组在 {test_file_relative_path} 里出现 {array_count} 次）')
+    if not checked_descriptions:
+        # 一句都认不出时不许当成「没有要核的」放过：那句登记被改了形状，这一格就再没人核（扫到 0 项不是通过）
+        mismatches.append('八节里一句「第二条流的段序列 `…`、N 次写、M 个状态 … 装置钉住 … `crates/….rs`」的登记都认不出：登记句的形状改了，第二条流就没人核')
     return mismatches, checked_descriptions
 
 
```
