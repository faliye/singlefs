#!/usr/bin/env bash
# 门禁 doc-experiments 的格函数库：preregistration-parallelism、preregistration-judge-features、judge-entries-in-sync、replay-comparison-units、pinned-apparatus-revision。
# 由 .claude/gate.d/doc-experiments.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── 格 preregistration-parallelism：E186 起的跑前登记有「并行与挂钟」一节，五样写全，预计挂钟与态数乘单态耗时对得上
# 判据（.claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行与第 3a 步「现算态数或输入规模与单态耗时……相乘得挂钟」）：这次改动新建或改动的
#   research/prompts/e<号>-preregistration.md，号不小于 PREREGISTRATION_PARALLELISM_FIRST_NUMBER（186）的，要有一节标题是「## <汉字序号>、并行与挂钟」，
#   节里（到下一个「## 」标题为止，围栏代码块里的行不算）五个标签行各至少一行、冒号后至少 8 个字：
#     独立单位：哪些单位互相独立            线程：分几条线程、取哪个登记值
#     串行：哪几步因计时或共享状态必须串行与为什么（没有就写「无」并写为什么都能并行）
#     预计挂钟：各段预计挂钟（推的写依据）  预计在用核数：各段预计在用几个核（推的写依据）
#   标签行可带列表记号（- 或 *）与加粗（**…**），冒号全角半角都认；「预计挂钟」「预计在用核数」两行还要带一个数字。
# 挂钟算式（判「预计挂钟：」那一行自己写的态数乘单态耗时，读法在下面的 wallclock_claims）：那一行按「；」「;」「。」切成段，每段认两种写法之一：
#   式子：「<数>[量词] × … × <数><时间单位> [÷ <数>] ≈|=|约 <挂钟>」，乘数里恰好一个带时间单位（单态耗时），其余是态数；算出的是乘积除以写的除数。
#   每：段里没有式子时，「每<量词>」后 40 个字以内的「<数><时间单位>」是单态耗时，它前面最近的、量词与「每」后那串字开头相同的「<数><量词>」是态数
#     （「<数><量词> × <数><量词> … [= <数><量词>]」算一个态数，有等号取等号右边）；一段里几对的乘积相加，与这一段最后一对之后最后写的「<数><时间单位>」比。
#   时间单位认 纳秒 微秒 毫秒 秒 分钟 小时 天 与 ns µs us ms s min h；数可带千分位逗号、小数、e 指数、万或亿；「10–25 分钟」这种写法当一个区间。
#   算出的与写的差一个数量级以上（写的上限乘 10 还不到算出的下限，或写的下限超过算出的上限乘 10）判红，点名那一段与怎么算的。
#   一段都认不出的登记（写不出态数或单态耗时，或写了而没有挂钟可比）报「本次未判」并点名，往 $GATE_NOT_RUN_FILE 报一行，不算绿。
# 态数与第六节（「## 六、」那一节，围栏里的不算）对（quantities_section_judgement）：上面认出的每一条用到的态数（式子里不带时间单位的乘数、「每」配上的那个「<数><量词>」），
#   在第六节里按同一份读法（segment_counts）找「<数><量词>」，前面紧挨「第」的是序号、不算：有同值而量词互为开头的算对上；没有同值、有量词逐字相同的算对不上，判红，
#   点名两边的原文与第六节的行号；第六节里没有同值也没有同量词的、态数没写量词的（「960 × 10000 × …」）算认不出，报「态数与第六节的比对本次未判」并点名，
#   往 $GATE_NOT_RUN_FILE 报一行，不算绿，也不判红。
# 起判：号不小于 186 的才判，E185 以下的登记不回补这一节（不设排除表：号以下的一律不判，号以上的没有例外）。
#   重跑登记（e<号>-r<n>-prereg.md）不判：差分重跑照设计员定义只写三样、其余「同上一次登记」，这一节跟着原登记走。
#   这次改动里一份要判的都没有：退 77（无对象可判），成功句与无对象句逐个列出号小于 186 不判的那几份。
# 管不到：写的数对不对、装置是不是照它实现、串行的理由成不成立——靠三方与 task-server 记进 task status 的每次尝试平均在用核数；
#   单态耗时只读「预计挂钟：」那一行，写在第六节或别的行、这一行里没重写的不读；态数只与第六节比，第九节（变异条数）这类别的节里写的不比；
#   第六节里同一个量词写着几个数的，有一个同值就算对上；「每」认的量词按开头相同配对，配错了靠 review；
#   没写除数的并行（「按 31 线程并行约 5 分钟」）按串行乘积比，一个数量级的余量盖得住的照绿。
# 样本：preregistration-parallelism-red（setup.sh 提交基准之后新建 e990 缺整节、e991 有节缺「串行：」一行、e992「预计在用核数：」没有数字，
#   改 e100（号小于 186、没有这一节）不判）；preregistration-parallelism-green（新建 e993 五样写全，改 e100 不判，报「查了 1 份」，e993 的挂钟写成散文、报本次未判）；
#   preregistration-parallelism-wallclock-red（真仓 E186、E187、E197 的「## 十四、并行与挂钟」逐字抄进同号登记，各改坏一个数：E186 变异 17 条改成 1700 条、
#   E187 式子里单态 0.5 毫秒改成 50 毫秒、E197 的 65 次改成 6500 次，三条逐个点名判红）；
#   preregistration-parallelism-wallclock-green（同三节原样：E186、E197 走「每」、E187 走式子，判了 3 条；另一份 e994 只写散文，报本次未判 1 份并点名）；
#   preregistration-parallelism-quantities-section-red（e995 第六节写「变异 17 条」，「预计挂钟：」写「变异 24 条、每条约 1 分钟…」，挂钟算式对得上而态数对不上，点名判红）；
#   preregistration-parallelism-quantities-section-green（e996 第六节写「960 段」与「10000 次发布」，「预计挂钟：」的式子写「960 段 × 10000 次 × 0.5 毫秒 ÷ 31 ≈ 155 秒」，比了 2 个、判绿；
#   另一份 e997 的第六节没写态数，报「态数与第六节的比对本次未判 1 份」并点名，不算红）；
#   preregistration-parallelism-git-error-red（setup.sh 建 git 仓并把 git 配置里的 diff.renames 写成 git 认不得的值，取改动路径的 git diff 一律失败：
#   红行之前有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文，原文经 CHANGE_RANGE_ERROR_FILE 从父进程带进格的输出）。
# 弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 parallelism-section-presence-only、parallelism-old-registrations-judged、
#   parallelism-wallclock-unchecked、parallelism-unjudged-as-green、parallelism-quantities-section-unchecked 五项。
# 退役判据：设计员定义不再要求这一节时删掉这一格（连续几周零红不算：它只判新写的登记，零红说明新登记都写全了，不说明对象没了）。
PREREGISTRATION_PARALLELISM_FIRST_NUMBER=186
cell_preregistration_parallelism() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些登记是这次改动新建或改动的，这一格无对象可判"; exit 77; }
  if ((! changed_paths_library_loaded || ! changed_paths_ok)); then
    print_change_range_error
    echo "  ✗ 取不到这次改动的路径（scripts/gate/changed-paths.sh 读不到，或它的 gate_changed_paths 失败）"
    echo "     → 怎么办：看上面 changed-paths.sh 打的错误，修好再单跑这一格（bash .claude/gate.d/doc-experiments.sh --check preregistration-parallelism）"
    exit 1
  fi
  python3 - "$SHARED_CHANGED_PATHS" "$PREREGISTRATION_PARALLELISM_FIRST_NUMBER" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" <<'PY'
import os, re, sys

changed_file, first_number, broken = sys.argv[1], int(sys.argv[2]), sys.argv[3].split(",")
NAME = re.compile(r"research/prompts/e([0-9]+)-preregistration\.md")
HEADING = re.compile(r"^##\s+[一二三四五六七八九十百]+、并行与挂钟\s*$")
LABELS = ("独立单位", "线程", "串行", "预计挂钟", "预计在用核数")
LABEL_LINE = re.compile(r"^\s*(?:[-*]\s+)?(?:\*\*)?(独立单位|线程|串行|预计挂钟|预计在用核数)(?:\*\*)?\s*[：:]\s*(?:\*\*)?\s*(.*)$")
NEEDS_NUMBER = ("预计挂钟", "预计在用核数")

# ── 挂钟算式的读法（判据写在这一格函数上方的注释）
SECONDS = {"纳秒": 1e-9, "ns": 1e-9, "微秒": 1e-6, "µs": 1e-6, "μs": 1e-6, "us": 1e-6, "毫秒": 1e-3, "ms": 1e-3,
           "秒": 1.0, "s": 1.0, "分钟": 60.0, "min": 60.0, "小时": 3600.0, "h": 3600.0, "天": 86400.0}
NUMBER = r"[0-9]+(?:,[0-9]{3})*(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?\s*(?:万|亿)?"
TIME_UNIT = r"(?:纳秒|微秒|毫秒|分钟|小时|秒|天|ns|µs|μs|us|ms|min|s|h)(?![A-Za-z])"
SPAN = r"\s*[–~\-－到至]\s*"
TIME = re.compile(rf"(?P<low>{NUMBER})(?:{SPAN}(?P<high>{NUMBER}))?\s*(?P<unit>{TIME_UNIT})")
TIMES = r"[×✕*]"
WORD = r"[一-鿿]+"
COUNT = re.compile(rf"(?P<number>{NUMBER})\s*(?P<word>{WORD})")
FACTOR = rf"{NUMBER}\s*(?:{WORD})?"
CHAIN = re.compile(rf"(?P<chain>{FACTOR}(?:\s*{TIMES}\s*{FACTOR})+)(?:\s*=\s*(?P<total>{NUMBER})\s*(?P<total_word>{WORD})?)?")
FORMULA_FACTOR = rf"{NUMBER}\s*(?:{TIME_UNIT}|{WORD})?"
FORMULA = re.compile(rf"(?P<factors>(?:{FORMULA_FACTOR}\s*{TIMES}\s*)+{FORMULA_FACTOR})(?:\s*[÷/]\s*(?P<divisor>{NUMBER}))?"
                     rf"\s*(?:≈|=|约等于|约)\s*(?P<result>{NUMBER}(?:{SPAN}{NUMBER})?\s*{TIME_UNIT})")
EACH = re.compile(rf"每(?P<word>{WORD})")
EACH_REACH = 40


def number_value(text):
    text = text.replace(",", "").strip()
    scale = 1.0
    if text.endswith("万"):
        scale, text = 1e4, text[:-1].strip()
    elif text.endswith("亿"):
        scale, text = 1e8, text[:-1].strip()
    return float(text) * scale


def time_span(found):
    unit = SECONDS[found.group("unit")]
    low = number_value(found.group("low")) * unit
    return low, (number_value(found.group("high")) * unit if found.group("high") else low)


def common_prefix(left, right):
    size = 0
    while size < min(len(left), len(right)) and left[size] == right[size]:
        size += 1
    return size


def formula_claims(segment):
    claims = []
    for found in FORMULA.finditer(segment):
        factors = [factor.strip() for factor in re.split(TIMES, found.group("factors"))]
        timed = [TIME.fullmatch(factor) for factor in factors]
        if sum(1 for item in timed if item) != 1:
            continue
        low = high = 1.0
        counts = []   # 这一条用到的态数：(值, 量词, 原文)，与第六节比（quantities_section_counts）
        for factor, item in zip(factors, timed):
            factor_low, factor_high = time_span(item) if item else (number_value(re.match(NUMBER, factor).group(0)),) * 2
            low, high = low * factor_low, high * factor_high
            if not item:
                counted = COUNT.match(factor)
                counts.append((factor_low, counted.group("word") if counted else "", factor))
        divisor = number_value(found.group("divisor")) if found.group("divisor") else 1.0
        claims.append((low / divisor, high / divisor, *time_span(TIME.search(found.group("result"))), f"式子「{found.group(0).strip()}」", counts))
    return claims


def segment_counts(segment):
    """→ [(位置, 态数, 量词, 原文)]：乘出来的一串算一个，单个的「<数><量词>」（不是时间）各算一个。"""
    counts, covered = [], []
    for found in CHAIN.finditer(segment):
        if TIME.search(found.group(0)):
            continue
        if found.group("total"):
            value, word = number_value(found.group("total")), found.group("total_word") or ""
        else:
            value, word = 1.0, ""
            for factor in re.split(TIMES, found.group("chain")):
                item = COUNT.match(factor.strip())
                value *= number_value(item.group("number") if item else factor)
                word = item.group("word") if item else ""
        counts.append((found.start(), value, word, found.group(0).strip()))
        covered.append((found.start(), found.end()))
    for found in COUNT.finditer(segment):
        if any(start <= found.start() < end for start, end in covered) or TIME.match(segment, found.start()):
            continue
        counts.append((found.start(), number_value(found.group("number")), found.group("word"), found.group(0).strip()))
    return counts


def each_claim(segment):
    counts, pairs, last_end = segment_counts(segment), [], 0
    for each in EACH.finditer(segment):
        unit_time = TIME.search(segment, each.end(), each.end() + EACH_REACH)
        candidates = [count for count in counts if count[0] < each.start() and common_prefix(count[2], each.group("word"))]
        if not unit_time or not candidates:
            continue
        _position, value, word, text = max(candidates, key=lambda count: count[0])
        low, high = time_span(unit_time)
        pairs.append((value * low, value * high, f"「{text}」乘「{segment[each.start():unit_time.end()].strip()}」", (value, word, text)))
        last_end = max(last_end, unit_time.end())
    stated = list(TIME.finditer(segment, last_end)) if pairs else []
    if not stated:
        return []
    return [(sum(pair[0] for pair in pairs), sum(pair[1] for pair in pairs), *time_span(stated[-1]),
             "、".join(pair[2] for pair in pairs) + f"，与写的「{stated[-1].group(0).strip()}」比", [pair[3] for pair in pairs])]


def wallclock_claims(line):
    """「预计挂钟：」那一行里认得出的每一条 → [(算出的秒数下限, 上限, 写的秒数下限, 上限, 怎么算的)]。"""
    claims = []
    for segment in re.split(r"[；;。]", line):
        claims.extend(formula_claims(segment) or each_claim(segment))
    return claims


