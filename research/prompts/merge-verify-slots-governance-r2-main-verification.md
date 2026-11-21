# merge-verify-slots-governance-r2：判决（2026-11-21）

正文 `research/prompts/_merge-verify-slots-governance-r2-body.md`，开工快照 `research/prompts/merge-verify-slots-governance-r2-snapshot/`。这是这一题的第二轮，第一轮判决 `research/prompts/merge-verify-slots-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（sonnet） | `research/prompts/merge-verify-slots-governance-r2-sonnet-output.md` | 第 5 格不成立两处（第一轮第 4 格「M2、M4、M5、M6 归 `agent-watch.py --selftest` 罩」够不着；第 2 格把 `crash-library-totals.sh:280` 判照留不对），其余站得住；第 3 格不成立一处（同一条消息第 280、281 行自相矛盾）；第 4 格锚串全中、理由三处不对 |
| 攻方（opus） | `research/prompts/merge-verify-slots-governance-r2-opus-output.md` | 第 1 格六种情形五种拦得住，「三样一起给」那一道生产那一半打中（只给另一个 `SINGLEFS_WORK_ROOT` 时删掉原工作根各槽的编译目录）、自证那一半不成立；第 2 格两项都没打中；「已经量的」十二句十句核到、两句只核到窄的一句 |

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」）：

- 倒推开工快照的树：`task-server/task submit --cwd ~/.local/share/singlefs/work/merge-verify-slots-r2-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py merge-verify-slots-governance-r2 --out ~/.local/share/singlefs/work/merge-verify-slots-r2-verdict/merge-verify-slots-governance-r2-tree --repository /home/fy5090/code/singlefs` type=succeeded exit=0：「清单点名 16 份，倒推出 16 份，sha256 对上 16 份」。另 `sha256sum -c` 那份清单 14 份全 OK。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-slots-governance-r2-sonnet-output.md research/prompts/merge-verify-slots-governance-r2-opus-output.md --root ~/.local/share/singlefs/work/merge-verify-slots-r2-verdict/merge-verify-slots-governance-r2-tree --background research/prompts/_merge-verify-slots-governance-r2-background.md`：「核了 58 处引文、71 处裸行号，对上 38 处，没判 91 处」，没有对不上的。判决用到的没判几处主 agent 逐处现查：
  - `scripts/crash/crash-library-totals.sh:280`、`:281`；
  - `.claude/agent-common.md:37` 与 `task-server/task_server_lib.py:286`、`:287`；
  - `.claude/memory-bounded-parallelism.tsv:10` 与三个开关的定义处；
  - `.claude/preflight-exclude` 第 164–167 行。
- 两条腿的模型都拷到 `~/.local/share/singlefs/work/merge-verify-slots-r2-verdict/` 下的副本里复跑（拷完逐份 sha256 与报告开头记的相同）：
  - 辩方：`task-server/task submit --cwd <核查目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <快照目录> <运行目录> <副本>` type=succeeded exit=0。读数：
    - 「name=judged_files_unchanged ok_count=14」；
    - `slot_nonrs` 各步 got 与 expected 逐步相同、「name=slot_nonrs step=end different=0」；
    - `registration_rows` 逐项相同，例「item=reason-names-stage-replay-jobs-by-memory got=0」「item=entry-calls-preflight got=0」；
    - `DIFFERENT` 0 行。
  - 模板自证 `task-server/task submit --cwd <核查目录> -- bash <副本>/tools/rerun.sh --selftest` type=command-failed exit=1，红在模板自己「拷进与仓无关的临时目录、传参应当跑通」那一格。这是腿模型的工具，不是被判文件，读数以整份复跑为准。
  - 攻方：`task-server/task submit --cwd <核查目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <冻结副本> <运行目录> <副本>` type=succeeded exit=0。读数：
    - 「name=slot_trim_race_summary arm=intact scenarios=120 violations=0」，`slot-lock-skipped` 那一臂各场景有违例；
    - 「arm=gap-take idle=删了 kept=删了 held=删了 busy=在」「arm=gap-zero idle=删了 kept=删了 held=删了 busy=在」；
    - 「arm=fix-namespace idle=在 kept=在 held=在 busy=在」「arm=fix-same-root idle=在 kept=在 held=在 busy=在」；
    - `copy_exclusions` 两种快照都不拷 `target-spec/`、`fixtures/target/`、`__pycache__`，「name=copy_exclusions_tracked count=0」；
    - `incremental_holder` 四臂（unset、empty、one、zero）读数与报告相同。
