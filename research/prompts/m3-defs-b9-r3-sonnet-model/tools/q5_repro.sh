#!/usr/bin/env bash
# Q5：判决没写去向的候选线，在 r1 / r3 冻结副本上现喂。每一段先打「## 段名」，再打原样输出（路径换占位在 rerun.sh 里做）。
# 用法：q5_repro.sh <仓根> <第三轮冻结副本> <工作根> <草稿目录>；只读冻结副本，写只落在 <草稿目录>。
set -uo pipefail
REPO="$1"; R3="$2"; WORKROOT="$3"; WORK="$4"
R1="${R1_FROZEN:-$WORKROOT/m3-defs-b9-r1/repo}"; R2="${R2_FROZEN:-$WORKROOT/m3-defs-b9-r2/repo}"
P="$REPO/research/prompts"; O1="$P/m3-defs-b9-r1-opus-model"; O2="$P/m3-defs-b9-r2-opus-model"; S2="$P/m3-defs-b9-r2-sonnet-model"
export PYTHONDONTWRITEBYTECODE=1
N="nice -n 19"
small_tree() {   # 源冻结副本 目标目录：只拷 .claude scripts records 与叫停登记/轮名材料
  local source="$1" target="$2" name
  mkdir -p "$target/research/prompts" "$target/research/results"
  rsync -a --exclude target --exclude .git "$source/.claude" "$source/scripts" "$source/records" "$target/"
  rsync -a --include='abandoned-rounds.tsv' --include='_*-body.md' --include='*-main-verification.md' --exclude='*' "$source/research/prompts/" "$target/research/prompts/"
  while read -r name; do mkdir -p "$target/research/prompts/$name"; done < <(cd "$source/research/prompts" && find . -maxdepth 1 -name '*-snapshot' -type d -printf '%f\n')
}

echo "## Q5-1 回车换行的派发提示：--check-prompt 与钩子 stdin 入口（r1 r2 r3 冻结副本）"
$N python3 -B "$(dirname "$0")/q5_crlf.py" r1="$R1" r2="$R2" r3="$R3" --work "$WORK/crlf"

echo "## Q5-2 heavy-thread-defaults：辩方 29 份样本对 r1 / r2 / r3 冻结副本的格（只列预期是「盲区 / 语义洞 / 误报」的样本是否被点名）"
for label in r1:"$R1" r2:"$R2" r3:"$R3"; do
  name="${label%%:*}"; root="${label#*:}"
  $N python3 -B "$O1/tools/z2_htd_cell.py" "$root" "$S2/feeds/s3/sample" > "$WORK/s3-$name.txt" 2>&1 || true
done
python3 -I - "$S2/feeds/s3/sample/EXPECT.tsv" "$WORK/s3-r1.txt" "$WORK/s3-r2.txt" "$WORK/s3-r3.txt" <<'PY'
import re, sys
expect = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
named = {}
for name, path in (("r1", sys.argv[2]), ("r2", sys.argv[3]), ("r3", sys.argv[4])):
    named[name] = set(re.findall(r"scripts/experiments/([A-Za-z0-9_]+\.sh)", open(path, encoding="utf-8").read()))
print("样本份数\t%d；点名份数 r1=%d r2=%d r3=%d" % (len(expect), len(named["r1"]), len(named["r2"]), len(named["r3"])))
for path, want in expect:
    f = path.split("/")[-1]
    if want.startswith(("hole", "green-semantic-hole", "red-false-positive")):
        print("%s\t预期=%s\tr1 点名=%s\tr2 点名=%s\tr3 点名=%s" % (f, want, *("是" if f in named[k] else "否" for k in ("r1", "r2", "r3"))))
PY

echo "## Q5-3 经自己的小函数取登记值（opus r2 的 make_tree 样本）对 r1 / r3 冻结副本的格"
for label in r1:"$R1" r3:"$R3"; do
  name="${label%%:*}"; root="${label#*:}"
  bash "$O2/feeds/s3/make_tree.sh" "$root" "$WORK/s3o-$name" > /dev/null 2>&1
  $N python3 -B "$O1/tools/z2_htd_cell.py" "$root" "$WORK/s3o-$name" > "$WORK/s3o-$name.txt" 2>&1 || true
  echo "$name\t$(grep -oE '[0-9]+ 处自己定' "$WORK/s3o-$name.txt" | head -1)\t$(grep -oE 'scripts/experiments/[a-z_]+\.sh:[0-9]+' "$WORK/s3o-$name.txt" | sed 's#scripts/experiments/##' | tr '\n' ' ')"
done | sed 's/\\t/\t/g'

echo "## Q5-4 重型测试闸 stdin 入口（辩方 s4 喂法）对 r1 / r3 冻结副本"
for label in r1:"$R1" r3:"$R3"; do
  name="${label%%:*}"; root="${label#*:}"
  echo "-- $name"; $N python3 -B "$S2/tools/s4_heavy_guard_feed.py" "$root" "$WORK/s4-$name" 2>&1 | cut -c1-120
done

echo "## Q5-5 叫停登记：前面补一句 + 按改名表真换一个词（r1 r2 r3）"
$N python3 -B "$(dirname "$0")/q5_rename.py" r1="$R1" r2="$R2" r3="$R3" --kb "$R3" --work "$WORK/rename"

echo "## Q5-6 豁免表里被钉的脚本挪位置（内容不变）：exempt-check 带基准"
$N python3 -B "$(dirname "$0")/q5_pathmove.py" r1="$R1" r3="$R3" --work "$WORK/pathmove"

echo "## Q5-7 缺行出路的 15 个形态（辩方 s2 喂法）对 r1 / r3 小树：钩子放拒与「定义读法」"
small_tree "$R1" "$WORK/small-r1"; small_tree "$R3" "$WORK/small-r3"
$N python3 -B "$S2/tools/s2_exit_matrix.py" "$WORK/small-r1" "$WORK/small-r3" "$WORK/s2" 2>&1 | sed -n '1,16p'

echo "## Q5-8 python3-markdown-it 有没有被声明（r3 冻结副本里点名它的文件；apt 手装名单里带 markdown 的行数）"
(cd "$R3" && grep -rIl 'markdown.it\|markdown_it' .claude scripts 2>/dev/null)
echo "apt-mark showmanual | grep -ic markdown = $(apt-mark showmanual 2>/dev/null | grep -ic markdown)"
echo "dpkg -l python3-markdown-it = $(dpkg -l python3-markdown-it 2>/dev/null | tail -n 1 | awk '{print $1, $2, $3}')"

echo "## Q5-9 doc-text 之内没经 shown_path 的输出点（r3 冻结副本行号）"
for p in '认不出的选项' '进不去项目根' '读不到禁用写法的登记'; do
  grep -nF "$p" "$R3/.claude/gate.d/doc-text.sh" | cut -c1-110
done
