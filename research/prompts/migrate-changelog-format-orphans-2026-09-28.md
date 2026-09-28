# 没点名任何决策 / 实验的原条目

`research/scripts/migrate-changelog-format.py` 搬迁时认不出它们属于哪一节，原样列在这里，去处待定。

## .claude/kb/decisions-history/2026-09.md:10053

### 2026-09-12（其五）：命名回扫收尾——四个登记过的格式常量名与两个盘上字段名写全，txg / jsn 登记成缩写

> 快查·改前：四个格式常量登记名与两个盘上字段名都用了缩写（如hdr、rec）。
>
> 快查·改后：四个常量名与两个字段名改写成全称，值和语义不变，txg、jsn 登记进缩写表。

- **改前**：format-const 登记名是 `UNIT_HDR_DATA`、`UNIT_HDR_PACKED`、`INODE_REC`、`JOURNAL_HDR`；决策正文里树表条目与 journal 记录头的两个字段叫 `prev_snap_txg`、`prev_hash`。
- **改后**：四个登记名依次改成 `DATA_UNIT_HEADER_BYTES`、`PACKED_UNIT_HEADER_BYTES`、`INODE_RECORD_BYTES`、`JOURNAL_HEADER_BYTES`，值一个没动，`JOURNAL_HEADER_BYTES` 标记里 stale 列的两条源码字面串跟着换名；两个字段改叫 `previous_snapshot_txg`、`previous_hash`，宽度、位置、语义一个没动。只改正文：各决策文件「## 历史版本」一节与 decisions-history.md 的旧条目照旧写旧名。
- **依据**：naming-lint 回扫报出 hdr、rec 两个缩写（`.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」一节：不用缩写）。登记名只改源码的话，`.claude/gate.d/27-format-constants.sh` 按名字找不到源码里的定义，那道同步检查就不再查它，所以 kb 登记位与源码一起改；`PACKED_UNIT_HEADER_BYTES`（103）与 `INODE_RECORD_BYTES`（140）另有六份实验源码早就用着同名同值的常量，改名后它们也进了门禁 27 的射程。字段名跟随代码里已经写成的全称。`txg`、`jsn` 在 kb 里是设计词汇（各有上百行），登记进 `.claude/abbreviations`，代码里零散写成全称的几处统一回缩写。改完之后受影响的 36 个实验用 `research/scripts/replay.sh` 复跑，全部与留存产物逐字节一致。


## .claude/kb/experiments-history.md:2436

### 2026-08-31：实验编号重排 —— 消掉那个空号，24 号及其之后整体下移一位

- **改前**：编号 1..56 里 **23 号是空的**——全仓连 git 历史都零命中，查不出它存在过。
- **改后**：24 号及其之后整体下移一位（原 24 号成为 23 号，依此类推到原 56 号成为 55 号），编号从 1 连到 55，无空号。
- **依据**：用户定案（2026-08-31）。空号本身不影响正确性，但它是一个**答不出来的问题**：
  检索到那个空号的人会以为有一份正文而找不到，而 kb 的目标是让模型不编
  （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 3 条）。
- **跟着改的**：kb 实验正文 33 份（文件名 + 标题）、`research/` 下的源码 / 产物 / 提示 / 变异表共 129 个文件改名，
  `research/scripts/replay.sh` 的对照表、`research/e7-index-bench/Cargo.toml` 的 19 个 bin，
  以及全仓 1000 余处 `E<n>` 引用。
- **重排是安全的，这一点是验过的不是推的**：产物内容里不含实验号（102 个产物里只有 2 个 LLM 问答文件提到 E 记号，
  与实验身份无关）⇒ 改名不改字节。复跑现查：**22 个实验逐字节一致、0 个对不上、0 个跑不了**，
  `research` 构建 392 个单测全绿，门禁的 `10-kb-rot` / `40-results-cited` / `85-repro-command` /
  `86-experiment-orphans` / `80-absolute-assertions` 全绿。
- ⚠️ **踩到一个坑，记下来**：第一遍用 `\bE(\d+)\b` 做替换，**漏掉了紧贴汉字的引用**
  （形如「按」字紧贴一个 E 记号、中间没有空格的那种写法）——Python 的 `\w` 把汉字算作单词字符，所以汉字与 `E` 之间没有词边界。
  第二遍改成**按简称反查编号**（简称 → 登记编号，不符就改号），这个做法幂等、不会二次移位，补回 61 处。


## .claude/kb/experiments-history.md:2927

### 2026-08-29（其七）：mutate.sh 有个会谎报盲区的 bug；门禁从来没构建过 research

**这两条都是「检查自己坏了」，方向都是让人以为出了问题而其实没有、或反过来。**

- **`research/scripts/mutate.sh` 提取「红了哪个测试」的正则是 `[a-z_]*`，不含数字。**
  **曾经**：任何名字里带数字的测试（`..._analytic_io_per_op_of_1_9375`、`..._reads_exactly_122`、
  `..._as_d9_predicted`）匹配不上 ⇒ `red` 为空 ⇒ **一条确实红了的变异被报成「一个测试都没红」**；
  **现在**：改成 `[A-Za-z0-9_]*`；
  **依据**：2026-08-29 手动复核 e7 的「缓存默认值改了」那条变异——
  脚本报「没红」，而 `cargo test` 明确显示该测试 FAILED。
  ⚠️ **方向是谎报盲区**（不会把没抓到报成抓到），但它会把人引去补一条本来就有的检查。
  ⚠️ **只有三个测试名带数字，且全是本轮新加的** ⇒ **此前那些「N 条变异全部被抓」的记录不受影响。**

- **门禁从来没有构建过 `research/`。**
  **曾经**：共享门禁的「构建与单测」只看 `crates/`，而本工程还没有 `crates/`，
  于是那一阶段恒报「不适用」；**而 kb 里几乎每条实测结论都由 `research/` 下的实验二进制背书**；
  **现在**：新增门禁阶段 `.claude/gate.d/15-research-build.sh`，双向证过会红；
  **依据**：本轮实测——`Cargo.toml` 里留了一个指向已删源码的 `[[bin]]`，
  `cargo test` 直接 `can't find bin`，**而门禁全绿**。
  **一个连编都编不过的证据仓库，比没有证据更糟：它看起来还在。**

- **口径更正**：本轮单测总数为 **160**（24 个批次）。
  此前报过 155 与 170 两个数，都是计数命令解析错——155 少算、170 含攻方腿那份已删代码的 11 个。


## .claude/kb/experiments-history.md:3051

### 2026-08-29（其二）：experiments.md 拆成索引 + 每实验一文件 + 变更史

- **曾经**：单文件 2556 行、30 个实验，正文与历史混在一起；
  **现在**：`experiments.md` 只留 30 行索引表，正文进 `experiments/NN-简称.md`，
  历史进 experiments-history.md；
  **依据**：与 `decisions.md` 拆分同一条——检索一次取出一个实验即可，
  不必把整份文档拉进上下文。零行内容丢失（剥掉简称与相对路径后逐行比对 HEAD）。
- **登记位跟着正文走**：编号的登记位是各正文文件里那行 `## E<n> 简称 —— 状态` 标题；
  索引表**不是**登记位。文件名用 `01-` 开头而不是字母加数字的形式，与决策拆分同一个理由——那种记号会被 doc-lint 判成裸引用。
- 门禁两个阶段跟着改路径：`10-kb-rot.sh` 的实验存在性与状态同步检查改扫 `experiments/`，
  `40-results-cited.sh` 的双向检查改成「索引 + 全部正文」一起扫。


