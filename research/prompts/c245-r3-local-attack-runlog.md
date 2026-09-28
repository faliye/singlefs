# c245-r3 本地攻方腿运行记录

提示：`research/prompts/c245-r3-local-attack.md`；核对表：`research/prompts/c245-r3-local-attack-translation-audit.md`；草稿目录 `/tmp/claude-1000/c245-r3-local/`（未用到，本轮没有落草稿）。

5b：主 agent 没给答案表路径，算术没比。

## 样本

| 样本 | 路径 | 退出码 | 词数 | oov-check | corruption-check | 判定 |
|---|---|---|---|---|---|---|
| s1 | `research/prompts/c245-r3-local-output-s1.md` | 0 | 93 | 绿，生词=0 拼接=0 | 灰（语料太小 cjk=0 words=76，判不了；ask-local.sh 内置闸已过、退出码 0） | 干净，通读一遍未见缺词、断句、粘连词 |
| s2 | `research/prompts/c245-r3-local-output-s2.md` | 0 | 119 | 绿，生词=1（dependence，真英文词，词表外真词误判） 拼接=0 | 绿，cjk=0 words=107 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0 | 干净，通读一遍未见缺词、断句、粘连词 |

两次调用即拿到两份干净样本，没有作废副本、没有带损坏样本。

## 交回

干净 2 份，参考 0 份：
- `research/prompts/c245-r3-local-output-s1.md`
- `research/prompts/c245-r3-local-output-s2.md`

核对表：`research/prompts/c245-r3-local-attack-translation-audit.md`
