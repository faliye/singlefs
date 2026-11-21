# 合入后验证快照放固定槽位、拆分之后的登记行（治理文件轮，第二轮）

这一批没改 `crates/` 下的代码，判的是合入后验证脚本的槽位一段、它的登记、规则与定义正文、几张登记表；方案照 `crates/` 今天的实现来谈（合入后验证编的是 `crates/` 下的包）。第一轮判决 `research/prompts/merge-verify-slots-governance-r1-main-verification.md`，调度记录 `records/2026-11-18-合入后验证压盘.md` 第 125 行起「治理文件三方第一轮」那一段。开工快照 `research/prompts/merge-verify-slots-governance-r2-snapshot/judged-sha256.txt` 点名的就是被判的文件。

## 问题

第一轮判决「三、改法」第 1–3 条与用户定的「槽位编译关掉增量」都已换上。判两件：改法本身对不对、有没有新毛病；第一轮的判决够不够得着、有没有判错。

被判的 `scripts/gate/merge-and-verify.sh` 与 `scripts/gate/merge-and-verify-selftest-lib.sh` 里另有别的会话这期间换上的两批（`--unresolved-to-commit-gate` 一族、变异锚点核一族），这一轮不判那两批，只判槽位一段（文件头「槽位」、挑槽那段 python、开关表槽位一族、槽位自证格）与 `scripts/gate/merge-and-verify-snapshot-lib.sh` 全份。

## 改了什么（主 agent 的观测，工具员报告与换上结局）

1. 槽位四处（工具员报告 `~/.local/share/singlefs/work/merge-verify-slots-r1-fixes/result.md`）：
   - 挑槽只在按 `used` 排好的前 `slots` 槽里挑（挑槽 python 的 `eligible`、`surplus`）。
   - `slots` 小于已有槽数（含 0）时，在 `slots.lock` 下清多出来的、没被占也没留着的槽，连同这一台编译根下它的编译目录；`slots=0` 也起挑槽进程的 `trim`。
   - 另加一道：只给自证的 `MERGE_AND_VERIFY_SLOT_TABLE`、`MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 要连同 `SINGLEFS_WORK_ROOT` 三样一起给，缺一样不占也不清；这一道没配格与开关。
   - 拷进槽之前删掉槽里快照下的 `target/`、`target-*/`、`__pycache__/`（`.git` 照留）；放槽命令收成 `slot_release_command` 一处，三处都打 `rm -rf -- '<槽>/kept' '<槽>/.owner' '<槽>/.deliverables' '<槽>/repo/target'`。
   - 收尾第一步打「槽位汇总：」：这一趟没占槽的打为什么，逐个列留着的槽（槽名、留了几天、`.owner` 里的会话、放槽命令），只报不收。没并进 `merge-and-verify-items-lib.sh` 的 `汇总：` 行（那份是别的会话在改的）。
   - 文件头与 `scripts/gate/merge-and-verify-slots.conf` 改成只在本机占槽；`CARGO_TARGET_DIR` 落在 `MULTI_HOST_RUN_PEER_SCRATCH` 下面时那一行说「第二台那一层给了编译目录」。
2. 关掉增量（工具员报告 `~/.local/share/singlefs/work/merge-verify-slots-incremental-off/result.md`，用户定案在调度记录第 145 行）：`choose_slot` 建好槽的编译目录之后调 `slot_incremental_off`，没设 `CARGO_INCREMENTAL` 的导出 0、删这一槽编译目录下已有的 `<档>/incremental` 与 `<目标三元组>/<档>/incremental`；调用方设了的照它、不盖、不删。工具员在文件头写的「生效的 incremental 进按条复用的单元键……不互相复用」那一句与 `.claude/kb/decisions/13-验证路线.md:464` 相左，主 agent 删掉了。
3. 正文：`.claude/rules/test-script-discipline.md:65` 补「固定路径、每趟按内容拷进来的那一份算同一棵树」；`.claude/skills/merge-verify/SKILL.md` 第 23 行措辞与放槽命令；`.claude/agent-common.md:37`、`.claude/main-agent.md:34` 给合入后验证的槽留例外；`scripts/mutation/prove-red.sh:232` 引规则那一句；`scripts/crash/crash-library-totals.sh` 第 35 行与第 281 行的编译目录说法。
4. 登记行（主 agent 自己改）：`.claude/gate.d/repository-copy-closure-exempt.tsv` 第 41 行理由改指 `scripts/gate/lib_function_units_unit_graph.py`、第 44 行删；`.claude/memory-bounded-parallelism.tsv` 第 10 行理由；`scripts/experiment-apparatus/README.md` 第 7 行不再逐个列；`.claude/preflight-exclude` 登 e203 拆出的四份模块。

## 已经量的（结局行 type= exit= 那一段）

- 槽位四处换上：`hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0。整份自证 type=succeeded exit=0「✓ 自检通过（查了 172 种）」；新开关七条与补了点名的六条单跑都 type=command-failed exit=1，红的格与工具员报告的点名表逐条相同。
- 关掉增量换上：type=succeeded exit=0；之后整份自证 type=succeeded exit=0「自检通过（查了 183 种）」；`slot-incremental-on`、`slot-incremental-cache-kept`、`slot-incremental-caller-overridden` 只红「占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0……」或「调用方设了 CARGO_INCREMENTAL 的照它……」那一格；`slot-target-on-shared-root` 红那一格与「占槽、放槽，槽的编译目录在这一台的编译根下」两格。
- 点这份脚本的开关复跑：换上关掉增量之后那一趟开跑前「174 条里拒算 174 条」（拒算在别的会话那一批的一处 `bash -c`，已由它改掉）；此刻别的会话交的那一趟复跑在跑，罩这份脚本全部开关。
- `.claude/singlefs-ai-sop/scripts/rules-lint.sh` type=succeeded exit=0「命中 0」；`doc-lint.sh` type=command-failed exit=1，这一批改过的文件里只命中 `.claude/agent-common.md:47`，那一行 HEAD 里就有、不是这一批改的。
- 门禁五格（第一轮判决里那一趟）之后，登记行五处改完，没再单跑那五格。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。

