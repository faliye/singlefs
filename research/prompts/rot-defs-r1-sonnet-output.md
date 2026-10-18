# rot-defs-r1 正推腿（Sonnet）报告：K1 每处改动与它引的现状一致吗

腿：three-way-forward（Sonnet），格 K1，立场「逐处核改动与现状一致」。基线：HEAD c1e9b324 加工作区与暂存区此刻的样子（被引的 `merge-and-verify.sh`、`run-scope-guard.sh`、`lib_run_scope.py`、`settings.json` 里的两条注册、`broad-mutation.py`、`mutate.sh` 等在暂存区，HEAD 里没有或是旧版：这些定义改动落在一批没提交的脚本改动之上，两者要同一批提交，单提交定义会指到 HEAD 里不存在的钩子）。
方法：先读被判的八份定义里的改动行（背景材料第五节的 diff 与仓里的现文逐处对过，行数 28 增 26 删与 `git diff HEAD --stat` 一致），再对每处读它引的脚本、钩子、规则、kb，能跑的钩子用 hook 输入喂一遍。引文的行号都是在被引文件里现查的。没建副本、没起编译。

## 一、结论总览（逐处，详见二到四节）

- 与现状一致：十六处（第 1-8、10-15、18、19 项：共用约束六处、实验执行员两处、实现员一处、写回员一处、分诊两处、攻方一处的路径、工具实现员一处、主 agent 的验证段两处）。
- 有冲突或对不上：四处（第 9、16、17、20 项）。主 agent 禁止段的「按崩溃判定库现算」比 kb 已定项 9 少了一半条件；派发表广谱变异行的「不换树」与驱动文件头、commit-stages 对不上字面；职责段与第 10 条的「`--all-packages` 才跑全部」漏了脚本里另四种照旧全跑的情形；实验执行员第 7 步的「指针指到的那一行要以 `E7RESULT name=<名>` 起头」只有一份未入库、检查没接的登记表撑着。
- 改动之外顺带核出的（给攻方与主 agent，不算 K1 的冲突）：三个，第 14、21、22 项。攻方定义 3b 的原型调 `enumerate_crash_states` 在 heavy-test-guard 下跑不了（实测被拒）；分诊定义第 2 步「这张表不跑」与 `mutate.sh` 的按条锚点失效对不上；分诊第 5 步缺 mutation-sampling 的「本次未跑」一档。

## 二、共用约束 `.claude/agent-common.md`（六处）

