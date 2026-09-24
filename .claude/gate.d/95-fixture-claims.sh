#!/usr/bin/env bash
# gate-stage: 阶段头部声称的判别力样本必须真的存在；全仓文件名与样本目录里的日期不许是编的
#
# 判据：三条，任一条不成立判红。
#   ① 一个阶段的头部注释里写了 `fixtures/<它自己的文件名>`，那个样本目录就必须存在；
#   ② `.claude/gate.d/fixtures/` 下的每个目录都要对应一个存在的阶段文件——孤儿样本目录没有任何东西会跑它，
#      而目录摆在那里看着就像验过了；
#   ③ 样本目录里至少要有 `red` 或 `green` 之一，每个子目录都要有 `expect`：
#      空目录既骗过第 ① 条，又会被判别力自检整个跳过；
#   ④ 日期不许是编的——全仓**文件名**里的日期，以及 `fixtures/` 下 `.md` **正文**里的日期，
#      都要落在可能的区间里。判据不自己写，source 上游 lib.sh 的 `date_out_of_range`
#      （早于本仓第一个提交宽限 7 天、或晚于最晚时区的今天，都不可能是真发生过的事）。
#
# 第 ④ 条与前三条是同一个病理：**摆在那里看着是真的，其实是造的**。
# 前三条管「声称验过了而其实没验」，第 ④ 条管「日期看着是真的而其实是编的」。
# 实测（2026-09-23）：门禁样本里 8 个文件名与 32 处正文写着 2026-01-01 / 2026-01-02，
# 比本仓第一个提交（2026-08-26）早大半年，而当天带一个 records/2026-01-01-临时实测.md
# 跑 64 道阶段加 doc-lint，0 道点名它。上游 doc-lint 的 M 段只认正文里的
# `### 日期` 与「实测（日期）」两种形态，够不着文件名，也够不着
# `已定（日期）`、`—— 已跑（日期）` 这类装成本仓事实的写法。
#
# ⚠️ 第 ④ 条的正文那一维**只扫 fixtures**，不扫真 kb 正文：`date_out_of_range` 的下界假设是
# 「本仓的事不可能早于本仓第一个提交」，这只对说本仓自己事的日期成立。实测扫全仓正文得 52 处越界，
# 其中 20 处在 prior-art.md（别家项目的真实日期）、decisions-history（RFC 演进史 2015–2017）与
# checks-owed.md（C510（编造的日期没人拦） 引 2026-01-01 当例子）里，全部合法——射程放到真 kb 正文就是误判。
# 文件名那一维没有这个反例（现查 90 条带日期路径全部说的是本仓自己的事），所以扫全仓。
#
# 为什么：上游的判别力自检（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh`）把没有样本的阶段
# 列成「未自检」，这很好——但它只看目录在不在，不看**阶段自己怎么说**。一个阶段的头部逐字写着
# 「判别力：fixtures/<自己>/red 是一个……必须判红；green ……必须判绿」，而那个目录压根不存在时，
# 读脚本的人以为验过了，自检的名单里那一行又只是一句轻描淡写的「未自检」，两边对不上没有人会发现。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：`91-archive-past-rounds.sh` 的头部就是这么写的，
# 而 `.claude/gate.d/fixtures/91-archive-past-rounds.sh/` 不存在——它立项那天起就没验过判别力。
#
# ⚠️ 射程：判的是「声称与目录对不对得上」，判不了样本本身有没有判别力——那一层归
# `stage-selftest.sh`（红样本真判红、绿样本真判绿）。**没有声称、也没有样本**的阶段这一道不判，
# 它们由自检的「未自检」名单管着。
#
# 判别力：fixtures/95-fixture-claims.sh/red 摆一个假的 gate.d，前三条各犯一次，必须判红；green 都干净。
# 第 ④ 条的坏日期由 red/setup.sh 在临时仓里现造，不摆进仓里——摆进来它会被这一条自己扫到
# （C441（全仓清扫工具会吃掉自己的判别力样本））。setup.sh 里要 git init 再提交一个真实日期的提交，
# 否则 project_start_date 取不到值、下界那一支根本走不到，红样本就只证了上界。
#
#   bash .claude/gate.d/95-fixture-claims.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
LIB="$(cd "$(dirname "$0")/../.." && pwd)/.claude/singlefs-ai-sop/scripts/lib.sh"
python_rc=0
python3 - <<'PY' || python_rc=$?
import glob, os, re, sys

