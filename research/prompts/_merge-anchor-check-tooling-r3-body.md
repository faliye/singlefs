# merge-anchor-check-tooling-r3：合入后验证「这一批补丁新弄断的变异锚点」核，治理文件轮第三轮正文（2026-11-21）

## 一、题面

攻第二轮判决 `research/prompts/merge-anchor-check-tooling-r2-main-verification.md`「三、改法」六条的修补，这一轮是最后一轮。

- 修补：工具员草稿 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r2-fix/`（报告 `result.md`；换上之前的原样在 `base/`、新版在 `new/`），换进主工作区五份：`scripts/gate/merge-and-verify.sh`、`scripts/gate/merge-and-verify-steps-lib.sh`、`scripts/gate/merge-and-verify-items-lib.sh`、`scripts/gate/merge-and-verify-selftest-lib.sh`、`scripts/gate/apply-writer-patch.py`。要点：合进主工作区那一段、真合之前多一条 `apply-writer-patch.py --dry-run <merge-patch>`，第一轮那条按文件比锚定行的核与开跑时另记的两份删掉；带 `--skip-apply` 不给补丁目录打「变异锚点核本次未核」；自证新加合入前 `--dry-run` 七格与 `--skip-apply` 三格、新开关四条；耗时注释写成范围；`apply-writer-patch.py` 自证样本树补上 `.claude/gate.d/stage-inputs.tsv`。
- 主 agent 量到的（观测）：头一次换上合入后验证四份之后自证红 7 格（自证驱动把树与补丁目录颠倒着传），当场退回；改好重换之后合入后验证自证 `exit=0`「✓ 自检通过（查了 192 种）」，`apply-writer-patch.py` 开关复跑「复跑了 52 条开关（1 份），自证红且点名的格与实际红的格逐格相同 52 条」「拒算 0」；`merge-and-verify.sh` 开关复跑在跑、还没结局。
- 另一个会话此前换上「槽位编译关掉增量」（`merge-and-verify-snapshot-lib.sh` 与另两份的槽位一段），不在这一题射程里。
- 方案按今天的实现来谈：这一题不碰 `crates/`，`crates/` 里没有相关实现。

## 二、要攻的地方

1. 合入前那一条 `--dry-run`：它在合入那一刻的主工作区上判，和接下来真合那一条之间主工作区再被改，会不会漏；它退非 0 时「合进主工作区」那一段打出的出路照着做能不能做成；它的 15 GiB、400–740 秒会不会让合入那一步撞内存上限被服务杀掉、杀掉之后出路是什么。
2. 删掉第一轮那条按文件比锚定行的核之后，有没有哪一形只有它拦得住、`--dry-run` 拦不住（例：别的会话在开跑之后到合入之前改了这一批改到的代码、又在合入之前改回）。
3. `--skip-apply` 三格与「本次未核」那一行：倒出的补丁之前的树对不对，三种情形判得分不分开。
4. 新开关四条与改了红格的六条：各红在点名那一格；`--dry-run` 七格里六格必拦、一格必放行，样本是不是真走到了 `apply()` 的锚点静态核那一步，不是红在前置的核上。
5. `apply-writer-patch.py` 自证样本树补的登记表：只放 `crates-mutation-replay.sh` 一行，够不够让 static-check 判到每一格要判的东西。

第一、二轮已攻过而没打中的角度不重复：判法复用 `static_check_report`、补丁之前的树摆法、多份补丁合成之后再核、核没做成一律判红、红了不打合入命令、RELEASE 去行号互相顶掉、日志全数与出路、两棵临时树里的登记表。

## 三、分工

| 腿 | 立场 |
|---|---|
| 云端正推（Sonnet） | 逐条核五份修补做的是不是第二轮判决「三、改法」说的；第二节五条各给结论 |
| 云端攻方（Opus） | 攻第二节五条：造修补之后才放过新弄断的行、或误拒、或核没做成却放行的输入；自证与新开关改回去会不会照样绿 |

## 四、交付要求

- 引规则写规则文件名加 `@` 小节标题或那份文件自己的行号；引脚本写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务、不起任何脚本：要量的，把样本与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；仓副本要建的也写成命令交主 agent；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
