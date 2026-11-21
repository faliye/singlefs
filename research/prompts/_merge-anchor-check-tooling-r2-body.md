# merge-anchor-check-tooling-r2：合入后验证「这一批补丁新弄断的变异锚点」核，治理文件轮第二轮正文（2026-11-21）

## 一、题面

攻第一轮判决 `research/prompts/merge-anchor-check-tooling-r1-main-verification.md`「三、改法」六条的修补，并派辩方腿复核第一轮判决。

- 修补：工具员草稿 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-fix/`（报告 `result.md` 第一到第八节；换上之前的原样在 `base/`、新版在 `new/`），换进主工作区六份：`scripts/gate/merge-and-verify.sh`、`scripts/gate/merge-and-verify-steps-lib.sh`、`scripts/gate/merge-and-verify-items-lib.sh`、`scripts/gate/merge-and-verify-selftest-lib.sh`、`scripts/gate/apply-writer-patch.py`、`scripts/gate/apply_writer_patch_plan.py`。另外照管变异键那一件的会话的要求，`apply_writer_patch_plan.py` 的 `STATIC_CHECK_DECLARED` 与 `STATIC_CHECK_TREE_PATHS` 都加进 `.claude/gate.d/stage-inputs.tsv`（它那边把 `scripts/mutation/` 下 static-check 改成读被判那棵树里的登记表，正在换）。
- 换上之后另一个会话又对着这一版换了「槽位编译关掉增量」，改了 `merge-and-verify.sh`、`merge-and-verify-selftest-lib.sh`、`merge-and-verify-snapshot-lib.sh` 的槽位一段，不在这一题射程里；这一轮攻的是主工作区此刻的样子（开工快照照此刻记）。
- 主 agent 量到的（观测）：头一次换上之后合入后验证自证红一格（「开跑那一刻的指纹」那一格 `IndexError`），当场退回；病根在自证那一格照旧写法 `record <补丁目录>` 起指纹程序，修好重换之后 `apply-writer-patch.py --selftest` `exit=0`、合入后验证自证 `exit=0`「✓ 自检通过（查了 181 种）」。两份脚本的弄坏开关复跑：`apply-writer-patch.py` 那一条在跑；`merge-and-verify.sh` 那一条由改槽位那一件的会话那一趟一起复跑，结局还没出。
- 方案按今天的实现来谈：这一题不碰 `crates/`，`crates/` 里没有相关实现。

## 二、要攻的地方

1. 改法第 1 条（开跑那一刻另记锚在这一批改到的文件上的变异行、合之前重算）：记的范围对不对（这一批改到的文件怎么算、别的会话追加的行锚在同一文件别处的算不算）、合之前那一条核在主工作区被并发改时会不会误拒或漏放。
2. 改法第 2 条（`RELEASE` 去行号再比）：去掉行号之后同一文件两行 RELEASE 原因相同时会不会互相顶掉，让新红被旧红抵掉。
3. 改法第 3 条（`--skip-apply` 那一趟倒出补丁之前的树照样核）：倒不回、补丁目录与主工作区已有别的改动叠在一起时，是不是一律判红、不放行。
4. 改法第 4、5 条：日志全数、出路三处照着做能不能做成；自证新格与十条新开关分不分得出各自那一处（三处改坏现在登成了开关）。
5. 管变异键那一件换上之后：合入后验证的锚点核与 `apply-writer-patch.py --dry-run` 的锚点静态核，补丁前后两棵临时树里都有被判那棵树的登记表吗；补丁改了登记表时两棵树里各是改前改后那一份吗。

第一轮已攻过而没打中的角度不重复：判法复用 `static_check_report`、补丁之前的树摆法、多份补丁合成一份之后再核、核没做成一律判红、红了不打合进主工作区的命令。

## 三、分工

| 腿 | 立场 |
|---|---|
| 云端辩方（Sonnet） | 复核第一轮判决：采的七条够不够得着、不采与「不归这一题」的判得对不对；再逐条核六条修补做的是不是判决说的 |
| 云端攻方（Opus） | 攻第二节五条：造修补之后才放过新弄断的行、或误拒、或核没做成却放行的输入；自证与新开关改回去会不会照样绿 |

## 四、交付要求

- 引规则写规则文件名加 `@` 小节标题或那份文件自己的行号；引脚本写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务、不起任何脚本：要量的，把样本与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；仓副本要建的也写成命令交主 agent；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
