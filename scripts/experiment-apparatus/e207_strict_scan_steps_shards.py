#!/usr/bin/env python3
# admission: always 每次照此刻各片文件现核，上一次合并的结论不替这一次作保
# run-condition: none 只读各片文件与 cargo 的 JSON 输出，不连服务、不起 cargo
"""E207（B8严格判据下的步数）入口 e207-strict-scan-steps-run.sh 的服务分片那一份小工具：读 cargo 的 JSON 找测试程序，合并各片、核账。

用法：
  e207_strict_scan_steps_shards.py executable --target <测试目标名>      标准输入是 `cargo test --no-run --message-format=json` 的输出；打测试程序的路径，恰好一个，不然退 1
  e207_strict_scan_steps_shards.py errors [--limit <条数，默认 10>]   标准输入是 `cargo test --no-run --message-format=json` 的输出（编不过时）；把编译器报的每条 error 的正文
      （文件:行与那几行源码，取 JSON 里 compiler-message 的 rendered）打出来，条数多的打前 <条数> 条、余下报条数；一条都没有也打一行说明。恒退 0
  e207_strict_scan_steps_shards.py merge --item <项> --count <共几片> --out <文件> <片文件>…
      片文件按片号从小到大给（第 0 片起）。核过才写 --out：写的是各片 E207RESULT 行按片的次序首尾相接（与不分片时同名同形的一份产物的正文，不含产物头），
      另打一行 name=shard_merge。核不过退 1、一个字节不写 --out，每处不对一行 ✗ 加一行 → 怎么办。
  e207_strict_scan_steps_shards.py counts --total <片数上限> <项>=<片数>…   片数之和不超过上限；超了把片数最大的那几项一片一片往下减到装得下，打 项=片数 各一行

片文件（装置历史用例 e207_strict_scan_steps_histories.rs 的分片，入口把标准输出里 E207RESULT 与 E207SHARD 两种行原样存下）：
  E207RESULT … 行（这一片管的序号区间里的读数；第 0 片另有 point、constants、sizes 三行；最后一片另有历史末的汇总、量窗、重挂载与 item_done）
  末行 E207SHARD name=shard_done item=<项> shard=<k>/<K> notes_from=<a> notes_to=<b> settle_calls_from=<c> settle_calls_to=<d> notes_total=<N>

核账（merge；每一条不过都红，点名哪一片）：
  ① 片数对：给的片文件个数等于 --count，片号依次 0…K-1，项名对；
  ② 区间首尾相接不重不漏：第 0 片 notes_from = 0，后一片的 notes_from = 前一片的 notes_to，最后一片 notes_to = notes_total，各片 notes_total 相同；
  ③ 固定点次数首尾相接：第 0 片 settle_calls_from = 0，后一片的 settle_calls_from = 前一片的 settle_calls_to；
  ④ 总数对得上：各片发布数（notes_to − notes_from）之和 = notes_total；最后一片正文里 name=history 行的 user_changes = notes_total、settle_calls = 最后一片的 settle_calls_to；
  ⑤ 完整：每片末行是片尾标记，最后一片正文最后一行是 name=item_done，别的片正文里没有 item_done；
  ⑥ 每片正文每一行都是 E207RESULT 起头、带这一项的名字（item= 或 for_item= 之外的行认不得）。
弄坏开关 E207_RUN_BREAK=<项>（只在入口自证里认：E207_SELFTEST_MARK 指到的文件内容是 e207-selftest）：
  E207_RUN_BREAK=merge-drops-a-shard  合并时丢掉第 1 片（片数 ≥ 3 时）的正文，核账照丢之前的片文件核
      sharded_product_equals_the_whole_product
  E207_RUN_BREAK=merge-gap-unchecked  不核 ② ③ ④
      shards_that_do_not_meet_turn_the_merge_red
  E207_RUN_BREAK=shards-guard-passes-always  入口自证的开关（在入口脚本里读：自证把守卫那一趟的标记文件写成 e207-selftest-guard-broken，守卫见了永远放行）
      the_shards_script_refuses_the_break_switch_outside_the_selftest
平常跑时设了 E207_RUN_BREAK（E207_SELFTEST_MARK 没指到入口自证建的标记文件）：在参数解析之前退 2，输出里点名它只在自证里认。
退出码：0 成；1 核不过或找不到测试程序；2 用法错或弄坏开关设在自证之外。
"""
import json
import os
import re
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "E207_RUN_BREAK"
SELFTEST_MARK_VARIABLE = "E207_SELFTEST_MARK"
SELFTEST_MARK_TEXT = "e207-selftest"
GUARD_BROKEN_MARK_TEXT = "e207-selftest-guard-broken"   # 自证给守卫证红用：标记文件是这句时守卫当作没有、一律放行
MARKER = re.compile(
    r"^E207SHARD name=shard_done item=(?P<item>\S+) shard=(?P<index>\d+)/(?P<count>\d+) notes_from=(?P<nf>\d+) notes_to=(?P<nt>\d+) "
    r"settle_calls_from=(?P<sf>\d+) settle_calls_to=(?P<st>\d+) notes_total=(?P<total>\d+)$"
)
HISTORY_LINE = re.compile(r"^E207RESULT name=history item=(?P<item>\S+) user_changes=(?P<changes>\d+) settle_calls=(?P<calls>\d+) ")


