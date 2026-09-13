# E151 第六次跑：跑前登记（2026-09-13，用户定向做 C319 的计数、C146 的两条断言、C243 的判别力自证——实现还没有，先在模型里做成会红的形态）

三笔账要的检查都落在实现上（提交路径、checker、崩溃点重放），今天没有被测对象。第六次跑把它们各做成 E151 装置里一条会红的量，实现起来时照抄口径；真实系统里的检查另登记在 verification-build.md。

**臂 `compact_r_protected` / `compact_r_unprotected`（C146 两条断言与 C243 自证的载体）**：用户数据按 D3 已定项 8 落最低空槽；每轮一条整理意图——挑占用最少的非空段当 `R`（不挑 bump 的开放段），把 `R` 里的活节点全部搬走（意图完成），搬走的目的地与用户数据的落点都排除 `R`（D26 已定项 1 第 3 条，protected）或不排除（unprotected，判别力自证那一半）；`R` 的槽下一轮回到空闲集合，那时看 `R` 是不是全空（停机谓词 `全空段数(现在) − segs0 ≥ 1` 在这个模型里的形态）。

| # | 量 | 坐实 | 推翻 |
|---|---|---|---|
| Q1（C146 ①） | `moves_landed_in_region`：本批整理自己写出的落点落在 `R` 里的次数 | protected 四格恒 0，unprotected 四格 > 0 | protected 任一格 > 0，或 unprotected 任一格 = 0（自证没有判别力） |
| Q2（C146 ②） | `fallback_policy_mismatches`：用户数据的落点与「最低空槽、排除开放段与 `R`」这条政策函数不一致的次数 | protected 四格恒 0，unprotected 四格 > 0（`R` 含最低空槽时它就落进去） | 同上 |
| Q3（C243 自证） | `intents_completed_with_region_empty`：意图完成、`R` 释放之后 `R` 全空的次数 | protected 四格 > 0，unprotected 四格 = 0（净增量塌向 0） | protected 任一格 = 0，或 unprotected 任一格 > 0 |
| Q4（C319） | `request_order_violations`：同一请求内相邻 key 后一个先到达的次数 | `arrival_by_request` 两组负载恒 0，`arrival`（均匀置换）runs8 上 > 0、uniform 上 0（每个请求只有一个 key） | 任一不成立 |

**只报不判**：整理释放出来的 `R` 下一轮被最低空槽的用户数据吃掉多少（`empty_peak_after_drain`、期末全空段）——那是 C146 ① 后半句「刚腾空的段被下一次分配立刻吃掉」，归 D26 已定项 1，本轮只给轨迹。

**作废条款**：任一常设单测红、变异表有一条没被抓 ⇒ 整轮作废重跑。公共格与第五次跑逐字同数，否则装置改坏。
