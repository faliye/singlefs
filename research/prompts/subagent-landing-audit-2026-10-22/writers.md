# 子 agent 落地审计：writers 组 32 个的抽查（2026-10-22）

批次：`records/2026-10-22-子agent结果落地审计.md#1`。输入 `~/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/inputs/1-writers.json`（landed 22 个、partial 10 个）。

## 结论

1. 五种判定：进了，现在还在 **26**；进了，后来被改写 **6**；没进，被另一份改动取代 **0**；没进，丢了 **0**；判不了 **0**。32 个都是逐个查了会话、主会话打补丁的命令和 git 历史判的，没有整组放行。
2. 机械落地率的假阳性很小。机械落地率减「实质改动的逐字落地率」，32 个平均 **+4.4 点**（中位 +1.2 点）；把读进来的 Bash 输出也算作原有内容的宽口径下平均 **+5.1 点**。landed 22 个平均只偏高 **+1.3 点**（宽口径 +1.4 点）。偏差集中在两个：a0ea839d9 **+68 点**（机械层算进的只有一份不进补丁的草稿探针，它真改的代码是用 Bash 写的，机械层没算到），af006c499 **+24 点**（宽口径 +30 点，整份 Write 重写已有文件、原文是用 Bash `cat` 读的）。除去 a0ea839d9，平均 +2.4 点。
3. 假阴性比假阳性大。机械层判「没找到」、其实在仓里只是改过形的行（子串命中或相似度 ≥0.75），平均占算进行数的 **8.6%**；partial 10 个里有 3 个（a901b3b2b、a76c7422a、a4486895b）算上改过形的行，实质落地率在 86%–92%。
4. 对「机械落地率 ≥70% 就算进了」这条判法：**可信，偏高约 1 点**。机械 ≥70% 的 22 个，实质逐字落地率全部 ≥70%（71%–100%），22 个都确实进了。但有两条限制：
   - 它说的是「进过」，不是「现在还在」：git 历史里出现过也算找到。ad6ea00bd 机械 98%，实质行现在还在工作树里的是 **0%**（`judging_closure.rs` 10-11 入库，10-18 整份删掉）。
   - 反过来不成立：机械 30%–70% 的 10 个全部进了（5 个后来被改写，5 个现在还在）。低于 70% 说明的是「后来被改得多」，不是「没进」。
5. 机械层还有范围上的漏算，比落地率的偏差影响更大：6 个子 agent 的主要产出不在它算的 Edit / Write 里（草稿目录里写好再拼进仓、或在 Bash 里用 python 改），机械层只看到一小部分。见「机械层没算到的产出」一节。

推翻条件：
- 有人找到这 26 个「现在还在」里任何一个，表里列的那行在当前工作树里不存在，或 `git log -S` 给出的提交里没有它，第 1 条就要改。
- 拿同一批会话换一种「原有行」判法（比如照子 agent 开工时主工作区的未提交状态逐文件重建底座），算出的偏差均值超过 +10 点，第 2、4 条要改。
- 机械 ≥70% 的子 agent 里出现一个主会话没打它补丁、内容也不在历史与工作树里的，第 4 条要改。

## 口径与做法

- 「算进的行」照机械层原样重算：成功的 Edit / Write，路径能映射到仓内路径的；Edit 取 new_string 里不在 old_string 里的行，Write 取全文；去首尾空白、不短于 12 字；同一工具调用只算一次。重算出的「算进 / 找到」与输入里 32 条的 `lines_written` / `lines_found_in_repo_or_history` 逐个相等（脚本 `analyze.py` 的输出，见草稿 `analyze-summary.txt`）。
- 算进的行逐行分成三类：
  - 原有行：子 agent 开工前同一文件里已有的行。底座口径认两样：开工前最后一次提交（`git rev-list -1 --before=<开工> HEAD`）里这份文件的行，与它开工后 Read 读到的这份文件的行。含 Bash 所见口径另认：命令里点名这份文件的 Bash 输出里出现过、它自己还没写过的行。
  - 重复写：它自己前面已经写过的同一文件同一行（反复 Write 同一份文件、反复改同一处）。
  - 实质行：剩下的，去重计数。
- 实质行的去向：
  - 逐字落地：在当前工作树同一文件里，或 2026-09-25 以来某次提交把它加进同一文件（搬家的文件按「文件名」或「上一级目录加文件名」认）；`lib_item_verdicts.py` 提交前改名为 `lib_verdict_library.py`，这一份按改名后的文件认。
  - 机械层也算找到、但不是它落地的证据：开工前仓里别处就有的同样一行（照抄的常见写法）、只出现在 `research/prompts/` 评审材料里、只出现在别的文件里。
  - 机械层没找到的：当前同一文件里有包含它或被它包含的行（子串）、有相似度 ≥0.75 的行（近似）、两样都没有（没了）。
- 「机械假阳性行」= 原有行里找到的 + 重复写里找到的 + 上一条第二类。「偏差」= 机械落地率减实质逐字落地率。
- 判定：每个子 agent 取一行实质行查 `git log -S` 与当前文件的行号，一行机械层没找到的查同样两条；从主会话找它交回之后打它补丁的命令（`apply-writer-patch.py`、`merge-and-verify.sh`、`git apply`、`compile-then-swap.py`）；直接改主工作区的不需要补丁。后来被改写的，另从全部会话里找后来哪个 Edit 的 old_string 带着它的行、new_string 去掉了（`who_rewrote.py`；只认得出 Edit，Write 整份重写与 Bash 里改的认不出）。

## 逐条判定表

两列落地率：「机械算的落地率」照输入原样；「实质改动的落地率」先给逐字的（两种原有行口径各一个数），再给算上改过形的（底座口径）。