def started_by_the_selftest():
    """E207_SELFTEST_MARK 指到的文件内容是 e207-selftest：这一趟是入口自证起的。"""
    try:
        with open(os.environ.get(SELFTEST_MARK_VARIABLE, ""), encoding="utf-8") as handle:
            return handle.read().strip() == SELFTEST_MARK_TEXT
    except OSError:
        return False


def broken(switch):
    """弄坏开关只在入口自证起的那几趟里认。"""
    if os.environ.get(BREAK_VARIABLE, "") != switch:
        return False
    return started_by_the_selftest()


def guard_broken_by_the_selftest():
    """弄坏开关 shards-guard-passes-always：入口自证把标记文件写成 GUARD_BROKEN_MARK_TEXT，守卫就永远返回 0。"""
    try:
        with open(os.environ.get(SELFTEST_MARK_VARIABLE, ""), encoding="utf-8") as handle:
            return handle.read().strip() == GUARD_BROKEN_MARK_TEXT
    except OSError:
        return False


def refuse_break_outside_the_selftest():
    """参数解析之前先拒：设了 E207_RUN_BREAK 而这一趟不是入口自证起的，退 2 并点名它只在自证里认。"""
    if guard_broken_by_the_selftest():
        return 0
    if os.environ.get(BREAK_VARIABLE, "") == "" or started_by_the_selftest():
        return 0
    print(f"  ✗ {BREAK_VARIABLE} 只在自证里认（selftest：{SELFTEST_MARK_VARIABLE} 指到入口自证建的标记文件时才认），这一趟不是自证起的", file=sys.stderr)
    print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> bash e207-strict-scan-steps-run.sh --selftest（经临时 prune-server 的包装）", file=sys.stderr)
    return 2


def problem(message, how):
    print(f"  ✗ {message}")
    print(f"     → 怎么办：{how}")


def read_shard(path):
    """(正文行, 片尾标记 dict 或 None, 没有换行收尾的残行有没有)。"""
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        text = handle.read()
    cut = bool(text) and not text.endswith("\n")
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines = lines[:-1]
    marker = None
    if lines and not cut:
        found = MARKER.match(lines[-1])
        if found:
            marker = {key: (value if key == "item" else int(value)) for key, value in found.groupdict().items()}
            lines = lines[:-1]
    return lines, marker, cut


def command_errors(arguments):
    limit = 10
    if arguments[:1] == ["--limit"] and len(arguments) == 2 and arguments[1].isdigit():
        limit = int(arguments[1])
    elif arguments:
        print("用法：errors [--limit <条数>]", file=sys.stderr)
        return 2
    errors = []
    for raw in sys.stdin:
        try:
            message = json.loads(raw)
        except ValueError:
            continue
        if message.get("reason") != "compiler-message":
            continue
        inner = message.get("message", {})
        text = inner.get("rendered") or inner.get("message") or ""
        if inner.get("level") == "error" and text and not inner.get("message", "").startswith("aborting due to"):
            errors.append(text.rstrip())
    if not errors:
        print("  编译器的 JSON 里没有 error 正文（可能是链接或 cargo 自己的错），看上面 cargo 的输出")
        return 0
    print(f"  ✗ 编译器报了 {len(errors)} 条 error，下面打前 {min(limit, len(errors))} 条的正文：")
    for text in errors[:limit]:
        print("\n".join("    " + line for line in text.splitlines()))
    if len(errors) > limit:
        print(f"  余下 {len(errors) - limit} 条没打")
    return 0


