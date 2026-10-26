# apparatus-rename-r1：目录搬迁的换算读法与它写进定义、规则的那几句，会不会放过真改动、与别处读法对不对得上（推论轮第一轮，治理轮）正文（2026-10-23）

## 一、题面

用户 2026-10-23 定把 `scripts/experiments/` 改名为 `scripts/experiment-apparatus/`，并在弹窗里定「先补门禁再改名」「改到的定义走一轮三方」（调度记录 `records/2026-10-23-实验装置目录改名.md`）。为了让改名不被门禁当成新增或改动，这一批加了一套「登记过的目录搬迁」换算：登记表 `.claude/gate.d/path-moves.tsv`，读法集中在 `.claude/gate.d/lib-lag-baseline.py`，几处门禁与装置调它。这一轮判这套换算本身，以及写进定义与规则的那几句。

被判的改动（开工快照 `research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt`，对 HEAD 的 diff 在同目录 `judged-vs-head.diff`；diff 里另有别的会话同期的改动，下面逐条点名的才是这一批的）：

1. `.claude/gate.d/lib-lag-baseline.py`：文件头「搬迁登记」一段；`read_path_moves`、`path_moves`、`moved_prefix_pattern`、`rewrite_moved_paths`、`rewrite_moved_bytes`、`moved_path`、`paths_before_moves`、`baseline_text` 里先换前缀与按搬迁之前的路径取、`baseline_file_bytes`、`path_move_problems`、`moved_only_paths`、`drop_moved_lines`，以及跳过 `<提交哈希>:` 后面路径的那一支；自证里新加的各项与弄坏开关 `path-moves-ignored`、`moved-only-kept`、`commit-ref-rewritten`。
2. `.claude/gate.d/path-moves.tsv`：登记表与它的一行。
3. `.claude/hooks/lib_long_run.py`：`exempt-check --base` 里 sha256 不同的行，按搬迁之前的路径取基准那一版、换算之后与此刻逐字节相同就放行。
4. `scripts/gate/changed-paths.sh`：改动范围剔掉与基准相比只差登记搬迁的文件，`gate_added_lines` 剔掉只因搬迁才变的行；弄坏开关 `LIB_CHANGED_PATHS_BREAK=moved-kept`。
5. `.claude/gate.d/doc-decisions.sh` 的 entry-added（按文件剔）与 shape（按行剔）、`.claude/gate.d/doc-experiments.sh` 的 quoted-result-lines 历史节（按行剔）、`.claude/gate.d/code-tooling.sh` 的 change-range-single-source 第 ④ 条（判登记表）与 long-run-header-exempt 的 moved 三份样本。
6. E171 装置 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py` 的 `pinned_sha256_now`（从最近一次搬迁起逐次多倒换一次，每一步都比钉值）、两个重钉的值 `ORCHESTRATOR_SHA256`、`SEGMENT_TWO_ORCHESTRATOR_SHA256`；同目录 `frozen_o3.py` 与 `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py` 的 `RegisteredMoves`、`freeze_differences` 里换回去再比；`fake_multi_host_run.py` 给伪造的第二台链登记表与读法库。
7. 定义与规则：`.claude/agent-common.md`、`.claude/main-agent.md` 长活豁免表那一句里「同一路径 sha256 变了也算涨，判红；」之后加的「登记在 `.claude/gate.d/path-moves.tsv` 的目录搬迁照登记换算，基准那一版换算之后与此刻逐字节相同的不算涨；」；`.claude/rules/path-moves.md`「怎么做」第 2 步加的登记一句、「门禁管哪一半」加的一句。

要回答：这套换算会不会把真改动放过去；几处读法是不是同一份、边界规则一不一致；写进定义与规则的那几句与脚本做的事对不对得上。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区现查）

- `crates/` 下与这套换算无关：`grep -rn 'path-moves\|path_moves\|rewrite_moved' crates/` 零命中。改名在 `crates/` 里改到的是注释、文档注释与 checker-tier 装置二进制的提示字符串里的路径；与 HEAD 相比只差路径的有 8 份（`crates/singlefs-checker-tier/src/bin/` 下 4 份、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-harness/` 下 2 份、`crates/singlefs-unit-graph/src/heavy_threads.rs`），产品代码只有 `inode_tree.rs` 一处 `///`，门禁 doc-process-records 的 crates-adversarial-review 格现跑没有把它列进要三方点名的文件。
- 改名的全仓文本替换用的是会话草稿里的一份脚本，前缀边界照 `lib-lag-baseline.py` 的 `moved_prefix_pattern`；它另把拆成 `"scripts", "experiments"` 的写法、前面紧挨字符串转义 `\n` `\t`、shell 默认值 `:-`、diff 行首 `-` `+` 的写法也换了，而 `moved_prefix_pattern` 不认这几种（边界规则在 `lib-lag-baseline.py` 的 `PATH_BEFORE_BOUNDARY`、`PATH_AFTER_BOUNDARY`）。例：`scripts/experiment-apparatus/e10-aging-compaction-run.sh` 与 HEAD 那一版相比只差路径，改动范围照旧把它算成改了。
- 仓里有意留旧路径的：kb 变更史三语里 `git show 31e47115:scripts/experiments/…` 6 处；讲这次改名本身的几行（调度记录、登记表、`research/prompts/subagent-landing-audit-2026-10-22/b11-guards.md` 第 297 行等）。
- 现跑结果（主工作区，2026-10-23）：`lib-lag-baseline.py --selftest` 22 项都过；`changed-paths.sh --selftest` 19 项通过；`lib_long_run.py exempt-check . --base HEAD` 报 6 行「照搬迁换算之后与基准那一版逐字节相同」；E171 `reuse_library.py --selftest` 43 项红 0；门禁 doc-experiments 的 decision-links 格「没回看」161 → 152（少的 9 页与 HEAD 相比只差路径）；doc-decisions 的 history-loss 格只剩 E156 一条（别的会话的改动）。
- 批 1、2b、2c 三个工具员的报告：`~/.local/share/singlefs/work/path-move-baseline/result.md`、`~/.local/share/singlefs/work/path-move-pins/result.md`、`~/.local/share/singlefs/work/path-move-change-range/result.md`（腿可以读，判的是代码，不是报告的结论句）。

