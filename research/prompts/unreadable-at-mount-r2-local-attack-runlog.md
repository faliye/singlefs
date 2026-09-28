# unreadable-at-mount-r2 本地攻方：运行记录（2026-09-28）

- 提示：`research/prompts/unreadable-at-mount-r2-local-attack.md`（105 行，sha256 `0f5c4a4caa2a618f2271925382c5c9aec4ad459189aabddd0601af8a2cd0224e`），攻击面是正文「Y4　轨迹网格」。
- 转述核对表：`research/prompts/unreadable-at-mount-r2-local-attack-translation-audit.md`（sha256 `dc3fcd9651dc4b927dd611c5258cdb77cec1b15ad71d0ca915fdd2d4150ca4d2`）。
- 调用：`nice -n 19 bash research/scripts/ask-local.sh <提示> > <前缀>-output-s<n>.md`，经 Bash 的 run_in_background 起，退出码取完成通知里的。一共调了 3 次：第 1 次退出码 5（作废副本 void1，那次重定向建出的 s1 是 0 字节，确认之后用 `>|` 重跑进 s1）；第 2 次退出码 0（s1）；第 3 次退出码 0（s2）。

## 每份样本

| 样本 | 退出码 | 词数（`wc -w`） | 生词清单（oov-check，带提示文件） | 判定 |
|---|---|---|---|---|
| `research/prompts/unreadable-at-mount-r2-local-attack-output-void1.md`（50 行，sha256 `fc2dd01f17f4a25b81e6c1ba545e4ac97fc21c9c7eb812fdf1767f9a9f44bcd5`） | 5 | 1412 | 无（生词=0，拼接=0；生词表没打满 300 字符） | 作废，参考样本：corruption-check 判红「单字母替换=1」，点名 `reuses`×6（提示里是 `reused`），出现在样本第 31、33、35、41、43、45 行 |
| `research/prompts/unreadable-at-mount-r2-local-attack-output-s1.md`（53 行，sha256 `65a840d0d703092671047bfc558d585e2a0d9fa770a7dca464f666365035a00d`） | 0 | 3005 | 无（生词=0，拼接=0；没打满） | 干净 |
| `research/prompts/unreadable-at-mount-r2-local-attack-output-s2.md`（177 行，sha256 `15051a9254b6268f09c180bfc06db2833b98ea3352e207f92ed24a5c842b154e`） | 0 | 1733 | 无（生词=0，拼接=0；没打满） | 干净 |

检测器原样输出（两条命令都带提示文件；void1 是对作废副本另跑的）：

```
红 research/prompts/unreadable-at-mount-r2-local-attack-output-void1.md  cjk=0 words=1202 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=1
     单字母替换: reuses×6（提示里是 reused）
绿 research/prompts/unreadable-at-mount-r2-local-attack-output-void1.md  生词=0 拼接=0
绿 research/prompts/unreadable-at-mount-r2-local-attack-output-s1.md  生词=0 拼接=0
绿 research/prompts/unreadable-at-mount-r2-local-attack-output-s1.md  cjk=0 words=2759 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0
绿 research/prompts/unreadable-at-mount-r2-local-attack-output-s2.md  生词=0 拼接=0
绿 research/prompts/unreadable-at-mount-r2-local-attack-output-s2.md  cjk=0 words=1574 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0
```

- 通读：s1、s2 逐行读过，没见缺头的粘连词、列表里整词缺失只剩标点、断句。两处照实记、判为措辞不是损坏：s2 通篇把「mount 1」写成「mount1」（每处一致，不是缺头粘连）；s2 第 118、153 行「config slots have unreadable or fail self-verify」语法不通，两行逐字相同，按措辞记，不按缺词记。主 agent 若按缺词判，s2 改记带损坏、只剩 s1 一份干净。
- 格式：s1 每格写成一行（没按提示分行），s2 按提示分行；只是排版，不影响判定。
- 行号：三份样本里没有自给的代码行号或文件行号（`grep -niE '\bline[s]? [0-9]|:[0-9]+\b'` 三份都无命中），没有要标「模型自给、未核」的。
- 算术：主 agent 没给答案表，算术没比。

## 交回

- 干净 2 份：s1、s2。参考样本 1 份：void1（作废副本）。路径见上表。

## 没做什么

- 没解读、没总结本地模型的答复，没比几份样本的方向；判它们打没打中归主 agent。
- 没跑代码，没跑门禁阶段。
- 没读禁读清单里的文件（`research/prompts/unreadable-at-mount-r2-opus-*`、`unreadable-at-mount-r2-sonnet-*`）。读了第一轮攻方模型目录 `research/prompts/unreadable-at-mount-r1-opus-model/`（正文第二节点名、不在禁读清单里），用来取撕裂三形的定义、今天与甲在撕裂三行上的单次结果、(a) 在两行账读不出上的单次结果。
