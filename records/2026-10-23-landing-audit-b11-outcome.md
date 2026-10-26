## 按结局桶（2026-10-22 起、到 2026-10-23 为止，280 个，折算合计 1165.99 百万）
桶                                        个数     花费占比  说明
written-landed                          109    51.7%  写了、不低于 70% 的行进了提交或工作树
written-partial                           8     3.4%  写了、30% 到 70%
verdict-archived                         44    18.9%  三方腿、交回点名的 -output.md 在仓、同轮有判决
three-way-report-or-verdict-missing       2     0.6%  三方腿、交回点名的 -output.md 不在仓或同轮没有判决
report-used-by-main                      99    22.0%  只交报告、交回后主 agent 点到
report-used-by-later-sub                  0     0.0%  只交报告、只有后来的子 agent 点到
report-unreferenced                       1     0.3%  只交报告、没人点到
written-overwritten                       7     1.4%  写了、低于 30%、主 agent 点到过
written-unused                            0     0.0%  写了、低于 30%、没人点到
cut-by-usage-limit                        0     0.0%  没交回、用量撞顶
cut-by-api-error                          0     0.0%  没交回、API 错
stopped-salvaged                          9     1.6%  没交回、被停掉、后来有人点到
stopped-abandoned                         0     0.0%  没交回、被停掉、没人点到
in-flight                                 1     0.1%  没交回、还在跑

## 并成调度记录「按结局分」那张表的五行
行                                               个数     花费占比
写了东西、不低于 70% 的行进了提交或工作树                    109    51.7%
写了东西、30% 到 70%                               8     3.4%
三方腿                                         44    18.9%
只交报告、交回后主 agent 有动作点到                       99    22.0%
看起来没效果的几桶                                    8     1.7%
其余（three-way-report-or-verdict-missing、cut-by-api-error、stopped-salvaged、in-flight）    12     2.3%

## 按类型 × 结局桶（个数，花费占全部的比例）
  clerk                     report-used-by-main                      24     5.4%
  clerk                     written-overwritten                       3     0.7%
  clerk                     written-landed                            3     0.5%
  clerk                     written-partial                           1     0.2%
  experiment-designer       written-landed                            4     1.0%
  experiment-designer       report-used-by-main                       3     0.6%
  experiment-runner         written-landed                           11     5.9%
  experiment-runner         report-used-by-main                       3     0.7%
  experiment-runner         written-overwritten                       1     0.4%
  experiment-runner         written-partial                           1     0.4%
  implementation-writer     written-landed                           33    23.0%
  implementation-writer     stopped-salvaged                          3     0.8%
  implementation-writer     written-partial                           1     0.8%
  implementation-writer     report-used-by-main                       1     0.1%
  investigator              report-used-by-main                       9     1.6%
  investigator              written-partial                           3     1.3%
  investigator              written-landed                            1     0.2%
  kb-scribe                 report-used-by-main                      27     7.3%
  kb-scribe                 written-landed                            8     1.4%
  kb-scribe                 stopped-salvaged                          4     0.2%
  sweep                     report-used-by-main                      18     2.3%
  three-way-attack          verdict-archived                         22     8.4%
  three-way-attack          three-way-report-or-verdict-missing       1     0.3%
  three-way-defense         verdict-archived                          8     3.7%
  three-way-defense         three-way-report-or-verdict-missing       1     0.4%
  three-way-forward         verdict-archived                         14     6.8%
  tooling-writer            written-landed                           49    19.8%
  tooling-writer            report-used-by-main                      14     4.0%
  tooling-writer            written-overwritten                       3     0.3%
  tooling-writer            stopped-salvaged                          2     0.6%
  tooling-writer            written-partial                           2     0.7%
  tooling-writer            report-unreferenced                       1     0.3%
  tooling-writer            in-flight                                 1     0.1%

## 描述带返工字样（REWORK_WORDS，词表在文件头）的派发：44 个，占个数 15.7%，占花费 14.2%
## 没交回的 10 个，按截断原因：停掉 9、还在跑 1
  ✓ 查了 280 个子 agent（`--since 2026-10-22`、`--until 2026-10-23`（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛），逐个归了结局桶；这份输出也落进了 records/landing-audit-b11-outcome-2026-10-23.txt（排他新建，回读与标准输出逐字相同）
