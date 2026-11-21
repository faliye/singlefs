# merge-verify-slots-governance-r2：辩方腿（three-way-defense，Sonnet）

日期：2026-11-21。轮：治理文件轮第二轮。立场：辩方，复核第一轮判决 `research/prompts/merge-verify-slots-governance-r1-main-verification.md`「二、判决」逐条，另判正文判据格第 3、4 格（第 5 格就是「第一轮判决够不够得着」）。
读到的被判文件此刻与开工快照 `research/prompts/merge-verify-slots-governance-r2-snapshot/judged-sha256.txt` 逐份对得上（量的：`sha256sum -c` 14 份全 OK）。没读禁读清单里的两样。
只判槽位一段、五处正文、五处登记行；`--unresolved-to-commit-gate` 一族、变异锚点核一族不判。第一轮攻方的角度不重攻。

## 读数口径

- 「量的」：我直接跑的只读命令（`grep`、`ls`、`cat`、`sha256sum`、`git show`、`git diff`、`awk`、`wc`、几段只读的 shell 循环与一小段只读 python），贴了命令或行号；`scripts/hosts/minimal-copy.py` 试拷过一次（122M，已删），为的是看副本里有没有看门狗自证要载入的文件。
- 「推的」：读代码推出来的；模型（下面）我没有跑，读数一律写「推的，待主 agent 跑」。
- 引文一律「原文」（`文件:行号`）；kb 条款本轮没有引。

## 模型与复跑命令

模型目录 `research/prompts/merge-verify-slots-governance-r2-sonnet-model/`，工具逐份 sha256（量的，`sha256sum` 现算）：

```
ef0115d1297c413af7eed56420fcdf7d809893232e4e9fb9dc9cbfc8c8557917  tools/progress_note_vs_selftest.py
3141bb424c37f5b74bf5e8219efbd7614ef60e702ea55a6fc3416dc12d8ff658  tools/registration_rows.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
74a7d3b84e961547100aa09b87c65237c33fb8da3fc1e8308ec0c47cf89d61c2  tools/rerun.sh
bf5767d28becdf432f7309a81ceae1e7f002ea75309a1a94a1e0808d18531337  tools/slot_nonrs_change.sh
```

`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 拷来只改 `rerun_steps` 的一份；三样读数：
- `slot_nonrs_change.sh`：真 `cargo build --offline` 编一个无依赖的小包（`include_str!` 读一份数据文件），按 `refresh_slot_snapshot` 同一组 rsync 参数拷进「槽」，量改数据文件、改 `Cargo.toml`、槽里修改时刻没跟上三种局面下 cargo 重不重编、`slot-changed-sources` 那句 awk 收不收、`lib_rebuilt_packages.py check` 认不认。
- `registration_rows.py`：只读，量登记行五处的锚串命中数、理由点名的东西在不在、理由里点名的对子对不对。
- `progress_note_vs_selftest.py`：在 `minimal-copy.py` 建的仓副本里把 `progress_note` 读进度文件的各支逐个弄坏，跑 `agent-watch.py --selftest`，量红不红（十项：基线、M1 至 M7、对照 C1 与 C2）。副本 122M（量的，试拷过一次已删），用完 `rerun.sh` 删。
探针的内存与时间：没量过（推的：小包编译与登记行读数几秒；看门狗自证十趟，每趟时长没量，`progress_note_vs_selftest.py` 每趟上限 1500 秒、超时只对自己起的那一组发信号）。

## 要主 agent 跑的命令

1. 模板自证（跑完看末行 `TASK_SERVER_RESULT` 的 `exit=0` 与「✓ 自检通过（查了 6 格）」）。起在草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-sonnet`：
`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-sonnet-model/tools/rerun.sh --selftest`
2. 模型整份复跑（起在草稿目录；看 `TASK_SERVER_RESULT` 的 `exit=0`，`task-server/task log <号> --all` 里 `name=slot_nonrs step=… verdict=DIFFERENT`、`name=registration_rows … verdict=DIFFERENT`、`progress_note_vs_selftest` 的 `DIFFERENT` 行各是几行；一行都没有算全部与我推的相同，有的以读数为准改判；`name=judged_files_unchanged ok_count=14` 是前提，不是 14 就是被判文件在我读完之后又改了）：
`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-snapshot /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r2-sonnet-model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r2-sonnet-model`
3. 登记行两格（仓根起；看 `TASK_SERVER_RESULT` 的 `exit=` 与点名「锚串……一处被判行都不命中」的行有没有，`exit=0` 且无该行算绿）：
`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 repository-copy-closure、memory-bounded-parallelism 两格; 范围=2 格; 为什么=核登记行五处改完之后锚串命中与理由' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check repository-copy-closure,memory-bounded-parallelism`
4. 准入与运行条件 lint（仓根起；在输出里找 `e203_admission_scan_cost_counters` 的行：四份模块应当 0 行，入口 `e203_admission_scan_cost_counters.py` 有没有被点名，见第 4 格 (e)）：
`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py /home/fy5090/code/singlefs`

