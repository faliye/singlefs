# shell-words-r3 正推腿甲（X1、X3）模型目录

- `tools/rerun.sh`：整份复跑。用法 `bash tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录> [--model <本目录>]`；冻结副本要有 `.claude/hooks/` 下五份被判钩子与 `r1.diff`、`fix.diff`、`fix2.diff`。
  它在 `--work` 里搭四棵树（now = 冻结副本五份；r2 = 打回 fix2.diff；r1 = 再打回 fix.diff；pre = 再打回 r1.diff），跑全部族与样本集，汇总写进 `--work/out/`，与本目录 `out/` 里登记的逐份比。
- `tools/myoracle.py`：真 bash 裁判，每条样本放进自己的进程组（`start_new_session` 即 setsid），结束与超时杀整组。`tools/feedhooks.py`：真钩子入口逐条喂。`tools/trans.py`、`show.py`、`probe.py`：迁移计数、判错列表、单条探针。
- `tools/mkx1_*.py`：X1 的样本造法（输出已拷进 `cases/`，`__W__` 是草稿目录占位）；`cases/cases-*.json` 里第一轮、第二轮两条腿的族清单是它们的 mkcases.py、fuzz2.py、fuzzhead.py 造出来的（只取清单）。
- `tools/x3*.py`、`x1sum.py`、`x8910.py`、`s2feed.py`、`bracecmp.py`、`smallchk.py`、`selftests.py`：X3 逐数重算与 X1 汇总。
- `oracle/bin/`：沙盒里的假 cargo、假 sudo（丢选项后照跑）、假 systemd-run（同）。`template/`：沙盒模板。
- `out/`：登记的汇总（x3fam、x3chg、x3pre、x3kf、x3s2、x3brace、x3small、x3selftests、x1、x8910）。