## 三、格

| 格 | 问什么 |
|---|---|
| G1 误放 | 换算之后「逐字节相同就不算改动 / 不算涨 / 认钉值」：有没有一种真改动经换算变成逐字节相同。尤其：登记表谁都能加一行，判法只核新前缀在、旧前缀不在、出处在——拿一行登记把别的改动（例：把一条命令里的目录 A 换成另一个已存在的目录 B）洗成「只差搬迁」做不做得到；倒换时此刻内容里本来就有的新前缀串会被换走，会不会让两份不同的文件换出同一份 |
| G2 读法一致 | 几处读法（`lib-lag-baseline.py` 两个方向的换算、`moved_only_paths`、`drop_moved_lines`、`lib_long_run.py`、`frozen_o3.py` 两份、`reuse_library.py` 的逐次倒换、改名用的替换）边界规则、提交引用的跳法、多次搬迁串起来的次序是不是同一套；哪一处与别处不一致会让同一份内容一处判同、一处判不同 |
| G3 改动范围剔除的副作用 | `changed-paths.sh` 剔掉只差搬迁的文件之后，依赖改动范围的格（show-me-test、crates-adversarial-review、agent-def-adversarial-review、knowledge-sync、batch-scope、term-renames、trilingual 几格等）有没有哪一格本该看见这类文件（新路径要核存在、搬家本身要同步记录）而现在看不见 |
| G4 定义与规则的句子 | 第 7 条那几句与脚本做的事逐字比：「不算涨」的条件写全没有（按搬迁之前的路径取基准、逐字节相同）；`path-moves.md` 第 2 步要不要写清「整串替换打不到的写法（拆开写的目录名、相对路径 `../experiments`、rsync 排除写法）要另搜」；有没有写进理由与来历（`.claude/singlefs-ai-sop/rules/rules-discipline.md`「1. 正文只写四样」） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | G1–G4 | 从脚本源码与规则正推：每一处换算做的是不是登记表与规则说的、几处是不是同一套 |
| 云端攻方（Opus） | G1–G4 | 找反例：造一个经换算被放过的真改动、一处判同一处判不同的内容、剔除之后漏判的格；能在草稿目录里拷脚本造样本实测的就实测 |

方案按 `crates/` 与脚本今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
