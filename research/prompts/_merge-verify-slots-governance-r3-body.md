# 合入后验证快照放固定槽位、拆分之后的登记行（治理文件轮，第三轮）

这一批没改 `crates/` 下的代码，判的是合入后验证脚本的槽位一段、看门狗自证、一份崩溃脚本的失败句、共用约束一句与三张登记表；方案照 `crates/` 今天的实现来谈（合入后验证编的是 `crates/` 下的包）。前两轮判决 `research/prompts/merge-verify-slots-governance-r1-main-verification.md`、`research/prompts/merge-verify-slots-governance-r2-main-verification.md`，调度记录 `records/2026-11-18-合入后验证压盘.md` 第 164 行起「治理文件三方第二轮」那一段。开工快照 `research/prompts/merge-verify-slots-governance-r3-snapshot/judged-sha256.txt` 点名的就是被判的文件。这是最后一轮（`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」最多三轮）。

## 问题

第二轮判决「三、改法」第 1–5 条都已换上。判这些修补本身对不对、有没有新毛病，加第二轮判决里写明只抽查过的地方。

被判的 `scripts/gate/merge-and-verify.sh` 与 `scripts/gate/merge-and-verify-selftest-lib.sh` 里另有别的会话换上的几族（`--unresolved-to-commit-gate`、变异锚点核、合入前 `--dry-run`），这一轮不判，只判槽位一段与 `scripts/gate/merge-and-verify-snapshot-lib.sh` 全份。

## 改了什么（主 agent 的观测，工具员报告与换上结局）

1. 「三样一起给」那一道（工具员报告 `~/.local/share/singlefs/work/merge-verify-slots-r2-same-root/result.md`，照第二轮攻方腿模型 `fix-same-root` 那一臂）：
   - `choose_slot` 没给 `MERGE_AND_VERIFY_SLOT_BUILD_ROOT` 时，拿工作根与 `.claude/hooks/lib_work_root.py shared work`（不看 `SINGLEFS_WORK_ROOT`，给数据根下的 work）按真路径比，不等就打一行为什么、不占也不清。取不到缺省工作根也不占不清。
   - 文件头「槽位」一段写了这一条；自证新加三格（只给另一个工作根时登记三槽、登记 0 槽各一格，原工作根下各槽编译目录都在；工作根就是缺省的那一个时照常占槽，经符号链接给）；开关两条 `slot-work-root-unchecked`、`slot-default-work-root-refused`。
2. `scripts/crash/crash-library-totals.sh:280` 失败句改成「编译目录归树、由服务给，不照仓根的 target 编」（工具员报告 `~/.local/share/singlefs/work/merge-verify-slots-r2-fixes/result.md`）。
3. 看门狗自证（同一份报告）：`scripts/agents/agent_watch_alerts.py` 的 `selftest_scratch_mentions` 加一格，造多于 `PROGRESS_TAIL_LINES` 行的进度文件，断言附的是末几行；开关 `AGENT_WATCH_BREAK=progresstailhead`（取头几行），`scripts/agents/agent-watch.py` 文件头开关表加一条。没加在 `scripts/agents/tests/test_agents.py`。
4. `.claude/agent-common.md:37`「用完就删」里 `task clean` 那一句改成照服务现行说法：编译目录归树不归任务，结局、`task clean`、取消都不删它，`task clean` 只清那一条任务自己的现场。同一说法的技能与钩子几处归别的会话在改，不在这一轮。
5. 登记行（主 agent 自己改）：`.claude/memory-bounded-parallelism.tsv:10` 三格与三条开关照代码配对；`.claude/preflight-exclude` e203 四行理由改成「入口自己的准入与运行条件归入口那一份，preflight-lint 照判入口」；`scripts/experiment-apparatus/README.md:7` 改成「全部用 `ls scripts/experiment-apparatus/*/main.rs` 列」。

## 已经量的（结局行 type= exit= 那一段）

- 第 1 条换上：`hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0「换上了 3 份」。整份自证 type=succeeded exit=0「自检通过（查了 202 种）」、`✗ 自检` 0 行；`slot-work-root-unchecked` 单跑 type=command-failed exit=1，只红「登记三槽……不占也不清」「登记 0 槽……不清」两格；`slot-default-work-root-refused` 单跑 type=command-failed exit=1，只红「工作根就是这一台缺省的那一个时照常占槽」一格。点这份脚本的开关复跑交了、等结局。
- 第 2、3 条换上：`hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0；`crash-library-totals.sh --selftest` type=succeeded exit=0；`agent-watch.py --selftest` type=succeeded exit=0「自检通过」、`✗ 自检` 0 行；`AGENT_WATCH_BREAK=progresstailhead` 单跑 type=command-failed exit=1，只红「进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」一格。点 `scripts/agents/agent-watch.py` 的开关复跑在跑（第 4 次尝试，到 128 / 140 条都对上）。
- 第 4 条：`replace-batch.py` 先 dry-run 再写「1 处替换」回读一致。
- 第 5 条：`replace-once.py` 逐处「命中 1 次」回读确认；改之前的那一趟门禁两格与 preflight-lint 见第二轮判决「一、腿与核查」，改之后没再跑。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。第一、二轮攻过的角度（调低上限、`slots=0` 不清、`repo/target`、三槽留满、第二台的说法、`slot-changed-sources` 只收 `.rs`、NFS 上的锁、cgroup 杀掉的尝试、清槽与同时挑槽的竞争、删 `target` 误删源树、关增量之后的环境）不重复攻。

1. **第 1 条修补**：
   - 统一任务服务交给命令的工作根与 `lib_work_root.py shared work` 在本机、在第二台各是什么，按真路径比会不会把正常的那一趟判成「不是缺省的那一个」而一槽都不占（符号链接、NFS 挂载点、`SINGLEFS_DATA_ROOT`）。
   - 取不到缺省工作根时的出路对不对。
   - 三格自证造的假数据根、假多机配置与假编译根，会不会在某种环境下碰到真的那一份。
   - 两条新开关与已有开关会不会连带红新格而开关表没点名。
2. **第 2–4 条修补**：
   - `crash-library-totals.sh` 第 280、281 行与它文件头、`task-server/task_server_lib.py:287` 是不是同一说法。
   - 看门狗新格分不分得出取头与取尾，有没有别的改坏照样绿。
   - `.claude/agent-common.md:37` 新句与 `task-server/task_server_lib.py:286`、`task-server/README.md:224` 是不是同一件事，合不合 `.claude/singlefs-ai-sop/rules/rules-discipline.md`。
3. **第 5 条修补**：三处登记行改完之后锚串命中、理由与代码或现场对得上。
4. **第二轮只抽查过的地方**：第二轮判决写明不改的几处（攻方腿记的只多占盘三处、`scripts/hosts/run_build_directory.py:109` 那句的对象）站不站得住；「已经量的」每一句能不能从日志里核到。

## 分工

正推腿判第 2、3 格；攻方腿判第 1、4 格。
