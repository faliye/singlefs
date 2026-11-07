# 产物归档之后，正文引它写成一行指针

**这是 singlefs 的项目本地规则**，不在共享 SOP 里。共享规则在 `.claude/singlefs-ai-sop/rules/`；「引产物就整行抄」在共享的 `evidence-discipline.md`。

## 怎么做

- 产物归档出工作区之后，正文里那一行写成一行指针：`产物行见 git show <提交>:<路径> 第 N 行（name=<名>）`。一条指针指一行，不把几行并成一句。
- 指到的那一行要以登记的产物行前缀起头（形态 `<前缀> name=<名>`），不以它起头的指针不算引产物。
- 前缀登记在项目根的 `.claude/product-line-prefixes`，一行一个前缀，`#` 后写它是哪一类产物的行。

## 判法

按指针 `git show` 取那一行，核它存在、且以登记表里某个前缀起头。这道检查接在 `.claude/gate.d/` 里；没接之前记在 `.claude/kb/checks-owed.md`。