QUANTITIES_SECTION = re.compile(r"^##\s+六、")


def quantities_section_counts(lines):
    """登记第六节（「## 六、」标题到下一个「## 」标题，围栏代码块里的行不算）里写的态数 → [(行号, 值, 量词, 原文)]；
    读法与「预计挂钟：」那一行同一份（segment_counts）；前面紧挨「第」的是序号（「第 3 条」），不算态数。没有第六节交空表。"""
    start = next((index for index, line in enumerate(lines) if QUANTITIES_SECTION.match(line)), None)
    if start is None:
        return []
    found, in_fence = [], False
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if line.startswith("## "):
            break
        if line.lstrip().startswith(("```", "~~~")):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for position, value, word, text in segment_counts(line):
            if word and not line[:position].rstrip().endswith("第"):
                found.append((index + 1, value, word, text))
    return found


def quantities_section_judgement(claims, quantities):
    """「预计挂钟：」那一行用到的每个态数与第六节比 → (对不上的说明, 认不出的原文, 比上了几个)。
    第六节里有同值、量词互为开头的算对上；没有同值而有量词逐字相同的算对不上；都没有（或态数没写量词）算认不出。"""
    mismatched, unknown, compared = [], [], 0
    for claim in claims:
        for value, word, written in claim[5]:
            related = [count for count in quantities if word and (count[2].startswith(word) or word.startswith(count[2]))]
            if any(abs(count[1] - value) <= 1e-9 * max(1.0, value) for count in related):
                compared += 1
                continue
            same_word = [count for count in related if count[2] == word]
            if same_word:
                compared += 1
                mismatched.append(f"「预计挂钟：」里的态数「{written}」与第六节写的"
                                  + "、".join(f"「{count[3]}」（第 {count[0]} 行）" for count in same_word) + "对不上")
            else:
                unknown.append(written)
    return mismatched, unknown, compared


def seconds_text(value):
    for unit, size in (("小时", 3600.0), ("分钟", 60.0), ("秒", 1.0)):
        if value >= size:
            return f"{value / size:.3g} {unit}"
    return f"{value:.3g} 秒"


with open(changed_file, encoding="utf-8") as handle:
    changed = sorted({line.strip() for line in handle if line.strip()})
judged, old, problems, mismatched, unjudged, claim_count, claimed = [], [], [], [], [], 0, []
states_mismatched, states_unjudged, states_compared = [], [], 0
for path in changed:
    found_name = NAME.fullmatch(path)
    if not found_name or not os.path.isfile(path):
        continue
    if int(found_name.group(1)) < first_number and "parallelism-old-registrations-judged" not in broken:
        old.append(path)
        continue
    judged.append(path)
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    start = next((index for index, line in enumerate(lines) if HEADING.match(line)), None)
    if start is None:
        problems.append(f"{path}：没有「## 十四、并行与挂钟」这一节")
        continue
    if "parallelism-section-presence-only" in broken:
        continue
    end = next((index for index in range(start + 1, len(lines)) if lines[index].startswith("## ")), len(lines))
    labelled, in_fence = {}, False
    for index in range(start + 1, end):
        if lines[index].lstrip().startswith(("```", "~~~")):
            in_fence = not in_fence
            continue
        found_label = None if in_fence else LABEL_LINE.match(lines[index])
        if found_label and found_label.group(1) not in labelled:
            labelled[found_label.group(1)] = (index + 1, found_label.group(2).strip().strip("*").strip())
    for label in LABELS:
        if label not in labelled:
            problems.append(f"{path}:{start + 1}：「并行与挂钟」一节里没有「{label}：」一行")
            continue
        number, text = labelled[label]
        if len(text) < 8:
            problems.append(f"{path}:{number}：「{label}：」后面不到 8 个字（「{text}」）")
        elif label in NEEDS_NUMBER and not re.search(r"[0-9]", text):
            problems.append(f"{path}:{number}：「{label}：」里没有数（各段写估值，推的写依据）")
    if "预计挂钟" not in labelled or "parallelism-wallclock-unchecked" in broken:
        continue
    number, text = labelled["预计挂钟"]
    claims = wallclock_claims(text)
    if not claims:
        if "parallelism-unjudged-as-green" not in broken:
            unjudged.append(f"{path}:{number}")
        continue
    claimed.append(path)
    for computed_low, computed_high, stated_low, stated_high, how, _counts in claims:
        claim_count += 1
        if stated_high * 10 < computed_low or stated_low > computed_high * 10:
            mismatched.append(f"{path}:{number}：预计挂钟与态数乘单态耗时差一个数量级以上：{how}；算出约 {seconds_text(computed_low)}"
                              f"{'' if computed_high == computed_low else '–' + seconds_text(computed_high)}，"
                              f"写的约 {seconds_text(stated_low)}{'' if stated_high == stated_low else '–' + seconds_text(stated_high)}")
    # 态数与第六节对：对不上判红，认不出的报本次未判（弄坏开关 parallelism-quantities-section-unchecked 下不比）
    if "parallelism-quantities-section-unchecked" not in broken:
        quantities_mismatched, quantities_unknown, quantities_compared = quantities_section_judgement(claims, quantities_section_counts(lines))
        states_compared += quantities_compared
        states_mismatched.extend(f"{path}:{number}：{problem}" for problem in quantities_mismatched)
        if quantities_unknown:
            states_unjudged.append(f"{path}:{number}（{'、'.join(quantities_unknown)}）")
old_text = f"；号小于 {first_number} 不判的 {len(old)} 份：{'、'.join(old)}" if old else ""
wallclock_text = f"；挂钟算式判了 {claim_count} 条（{len(claimed)} 份）"
unjudged_text = f"；挂钟算式本次未判 {len(unjudged)} 份（写不出态数或单态耗时）：{'、'.join(unjudged)}" if unjudged else ""
if not judged:
    print(f"  ! 这次改动里没有新建或改动 E{first_number} 起的跑前登记（research/prompts/e<号>-preregistration.md），这一格无对象可判{old_text}")
    sys.exit(77)
if unjudged:
    print(f"  ! 挂钟算式本次未判 {len(unjudged)} 份：「预计挂钟：」那一行写不出态数或单态耗时（或写了而没有挂钟可比）：{'、'.join(unjudged)}")
    print("     要这一格判它：在那一行写式子「<态数> × <单态耗时> [÷ <并行数>] ≈ <挂钟>」，或同一段里写「<数><量词>」与「每<量词>…<数><时间单位>」再写这一段的挂钟")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh preregistration-parallelism：挂钟算式本次未判 {len(unjudged)} 份（{'、'.join(unjudged)}）\n")
if states_unjudged:
    print(f"  ! 态数与第六节的比对本次未判 {len(states_unjudged)} 份：「预计挂钟：」里的态数在第六节里找不到同值或同量词的数（或态数没写量词）：{'；'.join(states_unjudged)}")
    print("     要这一格判它：第六节那个量的行里写出同一个态数与量词（「<数><量词>」，例：变异 17 条），「预计挂钟：」里的态数照它写")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh preregistration-parallelism：态数与第六节的比对本次未判 {len(states_unjudged)} 份（{'；'.join(states_unjudged)}）\n")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(problems)} 处没写全「并行与挂钟」（查了 {len(judged)} 份）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md「登记的固定节名」的「## 十四、并行与挂钟」一行补这一节：哪些单位互相独立（独立单位：）、"
          "分几条线程（线程：）、哪几步因计时或共享状态必须串行与为什么（串行：）、各段预计挂钟（预计挂钟：）与预计在用核数（预计在用核数：），推的写依据")
if mismatched:
    for problem in mismatched:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(mismatched)} 处预计挂钟与态数乘单态耗时差一个数量级以上（判了 {claim_count} 条、{len(claimed)} 份）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md 第 3a 步现算态数与单态耗时（量过的写命令与读数，估的写式子和依据），改「预计挂钟：」里写的挂钟，"
          "或改错了的态数、单态耗时；并行跑的写成式子「<态数> × <单态耗时> ÷ <并行数> ≈ <挂钟>」，读法在 doc-experiments.sh 这一格函数上方的注释")
if states_mismatched:
    for problem in states_mismatched:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(states_mismatched)} 处「预计挂钟：」的态数与第六节写的对不上（比了 {states_compared} 个态数）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md 第 3a 步，态数以第六节那个量的定义为准现算，「预计挂钟：」里的态数与挂钟照它改；"
          "第六节写错了的改第六节，两处写的是不同的量的，把「预计挂钟：」里的量词改成与第六节分得开的说法")
if problems or mismatched or states_mismatched:
    sys.exit(1)
states_text = f"；态数与第六节比了 {states_compared} 个" + (f"，本次未判 {len(states_unjudged)} 份：{'；'.join(states_unjudged)}" if states_unjudged else "")
print(f"  ✓ 跑前登记的「并行与挂钟」：查了 {len(judged)} 份（{'、'.join(judged)}），五样都写全了{old_text}{wallclock_text}{unjudged_text}{states_text}")
PY
}

# ── 格 preregistration-judge-features：跑前登记里起走崩溃判段的装置二进制，特性要带 gpu
# 为什么：E161 第 3 次重跑照登记原样起可行性档，没带 gpu 特性，多机配置开着 GPU 时判段只录不判、退 77（records/2026-10-29-subagent拆分提案.md 第一百二十六节 ③）。
# 对象：这次改动新建或改动的跑前登记，research/prompts/ 下名字以 e<号> 起头、带 prereg 的 .md（e<号>-preregistration.md、e<号>-r<n>-prereg.md、e<号>-preregistration-r<n>.md，重跑登记也判）。
# 判据：登记里每一处在命令位置上的 `cargo run` / `cargo test`（命令截到反引号、没转义的竖线、分号、& 或 ` -- ` 为止，-- 之后是给二进制的参数）：
#   命令位置只认三种（command_spans）：代码围栏里的行；行首就是命令的行（去掉缩进、引用号 >、列表记号之后，行首是命令、或行首那一对反引号里是命令）；
#   表格里整格只有一对反引号、反引号里是命令的格。「是命令」= 去掉开头的 $ 提示符之后以 ASCII 的命令字起头（字母、数字、_ . / ~ $ 引号、左括号），
#   而且从命令起头到这一处 cargo 之间（引号里的去掉）没有汉字与全角标点。行文里内嵌的（「36G 下 `cargo run --bin …` 的 352 秒」这类）、
#   标签之后的（「- 可行性档：`cargo run …`」）、表格里夹着字的、围栏里以汉字或 ✓ 起头的输出行都不判，只报数。
#   以没转义的反斜杠收尾的行（shell 续行）先与下一行并成一条再认：围栏里的行与围栏外行首是命令的行（不含表格行）都并，接连几行续行都并，
#   围栏收尾那一行不并进来；点名的行号是头一行的。shell 注释照 bash 认（引号外、行首或空白之后的 # 起到行尾）：注释里的反斜杠不续行，
#   续行接到整行注释就断（注释那一行单独认）；命令那一段（围栏里的行、行首是命令的行、表格整格的命令）行尾的注释截掉、不当命令的参数
#   （「# 以后加 --features gpu」不算带了），注释里写的 cargo 不判也不计数。
#   命令那一截照 shell 切词（引号里的空白不切），引号没配对的退回按空白切。
#   ① 点名了 --bin <名>（--bin=<名>）的才判，cargo test 另认 --test <名>（--test=<名>，测试目标）；一条命令点名几个目标逐个判；都没点名的不判、只报数；
#   ② 目标的源文件按包认：-p / --package 给了包名的取那个包，没给的在全部包里找同名、同种（bin 或 test）的目标，恰好一个才算认得；
#      包与目标的清单读被判的根里的 crates/*/Cargo.toml 与 scripts/experiment-apparatus/Cargo.toml（[package] 的 name、[[bin]] 与 [[test]] 的 name 与 path，
#      外加 src/bin/<名>.rs、src/bin/<名>/main.rs、tests/<名>.rs、tests/<名>/main.rs 的自动发现）；
#   ③ 源文件调了判段入口的算走崩溃判段：提到判段模块 JUDGE_MODULES（crash_amplification、crash_state_enumeration）之一（整词，路径、导入、`use … as <别名>` 的别名导入都算），
#      又写着判段入口表 JUDGE_ENTRIES 里的名字（整词，经别名写成 <别名>::<入口> 的照认）。判段入口是这两个模块里照多机配置选判段去路的公开入口
#      （配置开了 GPU 而没编进 gpu 特性时只录不判、打 NOT_RUN）；只取崩溃点、只在 CPU 上取样、只用常量的（crash_points_of_the_steps、plan_crash_points、
#      sample_crash_states_on_the_cpu 这类）不算。JUDGE_ENTRIES 与两个模块里的公开判段入口是不是逐个相同，由 judge-entries-in-sync 格判；
#   ④ 走判段的，特性（--features <值> / --features=<值> / -F <值> / -F<值>，可写几次、取并集，逗号或空白分隔；--all-features 算带上）里没有 gpu（或 <包>/gpu）判红，
#      点名登记的行与命令。scripts/hosts/compile-then-swap.py 收同一套写法，执行员照登记原样给。
# 认不出的（目标在清单里找不到、没给包名而几个包都有同名的目标、源文件读不出）报「本次未判」并点名，往 $GATE_NOT_RUN_FILE 报一行，不算绿。
# 这次改动里没有跑前登记、或登记里一条点名 --bin / --test 的 cargo run / cargo test 都没有：退 77；全部认不出：退 77 并报本次未判。
# 管不到的：判段入口写在 bin 引的别的模块里而 bin 的源文件自己没写判段模块与入口名字的（只读 bin 的那一份源文件）；入口名只出现在注释里也当调了（不剥注释，多要 gpu 不漏判）；
#   入口本身再起一个别名（use …::run_from_environment as 别名）之后调别名的照认（导入那一行写着入口名）；命令拼在变量里的；起法是不是经清场、
#   NOT_RUN_ 行有没有核（跑前登记该写的另两样，归设计员定义与执行员交回）；派发闸那一支不动，不在这里判。
# 样本：preregistration-judge-features-red（e990 列表行首的 cargo run 起走判段的 bin 没带特性、e991 表格整格的 cargo test 带了 lookup-work-counters 没带 gpu，两处判红；
#   同一份里不走判段的 bin 不带特性不报）；preregistration-judge-features-green（e992 三处走判段的 bin 各用 --features lookup-work-counters,gpu、--features=gpu、-F gpu，
#   不走判段的 bin 不带特性、-- 之后的「--bin」不当 cargo 的参数，判了 3 条；另一条 bin 在清单里找不到，报本次未判 1 条）；
#   preregistration-judge-features-prose-green（e993 里行文内嵌、标签之后、表格里夹着字、围栏里 ✓ 起头的四处起走判段的 bin 都没带 gpu，不在命令位置、不判；
#   围栏里一条带 gpu 的判了 1 条，判绿报「不在命令位置的 4 条不判」）；
#   preregistration-judge-features-continued-red（围栏里 --bin 续行到下一行、没带 gpu，并成一条判红）、
#   preregistration-judge-features-continued-green（--bin 在头一行、--features gpu 续行到下一行，并成一条判绿）；
#   preregistration-judge-features-alias-red（use crash_amplification as amplification 之后调 amplification::run_from_environment、没带 gpu 判红；
#   同一份里只从 crash_state_enumeration:: 取崩溃点、不调判段入口的 bin 不带 gpu 不报）；
#   注释行挡住命令的几份：preregistration-judge-features-comment-swallows-red
#   （围栏里注释行以反斜杠收尾、下一行没带 gpu：判红，第 7 行）、-comment-hides-red（同一围栏另一条带 gpu 的判绿、被注释挡住的那一条没带 gpu：
#   判红，第 8 行）、-comment-control-red（注释不以反斜杠收尾的对照，判红）；preregistration-judge-features-trailing-comment-red
#   （命令行尾注释里写着 --features gpu，截掉之后没带 gpu，判红）；preregistration-judge-features-continued-outside-fence-red（列表项里缩进的命令行，--bin 续行到下一行、
#   没带 gpu：并成一条判红）；preregistration-judge-features-spellings-green（-Fgpu、--features "lookup-work-counters gpu"
#   两条走判段的都算带了 gpu，判了 2 条）；preregistration-judge-features-test-target-red（cargo test 只点名 --test，测试目标调判段入口、没带 gpu，判红）；
#   行尾注释里写 --features gpu 的两份：preregistration-judge-features-list-item-trailing-comment-red
#   （列表项下缩进、行首是命令的一行，行尾注释写着 --features gpu：第 7 行判红）、
#   preregistration-judge-features-table-cell-trailing-comment-red（表格整格反引号里的同一条命令：第 7 行判红）。
# 弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 judge-features-unchecked、judge-binaries-all-counted、judge-commands-anywhere、continuation-unjoined、judge-by-module-marker、
# shell-comments-ignored、features-spelling-narrow、test-targets-unjudged、trailing-comments-kept-outside-fence 九项。
# 退役判据：崩溃判段不再挂 gpu 特性（判段的 GPU 路径一律编进去）时删掉这一格。
JUDGE_MODULES="crash_amplification crash_state_enumeration"
JUDGE_ENTRIES="run_from_environment run_from_environment_with_observers GivenStatesEngine judging_test_not_run_without_the_gpu_feature judge_not_run_in_this_binary
  enumerate_crash_states enumerate_crash_states_selecting enumerate_crash_states_versions enumerate_crash_states_selecting_versions
  enumerate_crash_states_with_expansion enumerate_crash_states_with_every_kind_of_observer enumerate_crash_states_with_attached_observers crash_point_replay_judge_not_run_line"
