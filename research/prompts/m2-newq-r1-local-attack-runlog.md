# m2-newq-r1 本地攻方运行记录

提示文件：research/prompts/m2-newq-r1-local-attack.md（sha256 c2a4c656ea76eaa4beb285a420a0df0fada2e6bcca4f65503eaf3a782b2d6f9c）
核对表：research/prompts/m2-newq-r1-local-attack-translation-audit.md（sha256 2ca57123e23cb50be3fbf6b34e5cd54dc75422bcbbb36460e89ff9b4bd9bdb3e）

调用记录（前台跑，未用 setsid / & / disown）：

| 样本 | 命令 | 退出码 | 行数 | 词数（wc -w） | oov-check 生词（去重） | corruption-check | 判定 |
|---|---|---|---|---|---|---|---|
| s1 | `nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-newq-r1-local-attack.md > research/prompts/m2-newq-r1-local-attack-output-s1.md` | 0 | 13 | 282 | clarifies | 绿（cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0） | 干净 |
| s2 | `nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-newq-r1-local-attack.md > research/prompts/m2-newq-r1-local-attack-output-s2.md` | 0 | 101 | 5476 | published'、falsified（合计出现 15 次，去重 2 个；两个都是词表外的真英文词/带标点的正常词，非拼接） | 绿（同上，全 0） | 干净 |

两份样本 sha256：
- s1: 990d2a78a27e716f19f942aeec785c631db7b2a0fdcfc13b7c615ebffa2b9fdf
- s2: 3b3679ad53a03fcd440a65f8399317356074bb370e1b7690f9e59c4b9932d481

无判红作废、无 void 副本（`ls research/prompts/ | grep m2-newq-r1-local-attack` 不含 `-void` 文件）。第二次调用即达到两份干净样本，未用满五次调用上限。两次调用之外未做任何解读、未采纳答复、未判打没打中。
