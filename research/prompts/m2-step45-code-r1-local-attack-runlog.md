# 运行记录：m2-step45-code-r1 本地攻方腿

提示文件：`research/prompts/m2-step45-code-r1-local-attack.md`（英文，攻击面 X5、X6、X7、X9）。
核对表：`research/prompts/m2-step45-code-r1-local-attack-translation-audit.md`。
命令：`bash research/scripts/ask-local.sh research/prompts/m2-step45-code-r1-local-attack.md`（前台跑，加 `nice -n 19`，不用 `setsid`/`&`/`disown`）。

## 逐次尝试

| 次序 | 落到的文件 | 退出码 | 判定 | 字词损坏闸的原样输出 | `oov-check.py` 结果 |
|---|---|---|---|---|---|
| 1 | `-output-void1.md` | 5 | 作废 | 生词=3 拼接=2：`AllocationRecords`(=allocation+records)、`prematurelyinstead`(=prematurely+instead)、`reprocesses` | 未跑（闸已判红，作废） |
| 2 | `-output-s1.md`（第 1 次判红之后的重跑） | 0 | 干净 | — | 绿，生词=1（`remount's`，合法所有格，不算损坏） |
| 3 | `-output-void2.md` | 5 | 作废 | `corruption-check.py` 判红：粘连=1（未点名具体词，见原样输出） | 未跑 |
| 4 | `-output-void3.md` | 5 | 作废 | `corruption-check.py` 判红：实词自复读=2：`reclaimed \| reclaimed` | 未跑 |
| 5 | `-output-void4.md` | 5 | 作废 | 生词=3 拼接=1：`yieldingshowing`(=yielding+showing)、`PoolAllocator's`、`placement's` | 未跑 |
| 6 | `-output-void5.md` | 5 | 作废 | 生词=2 拼接=1：`AllocationRecords`(=allocation+records)、`discrepancies` | 未跑 |
| 7 | `-output-s2.md` | 0 | 干净 | — | 绿，生词=1（`checker's`，合法所有格，不算损坏） |

第 2 次与第 7 次判绿的样本都另跑了 `python3 research/scripts/oov-check.py <样本>` 复核（第 5 步要求：闸判绿也要单独跑一遍看生词表），两次列出的生词都是英文合法所有格（`remount's`、`checker's`），不是缺头的粘连词，判定维持「干净」。

## 干净样本清单

- `research/prompts/m2-step45-code-r1-local-attack-output-s1.md`：41 行 1145 词（`wc -lw`）。
- `research/prompts/m2-step45-code-r1-local-attack-output-s2.md`：41 行 887 词（`wc -lw`）。

`wc -lw` 复核（与闸内部计数口径可能不同，只作独立交叉核对）：

```
  41 1145 research/prompts/m2-step45-code-r1-local-attack-output-s1.md
  41  887 research/prompts/m2-step45-code-r1-local-attack-output-s2.md
```

## 作废副本词数（`wc -w`，仅存档，不算观测）

```
910 m2-step45-code-r1-local-attack-output-void1.md
719 m2-step45-code-r1-local-attack-output-void2.md
1308 m2-step45-code-r1-local-attack-output-void3.md
1063 m2-step45-code-r1-local-attack-output-void4.md
746 m2-step45-code-r1-local-attack-output-void5.md
```

## 小结

七次调用：五次退出码 5（字词损坏，作废，副本落在 `void1`–`void5`）、两次退出码 0（`s1`、`s2`，均另跑 `oov-check.py` 复核为绿）。达到「至少两份干净样本」的要求，停止重试。全程英文提示、前台跑、未用 `setsid`/`&`/`disown`，未设 `ASK_LOCAL_ALLOW_CORRUPT`。

不做的事：不解读、不总结、不采纳 `s1.md` / `s2.md` 里本地模型的具体答复内容；这两份答复打没打中 X5 / X6 / X7 / X9 里的哪一条，由主 agent 判。

## 历史版本

（无。这是这一轮唯一一版运行记录。）