cell_preregistration_judge_features() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出哪些登记是这次改动新建或改动的，这一格无对象可判"; exit 77; }
  if ((! changed_paths_library_loaded || ! changed_paths_ok)); then
    print_change_range_error
    echo "  ✗ 取不到这次改动的路径（scripts/gate/changed-paths.sh 读不到，或它的 gate_changed_paths 失败）"
    echo "     → 怎么办：看上面 changed-paths.sh 打的错误，修好再单跑这一格（bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features）"
    exit 1
  fi
  python3 - "$SHARED_CHANGED_PATHS" "$JUDGE_MODULES" "$JUDGE_ENTRIES" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" <<'PY'
import glob, os, re, shlex, sys, tomllib

changed_file, modules, entries, broken = sys.argv[1], sys.argv[2].split(), sys.argv[3].split(), sys.argv[4].split(",")
MODULE_WORD = re.compile(r"\b(?:" + "|".join(map(re.escape, modules)) + r")\b")
ENTRY_WORD = re.compile(r"\b(" + "|".join(map(re.escape, entries)) + r")\b")
NAME = re.compile(r"research/prompts/e[0-9]+[^/]*prereg[^/]*\.md")
CARGO = re.compile(r"(?<![\w-])cargo\s+(?:\+\S+\s+)?(?P<command>run|test)(?![\w-])")
COMMAND_END = re.compile(r"`|\\?\||;|&|\s--(?:\s|$)")
FENCE = re.compile(r"^\s*(```|~~~)")
# 命令位置（判据在这一格函数上方的注释）：行首去掉的缩进、引用号与列表记号；命令起头的提示符；命令以哪些字起头；从起头到 cargo 之间不许有的字（汉字与全角标点）
LINE_MARKERS = re.compile(r"[ \t]*(?:>[ \t]*)*(?:[-*+][ \t]+|[0-9]+[.)][ \t]+)?")
PROMPT = re.compile(r"\$[ \t]+")
COMMAND_START = re.compile(r"[A-Za-z0-9_./~$'\"(]")
QUOTED = re.compile(r"'[^']*'|\"[^\"]*\"")
PROSE_CHARACTER = re.compile(r"[⺀-￿]")
TABLE_CELL_CODE = re.compile(r"\|[ \t]*`([^`|]+)`[ \t]*(?=\|)")


def command_spans(line, in_fence):
    """这一行里在命令位置上的几段 → [(命令起头的位置, 命令截止的位置)]；判据在这一格函数上方的注释。"""
    candidates = []
    if in_fence:
        candidates.append((len(line) - len(line.lstrip()), len(line)))
    elif line.lstrip().startswith("|"):
        candidates.extend((found.start(1), found.end(1)) for found in TABLE_CELL_CODE.finditer(line))
    else:
        start = LINE_MARKERS.match(line).end()
        if line.startswith("`", start):
            closing = line.find("`", start + 1)
            candidates.append((start + 1, closing if closing > start else len(line)))
        else:
            candidates.append((start, len(line)))
    spans = []
    for start, end in candidates:
        prompt = PROMPT.match(line, start)
        start = prompt.end() if prompt else start
        if start < end and COMMAND_START.match(line, start):
            spans.append((start, end))
    return spans


def in_command_position(line, spans, position):
    """这一处 cargo 落在某一段命令里，且从那一段起头到它之间（引号里的去掉）没有汉字与全角标点。"""
    if "judge-commands-anywhere" in broken:
        return True
    return any(start <= position < end and not PROSE_CHARACTER.search(QUOTED.sub("", line[start:position]))
               for start, end in spans)


def shell_comment_start(line):
    """这一行 shell 注释从哪起（引号外、行首或空白之后的 #）；没有注释交回 None。弄坏开关 shell-comments-ignored 下一律当没有注释。"""
    if "shell-comments-ignored" in broken:
        return None
    quote, index = None, 0
    while index < len(line):
        character = line[index]
        if quote == "'":
            quote = None if character == "'" else quote
        elif character == "\\":
            index += 2
            continue
        elif quote == '"':
            quote = None if character == '"' else quote
        elif character in "'\"":
            quote = character
        elif character == "#" and (index == 0 or line[index - 1] in " \t"):
            return index
        index += 1
    return None


def comment_line(line):
    """整行是 shell 注释（去掉缩进之后以 # 起头）：续行接到它就断，它自己也不续行（bash 里注释到行尾为止，注释里的反斜杠不是续行）。"""
    return shell_comment_start(line) is not None and not line[:shell_comment_start(line)].strip()


def continued(line):
    """行尾是没转义的反斜杠（shell 的续行），而且这个反斜杠不在注释里。"""
    return shell_comment_start(line) is None and (len(line) - len(line.rstrip("\\"))) % 2 == 1


def judge_entries_called(source_text):
    """源文件调了的判段入口（判据 ③）：提到两个模块之一（路径、导入、别名导入都算），又写着判段入口表里的名字；没调交回空表。"""
    if "judge-by-module-marker" in broken:
        return [f"{module}::" for module in modules if f"{module}::" in source_text]
    if not MODULE_WORD.search(source_text):
        return []
    return sorted(set(ENTRY_WORD.findall(source_text)))


def manifests():
    """→ {(包名, 目标种类 bin 或 test, 目标名): 源文件路径}：读 crates/*/Cargo.toml 与 scripts/experiment-apparatus/Cargo.toml
    （[[bin]]、[[test]] 的 name 与 path，外加 src/bin/ 与 tests/ 下的自动发现）。"""
    found = {}
    for manifest in sorted(glob.glob("crates/*/Cargo.toml")) + ["scripts/experiment-apparatus/Cargo.toml"]:
        if not os.path.isfile(manifest):
            continue
        try:
            with open(manifest, "rb") as handle:
                parsed = tomllib.load(handle)
        except (OSError, tomllib.TOMLDecodeError):
            continue
        package = parsed.get("package", {}).get("name")
        if not package:
            continue
        directory = os.path.dirname(manifest)
        for kind, folder in (("bin", "src/bin"), ("test", "tests")):
            for source in glob.glob(os.path.join(directory, folder, "*.rs")):
                found[(package, kind, os.path.basename(source)[:-3])] = source
            for source in glob.glob(os.path.join(directory, folder, "*", "main.rs")):
                found[(package, kind, os.path.basename(os.path.dirname(source)))] = source
            for target in parsed.get(kind, []):
                if target.get("name") and target.get("path"):
                    found[(package, kind, target["name"])] = os.path.normpath(os.path.join(directory, target["path"]))
    return found


def command_words(tail):
    """命令那一截照 shell 切词（引号里的空白不切，「--features "lookup-work-counters gpu"」是一个值）；引号没配对的退回按空白切。
    弄坏开关 features-spelling-narrow 下按空白切。"""
    if "features-spelling-narrow" in broken:
        return tail.split()
    try:
        return shlex.split(tail, comments=False, posix=True)
    except ValueError:
        return tail.split()


def command_options(command, tail):
    """cargo run / cargo test 之后那一截 → (包名或 None, [(目标种类, 目标名)], 特性集合, 带了 --all-features 吗)。
    目标：--bin <名>（--bin=<名>）；cargo test 另认 --test <名>（--test=<名>），弄坏开关 test-targets-unjudged 下不认。
    特性：--features <值>、--features=<值>、-F <值>、-F<值>，可写几次、取并集，值里逗号与空白都算分隔；弄坏开关 features-spelling-narrow 下 -F 只认与值分开写的。"""
    words = command_words(tail)
    narrow = "features-spelling-narrow" in broken
    target_options = {"--bin": "bin"}
    if command == "test" and "test-targets-unjudged" not in broken:
        target_options["--test"] = "test"
    package = None
    targets, features, all_features = [], set(), False
    index = 0
    while index < len(words):
        word = words[index]
        following = words[index + 1] if index + 1 < len(words) else ""
        if word in ("-p", "--package"):
            package, index = following, index + 2
            continue
        if word.startswith("--package="):
            package = word.split("=", 1)[1]
        elif word.startswith("-p") and len(word) > 2:
            package = word[2:]
        elif word in target_options:
            targets.append((target_options[word], following))
            index += 2
            continue
        elif "=" in word and word.split("=", 1)[0] in target_options:
            option, name = word.split("=", 1)
            targets.append((target_options[option], name))
        elif word in ("--features", "-F"):
            features.update(item for item in re.split(r"[,\s]+", following.strip("'\"")) if item)
            index += 2
            continue
        elif word.startswith("--features="):
            features.update(item for item in re.split(r"[,\s]+", word.split("=", 1)[1].strip("'\"")) if item)
        elif word.startswith("-F") and len(word) > 2 and not narrow:
            features.update(item for item in re.split(r"[,\s]+", word[2:].strip("'\"")) if item)
        elif word == "--all-features":
            all_features = True
        index += 1
    return package, targets, features, all_features


with open(changed_file, encoding="utf-8") as handle:
    changed = sorted({line.strip() for line in handle if line.strip()})
registrations = [path for path in changed if NAME.fullmatch(path) and os.path.isfile(path)]
if not registrations:
    print("  ! 这次改动里没有新建或改动的跑前登记（research/prompts/e<号>…prereg….md），这一格无对象可判")
    sys.exit(77)
binaries = manifests()
judged, problems, unknown, other, without_binary, not_command_position = [], [], [], [], 0, 0
for path in registrations:
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    in_fence = False
    following = 0
    while following < len(lines):
        line = lines[following]
        following += 1
        number = following
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        # 反斜杠续行的几行并成一条再认（判据 ①）：围栏里的行与围栏外行首是命令的行都并，行号记头一行；
        # shell 注释不续行、续行接到整行注释就断（comment_line）；弄坏开关 continuation-unjoined 逐行认
        joinable = in_fence or (not line.lstrip().startswith("|") and bool(command_spans(line, False)))
        while (joinable and "continuation-unjoined" not in broken and continued(line)
               and following < len(lines) and not FENCE.match(lines[following]) and not comment_line(lines[following])):
            line = line[:-1] + lines[following]
            following += 1
        # 命令那一段（围栏里的行、行首是命令的行、表格整格的命令）行尾的 shell 注释截掉，注释里的字不当命令的参数，注释里的 cargo 不判也不计数；
        # 弄坏开关 trailing-comments-kept-outside-fence 只截围栏里的
        spans, comment_regions = [], []
        for start, end in command_spans(line, in_fence):
            cut = in_fence or "trailing-comments-kept-outside-fence" not in broken
            comment = shell_comment_start(line[start:end]) if cut else None
            if comment is not None:
                comment_regions.append((start + comment, end))
                end = start + comment
            spans.append((start, end))
        for found in CARGO.finditer(line):
            if any(comment_from <= found.start() < comment_to for comment_from, comment_to in comment_regions):
                continue
            if not in_command_position(line, spans, found.start()):
                not_command_position += 1
                continue
            span_end = next((end for start, end in spans if start <= found.start() < end), len(line))
            tail = line[found.end():span_end]
            end = COMMAND_END.search(tail)
            tail = tail[:end.start()] if end else tail
            package, targets, features, all_features = command_options(found.group("command"), tail)
            written = " ".join(f"cargo {found.group('command')}{tail}".split())
            if not targets:
                without_binary += 1
                continue
            where = f"{path}:{number}"
            for kind, target in targets:
                candidates = [source for (owner, owned_kind, name), source in binaries.items()
                              if owned_kind == kind and name == target and (package is None or owner == package)]
                if len(candidates) != 1:
                    reason = f"清单里找不到这个 {kind}" if not candidates else f"没给包名，{len(candidates)} 个包都有同名的 {kind}"
                    unknown.append(f"{where}（--{kind} {target}：{reason}）")
                    continue
                try:
                    source_text = open(candidates[0], encoding="utf-8", errors="replace").read()
                except OSError:
                    unknown.append(f"{where}（--{kind} {target}：源文件 {candidates[0]} 读不出）")
                    continue
                called = judge_entries_called(source_text)
                if not called and "judge-binaries-all-counted" not in broken:
                    other.append(f"{where}（{target}）")
                    continue
                judged.append(where)
                has_gpu = all_features or any(feature == "gpu" or feature.endswith("/gpu") for feature in features)
                if not has_gpu and "judge-features-unchecked" not in broken:
                    carried = "、".join(sorted(features)) if features else "无"
                    problems.append(f"{where}：「{written}」起的 {target}（{candidates[0]}）调了判段入口 {'、'.join(called) or '（弄坏开关下一律算）'}，"
                                    f"特性没带 gpu（带的：{carried}）")