- 主 agent 另跑：
  - `bash .claude/gate.d/code-tooling.sh --check repository-copy-closure,memory-bounded-parallelism` type=command-failed exit=1。memory-bounded-parallelism 绿；repository-copy-closure 「锚串……不命中」0 行，红的三处是 `scripts/gate/lib_function_units_selftest_rust_cells.py:1026`、`:1027` 与 `scripts/gate/merge-and-verify-steps-lib.sh:486`，都不是这一批改的。
  - `python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py <仓根>` type=command-failed exit=1「89 个脚本……54 处不合格」，e203 只剩入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 三行。

## 二、判决

1. **第 1 格（改法拦不拦得住、有没有新毛病）**：
   - 只占前 `slots` 槽、`slots=0` 清多出来的槽、清槽与另一趟同时挑槽：都成立，模型 120 个场景 0 违例。
   - 删 `target` 一类不误删源树：成立，两种快照本来都不拷那几样，仓里跟踪的路径落在这类目录下 0 份。
   - 关掉增量之后 cargo 的环境、删 `incremental` 有进程开着时的出路：都成立。
   - 「三样一起给」那一道：打中，坐实。只给另一个 `SINGLEFS_WORK_ROOT`、不给自证那两样时，那一道不判，编译根取这一台真的那一个。清理拿新工作根的槽那一层比，把原工作根下留着的、别的趟占着的槽的编译目录都删了（`gap-take`、`gap-zero` 两臂），只有开着文件的那一槽留下。判定不会因此错，代价是那几槽下一趟从头编。自证里没有一格去掉这三样：`scripts/gate/merge-and-verify-selftest-lib.sh:27` 一起导出。
   - 攻方另记的几处只多占盘（不占槽那一趟照写 `incremental/`、调用方自设 0 时旧缓存不删、超上限清理排在删增量之前）与 `scripts/hosts/run_build_directory.py:109` 那句说法的对象，不改判定，这一轮不改。
2. **第 2 格（多机与旧东西）**：没打中。`MULTI_HOST_RUN_PEER_SCRATCH` 认法对；没有为旧东西留的分支。辩方补的一点成立：「第二台从不占槽」靠第二台那一层给 `CARGO_TARGET_DIR` 保证，脚本不验自己在哪一台；此刻没有人在第二台上不经那一层直接起它，照实写进文件头。
3. **第 3 格（正文）**：
   - 四处成立：规则第 65 行、技能第 22–23 行、两份定义的例外（不比槽宽）、`prove-red.sh:232`。
   - 一处不成立：`scripts/crash/crash-library-totals.sh:280`「编译目录归这一趟」与第 281 行「编译目录归树、不归任务」在同一条消息里相反。第一轮把第 280 行判成照留是判错了，改。
   - 辩方旁生的一处坐实：`.claude/agent-common.md:37` 与 `.claude/skills/dispatch/SKILL.md:18` 写「用完 `task-server/task clean <号>` 删」编译目录，而 `task-server/task_server_lib.py:286` 写 clean「只删那一条任务自己的现场与分片父任务的测试二进制」。这一句 HEAD 里就有，不是这一批改的，交管定义的会话。
4. **第 4 格（登记行）**：锚串都命中，理由三处不对。
   - `.claude/memory-bounded-parallelism.tsv:10` 把开关配错了格。代码里是 `break-switch-replay`—`replay-jobs-by-memory`、`selftest-cells-alone`—`cells-alone-jobs-by-memory`、`stage-break-switch-replay`—`stage-replay-jobs-by-memory`。
   - `.claude/preflight-exclude` e203 四行写「准入与运行条件由入口判」，入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 自己既没写 `admission:`、也不调 `preflight`（preflight-lint 点名它三行，HEAD 里就这样）。理由改成照实写。
   - `scripts/experiment-apparatus/README.md:7` 的 `ls -d …/*/` 列 48 个目录，只有 31 个有 `main.rs`。
