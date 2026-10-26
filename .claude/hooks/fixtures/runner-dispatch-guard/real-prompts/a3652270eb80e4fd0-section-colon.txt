# 派发：three-way-attack

重型测试：不跑
批次：records/2026-10-22-子agent结果落地审计.md#4
线程上限：31
禁读清单：无
草稿目录：/home/fy5090/.local/share/singlefs/work/landing-audit-r1-opus
认领：bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/landing-audit-r1-opus --purpose '三方 landing-audit-r1 的腿草稿（landing-audit-r1-opus）'
背景材料：research/prompts/_landing-audit-r1-background.md
攻击面：正文 research/prompts/_landing-audit-r1-body.md 节「三、格」的 K2（能不能合并、和已有脚本有没有重合）、K3（有没有矛盾）、K4（数与读数能不能核）
预计挂钟：2 小时内，读正文、约 30 份被判文件、gate-overlap 列表与全仓 grep，复算几个数，不编译

回复与报告用中文。

轮名：landing-audit-r1。前几轮的判决：无（第一轮）。照正文「交付」：报告写进 research/prompts/landing-audit-r1-opus-output.md。模型目录：research/prompts/landing-audit-r1-opus-model/。

## 关的是哪几条

用户点名要看的三面，加一面数：

1. **K2 合并与重合**：这一批新加或改动的格、判法、子命令、登记表（正文节「一、题面」那张表），每一样与已有的门禁、钩子、脚本比：管不管同一件事、同一批对象、同一个触发点；能不能并进已有的那一份；有没有整段相同或共用逻辑没抽成库。现查 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 并引它的输出；每一处能并或重合的，写出并成什么样、并了之后哪一处判法要删。
2. **K3 矛盾**：新写的句子、判法与已有的规则、定义、钩子、技能逐对比（正文 K3 列的几对是起点，不限于它们）。另有一处主 agent 派之前看到、请你判：`python3 scripts/agents/dispatch-prompt.py three-way-forward` 的骨架「报告：」一行默认填 `<草稿目录>/result.md`，而三方腿的报告要写进 `research/prompts/<轮>-*-output.md`（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；这是不是这一批改骨架带出来的矛盾，别的类型的骨架有没有同样的问题。全仓仍要求或按名字认 `report.md` 的地方用 `grep -rn` 数全，不截断。
3. **K4 数**：正文 K4 列的数，照调度记录写的命令在副本上复出来，或指出哪个复不出来、为什么；复算放草稿目录，经 `scripts/hosts/run-with-memory-cap.sh` 跑。

每一条打中的，写出：是哪两处（文件:行）、为什么算重合或矛盾、改法、它对不对得上调度记录里那条缺口（改了之后缺口还关不关得住）。没打中的格写「没打中」与你查了什么。

## 写法

- 方案按 `crates/` 与 `scripts/` 今天的实现来谈；被判的改动在 `research/prompts/landing-audit-r1-snapshot/judged-vs-index.diff`，混有别的会话的未暂存改动，按正文与调度记录点名的改法认这一批的。
- 报告分段写进报告路径：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建（`set -o noclobber`），之后 `>>` 追加；回复只写短句：报告路径、sha256、各格打中几条。
- 草稿与临时文件放草稿目录，不放会话共用的暂存目录。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 每条打中写推翻条件。

## 出口

K2、K3、K4 逐格写完，报告写完交回。