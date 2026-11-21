# merge-anchor-check-tooling-r1：合入后验证补「这一批补丁新弄断的变异锚点」核，治理文件轮第一轮正文（2026-11-20）

## 一、题面

攻换进主工作区的合入后验证改动：`scripts/gate/merge-and-verify.sh`、`scripts/gate/merge-and-verify-steps-lib.sh`、`scripts/gate/merge-and-verify-items-lib.sh`、`scripts/gate/merge-and-verify-selftest-lib.sh` 四份。工具员报告 `~/.local/share/singlefs/work/merge-anchor-static-check/result.md`，换上之前的原样在同一草稿目录（退回规格 `merge-anchor-check-revert-spec.json` 点名的备份）。

- 要关的缺口（主 agent 量到的）：代码轮第二轮修补经合入后验证第 1、2 步都绿、经 `apply-writer-patch.py <日志目录>/merge-patch` 合进主工作区，`crates/mutations.tsv` 里 5 行别批变异的原文在合入补丁的 `baseline/` 里各命中 1 次、合进去之后 0 次。原因：`scripts/gate/apply-writer-patch.py` 文件头「变异锚点静态核」那一段（`static_check_report`，拿没打补丁的树当基线、只判新出现的红）只在 `--dry-run` 那一支走（`apply_planned` 第 1007–1011 行），合入后验证打补丁与真合都不带 `--dry-run`；合完补交的 code-source-discipline mutation-tables 格此刻因为 crates 拆分留下的一千来行旧红整格红，判不出这一批新加的。
- 改法：`merge-and-verify.sh` 在快照上算好合进主工作区那一份补丁目录之后调新函数 `anchor_check_new_breaks`（`scripts/gate/merge-and-verify-steps-lib.sh`），它起内联程序 `MERGE_ANCHOR_CHECK_PROGRAM`：摆一棵「补丁之前的树」（快照的拷贝范围、补丁动到的文件换回 `merge-baseline` 里的原样、补丁新建的删掉），调 `apply-writer-patch.py` 的 `static_check_report` 比补丁前后，只把新红判红；新红记一项、这一趟退 1、不打合进主工作区的命令，逐行点名并给出路；核自己没做成也判红。带 `--skip-apply` 的、合起来没有改动的不做。
- 主 agent 量到的（观测）：换上用 `scripts/gate/hook-swap.py` 试换、真换都 `exit=0`；自证 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest` `type=succeeded exit=0`「✓ 自检通过（查了 167 种）」；换上之后头一趟真用（代码轮第三轮修补的合入后验证）「✓ 变异锚点核过：这一批补丁没有让变异行新红」「补丁前就红的 1004 处」，那一批修好了第二轮弄断的 5 行。两条新弄坏开关 `anchor-check-skipped`、`anchor-check-baseline-ignored` 的证红（`code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`）在跑。
- 方案按今天的实现来谈：读的是主工作区里换上之后的四份与 `scripts/gate/apply-writer-patch.py`；这一题不碰 `crates/`，`crates/` 里没有相关实现。

## 二、要攻的地方

1. 判法对不对：「补丁之前的树」摆得对不对（拷贝范围、换回原样、删新建、只留落在范围里的补丁段），会不会把补丁前就红的算成新红、或把新红漏成旧红；多份补丁目录合成一份之后才核，合成那一步会不会盖掉某份补丁弄断的行。
2. 走不到与放过：带 `--skip-apply`、合起来没有改动、样本树之外的自证跑法、只改变异表不改源码的补丁，这几条路上该不该核、核了没有；核没做成（拷贝范围算不成、static-check 判不了、内存不够被杀）是不是一律判红而不是放行。
3. 出路：判红时打出的出路照着做能不能做成；红了之后「合进主工作区」那一段确实不打。
4. 自证与开关：两格样本（必红、必绿）分不分得出这道核在不在、基线用没用；两条开关各红在点名那一格；`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」——复用 `static_check_report` 没有另写第二份判法。
5. 代价：每趟合入后验证多两趟 static-check（工具员引 `apply-writer-patch.py` 注释：各约 16 GiB、半分钟到一分钟，没在这一道里量），会不会让合入后验证撞内存上限或拖得过长。

## 三、分工

| 腿 | 立场 |
|---|---|
| 云端正推（Sonnet） | 逐条核四份改动做的是不是第一节说的、能不能拦住第一节量到的那一形；第二节五条各给结论 |
| 云端攻方（Opus） | 造这道核放过新弄断的行、或把旧红判成新红、或核没做成却放行的输入；自证与开关改回去会不会照样绿 |

## 四、交付要求

- 引规则写规则文件名加 `@` 小节标题或那份文件自己的行号；引脚本写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务、不起任何脚本：要量的，把样本与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
