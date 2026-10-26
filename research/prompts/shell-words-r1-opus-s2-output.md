# shell-words-r1 攻方腿重派（Opus，s2）报告：没做完（2026-10-26）

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

装置搭好、冒烟跑完、开始逐段读切词代码的时候，这条腿的回复被安全分类器拦下。照它的要求，被拦的那部分内容不再生成，这条腿停在这里交回主 agent。这是这一轮攻方腿第二次在同一处被拦（前一条见 `research/prompts/shell-words-r1-opus-output.md`）。

## 停之前做了的

- 读了共用约束、背景材料、前一条攻方腿的报告、own.diff 全文。
- 冻结副本四份钩子的 sha256 与快照清单 `research/prompts/shell-words-r1-snapshot/files-sha256.txt` 逐份相同，与主工作区四份 `cmp` 相同。
- 在草稿目录建了新版、旧版两份仓副本（各 103M，交回前已删）；旧版把 own.diff 反向打回去，heavy-test-guard.sh 的文件头注释与 `KNOWN_GUARD_BREAK_ITEMS` 两段没打回去（别的会话的改动挪了上下文），这两段不影响判定。
- 草稿目录 `tools/feed.py` 写成批量喂法（一份 jsonl 里的命令逐条喂两版，打出两边退出码）。冒烟三条（这一次真跑，与前一条腿的冒烟结果相同）：

| 喂哪份钩子 | 谁 | 命令 | 新版 | 旧版 |
|---|---|---|---|---|
| heavy-test-guard.sh | 主 agent，不带前缀 | `time -p cargo test --all` | rc=2 | rc=0 |
| heavy-test-guard.sh | 主 agent | `ls` | rc=0 | rc=0 |
| bash-command-detector.sh | clerk | `bash scripts/hosts/detached-run.sh start --label m -- sleep 1` | rc=2 | rc=2 |

第一行是改动想补的写法，新版拒合乎改动的意图，不是错判样本。

## 没打中的形状

没有。没试任何错判写法，取样范围为零。

## 这条腿自己的限度

- 六格都没判，攻方这一条腿第二次缺席。同一处连续两次被拦，再派同一种「找错判样本」的攻方腿大概率还停在这里（推的）；要不要换做法（例如由主 agent 或工具员按钩子文件头的判据直接补自证格、或只派正推腿逐族核判对的依据）由主 agent 定。
- 没建模型目录，没有 rerun.sh 与 sha256。

## 没做什么

没跑重型测试、cargo 与自证；没改冻结副本与主工作区；草稿目录里留着 `tools/feed.py`、`tools/smoke.jsonl`、`tools/reject.rej`（反向打补丁没打上的两段）与 `smoke.tsv`，没有仓副本。
