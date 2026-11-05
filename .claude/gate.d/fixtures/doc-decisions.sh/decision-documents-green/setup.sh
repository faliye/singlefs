#!/usr/bin/env bash
# 十三格的绿样本并在这一棵树里，这里先改编号、建 lib.rs、再造 git 历史与工作区改动：
# ① 样本在仓里用不冲突的编号存放（doc-lint 全仓查「一个编号只许一处登记位」），
#    跑的时候才按文件名次序改成 D1–D21 —— 格「kb 形状」第 5 段要求决策编号从 D1 起连号，
#    格「冻结层归属登记表」按文件名认 D15（四层图）与 D18（单元类登记表）。
# ② 格「冻结层归属登记表」从 crates/singlefs-format/src/lib.rs 现读树 ID 常量；仓里不放 .rs，这里造一份。
# ③ 格「未定项被别处定了」：第一次提交两条决策，D10 的未定项点名 D11；第二次提交 D11 定案（标题改成「—— 已定」、
#    分项挪进已定项、索引行跟着改），**同时**回头在 D10 那条未定项上补一句点名 D11 的复核 ⇒ 未定项比对方的状态行新，本该判绿。
# ④ 格「定了新东西没回看同文件未定项」：工作区里（不提交）给 D12 新增已定项 2（「### 已定项」索引表一行与分项定案标题「#### 已定项 2：丁问题」；
#    这一格只认首行登记标题的「—— 已定」与「已定项 N：…」，光秃秃的「### 已定（…）」不算），同时在它那条未定项上补一句复核、
#    索引表那一行的状态列与进度列跟着改成「已定 1 项 / 未定 1 项」「已实现 2 项 / 未实现 0 项」⇒ 本该判绿。
# decisions.md 的分项清单生成块存的是 ③ 定案与 ④ 新增已定项 2 之后的样子（格「决策分项清单与正文同步」只判工作区）。
set -e
while read -r stored final; do
  find .claude/kb records -name '*.md' -print0 | xargs -0 sed -i -E "s/D${stored}([^0-9]|\$)/D${final}\\1/g"
done <<'NUMBERS'
104 1
91 2
102 3
103 4
183 5
184 6
187 7
185 8
451 9
98 10
99 11
100 12
1602 13
1608 14
1615 15
1619 16
1621 17
1618 18
210 19
312 20
317 21
NUMBERS
mkdir -p crates/singlefs-format/src
cat > crates/singlefs-format/src/lib.rs <<'RS'
pub const TREE_IDENTIFIER_EXTENT: u64 = 11;
pub const TREE_IDENTIFIER_INODE: u64 = 12;
pub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;
pub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;
pub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;
pub const TREE_IDENTIFIER_LIVELIST: u64 = 16;
pub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;
pub const TREE_IDENTIFIER_DEADLIST: u64 = 18;
pub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;
pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 19;
RS
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
# 使用者配了彩色输出：格「定了新东西没回看同文件未定项」自己的 git diff 要带 --no-color --no-ext-diff，不然 hunk 头认不出来（第三轮三方 T5-e）
git config color.ui always && git config color.diff always
GIT_COMMITTER_DATE="2026-08-31T00:00:00" GIT_AUTHOR_DATE="2026-08-31T00:00:00" sh -c 'git add -A && git commit -qm base'
printf '%s\n' '## D11 戊样本 —— 已定' '' '### 已定项' '' '| # | 分项 | 定案 |' '|---|---|---|' \
  '| 1 | **乙问题** | 取甲 **状态：已定。** **进度：未实现。** |' '' '## 历史版本' '' '无。' > .claude/kb/decisions/11-戊样本.md
sed -i 's/^| D11（戊样本） | 已定 0 项 \/ 未定 1 项 |/| D11（戊样本） | 已定 1 项 \/ 未定 0 项 |/' .claude/kb/decisions.md
sed -i 's/那边不定它就定不了/那边不定它就定不了（2026-09-01 复核 D11：它定的是乙问题，这一项仍然开着）/' .claude/kb/decisions/10-丁样本.md
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm settle'
decision=.claude/kb/decisions/12-己样本.md
awk '/^### 未定项$/ { print "### 已定项"; print ""; print "| # | 分项 | 定案 |"; print "|---|---|---|"; print "| 2 | **丁问题** | 取甲 **状态：已定。** **进度：已实现。** |"; print ""; print "#### 已定项 2：丁问题"; print ""; print "取甲。"; print "" } { print }' "$decision" > "$decision.new" && mv "$decision.new" "$decision"
grep -q '^#### 已定项 2：丁问题$' "$decision" || { echo "setup：没插进已定项 2 的标题" >&2; exit 1; }
sed -i 's/两条出路都还没选/两条出路都还没选（2026-09-02 复核过，仍然开着）/' "$decision"
grep -q '两条出路都还没选（2026-09-02 复核过，仍然开着）' "$decision" || { echo "setup：没补上未定项的复核注" >&2; exit 1; }
sed -i 's/^\(| D12（己样本） | \)已定 0 项 \/ 未定 1 项\( | .* | \)已实现 1 项 \/ 未实现 0 项 |$/\1已定 1 项 \/ 未定 1 项\2已实现 2 项 \/ 未实现 0 项 |/' .claude/kb/decisions.md
grep -q '^| D12（己样本） | 已定 1 项 / 未定 1 项 | .* | 已实现 2 项 / 未实现 0 项 |$' .claude/kb/decisions.md || { echo "setup：D12 的索引行没改成已定 1 项、已实现 2 项" >&2; exit 1; }
