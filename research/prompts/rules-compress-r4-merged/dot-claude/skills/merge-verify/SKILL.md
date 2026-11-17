---
name: merge-verify
description: 主 agent 每次开始「写与验分两段」验证段之前读它（正文不在此刻的上下文里就整份重读）——合入后验证六步的命令、各支结局的出路、在快照上验什么、怎么合进主工作区，与新出现的拒算怎么判、拒算补丁目录怎么做。
---

# 合入后验证的细则

`.claude/main-agent.md`「写与验分两段」验证段与「合入后验证之后判新出现的拒算」两节写着每一步做什么、绿了红了去哪；`.claude/skills/merge-verify/SKILL.md` 写每一步的命令与各支结局的出路。节名、步号、条号与那两节相同。

## 写与验分两段

**验证段**

**整批重交**：第 1 步把这一批的补丁目录（有修补的连同修补的补丁目录，没有的原样）整批重交，不带 `--item` 与 `--fingerprint-of`，键相同的照复用；已经做过第 2 步的，第 2 步那几条再在新留下的快照上重交，再回到原来那一步。第 1 步只重跑红的那一条退 2、说那一趟开跑以来主工作区变了的，第 2 步还没做过，只重交第 1 步。验证段各步与拒算各条说「整批重交」都指这一套。

