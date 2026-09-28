# D23（journal 的角色与格式）/ E16（journal 的角色：WAL vs 意图日志） 写回报告——d23-e16r5-r1 判决

## 规格文件

- 原始（起草员产出，未改）：`/tmp/claude-1000/d23-writeback-spec/spec.json`、`/tmp/claude-1000/d23-writeback-spec/spec.md`（仓内同份副本 `research/prompts/d23-e16r5-r1-writeback-spec/`，均已由主 agent 拷入）。
- 本会话按主 agent 判的第②点，往其中一条（experiments-history.md 的 E16 节新条目）末尾加了一句，另存为 `/tmp/claude-1000/d23-writeback/spec.json`（8 条不变，只改第 6 条的 `new` 字段）；`kb-spec-check.py` 与 `replace-batch.py --dry-run` 都用的是这份改过的规格。

## 主 agent 已判的四点，逐点执行

1. E16 实验页自己「## 历史版本」节不另补条目——照办，没有改动那一节（现文仍是委托指针，未动）。
2. `experiments-history.md` 已有的「第五次跑第二段」历史条目同样抄错单元数——在新写的 E16 节条目末尾加一句「2026-09-28『第五次跑第二段』那一条里 full_root_level_four 的单元数 33692212 同样抄错，实为 457419312，以这一条为准」（`.claude/kb/experiments-history.md` 第 1129 行）；那条 2026-09-28 历史条目本身（第 1115–1121 行）一个字没改，按变更史纪律它记的是「那天说了什么」，不改历史。
3. 规格已由主 agent 拷进仓——现查确认 `research/prompts/d23-e16r5-r1-writeback-spec/` 已存在，内容与 `/tmp` 那份一致。
4. C595 取号——开工前现查 `.claude/kb/checks-owed.md` 全文件最大号仍是 C594，C595 未被占用，照规格写入。

## 执行

`research/scripts/replace-batch.py` 规格 8 条（改过第②点后）：

```
python3 research/scripts/kb-spec-check.py /tmp/claude-1000/d23-writeback/spec.json
  ✓ 规格 8 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 8 条）

python3 research/scripts/replace-batch.py --dry-run /tmp/claude-1000/d23-writeback/spec.json
✓ 8 处都命中得对（5 个文件），--dry-run 没写

python3 research/scripts/replace-batch.py /tmp/claude-1000/d23-writeback/spec.json
✓ 8 处替换，写了 5 个文件（改名换上新 inode），回读一致
```

写范围闸预检（每个目标文件各一次，均 exit=0，未附）。

决策正文里没有翻任何分项状态，没有动 `.claude/kb/decisions.md` 索引——D23（journal 的角色与格式） 的已定/未定项计数与「现状」句均未变（`decisions.md` 改前改后哈希相同），因此没有跑 `doc-decisions.sh --write`（触发条件「规格改了某条决策的现状」不成立）。第 3 步（`relabel-item.py`）不适用：本轮不翻状态。

## 逐条核对（git diff 全文核过，只含规格要求的改动）

- `.claude/kb/decisions/23-journal的角色与格式.md`：第 77 行 G23.2、第 86 行依据段 E16 前半句，逐字与规格「新串」一致。
- `.claude/kb/decisions-history.md`：D23 节 2026-09-28 日期块下新增 `#### D23（journal 的角色与格式） 已定项 1：……` 子标题（与当天已有的「已定项 14」「已定项 3」不重号），快查两行、改前/改后/用户原话/依据四条齐全。
- `.claude/kb/experiments/16-journal的角色WALvs意图日志.md`：第 110 行单元数改正（`full_root_level_four`=457419312，另三点=33692212）；第二段归因句插入「被三方打中、待重做」；「影响的决策」表 D23（journal 的角色与格式） 已定项 1 那一行回看改写。
- `.claude/kb/experiments-history.md`：E16 节 2026-09-28 日期块下新增 `#### 判决 d23-e16r5-r1：……` 子标题，末尾追加主 agent 判②点的更正句。
- `.claude/kb/checks-owed.md`：新增一行 C595（记账伪影与分开扫未进入库装置）。

## sha256sum（开工时 / 收尾时）

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/decisions/23-journal的角色与格式.md` | `5d81ea082dce1e5832d4dd4b1c2fa012abd4ccbfb87f8e523a3e04e15123df37` | `dc832559bdc2c44802aa0029e1f7c07807ffbffa4f67a699d4af94658bc99d78` |
| `.claude/kb/decisions-history.md` | `e5b0a228ba3eefff8adef2ecb0516767fe8ef4a9dada5f340695010679e447c6` | `01ab88ccc0cd35fe61b7b1190052cab955d087cd328970e6f93aed27ed2f67a9` |
| `.claude/kb/experiments/16-journal的角色WALvs意图日志.md` | `2a52d128bc2eb9ce01bb70f32f6b6569b354836ad25f11ccf69da5bd2791cf77` | `fa6962e833f5211831a2d0e834419542731e76d993a57b5c0ec994134a3d8cc9` |
| `.claude/kb/experiments-history.md` | `0defb97ba8b0e83700e1adce46fc0bfe31341e2caa5d07ad1528da4566bbded8` | `d10ebaef397b5a5c2df810dbfcc3380607b4e75c7310c5a4f7fdb6eb4656f306` |
| `.claude/kb/checks-owed.md` | `5d2965f2093a8ac07832ecc82e861868fe1ac05fcbacaeee2c527dda61d73940` | `fd1fb797f15283ffa9b61e7ec52fb5d0937893fba352d0ce777ad442375a9b8c` |
| `.claude/kb/decisions.md`（未动，只记对照） | `74e4250cffea57bdcd94882ea167063d6524a641acb0494de2e07b9be69939aa` | `74e4250cffea57bdcd94882ea167063d6524a641acb0494de2e07b9be69939aa` |

## 没做什么

- 没跑门禁阶段（`doc-decisions.sh` 等只读检查、`gate.sh` 全量或 `--staged`），照定义交给提交前的 `gate-triage`。
- 没跑 `doc-decisions.sh --write`：现状/项数未变，触发条件不成立。
- 没动 `.claude/kb/decisions.md` 索引，没翻任何分项状态。
- 没判定案对不对，规格外没加一个字；`.claude/kb/experiments/16-journal的角色WALvs意图日志.md` 自己的「## 历史版本」节按主 agent 第①点授权没有另补条目。
- 没提交。
