# 同步合并后补回的本地腿两份定义：第一轮判决（2026-09-27）

被判：`.claude/agents/three-way-local-attack.md`、`.claude/agents/three-way-local-defense.md`。正文 `research/prompts/_sync-local-legs-r1-body.md`，背景材料 `research/prompts/_sync-local-legs-r1-background.md`，开工快照 `research/prompts/sync-local-legs-r1-snapshot/defs-sha256.txt`（腿开工 2026-09-27；腿交齐后 `sha256sum -c` 十个文件全 OK，没有被别的会话改过的）。用户弹窗定「走一轮三方」，这一轮只开一轮。

## 一、这一轮交了什么

| 腿 | 报告 | 格 |
|---|---|---|
| 云端正推（sonnet） | `research/prompts/sync-local-legs-r1-sonnet-output.md`（sha256 2d0b0c06…6b022） | 丙、丁 |
| 云端攻方（opus） | `research/prompts/sync-local-legs-r1-opus-output.md`（sha256 bca32404…1f79），模型目录 `research/prompts/sync-local-legs-r1-opus-model/` | 乙 |
| 本地攻方 | 提示 `research/prompts/sync-local-legs-r1-local-attack.md`，运行记录 `-local-attack-runlog.md`，干净样本 s4（`>|` 重跑那一份）与 s5，s1–s3 带损坏、void1 判红作废 | 甲 |
| 核查员 | `research/prompts/sync-local-legs-r1-verifier-output.md`（sha256 82d762c9…2ecb） | 全部腿 |

核查员：两份云端报告的引文全部对上（自动 31 处、手工 56 处），opus 模型目录 18 个文件复跑逐字节相同，本地腿核对表与运行记录独立重跑对上；唯一 ✗ 是 opus 报告第 108 行把配对的 `textC.txt`（310 字节）写成了 B，拼接内容本身核实无误，不影响判定。本地腿自报一处执行失误：第 6 次调用（s5）发出之后才发现已过定义第 6 步「连续五次调用」的上限，s5 照收。

## 二、跑前条款

正文没有写跑前条款，四句里「跑前条款给的改法」一问空着，照实记。

## 三、逐条判决（主 agent 逐条现查过）

| 格 | 腿的说法 | 现查 | 判 |
|---|---|---|---|
| 甲 | 本地腿两份干净样本都答：腿开跑之前判不出这一次请求会不会超过前台上限（只知道上限 900 秒，不知道实际多久），所以「前台跑、超过上限时改用 run_in_background」这一支判不出来；与 `.claude/rules/three-way-inference.md` 第 206 行「本地腿用 Bash 的 `run_in_background` 起」在 300 秒一格上要求相反（s5 的表；s4 的表在这一格自相矛盾，只采它第 2 题） | 第 206 行原文现查如腿所引；`ask-local.sh` 第 21 行 `TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"` | **打中**。两份样本对「判不出」一致；它们第 4 题推的「前台超时被杀、退 124」与观测不符（见乙-1：工具把命令挪进后台，不杀），那一题不采 |
| 乙-1 | 前台跑满 Bash 上限时工具不杀进程、挪进后台接着跑；照第 3 步字面再起一条就是两个进程写同一个 `s<n>`；模型 3 格里拼接或被盖掉的样本两道损坏闸都判绿、退出码 0 | 主 agent 这一会话里亲眼见过同一行为（doc-lint 那一次「was moved to the background」）；核查员另起一次探针复现；模型复跑逐字节相同 | **打中**，而且与甲同一个根：前台那一支本身 |
| 乙-2 | 通知里的退出码是整条命令的；命令后面接了 `; echo` 这类，退 5 会报成 0；`oov-check` 对空文件打绿 | 模型复跑相同 | **弱打中**：前提是腿在定义给的命令后面自己多写一段；补一句「重定向之后不接别的命令」「0 字节不算干净」 |
| 乙-3、乙-4、乙-5 | 看门狗不误报「结束本轮却不会醒」；60 格命令形状三道钩子全放行、7 格对照照拒；写范围闸够不着这两条腿 | 模型复跑相同 | 没打中（一次云端抽样，只作线索，不拿它支撑结论） |
| 丙 | 第 4 步「非 0 非 5」是补集写法，罩住 2、3、4、6 与头注释的 1；第 6 步只管退 5 的重跑循环；没有一种退出码走两条规定。缺一句：退 6 同样留空文件占号、不回收 | `ask-local.sh` 第 124–130 行现查：退 6 前 `save_void` | 一致；补那一句 |
| 丁 | oov-check 带提示文件、300 字截断与 `oov-check.py` 第 249、257 行对得上；「打满了」没有显式标记，只能数那一行是不是正好 300 字符；local-defense 第 21 行还写着改名前的短称「输入」 | `oov-check.py` 第 257 行 `[:300]`；`three-way-local-defense.md` 第 21 行现查确有「输入」 | 补「正好 300 个字符就算打满」；第 21 行改成「输入（主 agent 必须给）」 |

## 四、写回了什么

- `.claude/agents/three-way-local-attack.md` 第 3 步：删掉前台那一支，一律 Bash 的 `run_in_background` 起，写明不先前台试跑的理由、重定向之后不接别的命令。这一形态就是 `.claude/rules/three-way-inference.md` 第 206 行已提交的规则，同内容的远端形态在 `governance-defs-r1`–`r3` 的 G3 格攻过三轮；在这一轮里是被判回到它。
- 同一份第 4 步：退出码 0 而样本 0 字节不算干净；退 6 那一次的号占着不回收。第 5 步：生词那一行正好 300 字符就算打满。
- `.claude/agents/three-way-local-defense.md` 第 21 行：「输入」改成「输入（主 agent 必须给）」。

## 五、交用户的

| 项 | 状态 |
|---|---|
| 第 4 步「0 字节不算干净」「退 6 占号不回收」、第 5 步「正好 300 字符算打满」 | 腿提的收严，**被攻过零轮** |
| 只开了一轮（用户定），甲、乙-1 的「打中」各只在这一轮成立，没有三轮多数 | 照用户的定，写回的主体是回到已提交规则的形态 |

## 回看决策

不涉及决策：被判的是两份 agent 定义的工作流写法，没有推翻或确立 `.claude/kb/decisions.md` 里任何一条。