unknown_text = f"；认不出、本次未判 {len(unknown)} 条：{'、'.join(unknown)}" if unknown else ""
other_text = f"；不走判段的 {len(other)} 条不判：{'、'.join(other)}" if other else ""
position_text = f"；不在命令位置（行文里内嵌、标签之后、表格里夹着字、围栏里的输出行）的 {not_command_position} 条不判"
if unknown:
    print(f"  ! 起装置的命令本次未判 {len(unknown)} 条：{'、'.join(unknown)}")
    print("     要这一格判它：命令写 -p <包名> --bin <名>（cargo test 也可 --test <名>），目标要在被判的根的 crates/*/Cargo.toml 或 scripts/experiment-apparatus/Cargo.toml 的清单里（[[bin]]、[[test]]，或 src/bin/、tests/ 下）")
    not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
    if not_run_file:
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"doc-experiments.sh preregistration-judge-features：起装置的命令本次未判 {len(unknown)} 条（{'、'.join(unknown)}）\n")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 这次改动的跑前登记里 {len(problems)} 条起走崩溃判段的装置命令没带 gpu 特性（走判段的判了 {len(judged)} 条）")
    print("     → 怎么办：照 .claude/agents/experiment-designer.md 写起法：装置走到崩溃判段（crash_amplification / crash_state_enumeration 的判段）的，"
          "cargo run / cargo test 带 --features gpu（经 task-server/task submit 交，清场由服务做），登记里写核 NOT_RUN_ 0 行；改完单跑这一格（bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features）")
    sys.exit(1)
if not judged and not other:
    print(f"  ! 这次改动的跑前登记（{len(registrations)} 份）里没有认得出的、点名 --bin 或 --test 的 cargo run / cargo test（没点名 --bin 或 --test 的 {without_binary} 条），"
          f"这一格无对象可判{unknown_text}{position_text}")
    sys.exit(77)
print(f"  ✓ 跑前登记里起走崩溃判段的装置命令都带了 gpu 特性：查了 {len(registrations)} 份，走判段的 {len(judged)} 条"
      f"{('（' + '、'.join(judged) + '）') if judged else ''}{other_text}；没点名 --bin 或 --test 的 {without_binary} 条{unknown_text}{position_text}")
PY
}

# ── 格 judge-entries-in-sync：判段入口表 JUDGE_ENTRIES 与两份判段源文件里的公开判段入口逐个相同
# 为什么：preregistration-judge-features 只认 JUDGE_ENTRIES 里的名字，两个模块里新加了公开入口而表没跟上，走判段的装置就漏判。
# 对象：被判的根里 JUDGE_SOURCES 那两份源文件（crash_amplification.rs、crash_state_enumeration.rs）此刻的全文，不看这次改动范围；两份都不在退 77。
# 认法（公开的判段入口）：
#   ① 切函数单元：去掉注释（// 与可嵌套的 /* */）、把串、原始串、字节串与字符字面量涂成空白之后配花括号；花括号前面那一截表头从方括号与圆括号外的
#      上一个分号或右花括号之后算起（签名里数组类型 [u8; N] 的分号在方括号里，不算）；文件顶层与行内 mod 里的 fn 各一个单元，
#      inherent impl（impl <类型> {…}，不带 for）里的 fn 记在那个类型下；函数体里嵌套的 fn 并进外层。#[cfg(test)] 的模块、带 #[test] 或 #[cfg(test)] 的 fn、
#      trait 的实现（impl … for …）里的方法不算。
#   ② 判段去路的记号：函数体（去掉注释、字面量照留）里有 cfg!(feature = "gpu")（这份二进制编没编进 gpu 特性，判段照它选去路），或调 judge_not_run_without_the_gpu_feature（判段不跑的原因）。
#   ③ 调用边只在这两份源文件的单元之间连：裸名 <名>( 与路径末段连到同名的自由函数；<类型>::<名> 与 Self::<名> 连到那个类型下的同名方法；.<名> 连到任一类型下的同名方法。
#   ④ 公开（可见性正是 pub，pub(crate) 这类不算）、顺调用边走得到带记号的单元（自己带记号也算）的，是公开的判段入口：自由函数记函数名，方法记它所在的类型名
#      （装置写 <类型>::<方法>，preregistration-judge-features 按类型名认）。走得到的单元从带记号的单元沿调用边倒着一次走全，环上的单元与函数在文件里的先后都不改结局。
# 判据：④ 认出的名字集合与 JUDGE_ENTRIES 逐个相同。源码里认出而表里没有的（新加的入口）判红，点名文件、行与限定名；表里有而源码里认不出的
#   （改了名、删了、不再公开、走不到记号）判红，点名那个名字。
# 管不到的：入口经别的模块（gpu_card_configuration.rs 这类）选去路、两份源文件里没有记号的；宏展开出来的函数；调用写在宏参数之外的别处（函数指针存进表再调）——
#   这几种 ③ 连不上边，漏判。认出的多了（.<名> 连到同名方法）只会多报、不漏。
# 样本：judge-entries-in-sync-green（两份源文件里恰好是表里那 13 个入口；另有只在注释里写 cfg!(feature = "gpu") 的公开函数、#[cfg(test)] 模块里调判段的公开函数、
#   不调记号的公开纯函数 judge_not_run_without_the_gpu_feature，都不算入口）；judge-entries-in-sync-red（同一份再加一个调 enumerate_crash_states 的公开函数，表里没有，判红）；
#   judge-entries-in-sync-stale-red（crash_point_replay_judge_not_run_line 改成不公开，表里有、源码里认不出，判红）；
#   签名带数组与调用成环的三份：judge-entries-in-sync-array-parameter-new-entry-red
#   （green 那两份源文件末尾加签名里带 [u8; 32] 的公开入口 enumerate_crash_states_keyed_by_the_plan，经 enumerate_crash_states 走到判段：第 49 行判红）、judge-entries-in-sync-array-parameter-on-the-path-green（路径上的私有函数 enumerate_crash_states_through_the_library 签名里带
#   [u8; 32]：判绿）、judge-entries-in-sync-cycle-new-entry-after-red（新公开入口
#   enumerate_crash_states_one_more_round 与 enumerate_crash_states 互相调、写在它后面：第 49 行判红）。
# 弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 judge-entries-new-unreported、judge-entries-stale-unreported、judge-entries-header-after-any-semicolon、
# judge-entries-cycle-memoized 四项。
# 退役判据：preregistration-judge-features 退役时一起删。
JUDGE_SOURCES="crates/singlefs-verification/src/crash_amplification.rs crates/singlefs-verification/src/crash_state_enumeration.rs"
cell_judge_entries_in_sync() {
  python3 - "$JUDGE_SOURCES" "$JUDGE_ENTRIES" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" <<'PY'
import os, re, sys

sources, entries, broken = sys.argv[1].split(), set(sys.argv[2].split()), sys.argv[3].split(",")
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
RAW_STRING = re.compile(r'(?:b|c|br|cr)?r(#*)"')
ROUTE_MARKER = re.compile(r'cfg!\s*\(\s*feature\s*=\s*"gpu"\s*\)|\bjudge_not_run_without_the_gpu_feature\s*\(')
TEST_ATTRIBUTE = re.compile(r"#\s*\[\s*(?:cfg\s*\(\s*test\s*\)|test)\s*\]")
CALL = re.compile(r"(?:(\.)\s*|\b(Self|[A-Z][A-Za-z0-9_]*)\s*::\s*)?\b([a-z_][A-Za-z0-9_]*)\b")


def blank(text, start, end):
    return "".join(character if character == "\n" else " " for character in text[start:end])


def lex(text):
    """认法 ①：→ (去掉注释的原文, 去掉注释、字面量涂成空白的代码)，两份与原文等长、换行留在原处。"""
    without_comments, code = [], []
    position, count = 0, len(text)

    def word_before(index):
        return index > 0 and (text[index - 1].isalnum() or text[index - 1] == "_")

    def keep(end, literal):
        without_comments.append(text[position:end] if literal else blank(text, position, end))
        code.append(blank(text, position, end))
        return end

    while position < count:
        character = text[position]
        if text.startswith("//", position):
            end = text.find("\n", position)
            position = keep(count if end < 0 else end, False)
            continue
        if text.startswith("/*", position):
            depth, end = 1, position + 2
            while end < count and depth:
                if text.startswith("/*", end):
                    depth, end = depth + 1, end + 2
                elif text.startswith("*/", end):
                    depth, end = depth - 1, end + 2
                else:
                    end += 1
            position = keep(end, False)
            continue
        raw = RAW_STRING.match(text, position) if character in "bcr" and not word_before(position) else None
        if raw:
            end = text.find('"' + raw.group(1), raw.end())
            position = keep(count if end < 0 else end + 1 + len(raw.group(1)), True)
            continue
        if character == '"' or (character in "bc" and text.startswith('"', position + 1) and not word_before(position)):
            end = text.index('"', position) + 1
            while end < count and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            position = keep(min(end + 1, count), True)
            continue
        if character == "'":
            end = position + 1   # 生命周期：只跳过引号
            if text.startswith("\\", position + 1):
                end = position + 2
                while end < count and text[end] != "'":
                    end += 2 if text[end] == "\\" else 1
                end = min(end + 1, count)
            elif position + 2 < count and text[position + 2] == "'":
                end = position + 3
            if end > position + 1:
                position = keep(end, True)
                continue
        identifier = IDENTIFIER.match(text, position) if (character.isalpha() or character == "_") else None
        end = identifier.end() if identifier else position + 1
        without_comments.append(text[position:end])
        code.append(text[position:end])
        position = end
    return "".join(without_comments), "".join(code)


def header_start(code, low, brace):
    """认法 ①：花括号前面那一截表头从哪起——往回找方括号与圆括号外的上一个分号或右花括号，表头从它之后算；没有就从 low 起。
    弄坏开关 judge-entries-header-after-any-semicolon 下不看括号（签名里 [u8; N] 的分号也算，表头里认不出 fn）。"""
    if "judge-entries-header-after-any-semicolon" in broken:
        return max(low, code.rfind(";", low, brace) + 1, code.rfind("}", low, brace) + 1)
    depth = 0
    for index in range(brace - 1, low - 1, -1):
        character = code[index]
        if character in ")]":
            depth += 1
        elif character in "([":
            depth -= 1
        elif character in ";}" and depth <= 0:
            return index + 1
    return low


def function_units(path):
    """认法 ①：→ [{name, owner, public, test, line, code, marked}]；owner 是 inherent impl 的类型名，自由函数是 None。"""
    text = open(path, encoding="utf-8", errors="replace").read()
    without_comments, code = lex(text)
    pairs, stack = {}, []
    for index, character in enumerate(code):
        if character == "{":
            stack.append(index)
        elif character == "}" and stack:
            pairs[stack.pop()] = index
    found = []

    def scan(low, high, owner, test_scope):
        position = low
        while position < high:
            brace = code.find("{", position, high)
            if brace < 0:
                return
            header_from = header_start(code, position, brace)
            header, closing = code[header_from:brace], pairs.get(brace, high)
            test = test_scope or bool(TEST_ATTRIBUTE.search(header))
            function = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", header)
            implementation = re.search(r"(?:^|\s|\])impl\b(.*)$", header, re.S)
            if function:
                visibility = header[:function.start()].split("]")[-1]
                found.append({"name": function.group(1), "owner": owner, "test": test,
                              "public": bool(re.search(r"\bpub\b(?!\s*\()", visibility)),
                              "line": text.count("\n", 0, header_from + function.start()) + 1, "code": code[brace:closing + 1],
                              "marked": bool(ROUTE_MARKER.search(without_comments[brace:closing + 1]))})
            elif implementation:
                rest = implementation.group(1).strip()
                if rest.startswith("<"):
                    depth = 0
                    for cut, character in enumerate(rest):
                        depth += {"<": 1, ">": -1}.get(character, 0)
                        if depth == 0:
                            rest = rest[cut + 1:]
                            break
                if re.search(r"\bfor\b", rest):
                    scan(brace + 1, closing, None, True)
                else:
                    type_name = re.match(r"\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)", rest)
                    scan(brace + 1, closing, type_name.group(1) if type_name else None, test)
            elif re.search(r"\bmod\s+[A-Za-z_][A-Za-z0-9_]*\s*$", header):
                scan(brace + 1, closing, None, test)
            position = closing + 1

    scan(0, len(code), None, False)
    for unit in found:
        unit["path"] = path
    return found


present = [path for path in sources if os.path.isfile(path)]
if not present:
    print(f"  ! 被判的根里两份判段源文件（{'、'.join(sources)}）都不在，这一格无对象可判")
    sys.exit(77)
units = [unit for path in present for unit in function_units(path) if not unit["test"]]
free_functions, methods = {}, {}
for unit in units:
    (free_functions if unit["owner"] is None else methods).setdefault(unit["name"], []).append(unit)


def callees(unit):
    """认法 ③。"""
    found = []
    for match in CALL.finditer(unit["code"]):
        dot, qualifier, name = match.groups()
        if dot or qualifier:
            owner = unit["owner"] if qualifier == "Self" else qualifier
            found += [method for method in methods.get(name, []) if dot or method["owner"] == owner]
        else:
            found += free_functions.get(name, [])
    return found


reaches = {}


def reaches_route(unit, visiting):
    """弄坏开关 judge-entries-cycle-memoized 下的走法：递归、记住每个单元的结果，环上截断时算出的「走不到」也记住（结局随先后翻）。"""
    if id(unit) in reaches:
        return reaches[id(unit)]
    if id(unit) in visiting:
        return False
    visiting.add(id(unit))
    result = unit["marked"] or any(reaches_route(callee, visiting) for callee in callees(unit))
    visiting.discard(id(unit))
    reaches[id(unit)] = result
    return result


def units_reaching_route():
    """认法 ④：顺调用边走得到带记号的单元（自己带记号也算）→ 它们的 id 集合。从带记号的单元沿调用边倒着一次走全，
    环与函数在文件里的先后都不改结局。"""
    if "judge-entries-cycle-memoized" in broken:
        return {id(unit) for unit in units if unit["public"] and reaches_route(unit, set())}
    callers = {}
    for unit in units:
        for callee in callees(unit):
            callers.setdefault(id(callee), []).append(unit)
    pending = [unit for unit in units if unit["marked"]]
    reached = {id(unit) for unit in pending}
    while pending:
        for caller in callers.get(id(pending.pop()), []):
            if id(caller) not in reached:
                reached.add(id(caller))
                pending.append(caller)
    return reached


reaching = units_reaching_route()
recognized = {}
for unit in units:
    if unit["public"] and id(unit) in reaching:
        qualified = f"{unit['owner']}::{unit['name']}" if unit["owner"] else unit["name"]
        recognized.setdefault(unit["owner"] or unit["name"], []).append(f"{unit['path']}:{unit['line']} {qualified}")
new_entries = [] if "judge-entries-new-unreported" in broken else sorted(set(recognized) - entries)
stale_entries = [] if "judge-entries-stale-unreported" in broken else sorted(entries - set(recognized))
for name in new_entries:
    print(f"  ✗ 源码里的公开判段入口 {name}（{'、'.join(recognized[name])}）不在 JUDGE_ENTRIES 里")   # gate-lint:detail
for name in stale_entries:
    print(f"  ✗ JUDGE_ENTRIES 里的 {name} 在两份判段源文件里认不出是公开的判段入口（没有这个公开的函数或类型方法，或走不到判段去路的记号）")   # gate-lint:detail
marked = sum(unit["marked"] for unit in units)
public = sum(unit["public"] for unit in units)
if new_entries or stale_entries:
    print(f"  ✗ 判段入口表 JUDGE_ENTRIES 与两份判段源文件对不上：源码里有、表里没有的 {len(new_entries)} 个，表里有、源码里认不出的 {len(stale_entries)} 个"
          f"（扫了 {len(units)} 个函数单元，公开的 {public} 个，带判段去路记号的 {marked} 个）")
    print("     → 怎么办：源码里新加的入口把名字加进 .claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh 的 JUDGE_ENTRIES（方法写类型名）；表里认不出的，改名的换成新名字、"
          "删了或不再走判段的从表里删；认法在这一格函数上方的注释。改完单跑这一格（bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync）")
    sys.exit(1)
print(f"  ✓ 判段入口表 JUDGE_ENTRIES 的 {len(entries)} 个名字与两份判段源文件里认出的公开判段入口逐个相同"
      f"（扫了 {len(present)} 份、{len(units)} 个函数单元，公开的 {public} 个，带判段去路记号的 {marked} 个）")
PY
}