| 子 agent | 桶 | 判定 | 机械：算进 / 找到 | 机械算的落地率 | 实质行（底座口径 / 含 Bash 所见口径） | 实质行占算进 | 实质改动的落地率（逐字，两口径） | 实质改动的落地率（含改过形） | 机械假阳性行（两口径） | 改过形还在（子串+近似） | 偏差：机械减实质逐字（两口径） | 实质行现在还在工作树 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| a86536e1da3310cff | landed | 进了，现在还在 | 354 / 350 | 99% | 338 / 327 | 95% / 92% | 99% / 99% | 100% | 16 / 27 | 4 | +0 / +0 点 | 98% |
| a7954321d4a7ed629 | landed | 进了，现在还在 | 316 / 245 | 78% | 299 / 299 | 95% / 95% | 76% / 76% | 88% | 17 / 17 | 34 | +1 / +1 点 | 67% |
| ae7ff5ead0f0c92a9 | landed | 进了，现在还在 | 1318 / 1221 | 93% | 1052 / 1049 | 80% / 80% | 89% / 89% | 95% | 281 / 284 | 64 | +3 / +3 点 | 89% |
| af50d62429598394c | landed | 进了，现在还在 | 113 / 112 | 99% | 102 / 81 | 90% / 72% | 97% / 96% | 98% | 13 / 34 | 1 | +2 / +3 点 | 97% |
| a6a319e5aca7b665e | landed | 进了，现在还在 | 412 / 382 | 93% | 347 / 347 | 84% / 84% | 91% / 91% | 95% | 65 / 65 | 12 | +1 / +1 点 | 85% |
| af8f5b46b56798315 | landed | 进了，现在还在 | 70 / 70 | 100% | 52 / 52 | 74% / 74% | 100% / 100% | 100% | 18 / 18 | 0 | +0 / +0 点 | 100% |
| a058a19e002ebf539 | landed | 进了，现在还在 | 288 / 264 | 92% | 270 / 270 | 94% / 94% | 90% / 90% | 96% | 21 / 21 | 16 | +2 / +2 点 | 89% |
| a70ad565379360ef4 | landed | 进了，现在还在 | 573 / 521 | 91% | 535 / 535 | 93% / 93% | 88% / 88% | 92% | 51 / 51 | 24 | +3 / +3 点 | 87% |
| aaf4b10d070375986 | landed | 进了，现在还在 | 293 / 273 | 93% | 219 / 215 | 75% / 73% | 91% / 92% | 97% | 73 / 76 | 12 | +2 / +2 点 | 91% |
| afd580592d74ce45c | landed | 进了，现在还在 | 131 / 98 | 75% | 126 / 126 | 96% / 96% | 71% / 71% | 71% | 8 / 8 | 0 | +3 / +3 点 | 71% |
| a658ad4f4485aae50 | landed | 进了，现在还在 | 150 / 110 | 73% | 146 / 146 | 97% / 97% | 73% / 73% | 95% | 4 / 4 | 32 | +1 / +1 点 | 71% |
| adf5a5efff8072fde | landed | 进了，现在还在 | 241 / 233 | 97% | 217 / 217 | 90% / 90% | 96% / 96% | 99% | 24 / 24 | 6 | +0 / +0 点 | 82% |
| a77511a9daad3f2c1 | landed | 进了，现在还在 | 19 / 19 | 100% | 16 / 15 | 84% / 79% | 100% / 100% | 100% | 3 / 4 | 0 | +0 / +0 点 | 88% |
| a53c20c1ec7d3d9f6 | landed | 进了，现在还在 | 570 / 558 | 98% | 511 / 464 | 90% / 81% | 97% / 97% | 98% | 60 / 107 | 3 | +0 / +1 点 | 80% |
| aca0f3516a6c4e4df | landed | 进了，现在还在 | 132 / 130 | 98% | 114 / 113 | 86% / 86% | 98% / 98% | 98% | 18 / 19 | 0 | +0 / +0 点 | 94% |
| a88ca34b375effef7 | landed | 进了，现在还在 | 104 / 104 | 100% | 103 / 103 | 99% / 99% | 100% / 100% | 100% | 1 / 1 | 0 | +0 / +0 点 | 99% |
| ad6ea00bde381c447 | landed | 进了，后来被改写 | 864 / 845 | 98% | 774 / 772 | 90% / 89% | 97% / 97% | 98% | 91 / 93 | 1 | +0 / +0 点 | 0% |
| a0c2df320382c14bd | landed | 进了，现在还在 | 213 / 204 | 96% | 166 / 166 | 78% / 78% | 95% / 95% | 98% | 47 / 47 | 6 | +1 / +1 点 | 95% |
| a54c2b11381199cbc | landed | 进了，现在还在 | 577 / 528 | 92% | 563 / 515 | 98% / 89% | 87% / 86% | 90% | 37 / 85 | 13 | +4 / +5 点 | 81% |
| aade021ff8ae5ca51 | landed | 进了，现在还在 | 178 / 159 | 89% | 168 / 168 | 94% / 94% | 89% / 89% | 98% | 10 / 10 | 15 | +1 / +1 点 | 80% |
| ac9b9ef5e6dc060de | landed | 进了，现在还在 | 197 / 182 | 92% | 185 / 183 | 94% / 93% | 92% / 92% | 99% | 12 / 14 | 14 | +0 / +1 点 | 92% |
| a1a02effcc69f1d3f | landed | 进了，现在还在 | 368 / 339 | 92% | 325 / 325 | 88% / 88% | 90% / 90% | 98% | 46 / 46 | 25 | +2 / +2 点 | 90% |
| a07572c1821583b48 | partial | 进了，后来被改写 | 34 / 15 | 44% | 34 / 34 | 100% / 100% | 41% / 41% | 53% | 1 / 1 | 4 | +3 / +3 点 | 38% |
| ae97e5805549afde0 | partial | 进了，后来被改写 | 152 / 49 | 32% | 148 / 147 | 97% / 97% | 32% / 31% | 53% | 2 / 3 | 31 | +0 / +1 点 | 18% |
| a0ea839d90c7b4c17 | partial | 进了，现在还在 | 114 / 77 | 68% | 112 / 112 | 98% / 98% | 0% / 0% | 0% | 77 / 77 | 0 | +68 / +68 点 | 0% |
| a901b3b2b7ccf8506 | partial | 进了，现在还在 | 12 / 4 | 33% | 12 / 12 | 100% / 100% | 33% / 33% | 92% | 0 / 0 | 7 | +0 / +0 点 | 33% |
| afca530b3e5a08c44 | partial | 进了，现在还在 | 10 / 4 | 40% | 10 / 10 | 100% / 100% | 30% / 30% | 40% | 1 / 1 | 1 | +10 / +10 点 | 0% |
| a76c7422a3579c0d8 | partial | 进了，现在还在 | 45 / 27 | 60% | 45 / 45 | 100% / 100% | 60% / 60% | 89% | 0 / 0 | 13 | +0 / +0 点 | 60% |
| af006c49937ea46bc | partial | 进了，后来被改写 | 785 / 394 | 50% | 696 / 557 | 89% / 71% | 26% / 20% | 36% | 212 / 281 | 70 | +24 / +30 点 | 24% |
| a4486895be9577c08 | partial | 进了，现在还在 | 14 / 7 | 50% | 14 / 14 | 100% / 100% | 50% / 50% | 86% | 0 / 0 | 5 | +0 / +0 点 | 50% |
| a871a511ed0c92de2 | partial | 进了，后来被改写 | 145 / 84 | 58% | 145 / 90 | 100% / 62% | 55% / 41% | 61% | 4 / 47 | 8 | +3 / +17 点 | 11% |
| a517cdac4b55c5820 | partial | 进了，后来被改写 | 125 / 74 | 59% | 112 / 112 | 90% / 90% | 54% / 54% | 65% | 13 / 13 | 12 | +5 / +5 点 | 54% |

- 底座口径：偏差均值 +4.4 点（中位 +1.2 点）；landed 22 个 +1.3 点，partial 10 个 +11.3 点；机械假阳性行占算进行均值 11.6%；除去 a0ea839d9 后偏差均值 +2.4 点
- 含 Bash 所见口径：偏差均值 +5.1 点（中位 +1.2 点）；landed 22 个 +1.4 点，partial 10 个 +13.3 点；机械假阳性行占算进行均值 14.3%；除去 a0ea839d9 后偏差均值 +3.1 点
- 改过形还在（机械判没找到、实际在仓里改过形）占算进行均值 8.6%


## 逐条依据

每条：判定；一行实质行的 `git log -S`（只查那份文件、2026-09-20 以来）与当前行号；一行机械层没找到的实质行同样两条；它写进哪棵树；主会话在它交回之后打它补丁的那条命令（摘录）。不足的在下一节「逐条补充」里补。

### a86536e1da3310cff（tooling-writer，2026-10-15，工具实现员：变异脚本收尾批）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/gate.d/stage-owners.tsv`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/stage-owners.tsv`，整行是 'checker-tier-broad-mutation.sh	gate-verdict	investigator,implementatio'（整行，这里只抄前 70 字） → 04ceaa904 2026-10-18
  `grep -nF <整行> .claude/gate.d/stage-owners.tsv` → 第 16 行
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a7954321d4a7ed629（experiment-runner，2026-10-03，Run E43 and E49 reruns）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/kb/experiments.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments.md`，整行是 '| E43（扩展点字节上限） | 已跑（08-30；记录头 311 → 487 重跑 2026-10-04） | 没夹出有约束力的上界：上界'（整行，这里只抄前 70 字） → f3b4e69ff 2026-10-10
  `grep -nF <整行> .claude/kb/experiments.md` → 第 68 行
- 机械层没找到的一行（`research/prompts/e49-r2-prereg.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- research/prompts/e49-r2-prereg.md`，整行是 '读数（`nice -n 19 … cargo test --release --bin e49_chain_width`，输出 `/tmp/'（整行，这里只抄前 70 字） → （无输出）
  当前工作树没有 `research/prompts/e49-r2-prereg.md`
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### ae7ff5ead0f0c92a9（implementation-writer，2026-10-16，Truncate history, translation and model wiring）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/src/model_comparison.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/model_comparison.rs`，整行是 '/// 改 size 被拒（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第'（整行，这里只抄前 70 字） → 0b926bd3f 2026-10-21
  `grep -nF <整行> crates/singlefs-harness/src/model_comparison.rs` → 第 1904 行
