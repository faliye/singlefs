# 合入后验证快照放固定槽位、拆分之后的登记行（治理文件轮，第一轮）

这一批没改 `crates/` 下的代码，判的都是合入后验证脚本、它的槽位登记、规则与技能正文、几张排除表与搬迁登记、一份工具测试；方案照 `crates/` 今天的实现来谈（合入后验证编的是 `crates/` 下的包，槽里放的是整棵仓的快照）。调度记录 `records/2026-11-18-合入后验证压盘.md` 第 99 行起「快照槽位换上与停验」「crates 拆分之后接手的登记行」「开工之后的开关复跑」三段。开工快照 `research/prompts/merge-verify-slots-governance-r1-snapshot/judged-sha256.txt` 点名的就是被判的文件。

## 问题

这一批换上了两组改动。判它们判得对不对、拦不拦得住要拦的事、有没有误伤与绕过，规则与技能正文写得对不对，登记行指得对不对。

## 改了什么（主 agent 的观测，工具员报告与换上结局）

1. **合入后验证快照放固定槽位**（用户定案在 `records/2026-11-13-全量提交门禁前收尾调度.md` 第 1508 行 ② ③）：
   - `scripts/gate/merge-and-verify.sh` 文件头「槽位」一段（第 80–113 行）与 `scripts/gate/merge-and-verify-snapshot-lib.sh`（`take_slot`、`refresh_slot_snapshot`、`release_slot`、`finish` 的槽位那一支）。
     - 快照放在 `<工作根>/merge-and-verify/slot.<随机>/repo`，两台共享工作根；一槽的编译目录在每一台的编译根下 `merge-and-verify-slots/<槽名>/target`，每台各一份。
     - 占槽由一个 python 进程持锁，持到这一趟收尾；拷进槽用 `rsync --recursive --links --checksum --no-times`，内容变了的文件修改时刻是此刻，没变的留着旧的。
     - 拷进槽时内容变了、新加、删掉的 `.rs` 写进 `<日志目录>/slot-changed-sources`，重编核对另读它。
     - 占到槽之后、编之前与收尾各量一次占盘，超过上限就整个删。
   - 槽数上限与占盘上限只从新登记的 `scripts/gate/merge-and-verify-slots.conf` 读（`slots=3`、`slot-target-limit-gib=150`），这两个值是工具员定的，没量过。
   - 自证里加了「槽位：…」九格（`scripts/gate/merge-and-verify-selftest-lib.sh`），开关表加「槽位」一族 10 条。
   - `.claude/rules/test-script-discipline.md` 第 65 行由「编译目录归这一趟：每一趟新建、跑完删……不跨趟、跨轮、跨会话、跨脚本共用」改成「不同的树不共用编译目录……同一棵树先后几趟可以接着用它那一份增量编」。
   - `.claude/skills/merge-verify/SKILL.md` 第 22、23、43 行与 `scripts/gate/README.md` 第 42、44 行跟上（快照在槽里、放槽怎么做）。