# ── 格 replay-comparison-units：重跑登记的比对对象，在单元表与 replay.sh 里都找得到，与重跑登记里的比对对象表逐行对得上
#
# 还着 .claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 3 行改法的后半句（另立一条检查：每份重跑登记的比对对象表里的每条正式跑，
# 在单元表和 replay.sh 里都找得到）。比对对象表在各份重跑登记里是散文表、写法各不相同，所以先立固定形态再立格：
# 机读登记 .claude/gate.d/replay-comparison-units.tsv，一行一个比对对象，六列（实验号、登记出处、单元名、单元表出处、留存产物名、复跑），列义写在那份文件头。
# 登记行由执行员在实现单元、登记复跑的同一次改动里加（阶段归属表这一格那一行）。
# 重跑登记里的比对对象表有固定形态（.claude/agents/experiment-designer.md 第 1 步）：表头一行逐字是
#   | # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
#   下一行是分隔行，之后每一行一个正式跑：# 列是正整数、一份登记里不重号，比对对象与重跑产物名两格各是一个文件名（可带一对反引号），七格；围栏代码块里的不算。
# 判四样，前三样每行各判：
#   ① 登记出处（仓内路径）的文件存在；
#   ② 单元名作为带双引号的字符串字面量在单元表出处那份文件里恰好出现一次（0 次是找不到，两次以上是指不到唯一一个单元；「甲」不会在「乙甲」里被数到）；
#   ③ 复跑列写驱动名的，scripts/experiment-apparatus/replay.sh 复跑表里有实验号相同、第 2 列是 @驱动名的行，且其中一行的第 4 列（期望产物）等于留存产物名。
#      复跑表的行经 lib-lag-baseline.py 的 replay_rows 取（与 results-cited、verdict-false-named、quoted-result-lines 三格同一份读法）。
#      复跑列写 none:理由 的不判③，改判只缩不涨（见下）。
# 只缩不涨：复跑写 none: 的行按（实验号，登记出处，单元名，留存产物名）与基准提交那一版登记表里复跑写 none: 的行比，基准里没有的判红；
#   四样都进键：只按（登记出处，留存产物名）成对比时，把一行已有 none 行的实验号或单元名换掉、借它的登记就照绿（样本 replay-comparison-units-rekey-red）。
#   重复行的判法（同一登记出处、同一留存产物名不许两行）不随这个键变。
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句写明；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 baseline_rows）。
#   补了复跑行（复跑列改写驱动名）的那一行自然不再是 none 行，不用另删什么。
#   ④ 比对对象表与登记表逐行对：判的登记是登记表点名的登记出处，加 research/prompts/ 顶层 e<号> 起头的 .md 里有固定形态表头的那几份。
#      登记表点名的登记出处里没有固定形态的表，判红。一行登记「对上」表里一行：留存产物名等于那一行的比对对象或重跑产物名；
#      重跑产物名不是 e<号>- 起头的全名（旧登记只写前缀之后那一段，如 E165 r5 的「重跑产物名都在前缀 e165-replay-bounded-enumeration-<日期>-r5- 之后」），
#      留存产物名以「-<那一段>」收尾也算对上——旧登记就这样认，不改冻结的登记。
#      表里每一行要有一行登记对上，或在登记表文件头点名不登记（一行一条，行首写「# 不登记的比对对象：<登记出处> #<号>：<理由>」，理由至少 4 个字）；
#      两样都没有、两样都有，判红。登记表里每一行要在它登记出处的表里对上一行，对不上判红。文件头点名的那一行在表里没有、写法不对、同一行点名两次，判红。
#      点名不登记的每一行，理由里要写「欠在 C<号>（简称）」，号要在 .claude/kb/checks-owed.md 开着的那张表里（读法 lib-owed.py）：没写、号不在、号已还清、
#      欠账表不在或认不出「已还清」标题，都判红，点名那一行（单元表里补不补单元归那条欠账，点名不登记就是登记在那条欠账上；这一判替掉了「点名行与基准比、只缩不涨」）。
#      成功句报比对对象表几份几行、对上几行、点名不登记几行。
# 登记表一行都没有（只有注释）而也没有一份登记里有固定形态的表，退 77，不记通过。
# 管不到：表头不是逐字那一行的比对对象表（登记表也没点名那份登记时，这一格看不见它）、sha256 与行数两列对不对、点名不登记的理由除欠账号之外成不成立
#   （欠账号指到的那条账是不是真在说这一行、简称与登记的对不对得上，靠 review）、留存产物名指到的文件在不在、单元表里没有登记的单元、replay.sh 里没有登记的驱动行。
# 样本：replay-comparison-units-red / -green / -zero / -rekey-red / -table-red / -table-green / -exclusion-owed-red，弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 comparison-* 各项。
# 退役判据：单元表与复跑表合成一份机读登记、这张登记表不再有人写时删掉这一格。
cell_replay_comparison_units() {
  UNITS=.claude/gate.d/replay-comparison-units.tsv
  REPLAY=scripts/experiment-apparatus/replay.sh
  [[ -f "$UNITS" ]] || { echo "  ! 没有 $UNITS，这一格无对象可判"; exit 77; }
  base=""
  if ((in_git_work_tree)); then
    if ((! changed_paths_library_loaded)); then
      echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    base="$shared_base"
  fi

  python3 - "$UNITS" "$REPLAY" "$base" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" "$LIB_LAG_BASELINE" "$STAGE_DIR/lib-owed.py" <<'PY'
import glob, importlib.util, os, re, sys

units_path, replay_path, base, broken_text, library, owed_library_path = sys.argv[1:7]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", library)
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
broken = set(broken_text.split(","))
units_name = os.path.basename(units_path)
NONE_PREFIX = "none:"
COLUMNS = 6

def none_identity(experiment, registration, unit, product):
    """复跑写 none: 的一行在「只缩不涨」里的键：（实验号，登记出处，单元名，留存产物名）。
    弄坏开关 comparison-none-key-pair 按（登记出处，留存产物名）成对：换实验号、换单元名借同一对键照绿。"""
    if "comparison-none-key-pair" in broken:
        return (registration, product)
    return (experiment, registration, unit, product)

def none_key(fields):
    """登记表一行复跑写 none: 的，交它的键（none_identity）；别的交 None。"""
    if len(fields) != COLUMNS or not fields[5].strip().startswith(NONE_PREFIX):
        return None
    return none_identity(*(fields[index].strip() for index in (0, 1, 2, 4)))

sources = {}
def source_text(path):
    if path not in sources:
        sources[path] = open(path, encoding="utf-8", errors="replace").read() if os.path.isfile(path) else None
    return sources[path]

# 重跑登记里比对对象表的固定形态（.claude/agents/experiment-designer.md 第 1 步）
FIXED_HEADER = "| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |"
TABLE_COLUMNS = 7
SEPARATOR_ROW = re.compile(r"\s*\|(\s*:?-{3,}:?\s*\|)+\s*")
FENCE_OPEN = re.compile(r" {0,3}(`{3,}|~{3,})")
FULL_PRODUCT_NAME = re.compile(r"e[0-9]+-")
EXCLUSION_START = re.compile(r"#\s*不登记的比对对象：")
EXCLUSION = re.compile(r"#\s*不登记的比对对象：(?P<registration>\S+)\s+#(?P<row>[0-9]+)：(?P<reason>.+)")
MINIMUM_REASON_CHARACTERS = 4
# 点名不登记的每一行指到的欠账：理由里写「欠在 C<号>（简称）」，号要在欠账表开着的那张表里（读法 lib-owed.py，与读欠账表的各格同一份）
OWED_PATH = ".claude/kb/checks-owed.md"
OWED_REFERENCE = re.compile(r"欠在\s*(?P<number>C[0-9]+)（(?P<name>[^（）]+)）")
owed_spec = importlib.util.spec_from_file_location("lib_owed", owed_library_path)
owed_library = importlib.util.module_from_spec(owed_spec)
owed_spec.loader.exec_module(owed_library)

def exclusion_owed_problem(reason, owed):
    """点名不登记的一行的理由指没指到一条开着的欠账：指到了交 None，没指到交一句。弄坏开关 comparison-exclusion-owed-unchecked 不判。"""
    if "comparison-exclusion-owed-unchecked" in broken:
        return None
    found = OWED_REFERENCE.search(reason)
    if not found:
        return "「不登记的比对对象」没写欠在哪一条账（理由里要写「欠在 C<号>（简称）」）"
    number = found.group("number")
    if not owed.file_found:
        return f"写的 {number} 查不了：{OWED_PATH} 不在"
    if not owed.paid_heading_found:
        return f"写的 {number} 查不了：{OWED_PATH} 里认不出「已还清」那一行整行标题，分不出哪些欠账还开着"
    if number in owed.paid_names:
        return f"写的 {number}（{owed.paid_names[number]}）在 {OWED_PATH} 里已还清：还清了就该补上单元、改登记成正式行，不再点名不登记"
    if number not in owed.open_names:
        return f"写的 {number} 在 {OWED_PATH} 里没有登记"
    return None

def markdown_cells(line):
    """表格的一行切成格：反引号里的 | 与 \\| 不切。"""
    body = line.strip()
    body = body[1:] if body.startswith("|") else body
    body = body[:-1] if body.endswith("|") and not body.endswith("\\|") else body
    cells, current, in_code, index = [], [], False, 0
    while index < len(body):
        character = body[index]
        if character == "\\" and body[index + 1:index + 2] == "|":
            current.append("|")
            index += 2
            continue
        if character == "`":
            in_code = not in_code
        if character == "|" and not in_code:
            cells.append("".join(current).strip())
            current = []
        else:
            current.append(character)
        index += 1
    cells.append("".join(current).strip())
    return cells

def file_name_cell(text):
    """格里的一个文件名（可带一对反引号）；不是一个不带空白的名字交 None。"""
    name = text.strip()
    if len(name) >= 2 and name.startswith("`") and name.endswith("`"):
        name = name[1:-1].strip()
    return name if name and not re.search(r"[\s`|]", name) else None

def comparison_table(path):
    """→ (rows, problems)：rows 是 None（这份登记里没有表头逐字的比对对象表）或 [{row, object, rerun, line}]；围栏代码块里的不算。"""
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError as error:
        return None, [f"{path}：读不了（{error.strerror}）"]
    rows, problems, fence, numbered, index = None, [], None, {}, 0
    while index < len(lines):
        line = lines[index].rstrip("\r")
        index += 1
        opener = FENCE_OPEN.match(line)
        if fence is not None:
            if opener and opener.group(1)[0] == fence[0] and len(opener.group(1)) >= len(fence) and not line.strip()[len(opener.group(1)):].strip():
                fence = None
            continue
        if opener:
            fence = opener.group(1)
            continue
        if line.strip() != FIXED_HEADER:
            continue
        rows = [] if rows is None else rows
        if index >= len(lines) or not SEPARATOR_ROW.fullmatch(lines[index].rstrip("\r")):
            problems.append(f"{path} 第 {index} 行是比对对象表头，下面一行不是分隔行（|---|…）")
            continue
        index += 1
        while index < len(lines) and lines[index].lstrip().startswith("|"):
            number, text = index + 1, lines[index].rstrip("\r")
            index += 1
            cells = markdown_cells(text)
            if len(cells) != TABLE_COLUMNS:
                problems.append(f"{path} 第 {number} 行：比对对象表这一行不是 {TABLE_COLUMNS} 格：{text[:90]}")
                continue
            row, comparison_object, rerun_product = cells[0], file_name_cell(cells[1]), file_name_cell(cells[5])
            if not re.fullmatch(r"[1-9][0-9]*", row):
                problems.append(f"{path} 第 {number} 行：比对对象表的 # 列要写正整数，一行一个正式跑：{row[:30]}")
                continue
            if comparison_object is None or rerun_product is None:
                problems.append(f"{path} 第 {number} 行：比对对象与重跑产物名两格各要是一个文件名（可带一对反引号）：{text[:90]}")
                continue
            if row in numbered:
                problems.append(f"{path} 第 {number} 行：比对对象表 #{row} 与第 {numbered[row]} 行重号")
                continue
            numbered[row] = number
            rows.append({"row": row, "object": comparison_object, "rerun": rerun_product, "line": number})
    return rows, problems

def covers(product, row):
    """登记表一行的留存产物名对不对得上比对对象表的这一行。"""
    if product in (row["object"], row["rerun"]):
        return True
    if FULL_PRODUCT_NAME.match(row["rerun"]) or "comparison-table-suffix-unrecognized" in broken:
        return False
    return product.endswith("-" + row["rerun"])

problems, registered, seen = [], [], {}
units_text = open(units_path, encoding="utf-8").read()
for number, fields in lag_baseline.content_rows(units_text):
    where = f"{units_name} 第 {number} 行"
    if len(fields) != COLUMNS or not all(field.strip() for field in fields):
        problems.append(f"{where}：不是六列（实验号、登记出处、单元名、单元表出处、留存产物名、复跑）或有一列是空的：{'<制表符>'.join(fields)[:90]}")
        continue
    experiment, registration, unit, unit_source, product, rerun = (field.strip() for field in fields)
    if not re.fullmatch(r"E[0-9]+", experiment):
        problems.append(f"{where}：实验号不是 E<数字> 的形状：{experiment}")
        continue
    if rerun.startswith(NONE_PREFIX):
        if len(rerun[len(NONE_PREFIX):].strip()) < 4:
            problems.append(f"{where}：复跑写了 none: 而理由不到 4 个字：{rerun}")
            continue
    elif not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", rerun):
        problems.append(f"{where}：复跑不是 replay.sh 里的驱动名（不带 @），也不是 none:理由：{rerun}")
        continue
    key = (registration, product)
    if key in seen:
        problems.append(f"{where}：和第 {seen[key]} 行是同一条（登记出处与留存产物名都相同）：{product}")
        continue
    seen[key] = number
    registered.append((number, experiment, registration, unit, unit_source, product, rerun))

replay = lag_baseline.replay_rows(replay_path)
driver_rows = 0
none_rows = []
for number, experiment, registration, unit, unit_source, product, rerun in registered:
    where = f"{units_name} 第 {number} 行（{experiment}「{unit}」）"
    if "comparison-registration-unchecked" not in broken and not os.path.isfile(registration):
        problems.append(f"{where}：登记出处不在：{registration}")
    if "comparison-unit-unchecked" not in broken:
        text = source_text(unit_source)
        if text is None:
            problems.append(f"{where}：单元表出处不在：{unit_source}")
        else:
            count = text.count(unit) if "comparison-unit-substring" in broken else text.count(f'"{unit}"')
            if count == 0:
                problems.append(f"{where}：单元名「{unit}」在 {unit_source} 里一次都没找到（要作为带双引号的字符串字面量出现）")
            elif count > 1 and "comparison-unit-count-loose" not in broken:
                problems.append(f"{where}：单元名「{unit}」在 {unit_source} 里出现 {count} 次，不是恰好一次，指不到唯一一个单元")
    if rerun.startswith(NONE_PREFIX):
        none_rows.append((number, experiment, registration, unit, product))
        continue
    driver_rows += 1
    if replay is None:
        problems.append(f"{where}：读不了 {replay_path}（不在，或没有读权限），驱动行没法核")
        continue
    mine = [row for row in replay if row.experiment == experiment and row.driver == "@" + rerun]
    if not mine:
        if "comparison-driver-unchecked" not in broken:
            problems.append(f"{where}：{replay_path} 复跑表里没有 {experiment} 的驱动行 @{rerun}")
    elif "comparison-product-unchecked" not in broken and product not in {row.product for row in mine}:
        problems.append(f"{where}：@{rerun} 那一行的产物名是 {'、'.join(sorted({row.product for row in mine}))}，不是留存产物名 {product}")

# ④ 比对对象表与登记表逐行对
table_count = table_rows = covered_rows = excluded_rows = 0
table_checked = "comparison-table-unchecked" not in broken
if table_checked:
    exclusions = {}
    owed_table = owed_library.read_owed_table(OWED_PATH)
    for number, line in enumerate(units_text.split("\n"), 1):
        stripped = line.strip()
        if not EXCLUSION_START.match(stripped):
            continue
        found = EXCLUSION.fullmatch(stripped)
        if not found or len(found.group("reason").strip()) < MINIMUM_REASON_CHARACTERS:
            problems.append(f"{units_name} 第 {number} 行：「不登记的比对对象」要写成「# 不登记的比对对象：<登记出处> #<号>：<理由>」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字：{stripped[:90]}")
            continue
        owed_problem = exclusion_owed_problem(found.group("reason"), owed_table)
        if owed_problem:
            problems.append(f"{units_name} 第 {number} 行：{owed_problem}：{stripped[:90]}")
        key = (found.group("registration"), found.group("row"))
        if key in exclusions:
            problems.append(f"{units_name} 第 {number} 行：{key[0]} #{key[1]} 在第 {exclusions[key]} 行已经点名不登记")
            continue
        exclusions[key] = number
    by_registration = {}
    for item in registered:
        by_registration.setdefault(item[2], []).append(item)
    candidates = set(by_registration) | {path for path, _row in exclusions} | set(glob.glob("research/prompts/e[0-9]*.md"))
    tables = {}
    for path in sorted(path for path in candidates if os.path.isfile(path)):
        rows, table_problems = comparison_table(path)
        problems.extend(table_problems)
        if rows is not None:
            tables[path] = rows
    for path, items in sorted(by_registration.items()):
        if os.path.isfile(path) and path not in tables:
            for number, experiment, registration, unit, unit_source, product, rerun in items:
                problems.append(f"{units_name} 第 {number} 行（{experiment}「{unit}」）：登记出处 {path} 里没有表头逐字是「{FIXED_HEADER}」的比对对象表")
    for path, rows in sorted(tables.items()):
        table_count += 1
        table_rows += len(rows)
        items = by_registration.get(path, [])
        for number, experiment, registration, unit, unit_source, product, rerun in items:
            if not any(covers(product, row) for row in rows):
                problems.append(f"{units_name} 第 {number} 行（{experiment}「{unit}」）：留存产物名 {product} 在 {path} 的比对对象表里一行都对不上"
                                f"（要等于某一行的比对对象或重跑产物名）")
        for row in rows:
            covered = any(covers(item[5], row) for item in items)
            excluded = (path, row["row"]) in exclusions and "comparison-exclusions-ignored" not in broken
            described = f"{path} 比对对象表 #{row['row']}（比对对象 {row['object']}，重跑产物名 {row['rerun']}）"
            if covered and excluded:
                problems.append(f"{described}：登记表里有一行对上，文件头第 {exclusions[(path, row['row'])]} 行又点名不登记")
            elif covered:
                covered_rows += 1
            elif excluded:
                excluded_rows += 1
            else:
                problems.append(f"{described}在 {units_name} 里没有一行对得上，文件头也没点名不登记")
    for (path, row_number), number in sorted(exclusions.items()):
        if row_number not in {row["row"] for row in tables.get(path, [])}:
            problems.append(f"{units_name} 文件头「不登记的比对对象」点名的 {path} #{row_number} 在那份登记的比对对象表里没有（第 {number} 行）")

# 只缩不涨：复跑写 none: 的行与基准那一版登记表里复跑写 none: 的行比
baseline_none_keys, baseline_problem = None, None
if base:
    baseline, baseline_problem = lag_baseline.baseline_rows(base, units_path)
    if baseline is not None:
        baseline_none_keys = {key for key in (none_key(fields) for _number, fields in baseline) if key is not None}
grown = ([row for row in none_rows if none_identity(*row[1:]) not in baseline_none_keys]
         if baseline_none_keys is not None and "comparison-lag-growth-unchecked" not in broken else [])

if problems:
    print(f"  ✗ {units_path} 里 {len(problems)} 处对不上：")                     # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                                 # gate-lint:detail
    print(f"     → 怎么办：列义在 {units_path} 文件头。登记出处写仓内现存路径；单元名照单元表里的字符串字面量逐字抄，要在单元表出处里恰好出现一次；")
    print(f"               复跑写 scripts/experiment-apparatus/replay.sh 复跑表里的驱动名（不带 @），留存产物名照那一行第 4 列抄；确实没有复跑行的写 none:理由（只缩不涨）。")
    print(f"               重跑登记的比对对象表照固定形态写（表头逐字「{FIXED_HEADER}」，一行一个正式跑，.claude/agents/experiment-designer.md 第 1 步）；")
    print(f"               表里每一行要有一行登记对上（留存产物名等于比对对象或重跑产物名），单元表里没有单元的在 {units_name} 文件头写「# 不登记的比对对象：<登记出处> #<号>：<理由>」，")
    print(f"               理由里写「欠在 C<号>（简称）」：没有欠账号的，先在 `{OWED_PATH}` 登记欠账号，同一行写上；号已还清的，补上单元、改登记成正式行。")
if baseline_problem is not None:
    print(f"  ✗ {baseline_problem.sentence()}，复跑写 none: 的行只缩不涨没比")
    print("     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑；取不到基准那一版不是「基准里还没有这张表」。")
if grown:
    print(f"  ✗ none 行比基准多：{units_name} 里这几行复跑写 none:，基准 {base} 那一版里没有这样的行：")  # gate-lint:summary
    for number, experiment, registration, unit, product in grown:
        print(f"     第 {number} 行 {experiment}「{unit}」{product}（{registration}）")  # gate-lint:detail
    print("     → 怎么办：复跑写 none: 的行只缩不涨，按（实验号，登记出处，单元名，留存产物名）与基准比：换了实验号或单元名就是新的比对对象。")
    print("               新登记的比对对象要在 replay.sh 复跑表里加一行驱动行、复跑列写驱动名，不许写 none: 把它按下去，也不许改一行已有 none 行的实验号或单元名借它的登记。")
if problems or baseline_problem is not None or grown:
    sys.exit(1)

if not registered and not table_count:
    print(f"  ⊘ 本次无对象可判：{units_path} 里一行比对对象都没有")
    sys.exit(77)
compared = f"与基准 {base} 比没涨" if baseline_none_keys is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
experiments = sorted({row[1] for row in registered})
tables_compared = (f"；比对对象表 {table_count} 份 {table_rows} 行：登记表对上 {covered_rows} 行、文件头点名不登记 {excluded_rows} 行"
                   + ("" if "comparison-exclusion-owed-unchecked" in broken else "（点名行都欠在开着的欠账上）")
                   if table_checked else "")
print(f"  ✓ 比对对象 {len(registered)} 行（{'、'.join(experiments)}）：登记出处都在，单元名在单元表出处里都恰好出现一次；"
      f"复跑写驱动名的 {driver_rows} 行在 replay.sh 里都有且产物名相符；复跑写 none 的 {len(none_rows)} 行只缩不涨，{compared}{tables_compared}")
PY
}

