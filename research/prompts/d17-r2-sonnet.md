你是 singlefs 三方论证的**正推腿**。立场：从已定条款、登记表与 E142（第一个事务的干跑） 第四次跑的源码出发，把候选条款甲′**一步步重推一遍**，每一步写明推自哪条条款或源码第几行；推不出来的地方明说推不出来，不许补。

## 先读

1. 背景材料：`research/prompts/_d17-r2-background.md`（正文 + 小节清单 + 附录，附录是从 kb 整段抄的条款，引用时整行抄）。第一轮的判决在 `research/prompts/d17-r1-main-verification.md`。
2. 自己去查：
   - `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 的 `fn mkfs`、`fn warm_up`、`fn publish_first_file`、`fn split_into_segments`，以及单测 `layer0_state_count_is_262162_with_zero_violations`
   - `research/results/e142-first-txn-dry-run-2026-09-13-warmup.out` 的 `name=warm_up`、`name=write_list`、`name=layer0` 三行
   - `.claude/kb/layout/01-first-txn.md` 零与八两节
   - `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（切换 / 回退各写什么）

## 要回答的（按材料第三节的判据编号）

- **4-1（最值钱）**：登记表每一行与源码逐段对一遍：mkfs 的 m1/m2/屏障/m3×3/m4/屏障，暖机的屏障/记录/屏障/根/超级块，事务的单元/屏障/记录/屏障/根/超级块；相邻路径之间没有屏障那句对不对。
- **4-2**：候选判据下今天是不是 1；「逐路径读」怎么在条款文本里堵死。
- **4-3**：切换 / 回退标预想，条款能不能写；条款里那句怎么写。
- **4-4**：门禁形态过不过「改一段切法由绿转红」；kb 表与源码常量之间谁盯。
- **4-5**：与 D13「一」、D17 腰线、D12 要还的债、C220 有没有逐字矛盾。
- 最后一句：甲′ / 乙 / 丙你判哪一个成立，依据是上面哪几格；若判甲′，把条款逐字写出来（可以改措辞，但改了哪里要标出来）。

## 交付

- **分段写**：把报告写进 `research/prompts/d17-r2-sonnet-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（文件已存在就报错停下，不许覆盖），后面的段追加。回复里只写几句要点。
- 引条款**整行抄**；现查的写明命令与行号。
- 不许用「本条」「上文」这类指代，每一格自足。
