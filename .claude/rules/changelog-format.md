# kb 历史怎么组织

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——共享的 `kb-discipline.md` 只要求正文只写现状；这份定的是
哪几类文件有历史、历史住在哪、`decisions-history.md` 与 `experiments-history.md` 两份变更史怎么组织。共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 只有决策与实验有历史，别的文件没有

历史只记两类：决策（`.claude/kb/decisions/`）与实验（`.claude/kb/experiments/`）。它们的历史各住一份变更史：
`.claude/kb/decisions-history.md`、`.claude/kb/experiments-history.md`。

其余文件（`records/`、`.claude/kb/milestone/`、欠账表、不变量清单、布局表、`tooling.md`、`prior-art.md` 这类）不留历史节：
改动直接改正文，经过靠 git。哪些 kb 文件带历史节登记在 `.claude/history-carriers`（一行一个目录或文件），
上游 doc-lint 规则 C 按它判：登记的必须以「## 历史版本」收尾，没登记的不许有这一节。

## 变更史：条目、日期、变更清单三层

```markdown
## D24（后台重活能不能卸给 GPU）

### 2026-08-28

- 新立，待定：三个候选全落在第三格；三条硬约束；判据是「这一格的瓶颈是不是 CPU」。
- 「有 AES-NI ⇒ GPU 输」先降级为推断，随后被 E6（加密算法选型） 多核档实测翻成「GPU 赢」；不许写成一定赢。
  - **依据**：E6（加密算法选型） 多核档，两条阳性对照都过。
  - **改前**：`git show c5b81923:.claude/kb/decisions.md` 里 D24 那一节。
```

- **条目**：每条决策一个 `## D<n>（简称）` 节，每个实验一个 `## E<n>（简称）` 节。节顶上不写现状，现状只在 `decisions.md`、`experiments.md` 的索引表里。
- **日期**：节里按 `### YYYY-MM-DD` 分块，同一节同一天只有一块，新的日期在上。
- **变更**：块下每条改动一个顶格的 `- ` 项，写结论、依据、改前，正文缩进两格放在项下。不写 `#### ` 子条目、「（其N）」、「> 快查·改前 / 改后」两行。
- **日期只在标题上**：项里不写与所属 `### ` 相同的日期；别的日期照写。路径、产物名、引的小节名里的日期是名字的一部分，不动。
- **改前只写引用**：`git show <提交>:<路径>` 一行，不整段照抄旧原文；那一版之外的中间态才照录几行。
- **一条改动点名几条决策**：只在标题里点名的第一条决策的节里写全文，别的节写一行「- 见 D<n>（简称） 节同日条目「摘要」」。实验同理。

同一天可以先追加几个 `- ` 项；提交前主 agent 把当天的项整理成结论，门禁在 `--staged` 那一趟判。
整理只许做四种：与标题同日的日期删掉、包装（快查、现状、`####`）删掉、改前原文换 git 引用、批量条目换指针；
数、编号、限定词、「因为」引出的理由、引名一个都不删，改前没有的依据句也不加。整理前后跑
`python3 research/scripts/history-consolidate.py loss-check <改前> <改后>`，退出码不是 0 不许暂存。

## 门禁管哪一半

门禁 doc-decisions（`.claude/gate.d/doc-decisions.sh`）的两格：`entry-added`（决策正文改了，`decisions-history.md` 要有新增的 `- ` 项或日期块）、
`shape`（日期块住在自己的节、同节同一天只一块、两份变更史里不许 `#### `、快查行与现状行、新开的块不许空）。具体判据以那份脚本的文件头为准。
数与编号有没有丢由 `research/scripts/history-consolidate.py loss-check` 判，暂存前跑。

**它管不了的**：结论写得对不对、一条改动该点名的决策或实验有没有点全、整理时限定词丢没丢——这几样靠人。