def command_executable(arguments):
    if len(arguments) != 2 or arguments[0] != "--target":
        print("用法：executable --target <测试目标名>", file=sys.stderr)
        return 2
    found = []
    for raw in sys.stdin:
        try:
            message = json.loads(raw)
        except ValueError:
            continue
        if message.get("reason") != "compiler-artifact" or not message.get("executable"):
            continue
        target = message.get("target", {})
        if target.get("name") == arguments[1] and "test" in target.get("kind", []):
            found.append(message["executable"])
    found = sorted(set(found))
    if len(found) != 1:
        print(f"  ✗ cargo 的输出里该恰好有一个测试目标 {arguments[1]} 的测试程序，找到 {len(found)} 个：{found}")
        print("     → 怎么办：看 cargo test --no-run --message-format=json 的输出（编译日志在计划那一步的日志里），目标名对不对")
        return 1
    print(found[0])
    return 0


def merge(item, count, paths):
    """核各片、拼正文。交回 (正文行 list 或 None, 汇总文字)。"""
    problems = []
    if len(paths) != count:
        problem(f"{item}：要 {count} 片，给了 {len(paths)} 个片文件", "合并那一步缺片（TASK_SERVER_SHARD_MISSING 或片没成）：补跑缺的片，同一条命令重交只补没判的片")
        return None, ""
    if broken("merge-drops-a-shard") and count >= 3:
        pass  # 下面拼正文时丢，核账照给的核
    shards = []
    for index, path in enumerate(paths):
        lines, marker, cut = read_shard(path)
        if marker is None:
            problems.append((f"{item} 第 {index} 片（{path}）末行不是片尾标记" + ("，文件被截断（末行没有换行）" if cut else ""),
                             "这一片没做完：同一条命令重交，库里没有的片现判"))
            shards.append((lines, None))
            continue
        if marker["item"] != item or marker["index"] != index or marker["count"] != count:
            problems.append((f"{item} 第 {index} 片的片尾标记写的是 {marker['item']} 第 {marker['index']}/{marker['count']} 片",
                             "片文件放错了位置：看 outputs.tsv 的片号与各片产物目录"))
        shards.append((lines, marker))
    if problems:
        for message, how in problems:
            problem(message, how)
        return None, ""
    if not broken("merge-gap-unchecked"):
        problems.extend(gap_problems(item, count, shards))
    last_lines = shards[-1][0]
    if not last_lines or last_lines[-1] != f"E207RESULT name=item_done item={item}":
        problems.append((f"{item} 最后一片（第 {count - 1} 片）正文最后一行不是 name=item_done", "最后一片没跑完历史、量窗与重挂载：看那一片的日志"))
    for index, (lines, _marker) in enumerate(shards[:-1]):
        if f"E207RESULT name=item_done item={item}" in lines:
            problems.append((f"{item} 第 {index} 片（不是最后一片）正文里有 item_done", "非最后一片到区间末就该收尾：看历史用例的分片收尾"))
    for index, (lines, _marker) in enumerate(shards):
        for number, line in enumerate(lines, 1):
            if not line.startswith("E207RESULT ") or (f"item={item}" not in line and f"for_item={item}" not in line):
                problems.append((f"{item} 第 {index} 片正文第 {number} 行认不得：{line[:100]}", "片文件里只该有这一项的 E207RESULT 行与末行的片尾标记"))
                break
    if problems:
        for message, how in problems:
            problem(message, how)
        return None, ""
    body = []
    for index, (lines, _marker) in enumerate(shards):
        if index == 1 and count >= 3 and broken("merge-drops-a-shard"):
            continue
        body.extend(lines)
    first, last = shards[0][1], shards[-1][1]
    summary = (f"name=shard_merge item={item} shards={count} notes={last['nt']} settle_calls={last['st']} "
               f"lines={len(body)} per_shard_notes={','.join(str(marker['nt'] - marker['nf']) for _lines, marker in shards)} first_from={first['nf']}")
    return body, summary


