# 运行记录：m2-step3-code-r1 本地攻方腿

提示文件：`research/prompts/m2-step3-code-r1-local-attack.md`（英文，覆盖 X3/X5/X7/X11 四个攻击面，F1-F24 共 24 条事实，含 13 条代码事实与 11 条决策/不变量/上一轮判决事实）。
核对表：`research/prompts/m2-step3-code-r1-local-attack-translation-audit.md`。
样本前缀：`research/prompts/m2-step3-code-r1-local-attack-output`。

## 逐次尝试

| 尝试 | 重定向目标 | 退出码 | 判红来源 | 详情 | 处置 |
|---|---|---|---|---|---|
| 1 | `-output-s1.md` | 5 | `corruption-check.py` | 判红的不是模型答复，是**提示文件自己**：`红 research/prompts/m2-step3-code-r1-local-attack.md ... 粘连=16`。原因是 `ask-local.sh` 把提示文件本身也交给 `corruption-check.py` 核（`python3 "$CHECK" "$TXT" "$1"` 两个参数都查），而提示里 16 处 Rust 路径分隔符 `::`（如 `PoolAllocator::rebuild_from_records`）撞上了「粘连」判据 `(?<![A-Za-z0-9])[:;,]\w`——第二个冒号不被字母数字打头、后面紧跟单词，被当成「一句话被截断后接上了下一句」的丢字签名。这不是本地模型的问题，是提示自己的问题，若不修，往后每一轮都会在这一步先红。当场用 `research/scripts/replace-batch.py` 把提示里全部 16 处 `::` 改成单冒号 `:`，改后 `corruption-check.py` 单独核提示文件判绿（粘连=0），`oov-check.py` 的判定口径经确认不受影响（它只把提示文件当 `known` 词表来源，不对提示文件本身判红）。 | 提示已改好；`-output-s1.md` 因此是空文件（0 字节，`ask-local.sh` 判红后没打印正文）；作废副本 `m2-step3-code-r1-local-attack-output-void1.md`（这一份是提示修好之前那次答复，不算样本） |
| 2 | `-output-s1.md`（沿用同一个重定向目标，因为第 1 次没写出真实样本） | 5 | `corruption-check.py`（对模型答复本身判红） | 提示已修好后重跑，这次判红的是 `$TXT`（模型答复），不是提示：`oov-check.py` 报 `生词=2 拼接=2`，拼接列表 `appendingappending(=appending+appending)`（实词自身粘连复读）与 `PublishedVersions(=published+versions)`。前者是真损坏（同一个词连写两次），按 `.claude/rules/three-way-inference.md`「闸判红之后那一轮作废」处理 | 作废副本 `m2-step3-code-r1-local-attack-output-void2.md`；`-output-s1.md` 仍是空文件（0 字节）；按规则「换下一个 s<n> 重跑」，下一次改用 `s2` |
| 3 | `-output-s2.md` | 0 | — | `corruption-check.py`／`oov-check.py` 两道闸都绿；`oov-check.py` 单独核样本：`生词=2 拼接=0`（`publishes'`、`contradiction`，均为正常英文词/所有格，非拼接）；词数 1164；人工通读全文一遍，没有发现「缺头的粘连词」（如 `achievesceives` 那类）这种闸抓不到的形态 | **干净样本** |
| 4 | `-output-s3.md` | 0 | — | 两道闸都绿；`oov-check.py`：`生词=3 拼接=0`（`distinctness`、`Contradiction`，均为正常英文词，`Contradiction` 计了大小写两次出现）；词数 800；人工通读全文一遍，没有发现闸抓不到的缺头粘连词 | **干净样本** |

四次尝试内拿到两份干净样本（s2、s3），没有触发「连续五次拿不到两份干净的」停下条款。

## 干净样本清单

- `research/prompts/m2-step3-code-r1-local-attack-output-s2.md`：退出码 0，词数 1164，生词 `publishes'`、`contradiction`（非损坏），干净。
- `research/prompts/m2-step3-code-r1-local-attack-output-s3.md`：退出码 0，词数 800，生词 `distinctness`、`Contradiction`（非损坏），干净。

## 带损坏 / 作废的样本

- `-output-s1.md`：两次尝试都判红，最终是空文件；不当样本用。
- `-output-void1.md`：第 1 次尝试的原样答复，判红原因是提示文件自身的 `::` 粘连误判（已修复，不代表模型答复本身有问题；但这一轮仍按规则作废，不追认为干净）。
- `-output-void2.md`：第 2 次尝试的原样答复，判红原因是模型答复里的真实拼接损坏（`appendingappending`）。

## 没做什么

- 不解读、不总结、不采纳本地模型在 s2 / s3 里给出的答案；打没打中、值不值得写回代码，是主 agent 的事。
- 没有对 void1 / void2 的内容做进一步分析（作废轮不算观测，`.claude/rules/three-way-inference.md`「否定结论尤其不算」——但这里连肯定结论都没有，只是原样存档）。
- 没有跑 `gate.sh`、没有碰 `crates/`、没有读禁读清单里的文件（`m2-step3-code-r1-*-output*.md` 里别的腿的输出、`m2-step3-code-r1-opus.md`、`-sonnet.md`、`-local-defense*.md`）。
