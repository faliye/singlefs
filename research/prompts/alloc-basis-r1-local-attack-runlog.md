# 本地攻方运行记录：alloc-basis-r1

提示文件：`research/prompts/alloc-basis-r1-local-attack.md`（英文，自足；覆盖 Y2、Y5 两组判据，不碰 Y1/Y3/Y4/Y6/Y7）。
核对表：`research/prompts/alloc-basis-r1-local-attack-translation-audit.md`（16 条转述逐句核对）。
草稿目录：`/tmp/claude-1000/alloc-basis-r1-local-attack/`（本轮未用到，产物直接落地）。

两次前台调用（未用 setsid / 后台 / disown），均命中同一个网关 `http://127.0.0.1:8200/v1/chat/completions`。

## 样本 1：`alloc-basis-r1-local-attack-output-s1.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r1-local-attack.md > research/prompts/alloc-basis-r1-local-attack-output-s1.md`
- 退出码：0
- 词数：`wc -w` = 462；`corruption-check.py` 自报 words=462
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=0 拼接=0，退出码 0（含计划第十一节说的「缺头粘连词」一类，本样本 0 命中）
- 判定：干净
- 内容形态（如实记录，不判打没打中）：24 行 Y2 答案（A/B/C × Serial/Merged × 要求 1/2）+ 12 行 Y5 答案（A/B/C × Scenario1/2，each answer 含 I-3.1 与 I-5.2），每条都带一句「This would be refuted by」；未按提示要求为回答「no」的 Y2 组合构造出带设备数、逐次发布分配/释放、F 值、环里最旧有效根 txg、df 读数的具体可达历史，只给了抽象的反驳条件句；Y5 部分同样未给出具体的镜像状态（哪些落点、哪些根槽持有什么内容）。

## 样本 2：`alloc-basis-r1-local-attack-output-s2.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/alloc-basis-r1-local-attack.md > research/prompts/alloc-basis-r1-local-attack-output-s2.md`
- 退出码：0
- 词数：`wc -w` = 459；`corruption-check.py` 自报 words=495（两个计数器口径不同，均如实记录）
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=0 拼接=0，退出码 0
- 判定：干净
- 内容形态（如实记录）：与样本 1 同形（24 行 Y2 + 12 行 Y5，每条带反驳句），同样未构造具体历史/具体镜像状态。

## 两份样本之间的差异（如实记录，不解读、不裁决）

- Y2.A.Merged.1：样本 1 答 no；样本 2 答 Yes。
- Y2.B.Merged.2：样本 1 答 yes；样本 2 答 No。
- Y2.C.Merged.2：样本 1 答 no；样本 2 答 Yes。
- Y5.A.Scenario1 / Y5.A.Scenario2：样本 1 答「I-3.1 violated, I-5.2 holds」；样本 2 答「I-3.1 violated, I-5.2 violated」。
- Y5.B.Scenario1 / Y5.B.Scenario2：样本 1 答「I-3.1 violated, I-5.2 holds」；样本 2 答「I-3.1 holds, I-5.2 holds」。
- 其余各格两份样本一致。
- 按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」：以上差异是本地模型两次采样的观测记录，不据此下结论，也不当作「不一致因此都不算」处理——两份都已判定为字词层面的干净样本，交主 agent 判断。

## 作废副本

无。两次调用均未触发字词损坏闸，`ask-local.sh` 没有生成任何 `-output-void*.md`（已现查 `research/prompts/` 目录，`alloc-basis-r1-local-attack` 前缀下只有 `-output-s1.md` 与 `-output-s2.md` 两个产物文件）。

## 样本数

达到「至少两份干净样本」的要求，两次调用（s1、s2）全部判绿，未消耗任何重试额度，未触及「连续五次拿不到两份干净的」这条停下条件。