def gap_problems(item, count, shards):
    problems = []
    markers = [marker for _lines, marker in shards]
    total = markers[0]["total"]
    for index, marker in enumerate(markers):
        if marker["total"] != total:
            problems.append((f"{item} 第 {index} 片的 notes_total={marker['total']} 与第 0 片的 {total} 不同", "各片算的历史长度该相同：看历史用例的 notes_of_the_history"))
    if markers[0]["nf"] != 0:
        problems.append((f"{item} 第 0 片的 notes_from={markers[0]['nf']}，该是 0", "区间从头起：看历史用例的 shard_interval"))
    if markers[0]["sf"] != 0:
        problems.append((f"{item} 第 0 片的 settle_calls_from={markers[0]['sf']}，该是 0", "第 0 片放历史时还没有固定点"))
    for index in range(1, count):
        before, after = markers[index - 1], markers[index]
        if after["nf"] != before["nt"]:
            problems.append((f"{item} 第 {index - 1} 片到 notes_to={before['nt']}、第 {index} 片从 notes_from={after['nf']} 起：区间没有首尾相接（{'漏了' if after['nf'] > before['nt'] else '重了'} {abs(after['nf'] - before['nt'])} 个）",
                             "区间不重不漏：看历史用例的 shard_interval，不出产物"))
        if after["sf"] != before["st"]:
            problems.append((f"{item} 第 {index - 1} 片的固定点次数到 {before['st']}、第 {index} 片从 {after['sf']} 起：对不上", "各片放同一段历史，次数该首尾相接：看两片的日志"))
    if markers[-1]["nt"] != total:
        problems.append((f"{item} 最后一片到 notes_to={markers[-1]['nt']}，总数是 {total}", "最后一片该到总数：看历史用例的 shard_interval"))
    published = sum(marker["nt"] - marker["nf"] for marker in markers)
    if published != total:
        problems.append((f"{item} 各片发布数之和 {published} 不等于总数 {total}", "区间不重不漏"))
    history = [HISTORY_LINE.match(line) for line in shards[-1][0]]
    history = [found for found in history if found and found.group("item") == item]
    if len(history) != 1:
        problems.append((f"{item} 最后一片正文里 name=history 行有 {len(history)} 条，该恰好 1 条", "最后一片做完历史才打这一行"))
    else:
        if int(history[0].group("changes")) != total:
            problems.append((f"{item} name=history 行的 user_changes={history[0].group('changes')}，不等于总数 {total}", "用户改动的个数与 notes_of_the_history 的式子对不上：两边都查"))
        if int(history[0].group("calls")) != markers[-1]["st"]:
            problems.append((f"{item} name=history 行的 settle_calls={history[0].group('calls')}，最后一片的 settle_calls_to={markers[-1]['st']}", "固定点次数对不上：看历史用例的片尾标记"))
    return problems


def command_merge(arguments):
    item = count = out = None
    paths = []
    index = 0
    while index < len(arguments):
        flag = arguments[index]
        if flag in ("--item", "--count", "--out") and index + 1 < len(arguments):
            value = arguments[index + 1]
            if flag == "--item":
                item = value
            elif flag == "--count" and value.isdigit() and int(value) >= 1:
                count = int(value)
            elif flag == "--out":
                out = value
            else:
                print(f"用法：--count 要正整数：{value}", file=sys.stderr)
                return 2
            index += 2
        elif flag.startswith("--"):
            print(f"认不得的参数：{flag}", file=sys.stderr)
            return 2
        else:
            paths.append(flag)
            index += 1
    if item is None or count is None or out is None:
        print("用法：merge --item <项> --count <共几片> --out <文件> <片文件>…", file=sys.stderr)
        return 2
    body, summary = merge(item, count, paths)
    if body is None:
        return 1
    temporary = out + ".partial"
    with open(temporary, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(line + "\n" for line in body))
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, out)
    print(summary)
    return 0


def command_counts(arguments):
    if len(arguments) < 3 or arguments[0] != "--total" or not arguments[1].isdigit():
        print("用法：counts --total <片数上限> <项>=<片数>…", file=sys.stderr)
        return 2
    limit = int(arguments[1])
    wanted = {}
    for pair in arguments[2:]:
        name, _, number = pair.rpartition("=")
        if not name or not number.isdigit() or int(number) < 1:
            print(f"认不得：{pair}", file=sys.stderr)
            return 2
        wanted[name] = int(number)
    if len(wanted) > limit:
        print(f"  ✗ {len(wanted)} 项每项至少一片，超过服务给的上限 {limit}", file=sys.stderr)
        print("     → 怎么办：少点几项（--item / --segment）再交", file=sys.stderr)
        return 1
    while sum(wanted.values()) > limit:
        biggest = max(wanted.values())
        name = [key for key, value in wanted.items() if value == biggest][0]
        wanted[name] -= 1
    for name, number in wanted.items():
        print(f"{name}={number}")
    return 0


def main(arguments):
    commands = {"executable": command_executable, "errors": command_errors, "merge": command_merge, "counts": command_counts}
    if not arguments or arguments[0] not in commands:
        print(f"用法：e207_strict_scan_steps_shards.py {'|'.join(commands)} …", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    refused = refuse_break_outside_the_selftest()
    if refused:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