## 一、第 5 格：第一轮判决逐条复核

判定三种：站得住 / 够不着（判决引的依据对不上它要证的那件事）/ 同样打中别的臂（判决的改法或说法对所有替代方案一样成立、没有哪个方案独自躲开）。每条后面写「什么现象会推翻它」。

| 判决条目（「二、判决」） | 辩方判定 | 证据与推翻条件 |
|---|---|---|
| 第 1 格：改过的包不重编而判绿——不成立（拷进槽对内容变了的文件写新、修改时刻是此刻，cargo 照修改时刻重编） | 站得住，带两个前提 | 拷法：「rsync --recursive --links --checksum --no-times --executability」（`scripts/gate/merge-and-verify-snapshot-lib.sh:276`），内容变了的整份重写、没变的不动。前提：这一槽的编译目录这一趟只有这一趟在用（槽锁）；写源文件与编译在同一台（槽只在本机占，见下「第二台」那一行）。模型 `slot_nonrs_change.sh` 的 data 与 manifest 两步用真 cargo 量：rsync 写 1 份、cargo 重编 1 次（推的，待主 agent 跑）。推翻：模型 `step=data-compile-lines got=0` 或 `step=manifest-compile-lines got=0`（rsync 写了、cargo 没重编）。 |
| 第 1 格：`slot-changed-sources` 只收 `.rs`，归 `scripts/gate/lib_rebuilt_packages.py` 本来的射程，这一轮不改 | 站得住；「不是槽位这一版带来的」只对一半；改法同样打中别的臂 | 一、把 awk 放宽成收全部写了的文件不顶用：awk 在「path ~ /\.rs$/ { print path }」（`scripts/gate/merge-and-verify-snapshot-lib.sh:284`），而核对那一端本来就丢掉非 `.rs`：「name = package_of(tree, relative) if relative.endswith(".rs") else None」（`scripts/gate/lib_rebuilt_packages.py:144`），丢掉的路径退 77「本次未判」（`scripts/gate/lib_rebuilt_packages.py:25`）。只有改 lib 才有用，而 lib 同时是补丁那一路与 `scripts/mutation/prove-red.sh` 的核对（`lib_rebuilt_packages.py` 文件头「调它的」），所以替代方案「在槽位一侧放宽」打不中，「在 lib 里放宽」同样打中补丁一路与证红，不是槽位独有的改法。二、为什么不致判绿：非 `.rs` 输入的新鲜与否由 cargo 自己的指纹判，数据文件、`Cargo.toml` 内容变了在槽里的修改时刻是此刻（上一行）；模型 data 与 manifest 两步量 cargo 重编（推的，待跑）。三、「不是槽位带来的」对一半：lib 文件头写「rustc 的 dep-info 只记它们」（`scripts/gate/lib_rebuilt_packages.py:15`）不准——`include_str!` 读的文件也进 dep-info，cargo 照它们的修改时刻判（模型 data 步重编 1 次，推的）；而一趟一份的编译目录是新建的、每个包必编，核对在那一种里多余，槽里「修改时刻那一层」先坏了（弄坏开关 `slot-copy-keeps-old-times` 就是这一层）时核对是唯一的保险，它只认 `.rs`：模型 stale-data 步（数据文件、产物是旧的、核对退 77）与 stale-rs 步（`.rs`、核对退 1）各量一次（推的，待跑）。结论：「不改」站得住，因为那一层坏了才出现；文件里补一句「槽位让核对成为修改时刻那一层之后的唯一保险」比说「本来如此」准。推翻：模型 `step=stale-rs-check-exit got=0`（`.rs` 也没被点名）。 |
| 第 1 格：NFS 上两台同时挑槽此刻走不到（第二台那一层一律给 `CARGO_TARGET_DIR`、从不占槽） | 站得住，但那句靠派发保证、不是脚本验的 | 「环境里 CARGO_TARGET_DIR=<这一趟的本地目录>/target、」（`scripts/hosts/multi-host-run.sh:64`）；`choose_slot` 一看到它就退：「if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:391`）。脚本不验自己在哪一台：挑槽的编译根取「这一台」：「this = addresses.resolve(path).this」（`scripts/gate/merge-and-verify.sh:1205`）。反例输入（推的）：在第二台上不经 `scripts/hosts/multi-host-run.sh`、环境里没有 `CARGO_TARGET_DIR`，直接起 `bash scripts/gate/merge-and-verify.sh --item …`——`choose_slot` 会在共享工作根的 `merge-and-verify/` 下挑槽、锁走 NFS、编译目录落第二台的编译根，文件头「只在本机占槽：第二台那一层」（`scripts/gate/merge-and-verify.sh:82`）就不真了。此刻合入后验证只由主 agent 经服务跑，没有人这样起，判决的「走不到」站得住；推翻：任何一次在第二台上不带 `CARGO_TARGET_DIR` 起这份脚本。 |
| 第 1 格：cgroup 杀掉的尝试留下的槽被抢错——不成立 | 站得住（推的） | 攻方腿量到的那次撞上限重来，尝试 1 在第二台、没占槽，没走到这一支，所以只有推的：锁随进程放；别的任务先占了那一槽，下一次尝试先挑「更早尝试记下的槽」，被占就挑别的槽、整份按内容拷，判不错。推翻：同一任务两次尝试都在本机、第一次占槽被杀、别的任务先占了那一槽，第二次的 `rebuilt.log` 出现「没重编」。 |
| 第 1 格：三槽留满之后照一趟一份跑而不报——成立，改法只报不收；攻方腿说 `slot.4u3rewvh`「被一趟作废了的验证占着」不对 | 站得住；改法「只报不收」是唯一不打中那一槽的方案 | 现读（量的）本机 `merge-and-verify/slot.4u3rewvh/`：`kept`、`.owner`、`.deliverables` 都在；`.deliverables` 写「repo 基线 判定键合入后验证第六趟的快照（十二份补丁），照用户「停这一趟，先合再跑崩溃用例」停在验证包那一步，第 2 步不走 GPU 的几条在它上面跑」。调度记录同一件事：「槽 `slot.4u3rewvh` 是它这一批合入后验证留着的快照」（`records/2026-11-18-合入后验证压盘.md:137`）。判决原来只靠这条转述，现在有槽里的产物为证。攻方腿说对的一半：这一趟确实被取消过（用户叫停）、留着确实没有期限（「留着没有期限，只由收尾的槽位汇总逐个报、不收」，`scripts/gate/merge-and-verify.sh:119`）；说错的一半：「作废」。同样打中别的臂：任何按天数、按「没人占」自动收留着的槽的改法都会收走这一槽，所以只报不收是不打中它的方案。推翻：槽里 `.deliverables` 没有「repo 基线」那一行、又没有人认领它。 |
| 第 1 格：占盘上限的读法与超限处理（三处坐实） | 本腿不复核 | 归攻方腿，也不在辩方腿的四条验收里。 |
| 第 2 格：规则与技能正文——五处里「同一棵树」缺判据要补，第 23 行措辞要改，旧规矩两处要改 | 站得住（缺陷当时都在）；改完的结果见第 3 格 | `git diff HEAD` 的旧文：规则第 65 行旧写「编译目录归这一趟：每一趟新建、跑完删」，技能旧写「被下一趟替掉的那一趟的快照」，`.claude/agent-common.md:37` 与 `.claude/main-agent.md:34` 旧写「不留给以后的趟或别的任务复用」且没有槽的例外（量的）。 |
| 第 3 格：登记行不成立，五处要改 | 站得住（缺陷当时都在）；改完之后三处仍有毛病，见第 4 格 | `.claude/gate.d/repository-copy-closure-exempt.tsv` 第 41 行旧理由指 `lib_function_units.py` 的 `build_unit_graph`、`.claude/preflight-exclude` 没有 e203 四份模块的行（`git show HEAD:.claude/preflight-exclude` 里 `e203` 命中 0，量的）、`scripts/experiment-apparatus/README.md:7` 旧文只列五个装置目录（转述第一轮正推腿报告，HEAD 里没有这一行，我没有旧文可核）。 |
| 第 4 格：用例在模块名换掉时报 `KeyError`、不静默——成立；M2、M4、M5、M6 照绿，「归 `scripts/agents/agent-watch.py --selftest` 罩」，不加 | 「照绿」站得住；「归自证罩」够不着；「不加」要另找理由 | 见下「第 4 格的展开」。 |
| 第 5 格：多机——判定成立；第二台从不占槽，只改说法；「只在本机占槽照实写，要不要让第二台也占槽交用户」 | 站得住（与上面「NFS 两台」同一条的边界） | 文件头照实写了：「只在本机占槽：第二台那一层（scripts/hosts/multi-host-run.sh 文件头 ⑤）一律给 CARGO_TARGET_DIR」（`scripts/gate/merge-and-verify.sh:82`）；那一行的说法「第二台那一层（scripts/hosts/multi-host-run.sh）给了编译目录 CARGO_TARGET_DIR（槽只在本机占）」（`scripts/gate/merge-and-verify-snapshot-lib.sh:394`）与「调用方给了编译目录」（`scripts/gate/merge-and-verify-snapshot-lib.sh:396`）分得开。调度记录里用户已定「只本机占槽」。同样打中别的臂：让第二台也占槽要同时改第二台那一层给编译目录的写法与过 NFS 上的锁，替代方案的代价都在这两处，说法照实写对哪个方案都成立。 |
| 「已经量的」两句是主 agent 正文写错（省盘没量到、66 处都在拆分前就有的文件上） | 站得住（未重核「省盘」）；「66 处」那一句此刻的状态见第 4 格 (e) | 此刻四份 e203 模块在 `.claude/preflight-exclude` 里各有一行（量的，第 164 至 167 行）。 |

### 第 4 格的展开：M2、M4、M5、M6 与「归 `--selftest` 罩」

第一轮判决写的是「M2、M4、M5、M6 四种改坏照绿，那几处归 `scripts/agents/agent-watch.py --selftest` 罩」。我对这一句的依据逐项读了自证里的格（读代码，推的，待主 agent 跑模型 `progress_note_vs_selftest.py`）：

- 自证里碰进度文件的断言有三处（另一处是 `selftest_scratch_mentions` 断言没有「草稿目录：」一行时写「取不到 progress.md」，与 M2 至 M6 无关）：造一份三行的进度文件（`scripts/agents/agent_watch_alerts.py:1325`），整点询问只断言末行字样在不在：「if inquiry is None or "第三格在跑，预计 40 分钟" not in inquiry[1]:」（`scripts/agents/agent_watch_alerts.py:1348`）；另一处断言只比告警名集合（`scripts/agents/agent_watch_alerts.py:1344`「if got != wanted:」）。「已消失或不可读」那句在自证里一格都没断言（`grep -n '已消失或不可读' scripts/agents/*.py scripts/agents/tests/*.py` 命中 `agent_watch_alerts.py` 的 179、184、501 三行（501 是会话记录那一句）与单元用例 `scripts/agents/tests/test_agents.py:297`，量的）。
- M2（第 184 行改字）与 M5（第 183 行 `except OSError` 改成 `except KeyError`）：只有「文件在 `readable_mtime` 之后、`open` 之前消失」才走到，自证与单元用例都造不出这个窗口——两处都罩不到。M5 的后果是 `OSError` 逃出 `progress_note`，不是句子不同。
- M4（`readable_mtime` 读不到时交 0.0）：往下 `open` 抛 `OSError`，走第 184 行，输出与原来逐字相同——等价改坏，没有任何测试能杀它，不是漏测。
- M6（取头不取尾）：自证造的文件恰好三行，而「PROGRESS_TAIL_LINES = 3」（`scripts/agents/agent_watch_alerts.py:64`），取头三行与取尾三行是同一批行；`agent-watch.py` 文件头写「AGENT_WATCH_BREAK=progressnote  progress_note() 告警里不附 progress.md 的末几行」（`scripts/agents/agent-watch.py:404`）红的是整句不附，不是取错行。所以 M6 在自证里也照绿——判决这一句够不着。
- 同样打中别的臂：任何「再补一条用例」的改法，对 M2、M5、M6 一样要补（M2、M5 要让 `open` 抛 `OSError`；M6 要进度文件多于三行），对 M4 谁都杀不掉。所以判决的结论「不加」可以按成本取舍，但理由要改成「M4 是等价改坏；M2、M5 只在读文件的竞态窗口里走到；M6 现在没有任何测试罩」，不是「归自证罩」。
- 推翻：模型里 M2、M4、M5、M6 任一项的结局是 RED（自证红了，说明我读漏了一条走到那几行的路），或 C1 对照是 GREEN（说明自证连末行字样都没读）。

## 二、第 3 格：五处正文与脚本现行行为、与 `rules-discipline.md` 对不对得上

依据：`.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」（怎么做、射程与例外、默认选项、指到别处；不写为什么、经过、旧值）、「3. 搬出来的东西直接删掉」、「7. 正文不许用位置指代，也不许自称」、「8. 正文不写某一个使用者的东西」。四个文件里规则文件是 `.claude/rules/test-script-discipline.md`（该文件铁律直接管），技能与两份定义「同规矩」（`rules-discipline.md` 第 7 行「agent 定义与 skill 正文同规矩」）。第 3 格总答：**不成立（一处）**——四处成立，`crash-library-totals.sh` 一处同一条消息里自相矛盾；另有一条同类旁生。

| 处 | 判定 | 与脚本现行行为 | 与 rules-discipline | 证据与推翻条件 |
|---|---|---|---|---|
| `.claude/rules/test-script-discipline.md:65` | 成立 | 对得上：规则「固定路径、每趟按内容拷进来的那一份算同一棵树」，文件头「固定路径、每趟按内容拷进来的一槽算一棵树」（`scripts/gate/merge-and-verify.sh:80-81`），拷法按内容比（`scripts/gate/merge-and-verify-snapshot-lib.sh:276`）；同一节下一条「拷完把内容变了的文件的修改时刻改到此刻」与槽的拷法同向（内容变了的重写、没变的不动） | 合：一句判据加例外，没有论证、日期、旧值，没有使用者名与位置指代（rules-lint 命中 0 是主 agent 量的，我没跑） | 小毛病：同一行「同一棵树先后几趟可以接着用它那一份增量编」里的「增量编」，在脚本文件头里是另一个意思——「增量编译（slot_incremental_off…）」指 `<档>/incremental` 缓存、槽位那一种已关掉（`scripts/gate/merge-and-verify.sh:127` 「所以槽位那一种关掉增量：export CARGO_INCREMENTAL=0」）。按规则字面读不矛盾（规则只是准许），但读文件头的人会以为槽位在做「增量编」。推翻：规则文件里有一条要求「必须增量编」。 |
| `.claude/skills/merge-verify/SKILL.md:22-23` | 成立（一处措辞） | 放槽命令与 `slot_release_command` 逐字同一条：技能写「`rm -rf -- <槽>/kept <槽>/.owner <槽>/.deliverables <槽>/repo/target`」，脚本是 `printf "rm -rf -- '%s/kept' '%s/.owner' '%s/.deliverables' '%s/repo/target'\n"`（`scripts/gate/merge-and-verify-snapshot-lib.sh:327`）；「路径在「拷好快照」那一行与日志目录里的 `slot`」：拷好快照那一行在 `scripts/gate/merge-and-verify.sh:2660`、日志目录的 `slot` 文件名 `SLOT_RECORD_NAME="slot"`（`scripts/gate/merge-and-verify.sh:1096`） | 合：怎么做与射程；唯一像旧写法的是「下一趟不替掉它」，它写的是现行行为的反面，没有旧值 | 措辞：「一槽一个编译目录接着上一趟增量编」（`.claude/skills/merge-verify/SKILL.md:22`）同上「增量编」问题；`scripts/gate/README.md:42` 有同一句「一槽一个编译目录接着增量编」（不在这五处里）。建议两处改成「接着上一趟的产物编」。推翻：脚本里有一支不关增量。 |
| `.claude/agent-common.md:37` 的例外 | 成立，不比槽宽 | 例外写「合入后验证的槽（快照与它的编译目录）例外，照 `scripts/gate/merge-and-verify.sh` 文件头「槽位」留给下一趟接着编、用完放槽」。槽 = `<工作根>/merge-and-verify/slot.<随机>/` 加这一台编译根下它那一份编译目录（文件头「槽位」一段，`scripts/gate/merge-and-verify.sh:80` 起）；一趟一份那一种（没占到槽）的快照与编译目录不在例外里：编译目录收尾自己删、日志目录里的 `repo/` 技能要求删（`.claude/skills/merge-verify/SKILL.md:23`「日志目录 `repo/` 里的删」）。槽的编译目录超 `slot-target-limit-gib` 脚本自己整个删，例外没有放宽到「不限量」 | 合：射程与例外、指到脚本文件头；括号点明「快照与它的编译目录」 | 推翻：存在合入后验证留下的、不在 `slot.<随机>/` 与它的编译目录里的东西被这条例外放过去。 |
| `.claude/main-agent.md:34` 的例外 | 成立，不比槽宽 | 同上，「合入后验证的槽例外，照 `scripts/gate/merge-and-verify.sh` 文件头「槽位」留给下一趟接着编、用完放槽」；这一条没有括号点明范围，字面是「槽」整个（含 `kept`、`.owner`、`repo/target`）——放槽命令会把后三样删掉，所以不比槽宽 | 合 | 小建议：与 `.claude/agent-common.md:37` 补成同一句括号，两份定义不各写各的。推翻：同上。 |
| `scripts/mutation/prove-red.sh:232` | 成立 | 证红自己「每一趟新建、跑完删」；调用方给了 `CARGO_TARGET_DIR` 的照它、不删（`scripts/mutation/prove-red.sh:233`「照它的、不删，归调用方」；`scripts/mutation/prove_red_core.py:274` 的 `new_trip_build_directory` 是没给时现建的那一个） | 合（脚本注释只写现状：「证红自己的做法」「只要不同的树不共用编译目录」，没有历史） | 旁生（不归这一处）：`scripts/mutation/prove_red_core.py:248` 把「这一趟自己专用的编译目录」归到规则「编译与算法都要快」名下，规则现在只写「不同的树不共用」；第一轮判决把 `prove_red_selftest.py` 三处划成「那份脚本自己的做法、照留」，这一处是引规则，口径与 `prove-red.sh:232` 不一致，可改成同一句。推翻：规则里写了「编译目录专用」。 |
| `scripts/crash/crash-library-totals.sh:35` 与 `:281` | 35、281 成立；同一条消息里的 280 不成立 | 35、281 与 `task-server/task_server_lib.py:287`「wait 末行不带 build=（编译目录归树、不归任务）」同一个说法；而紧挨着的第 280 行仍是「读库程序要编，编译目录归这一趟，不照仓根的 target 编」（`scripts/crash/crash-library-totals.sh:280`）。两行是同一次退 2 打出来的相邻两句，一句说归这一趟、一句说归树不归任务（反例输入：不设 `CARGO_TARGET_DIR` 起这份脚本，`scripts/crash/crash-library-totals.sh:475` 的 `build-directory-required` 那一格走的就是它，两行连着出现）。第一轮判决把 280 划成「照留」（「是那份脚本自己的做法」），但 `scripts/crash/crash-amplification.sh:746` 的同句话「编译目录归这一趟」说的是自己在运行目录下建一个，这份脚本不建，用的是调用方给的，「归这一趟」对它不真。 | 合（没有历史），但自相矛盾 | 改法建议：280 去掉「编译目录归这一趟，」，只留「不照仓根的 target 编」。推翻：`crash-library-totals.sh` 在没给 `CARGO_TARGET_DIR` 时自己建一份、跑完删（现在是退 2，不建）。 |

### 旁生一条（不归这五处，写给主 agent 判）

`.claude/agent-common.md:37` 同一行还写着「经服务跑的编译目录由服务给，交任务的主 agent 用完 `task-server/task clean <号>` 删，谁都不自己 `rm`」。服务那一端相反：「wait 末行不带 build=（编译目录归树、不归任务）」（`task-server/task_server_lib.py:287`），`task clean` 只在弄坏开关下才删组里的编译目录：「if lib.broken("clean-removes-build-slot", self.environment):」（`task-server/task_server_scheduler_starting.py:746`），自证也断言「那个编译目录照旧在」（`task-server/task_server_selftest_lifecycle.py:860`）。这是第一轮判决对 `crash-library-totals.sh:281` 已经判成旧写法的同一类（编译目录归树）。同类句还在 `.claude/skills/dispatch/SKILL.md:18` 与 `task-server/task` 用法（`task-server/task:68`「这条任务留下的编译目录」）。推的（读代码，没跑自证）；推翻：`task-server/task clean <号>` 在没有弄坏开关时删了组里那一个编译目录。

## 三、第 4 格：登记行五处改完之后锚串命中与理由

命中数是我直接跑只读循环量的（`grep -cF -- <锚串> <路径>`，每行一次），模型 `registration_rows.py` 复现同一批数（推的，待主 agent 跑）。第 4 格总答：**不成立（三处理由有毛病）**——锚串全部命中，毛病在理由。

| 处 | 锚串命中 | 理由成立吗 | 结论与推翻条件 |
|---|---|---|---|
| (a) `.claude/gate.d/repository-copy-closure-exempt.tsv:41` 理由改指 `scripts/gate/lib_function_units_unit_graph.py` 的 `build_unit_graph` | 1（`.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh` 里含锚串的行 1 行，第 84 行：「cp --preserve=timestamps "$STAGE_REPOSITORY/.claude/heavy-thread-default" "$build_home/workspace/.claude/"」） | 成立：函数在「def build_unit_graph(repository, source, thread_registry, lock_file, cache, directory, program):」（`scripts/gate/lib_function_units_unit_graph.py:106`），同样手抄 crate 与登记文件：「shutil.copytree(source, os.path.join(workspace, "crates", "singlefs-unit-graph")」（第 111 行）、「shutil.copy2(thread_registry, os.path.join(workspace, ".claude", "heavy-thread-default"))」（第 113 行）；表里的理由「与 scripts/gate/lib_function_units_unit_graph.py 的 build_unit_graph 一起改」（`.claude/gate.d/repository-copy-closure-exempt.tsv:41`）说的就是这两处同一份手抄 | 成立。推翻：`build_unit_graph` 不再手抄，或门禁 `repository-copy-closure` 格对那一行判红（待主 agent 跑命令 3）。 |
| (b) 同表第 44 行删 | 全表 45 个登记行（50 行含注释与空行），零命中的行 0 个；有 1 行命中 2 行（第 47 行 `scripts/experiment-apparatus/replay.sh` 的 `cp -- "research/results/$name" "$target"`，≥1 就算命中） | 成立（只能核结果）：现表第 44 行是别的行（`scripts/experiment-apparatus/e171-row-shard-selftest.sh` 的 `cp`），当时被删的那一行的锚串现表里没有，核不到「删的就是那一行」，只能核到「现在没有零命中的行」 | 成立。推翻：命令 3 里 `repository-copy-closure` 格仍点名「一处被判行都不命中」。 |
| (c) `.claude/memory-bounded-parallelism.tsv:10` 理由照代码改 | 入口 `.claude/gate.d/code-tooling.sh` 在表里恰好 1 行（表共 8 个登记行） | **部分不成立**：前半成立——`research-script-selftests` 格分批并行、大自证登记在 `SELFTEST_RUNNERS_MEMORY_BOUND`（「SELFTEST_RUNNERS_MEMORY_BOUND=(」，`.claude/gate.d/lib/code-tooling-selftest-cells.sh:211`）；后半把格与弄坏开关配错：表里写「break-switch-replay 与 stage-break-switch-replay 两格只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下按内存上限与每趟内存算同时跑几条」（`.claude/memory-bounded-parallelism.tsv:10`），代码里的对子是「break-switch-replay、selftest-cells-alone 两格只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下用它」（`.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:357`），`stage-break-switch-replay` 自己的开关是另一个：「if broken == "stage-replay-jobs-by-memory" and budget is not None and known_peaks:」（同文件第 1037 行）。三格三开关：`break-switch-replay`—`replay-jobs-by-memory`、`selftest-cells-alone`—`cells-alone-jobs-by-memory`、`stage-break-switch-replay`—`stage-replay-jobs-by-memory`；理由里少了 `selftest-cells-alone` 与 `stage-replay-jobs-by-memory`，把 `stage-break-switch-replay` 配给了别人的开关。另：同库自己写那一格「不按内存收，理由同 break-switch-replay 格」（同文件第 659 行）并说它不在登记表里，与这一行把它列进入口理由的写法也不一致。门禁判不出这一类（它只判入口在不在、登没登两次、理由够不够长、代码里有没有按内存定并发的写法），所以绿也不证明理由对。 | 理由改成三格三开关：「break-switch-replay、selftest-cells-alone、stage-break-switch-replay 三格只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory、stage-replay-jobs-by-memory 下按内存收并行」。推翻：模型 `registration_rows.py` 的 `item=reason-names-selftest-cells-alone-cell got=1`（理由里已经有了）。 |
| (d) `scripts/experiment-apparatus/README.md:7` 改成不逐个列 | 不适用（README 行，没有锚串） | **部分成立**：例子 `e139_tightened_floor/`、`e203_admission_scan_cost/` 都在（量的：`ls -d` 命中），「`main.rs` 是目标根」「`e203_admission_scan_cost_counters.py` 入口加同目录四份部分」「副本的输入键与变异检查读这五份合起来的全文」都对得上（`scripts/experiment-apparatus/e203_admission_scan_cost_counters_applying_edits_and_copying_the_tree.py:318`）；但给的列法列多了：「全部用 `ls -d scripts/experiment-apparatus/*/` 列」（`scripts/experiment-apparatus/README.md:7`）列出 48 个目录，其中有 `main.rs` 的 31 个，另 17 个是 Python 装置目录与 `__pycache__/`、`target/`、`long-run-lib/`、`heavy_threads_default/`（量的：`ls -d */ \| wc -l` 得 48，带 `main.rs` 的 `ls -d */main.rs` 得 31） | 第一轮判的「列举不全」已解，新毛病是命令列出的是超集；列法改成 `ls -d scripts/experiment-apparatus/*/main.rs`。推翻：只算分成多份 Rust 模块的装置，这 48 里每个目录都有 `main.rs`。 |
| (e) `.claude/preflight-exclude` 登 e203 四份模块（第 164 至 167 行） | 4 行各 1（量的：每份模块路径在该表里作为行首恰好 1 行；四份模块文件都在、`__main__` 各 0 处） | **理由的后半不成立**：四行都写「被 scripts/experiment-apparatus/e203_admission_scan_cost_counters.py import 的模块，没有自己的入口，不单独调；准入与运行条件由入口判」（`.claude/preflight-exclude:165`）。前半成立（四份都没有 `__main__`）；后半不成立：入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 自己既没有文件头的 `admission:` 与 `run-condition:` 注释行、也不调 `preflight`、`.claude/preflight-exclude` 里也没有它自己的一行（量的：入口文件头 30 行里 `# admission` 0 行，全文 `preflight` 0 处，表里行首等于入口路径 0 行），而 lint 的要求是「文件头的注释块里（第一行代码之前）至少一行 admission 与一行 run-condition」（`.claude/singlefs-ai-sop/scripts/preflight-lint.py:21`），`scripts/experiment-apparatus` 是登记目录（`.claude/preflight-dirs:2`）。这句话是照 `replay-*` 那八行抄的（那八份的入口 `replay.sh` 有 `# admission: always`，量的），对 e203 不真；入口那份在拆分之前的版本里就没有（`git show HEAD:scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 开头无 `admission`，量的），不是这一批带来的，也不是第一轮的判决要改的，但这一句写成了「由入口判」就把它掩过去了 | 去掉「；准入与运行条件由入口判」，留前半；入口自己没有准入与运行条件是另一笔存量（lint 在命令 4 里会点名它还是不点名，待主 agent 跑）。推翻：命令 4 的输出里入口没有被点名（那就是我读漏了它别处另有登记）。 |

## 四、判据格答案

| 格 | 答 | 一句话依据 |
|---|---|---|
| 第 5 格：第一轮判决够不够得着、有没有判错 | **不成立**（有两处够不着；其余站得住） | 站得住：第 1 格的「不重编而判绿不成立」、`slot-changed-sources` 只收 `.rs` 不改、NFS 两台走不到、cgroup 抢槽不成立、三槽留满只报不收与 `slot.4u3rewvh` 不是作废、第二台只改说法。够不着：第 4 格「M2、M4、M5、M6 归 `--selftest` 罩」（M6 没有任何测试罩，M4 是等价改坏，M2、M5 只在读文件的竞态窗口走到）；第 2 格把 `crash-library-totals.sh:280` 划成「照留」（与同条消息 281 相邻自相矛盾）。 |
| 第 3 格：五处正文对不对得上、合不合 `rules-discipline.md`、例外有没有比槽宽 | **不成立（一处）** | 成立四处：规则第 65 行、技能第 22 至 23 行、两份定义的例外（都不比槽宽）、`prove-red.sh:232`；不成立一处：`crash-library-totals.sh` 280 与 281 同一条消息说「归这一趟」又说「归树、不归任务」。三处小建议（「增量编」措辞、两份定义补同一句括号、`prove_red_core.py:248` 的引规则口径）与一条旁生（`.claude/agent-common.md:37` 的 `task clean` 句与服务相左）。 |
| 第 4 格：五处登记行锚串与理由 | **不成立（三处理由）** | 锚串全部命中（第 41 行 1 行命中、全表零命中行 0 个、四份 e203 模块各 1 行、表里 `code-tooling.sh` 恰好 1 行）；理由有毛病三处：`.claude/memory-bounded-parallelism.tsv:10` 格与弄坏开关配错、`.claude/preflight-exclude` 四行「准入与运行条件由入口判」对 e203 不真、`scripts/experiment-apparatus/README.md:7` 的列法列出 48 个目录（只有 31 个有 `main.rs`）。 |

### 打中的反例输入汇总（给主 agent 核；除标「量的」外都是推的，待命令 2、3、4）

1. 第 4 格：把 `agent_watch_alerts.py` 里 `[-PROGRESS_TAIL_LINES:]` 改成 `[:PROGRESS_TAIL_LINES]`，跑 `agent-watch.py --selftest`——预期照绿（自证造的进度文件恰好三行）。模型 `progress_note_vs_selftest.py` 的 M6 一行与对照 C1（取头两行，预期红）。
2. `crash-library-totals.sh`：不设 `CARGO_TARGET_DIR` 起它，退 2，标准错误里相邻两行一句「编译目录归这一趟」一句「编译目录归树、不归任务」（读 280、281 两行可见，量的）。
3. `.claude/memory-bounded-parallelism.tsv:10`：模型 `registration_rows.py` 的 `reason-names-selftest-cells-alone-cell`（预期 0）与 `reason-names-stage-replay-jobs-by-memory`（预期 0）。
4. `.claude/preflight-exclude:164-167`：命令 4 的输出里入口 `e203_admission_scan_cost_counters.py` 被点名（预期会点名）。
5. `scripts/experiment-apparatus/README.md:7`：模型 `apparatus-directories-from-that-ls`（预期 48）对 `apparatus-directories-with-main-rs`（预期 31）。
6. 槽位一侧（对第一轮判决的补充，不是打中）：在第二台上不带 `CARGO_TARGET_DIR` 直接起 `scripts/gate/merge-and-verify.sh`；模型 `slot_nonrs_change.sh` 的 `stale-data-check-exit`（预期 77）与 `stale-rs-check-exit`（预期 1）。

### 对第一轮判决「改法」的补充（不是打中）

- 槽位文件头与规则、技能、README 里的「增量编」与「关掉增量」同时在：建议统一说「接着上一趟的产物编」。
- 重编核对的说明「rustc 的 dep-info 只记它们」（指包里的 `.rs`，`scripts/gate/lib_rebuilt_packages.py:15`）与 cargo 记 `include_str!` 读的文件不符；改不改归 `lib_rebuilt_packages.py` 本来的射程，这一轮不动的判决站得住，但它的文件头在槽位上第一次真被依赖。

## 没做什么

- 没有往统一任务服务交任务，上面「要主 agent 跑的命令」四条和模型的复跑都没有跑；模型里所有读数写「推的，待主 agent 跑」，判决要引的数以主 agent 复跑为准。
- 没有改任何被判文件（只读），没有新建判定库。
- 没有复核第一轮第 1 格「占盘上限的读法与超限处理」「已经量的」里「省盘没量到」与 replace-batch 那两句（攻方腿的范围）；没有核「NFS 上两台的锁互斥」（走不到，没有第二台的挂载选项可读）。
- 没有判 `--unresolved-to-commit-gate` 一族与变异锚点核一族；没有读别的会话那一批换上的内容。
- 槽位一段的新开关与自证格（第一轮改法第 1 条「每条配自证格与弄坏开关」）本腿没有逐条核，那是攻方腿的第 1、2 格。
- 没有核 `scripts/gate/README.md:42` 之外的 README 行（只核了 `scripts/experiment-apparatus/README.md:7`）。
