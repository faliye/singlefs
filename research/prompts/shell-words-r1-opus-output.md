# shell-words-r1 攻方腿（Opus）报告：没做完（2026-10-26）

## 各格判定一览

| 格 | 判定 |
|---|---|
| W1 花括号展开的射程 | 没核成 |
| W2 heredoc 进没收尾的替换 | 没核成 |
| W3 detached-run 写法的新认法 | 没核成 |
| W4 env 宽松赋值的收紧 | 没核成 |
| W5 time、builtin、function、bash 选项与 `//` 路径 | 没核成 |
| W6 语料证据够不够 | 没核成 |

六格都是「没核成」，不是「没打中」，不能拿来支撑新认法站得住。

## 为什么停

装置搭好、冒烟跑完之后，这条腿的回复被安全分类器拦下，输出没有送达。照它的要求，被拦的那部分内容不再生成，这条腿停在这里交回主 agent。没有造反例，也没有建模型目录，所以没有复跑命令与 sha256。

## 停之前做了的

- 读了派发提示要求先读的材料与 own.diff 全文。
- 核过主工作区四份被判文件与冻结副本逐字节相同（`cmp` 四份都相同）。
- 在草稿目录建了两份仓副本，一份换上冻结钩子（新版），一份把 own.diff 反向打回去（旧版；heavy-test-guard.sh 里文件头注释与 KNOWN_GUARD_BREAK_ITEMS 元组两段因别的会话的改动没打回去，不影响判定）。
- 喂法：草稿目录 `tools/feed.py` 把同一条命令按 hook 输入 JSON 喂给新旧两份副本的钩子，打印退出码与 stderr。冒烟三条（这一次真跑）：

| 喂哪份钩子 | 谁 | 命令 | 新版 | 旧版 |
|---|---|---|---|---|
| heavy-test-guard.sh | 主 agent，不带前缀 | `time -p cargo test --all` | rc=2 | rc=0 |
| bash-command-detector.sh | clerk | `bash scripts/hosts/detached-run.sh start --label m -- sleep 1` | rc=2 | rc=2 |
| heavy-test-guard.sh | 主 agent | `ls` | rc=0 | rc=0 |

第一行是改动想补的写法，新版拒合乎改动的意图，不是反例。

## 没打中的形状

没有。没试任何攻击写法，取样范围为零。

## 这条腿自己的限度

- 六格都没判，攻方这一条腿等于缺席；要不要重派由主 agent 定。
- 两份仓副本交回前删了（repo 173M、repo-orig 173M、orig-hooks 1.1M）。

## 没做什么

没跑重型测试、cargo 与自证；没改冻结副本与主工作区；没建模型目录，没有 rerun.sh。
