# unreadable-at-mount-r1 本地攻方：运行记录（2026-09-28）

- 提示：`research/prompts/unreadable-at-mount-r1-local-attack.md`（149 行，sha256 `24609d7a8de8ce30f2e6d7302e08d262996fde58e825e56acbfc0f6bad2c16b0`），攻击面是正文「W4　结局网格」。
- 转述核对表：`research/prompts/unreadable-at-mount-r1-local-attack-translation-audit.md`（sha256 `a54c9188312847190572a225295da0b28c02a4984c0520db531deb6894cc93a3`）。
- 调用：`nice -n 19 bash research/scripts/ask-local.sh <提示> > <前缀>-output-s<n>.md`，经 Bash 的 run_in_background 起，退出码取完成通知里的。一共调了 2 次，两次都是退出码 0，没有作废副本（`ls research/prompts/unreadable-at-mount-r1-local-attack-output-void*` 无匹配）。

## 每份样本

| 样本 | 退出码 | 词数（`wc -w`） | 生词清单（oov-check） | 判定 |
|---|---|---|---|---|
| `research/prompts/unreadable-at-mount-r1-local-attack-output-s1.md`（282 行，sha256 `a975f1b9f77844a1fd13f0d840eaecde3916fdc6a63b0dc0fd07493044468c65`） | 0 | 2451 | 无（生词=0，拼接=0；生词表没打满 300 字符） | 干净 |
| `research/prompts/unreadable-at-mount-r1-local-attack-output-s2.md`（322 行，sha256 `de2a0c0fafb7de2aa6ad65aabeeec877aa785dbf073000aa0371dd6676505cc7`） | 0 | 1523 | 无（生词=0，拼接=0；生词表没打满 300 字符） | 干净 |

检测器原样输出（两条命令都带提示文件）：

```
绿 research/prompts/unreadable-at-mount-r1-local-attack-output-s1.md  生词=0 拼接=0
绿 research/prompts/unreadable-at-mount-r1-local-attack-output-s1.md  cjk=0 words=2316 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0
绿 research/prompts/unreadable-at-mount-r1-local-attack-output-s2.md  生词=0 拼接=0
绿 research/prompts/unreadable-at-mount-r1-local-attack-output-s2.md  cjk=0 words=1503 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0
```

- 通读：两份逐行读过，没见缺头的粘连词、列表里整词缺失只剩标点、断句这类损坏。
- 行号：两份样本里没有自给的代码行号或文件行号（`grep -niE '\bline[s]? [0-9]|:[0-9]+\b'` 两份都无命中），没有要标「模型自给、未核」的。
- 算术：主 agent 没给答案表，算术没比。

## 交回

- 干净 2 份：s1、s2（路径见上表）。参考样本 0 份。

## 没做什么

- 没解读、没总结本地模型的答复，没比两份样本的方向；判它们打没打中归主 agent。
- 没跑代码，没跑门禁阶段。
- 没读禁读清单里的文件（`research/prompts/unreadable-at-mount-r1-opus-*`、`unreadable-at-mount-r1-sonnet-*`）。