- 机械层没找到的一行（`crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs`，整行是 '/// 重写上段根，标签 4 换回 2，最后一批不放单元，判决第六节 S1）；4 变长到 P + 6（跨进单元 1 留洞，标签 4）；5 截'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-harness/tests/truncation_random_histories_against_the_model.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m4-trunc-wiring-writer/repo
- 主会话在它交回之后打补丁（2026-10-16）：`…fs/work/m4-trunc-apply-backup/wiring; mkdir -p $B; cd $W && git apply --numstat $P/crates.patch && git apply --check $P/crates.patch && echo CHECK-OK; cat $P/mutations-delete.txt; ls ~/.local/share/singlefs…`

### af50d62429598394c（implementation-writer，2026-10-21，Add a sensitive case for row 4796）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-checker-tier/tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs`，整行是 '// 恢复内核记 extent 上段叶条目那一段（`crash_judge_common.wgsl` 的 `R_UPPER`，每条 2 字）'（整行，这里只抄前 70 字） → 0b926bd3f 2026-10-21
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs` → 第 121 行
- 写进哪里：~/.local/share/singlefs/work/m4-validate-4796-writer/copy
- 主会话在它交回之后打补丁（2026-10-21）：`…inglefs/ $W/tree/ && du -sh $W/tree && python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m4-validate-4796-writer/patch --root $W/tree 2>&1 | tail -15…`

### a6a319e5aca7b665e（experiment-runner，2026-10-08，E176 第二段）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md`，整行是 '| M 行翻面观测第二句：M2、M3 每次 unlink 比 M1 多重写的叶片数 | 已够判。位置记法：Δ2p、Δ3p 在三个窗口都「不翻'（整行，这里只抄前 70 字） → f3b4e69ff 2026-10-10
  `grep -nF <整行> .claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md` → 第 617 行
- 机械层没找到的一行（`.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md`，整行是 '两点同判的 205 对里 74 对挪门槛之后转成不同；挪门槛自证做不了的对数是 131（两点值逐位相等：「借多少」32 对，是两种借法都一次'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### af8f5b46b56798315（implementation-writer，2026-10-13，修 c499 用例：记录槽提示下的整环扫描断言）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/src/read_tally.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/read_tally.rs`，整行是 '/// journal 提示怎么答由 [`JournalHintAnswer`] 定；写、清零与屏障原样转交、不记。读是 `&self`，所'（整行，这里只抄前 70 字） → d6dd6da42 2026-10-14
  `grep -nF <整行> crates/singlefs-harness/src/read_tally.rs` → 第 165 行
- 写进哪里：~/.local/share/singlefs/work/impl-c499-hint/copy
- 主会话在它交回之后打补丁（2026-10-13）：`…~/.local/share/singlefs/work/impl-c499-hint/patch; ls "$P"; git apply --stat "$P/crates.patch" | tail -n 3; python3 scripts/gate/apply-writer-patch.py "$P" --dry-run 2>&1 | tail -n 3 | cut -c1-200; df…`

### a058a19e002ebf539（implementation-writer，2026-10-05，27-2 25-1 follow-ups and E171 device rev5）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs`，整行是 '//! 同一个 135（第 25 批代码轮正推 K1 第一行与第四行，判决 `research/prompts/m3-formal-code'（整行，这里只抄前 70 字） → f8a9ddaab 2026-10-10
  `grep -nF <整行> crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs` → 第 5 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`，整行是 '/// 一条注入：落地串满足全部条件的态，`piece` 那一件 panic。各条选中的态互不相交（造的人保证；[`PiecePanicIn'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` → （无输出）
- 写进哪里：/tmp/claude-1000/m3-formal-27-2/repo
- 主会话在它交回之后打补丁（2026-10-05）：`…_WITH_MEMORY_CAP_WAIT_SECONDS=3600 && bash research/scripts/merge-and-verify.sh --threads 7 /tmp/claude-1000/m3-formal-27-2/patch > /tmp/claude-1000/-home-fy5090-code-singlefs/3c4dae2e-4622-4601-9f3b-…`

### a70ad565379360ef4（implementation-writer，2026-10-17，W13 修红批第二批）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/tests/writable_mount_refuses_a_deleted_inodes_root_whose_entries_are_wider_than_eight_bytes.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/writable_mount_refuses_a_deleted_inodes_root_whose_entries_are_wider_than_eight_bytes.rs`，整行是 '/// 码 2 节点头（key 宽 k）：条目数在 82 + 2k、条目宽在 84 + 2k、条目区从 115 + 2k 起（D18（块里携'（整行，这里只抄前 70 字） → df5866f67 2026-10-18
  `grep -nF <整行> crates/singlefs-harness/tests/writable_mount_refuses_a_deleted_inodes_root_whose_entries_are_wider_than_eight_bytes.rs` → 第 29 行
- 机械层没找到的一行（`crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs`，整行是 '/// 真做取号、写行、推抬 F、续做 19 次发布落盘之后才交 C841；读一次两边共用之后在取号之前拒、交那次读错，盘不变。' → （无输出）
  `grep -nF <整行> crates/singlefs-harness/tests/writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-b44-w13/repo
- 主会话里没找到点它交回路径的打补丁命令，见逐条补充

### aaf4b10d070375986（implementation-writer，2026-10-08，10z-k: GPU fixes from ghi round）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/src/gpu_memory_ledger.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/gpu_memory_ledger.rs`，整行是 '/// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K1-1，攻方 `opus_k11_poll_race.rs` 的 ta'（整行，这里只抄前 70 字） → f8a9ddaab 2026-10-10
  `grep -nF <整行> crates/singlefs-harness/src/gpu_memory_ledger.rs` → 第 5321 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/src/gpu_unit_checks.rs`，整行是 '/// （wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。取留着的设备不等别的线程起设备（`DeviceSt'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/src/gpu_unit_checks.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-r1/batch10z-k/repo
- 主会话里没找到点它交回路径的打补丁命令，见逐条补充

### afd580592d74ce45c（tooling-writer，2026-10-09，Formal verification per-item and reuse）

判定：**进了，现在还在**。

- 机械层没找到的一行（`.claude/rules/verification.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/rules/verification.md`，整行是 'checker-tier-research-build-and-replay 的 experiment-replay 格按实验、checke'（整行，这里只抄前 70 字） → 04ceaa904 2026-10-18；0df2bd4aa 2026-10-10
  `grep -nF <整行> .claude/rules/verification.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a658ad4f4485aae50（tooling-writer，2026-10-14，工具实现员：准入与门禁单元键）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/hooks/selftest-registry.tsv`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/hooks/selftest-registry.tsv`，整行是 'scripts/gate/admission.py	light	# 临时仓里跑：单元键的阳性与阴性对照、判定库、门禁复用答法、实验键、复跑表'（整行，这里只抄前 70 字） → 04ceaa904 2026-10-18
  `grep -nF <整行> .claude/hooks/selftest-registry.tsv` → 第 24 行
- 机械层没找到的一行（`.claude/hooks/selftest-registry.tsv`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/hooks/selftest-registry.tsv`，整行是 '（假 cargo、假 rustc，照 cases/ 打日志），阶段走样本档、判绿退 3；unit-key-samples-* 另由 admi'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/hooks/selftest-registry.tsv` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### adf5a5efff8072fde（implementation-writer，2026-10-03，第 12 批实现员 Y：崩溃注入与挂载透明）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/src/mount_transparency.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/mount_transparency.rs`，整行是 '/// 按位置取界（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verificati'（整行，这里只抄前 70 字） → 6c1509edd 2026-10-03
  `grep -nF <整行> crates/singlefs-harness/src/mount_transparency.rs` → 第 514 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/src/crash_injection.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/src/crash_injection.rs`，整行是 '/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件，有树而文件集合为空（删完那一版）时建一个 size 0 的 inode' → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/src/crash_injection.rs` → （无输出）