1. 合入后验证这样交，不带 `--skip-apply`，补丁目录照写段打的次序全给（有段树的照打进段树的次序）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 <N> 份补丁; 范围=<N> 份补丁; 为什么=<这一批补丁改到哪些测试目标>' task-server/task submit -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh <补丁目录…>`
   脚本编译之后另跑 fmt、clippy、build，再跑验证包那一步（逐项交门禁 `crash-point-replay`、不起整格）；快照留在它日志目录的 `repo/`。挑哪些目标、验证包那一步交什么、快照与先验后合，照 `scripts/gate/merge-and-verify.sh` 文件头。没带 `env MERGE_AND_VERIFY_KEEP_COPY=1` 的那一趟不留快照、不打合进主工作区的命令，带上重交。
   快照：被下一趟替掉的那一趟的快照，下一趟交出去就删（路径与删法在它收尾那一行）；最后一趟的留到第 3 步合完。要留着的快照，在它日志目录（`repo/` 的上一层）的 `.deliverables` 写一行「repo 基线 <为什么留>」，删快照的同一步删这一行。退 3 而用户不同意、这一批放弃、任务被停，这三种情形同样删那一趟的快照（被停的那一趟带着 `env MERGE_AND_VERIFY_KEEP_COPY=1`，快照也留着）；每删一棵之前，这棵快照上还有主 agent 自己的任务在排在跑的，先 `task-server/task cancel <号>`，再 `du -sh` 记大小。
   结局分五支：
   - 绿（末行 `TASK_SERVER_RESULT` 是 `exit=0`）：它在汇总行之后打出的「验证段第 2 步」与「合进主工作区」两段，留给第 2、3 步。
   - 退 3（判不出改到哪个测试目标，测试那一步与验证包那一步没跑；几种原因在脚本文件头「挑测试目标」）：整包归提交时的整轮门禁，提交之外要整包跑照 `.claude/rules/verification.md`「一个门禁：两种跑法」「提交之外」那一条确认一次（带 `--whole-package-confirmed` 的由钩子弹，主 agent 不另问）。用户同意的，在这一趟的快照上整包跑脚本点名的那几个包（照 `.claude/agent-common.md`「日常开发的测试不跑整个包」确认三遍、带 `--whole-package-confirmed`，`--cwd` 写快照），绿了接第 2 步；用户不同意的不合，这一批停在验证段等用户定。
   - 退 1（红）：修补的实现员副本从这一趟的快照建。脚本在「→ 怎么办：」那一行打出只重跑红的那几项的整条命令（已带范围声明、`task-server/task submit --`、`env MERGE_AND_VERIFY_KEEP_COPY=1` 与 `--fingerprint-of <这一趟的日志目录>`），主 agent 照原样交，修补的补丁目录照第 5 步接在最后，别的不改写、不再插 submit；那一条退 2、说那一趟开跑以来主工作区变了的，照它的出路整批重交。缩过域的（给了 `--crash-expand-up-to <n>`，或环境里设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`），那一条自己带同一个 n，照原样交，汇总写「缩过域、不算门禁通过」。
   - 退 2（打补丁核不过，脚本整批停）：照它出路里段树那一句办，新段树照 `.claude/main-agent.md`「写与验分两段」写段第 2 条建。
   - 退 75（按条复用那一层连不上判定库服务 prune-server，测试那一步没跑）：不算绿、不进第 2 步，等服务回来照原命令重交。
   `--skip-apply` 只用于改动确已在主工作区上的（主 agent 自己写的探索性改动这类）：只编、只验，脚本先逐份核补丁已打在树上，核不过退 1；绿了不打合进主工作区的命令。起验证的命令（这一条、带 `--item` 的门禁与崩溃放量入口、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头；机器、内存上限与线程都不写（`.claude/agent-common.md`「子 agent 不交任务」）。
2. `--cwd` 与命令里的 `<根>` 都写第 1 步最后那一趟留下的快照（它是那一趟开跑那一刻的主工作区加这一批全部补丁，登记行跟着补丁在里面，带门禁表与仓里的脚本）。要交、都要绿的是：
   - 第 1 步在「验证段第 2 步」那一段打出的 `crash-case-check.py`（placement、modules、provenance、must-red）与 `refused-test-registry.py list` 两条（拒算照「合入后验证之后判新出现的拒算」判）；
   - 登记的崩溃枚举用例（门禁 `crash-point-replay` 按块那一路）、崩溃放量分项、formal-verification 带 `--item` 的那几条，点这一段全部补丁改到的项，命令照 `.claude/agents/implementation-writer.md` 第 1d 步；
   - crates-mutation-replay 带 `--item`，点那一段列出的这一批新写、改写的变异行，在全域上判；
   - 补丁改了 `.rs` 的，命名检查两条（`.claude/agent-common.md`「命名检查交主 agent 跑」）；
   - 写段缩过域、第 1 步没罩到的 `cargo test` 点名目标（实现员报告里 `env <缩域变量>=<n>` 起的那几条），不缩域再判一次。
   这一步点到会走崩溃枚举的目标的，范围声明的为什么一段末尾写全域理由（写法照 `.claude/agent-common.md`「不做」一节「门禁只有一个，跑法两种」那一条）。
   崩溃放量分项放开全域时入口退 79（点名的分项没缩域、这一趟要现判的崩溃状态超 200000 或估不出；`scripts/crash/crash-amplification.sh` 文件头退出码一节），或某一项用时超过四小时被服务打回（结局 `over-four-hours`）：都不算绿，也不拿缩域顶替；点名项的命令带 `--full-gate` 服务当场拒，没有全域的跑法，主 agent 弹窗请用户在两种里定——先拆再合（拆出的改动进写段下一批，这一批停在验证段），或留给提交时不指定范围的那一趟判（写进调度记录，这一批照样合）。
   这一步有红的：修补交实现员（`refused-test-registry.py list` 报的照「合入后验证之后判新出现的拒算」第 2 条办），副本从这棵快照建、修补以它为基线写，修补的补丁目录接在这一批最后，整批重交；第 2 步重交时点新一趟第 1 步在「验证段第 2 步」那一段列的项，加这一段全部补丁（含修补）改到的项。
3. 「合进主工作区」那一段的各条（第 1 步最后那一趟在汇总行之后打出）：
   - 先在仓根直接跑核的那几条（主工作区自那一趟开跑以来，补丁改到的包连同依赖它们与被它们依赖的包、树根的 `Cargo.toml`、`Cargo.lock`、`.cargo/` 变没变，这一批要换掉、删掉的变异行被改没有；`scripts/gate/merge-and-verify.sh` 文件头「开跑那一刻的指纹」），都退 0 才往下；退非 0 不合，整批重交。
   - 再交 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <日志目录>/merge-patch`：几份补丁连同登记行合成的一份补丁目录，它一次整份写完或一个字不改。它退 2（一个字没改）照那一段写的原因分开办：开着的三方轮开工快照里有这几份文件的，等那一轮判决写完；同一个仓有别的在换的，等它换完；这两样等完重做核的那几条、再交同一条，不重验。三方合并冲突落在登记表（`.claude/gate.d/stage-inputs.tsv`、`.claude/gate.d/crash-point-replay-items.tsv`、`.claude/gate.d/refused-tests.tsv`、`scripts/crash/crash-amplification.sh` 的 `ITEMS`）或变异表 `crates/mutations.tsv` 上的，不重做核的那几条、不再交同一条、不照拒绝说明手工并，整批重交。别的文件上的三方合并冲突，与核和换之间主工作区被改了的，重做核的那几条：退非 0 整批重交，退 0 再交同一条。走不到的理由那一节行号对不上的，照拒绝说明点名的 文件:行 在 `<日志目录>/merge-patch/result.md` 末尾补一节再交同一条。结局不是 2 的（被停、撞内存上限这类）先交 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --pending-status`，退 3 的照它的出路 `--rollback-pending` 退干净（或 `--finish-pending` 换完），再照退 2 那几种办。
   - 合完交 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --read-outcome <日志目录>/merge-patch/apply-outcome.tsv`，退 0 才算合全；再交它打出的「补丁打进主工作区之后另交」那几条（code-source-discipline 的 mutation-tables 格这类读主工作区的），删这一趟留下的快照。
   - 核的那几条连着三趟退非 0（主工作区一直有人在改补丁碰的包）：跑 `python3 scripts/agents/who-touched.py <核那一条点名变了的路径>… --since-minutes <自那一趟开跑以来的分钟数>` 认出在改的会话，SendMessage 请它们在这一批合完之前不写那几个包，回了再整批重交；认不出、请不动的弹窗问用户。
   - 验证段里跨轮停下，续做时照 `--read-outcome` 那一条判：退 0 的已合，接合完之后那几样；别的从第 1 步整批重交。