2. **crates 拆分之后的登记行**：
   - `.claude/gate.d/path-moves.tsv` 新加 49 行 `X.rs → X/main.rs`，是 HEAD 里有、磁盘上没了、同名目录下有 `main.rs` 的那几份，现算得出。
     - 这次改动里已登的 17 行 `crates/singlefs-verification/src/bin/<名>.rs → crates/singlefs-implementation-experiments/src/bin/<名>.rs`，被拆分又挪成 `<名>/main.rs`，所以新前缀直接改成指 `<名>/main.rs`。原因是再接一行会串（`.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」）。
   - 六张表改了 68 处，都把路径改到拆出的那一份文件：`.claude/heavy-thread-exempt`、`.claude/gate.d/repository-copy-closure-exempt.tsv`、`.claude/gate.d/hardcoded-quota-exempt.tsv`、`.claude/memory-bounded-parallelism.tsv`、`.claude/preflight-exclude`、`.claude/term-rename-exempt`。
     - `.claude/preflight-exclude` 删了 17 行（装置与 src/bin 拆成目录之后入口不再是那份 `.rs`），另登了 replay.sh 的八份库与 e165 两份模块。
     - `.claude/term-rename-exempt` 有 9 行整份豁免改成目录写法。
   - `scripts/agents/README.md`、`scripts/hosts/README.md`、`scripts/crash/README.md`、`scripts/prune-server/README.md`、`scripts/gate/README.md`、`scripts/experiment-apparatus/README.md` 加了拆出文件的表行。
   - `scripts/agents/tests/test_agents.py` 的 `WatchTests.test_progress_deleted_before_read_is_reported`：补丁改打在 `sys.modules['agent_watch_alerts']` 上，因为 `progress_note` 拆进了 `agent_watch_alerts`，在那个模块里按全局名查 `progress_file_of`。

## 已经量的（结局行 type= exit= 那一段）

- 第 1 组：
  - 换上之后自证 type=succeeded exit=0。
  - 规则与技能经 `scripts/kb/replace-batch.py` 写，5 处命中、回读一致。
  - 开关复跑（`bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`）type=command-failed exit=1：「开关 153 条、对上 149 条」。槽位那一族 10 条全部对上；不符的 4 条（ignore-red、merge-in-commands-dropped、fallback-runs-whole-workspace、items-ignored）红在别的会话这期间新加的「判不出的留给提交时整轮门禁」几格，那个会话认了，在补点名，这一轮不判这 4 条。
  - 省盘没量到：拆分之后还没有一趟真跑的合入后验证编出来。
- 第 2 组：
  - `test_agents.py` 那一个用例「Ran 1 test … OK」type=succeeded exit=0；证红副本（补丁打回入口模块）「FAILED (errors=1)」type=command-failed exit=1。
  - preflight-lint type=command-failed exit=1：拆分带出的十份已消，剩 66 处都在拆分前就有的文件上。
  - `scripts/kb/sweep-term.py --check` exit=2，只剩 `.claude/agents/implementation-writer.md` 一条按行豁免对不上（HEAD 里那一句也不在）。
  - code-tooling 七格（heavy-thread-defaults、hardcoded-quota、repository-copy-closure、memory-bounded-parallelism、peer-launch-direct、change-range-single-source、directory-layout）type=command-failed exit=1，七格都红。原因归判法：排除表只缩不涨不认拆分族、扫描格逐份文件判、装置 `tests.rs` 落在 scripts/ 第 4 层、preflight 只认一层。已转给管这几格的会话，这一轮不判判法本身，只判登记行写得对不对。

## 已知还在说旧规矩的几处（主 agent 现查）

`grep -rn "编译目录归这一趟"` 在 `research/`、`records/` 之外命中：`scripts/crash/crash-library-totals.sh:280` 的失败句、`scripts/mutation/prove_red_selftest.py:544`、`:570`、`:575` 三处自证说明；`scripts/hosts/multi-host-run.sh:380` 与 `task-server/task_server_lib.py:1159` 写「第二台每一趟新建、跑完删」。这几处没改。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。

1. **槽位拦不拦得住它要拦的事**：
   - 同一槽先后两趟，前一趟留下的产物会不会让改过的包不重编而判绿（`slot-changed-sources` 漏收的情形：改了 `.rs` 以外的 `build.rs` 输入、`Cargo.toml`、`.wgsl`、`include_str!` 读的数据文件）。
   - 两台同时挑槽时排他锁在 NFS 上靠不靠得住（工作根挂的是 NFS 4.2、`local_lock=none`）。
   - 被 cgroup 杀的那一次尝试留下的槽，下一次尝试与别的任务会不会抢错。
   - 留着的槽（`kept`）在主 agent 忘了放槽时会不会把三槽占满，占满之后这一趟照一趟一份跑而不报。
   - 占盘上限 150 GiB 每台三槽合计 450 GiB，与本机根分区余量对不对得上（推的，没量过；这一格只判上限的读法与超限处理对不对，数值本身交用户）。
2. **规则与技能正文**：
   - `.claude/rules/test-script-discipline.md` 第 65 行新句合不合 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（只写怎么做），与 `scripts/gate/merge-and-verify.sh` 文件头「槽位」的行为是不是一回事（「同一棵树」指什么，槽里每一趟拷进来的快照算不算同一棵）。
   - `.claude/skills/merge-verify/SKILL.md` 三处放槽的写法，与脚本收尾打的那一行是不是同一条命令。
   - 「已知还在说旧规矩的几处」里每一处该不该跟着改：说的是那份脚本自己的做法（照留），还是在引已改的规矩（要改）。
3. **登记行指得对不对**：
   - 六张表里改过的每一行，锚串在新路径那份文件里是不是恰好命中、理由在新文件上还成不成立。主 agent 已知 `.claude/gate.d/repository-copy-closure-exempt.tsv` 第 13 行（e171 `row_driver.py`）拆分前就对不上，没动。
   - `.claude/preflight-exclude` 删掉的 17 行，拆后的入口（`<名>/main.rs` 或目录下的入口）还需不需要排除；新登的十行「同目录 source 的函数库、没有自己的入口」对不对。
   - `.claude/gate.d/path-moves.tsv` 新加与改过的 66 行，各自满足不满足 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」里的搬迁实据，生效的行之间串不串。
   - 六份 README 新加的表行，点名的文件在不在、说的用途对不对。
4. **测试那一处**：`scripts/agents/tests/test_agents.py` 改后的用例，`progress_note` 读进度文件的那一支被改坏时还会不会红；补丁打在 `sys.modules['agent_watch_alerts']` 上，要是 `agent-watch.py` 换一种载入方式（模块名不叫 `agent_watch_alerts`），它是报错还是静默过。
5. **多机与不兼容旧东西**：
   - 槽的编译目录按台分、快照在共享根，派到两台中任何一台都跑得起、判得对没有（`.claude/rules/verification.md`「所有任务都要多机可用」）。
   - 槽位登记读坏了退 1、不退回旧做法；`slots=0` 照一趟一份那一种跑，算不算给旧做法留的开关（`.claude/agent-common.md`「写」一节「里程碑 17 收尾之前不兼容任何旧东西」）。
   - 有没有别的为旧路径、旧行为留的分支。

## 分工

正推腿判第 2、3、4 格；攻方腿判第 1、5 格，再核「已经量的」那一节里每一句能不能从日志里核到。