- 写进哪里：/tmp/claude-1000/m3-batch12/y/repo
- 主会话在它交回之后打补丁（2026-10-03）：`…'合入后验证一趟跑 harness 档全部二进制，上限按整批最大那一个给' bash research/scripts/merge-and-verify.sh --memory 16G --threads 31 /tmp/claude-1000/m3-batch12/x/patch /tmp/claude-1000/m3-batch12/y/patch /tmp/claude-1000/m3-ba…`

### a77511a9daad3f2c1（implementation-writer，2026-10-21，Add GPU parity sample for B2 reading）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`，整行是 '/// 这两个场景在对拍里由 GPU 判，那张表收窄时变异副本里也抓得到）；再加 I-3.15 隔离判据读一份头用不了、没有 key 指着的'（整行，这里只抄前 70 字） → 0b926bd3f 2026-10-21
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` → 第 3717 行
- 写进哪里：~/.local/share/singlefs/work/m4-validate-i315-b2-gpu-sample-writer/tree
- 主会话在它交回之后打补丁（2026-10-21）：`…s/work/m4-validate-4796-verify/tree && python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m4-validate-i315-b2-writer/patch --root $T 2>&1 | tail -2 && python3 scripts/gate/apply-w…`

### a53c20c1ec7d3d9f6（tooling-writer，2026-10-14，工具实现员：QEMU LKMM Verus 单元键）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/tier-key-samples-init-heredoc/setup.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/tier-key-samples-init-heredoc/setup.sh`，整行是 'printf '\''kernel\t.qemu-key-samples/kernel.img\nbusybox\t%s\tBusyBox v1.'（整行，这里只抄前 70 字） → 04ceaa904 2026-10-18
  `grep -nF <整行> .claude/gate.d/fixtures/checker-tier-qemu-device-streams.sh/tier-key-samples-init-heredoc/setup.sh` → 第 98 行
- 机械层没找到的一行（`.claude/gate.d/fixtures/checker-tier-formal-verification.sh/verus-proof-green/expect`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/fixtures/checker-tier-formal-verification.sh/verus-proof-green/expect`，整行是 'want=3 个 fn 各一项，各项 verified 之和 2、0 errors（登记 2）' → （无输出）
  `grep -nF <整行> .claude/gate.d/fixtures/checker-tier-formal-verification.sh/verus-proof-green/expect` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### aca0f3516a6c4e4df（implementation-writer，2026-10-01，m3-code-r2 判决改法 J1–J5）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`，整行是 '/// J1 的一格（代码三方 m3-code-r2，攻方改错 RG「把区域号当盘号」）：P 是 txg 8，根环槽在区域 2、槽 2，区域'（整行，这里只抄前 70 字） → 6c1509edd 2026-10-03
  `grep -nF <整行> crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` → 第 713 行
- 机械层没找到的一行（`crates/singlefs-core/src/block_device.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-core/src/block_device.rs`，整行是 '// （对齐、落在设备范围内），长度 0 也在 `write_zeroes_at` 里先交回、不到这里。跨过文件末尾的打洞，文件内那一半照常'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-core/src/block_device.rs` → （无输出）
- 写进哪里：/tmp/claude-1000/m3-r1/code-r2-fixes/work
- 主会话在它交回之后打补丁（2026-10-01）：`… wc -l < $B; cd ~/code/singlefs && python3 research/scripts/apply-writer-patch.py /tmp/claude-1000/m3-r1/code-r2-fixes/patch --root /tmp/claude-1000/m3-r1/merge-wt 2>&1 | grep -E '✓|✗|退出码' | head -4…`

### a88ca34b375effef7（experiment-runner，2026-10-03，22-4a E170 r2 segment 1b）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_one_b_anchors_and_freeze.py`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_one_b_anchors_and_freeze.py`，整行是 'python3 <本文件> --dumps research/target/e170-r2-today-dumps/e170-dump --'（整行，这里只抄前 70 字） → 503b80e2a 2026-10-11；51804a684 2026-10-10
  当前工作树没有 `research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_one_b_anchors_and_freeze.py`
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### ad6ea00bde381c447（implementation-writer，2026-10-10，Merge impl V: versions and store）

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`crates/singlefs-checker-tier/build.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/build.rs`，整行是 'use std::path::{Path, PathBuf};' → 73866746b 2026-10-11
  `grep -nF <整行> crates/singlefs-checker-tier/build.rs` → 第 22 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/src/judging_closure.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/src/judging_closure.rs`，整行是 '/// 这一趟列过目录的每个目录与工作区的 crate 目录（仓内路径）：`build.rs` 照它报 `rerun-if-changed`'（整行，这里只抄前 70 字） → （无输出）
  当前工作树没有 `crates/singlefs-checker-tier/src/judging_closure.rs`
- 写进哪里：~/.local/share/singlefs/work/m3-merge-impl/v/repo
- 主会话在它交回之后打补丁（2026-10-10）：`…ork/m3-merge-impl/v/patch; sha256sum $P/* | cut -c1-16,65-; git apply --check $P/crates.patch && echo APPLY-OK; git apply --stat $P/crates.patch | tail -6; ls research/prompts/ | grep 'm3-batch56-code…`

### a0c2df320382c14bd（implementation-writer，2026-10-21，Finish r3: classifier, GPU parity, GPU mutations）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`，整行是 '/// （判决 `research/prompts/m4-i315-r1-main-verification.md` 第三节 C1「乙二」：'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` → 第 3718 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`，整行是 '/// 再加隔离判据「没有位置项可比」那一格的六形（判决 `research/prompts/m4-i315-r3-main-verific'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m4-i315-r3-writer-2/tree
- 主会话在它交回之后打补丁（2026-10-22）：`…ore、checker、harness、checker 档，整段统一编一次验一次' bash scripts/gate/merge-and-verify.sh --skip-apply ~/.local/share/singlefs/work/m4-fix-red-scenario-tables-writer/patch ~/.local/share/singlefs/work/m4-fix-re…`

### a54c2b11381199cbc（experiment-runner，2026-10-03，E171 第八段：补位与分类）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/kb/experiments/171-三oracle合谋测试.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/171-三oracle合谋测试.md`，整行是 'E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_d'（整行，这里只抄前 70 字） → 0c065788f 2026-10-03
  `grep -nF <整行> .claude/kb/experiments/171-三oracle合谋测试.md` → 第 157 行
- 机械层没找到的一行（`.claude/kb/experiments/171-三oracle合谋测试.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/171-三oracle合谋测试.md`，整行是 '- Python 一侧变异 67 条，表 `research/mutations/e171_three_oracle_collusion.t'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/kb/experiments/171-三oracle合谋测试.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### aade021ff8ae5ca51（tooling-writer，2026-10-13，C350 timing load field）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`scripts/experiment-apparatus/README.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- scripts/experiments/README.md`，整行是 '- 计时实验（`replay.sh` 复跑表里判据写 `timing` 的行）的收尾行写成 `E7RESULT name=done emit'（整行，这里只抄前 70 字） → 1b6f98657 2026-10-14
  `grep -nF <整行> scripts/experiment-apparatus/README.md` → 第 8 行
- 机械层没找到的一行（`scripts/experiment-apparatus/replay.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- scripts/experiment-apparatus/replay.sh`，整行是 'echo "  ✓ replay.sh 自检通过（查了 $checked 项：构建失败返回 2 并打第一处错、日志末尾、日志路径与重现命令，'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> scripts/experiment-apparatus/replay.sh` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### ac9b9ef5e6dc060de（experiment-runner，2026-10-18，实验执行：E180 分片与按需收紧（接续））

判定：**进了，现在还在**。

- 还在或进过历史的一行（`research/prompts/e180-preregistration.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- research/prompts/e180-preregistration.md`，整行是 '| 4.3 | 第一段分成 29 片（5 个几何 × HF、HO、HU、HM、HC，加 g_unequal 之外 4 个几何的 HL）：一片'（整行，这里只抄前 70 字） → e1b1ceb95 2026-10-21
  `grep -nF <整行> research/prompts/e180-preregistration.md` → 第 628 行
- 机械层没找到的一行（`scripts/experiment-apparatus/e180-reading-shards.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- scripts/experiment-apparatus/e180-reading-shards.sh`，整行是 '# long-run-resume: 同一条命令重起就是续跑：装置把一片先写进 <片>.partial-<进程号>、刷盘再换名成 <片>（.'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> scripts/experiment-apparatus/e180-reading-shards.sh` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a1a02effcc69f1d3f（implementation-writer，2026-10-09，Batch 53 close code-round gaps）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-core/src/inode_tree.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-core/src/inode_tree.rs`，整行是 '/// 三片各 117 条、删中间那片的一条（判决 `research/prompts/m3-batch50-code-r1-main-ve'（整行，这里只抄前 70 字） → f8a9ddaab 2026-10-10
  `grep -nF <整行> crates/singlefs-core/src/inode_tree.rs` → 第 1217 行