5. **第 5 格（第一轮判决够不够得着）**：
   - 第一轮第 1 格的几条、三槽留满只报不收、第二台只改说法：站得住。
   - `slot.4u3rewvh` 不是作废：槽里 `.deliverables` 现读有「repo 基线」那一行，比第一轮引的调度记录转述更直接。
   - 两处够不着，坐实：
     - 第一轮第 4 格「M2、M4、M5、M6 归 `scripts/agents/agent-watch.py --selftest` 罩」不对。自证造的进度文件恰好 3 行，`PROGRESS_TAIL_LINES` 也是 3，M6（取头不取尾）在自证里照绿。M4 是等价改坏；M2、M5 只在读文件的竞态窗口走到。
     - 第一轮第 2 格把第 280 行判照留（见第 3 格）。
6. **「已经量的」**：两句只核到窄的一句。
   - 「红的格与点名表逐条相同」只对新开关七条字面成立；补了点名的六条另红早先已点名的格（`verify-in-main-tree` 红了 28 格），这几条与文件头点名是否逐格相同归开关复跑那一格判。
   - 「没再单跑那五格」：主 agent 这一轮另跑了其中两格（见核查）。

## 三、改法

第二轮的改法，改完开第三轮（第三轮攻第二轮的修补，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」，最多三轮），不碰产品代码，不做 A/B。

1. 「三样一起给」那一道（工具员）：照攻方腿模型 `fix-same-root` 那一臂改——没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时，工作根要等于这一台缺省的那一个，不等就不占也不清，打一行为什么。不取 `fix-namespace`：它改编译根下的布局，此刻两槽 105G、91G 的编译目录会成孤儿。自证加一格「只给另一个工作根时不占也不清、原工作根下各槽的编译目录都在」，配弄坏开关。换上要等别的会话那一批（它改 `scripts/gate/merge-and-verify-selftest-lib.sh` 与文件头开关表）换上之后，对着那一版出。
2. `scripts/crash/crash-library-totals.sh:280` 失败句改成与第 281 行一致（工具员，同一件）。
3. `scripts/agents/agent-watch.py --selftest` 或 `scripts/agents/tests/test_agents.py` 加一格：进度文件行数多于 `PROGRESS_TAIL_LINES`，M6 那种取头不取尾的改坏要红（工具员，同一件）；M4 等价、M2 与 M5 只在竞态窗口走到，写进那一格的说明。
4. 登记行（主 agent 自己改）：`.claude/memory-bounded-parallelism.tsv:10` 开关与格照代码配对；`.claude/preflight-exclude` e203 四行理由照实写；`scripts/experiment-apparatus/README.md:7` 改成按 `main.rs` 列。
5. `.claude/agent-common.md:37` 与 `.claude/skills/dispatch/SKILL.md:18` 的 `task clean` 说法交管定义的会话。

## 交用户的

没有要用户定的。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：槽位一段判了只占前 slots 槽、清多出来的槽、关掉增量与第二台的说法，成立
- scripts/gate/merge-and-verify-snapshot-lib.sh：choose_slot、take_slot、refresh_slot_snapshot、slot_incremental_off 判了清槽竞争、删 target 与 incremental，「三样一起给」那一道打中
- scripts/gate/merge-and-verify-selftest-lib.sh：槽位自证格判了，没有一格去掉自证那三样
- scripts/gate/merge-and-verify-slots.conf：只在本机占槽的说法成立
- .claude/rules/test-script-discipline.md：第 65 行同一棵树的判据与脚本一致
- .claude/skills/merge-verify/SKILL.md：第 22–23 行放槽命令与脚本同一条
- .claude/agent-common.md：第 37 行槽的例外不比槽宽；同一行 task clean 的说法与服务相反，交管定义的会话
- .claude/main-agent.md：第 34 行槽的例外不比槽宽
- scripts/mutation/prove-red.sh：第 232 行引规则那一句成立
- scripts/crash/crash-library-totals.sh：第 280 行与第 281 行同一条消息说法相反
- .claude/gate.d/repository-copy-closure-exempt.tsv：第 41 行锚串与理由对，全表零命中 0 行
- .claude/memory-bounded-parallelism.tsv：第 10 行开关与格配错
- scripts/experiment-apparatus/README.md：第 7 行列的目录比装置多
- .claude/preflight-exclude：e203 四行锚串对，理由不真

## 回看决策

不涉及决策：这一轮判的是合入后验证的槽位、规则与定义正文、几张登记表，都是治理文件，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里的任何一条。
