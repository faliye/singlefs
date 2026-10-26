#!/usr/bin/env bash
# 日语压成了摘要：README 日语那份段数、列表项数、表格行数都与中文相同，标题也对得上，残留中文、术语都查不出来，
# 但「结果」一节的两段与两个列表项只留了一句大意（照 E178 英语曾经整页压过的样子），内容量比都不到门槛，逐段点名判红；
# 一个表格行压得只剩不到门槛一点点（门槛那一档）。英语那份逐段译全。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本'
put README.md '# 样本' '' '**中文** · [English](README.en.md) · [日本語](README.ja.md)' '' \
  '## 结果' '' \
  '第一段量的是删除判定的设备读次数：三档负载下每次删除恒为零点零零零零一次设备读，引用计数一侧在两倍与八倍两档跳到零点九九八与一点七五一，差了五个数量级。' '' \
  '第二段换成稀疏旁表做辅助：主表照旧，旁表只记被引用两次以上的块，旁表的大小随快照个数线性增长，十六个快照时占主表的百分之三。' '' \
  '- 第一个列表项：装置连跑五遍，产物的校验和逐字节相同，说明没有隐藏状态，但不说明统计上稳定。' \
  '- 第二个列表项：变异表二十二条全抓，只说明写了的断言都在跑，不说明没写的断言不缺。' '' \
  '| 档 | 读次数 | 说明 |' '|---|---|---|' \
  '| 八倍 | 一点七五一 | 引用计数在这一档要多读一次索引节点，因为旁表的那一页常被挤出缓存 |'
put README.en.md '# Sample' '' '[中文](README.md) · **English** · [日本語](README.ja.md)' '' \
  '## Results' '' \
  'The first part measures device reads for the deletion judgment: across the three load tiers every deletion costs a constant 0.00001 device reads, while reference counting jumps to 0.998 and 1.751 at the two-times and eight-times tiers, five orders of magnitude apart.' '' \
  'The second part adds a sparse side table: the main table stays as it was, the side table records only blocks referenced at least twice, and its size grows linearly with the number of snapshots, reaching three percent of the main table at sixteen snapshots.' '' \
  '- First list item: five consecutive runs of the apparatus produce byte-identical artifact checksums, which shows there is no hidden state but not that the result is statistically stable.' \
  '- Second list item: all twenty-two rows of the mutation table are caught, which only shows that the assertions that were written are running, not that no assertion is missing.' '' \
  '| Tier | Reads | Note |' '|---|---|---|' \
  '| Eight times | 1.751 | Reference counting reads one more index node at this tier, because the side-table page is often pushed out of the cache |'
put README.ja.md '# サンプル' '' '[中文](README.md) · [English](README.en.md) · **日本語**' '' \
  '## 結果' '' \
  '第1部：削除判定はほぼ読まない。' '' \
  '第2部：疎なサイドテーブルを加えた。' '' \
  '- 第1項目：5回とも同一。' \
  '- 第2項目：22件すべて捕捉。' '' \
  '| 段 | 読み取り回数 | 説明 |' '|---|---|---|' \
  '| 8倍 | 1.751 | 参照カウントはこの段で索引ノードを一回多く読む（サイドテーブルで） |'
