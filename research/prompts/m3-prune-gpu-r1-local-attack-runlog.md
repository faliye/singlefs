# 本地攻方运行记录：m3-prune-gpu-r1

提示文件：`research/prompts/m3-prune-gpu-r1-local-attack.md`（英文，自足，无 markdown 强调；QUESTION 0 到 QUESTION 4 共覆盖 U3、U5、Q1、Q3、G1、G2、S1 的容量与吞吐算术，与云端攻方 Opus 的精确性攻击面不重叠——提示 SCOPE 一节显式排除「剪得对不对」这一题，只留算术）。
核对表：`research/prompts/m3-prune-gpu-r1-local-attack-translation-audit.md`（表一 6 条转述/引号句逐句核对，表二给提示里每个 FACT 编号的来源文件:行；另发现并现场修正一处笔误：FACT 0F.5 首稿误写「60 named operations」，核对时用 `research/scripts/replace-once.py` 定点改成「61」，已在正确版本上取样，见核对表末尾说明）。
主 agent 未给算好的答案表路径；本轮没有答案表，算术没比（5b 一节）。

## 样本编号与调用

四次调用，均未用 setsid / 后台常驻 / disown（`run_in_background` 起、结束本轮等完成通知）：

1. `bash research/scripts/ask-local.sh research/prompts/m3-prune-gpu-r1-local-attack.md > research/prompts/m3-prune-gpu-r1-local-attack-output-s1.md` → 退出码 5
2. `bash research/scripts/ask-local.sh research/prompts/m3-prune-gpu-r1-local-attack.md >| research/prompts/m3-prune-gpu-r1-local-attack-output-s1.md` → 退出码 5
3. `bash research/scripts/ask-local.sh research/prompts/m3-prune-gpu-r1-local-attack.md >| research/prompts/m3-prune-gpu-r1-local-attack-output-s1.md` → 退出码 0（干净）
4. `bash research/scripts/ask-local.sh research/prompts/m3-prune-gpu-r1-local-attack.md > research/prompts/m3-prune-gpu-r1-local-attack-output-s2.md` → 退出码 0（干净）

## 作废副本 1：`m3-prune-gpu-r1-local-attack-output-void1.md`（参考样本，判红）

- 对应调用 1；重定向建出的 `s1` 先确认大小为 0 字节，已核实。
- `python3 research/scripts/corruption-check.py`：绿（`cjk=0 words=618 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`），退出码 0。
- `python3 research/scripts/oov-check.py`（带提示文件）：红（`生词=2 拼接=1`；`拼接: microsecondsus(=microseconds+us)`；`生词: overcount microsecondsus`），退出码 1。
- 判红原样行：损坏在第 17 行，词 `microsecondsus`（`microseconds` 与 `us` 粘连，缺头类拼接损坏的姐妹形态——这里是整词粘连，非缺头）。
- 判定：带损坏，参考样本；损坏处所在的那一句（4.1 那一句的单位数字）不用，其余内容不当结论依据。

## 作废副本 2：`m3-prune-gpu-r1-local-attack-output-void2.md`（参考样本，判红）

- 对应调用 2；重定向建出的 `s1` 先确认大小为 0 字节，已核实。
- `python3 research/scripts/corruption-check.py`：红（`实词自复读=1`；`实词自复读: dominates`），退出码 1。
- `python3 research/scripts/oov-check.py`（带提示文件）：绿（`生词=1 拼接=0`；`生词: influential`），退出码 0。
- 判红原样行：损坏在第 2 行，「FACT 0D.2's contribution dominates dominates 2^300 dominates all other states...」——`dominates` 连续三次出现（实词自复读）。
- 判定：带损坏，参考样本；损坏处所在的那一句（QUESTION 0 结论句）不用，其余内容不当结论依据。

## 样本 1（干净）：`m3-prune-gpu-r1-local-attack-output-s1.md`

- 对应调用 3；退出码 0。
- 词数：`wc -w` = 1217。
- `python3 research/scripts/oov-check.py`（带提示文件）：绿，生词=1（`overcount`，未打满 300 字符），拼接=0，退出码 0。
- 通读结果：QUESTION 0、0-PRIME、1（六行 FACT 1B.1–1B.6）、2（五行 FACT 2A.1–2A.5 加批数）、3（四个段长）、4（五行 FACT 4.1–4.5）全部作答齐全；每题自带的「This would be refuted by」句齐全；未见缺词、断句、孤零标点或粘连词。
- 判定：干净。

## 样本 2（干净）：`m3-prune-gpu-r1-local-attack-output-s2.md`

- 对应调用 4；退出码 0。
- 词数：`wc -w` = 1007。
- `python3 research/scripts/oov-check.py`（带提示文件）：绿，生词=1（`overcount`，未打满 300 字符），拼接=0，退出码 0。
- 通读结果：同样全部作答齐全（QUESTION 0、0-PRIME、1、2、3、4 逐项）；未见缺词、断句、孤零标点或粘连词。
- 判定：干净。

## 样本数与停止条件

四次调用（两次判红作废、两次干净）取得两份干净样本（样本 1、样本 2），达到「至少两份干净样本」的下限，未触发「连续五次调用拿不到两份干净就停」。已停止，不再调用 `ask-local.sh`。