- 机械层没找到的一行（`crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs`，整行是 '//! 两种写法退役不同的叶、池级 checker 都不红）；再随机 unlink 几个活着的号。每次用户改动之后逐片比记录号与身份，逐次比'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-batch53/gaps/repo
- 主会话在它交回之后打补丁（2026-10-09）：`…nglefs/work/m3-batch50/merge-copy; python3 research/scripts/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch53/gaps/patch --root $D/repo 2>&1 | tail -3; diff -rq --exclude=target $D/repo/cr…`

### a07572c1821583b48（tooling-writer，2026-10-12，批4甲：派发闸三处收紧）

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`research/prompts/abandoned-rounds.tsv`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- research/prompts/abandoned-rounds.tsv`，整行是 '# 派发闸（.claude/hooks/runner-dispatch-guard.sh 的 ⑥）也读这张表：用户叫停、不写判决的轮，行里要'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> research/prompts/abandoned-rounds.tsv` → （无输出）
- 机械层没找到的一行（`.claude/hooks/runner-dispatch-guard.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/hooks/runner-dispatch-guard.sh`，整行是 '"「多机：<台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .cl'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/hooks/runner-dispatch-guard.sh` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-tooling-b4-dispatch/repo
- 主会话在它交回之后打补丁（2026-10-12）：`…3-tooling-b4-gates m3-tooling-b4-defs; do printf '%s: ' $p; git apply --check $W/$p/patch.diff 2>&1 | grep -v 'has type' | head -3; git apply $W/$p/patch.diff 2>/dev/null && echo applied; done; git di…`

### ae97e5805549afde0（experiment-runner，2026-10-08，E175 第一段）

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`，整行是 '| `e175-product-path-inode-watermark-regression-2026-10-08/d-P.out`、`e'（整行，这里只抄前 70 字） → f3b4e69ff 2026-10-10
  `grep -nF <整行> .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md` → 第 64 行
- 机械层没找到的一行（`.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`，整行是 '| W：C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形 | 还差「随机历史」那一半 | 「读数」一节的'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a0ea839d90c7b4c17（implementation-writer，2026-10-16，W11 修红：预演连抬 F）

判定：**进了，现在还在**。

- 机械层没找到的一行（`crates/singlefs-harness/tests/probe_w11_continuation_floor_raise.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/probe_w11_continuation_floor_raise.rs`，整行是 '//! 草稿探针（不进补丁）：窄盘上第一个文件长到几个单元、unlink 到底、回退回来、发意图、崩，再按批大小 1、退役容器一片 1 条可'（整行，这里只抄前 70 字） → （无输出）
  当前工作树没有 `crates/singlefs-harness/tests/probe_w11_continuation_floor_raise.rs`
- 写进哪里：~/.local/share/singlefs/work/m3-b44-w11/copy
- 主会话在它交回之后打补丁（2026-10-16）：`…timeout 900 python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m3-b44-w11/patch 2>&1 | tail -4 | cut -c1-200; ls crates/singlefs-harness/tests/ | grep -…`

### a901b3b2b7ccf8506（tooling-writer，2026-10-13，批7收尾：排除表与开关映射）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/gate.d/code-tooling.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/code-tooling.sh`，整行是 '#   long-run-exempt-sha256-unchecked（不比登记的 sha256）：long-run-header-exe'（整行，这里只抄前 70 字） → 021b4db09 2026-10-14
  `grep -nF <整行> .claude/gate.d/code-tooling.sh` → 第 3180 行
- 机械层没找到的一行（`.claude/gate.d/code-tooling.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/code-tooling.sh`，整行是 '#                            long-run-header-exempt 格的四个开关' → （无输出）
  `grep -nF <整行> .claude/gate.d/code-tooling.sh` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-tooling-b7-fixup/repo
- 主会话里没找到点它交回路径的打补丁命令，见逐条补充

### afca530b3e5a08c44（experiment-runner，2026-10-01，E168 fix S7 accounting halt）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`research/prompts/e168-preregistration.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- research/prompts/e168-preregistration.md`，整行是 '9. S7 判 checker 的那一版镜像。红的不是补丁表的删除路径，也不是判的时刻：记账行里的「已分配」是发布那一刻分配器的计数（`cr'（整行，这里只抄前 70 字） → 31128273b 2026-10-10；146eb4b6e 2026-10-03
  当前工作树没有 `research/prompts/e168-preregistration.md`
- 机械层没找到的一行（`.claude/kb/experiments/168-删除借保留量的环窗口.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/kb/experiments/168-删除借保留量的环窗口.md`，整行是 '- 问题单四行都还没有数。第一段带前缀跑过两次，都没入库。第一次停在 P2-S4 的 `valley_hook_read_outside_a'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/kb/experiments/168-删除借保留量的环窗口.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a76c7422a3579c0d8（tooling-writer，2026-10-11，Exempt test code from code rounds）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`.claude/gate.d/fixtures/doc-process-records.sh/crates-adversarial-review-test-code-only/setup.sh`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/fixtures/doc-process-records.sh/crates-adversarial-review-test-code-only/setup.sh`，整行是 'printf '\''name\tfile\nworkload-two\tcrates/singlefs-harness/src/lib.rs\n'（整行，这里只抄前 70 字） → 021b4db09 2026-10-14
  `grep -nF <整行> .claude/gate.d/fixtures/doc-process-records.sh/crates-adversarial-review-test-code-only/setup.sh` → 第 26 行
- 机械层没找到的一行（`.claude/rules/implementation-workflow.md`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/rules/implementation-workflow.md`，整行是 '| 2 多 agent 正反对抗推理 | 改完的产品代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/rules/implementation-workflow.md` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### af006c49937ea46bc（implementation-writer，2026-10-06，Batch 10s: card pool to all GPU tests, call-site test）

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`crates/singlefs-harness/src/gpu_memory_ledger.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/gpu_memory_ledger.rs`，整行是 '/// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（2026-10-06 日志里那一行）：45 个状态的流只开 45'（整行，这里只抄前 70 字） → f8a9ddaab 2026-10-10
  `grep -nF <整行> crates/singlefs-harness/src/gpu_memory_ledger.rs` → 第 3515 行
- 机械层没找到的一行（`crates/singlefs-harness/src/gpu_memory_ledger.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/src/gpu_memory_ledger.rs`，整行是 '/// `/proc/<进程号>/stat` 的原文里进程的启动时刻（第 22 项）。第 2 项是括号里的进程名，名字里可以有空白与括号，从'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-harness/src/gpu_memory_ledger.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m3-r1/batch10s/tree
- 主会话在它交回之后打补丁（2026-10-06）：`…| grep -c '^[+-]' $P/crates.patch; python3 research/scripts/apply-writer-patch.py $P --dry-run 2>&1 | tail -2…`

### a4486895be9577c08（implementation-writer，2026-10-03，Batch 8 merge pin fixes）

判定：**进了，现在还在**。