1. 仓副本拷贝清单加 `.claude/heavy-thread-default`：一致。
   「副本只拷要编的（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，要门禁表的再加 `.claude/gate.d`）」（`.claude/agent-common.md:46`）。
   现状：「pub const REGISTRATION_FILE_TEXT: &str = include_str!(concat!(」（`crates/singlefs-harness/src/heavy_threads.rs:22`）读「"../../.claude/heavy-thread-default"」（`crates/singlefs-harness/src/heavy_threads.rs:25`）；`crates/singlefs-unit-graph/src/heavy_threads.rs` 同一个字面量；`compile-then-swap.py` 文件头做法第 1 步也拷它：「与每台默认重活线程数的登记文件 .claude/heavy-thread-default」（`scripts/hosts/compile-then-swap.py:15`）。
   别的 crate 里读仓根外文件的 `include_str!`：`grep -rn 'include_str!' crates --include=*.rs -A4 | grep -E '\.claude|\.\./\.\./'` 只命中上面两个 heavy_threads.rs 和两处注释、测试里的字符串；`crates/*/build.rs` 只有 `singlefs-checker-tier/build.rs`，里面 `grep -n '\.claude\|scripts/\|\.\./\.\./'` 零命中。清单没有漏拷的第三个文件。
   推翻它的现象：某个 crate 的 `include_str!`/`build.rs` 读了 `.claude/` 下别的文件，或副本只拷这五样时编过。

2. 轻型三条的「开头都带范围声明」：一致（实测）。
   「三条开头都带范围声明 `SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=…; 范围=…; 为什么=…'`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头，没带的被它拒」（`.claude/agent-common.md:56`）。
   我把 hook 输入（子 agent 类型 implementation-writer）喂给 `bash .claude/hooks/run-scope-guard.sh`：崩溃点重放 `--quick` 不带声明退出码 2、带 `种类=quick` 声明退出码 0；形式化 `--quick <根>` 不带声明退出码 2。拒绝说明首行原样：`✗ 范围声明不合，这条命令没跑（run-scope-guard.sh）：`，其下 `✗ 没带范围声明`。
   命令在 `bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/crash/crash-point-replay.sh --quick` 前面加赋值，外面写的声明带进里面那一条：hook 文件头写了「包在外面的照剥、剥出来的照同一张表判，外面写的声明带进里面那一条」，实测放行。
   推翻它的现象：带了声明的轻型命令被该 hook 拒，或不带的被放行。

3. `E168_THREADS` 取代 `KS_THREADS`：一致，附一条措辞注。
   「实验装置自己的线程变量（`E168_THREADS` 这类）同样设成 N」（`.claude/agent-common.md:62`）。
   `grep -rnF 'KS_THREADS' scripts crates .claude` 零命中，`E168_THREADS` 在 `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:305` 是装置读的变量名。
   注：`capped.sh` 已经把它设上：「E165_THREADS（E165 装置）、E168_THREADS（E168 装置），以及通用的 SINGLEFS_THREAD_CAP」（`scripts/hosts/capped.sh:21`），而同一行括注只说「它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N」。照 `capped.sh N` 起的子 agent 不用再单设它；这句「同样设成 N」对它是多余但无害。例子换成 `capped.sh` 变量清单里没有的才算「这类」，这是措辞问题，不会让子 agent 走错。
   推翻它的现象：`capped.sh` 变量清单（`scripts/hosts/capped.sh:38` 起的 `variable_names=(` 清单，`E168_THREADS` 在 `scripts/hosts/capped.sh:50`）里去掉 `E168_THREADS` 而装置仍只读它。

4. 派发闸加「提示正文里有许可子 agent 跑 checker 档 / 重型测试的句子」：一致。
   「或提示正文里有许可子 agent 跑 checker 档 / 重型测试的句子（轻型三条带 `--quick` 的写法除外）」（`.claude/agent-common.md:86`）。
   现状：`.claude/hooks/runner-dispatch-guard.sh` 文件头 ① 写「有那一行也照判提示正文里许可子 agent 跑 checker 档 / 重型测试的句子（heavy_permission_sentences），有一句就拒、引出那几句：」（`.claude/hooks/runner-dispatch-guard.sh:9`），并写「轻型那三条带 --quick（可再带 --item）的写法许可子 agent 跑照放」（`.claude/hooks/runner-dispatch-guard.sh:15`）。挂在 Agent / Task（同一文件第 2 行 `hook-events` 与 `.claude/settings.json` 里 matcher「Agent|Task」）。
   推翻它的现象：派发提示里写「可以跑 checker 档」而该 hook 放行。

5. 新加「范围声明闸」条目：一致，但比 hook 实际管的窄（不会让人走错，会让人漏看）。
   「范围声明闸（`.claude/hooks/run-scope-guard.sh`，挂 Bash，主 agent 与子 agent 都判）」「子 agent 起 whole 一律拒（写法在它文件头）」（`.claude/agent-common.md:87`）。
   注册：`.claude/settings.json:44` 是 `bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/run-scope-guard.sh`，在 PreToolUse 的 Bash matcher 下（用 `python3` 读 settings 列出：`PreToolUse Bash bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/run-scope-guard.sh`）。
   子 agent 的 whole 拒：hook 文件头「子 agent（hook 输入带 agent_type）的 whole 一律拒，说明写「交回主 agent」」（`.claude/hooks/run-scope-guard.sh:44`）；实测子 agent 用 `merge-and-verify.sh --skip-apply` 加 `种类=whole` 声明被拒，说明里写 `✗ 子 agent 不跑整轮：这一条是合入后验证（…）。交回主 agent`；主 agent（hook 输入不带 agent_type）带 `种类=whole; … 范围=整轮` 退出码 0，带 `种类=items` 退出码 2，说明：`✗ 声明的是点名的几项（items），命令跑的是整轮（whole）：合入后验证（…）`。
   窄在哪：条目写「起验证的命令（轻型三条、门禁阶段的一道或几格、`cargo test` 点名的包与目标、合入后验证、实验复跑）」，hook 另管崩溃放量三份入口、整轮门禁 `gate-staged.sh`/`gate.sh`（whole）、`cargo test` 整包与整工作区（whole），以及把命令位置上执行的驱动脚本读进去逐行判（「驱动脚本    命令位置上执行的别的 shell 脚本……里面每一条要声明的命令照同一张表判」，`.claude/hooks/run-scope-guard.sh` 文件头）。子 agent 把验证链写进脚本再跑时，脚本里每条要声明的命令各自要带声明；条目没写这一点。
   推翻它的现象：settings.json 里该 hook 不在 PreToolUse/Bash 下，或子 agent 带对的 whole 声明被放行。

6. 新加「定时闸」条目：一致。
   「定时闸（`.claude/hooks/timer-only-remind-guard.sh`，挂 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor、Bash，主 agent 与子 agent 一样）」（`.claude/agent-common.md:88`）。
   现状：「主 agent 与子 agent 一样拒。」（`.claude/hooks/timer-only-remind-guard.sh:5`）；settings 里 `"matcher": "CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash"`（`.claude/settings.json:67`），命令 `.../timer-only-remind-guard.sh`（`.claude/settings.json:71`）；`hook-events` 一行六个事件与条目一致。条目后半「除 `bash scripts/agents/remind.sh` 之外的定时办法」与规则 `.claude/rules/timer-only-remind.md:5`「本工程设定时（到点提醒、周期检查、过一阵再回来看一眼）只用 `bash scripts/agents/remind.sh`」一致（文件 `scripts/agents/remind.sh` 在）。
   推翻它的现象：该 hook 的 matcher 少了条目里点名的某个工具。

## 三、实验执行员 `.claude/agents/experiment-runner.md`（三处）

7. 第 3 步收尾加「全部复用」：一致。
   「收尾要有「已还原，基线仍全绿」，或表里每条都按判定库复用时的「全部复用」」（`.claude/agents/experiment-runner.md:36`）。
   现状：「全部复用时不起 cargo：基线与还原之后那一次都不跑、副本里的源码一个字没改，收尾打一行「全部复用」代替「已还原，基线仍全绿」；」（`scripts/mutation/mutate.sh:22`），打印处 `echo "全部复用：表里 $reused_scope，没起 cargo（…）"` 在 `scripts/mutation/mutate.sh:1961`，平常收尾 `echo "已还原，基线仍全绿"`（`scripts/mutation/mutate.sh:2342`）。
   细节：全部复用路径的触发是 `((LIBRARY_REUSED + ANCHOR_BROKEN_COUNT == mutation_count && mutation_count > 0))`（`scripts/mutation/mutate.sh:1848`），含有锚点失效的条时也打「全部复用：表里 … 条按判定库复用、锚点失效 … 条没跑」；此时计数行里「锚点失效」不为 0，第 5 步的判红会接住。不构成走错。
   推翻它的现象：`mutate.sh` 在全部按库复用时仍打「已还原，基线仍全绿」，或在有现判的条时打「全部复用」。

8. 产出「计数：」六栏：一致。
   现状：`echo "计数：内存撞顶 $memory_hit_count 条（上限 $MEMORY_MAX）、超时 $timeout_count 条、排不上 $queue_full_count 条、被总上限挤掉 $squeezed_count 条、显存不够 $gpu_memory_count 条、锚点失效 $anchor_count 条$LIBRARY_COUNT_TEXT"`（`scripts/mutation/mutate.sh:2319`），六栏栏名与定义里写的逐个相同；后面还接「、现判 N 条、复用 M 条、拒算 R 条」（`LIBRARY_COUNT_TEXT`，`scripts/mutation/mutate.sh:1844`）三栏，定义说「六栏」指的是前六栏，与 `.claude/rules/mutation-sampling.md:79` 的「`mutate.sh` 收尾那行「计数：」报内存撞顶、超时、排不上、被总上限挤掉、显存不够、锚点失效六栏」同一口径。
   「不为 0 的整轮已判失败」：锚点失效不为 0 退 3：「3 有锚点失效的条（别的条照跑完；同时有 1 那几种的也退 3，逐条的行照打）；」（`scripts/mutation/mutate.sh:115`）。
   推翻它的现象：收尾行栏名或次序变了，或锚点失效不为 0 时 `mutate.sh` 退 0。

9. 第 7 步实验页指针：形式一致，「要以 `E7RESULT name=<名>` 起头」这一半的出处不在已入库的规则里。
   「指针指到的那一行要以 `E7RESULT name=<名>` 起头」（`.claude/agents/experiment-runner.md:43`）。
   形式：实验页里已有同形指针，例 `.claude/kb/experiments/172-删除写出的字节的干跑.md:237` 的「产物行见 git show 18b30adbfe94:research/results/e172-deletion-write-dry-run-2026-10-05.out 第 258 行（name=verdict）」；我 `git show 18b30adbfe94:research/results/e172-deletion-write-dry-run-2026-10-05.out | awk 'NR==258'` 输出 `E7RESULT name=verdict row=1 not_equal_cells=0 discriminable_false_rows=2 cells_without_hits=0 cells_without_hits_list=no`，与指针「name=verdict」对得上、以 `E7RESULT name=` 起头。
   出处：这条要求只写在 `.claude/product-line-prefixes`：「E7RESULT  # 实验装置（scripts/experiments/ 下的 e<号>_* 二进制）打的结果行，name=<名> 是那一行的量名」（`.claude/product-line-prefixes:3`），同一文件说「一行一个前缀，# 后写它是哪一类产物的行；判它的检查还没接（C837 欠的那一半）。」（`.claude/product-line-prefixes:2`）。`git ls-files --error-unmatch .claude/product-line-prefixes` 报 `Did you forget to 'git add'?`，即这份登记表没入库；`grep -n '产物行见' .claude/gate.d/doc-experiments.sh` 零命中，没有任何门禁格判指针。`quoted-result-lines` 格只判「以 `E7RESULT ` 开头的行」（`.claude/gate.d/doc-experiments.sh:1591` 起的判据），指针行不以它开头，不在其射程。
   判定：一致（形式）/ 规则没说（把它写成要求的出处没入库、没检查）。定义把「要以…起头」写成了硬要求，写这句的人保证；与登记表一起入库就闭合。
   推翻它的现象：入库后登记表前缀不是 `E7RESULT`，或有已存在的指针指到的行不以它起头（我只核了页 172 的一处，没有全仓扫指针）。

## 四、实现员、写回员、分诊员、攻方、工具实现员

10. 实现员 7a 加「崩溃点重放合表要点名它」：一致，另有一处信息缺口。
   改后句尾「由主 agent 或 tooling-writer 登记，连同崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 要点名它的那一项（没被点名的入口 `scripts/crash/crash-point-replay.sh` 不跑，`crash-case-check.py` 的 placement 判红）」在 `.claude/agents/implementation-writer.md:48`（整行很长，这里只抄到这一句）。
   现状：规则 `.claude/rules/verification.md:75` 写「没被点名的，入口 `scripts/crash/crash-point-replay.sh` 不跑，`scripts/crash/crash-case-check.py` 的 placement 判红」；`scripts/crash/crash-case-check.py` 的判项表第 6 行「placement     崩溃枚举用例住 checker 档包、标了 #[ignore] 的都登记进崩溃点重放合表（checker-tier-crash-point-replay 开跑前判）」（`scripts/crash/crash-case-check.py:6`）。合表在 `.claude/gate.d/crash-point-replay-items.tsv`（`git ls-files --error-unmatch` 通过，已入库；表里第二列确为 stage-inputs 的 `crash-case:` 键：该表头「② 重型里崩溃点重放那一套的部分：stage-inputs.tsv 里 crash-case: 的键」（`.claude/gate.d/crash-point-replay-items.tsv:5`））。
   实现员的写范围表里没有 `.claude/gate.d/`：`.claude/hooks/agent-write-scope.tsv:8` 是「implementation-writer	crates/**	implementation-writer.md「写范围」」，所以「由主 agent 或 tooling-writer 登记」对。
   缺口（K2 的料）：报告只被要求给「stage-inputs.tsv 的 crash-case: 行原文」，合表那一行（项名、第二列键、第三列放量分项、第四列轻型测试目标）没有要求实现员在报告里给；合表第四列「轻型取自重型的哪一项」必须非空且都在第二、三列点名的测试目标里（`.claude/gate.d/crash-point-replay-items.tsv:7-8`），登记的人手里没有实现员对「关键步骤拿进轻型」的意见时，只能自己猜。这一句不算错，漏了一个应交的东西。
   推翻它的现象：`crash-case-check.py` 的 placement 对「登记了 crash-case: 但没被合表点名」不红（它自带 placement 红样本：「登记了 crash-case: 却没被合表点名，placement 应当报出来」（`scripts/crash/crash-case-check.py:465`））。

11. 写回员：一致。
   「`.claude/kb/decisions.md` 的分项清单生成块与索引表状态、进度两列由第 3 步的 `doc-decisions.sh --write` 重生成」（`.claude/agents/kb-scribe.md:22`）。
   现状：「#   bash .claude/gate.d/doc-decisions.sh --write [--check decision-items-sync] [项目根]  重写 decisions.md 的分项清单生成块与索引表「状态」「进度」两列，写完回读判一遍」（`.claude/gate.d/doc-decisions.sh:39`）；格 `decision-items-sync` 判「decisions.md 的分项清单生成块与索引表「状态」「进度」两列，与生成器的输出逐字相同」（同一文件第 23 行 gate-cell）；kb-scribe 第 3 步末尾确是 `bash .claude/gate.d/doc-decisions.sh --write`（`.claude/agents/kb-scribe.md:33`）。
   推翻它的现象：`--write` 之后 `decision-items-sync` 仍红，或它没改「进度」列。

12. 分诊员第 4 步与第 5 步：一致（实测代码）。
   第 4 步「表里每条都按判定库复用时是「全部复用：…没起 cargo」那一行」（`.claude/agents/mutation-triage.md:33`），现状见第 7 项。
   第 5 步「另外四栏「被总上限挤掉」「排不上没跑」「scope 起不来没跑」「锚点失效」不为 0 也让整道判红」（`.claude/agents/mutation-triage.md:34`）。
   现状：门禁 checker-tier-crates-mutation-replay 的计数行 `f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条、锚点失效 {len(anchor_broken)} 行"`（`scripts/mutation/crates-mutation-rows.py:2444`），判红条件含 `anchor_broken`：「or not_run_record_problem or anchor_broken)」（`scripts/mutation/crates-mutation-rows.py:2518`）；四栏栏名与定义一一相同（「锚点失效」那一栏的单位是「行」，定义只写栏名，没问题）。`mutate.sh` 的 `⚓` 记号：`锚点失效（.claude/rules/mutation-sampling.md 第七类）`段在 `scripts/mutation/mutate.sh:46`，「不起 cargo、不进按条判定库、不复用；别的条照跑照判、照入库。收尾「计数：」里一栏「锚点失效 A 条」（接在现判、复用、拒算那三栏前面），A 不为 0 整轮退 3（判完别的条之后）。」（`scripts/mutation/mutate.sh:47`）；规则也同口径：「- `mutate.sh` 在派活之前逐条核锚点，锚点失效的那一条打一行 `⚓`、不跑、不入判定库，别的条照跑；收尾「计数：」报「锚点失效 N 条」，N 不为 0 整轮退出码 3。」（`.claude/rules/mutation-sampling.md:57`）。
   「六栏」栏名核过（第 8 项）。`💥` 与 `⚠️` 两栏两个脚本的写法与改前相同，没动。
   推翻它的现象：门禁计数行改了栏名，或锚点失效不为 0 时整道不红。

13. 分诊员产出行加「锚点失效」：一致，同第 8 项。

14. 攻方 3b 的枚举函数路径：路径一致；这条句子在 heavy-test-guard 下照做不了（改动之外的发现，已实测）。
   改后「调 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 里每一段都展开的 `enumerate_crash_states` / `enumerate_crash_states_versions`」（`.claude/agents/three-way-attack.md:35`）。
   现状：「pub fn enumerate_crash_states(」在 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs:242`，「pub fn enumerate_crash_states_versions(」在 `crates/singlefs-checker-tier/src/crash_state_enumeration.rs:283`；`grep -nE 'fn (enumerate_crash_states|enumerate_crash_states_versions)\b' crates/singlefs-checker-tier/src/*.rs` 只命中这一个文件两行，`crash.rs` 里只有文档注释提到它们（`crates/singlefs-checker-tier/src/crash.rs:638`、`crates/singlefs-checker-tier/src/crash.rs:2371` 的 `///` 行），没有 `fn`。「pub fn crash_state_count_with_torn_in_place_overwrites(」在 `crates/singlefs-checker-tier/src/crash.rs:2374`，定义里这一句的「crash.rs」没错。`closed_form_state_count` 在 `crates/singlefs-harness/src/memory_pool.rs:625`（`crash.rs` 只是 `use` 它，同句「写表里没有原地覆写时它等于 `closed_form_state_count`」不指路径，不受影响）。
   不能照做的原因（实测）：这两个函数在 `singlefs-checker-tier` 这个包里，文件开头 `use crate::crash::{`（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:40`）一路 `crate::` 引同包模块，原型要调只能把该包写进依赖表。heavy-test-guard 文件头：「链接它的包（探针包、原型包：Cargo.toml 的依赖表里写它、或经 path 依赖间接依赖它）：cargo test、cargo run、cargo bench 的范围」（`.claude/hooks/heavy-test-guard.sh:29`）按 checker 档拒。我在草稿目录造了依赖表写着 `singlefs-checker-tier = { path = … }` 的探针包，以 agent_type=three-way-attack 把 `bash scripts/hosts/run-with-memory-cap.sh 8G cargo run --manifest-path <探针>/Cargo.toml` 喂给 `bash .claude/hooks/heavy-test-guard.sh`，退出码 2，首行原样：`✗ 重型测试被拒：cargo run 起的包 /home/fy5090/.local/share/singlefs/work/rot-defs-r1-sonnet/probe 依赖 singlefs-checker-tier（探针包链接 checker 档，里面能起崩溃枚举，默认只在提交时跑；按包判，不看它挑哪个目标）（checker 档）：three-way-attack 不跑「checker 档」`。
   后一句「在自己的原型里这样调枚举函数跑小流不算重型」（`.claude/agents/three-way-attack.md:36`）因此与该 hook 冲突：原型包依赖 checker 档包就是重型，子 agent 一律拒；`grep -rlE 'fn +enumerate_crash_states' crates scripts` 另外只有 `scripts/experiments/e150_rollback_reuse_of_abandoned_roots.rs` 里一个自己写的同名函数，不是入库实现。这个冲突改前就在（旧句「调同一份文件里…」同样指向该包），这一轮的改动让路径变准、没让它变对；攻方腿照做第一步就被拒，是 K2 要攻的点。
   推翻它的现象：把依赖写成 `[dev-dependencies]` 或别的写法后 hook 放行了该探针；或有不依赖 checker 档包的入库路径能调到这两个函数。

15. 工具实现员「没做什么」加广谱变异：一致。
   「没跑重型测试（…checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法…）」（`.claude/agents/tooling-writer.md:51`）。
   现状：heavy-test-guard 把广谱变异列为重型：「提交门禁那一道 .claude/gate.d/checker-tier-broad-mutation.sh 同算这一类」「子 agent 一律不跑；主 agent 带前缀能跑」（`.claude/hooks/heavy-test-guard.sh:53`）；门禁文件在 `.claude/gate.d/checker-tier-broad-mutation.sh`。共用约束第 56 行同口径（广谱变异「起 cargo 或改库目录的每一种用法」）。
   推翻它的现象：tooling-writer 的放行表里允许它跑 `broad-mutation.py` 的某种起 cargo 的用法。

## 五、主 agent `.claude/main-agent.md`（六处）

16. 职责段与第 10 条与派发表「合入之前」行：合入后验证改成「只跑改到的包」。主体一致，但「`--all-packages` 才跑全部」说窄了。
   改后句（职责段）：「打全部补丁、编一次、跑补丁改到的包与依赖它们的包的测试——都除 checker 档包 `singlefs-checker-tier`，带 `--all-packages` 才跑全部——与 fmt、clippy」在 `.claude/main-agent.md:7`（整行很长，抄到这一句）；第 10 条：「跑补丁改到的包与依赖它们的包（除 checker 档包；`--all-packages` 跑全部）的测试」（`.claude/main-agent.md:29`）；派发表：「跑补丁改到的包与依赖它们的包（工作区除 checker 档包 `singlefs-checker-tier`，成员从树根 `Cargo.toml` 的 `[workspace] members` 现读；`--all-packages` 跑全部）的测试」（`.claude/main-agent.md:90`）。
   现状（暂存区里的新脚本，HEAD 没有：`git show HEAD:scripts/gate/merge-and-verify.sh | grep -c all-packages` 输出 `0`，工作区 `grep -c all-packages scripts/gate/merge-and-verify.sh` 输出 `13`，与背景材料第二节同数）：「--all-packages 测试步照旧跑工作区成员减掉 checker 档包的全部包，不按补丁挑。」（`scripts/gate/merge-and-verify.sh:8`）。包的挑法对：只留补丁改到的包与依赖它们的包，且减掉 `singlefs-checker-tier`（「减掉 checker 档包 singlefs-checker-tier（那归提交时的主 agent）」，`scripts/gate/merge-and-verify.sh:26`；只留改到的包与依赖它们的包的一段从 `scripts/gate/merge-and-verify.sh:29` 起），fmt、clippy、build 另跑（「fmt    `cargo fmt --all -- --check`」，`scripts/gate/merge-and-verify.sh:36`）。
   对不上的两句并排：
   - 定义：「跑补丁改到的包与依赖它们的包的测试——都除 checker 档包 `singlefs-checker-tier`，带 `--all-packages` 才跑全部」（`.claude/main-agent.md:7` 的这一段）
   - 脚本：「或没有 crates.patch）时测试这一步不跑、报「测试本次未跑」，不记红也不算测过。照旧全跑的：带 --all-packages；没给补丁目录；」（`scripts/gate/merge-and-verify.sh:33`）「补丁改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；补丁改到不落在任何包里的路径（测试运行时可能读它，判不出归哪个包）。」（`scripts/gate/merge-and-verify.sh:34`）
   差在哪：脚本里除 `--all-packages` 之外，没给补丁目录、补丁改到工作区根 `Cargo.toml` / `Cargo.lock` / `.cargo/`、补丁改到不落在任何包里的路径，也都「照旧全跑」；反过来补丁只改了 checker 档包时测试一个包都不跑、报测试本次未跑（上面第 33 行那一句，和 `scripts/gate/merge-and-verify.sh:32` 的「一个包都没挑上（补丁只改了 checker 档包、」）。照定义做的主 agent 在这几种情形里会对「跑了哪些包」判错：多跑不碍事，「测试本次未跑」那一种（补丁只碰 checker 档）定义没提，主 agent 要靠脚本输出认出来，不能当「测过」。程度：轻（不会让人跑错命令）。
   hook 侧同口径：`.claude/hooks/lib_run_scope.py:595` 写「merge-and-verify：挑包在脚本里做（只跑补丁改到的包与依赖它们的包，--all-packages 才全跑），种类照旧写 whole」，所以这一说法有同侧的先例；同样说窄。
   推翻它的现象：脚本把「没给补丁目录」等三种情形改成只跑改到的包、或定义里补上这几种。

17. 禁止段「按崩溃判定库现算」：冲突，主 agent 的话比 kb 已定项 9 少了一半条件。
   定义：「里程碑收尾、动了崩溃点重放或判器的那一批（按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上，D13（验证路线） 已定项 9，不按路径清单判）提交之前，先弹窗请用户跑一趟全量」（`.claude/main-agent.md:10`，整行抄到这句）。
   kb：「「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。」（`.claude/kb/decisions/13-验证路线.md:256`，整行）
   差在哪：kb 要两个条件同时成立——单元键对不上，且对不上的键里有判器单元变了；定义只留第一个。只改了崩溃点重放无关单元（库里有判定块的任何单元键对不上，例如一个共用工具函数）的批，按定义要弹窗，按 kb 不用。照定义做的主 agent 会多弹窗；没有漏弹的方向。
   实现侧的第二个问题（规则没说）：kb 说「阶段把作废块数与变了的判器单元清单打出来」，我在 `scripts`、`crates`、`.claude/gate.d`、`.claude/skills`、`.claude/hooks` 里 `grep -rlE '作废块数|变了的判器|判器单元|作废的块'` 零命中，没找到打这份清单的阶段或脚本；`.claude/skills/crash-test/commit-stages.md` 里关于这件事只有「动了崩溃点重放或判器的那一批，D13（验证路线） 已定项 9」一句。「现算」今天没有现成工具，主 agent 照定义只能自己对库。这条属于 kb 与实现对不上，复核不了它是未实现还是在我没搜到的位置。
   推翻它的现象：找到打印「作废块数与变了的判器单元」的阶段；或 kb 已定项 9 的判据改成只看单元键对不上。

18. 验证段第 1 步加范围声明：一致。
   「起验证的命令（这一条、轻型三条、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头」（`.claude/main-agent.md:50`）。
   现状：hook 文件头「整轮门禁   scripts/gate/gate-staged.sh、.claude/singlefs-ai-sop/scripts/gate.sh 与它的包装 .claude/scripts/gate.sh：whole。」（`.claude/hooks/run-scope-guard.sh:33`）、「整轮门禁（gate-staged.sh、gate.sh）的为什么里写了「提交前整轮」就放，不数字数。」（`.claude/hooks/run-scope-guard.sh:22`）；`merge-and-verify.sh` 在 hook 里种类是 whole：实测主 agent 输入带 `种类=whole; … 范围=整轮` 退出码 0，带 `种类=items` 退出码 2。
   提醒：该 hook 文件头仍写「合入后验证  scripts/gate/merge-and-verify.sh：whole（还不认只跑改到的包，等 4-3i 第 8 条）。」（`.claude/hooks/run-scope-guard.sh:32`），与新脚本对不上（背景材料第二节已说归 tooling-writer 下一批）；它不改变主 agent 要写的种类（都是 whole），只让「写法照文件头」读到一句过时的话。
   推翻它的现象：`merge-and-verify.sh` 在 hook 里被改认 items，种类 whole 开始被拒。

19. 验证段第 3 步分产品代码与测试代码：一致。
   「整段的合入里有产品代码改动的，代码轮三方开一轮，快照取在全部合入之后，攻整段的合入；整段只改测试代码或只改注释的不开代码轮」（`.claude/main-agent.md:52`）。
   规则：「只改了注释的产品代码文件也不走（去掉注释与空白之后代码与改前逐个相同」（`.claude/rules/implementation-workflow.md:10`）。口径一致；规则里「文档注释里会编成 doctest 的代码块、文档注释与 `// SAFETY:` 注释的增删照代码算」这一限定，定义靠后半句「射程在 …」指过去，没抄；定义那一句「只改注释」读得比规则宽，指向规则就够。
   推翻它的现象：规则第 2 步射程改成只改注释的也走代码轮。

20. 派发表「暂存之后、提交之前跑门禁」行的广谱变异：对不上字面（「不换树」）。
   定义：「先起广谱变异驱动（不带选条参数就是剪枝：每条变异按单元键沿用已判过的、沿用不了的现判，不换树、不按提交选条；照 `.claude/skills/crash-test/commit-stages.md` 第 2b 步）」（`.claude/main-agent.md:94`，抄这一句）。
   commit-stages 第 2b 步：「判定按每条变异的单元键存在一份固定库目录（`~/.local/share/singlefs/broad-mutation-units/`）里、按单元键逐条沿用，不按提交、不按 HEAD 选条；没有「一版」，不包暂存树、不换版本：」（`.claude/skills/crash-test/commit-stages.md:23`），同一步稍后「驱动先把工作区逐文件换进库目录的专用树、生成清单并按单元记进库」（`.claude/skills/crash-test/commit-stages.md:26`）；驱动：「broad-mutation.py --refresh-list                 只换树、生成清单、把清单按单元记进库（不编不跑），之后 --list-items / --summary 在同一份源码上列得出全部条」（`scripts/mutation/broad-mutation.py:17`）、`--sync-tree` 已删，原因「「一版」不再是容器：每一趟开跑自己把 --source 那棵树换进专用树」（`scripts/mutation/broad-mutation.py:515`）。
   差在哪：commit-stages 说「不包暂存树、不换版本」，驱动每一趟开跑都会把 `--source` 那棵树换进专用树；定义写「不换树」，字面上与「每一趟开跑自己把 --source 那棵树换进专用树」反着。意思是想说「不再换成暂存树 / 不换版本」的话，那是对的，但「不换树」会让读的人以为驱动不碰树；要改成「不包暂存树、不换版本」才与出处同字。「不按提交选条」与 `REMOVED_ARGUMENTS` 的「不按提交选条：每条变异按单元键过沿用判据，沿用不了的才现判」（`scripts/mutation/broad-mutation.py:517`）同口径，这一半一致。「每条变异按单元键沿用已判过的、沿用不了的现判」与驱动文件头「沿用得了沿用、沿用不了现判」一致。
   推翻它的现象：定义的「树」特指暂存树，且读它的主 agent 都这样读（我没法核读者怎么读，只核了字面）。

## 六、改动之外顺带核出的（给攻方腿与主 agent，不算 K1 的冲突）

21. 分诊第 2 步的「这张表不跑」与按条锚点失效对不上（这一轮把第 4、5 步改成认 `⚓` 之后，同一份定义里两处自相矛盾）。
   定义第 2 步：「2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。」（`.claude/agents/mutation-triage.md:31`）。
   规则：「- `mutate.sh` 在派活之前逐条核锚点，锚点失效的那一条打一行 `⚓`、不跑、不入判定库，别的条照跑；收尾「计数：」报「锚点失效 N 条」，N 不为 0 整轮退出码 3。」（`.claude/rules/mutation-sampling.md:57`）；驱动：「不起 cargo、不进按条判定库、不复用；别的条照跑照判、照入库。收尾「计数：」里一栏「锚点失效 A 条」（接在现判、复用、拒算那三栏前面），A 不为 0 整轮退 3（判完别的条之后）。」（`scripts/mutation/mutate.sh:47`）。
   差在哪：第 2 步让分诊员发现一条锚点不唯一就整张表不跑；驱动和规则是只那一条不跑、别的条照跑。改后第 5 步的 `⚓` 栏只有表照跑才会非 0。两者不会同时成立；分诊员按第 2 步做，第 5 步这一栏永远走不到。方向保守（少跑），不会漏判，但分诊员拿不到其余条的结果。

22. 分诊第 5 步缺「本次未跑」一档，与规则对不上。
   定义：「那一行出现了（跑了没红，或跑到它、跑完它之后进程被杀）记没红」（`.claude/agents/mutation-triage.md:34`）。
   规则：「那一行是 `ok`、输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它，记「本次未跑」（不算抓到、不算没红，另起一行报）」（`.claude/rules/mutation-sampling.md:79`）。门禁 checker-tier-crates-mutation-replay 确有这一档：「say(f"  本次未跑：{len(not_run)} 条（」（`scripts/mutation/crates-mutation-rows.py:2462`），上一行注释写「# 本次未跑的逐条列出来（文件头「本次未跑」）：不算抓到、不算没红；「计数：」那一行的格式不动，另起一行报」（`scripts/mutation/crates-mutation-rows.py:2459`）。
   差在哪：没有 GPU 的机器上点名的测试打 `NOT_RUN_NO_GPU_ADAPTER` 返回成功，分诊员按定义会记成「没红」，规则要的是单列「本次未跑」。规则 79 行不在这一轮改的八份里，是定义落后于规则；放在这里给 K3 一个现成的料。

## 七、背景材料第二节十三条观测，我自己再核的结果

- 第一条（`merge-and-verify.sh` 的挑包）：复现 `0` 与 `13`，在暂存区；挑包逻辑读过，见第 16 项。
- 第二条（`run-scope-guard.sh` 文件头与注册）：文件头第 32 行原文在 `.claude/hooks/run-scope-guard.sh:32`；`.claude/settings.json:44` 注册；实测见第 5、18 项。
- 第三条（定时闸注册事件）：核过，见第 6 项。
- 第四条（`mutate.sh` 第 22、1961、2319 行与规则 79 行）：核过，三个行号都对，见第 7、8 项。
- 第五条（`heavy_threads.rs` 第 22 行）：核过，见第 1 项。
- 第六条（枚举函数的位置）：核过，见第 14 项。
- 第七条（`verification.md` 的登记写法）：核过，见第 10 项。
- 第八条（`doc-decisions.sh` 第 39 行）：核过，见第 11 项。
- 第九条（广谱变异是重型）：核过，见第 15 项。
- 第十条（commit-stages 第 2b 步）：核过；`--sync-tree`、`--commit`、`--changed-since` 已删，见第 20 项。
- 第十一条（implementation-workflow 第 2 步）：核过，见第 19 项。
- 第十二条（派发闸 ①）：核过，见第 4 项。
- 第十三条（D13 已定项 9）：kb 原文核过；与主 agent 的转述有出入，见第 17 项。

## 八、判定一览

| 格（K1，按项号） | 判定 | 一句话 |
|---|---|---|
| 1 共用约束：仓副本拷 `.claude/heavy-thread-default` | 一致 | `heavy_threads.rs` 的 `include_str!` 读它，别处没有第二个仓外读取 |
| 2 共用约束：轻型三条带范围声明 | 一致 | 实测不带被拒、带 `种类=quick` 放行 |
| 3 共用约束：`E168_THREADS` | 一致 | 名字在装置里是真变量，`KS_THREADS` 零命中；`capped.sh` 已设它，例子偏多余 |
| 4 共用约束：派发闸许可句 | 一致 | 与 `runner-dispatch-guard.sh` 文件头 ① 逐点相同 |
| 5 共用约束：范围声明闸条目 | 一致 | 注册与子 agent 拒 whole 实测；条目比 hook 实管的窄（漏了驱动脚本逐行判） |
| 6 共用约束：定时闸条目 | 一致 | 六个事件与 settings 的 matcher 逐个相同 |
| 7 实验执行员：第 3 步「全部复用」 | 一致 | `mutate.sh` 22、1961 行如此 |
| 8 实验执行员：产出六栏 | 一致 | 栏名与 2319 行逐个相同，不为 0 退 3 |
| 9 实验执行员：第 7 步指针 | 形式一致 / 规则没说 | 要求只写在未入库、检查没接的登记表里 |
| 10 实现员：7a 合表点名 | 一致 | 规则与 placement 判法都在；报告不要求给合表那一行（缺口） |
| 11 写回员：`--write` 重生成 | 一致 | `doc-decisions.sh` 39 行如此 |
| 12 分诊员：第 4、5 步 | 一致 | 四栏判红与 `⚓`、全部复用都在代码里 |
| 13 分诊员：产出六栏 | 一致 | 同第 8 项 |
| 14 攻方：枚举函数路径 | 路径一致 | 改动之外：原型链接该包就被 heavy-test-guard 拒（实测），句子做不了 |
| 15 工具实现员：广谱变异 | 一致 | heavy-test-guard 文件头列为重型、子 agent 一律不跑 |
| 16 主 agent：只跑改到的包 | 部分一致 | 「`--all-packages` 才跑全部」漏了脚本里另三种照旧全跑、一种本次未跑 |
| 17 主 agent：「按崩溃判定库现算」 | 冲突 | 比 kb 已定项 9 少「对不上的键里有判器单元变了」；打清单的阶段没找到 |
| 18 主 agent：验证段第 1 步声明 | 一致 | 种类 whole 实测；hook 文件头 32 行过时（已知） |
| 19 主 agent：验证段第 3 步 | 一致 | 与 implementation-workflow 第 2 步射程同口径 |
| 20 主 agent：广谱变异「不换树」 | 对不上字面 | 驱动每趟把 `--source` 换进专用树，出处写的是「不包暂存树、不换版本」 |
| 21（改动之外）分诊第 2 步 | 冲突 | 「这张表不跑」与按条锚点失效、与改后第 5 步的 `⚓` 栏矛盾 |
| 22（改动之外）分诊第 5 步 | 规则没说 | 缺 mutation-sampling 的「本次未跑」一档 |

每条结论的推翻条件写在各项末尾；总体：把对应脚本、钩子、规则、kb 的那一行改了，或实测换一种输入得到相反的退出码，该项即作废。

## 九、没做什么

- 没建副本，没起编译，没跑 `cargo`。heavy-test-guard、run-scope-guard 的实测是把 hook 输入（JSON）喂给钩子脚本本身，没起被拒的命令；探针包只是草稿目录里一份没编过的 `Cargo.toml` 加空 `main.rs`，用完已删。
- 没做 K2（子 agent 照做走一遍的完整演练）、K3（漏改扫描）：第 5、10、14、21、22 项是核 K1 时顺带撞到的，不是系统扫的结果，漏的肯定比这多。
- 没全仓扫实验页指针是否都指向 `E7RESULT` 开头的行，只核了页 172 的一处（第 9 项）。
- 没核「读的人怎么读」（第 17、20 项的措辞问题只核了字面）。
- 没判 `D13（验证路线）` 已定项 9 本身该不该改，也没替主 agent 决定哪一处该怎么改。
- 复核不了：主 agent 在第二节写的「归 tooling-writer 下一批」这类调度意图；kb 说「阶段把作废块数与变了的判器单元清单打出来」的那个阶段是没实现还是在别处（第 17 项）。