STAGE_DIR = ".claude/gate.d"
FIXTURE_DIR = os.path.join(STAGE_DIR, "fixtures")
stages = sorted(glob.glob(os.path.join(STAGE_DIR, "[0-9][0-9]-*.sh")))
if not stages:
    print(f"  ✗ {STAGE_DIR} 下一个 NN-*.sh 都没有")
    print("     → 怎么办：门禁目录搬了位置就同步改这个阶段里的路径；扫到 0 个阶段而报绿，与判过了一模一样。")
    sys.exit(1)

claimed_without_directory, hollow = [], []
claiming = 0
for stage in stages:
    name = os.path.basename(stage)
    head = open(stage, encoding="utf-8").read()
    claims = re.search(rf"fixtures/{re.escape(name)}", head) is not None
    directory = os.path.join(FIXTURE_DIR, name)
    if claims:
        claiming += 1
        if not os.path.isdir(directory):
            claimed_without_directory.append(f"{name}：头部写着 fixtures/{name}/…，而 {directory} 不存在")
            continue
    if not os.path.isdir(directory):
        continue
    kinds = [kind for kind in ("red", "green") if os.path.isdir(os.path.join(directory, kind))]
    if not kinds:
        hollow.append(f"{name}：{directory} 在，里面一个 red / green 都没有")
        continue
    for kind in kinds:
        if not os.path.isfile(os.path.join(directory, kind, "expect")):
            hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")

orphans = []
if os.path.isdir(FIXTURE_DIR):
    for entry in sorted(os.listdir(FIXTURE_DIR)):
        if os.path.isdir(os.path.join(FIXTURE_DIR, entry)) and not os.path.isfile(os.path.join(STAGE_DIR, entry)):
            orphans.append(f"{entry}：样本目录在，而 {STAGE_DIR}/{entry} 不存在")

failed = False
if claimed_without_directory:
    failed = True
    print(f"  ✗ {len(claimed_without_directory)} 个阶段的头部声称有判别力样本，而目录不存在：")  # gate-lint:summary
    for entry in claimed_without_directory:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路，别两边都不做。① 把样本建起来（fixtures/<阶段名>/{red,green}/ 各一份加 expect），")
    print("               让那句声称变成真的；② 这个阶段确实装不进沙箱（要真起虚机、要 cargo 真跑），就把头部那句改成")
    print("               「本阶段没有 fixtures 样本：<为什么装不进>，判别力靠 <别的什么> 」——说清楚，别留一句对不上的声称。")
if orphans:
    failed = True
    print(f"  ✗ {len(orphans)} 个孤儿样本目录，没有对应的阶段：")  # gate-lint:summary
    for entry in orphans:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：阶段退役时把它的样本目录一起删掉；阶段改名了就把目录改成新名字——")
    print("               留着的目录没有任何东西会跑它，而它摆在那里看着就像那个阶段验过了。")
if hollow:
    failed = True
    print(f"  ✗ {len(hollow)} 个样本目录是空壳：")  # gate-lint:summary
    for entry in hollow:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：red 至少要有一条 want=（失败信息里必须出现的片段），green 写 exit=0；")
    print("               少了 expect，判别力自检会跳过那一份，而目录在、看着像验过了。")
if failed:
    sys.exit(1)

print(f"  ✓ 阶段头部声称的判别力样本都在（{len(stages)} 个阶段里 {claiming} 个声称有样本），"
      f"{FIXTURE_DIR} 下没有孤儿目录，也没有缺 expect 的空壳")
PY