4. 没有细则，`.claude/main-agent.md` 验证段第 4 步那一句就是全文。
5. 各处红了怎么修补：
   - 第 1 步红的（补丁还没合进主工作区）：修补以那一趟的快照为基线写（派发提示写明快照路径，实现员的副本从它建），把修补的补丁目录接在第 1 步「→ 怎么办：」打出的那一条最后交；修补还改到别的测试目标的，去掉 `--item` 与 `--fingerprint-of`、范围改写补丁份数。
   - 第 2 步红的：照第 2 步「这一步有红的」那一句。
   - 已合进主工作区之后才红的（代码轮判决打中这类）：修补的实现员副本从主工作区建，第 1 步那一条只给这一批修补的补丁目录，再走第 2 步。
   - 改钉值的实现员照各份实现员报告的算法改，只对合入时改过锚点或冲突的变异行跑 `scripts/mutation/prove-red.sh` 证红（实现员各自证过的不重证，整表归 crates-mutation-replay）。
6. 没有细则，`.claude/main-agent.md` 验证段第 6 步那一句就是全文。

## 合入后验证之后判新出现的拒算

实现员列「拒算的测试」那一节的写法在 `.claude/agents/implementation-writer.md` 第 3c 步。`scripts/gate/merge-and-verify.sh` 与 `scripts/gate/apply-writer-patch.py` 都照打只含拒算表的补丁目录。

1. 判法：对着测试源码，看它的单元闭包为什么读活树。改得成算得出单元键的写法：输入编进二进制、在测试里现造、不读运行时拼出的仓根路径。收的行是「测试函数全名<TAB>为什么非读活树不可」（理由至少 8 个字，写得出判的依据）。收的行这样做成拒算补丁目录（都在自己认领的草稿目录 `<草稿>` 下）：主工作区此刻的拒算表拷三份，到 `<草稿>/a/.claude/gate.d/`、`<草稿>/b/.claude/gate.d/` 与 `<补丁目录>/baseline/.claude/gate.d/`；收的行追加到 `b` 那一份末尾（这一批要删的「已不拒算」行在 `b` 那一份里删）；在 `<草稿>` 里 `git diff --no-index --no-prefix --full-index a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv > <补丁目录>/crates.patch`（有差异退 1）；判的依据逐条写进 `<补丁目录>/result.md`。拒算补丁目录要加行、删行的，照这一条从主工作区此刻重做一份，替掉这一批里原来那一份。拒算补丁目录跟着验证段第 1 步在快照上验、跟着 `merge-patch` 合进主工作区。
2. 那一条写成 `task-server/task submit --cwd <快照> -- python3 scripts/gate/refused-test-registry.py list <快照>`，读 `task-server/task log <号>`。有「未登记」的，先判那条测试函数是不是这一批补丁新加或改到的（对着这一批各份补丁目录的 `crates.patch` 看）：是的照第 1 条判，收的补进拒算补丁目录、不收的退给实现员改测试；不是的不归这一批，认它是哪个会话的、发消息给它（`.claude/main-agent.md`「一轮怎么开、怎么收」第 15 条），这一批的第 2 步不等它。这一批补丁让它不再拒算的「已不拒算」行，照第 1 条在拒算补丁目录里删（只缩不涨）；拒算补丁目录重做了的，整批重交；改测试的修补照验证段第 2 步「这一步有红的」那一句重交。这一批之前就有的「已不拒算」行不归这一批，主 agent 在主工作区直接删。
3. 没有细则，`.claude/main-agent.md`「合入后验证之后判新出现的拒算」第 3 条那一句就是全文。