- 还在或进过历史的一行（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`，整行是 '/// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录'（整行，这里只抄前 70 字） → d6dd6da42 2026-10-14；6c1509edd 2026-10-03
  `grep -nF <整行> crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` → （无输出）
- 机械层没找到的一行（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`，整行是 'const NOT_APPLICABLE_WITHOUT_FILE: [&str; 28] = [' → （无输出）
  `grep -nF <整行> crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a871a511ed0c92de2（tooling-writer，2026-10-01，广谱变异接门禁（接手））

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`.claude/hooks/lib_heavy_tests.py`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/hooks/lib_heavy_tests.py`，整行是 '("广谱变异的提交门禁经 nice 只跑判定那一格", ["nice", "-n", "19", "bash", ".claude/gate'（整行，这里只抄前 70 字） → 0da17aa14 2026-10-03
  `grep -nF <整行> .claude/hooks/lib_heavy_tests.py` → 第 4020 行
- 机械层没找到的一行（`.claude/gate.d/stage-inputs.tsv`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- .claude/gate.d/stage-inputs.tsv`，整行是 'checker-tier-broad-mutation.sh	crates/ Cargo.toml Cargo.lock litmus/ .'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> .claude/gate.d/stage-inputs.tsv` → （无输出）
- 写进哪里：主工作区
- 它直接改主工作区，没有补丁要打

### a517cdac4b55c5820（implementation-writer，2026-10-15，Fix cluster 丙: checker-tier tests）

判定：**进了，后来被改写**。

- 还在或进过历史的一行（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`，整行是 '// `CHECKER` 行（GPU 判器判：`I-3.6=46/0 I-3.7=46/0 I-3.15=9/0 I-5.6=0/0 I-3'（整行，这里只抄前 70 字） → 0b926bd3f 2026-10-21
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` → 第 310 行
- 机械层没找到的一行（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`）：
  `git log --since=2026-09-20 --format='%h %cs' -S<整行> -- crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`，整行是 '// `.claude/kb/checks-owed.md` C821（GPU 判器没有扫描重建内核）），GPU 判器那一趟评估过的状态数恒'（整行，这里只抄前 70 字） → （无输出）
  `grep -nF <整行> crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` → （无输出）
- 写进哪里：~/.local/share/singlefs/work/m4-fix-checker-tests-writer/copy
- 主会话在它交回之后打补丁（2026-10-15）：`…-dir 里，所有 worktree 共用；在 worktree（m4-predev）里跑 `scripts/gate/apply-writer-patch.py` 只打那棵 worktree，也排在主工作区那趟 `merge-and-verify.sh` 后面（`records/2026-10-13-里程碑四预开发.md`「簇丙交回」那一条）。要拦什么：锁的范围比它护的对象大；出路：锁按工作树（…`


## 逐条补充

- **a86536e1d、a7954321d、a6a319e5a、afd580592、a658ad4f4、a53c20c1e、a88ca34b3、a54c2b113、aade021ff、ac9b9ef5e、ae97e5805、afca530b3、a76c7422a、a4486895b、a871a511e**：都直接改主工作区（上面「写进哪里：主工作区」），没有补丁要打，主会话找不到打补丁的命令是对的。
- **a70ad5653（W13）**：主会话 2026-10-17 `W=~/.local/share/singlefs/work; ls $W/m3-b44-w13/patch $W/m3-b44-w14/patch; git apply --check $W/m3-b44-w13/patch/crates.patch && echo W13-check-ok; …`；落进 df5866f67（2026-10-18），见上表那一行。
- **aaf4b10d0（10z-k）**：主会话 2026-10-08 `W=$HOME/.local/share/singlefs/work/m3-r1/merge-10z-k; P=$HOME/.local/share/singlefs/work/m3-r1/batch10z-k/patch; …`，落进 f8a9ddaab（2026-10-10）。
- **a901b3b2b（批 7 收尾）**：补丁打进集成副本，不是主工作区。主会话 2026-10-13 `cd ~/.local/share/singlefs/work/m3-tooling-b2-integration/repo && sha256sum ../../m3-tooling-b7-fixup/patch.diff | cut -c1-16 && git apply ../../m3-tooling-b7-fixup/patch.diff && git add -A && git commit -qm "b7 fixup" …`；集成副本随 021b4db09（2026-10-14）进仓。机械层没找到的 8 行里 7 行今天以改过形在 `code-tooling.sh`（例：它写的行以「。」收尾，今天那一行以「；」收尾）。
- **a517cdac4（簇丙）**：主会话 2026-10-15 `cd $B/m4-predev; python3 scripts/gate/apply-writer-patch.py ../m4-fix-checker-tests-writer/patch …`，打进 m4-predev，随 0b926bd3f（2026-10-21）进仓。改了名的用例还在：`grep -rn "fn a_discard_moved_onto_the_segment_of_the_instance_table_every_later_root_names_is_red" crates/` → `crates/singlefs-checker-tier/tests/crash_enumeration_lifecycle_closure_stream.rs:468`。被改写的一半：它在随机历史那份用例里加的「去掉 I-1.13 那一位再比」今天变回整份比，当前那一行是 `"{} 张表上 GPU 一趟与 CPU 一趟的对拍字（整份，连同 I-1.13）不同：\n{}",`（它写的是「去掉内核不置位、对拍不比的那几位，C821 的 I-1.13」）。Edit 里认不出改写人。
- **ad6ea00bd（实现 V）**：上面挑出来的 `use std::path::{Path, PathBuf};` 是常见写法，不能当它的证据，换这两条：
  `git log --since=2026-09-20 --format="%h %cs %s" --name-status -- crates/singlefs-checker-tier/src/judging_closure.rs crates/singlefs-checker-tier/build.rs` →
  `73866746b 2026-10-11 …` 下 `A crates/singlefs-checker-tier/build.rs`、`A crates/singlefs-checker-tier/src/judging_closure.rs`；`d6dd6da42 2026-10-14 …` 两份都是 `M`；`df5866f67 2026-10-18 代码：判定按函数单元键复用、函数单元图 crate、…` 下 `M build.rs`、`D judging_closure.rs`。
  它写的 774 行实质行里 753 行只在历史里（表里「现在还在工作树」0%）。改写人：Edit 里认得出 a0fdb76ed54a021e1（2026-10-14「实现员巳：阳性对照先核表归属、判器闭包算法」）与 a98a2d32a1b12a934（2026-10-10「Merge impl E: engine」）各改过它几行；整份删掉在 df5866f67。
- **afd580592（形式化验证逐条）**：机械层算进的 131 行里 124 行在 `research/scripts/lib_item_verdicts.py`，这份文件提交前改了名。主会话 2026-10-10 一条命令里的原话：「暂存时补 `lib_verdict_library.py`、新样本、27 份改名样本表，`lib_item_verdicts.py` 暂存区那条 A 撤掉」。改名后的文件今天在 `scripts/crash/lib_verdict_library.py`：
  `grep -nF 'columns = "、".join(f"{reason} {count}"' scripts/crash/lib_verdict_library.py` → 第 502、509 行；`git log --since=2026-09-20 --format="%h %cs" -S'columns = "、".join(f"{reason} {count}"' -- research/scripts/lib_verdict_library.py scripts/crash/lib_verdict_library.py` → `51804a684 2026-10-10`。
  它主要的改动在草稿目录里写好再拼进仓（见「机械层没算到的产出」），机械层没算。