1. **改法拦不拦得住第一轮打中的事、有没有新毛病**：
   - 已有 3 槽、上限 1 时只占前 1 槽；`slots=0` 时清多出来的槽而不碰留着的、占着的（`slot.4u3rewvh` 此刻是别的会话有意留着的槽）。
   - 清槽在 `slots.lock` 下、对每一槽另加一次锁，与另一趟同时挑槽时会不会删掉别的趟刚占的槽。
   - 「三样一起给」那一道漏了会怎样、自证里别的格会不会因此去删真编译根下的槽编译目录。
   - 拷进槽之前删 `target` 一类，会不会删到源树里本来就叫 `target` 的目录（例如测试样本里的）。
   - 关掉增量之后，占槽那一趟与不占槽那一趟各起的 cargo 收到的环境对不对；删 `incremental` 用 `run_build_directory.py remove`，有进程开着时的出路。
2. **多机与不兼容旧东西**：只在本机占槽、第二台那一行的认法（`MULTI_HOST_RUN_PEER_SCRATCH`）认得对不对；有没有为旧槽布局、旧放槽命令留的分支（`.claude/agent-common.md`「写」一节「里程碑 17 收尾之前不兼容任何旧东西」）。
3. **正文**：第 3 条的五处正文与脚本现行的行为对不对得上，合不合 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（只写怎么做）；`.claude/agent-common.md:37`、`.claude/main-agent.md:34` 的例外有没有放得比槽更宽。
4. **登记行**：第 4 条的五处改完之后锚串与理由对不对。
5. **辩方：第一轮判决够不够得着**：第一轮「二、判决」逐条复核——判「不成立」「成立」的依据站不站得住，判出局的那一方（例如第 1 格「`slot-changed-sources` 只收 `.rs` 不改」、第 4 格「M2、M4、M5、M6 照绿不加」、「第二台从不占槽只改说法」）有没有被判错，改法是不是同样打中所有替代方案。

## 分工

攻方腿判第 1、2 格，并核「已经量的」那一节每一句能不能从日志里核到；辩方腿判第 5 格，另判第 3、4 格。第一轮攻过的角度（调低上限、`slots=0` 不清、`repo/target` 不量不删、三槽留满只打一行、第二台的说法、`slot-changed-sources` 只收 `.rs`、NFS 上的锁、cgroup 杀掉的尝试）不重复攻，只攻它们的修补。
