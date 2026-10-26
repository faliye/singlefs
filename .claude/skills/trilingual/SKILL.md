---
name: trilingual
description: 改 singlefs 里登记为三语的文件（仓根 README、briefs/ 下的简报、.claude/kb/ 下的全部文档）时用它——中文原文改了，英语与日语两份怎么同步、用词照哪张表、提交前门禁 doc-text 的 trilingual 六格判什么。
---

三语文件一组三份：`<名>.md` 是中文（默认），`<名>.en.md` 是英语，`<名>.ja.md` 是日语。哪些文件受管登记在 `.claude/trilingual-documents`（README、`briefs/*.md`、`.claude/kb/` 下的全部 `.md`）。门禁 doc-text 的 trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission 六格判，判法在 `.claude/gate.d/lib-trilingual.py` 文件头。

## 什么时候用

- 改、新建、删掉一份登记过的中文文件（kb 的决策页、实验页、欠账表、里程碑、变更史这类都算）。
- 写回员、回扫员、实现员交回的改动碰了 `.claude/kb/`，暂存之前。
- 门禁 doc-text 的 trilingual 某一格红了。

## 改一组的做法

1. 先改中文那份。英语、日语两份在同一次改动里跟着改：中文改了哪几段，两份译文改对应的那几段；中文新建，两份译文同时新建；中文删掉，两份一起删。还没有译文的组，碰了中文那份就要把两份译文补上（新建，整份译）。中文那份没动、只改译文（补译、修译名）的，单改那一份就行，不要求三份同改。
2. 结构三份逐项相同：编号集合（D、E、C、I 编号）、链接目标集合、各级标题个数、表格行个数、围栏代码块个数。不增删段落、不合并表格、不把表改成段落。
3. 顶上第三行是语言切换行，三份写法照已有的：中文 `**中文** · [English](<名>.en.md) · [日本語](<名>.ja.md)`，英语 `[中文](<名>.md) · **English** · [日本語](<名>.ja.md)`，日语 `[中文](<名>.md) · [English](<名>.en.md) · **日本語**`。
4. 用词照 `.claude/kb/translation-glossary.md`：中文列的串出现在中文那份里，英语列的串就要出现在英语那份里（不分大小写），日语列的串就要出现在日语那份里。新术语先加进术语表再用。编号的简称在英语、日语版里写成 `E<编号> (<译名>)`，半角括号。
5. 链接目标与中文那份指同一个文件：指别的三语文件时，译文可以链中文那份 `<名>.md`，也可以链同一种语言的 `<名>.en.md` / `<名>.ja.md`（判法比之前把 `.en.md`、`.ja.md` 归成 `<名>.md`）。
6. 几个会话共写一份文件时，三份都用 `scripts/kb/replace-once.py`、`scripts/kb/insert-row.py` 定点改，不整份重写。

## 子 agent 改了 kb 之后

写回员、回扫员这类 agent 的定义里没有三语这一步，它们交回的 kb 改动只有中文。主 agent 暂存之前：

1. `git diff --name-only` 与 `git status --short` 列出这一批碰到的登记过的中文文件。
2. 条目少的主 agent 自己译；多的派 `clerk`（`model: opus`）起草，派发提示给：这几份中文文件与它们这一批的 diff、术语表路径、这一节「改一组的做法」；clerk 不改 kb，交 `scripts/kb/replace-batch.py` 能吃的规格，主 agent 先 `--dry-run` 再照规格写进两份译文。
3. 译文改完再暂存，三份一起进暂存区。

## 判一次

扫仓的门禁只在提交时跑；暂存之后、跑整轮之前要单看三语六格，照 `.claude/skills/gate/staged-gate.md` 带前缀与范围声明只点这几格：

```bash
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的 trilingual 六格; 范围=6 格; 为什么=暂存了三语文件，整轮之前先看三语六格' SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/doc-text.sh --check trilingual-files,trilingual-structure,trilingual-changed-together,trilingual-glossary,trilingual-untranslated,trilingual-omission
```

| 红的格 | 怎么办 |
|---|---|
| trilingual-files | 缺哪份补哪份；登记表的行照它的出路改 |
| trilingual-structure | 按它列的那一项（编号、链接、标题、表格行、围栏）把译文对齐中文 |
| trilingual-changed-together | 它点名的那一组（改了中文那份、没碰全两份译文的），把没碰的译文照中文的改动改上；只改译文的组不报 |
| trilingual-glossary | 它点名的那一组，把译文里的词换成术语表里的译名；术语表本身写错了照它的出路改表 |
| trilingual-untranslated | 它点名的那一行照中文那一份同一行整行译（写着「与中文逐字相同」的是照抄没译的行，照它点名的中文那一行译）；反引号段、路径、链接目标逐字照中文，要原样留的中文原话放进「」或反引号；整组不该判的（术语表这类）照它的出路登记进 `.claude/gate.d/trilingual-untranslated-exempt.tsv` |
| trilingual-omission | 它点名的那一段（内容量比过短的：漏了一段或压成摘要；过长的：多半对错了段），照中文那一份同一段整段补译或重译，不加不减；「这一节段数与中文不同」的照中文那一节逐段对齐；编号行配不上的补上缺的那一行或删掉多出来的。比值与门槛在 `.claude/gate.d/lib-trilingual.py` 文件头「判法细节」的 omission 一段 |