- **a88ca34b3（E170 第一段乙）**：脚本随 503b80e2a（2026-10-11「研究脚本、变异表与实验装置搬出 research/」）搬家，`ls scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_one_b_anchors_and_freeze.py` 有这份文件。
- **a4486895b（第 8 批改钉）**：用例文件搬进了 `common_images_for_the_checker_tier/`：`grep -n "I-3.5（引用区间的精确性） 也不在这张表里" crates/singlefs-harness/tests/common_images_for_the_checker_tier/writable_mount_of_a_formatted_pool.rs` → 第 268 行。机械层没找到的 7 行里 5 行是 `crates/mutations.tsv` 的变异行，今天整行多了别的列，子串命中（例：`grep -nF` 它写的「二·1 I-8.10 判定恒真…」那一行 → 第 1591 行）。
- **a0c2df320（r3 收尾）**：它的改动在工作树里还没提交，所以 `git log -S` 无输出、`grep` 有行号。`git status --short -- …hand_out.rs …history.rs …fault_injection.rs` → `MM …hand_out.rs`、`M  …fault_injection.rs`、`MM …history.rs`。主会话 2026-10-22 `merge-and-verify.sh --skip-apply` 那一条点了七份补丁目录，它的 `m4-i315-r3-writer-2/patch` 是其中之一。
- **a07572c18（批 4 甲）**：上面挑出来的 `abandoned-rounds.tsv` 那一行今天只在 `research/prompts/m3-defs-b6-r1-opus-model/gate-samples/…/abandoned-rounds.tsv` 这份评审样本里，不算它落地；换这两条：
  `grep -n "abandoned-rounds.tsv" .claude/hooks/runner-dispatch-guard.sh` → 第 36、269、2097 行（第 36 行就是它写的那一行，`git log -S` 那一行 → `d9f06759d 2026-10-14`）；`grep -n "线程上限已判" .claude/hooks/runner-dispatch-guard.sh` → 第 77、81、91 行（G1 的判法还在，文字已换）。
  机械层没找到的 19 行里 15 行没了，多是「多机：」那一行的旧说法。改写人：Edit 里认得出 a6b1f6792da5d83a5（2026-10-13「批6：长活文件头与三处零轮修」），它的 old_string 带着「不许写现算（host-resources、available_threads）、不许写死台数线程内存；」那一行。
- **ae97e5805（E175 第一段）**：实验页同一天被 a353021c3a529e419（2026-10-08「E175 两段在冻结副本上重跑」）重写，Edit 里认得出被换掉的 58 行里 55 处出自它（例：「- **产品臂 P**：F3a 16 105 段（操作步 143 335）、F2 256 段 × 60 步，同一条时间线上的重发号（Q1）0 次，…」那一行）。第一段的读数还在页上：`grep -n "8 932" ".claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md"` → 第 20、145、156、184 行。所以判「进了，后来被改写」，不是「被取代」：读数没被推翻，只是页面换了写法并加了第二段。
- **a0ea839d9（W11）**：机械层算进的 114 行全是 `probe_w11_continuation_floor_raise.rs`，它自己写明「草稿探针（不进补丁）」；其中 72 行是从已有用例照抄的（开工前仓里就有），机械层因此算出 68%。它真正的改动用 Bash 里的 python 写进副本的 `mount.rs`、`mounted_session.rs` 与两份用例，主会话 2026-10-16 `timeout 900 python3 scripts/gate/apply-writer-patch.py ~/.local/share/singlefs/work/m3-b44-w11/patch` 打进主工作区：
  `grep -rn "fn acquire_then_publish_until_the_continuation" crates/` → `crates/singlefs-core/src/mount.rs:7340`；`git log --since=2026-09-20 --format="%h %cs %s" -S"fn acquire_then_publish_until_the_continuation" -- crates/singlefs-core/src/mount.rs` → `df5866f67 2026-10-18 …`；新用例名在 `crates/mutations.tsv` 与 `crates/singlefs-harness/tests/opening_a_container_past_the_largest_number_returns_an_error.rs` 里都有。
- **af006c499（第 10s 批）**：`gpu_memory_ledger.rs` 是整份 Write（1087 行），写之前用 Bash `cat src/gpu_memory_ledger.rs` 读过副本里已有的那份（底座提交里没有这份文件，所以底座口径认不出原文，含 Bash 所见口径多认出 139 行原有行）。主会话 2026-10-06 打进主工作区（`P=~/.local/share/singlefs/work/m3-r1/batch10s/patch; M=~/.local/share/singlefs/work/m3-r1/merge-10s; … python3 research/scripts/apply-writer-patch.py $P 2>&1 | grep -v '^!' | tail -3; …`）。之后到 10-10 提交之前被后续批次改写：Edit 里认得出 ae966ba53424b4a74（2026-10-07「Batch 10w: product-path ledger follow-ups」）换掉 8 行、a503343fce9661c88（2026-10-10「Merge impl E8: environment races」）换掉 3 行；改写前的样子留在评审材料里：`git log --since=2026-09-20 --format="%h %cs %s" -S"/// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份，与空闲显存无关。"` → `04ceaa904 2026-10-18 …`、`68ddefc0b 2026-10-10 三方论证材料：m3 其余各轮（batch10 系列…`（都是 `research/prompts/` 下的文件），今天的那一行是「/// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份。数照一张 5090（空 30 000 MiB，3/4 是 22 500）。」。`gpu_card_configuration.rs` 那一份 74 行今天还在。
- **a871a511e（广谱变异接门禁）**：门禁文件还在（`ls .claude/gate.d | grep broad` → `checker-tier-broad-mutation.sh`），它写的 12 份样本在 04ceaa904（2026-10-18）整批删换：`git log --since=2026-09-20 --diff-filter=D --name-only -- '.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/*'` 数出那一次删了 160 个文件。它改的五份文档（`crash-verifier.md`、`main-agent.md`、`agent-common.md`、两份规则）里的 27 行，历史里一行原样也找不到，进仓时已是别的写法；`stage-inputs.tsv` 那一行用整行做 `git log -S` 有输出（4afa8df18 2026-10-03、8c07d1f6c 2026-10-11），整行比对却没命中，是作为更长一行的一部分进去的。Edit 里认不出改写人。
- **afca530b3（E168 修 S7）**：机械层只算到实验页与登记那 10 行。它的装置修复写在 `/tmp/claude-1000/e168-runner5/draft/e168.rs`、经 `compile-then-swap.py` 换进主工作区：`grep -n "fn unmark_the_filled_slots" crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs` → 第 1530 行；`git log --since=2026-09-20 --format="%h %cs %s" -S"fn unmark_the_filled_slots"` → `13ca0a19a 2026-10-03 …`。
- **a76c7422a（测试代码不开代码轮）**：门禁格还在：`grep -n "TEST_CODE_SOURCES=" .claude/gate.d/doc-process-records.sh` → 第 343 行（今天多了 `singlefs-unit-graph`）；`git log -S"TEST_CODE_SOURCES=" -- .claude/gate.d/doc-process-records.sh` → `021b4db09 2026-10-14`。机械层没找到的 18 行里 13 行在规则、`experiment-runner.md` 与 `main-agent.md` 里，5 行在新样本里；今天这些文件里没有它们的原样，Edit 里认不出改写人。
- **ac9b9ef5e（E180 分片）**：装置草稿 `e180.rs` 不在机械层范围里；主会话 2026-10-18 `bash scripts/hosts/capped.sh 31 python3 scripts/hosts/compile-then-swap.py $R/e180.rs crates/singlefs-checker-tier/src/bin/e180_a…` 换进主工作区，`e180_admission_reading_versus_writable_bytes.rs` 进了 0b926bd3f（2026-10-21）。`e180-reading-shards.sh` 机械层没找到的 13 行，Edit 里认得出 aafa7dde41413cf1f（2026-10-19「长跑分片核装置版本（C862）」）换掉了其中 10 行。

## 假阳性与假阴性从哪来（32 个合计，行数由 `refine.py` 数出）

合计算进 8917 行、机械找到 7682 行（86.1%）。实质行底座口径 7956 行，逐字落地 6456 行（81.1%）；含 Bash 所见口径 7621 行，逐字落地 6204 行（81.4%）。

