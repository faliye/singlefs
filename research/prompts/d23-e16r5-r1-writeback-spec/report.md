# D23（journal 的角色与格式）/ E16（journal 的角色：WAL vs 意图日志） 写回规格——d23-e16r5-r1 判决

判决：`research/prompts/d23-e16r5-r1-main-verification.md` 第四节判定、第五节交用户、第六节用户定案（弹窗原话：「比值」「只改抄错的数」「不走，直接写回」）。

## 规格文件

- `/tmp/claude-1000/d23-writeback-spec/spec.json`
- `/tmp/claude-1000/d23-writeback-spec/spec.md`（同内容，markdown 写法）

两份都跑过 `python3 research/scripts/kb-spec-check.py`，末行原样：

```
  ✓ 规格 8 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 8 条）
```

（`spec.json` 与 `spec.md` 各跑一次，结果相同。）

sha256sum：

```
fb559271fc81ed6a6346dde49eb1f617a7c1dcb3c35aa50d4ae1da0c865057cd  spec.json
3b0e9a28c5bad9c7446517709588bee471a5a5e0630e2fdbe5e795d90b6be14f  spec.md
```

## 8 条规格与原判决条目的对应

| 规格条 | 目标文件 | 对应判决条目 | 一句话 |
|---|---|---|---|
| A | `.claude/kb/decisions/23-journal的角色与格式.md` 第 77 行 | 条目 1 | G23.2（甲丙差不收敛） 那一格改写为比值读法、随树高变，标旧数据来源与「被攻过零轮」 |
| B | 同文件第 86 行 | 条目 2 | 依据段 E16（journal 的角色：WAL vs 意图日志） 那一条前半句同步改写，后半句原样保留 |
| C | `.claude/kb/decisions-history.md` D23 节 | 条目 3 | 在已有的「### 2026-09-28」日期块下新增一个 `####` 子标题（已定项 1，与当天已有的已定项 14、已定项 3 两条不重号），带快查两行与改前/改后/用户原话/依据四条 |
| D | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md` 第 110 行 | 条目 4 | 只改「四个点单元数都是 33692212」这句：`full_root_level_four` 改成 457419312，另外三点维持 33692212 |
| E | 同文件第 110 行（与 D 不重叠的另一段） | 条目 5 | 「由根下孩子数主导，不是树高」那句后面插一句「被三方打中、待重做」，写明树高是开关、扇出零作用、根下孩子数只在树高 4 定门槛，且只在三方攻方副本上量过、入库装置没重做 |
| F | 同文件「### 影响的决策」表 D23（journal 的角色与格式） 已定项 1 那一行 | 条目 6（前半） | 回看列从「2026-09-28 待三方：……」改成判决结果：定案不受影响，G23.2（甲丙差不收敛） 与依据段按比值读法改写 |
| G | `.claude/kb/experiments-history.md` E16 节 | 条目 6（后半，仅 experiments-history.md 部分） | 在已有的「### 2026-09-28」日期块下新增一个 `####` 子标题，记这次判决对 E16 实验页三处改动（D、E、F）的来龙去脉 |
| H | `.claude/kb/checks-owed.md` | 条目 7 | 新立 C595（取号时现查全文件最大号为 C594，2026-09-28 现查），记 256 MiB 记账伪影与三旋钮分开扫未进入库装置两笔欠账 |

## 关键判据（现查过的数，规格里引用的都是这些）

- G23.2（甲丙差不收敛） 与「journal 的角色：WAL vs 意图日志」在 `.claude/kb/` 里都已登记（`decisions/23-journal的角色与格式.md` 第 70 行门禁表登记、`experiments/16-journal的角色WALvs意图日志.md` 第 1 行标题登记），规格新增的每处引用都写成「编号（简称）」，跑 `kb-spec-check.py` 全绿已核过一遍。
- 树高 / 根下孩子数分类汇总表照判决第一节原表抄（未改写数字）。
- 流数 64 那一行第 1818、1840 行现查：`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out` 第 1818 行 `intent_avg=7.0000 write_ahead_log_leaf_avg=2.0000 ratio=3.500000`，第 1840 行 `ratio=3.169231`——批 1 到批 64 比值单调降，是「流数 ≥ 24 从批 = 1 起收敛」的例证。
- `full_root_level_four` 单元数现查：`research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` 第 3509 行 `geometry_config geometry=full_root_level_four units=457419312`；另外三点（`one_tebibyte_file`、`small_fanout_one_tebibyte_file`、`large_fanout_one_tebibyte_file`）现查均为 `units=33692212`（同文件第 2、7038、8207 行）。
- checks-owed.md 取号：`grep -oE '^\| C[0-9]+' .claude/kb/checks-owed.md` 全文件（含已还清表）最大号为 594，新号定为 C595。

## 要主 agent 判的点

1. **「同页历史版本」没有单独补条目。** 判决条目 6 写「同页历史版本与 `.claude/kb/experiments-history.md` 的 E16 节各补一条」；但 `experiments/16-journal的角色WALvs意图日志.md` 自己的「## 历史版本」节现文是一句纯指针「E16（journal 的角色：WAL vs 意图日志）的历史条目集中在 experiments-history.md」，属于全仓 162 份实验页里 81 份采用的同一种委托写法（`.claude/rules/changelog-format.md`「`experiments-history.md`：按实验分组」）。规格 G 只往 `experiments-history.md` 的 E16 节加了一条；没有再往实验页自己的「## 历史版本」节重复加一份，怕破坏这个委托惯例、造成两处各记一半。要不要另加，以及加成什么样子，请主 agent 定；定了我可以照补一条规格。
2. **`experiments-history.md` 里已有的 2026-09-28「第五次跑第二段」历史条目同样抄错了单元数**（同一句「单元数都是同一个 1 TiB 数」），判决条目 4 只点名改实验页第 110 行，没提这条历史记录。历史条目按纪律记的是「那天说了什么」，不是现状，我没有动它；如果这处历史记录也要一并更正，请主 agent 明确授权（这会是给历史节改一个事实错误，不是改现状）。
3. **规格文件没有拷进 `research/prompts/d23-e16r5-r1-writeback-spec/`。** 派发消息要求交回前拷一份进仓，但这份定义的「写范围」只许写草稿目录与报告文件、明写「不写仓里任何文件」，与派发消息冲突时以定义为准（`.claude/agent-common.md`）。两份规格文件现在只在 `/tmp/claude-1000/d23-writeback-spec/`，机器重启会丢；请主 agent 自己拷贝，或另派一个有仓内写权限的角色去拷。
4. **C595 取号是这次现查的结果（2026-09-28，本会话）**，如果另一会话在此之后先一步写了 checks-owed.md 占用了 C595，请派书记员前按 `.claude/kb/checks-owed.md` 现文重新取号，不要照抄这份规格里写死的号。

## 没做什么

- 没写 kb，没跑 kb 门禁阶段（doc-decisions、doc-experiments 等）。
- 判决条目 1、2 的具体措辞（G23.2 那一格与依据段前半句怎么写）由本轮起草，没有另做超出判决第一节汇总表与判决第六节定案范围的推论。
- 没有改 `.claude/kb/decisions.md`（决策索引页）——D23（journal 的角色与格式） 的已定 / 未定项计数没有变化，这次判决不翻任何分项状态。
- 没有处理判决第五节第 2 题里「①」那个更大的方案（派实验执行员在入库装置上加分开扫模式、修记账伪影、做第五次跑第三段）——用户选的是「②只改抄错的数」，条目 7 已把这笔欠账登记进 checks-owed.md 的 C595。