# ── 第 ④ 条：日期不许是编的 ────────────────────────────
# 判据 source 上游 lib.sh，不另写一份：两套装置算同一个量，就要落到同一个数。
# lib.sh 自带 set -euo pipefail，所以整段关在子 shell 里，判定经文件带出来
# （command-safety.md：子 shell 里的赋值传不回父进程）。
date_report="$(mktemp)"
date_rc=0
(
  source "$LIB"
  require_date_arithmetic
  git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'SKIP\t不是 git 仓，列不出要扫的文件\n'; exit 0; }
  start_date="$(project_start_date "$(pwd)" || true)"; start_date="${start_date:-$SOP_START_DATE}"
  # 阳性对照：判据先对一个确知越界的日期跑一次，报不出越界就说明它已经失效。
  # 少了这一句，上游把 date_out_of_range 改名或删掉之后，这一条会**静默报绿**——
  # 调用落在 `if` 条件位置，command not found 的 127 被读成「没越界」，COUNT 照样打印。
  # 2026-09-23 实测：把 lib.sh 里的函数改个名，这一条报「✓ 日期都可能是真的」，数看着完全正常。
  if ! date_out_of_range "1970-01-01" "$start_date" >/dev/null 2>&1; then
    printf 'BROKEN\t判据对 1970-01-01 都报不出越界：date_out_of_range 被改名、删掉，或边界口径变了\n'
    exit 0
  fi
  date_re='[0-9]{4}-[0-9]{2}-[0-9]{2}'
  # 唯一的排除：这一条自己的红样本，里面的越界日期是它的判别力本身（C441（全仓清扫工具会吃掉自己的判别力样本）
  # 说的就是清扫工具会吃掉自己的样本）。**不静默挖空**——跳过几份现算着报进成功句，
  # 读的人看得见排除了什么（show-me-test.md：排除的每一份都要登记，跳过清单要与被扫集合出自同一份数据）。
  own_fixture=".claude/gate.d/fixtures/95-fixture-claims.sh/"
  scanned=0; dated=0; bodies=0; body_dates=0; skipped_own=0
  # 路径走 NUL：git 默认把非 ASCII 路径整条引起来并转义成八进制，报出来的那一行既搜不到也对不上 want
  while IFS= read -r -d '' path; do
    # --cached 列的是**索引**不是磁盘：git mv 过又被别的会话 `git reset` 掉暂存时，
    # 索引里还留着旧名，而那个文件早已不在工作区。不滤掉就会判红一批根本不存在的文件
    # （2026-09-23 真仓实测：6 个 git mv 改过名的样本产物被判红，磁盘上一个都没有）。
    [[ -e "$path" ]] || continue
    scanned=$((scanned + 1))
    # 按月命名的文件（.claude/kb/decisions-history/<年-月>.md）不含「日」，上面那把尺抓不到。
    # 判法不能拿 2026-08 当 2026-08-01 比——那会误判现存的 decisions-history/2026-08.md
    # （它的第一天早于下界，而这个月本身是合法的）。整月与允许区间没有交集才算越界，
    # 而「没有交集」等价于两个端点都越界，所以不必分辨越的是哪一端，措辞变了也不受影响。
    if [[ "$path" =~ (^|/)([0-9]{4}-(0[1-9]|1[0-2]))\.md$ ]]; then
      month="${BASH_REMATCH[2]}"
      month_end="$(date -d "$month-01 + 1 month - 1 day" +%F)"
      dated=$((dated + 1))
      if date_out_of_range "$month-01" "$start_date" >/dev/null \
         && why="$(date_out_of_range "$month_end" "$start_date")"; then
        printf 'BADNAME\t%s\t%s（整月）\t%s\n' "$path" "$month" "$why"
      fi
    fi
    [[ "$path" =~ $date_re ]] || continue
    dated=$((dated + 1))
    while IFS= read -r found; do
      [[ -n "$found" ]] || continue
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADNAME\t%s\t%s\t%s\n' "$path" "$found" "$why"
      fi
    done < <(grep -oE "$date_re" <<<"$path" | sort -u)
  done < <(git ls-files -z --cached --others --exclude-standard | sort -z -u)
  # 正文只扫 fixtures：真 kb 正文里的日期常常说的是外部的事（别家项目、RFC 历史、引编造日期当例子），
  # 而下界假设「本仓的事不可能早于本仓第一个提交」对它们不成立。
  while IFS= read -r -d '' body; do
    [[ -e "$body" ]] || continue
    if [[ "$body" == "$own_fixture"* ]]; then skipped_own=$((skipped_own + 1)); continue; fi
    bodies=$((bodies + 1))
    while IFS= read -r hit; do
      [[ -n "$hit" ]] || continue
      line_number="${hit%%:*}"; found="${hit##*:}"
      body_dates=$((body_dates + 1))
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADBODY\t%s:%s\t%s\t%s\n' "$body" "$line_number" "$found" "$why"
      fi
      # 整行去重，不拿行号当唯一键：`sort -u -t: -k1,1n` 按行号去重，
      # 一行上的第二个日期会被丢掉（实测：`实测（2026-09-20）…已定（2026-01-01）` 判绿）
    done < <(grep -noE "$date_re" "$body" 2>/dev/null | sort -u)
  done < <(git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/' | sort -z -u)
  printf 'COUNT\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$scanned" "$dated" "$bodies" "$body_dates" "$start_date" "$skipped_own" "$own_fixture"
) > "$date_report" 2>&1 || date_rc=$?