# ── 格 pinned-apparatus-revision：钉了装置 sha256 的起跑脚本，钉值变了或装置源码变了，它的跑前登记第十二节要写着新钉值的修订、已有的行不改写、新钉值写在基准钉值之后
# 为什么非这样不可：起跑脚本把装置源码的 sha256 钉在 registered_apparatus_sha256=，开跑时装置、钉值、登记第十二节三处对不上就退 3 停机；
#   钉值是绕过这道停机的那一步（装置改了、登记没跟着写修订，起跑脚本里只把钉值改成现算值，开跑照过），所以钉值变了与装置变了都要触发。
#   这一格把它提到提交时：钉值或装置源码这次改动里变了，登记第十二节就要在这次改动里新加一行写着新钉值的修订（新 sha256、改了什么）。
# 对象：scripts/experiment-apparatus/*.sh 里行首有 registered_apparatus_sha256=<64 位小写十六进制> 的起跑脚本，值后面可以跟 # 起头的行尾注释
#   （自证里重赋成变量的写法不是 64 位十六进制，不认）。
# 每份起跑脚本怎么找装置与登记（只读脚本自己）：
#   装置源码：行首的 apparatus_source=<路径>；没有就取非注释行里 cargo 命令的 --bin <名>，在 crates/*/src/bin/<名>.rs 与 crates/*/src/bin/<名>/main.rs 里恰好认出一份。认不出判红。
#   跑前登记：脚本注释行里写成「跑前登记 research/prompts/<名字带 prereg 的>.md」的那一处（「跑前登记」与路径之间只隔空白、反引号或「）；注释里别处点名的登记（沿用谁的几何、
#     参照谁）不算。三份 E185 起跑脚本（scripts/experiment-apparatus/e185-r2-segment-run.sh、e185-r3-segment-run.sh、e185-r4-segment-run.sh）认的都是文件头注释里
#     「…第 N 次重跑一段的跑法（跑前登记 research/prompts/e185-rN-prereg.md 第五节「跑法」…」那一行里的那一处。写了几处不同的跑前登记、或注释里点名了登记而没有一处写成「跑前登记 <路径>」，判红。
# 判据：钉值与基准那一版的起跑脚本不同（基准里没有这份脚本或没有钉值行也算不同），或装置源码在改动范围里（相对 gate 基准的工作区、暂存区与未跟踪文件），就要核登记第十二节
#   （标题「## 十二、…」起到下一个「## 」之前，围栏里的标题不算）；两样都没变的不判。登记在树里的，逐行去首尾空白比、同一行写了几遍按次数算，按这个次序核：
#   「旧行原样在」：基准那一版第十二节里的每一行都原样在（改写、删掉任何一行都不算新加修订；登记是新文件、基准里没有它的，这一判没有对象）；
#   「新加修订」：装置源码变了的（钉值改没改都算），第十二节有基准那一版里没有的非空行，新加的行里出现起跑脚本此刻钉值的前 12 位，
#       并且写着钉值的新加行里至少一行与基准那一版第十二节里已有的行都不相同（把已有的一行再抄一遍不算新加修订）；
#       装置没变、只改了钉值的，第十二节里写着新钉值的前 12 位（这一次新写的，或早先登记时就写着的）；
#   「钉值最后写下」：钉值只认第十二节里最后写下的那个：钉值与基准钉值不同、两个的前 12 位都在第十二节里出现时，新钉值最后一次出现的位置要在基准钉值最后一次出现之后
#       （钉回登记里已被后续修订作废的旧值判红；新加一行写回旧值的回退，旧值最后一次出现在新加的那一行，放行）。
#   登记没有第十二节判红。
#   登记文件不在树里的：先看它是不是被 git mv 改名走的（这次改动里、或 git 历史里删它的那一次提交里，按内容认出改名），是就判红、点名新名字，
#   出路是把起跑脚本里「跑前登记 <路径>」改成新名字；不是改名的当已归档（archive-past-rounds 格删的）：先取基准那一版（这次改动里才从树里删的），
#   再取 git 历史里它被删之前的那一版，在那一版的第十二节里核钉值前 12 位有没有出现与「钉值最后写下」（那一版没法再新加行，不核「旧行原样在」与「新加修订」）；
#   历史里也找不到它的不判，成功句点名。
# 管不到：钉值是不是装置此刻的 sha256（对不上由开跑时的停机判）、新加的那一行写没写对改了什么；只认脚本点名的那一份装置源码文件，装置是目录时目录里别的文件变了不认；
#   注释里点名的登记路径写错、历史里也从没有过这份文件的，与已归档的分不开，都归「不判」；「钉值最后写下」只比此刻钉值与基准钉值两个值，
#   第十二节里别的哈希（别的文件的 sha256、更早几次的钉值）不进比较；改名时内容也改得多、git 认不出改名的，当已归档判。
# 样本：pinned-apparatus-revision-red（两份起跑脚本的装置源码都变了：一份登记完全没动，一份登记只改了第五节，第十二节没动，两份判红；另一份装置变了而登记已归档、一份装置没变，不判）；
#   pinned-apparatus-revision-green（两份登记的第十二节都新加了一行，其中一份第十三节也加了行；登记已归档的一份与装置没变的一份不判，判了 2 份）；
#   pinned-apparatus-revision-unresolved-red（一份起跑脚本既没有 apparatus_source= 也没有 cargo --bin，一份没有点名登记，两份判红，装置有没有变都判）。
#   pinned-apparatus-revision-inplace-rewrite-red（装置与钉值都改了，登记第十二节里写着旧钉值的那一行原地改成新钉值）、-internal-whitespace-red（装置改了、钉值没改，那一行只在词中间多一个空格）、
#   -duplicated-line-red（装置改了，第十二节把已有的一行再抄一遍）、-repin-only-red / -repin-only-green（装置在早先的提交里改了，这一次只把钉值改成现算值：登记没动判红 / 登记第十二节新加了含新钉值的一行判绿）、
#   -pin-trailing-comment-red（钉值行带行尾注释、装置改了、登记没动；另一份起跑脚本装置没变）、-registry-mentioned-after-another-red（头一句注释先点名了别的登记，自己的登记写成「跑前登记 <路径>」、第十二节没动）、
#   -registry-unlabeled-red（一份只在注释里点名了登记没写成「跑前登记」，一份写了两处不同的跑前登记）、-added-line-without-pin-red（第十二节新加的一行没有新钉值）、
#   -pin-in-other-section-red（新钉值只出现在第五节新加的行里）、-archived-history-red / -archived-history-green（登记已从树里删掉、历史里那一版第十二节没有钉值 / 有钉值）、
#   -pin-prefix-green（新加的一行只写了新钉值的前 16 位，判绿）。
#   -older-line-rewritten-red（第十二节两行，登记时那一行 sha1、修订那一行写着基准钉值 sha2；装置改了、钉值改成 sha3，登记时那一行原地改成 sha3，没新加行：「旧行原样在」判红）；
#   -archived-revert-to-superseded-red（登记已归档，那一版第十二节先写 sha1（交回时）后写 sha2（以这一份为准）；装置回退成 sha1 那一版、钉值改成 sha1：「钉值最后写下」判红）；
#   -intree-repin-superseded-red（登记在树里，第十二节先写 sha1 后写 sha2（以这一份为准）；装置没变，钉值改成作废的 sha1：「钉值最后写下」判红）；
#   -revert-with-new-line-green（登记在树里，装置回退成 sha1 那一版、钉值改成 sha1，第十二节末尾新加一行「撤回、回退到 sha1」：判绿）；
#   -registry-renamed-red（两份起跑脚本的登记都被 git mv 改了名、脚本没跟着改：E955 装置改了、钉值没动，E956 装置改了、钉值改了、新名字的登记新加了修订；都判红、点名新名字）。
# 弄坏开关见文件头 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK 的 pin-revision-unchecked、pin-base-ignored、pin-whole-registry-counted、pin-bin-flag-unresolved、pin-archived-judged、pin-unresolved-ignored、
#   pin-change-untriggered、pin-any-added-line、pin-copied-line-counted、pin-base-lines-rewritable、pin-base-lines-pin-bearing-only、pin-last-value-unchecked、pin-last-value-first-occurrence、
#   pin-rename-unrecognized、pin-trailing-comment-unrecognized、pin-registry-first-mention、pin-archived-history-unread、pin-prefix-whole-value 十八项。
# 退役判据：起跑脚本不再把装置 sha256 钉在脚本里（钉值改从登记读成单一出处）时删掉这一格。
cell_pinned_apparatus_revision() {
  ((in_git_work_tree)) || { echo "  ! $ROOT 不是 git 仓，分不出装置源码是不是这次改动里变的，这一格无对象可判"; exit 77; }
  if ((! changed_paths_library_loaded || ! changed_paths_ok)); then
    print_change_range_error
    echo "  ✗ 取不到这次改动的路径（scripts/gate/changed-paths.sh 读不到，或它的 gate_changed_paths 失败）"
    echo "     → 怎么办：看上面 changed-paths.sh 打的错误，修好再单跑这一格（bash .claude/gate.d/doc-experiments.sh --check pinned-apparatus-revision）"
    exit 1
  fi
  python3 - "$SHARED_CHANGED_PATHS" "$shared_base" "${EXPERIMENT_PAGES_AND_PRODUCTS_BREAK:-}" <<'PY'
import collections, glob, os, re, subprocess, sys

changed_file, base, broken_text = sys.argv[1:4]
broken = broken_text.split(",")
STARTER_PATTERN = "scripts/experiment-apparatus/*.sh"
# 弄坏开关 pin-trailing-comment-unrecognized：钉值行后面不许带行尾注释（带了的整份起跑脚本不进对象）。
PIN = re.compile(r"^registered_apparatus_sha256=([0-9a-f]{64})[ \t]*$" if "pin-trailing-comment-unrecognized" in broken
                 else r"^registered_apparatus_sha256=([0-9a-f]{64})[ \t]*(?:#.*)?$", re.M)
SOURCE_ASSIGNMENT = re.compile(r"^apparatus_source=([^\s#]+)[ \t]*(?:#.*)?$", re.M)
BIN_FLAG = re.compile(r"\bcargo\b.*?--bin[ =]([A-Za-z0-9_-]+)")
REGISTRY_NAME = re.compile(r"research/prompts/[A-Za-z0-9._-]*prereg[A-Za-z0-9._-]*\.md")
LABELED_REGISTRY = re.compile(r"跑前登记[ \t`「]*(research/prompts/[A-Za-z0-9._-]*prereg[A-Za-z0-9._-]*\.md)")
NO_REGISTRY_REASON = "脚本注释里没有点名 research/prompts/ 下名字带 prereg 的 .md"
FENCE = re.compile(r"^\s*(```|~~~)")
HEADING = re.compile(r"^##[ \t]")
TWELFTH = re.compile(r"^##[ \t]+十二、")
# 钉值在登记里要出现的位数；弄坏开关 pin-prefix-whole-value：要整串 64 位。
PIN_PREFIX_LENGTH = 64 if "pin-prefix-whole-value" in broken else 12

with open(changed_file, encoding="utf-8", errors="replace") as handle:
    changed = {line for line in handle.read().split("\n") if line}


def read_text(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return handle.read()


def apparatus_source_of(text):
    """起跑脚本点名的装置源码路径；认不出时给 None 与原因。"""
    match = SOURCE_ASSIGNMENT.search(text)
    if match:
        return match.group(1), None
    if "pin-bin-flag-unresolved" in broken:
        return None, "没有行首的 apparatus_source=<路径>"
    names = []
    for line in text.split("\n"):
        if line.lstrip().startswith("#"):
            continue
        found = BIN_FLAG.search(line)
        if found and found.group(1) not in names:
            names.append(found.group(1))
    candidates = sorted({path for name in names for pattern in (f"crates/*/src/bin/{name}.rs", f"crates/*/src/bin/{name}/main.rs") for path in glob.glob(pattern)})
    if len(candidates) == 1:
        return candidates[0], None
    return None, f"没有行首的 apparatus_source=<路径>，cargo 命令的 --bin 也没能在 crates/*/src/bin/ 下恰好认出一份（认出 {len(candidates)} 份）"


def registry_of(text):
    """起跑脚本自己的跑前登记路径：注释里写成「跑前登记 <路径>」的那一处；认不出时给 None 与原因。
    弄坏开关 pin-registry-first-mention：取注释里头一处点名的 research/prompts/ 下 prereg 的 .md，不看前面写没写「跑前登记」。"""
    comments = [line for line in text.split("\n") if line.lstrip().startswith("#")]
    if "pin-registry-first-mention" in broken:
        for line in comments:
            found = REGISTRY_NAME.search(line)
            if found:
                return found.group(0), None
        return None, NO_REGISTRY_REASON
    labeled = []
    for line in comments:
        for found in LABELED_REGISTRY.finditer(line):
            if found.group(1) not in labeled:
                labeled.append(found.group(1))
    if len(labeled) == 1:
        return labeled[0], None
    if len(labeled) > 1:
        return None, f"注释里「跑前登记」后面点名了 {len(labeled)} 份不同的登记（{'、'.join(labeled)}），认不出哪一份是它自己的"
    if any(REGISTRY_NAME.search(line) for line in comments):
        return None, "脚本注释里点名了 research/prompts/ 下名字带 prereg 的 .md，但没有一处写成「跑前登记 <路径>」，认不出哪一份是它自己的登记"
    return None, NO_REGISTRY_REASON


def pin_of(text):
    """起跑脚本里钉的装置 sha256（行首 registered_apparatus_sha256= 那一行的值）；没有给 None。"""
    found = PIN.search(text)
    return found.group(1) if found else None


def twelfth_section(text):
    """第十二节的行（标题之后到下一个「## 」之前，围栏里的标题不算）；没有这一节给 None。"""
    section, found, inside, fenced = [], False, False, False
    for line in text.split("\n"):
        if FENCE.match(line):
            fenced = not fenced
        elif not fenced and HEADING.match(line):
            if inside:
                break
            inside = found = bool(TWELFTH.match(line))
            continue
        if inside:
            section.append(line.rstrip())
    return section if found else None


def nonblank(lines):
    return collections.Counter(line.strip() for line in lines if line.strip())


def base_text(path, registry_comparison=False):
    """基准那一版里的文件内容，基准里没有它给空串；git 取不到给 None 与原因。
    弄坏开关 pin-base-ignored 只对登记拿来比新加行的那一版生效（registry_comparison）：不取基准那一版，当基准里没有。"""
    if not base or (registry_comparison and "pin-base-ignored" in broken):
        return "", None
    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", path], capture_output=True, text=True)
    if listed.returncode != 0:
        return None, f"git ls-tree --name-only {base} -- {path} 退出 {listed.returncode}：{listed.stderr.strip()}"  # change-range:exempt 报错文字，点名失败的是哪条子命令，不是调用
    if not listed.stdout.strip():
        return "", None
    shown = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True)
    if shown.returncode != 0:
        return None, f"git show {base}:{path} 退出 {shown.returncode}：{shown.stderr.decode('utf-8', errors='replace').strip()}"
    return shown.stdout.decode("utf-8", errors="replace"), None


def deleted_registry_text(path):
    """树里已经没有的登记：基准那一版（这次改动里才从树里删的），或 git 历史里它被删之前的那一版。
    给 (文本, 出处说明, None)；基准与历史里都没有它给 (None, None, None)；git 失败给 (None, None, 原因)。"""
    text, reason = base_text(path)
    if text is None:
        return None, None, reason
    if text:
        return text, f"基准 {base} 那一版（这次改动里已从树里删掉）", None
    logged = subprocess.run(["git", "log", "-1", "--diff-filter=D", "--format=%H", "HEAD", "--", path], capture_output=True, text=True)
    if logged.returncode != 0:
        return None, None, f"git log -1 --diff-filter=D --format=%H HEAD -- {path} 退出 {logged.returncode}：{logged.stderr.strip()}"
    deleted_at = logged.stdout.strip()
    if not deleted_at:
        return None, None, None
    shown = subprocess.run(["git", "show", f"{deleted_at}^:{path}"], capture_output=True)
    if shown.returncode != 0:
        return None, None, f"git show {deleted_at}^:{path} 退出 {shown.returncode}：{shown.stderr.decode('utf-8', errors='replace').strip()}"
    return shown.stdout.decode("utf-8", errors="replace"), f"git 历史 {deleted_at[:12]} 里它被删之前的那一版", None


def rename_target(name_status, path):
    """git diff -M --name-status --diff-filter=R 的输出里，path 被改名成了什么；没有给 None。"""
    for line in name_status.split("\n"):
        cells = line.split("\t")
        if len(cells) == 3 and cells[0].startswith("R") and cells[1] == path:
            return cells[2]
    return None


def renamed_registry(path):
    """树里已经没有的登记是不是被 git mv 改名走的：先看这次改动里（基准到工作区与暂存区），再看 git 历史里删它的那一次提交，都按内容认改名（-M）。
    给 (新路径, 在哪改的名, None)；不是改名给 (None, None, None)；git 失败给 (None, None, 原因)。"""
    if base:
        changed_now = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-M", "--name-status", "--diff-filter=R", base],
                                     capture_output=True, text=True, errors="replace")
        if changed_now.returncode != 0:
            return None, None, f"git diff -M --name-status --diff-filter=R {base} 退出 {changed_now.returncode}：{changed_now.stderr.strip()}"  # change-range:exempt 报错文字，点名失败的是哪条子命令，不是调用
        target = rename_target(changed_now.stdout, path)
        if target:
            return target, f"这次改动里（基准 {base} 起）", None
    logged = subprocess.run(["git", "log", "-1", "--diff-filter=D", "--format=%H", "HEAD", "--", path], capture_output=True, text=True)
    if logged.returncode != 0:
        return None, None, f"git log -1 --diff-filter=D --format=%H HEAD -- {path} 退出 {logged.returncode}：{logged.stderr.strip()}"
    deleted_at = logged.stdout.strip()
    if not deleted_at:
        return None, None, None
    in_commit = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-M", "--name-status", "--diff-filter=R", f"{deleted_at}^", deleted_at],
                               capture_output=True, text=True, errors="replace")
    if in_commit.returncode != 0:
        return None, None, f"git diff -M --name-status --diff-filter=R {deleted_at}^ {deleted_at} 退出 {in_commit.returncode}：{in_commit.stderr.strip()}"  # change-range:exempt 报错文字，点名失败的是哪条子命令，不是调用
    target = rename_target(in_commit.stdout, path)
    return (target, f"git 历史 {deleted_at[:12]} 里", None) if target else (None, None, None)


