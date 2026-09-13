你是 singlefs 三方论证的**反推腿**。立场：**假设候选条款甲′是错的**，去找它错在哪。第一轮你这条腿（Opus）打中了三处（`research/prompts/d17-r1-opus-output.md`），这一轮的材料按那三处改过；先核第一轮打中的每一处在这一版里有没有真被堵上，再找新的。**打不中就明说构造不出，不许造弱反例。**

## 先读

1. 背景材料：`research/prompts/_d17-r2-background.md`（正文 + 小节清单 + 附录）。第一轮：`research/prompts/d17-r1-main-verification.md` 与 `research/prompts/d17-r1-opus-output.md`。
2. 自己去查：
   - `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 的 `fn mkfs`、`fn warm_up`、`fn publish_first_file`、`fn split_into_segments`、`fn enumerate_layer0`（root_index 怎么取）
   - `research/results/e142-first-txn-dry-run-2026-09-13-warmup.out` 全部 68 行
   - `.claude/kb/first-txn-layout.md` 八（登记表）与零（写清单）
   - `.claude/kb/checks-owed.md` 的 C316、C220、C8 三行与已还清表里的 C313
   - `.claude/kb/decisions/22-单元原子性怎么合成.md` 未定项 6 与乙问（zoned 上固定结构能不能原地轮转）——第 5 问要用

## 要攻的

1. **攻 4-1 登记表**：逐段对源码。mkfs 三个根 FUA 各自成段对不对（`split_into_segments` 里 FUA 怎么切）；暖机的第一道屏障前面没有写、算不算空段；「相邻路径之间没有屏障」在 mkfs → 暖机之间成不成立（mkfs 末尾有屏障）。
2. **攻 4-2「今天 = 1」**：给一个按候选判据能数出 2 的读法，并判条款文本堵没堵死；或者构造今天这条线内部两条路径**必须**分成两个重放战役的理由（例如 mkfs 不在层 0 枚举里，G19）。
3. **攻 4-3 预想的行**：切换 / 回退按 D23 已定项 14 会不会多一种步骤或多一道序点（写实例表单元是不是「写单元」；回退行与中间实例行写在同一个单元里吗）。
4. **攻 4-4 门禁**：「改一段切法由绿转红」——E142 那条单测钉的是段长向量，改一段的**内容种类**（例如把一条记录换成一个单元）它红不红；kb 登记表改了一格而源码没改，谁红。
5. **攻 4-5 与第 5 问**：zoned 开线按候选条款数出几个类；D22 乙问未定（zoned 上超级块槽能不能原地轮转）会不会让「+1 还是 +2」今天无法判——那样「每开一条线重算」的输入就还是悬的。
6. **最强的一条**：若候选甲′该被更简单或更严的东西取代，说清是什么、凭哪些已定条款。

## 交付

- **分段写**：把报告写进 `research/prompts/d17-r2-opus-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（文件已存在就报错停下，不许覆盖），后面的段追加。回复里只写几句要点。
- 引条款**整行抄**；现查的写明命令与行号。
- 不许用「本条」「上文」这类指代，每一格自足。