| 来源 | 行数（底座口径） | 行数（含 Bash 所见口径） | 机械层算不算找到 | 说明 |
|---|---|---|---|---|
| 原有行 | 426 | 761 | 底座口径 426 行全找到；宽口径 707 行找到 | 底座口径下全部出自 Edit：new_string 里的一行与同一文件别处已有的行逐字相同（常见写法、同样的参数行）。Edit 原样带过的上下文行机械层本来就按 old_string 剔掉了，这一类是 0。Write 整份重写已有文件，底座口径认出 0 行；88 次 Write 的目标在底座提交里都不存在、写前也都没 Read 过。宽口径多出的 335 行里，af006c499 一个就占 139 行，是 `cat` 读过之后整份 Write 的真原文；其余多是样本里的 `set -euo pipefail` 这类常见行，以及子 agent 自己用 Bash 写过又看到的行，宽口径会多认 |
| 重复写 | 535 | 535 | 找到 486 | 反复 Write 同一份文件 326 行、反复改同一处 209 行，机械层每次都算，分子分母一起涨 |
| 照抄的已有行 | 130 | 129 | 全找到 | 开工前仓里别处就有的行；a0ea839d9 的草稿探针占 72 行 |
| 只在评审材料里 | 154 | 127 | 全找到 | 只出现在 `research/prompts/` 下的 diff、附录等材料里，代码里没有；af006c499 占 112 行 |
| 只在别的文件里 | 30 | 29 | 全找到 | 同一行出现在不相干的文件里 |
| 改过形还在：子串 | 204 | 202 | 没找到 | 后来在行首行尾加了字、或几行并成一行（例：a86536e1d 在 `mutate.sh` 写的一行今天是第 15 行的一部分，那一行以 `#   （bin` 开头） |
| 改过形还在：近似 | 229 | 209 | 没找到 | 同一文件里有相似度 ≥0.75 的行，多是后来改了几个字 |
| 没了 | 753 | 721 | 没找到 | 当前同一文件里找不到原样也找不到近似；af006c499 292 行、ae97e5805 70 行、a871a511e 53 行占多数 |

两个已知偏差的量：
- 假阳性（机械层算找到、但不是这一行落地的证据）：底座口径 1226 行，占算进行的 13.7%；宽口径 1478 行，占 16.6%。逐个子 agent 平均 11.6% / 14.3%。它们大多同时在分子分母里，所以落地率只偏高平均 4.4 / 5.1 点。
- 假阴性（机械层算没找到、其实以改过形在仓里）：433 行，占算进行的 4.9%；逐个子 agent 平均 8.6%。这一类在 partial 组里多：partial 10 个里 3 个算上它就过了 70%。题目里说的「下一次提交之前被别的 agent 再改过」就落在这一类与「没了」里；改写人从 Edit 认得出的见 `who_rewrote.txt`（32 个里 13 个认得出至少一个改写人），其余是 Write 整份重写或 Bash 里改的，认不出。

## 机械层没算到的产出

机械层只算路径能映射进仓的 Edit / Write。下面 6 个子 agent 的主要改动不在里面，判定时另查了：

| 子 agent | 没算到的写法 | 去向（`drafts.txt`、`drafts2.txt` 里的数） |
|---|---|---|
| a658ad4f4 | 草稿目录 `tooling-admission-units/new/` 里分段写好再拼进 `scripts/gate/admission.py` 等 | `part-units.py` 544 行里 521 行、`part-head.py` 370 行里 351 行、`part-logic.py` 290 行里 285 行、`part-selftest-new.py` 427 行里 418 行今天在 `scripts/gate/admission.py` |
| a53c20c1e | 草稿目录 `tooling-qemu-lkmm-verus-units/draft/` 里的整份新版与改写脚本 | `lib_verdict_library.py` 595 行里 567 行今天在 `scripts/crash/lib_verdict_library.py`；`qemu-stage-tail.sh` 327 行里 287 行今天在 `.claude/gate.d/checker-tier-qemu-device-streams.sh` |
| afd580592 | 草稿目录 `formal-verification-items-and-reuse/draft/` | `verus-proof-check.py` 新增 380 行里 260 行、`e165-pin-red-examples.py` 200 行里 148 行今天在各自的仓内文件 |
| ac9b9ef5e | 装置草稿 `m3-close-w4-e180-runner4b/e180.rs`，主会话用 `compile-then-swap.py` 换进 | 新增 191 行里 166 行今天在 `e180_admission_reading_versus_writable_bytes.rs` |
| afca530b3 | 装置草稿 `/tmp/claude-1000/e168-runner5/draft/e168.rs` | 新增 82 行里 56 行今天在 `e168_delete_ring_window_borrow.rs` |
| a0ea839d9 | Bash 里的 python 改副本里的 `mount.rs`、`mounted_session.rs` 与两份用例 | 见「逐条补充」；机械层只算到不进补丁的草稿探针 |

这 6 个在表里的机械落地率与实质落地率都只反映它们一小部分产出；判定按它们真正的产出下的。

## 汇总

- 判定：进了，现在还在 26（landed 21、partial 5）；进了，后来被改写 6（landed 1：ad6ea00bd；partial 5：a07572c18、ae97e5805、af006c499、a871a511e、a517cdac4）；没进，被另一份改动取代 0；没进，丢了 0；判不了 0。
- 这 32 个是从「写了、找得到」的两个桶里抽的，「没进」两类是 0 是抽样决定的，不说明全体没有；要量「没进」得从 written-overwritten、written-unused 两桶抽。
- 假阳性的平均偏差：底座口径 +4.4 点（中位 +1.2），含 Bash 所见口径 +5.1 点（中位 +1.2）；landed 组 +1.3 / +1.4 点，partial 组 +11.3 / +13.3 点；除去 a0ea839d9 是 +2.4 / +3.1 点。
- 「机械落地率 ≥70% 就算进了」：可信，偏高约 1 点（landed 22 个平均）。要用它的时候补两条：「进了」不等于「现在还在」，要看现状得另外只比当前工作树；低于 70% 不能当「没进」，这一组 10 个全进了。机械层范围外的产出（草稿拼装、Bash 改写）要另算。

## 没做什么

- 没做派发提示之外的活；没跑重型测试；没编译、没跑测试；没做 git 写操作；没提交；没改仓内任何文件。
- 原有行的判法没法照子 agent 开工那一刻主工作区的未提交状态逐文件重建，用了「底座提交加 Read 读到的」与「再加 Bash 输出里见过的」两种口径夹着给。
- 改写人只认得出 Edit 换掉的；Write 整份重写与 Bash 里改的认不出，表里「后来被改写」的几条，改写人列得不全。
- 搬家的文件按「文件名」或「上一级目录加文件名」认，名字常见的（`setup.sh`、`expect` 这类）按上一级目录加文件名认；`lib_item_verdicts.py` 的改名是查主会话认的，别的改名没逐个查，可能漏认（漏认会让实质落地率偏低）。
- m5-dev 工作树也比对过，32 个里没有一行只在它里面。
- 开工时我起过一条在 `/tmp/claude-1000` 下全目录 grep 的命令，超时转进后台，自己停不了（钩子判那个进程属于 Claude 会话、拒停），之后它自己跑完退出了（任务通知 exit 0），没留下进程。

## 草稿与产物

都在 `~/.local/share/singlefs/work/landing-audit-writers/`（不入库，主 agent 定要不要留）：
- 脚本：`extract_ops.py`、`extract_ops2.py`、`index_lines.py`、`index_lines2.py`、`analyze.py`、`analyze2.py`（底座口径）、`analyze3.py`（按 Edit / Write 分类计数）、`analyze4.py`（含 Bash 所见口径）、`refine.py`、`who_rewrote.py`、`evidence.py`、`gen_evidence.py`、`gen_evidence2.py`。
- 数据与输出：`ops.json`、`ops2.json`、`trees.pkl`、`hist.pkl`、`analysis-lower.json`、`analysis-upper.json`、`refined-lower.json`、`refined-upper.json`、`refined.txt`、`refined-upper.txt`、`who_rewrote.json`、`who_rewrote.txt`、`evidence.txt`、`evidence-section.md`、`targeted1.txt`、`targeted2.txt`、`targeted3.txt`、`drafts.txt`、`drafts2.txt`、`final-table.md`、`final-table2.md`、`handbacks.txt`、`perfile.txt`。
- 历史比对读的是机械层留下的 `/tmp/claude-1000/-home-fy5090-code-singlefs/412f440a-3f86-449e-adc8-1563bd743223/scratchpad/audit/gitlog.patch`（2026-09-25 以来 145 个提交）与 `lineset.pkl`。
- 没建仓副本或编译目录。