def occurrence_position(lines, prefix):
    """prefix 在这几行里最后一次出现的位置（行序号，行内偏移）；没出现给 None。
    弄坏开关 pin-last-value-first-occurrence：取第一次出现的位置。"""
    first_only = "pin-last-value-first-occurrence" in broken
    found = None
    for line_index, line in enumerate(lines):
        lowered = line.lower()
        column = lowered.find(prefix) if first_only else lowered.rfind(prefix)
        if column < 0:
            continue
        found = (line_index, column)
        if first_only:
            break
    return found


def superseded_pin_note(section, pin, base_pin):
    """钉值只认第十二节里最后写下的那个：此刻钉值与基准钉值不同、两个的前 12 位都在第十二节里出现，而基准钉值最后一次出现在此刻钉值最后一次出现之后，
    给一句说明（钉回了已被后续修订作废的值）；别的给 None。弄坏开关 pin-last-value-unchecked：一律给 None。"""
    if "pin-last-value-unchecked" in broken or not base_pin or base_pin[:12] == pin[:12]:
        return None
    pin_at, base_pin_at = occurrence_position(section, pin[:12]), occurrence_position(section, base_pin[:12])
    if pin_at is None or base_pin_at is None or base_pin_at < pin_at:
        return None
    return (f"第十二节里基准钉值前 12 位 {base_pin[:12]} 写在钉值前 12 位 {pin[:12]} 之后：钉值只认第十二节里最后写下的那个，"
            f"{pin[:12]} 是已被后续修订作废的值")