if grep -qE '^BAD(NAME|BODY)' "$date_report"; then
  date_rc=1
  if grep -q '^BADNAME' "$date_report"; then
    echo "  ✗ $(grep -c '^BADNAME' "$date_report") 个文件名里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADNAME' "$date_report")
  fi
  if grep -q '^BADBODY' "$date_report"; then
    echo "  ✗ $(grep -c '^BADBODY' "$date_report") 处样本正文里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADBODY' "$date_report")
  fi
  echo "     → 怎么办：写真实发生的那一天——记录写它是哪天记下的，产物写它是哪天跑的，"
  echo "       样本写它所属 fixture 的创建日：git log --reverse --format=%ad --date=short -- <fixture 目录>；"
  echo "       样本里要表示先后两次事件的，就用创建日和它的次日，别用 2026-01-01 / 2026-01-02 这种占位。"
  echo "       查不出来也别填占位日期：占位日期读起来和真日期一模一样，后面每一个引用它的人都会当真。"
  echo "       红样本非要一个编造日期不可的，让 fixtures/<阶段>/red/setup.sh 在临时仓里现造，别摆进仓里——"
  echo "       摆进来它会被这一条自己扫到，红的是样本而不是被测的东西。"
elif grep -q '^BROKEN' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期判据失效了，这一条什么也没在判："
  grep '^BROKEN' "$date_report" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "     → 怎么办：判据是上游 .claude/singlefs-ai-sop/scripts/lib.sh 的 date_out_of_range，本仓不另写一份。"
  echo "       先 grep 那个文件确认函数还在、名字没变；同步过副本就读一遍 CHANGELOG 看边界口径改没改。"
  echo "       上游真改了名，改这一条去跟上新名字，别在本仓另抄一份判据——两套装置算同一个量就会分叉。"
elif grep -q '^SKIP' "$date_report"; then
  printf '  ⊘ 日期这一条未跑：%s\n' "$(grep '^SKIP' "$date_report" | cut -f2)"
elif ! grep -q '^COUNT' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期这一条没跑完，报告里没有计数行："
  sed 's/^/      /' "$date_report"   # gate-lint:detail
  echo "     → 怎么办：上面是子 shell 的原样输出；多半是 lib.sh 取不到或 git 列不出文件，按它说的修。"
else
  read -r _ scanned dated bodies body_dates start_date skipped_own own_fixture < <(grep '^COUNT' "$date_report")
  # 计数缺一个就判红：成功句是给人看的唯一结论，它报着空数照样绿，比没有这一条更糟
  # （2026-09-23 变异实测：砍掉「没有 COUNT 就判红」那一支，屏幕上出现「%s 个文件里  条带日期」四个数全空）。
  if ! [[ "$scanned" =~ ^[0-9]+$ && "$dated" =~ ^[0-9]+$ && "$bodies" =~ ^[0-9]+$ && "$body_dates" =~ ^[0-9]+$ && "$skipped_own" =~ ^[0-9]+$ ]]; then
    date_rc=1
    echo "  ✗ 日期这一条的计数行残缺，报不出检查了多少项："
    grep '^COUNT' "$date_report" | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：子 shell 多半在数完之前就退出了，上面那行是它吐出的原样；"
    echo "       扫到 0 项也不是通过，报着空数的成功句与判过了一模一样（show-me-test.md）。"
  else
    printf '  ✓ 日期都可能是真的（文件名：%s 个文件里 %s 条带日期；样本正文：%s 份文件里 %s 个日期；下界 %s 往前宽 7 天）\n' \
      "$scanned" "$dated" "$bodies" "$body_dates" "$start_date"
    printf '    没扫的 %s 份：%s 下的文件——这一条自己的红样本，里面的越界日期就是它的判别力（C441（全仓清扫工具会吃掉自己的判别力样本））\n' \
      "$skipped_own" "$own_fixture"
  fi
fi
rm -f "$date_report"

if [[ "$python_rc" -ne 0 || "$date_rc" -ne 0 ]]; then
  exit 1
fi
exit 0
