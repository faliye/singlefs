# kb 写回规格起草报告（第六批）

## 产出

- `/tmp/claude-1000/kb-batch6-drafter/spec-jia.json`（7 条，任务 1+2：C577 加屏障、C554 乙-配置续实现落地）
  sha256: ae57c2a26d45f2c57d67e33948c7ae39e8d94e959d639b5937bb7e0d15a1b5a7
- `/tmp/claude-1000/kb-batch6-drafter/spec-jia.md`（同内容 markdown 镜像）
  sha256: 68e5a3180c8bbd7fbec943c7e0a23ca149d433d58340e743ded693e8f0a03515
- `/tmp/claude-1000/kb-batch6-drafter/spec-yi.json`（8 条，任务 3+4：checker/读者一侧条款、invariants 射程扩充）
  sha256: ab9d83ec28bbf476b6a0d72c16e6693168d4b481d52d65405a6e4142cc220f44
- `/tmp/claude-1000/kb-batch6-drafter/spec-yi.md`（同内容 markdown 镜像）
  sha256: 356d8fa19452301735ff57c7c81c1e6ad8ceaebc8d3b1f032548f20c59ed8fa3

`kb-spec-check.py` 原样末行：

```
$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch6-drafter/spec-jia.json
  ✓ 规格 7 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 7 条）

$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch6-drafter/spec-yi.json
  ✓ 规格 8 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 8 条）

$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch6-drafter/spec-jia.md
  ✓ 规格 7 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 7 条）

$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch6-drafter/spec-yi.md
  ✓ 规格 8 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 8 条）
```

每条旧串另跑过 `grep -cF`（用临时文件传模式，避开转义），全部恰好 1 次，命令与输出存在
`/tmp/claude-1000/kb-batch6-drafter/grep-verify.txt`（15 行，逐条列出）。

## 甲（7 条）：C577 加屏障 + C554 乙-配置续实现落地

1. `.claude/kb/decisions/16-发布语义.md` 已定项 7 **定案**：持久顺序末尾加一道屏障（系统配置槽 → 屏障），
   fsync 改成等系统配置轮换持久之后才返回；追加实现落点（待派，排实审 A3c 之后）与代价说明。
2. 同文件已定项 7 **射程「屏障口径」**：三个序点改四个，加屏障来历指回 C577 那条用例更早红的原因。
3. `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14「续」那句：实现状态从「实现在做」改「已交回并打上」，
   落点写清；加三句——回卷写带同一见证值不退 0、mkfs 字节不变、取号那一刻某块盘见证槽读不出就拒（实审 A3c 在做）。
4. `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11：「取号失败回卷写回 tail 0」改成
   「取号失败回卷写回取号那一刻的见证值」。
5. `.claude/kb/checks-owed.md` C577 行：「怎么拦」「前置」两栏改成现状——乙-配置续已落地但同一条用例仍更早红
   （抬 F 之前 I-7.4），加屏障用户已定、待派实现、排实审 A3c 之后。
6. `.claude/kb/decisions-history/2026-09.md` 新增（其十五）：D23 已定项 14、D18 已定项 11 的实现落地。
7. 同文件新增（其十四）：D16 已定项 7 加屏障。

## 乙（8 条）：checker / 读者一侧条款 + invariants 射程扩充

1. `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 9：补「读者遇格式版本不是 1 一律拒收」。
2. 同文件已定项 16：补「读者判固定结构槽距 / physical_block_size 界，越界即拒（checker 报违例、实现整池拒挂载）」
   ——处置按用户 2026-09-27 JST 14:0x 定的「整池拒」写，没写「这一槽不可择」。
3. `.claude/kb/decisions/09-加密.md` 已定项 10 射程末句：从「checker 那一半实现在做」改成现状
   「池级 checker 判单元头 29 字节全 0；池级码 1 / 码 3 与指针头部在 A3-checker-2 里做」。
4. `.claude/kb/decisions/15-格式冻结政策.md` 已定项 4 依据第 3 条：追加一句标注「今天的读者不核版本号」
   已过时，指回 D22 已定项 9。
5. `.claude/kb/invariants.md` I-2.4：射程从「单元头 29 字节」扩到「单元头与指针头部的加密预留位」
   （D19 已定项 3），checker 状态列补一句指针头部那一半还没落地。
6. 同文件 I-1.10：补「条目宽为 0 时条目数必须为 0，否则判该节点损坏」，checker 状态列同样补落地说明。
7. `.claude/kb/decisions-history/2026-09.md` 新增（其十七）：invariants.md I-2.4、I-1.10 射程扩充。
8. 同文件新增（其十六）：D22 已定项 9/16、D9 已定项 10、D15 已定项 4 四条一起收进一条变更史（同一份
   实审 A3-checker 报告、同一次主 agent 判断产生）。

依据段核查：本批全部改动都不在任何分项的「**依据**」段里增删引用的实验号（E-number）——D23 已定项 14 的
依据段（第 408–434 行）已在更早批次指到 E158（择根与修复四岔路），本批未再改这一段；其余各条改动的都是
**定案 / 射程** 段。`kb-spec-check.py` 的第④条（依据配对）因此在全部 15 条上都不触发，已用工具自身的判定
结果核过（两份规格都报「依据配对……都过」）。

## 要主 agent 判的点

1. D16 已定项 7 射程里第一条 bullet「『fsync 等根槽持久之后才返回』的射程」，标题与内容仍写旧的返回
   条件（根槽持久），与本批改过的新定案（fsync 等系统配置轮换持久之后才返回）字面不一致；派发提示只点名
   改「定案」与「屏障口径」两处，这条 bullet 没有改，要不要跟着改（连带影响它下面「新实例第一个根是单点」
   那段论证是不是也要挪一句），交主 agent 判，本批规格未处理。
2. D18 已定项 11 这处改动（回卷写从 tail=0 改成见证值）对「取号之前逐盘核带不带所选那一版」那道判据
   有一处读法变化：实现员报告第五节第 5 点已指出（推的、没写用例）——盘在核过之后、下一次挂载之前单元
   又坏了的窄窗口里，改前判「落后」去读单元，改后可能判「不落后」不读。要不要为这一格补条款或用例，
   实现员报告原句已写「交主 agent 定」，本批规格没有替它选。
3. D23 已定项 14 那句「系统配置没见证到的最新根，『续』落地之后由下一次挂载在自己的 c_见证 里看到」，
   实现员报告第五节第 5 点要求「这半句要不要改，请书记员对着 C554 的 Q1 现查」；本批没有改这半句
   （保持原样），是否需要改交主 agent 判。
4. 甲、乙两份规格都以决策变更史当前最上面那条标题行（其十三）当锚点插入新条目；两份谁先由书记员写、
   四条新条目（其十四～十七）最终在文件里谁在最上面，取决于写入顺序，不是严格按编号升序排列——这只是
   排版顺序，不影响每条内容与编号的正确性，不需要判，写在这里存档。

## 没做什么

- 没写 kb，没跑 kb 门禁阶段（`21-decision-items-sync.sh`、`49-history-brief.sh`、`75-decision-experiment-links.sh`
  这几道由书记员写完之后跑）。
- 没判定案对不对，规格里的判断以判决与实现员报告为准，没有另做推论；四处「要主 agent 判的点」按原样列出，
  没有替主 agent 选。
- 没有为「加屏障代价另登记一个小实验量」「A3-checker-2」建立新的 checks-owed 号：派发提示没有给具体
  号位与措辞，没有代主 agent 现查空号并占号。
- 重型测试没有跑。