starters = sorted(path for path in glob.glob(STARTER_PATTERN) if os.path.isfile(path) and PIN.search(read_text(path)))
if not starters:
    print(f"  ⊘ 本次无对象可判：{STARTER_PATTERN} 里没有钉了装置 sha256（行首 registered_apparatus_sha256=<64 位十六进制>）的起跑脚本")
    sys.exit(77)
problems, revised, repinned, unchanged, archived = [], [], [], [], []
for starter in starters:
    text = read_text(starter)
    pin = pin_of(text)
    source, source_reason = apparatus_source_of(text)
    registry, registry_reason = registry_of(text)
    unrecognized = []
    if source is None:
        unrecognized.append(f"认不出装置源码（{source_reason}）")
    elif not os.path.isfile(source):
        unrecognized.append(f"装置源码 {source} 不在树里")
    if registry is None:
        unrecognized.append(f"认不出跑前登记（{registry_reason}）")
    if unrecognized:
        if "pin-unresolved-ignored" not in broken:
            problems.append(f"{starter}：钉了装置 sha256，{'；'.join(unrecognized)}")
        continue
    previous_starter, reason = base_text(starter)
    if previous_starter is None:
        problems.append(f"{starter}：取不到基准那一版的起跑脚本，{reason}")
        continue
    base_pin = pin_of(previous_starter)
    apparatus_changed = source in changed
    pin_changed = pin != base_pin
    # 弄坏开关 pin-change-untriggered：钉值变了不触发，只有装置源码变了才核登记。
    if not apparatus_changed and not (pin_changed and "pin-change-untriggered" not in broken):
        unchanged.append(starter)
        continue
    pin_note = f"钉值从 {base_pin[:12] if base_pin else '（基准里没有这份脚本或钉值行）'} 改成了 {pin[:12]}"
    if apparatus_changed and pin_changed:
        why = f"装置源码 {source} 在这次改动里变了、{pin_note}"
    elif apparatus_changed:
        why = f"装置源码 {source} 在这次改动里变了"
    else:
        why = pin_note
    origin = ""
    if os.path.isfile(registry):
        current = read_text(registry)
        previous, reason = base_text(registry, registry_comparison=True)
        if previous is None:
            problems.append(f"{starter}：取不到基准那一版的登记 {registry}，{reason}")
            continue
    else:
        # 被 git mv 改名走的登记不是归档：改名之前那一版不再是这份起跑脚本的登记，判红、出路是改起跑脚本里的登记路径。
        # 弄坏开关 pin-rename-unrecognized：不看是不是改名，照已归档的取改名之前那一版判。
        if "pin-rename-unrecognized" not in broken:
            renamed, renamed_where, reason = renamed_registry(registry)
            if reason:
                problems.append(f"{starter}：登记 {registry} 不在树里，查不了它是不是被改名走的，{reason}")
                continue
            if renamed:
                problems.append(f"{starter}：{why}，起跑脚本点名的登记 {registry} 不在树里，{renamed_where}被 git mv 改名成了 {renamed}（不是归档）："
                                f"把起跑脚本注释里「跑前登记 {registry}」改成「跑前登记 {renamed}」，再照新名字的登记核修订")
                continue
        # 弄坏开关 pin-archived-history-unread：树里没有的登记不去基准与 git 历史里取，一律当已归档不判。
        if "pin-archived-history-unread" in broken:
            current, origin, reason = None, None, None
        else:
            current, origin, reason = deleted_registry_text(registry)
        if reason:
            problems.append(f"{starter}：登记 {registry} 不在树里，取不到它被删之前的那一版，{reason}")
            continue
        if current is None:
            if "pin-archived-judged" in broken:
                problems.append(f"{starter}：{why}，登记 {registry} 不在树里")
            else:
                archived.append(f"{starter}（{registry}）")
            continue
        previous = ""
    section = twelfth_section(current)
    if section is None:
        problems.append(f"{starter}：{why}，登记 {registry} 里没有「## 十二、…」一节")
        continue
    judged = f"{starter}（{registry}，取自 {origin}）" if origin else f"{starter}（{registry}）"
    # 弄坏开关 pin-revision-unchecked：钉值或装置变了也不核登记第十二节有没有新加的行。
    if "pin-revision-unchecked" in broken:
        (revised if apparatus_changed else repinned).append(judged)
        continue
    token = pin[:PIN_PREFIX_LENGTH]

    def has_pin(line):
        return token in line.lower()

    if origin:
        # 已归档的登记没法再新加行：只核那一版的第十二节里写着钉值，与「钉值最后写下」。
        if not any(has_pin(line) for line in section):
            problems.append(f"{starter}：{why}，登记 {registry}（不在树里，取自 {origin}）的第十二节里没有出现钉值前 12 位 {pin[:12]}")
            continue
        superseded = superseded_pin_note(section, pin, base_pin)
        if superseded:
            problems.append(f"{starter}：{why}，登记 {registry}（不在树里，取自 {origin}）的{superseded}；归档的那一版没法再新加行，回退要写进新的重跑登记")
            continue
        (revised if apparatus_changed else repinned).append(judged)
        continue
    previous_section = twelfth_section(previous) or []
    # 「旧行原样在」：基准那一版第十二节里的每一行都原样在（修订只许新加行）。
    # 弄坏开关 pin-base-lines-rewritable：不核；pin-base-lines-pin-bearing-only：只核写着基准钉值前 12 位的行。
    if "pin-base-lines-rewritable" not in broken:
        kept_lines = previous_section
        if "pin-base-lines-pin-bearing-only" in broken:
            kept_lines = [line for line in previous_section if base_pin and base_pin[:12] in line.lower()]
        lost = nonblank(kept_lines) - nonblank(section)
        if lost:
            problems.append(f"{starter}：{why}，登记 {registry} 的第十二节里基准 {base} 那一版的 {sum(lost.values())} 行被改掉或删掉了（修订只许新加行，不改写已有的）")
            continue
    if apparatus_changed:
        # 「新加修订」：装置变了，第十二节要新加写着新钉值的行，写着钉值的新加行里至少一行不是基准那一版里已有的行再抄一遍。
        if "pin-whole-registry-counted" in broken:
            base_lines = nonblank(previous.split("\n"))
            added = nonblank(current.split("\n")) - base_lines
        else:
            base_lines = nonblank(previous_section)
            added = nonblank(section) - base_lines
        if not added:
            problems.append(f"{starter}：{why}，登记 {registry} 的第十二节没有新加的行（基准 {base}）")
            continue
        # 弄坏开关 pin-any-added-line：新加的行里有没有新钉值不看，有一行新加的就算修订。
        if "pin-any-added-line" not in broken and not any(has_pin(line) for line in added):
            problems.append(f"{starter}：{why}，登记 {registry} 的第十二节新加的 {sum(added.values())} 行里没有出现钉值前 12 位 {pin[:12]}（基准 {base}）")
            continue
        # 弄坏开关 pin-copied-line-counted：与基准那一版里已有的行逐字相同的新加行也算修订。
        if ("pin-copied-line-counted" not in broken and any(has_pin(line) for line in added)
                and not any(has_pin(line) and line not in base_lines for line in added)):
            problems.append(f"{starter}：{why}，登记 {registry} 的第十二节新加的写着钉值前 12 位 {pin[:12]} 的行，都是基准 {base} 那一版第十二节里已有的行再抄一遍，"
                            "不是新加修订")
            continue
    elif not any(has_pin(line) for line in section):
        # 装置没变、只改了钉值：新钉值要在第十二节里写着（这一次新写的，或早先登记时就写着的）。
        problems.append(f"{starter}：{why}，登记 {registry} 的第十二节里没有出现钉值前 12 位 {pin[:12]}（基准 {base}）")
        continue
    # 「钉值最后写下」：钉值只认第十二节里最后写下的那个。
    superseded = superseded_pin_note(section, pin, base_pin)
    if superseded:
        problems.append(f"{starter}：{why}，登记 {registry} 的{superseded}（回退到旧值的，在第十二节末尾新加一行写回旧值与为什么回退）")
        continue
    (revised if apparatus_changed else repinned).append(judged)

if problems:
    print(f"  ✗ {len(problems)} 份钉了装置 sha256 的起跑脚本对不上它的跑前登记：")   # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                                      # gate-lint:detail
    print("     → 怎么办：补登记第十二节一行修订（新 sha256、改了什么）：在上面点名的登记里「## 十二、…」一节末尾新写一行，写改完的装置 sha256（sha256sum <装置源码>）与改了什么，")
    print("               再把起跑脚本的 registered_apparatus_sha256 钉成同一个值（钉值行末尾可以带 # 注释）；只改钉值的也要先在登记里新写这一行，写出新钉值的前 12 位以上，")
    print("               已有的行不改写、不重抄；回退到登记里写过的旧值的，同样在第十二节末尾新加一行写回旧值与为什么回退（钉值只认第十二节里最后写下的那个）；")
    print("               登记已归档（不在树里）的，新钉值要写进新的重跑登记，归档的那一版没法再新加行；")
    print("               登记被 git mv 改了名的，把起跑脚本注释里「跑前登记 <路径>」改成红行点名的新名字；")
    print("               认不出装置源码的起跑脚本加一行行首的 apparatus_source=<装置源码路径>，认不出登记的在脚本头注释里写成「跑前登记 research/prompts/<登记文件名>」；")
    print("               取不到基准那一版的，按上面 git 的报错修好仓库状态再跑。")
    sys.exit(1)

def named(entries):
    return f"（{'、'.join(entries)}）" if entries else ""

print(f"  ✓ 钉了装置 sha256 的起跑脚本 {len(starters)} 份：装置源码这次改动里变了的 {len(revised)} 份，登记第十二节都新加了修订{named(revised)}；"
      f"装置没变而钉值改了的 {len(repinned)} 份，登记第十二节里都写着新钉值{named(repinned)}；"
      f"在树里的登记基准那一版第十二节的行都原样在，钉值改了的新钉值都写在基准钉值之后；"
      f"装置没变不判 {len(unchanged)} 份；登记已归档、不在树里不判 {len(archived)} 份{named(archived)}；基准 {base}")
PY
}
