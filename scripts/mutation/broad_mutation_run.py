"""broad-mutation.py 拆出的一份。
一次跑：选条、算键与读库的提速、汇总（summarise）、命令行的参数、跑全部项（run_items）、复用哨兵、分片的计划与计数。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from broad_mutation_base import (COMPOSITIONS_TABLE, DECISIVE_OUTCOMES, DEFAULT_LOCK_WAIT_SECONDS, DEFAULT_STOP_GRACE_SECONDS,
    DEFAULT_SUSPECT_STREAK, DEFAULT_SUSPECT_TEST_SECONDS, DRIVER_ERROR, EXIT_GATE_RED, EXIT_INCONSISTENT, EXIT_USAGE, GPU_ROW_OUTCOMES,
    GPU_VERDICT_COLUMNS, HeavyThreadDefaultError, NOT_RUN, OUTCOME_TEXT, REPOSITORY, ROCKSDB_PREBUILT_PATH, ROW_OUTCOMES, SCRIPT_KEY, SELF_PATH,
    STAGE_ONE_PACKAGES, SUBMIT_PREFIX, SUMMARY_COLUMNS, SUMMARY_KEYS, VERDICT_COLUMNS, VERIFICATION_PACKAGE, default_heavy_threads)
from broad_mutation_judging import (ItemState, Judgement, break_is, composition_difference, gpu_decided_keys, named_items_rejudged, note,
    process_item, prune_rules, read_gpu_verdicts, refuse, say, select_items, stage_two_skipped, test_mode, today_text, write_atomically)
from broad_mutation_library import (Stopped, generate_list, in_run_scope, prune_server_client, record_list_cache, restore_list, reuse_checks_library)
from broad_mutation_runner import (Runner, package_arguments, registered_functions)
from broad_mutation_gpu import (combined_states, gpu_views, run_gpu_phase)


# ── 算键与读库的提速（不改键、不改状态）──────────────────────────────────────────────────────────────────────
# 这一节的提速是外挂上去的（装在 Judgement 里的 UnitTree、PRUNE_MODULE 与 Library 的实例上，本文件里 Judgement、UnitTree 的方法一字不动），库里的判定一条都不作废。
# 算键、算状态里互不依赖的活（每条变异的键）按线程数并行（mutation-prune.py 的 prefill_records，fork 出子进程、内存放得下才开），结果按条的次序收；
# 状态不缓存、不并行：它读判定库此刻的行，两片之间库里会写进新判定。

def key_identity(item):
    """一条变异在 Judgement.keys 里的身份（与 Judgement.key_of 里那一个相同）。"""
    return (item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"])


def speed_up_judgement(judgement):
    """给这一棵树上的 Judgement 装提速（mutation-prune.py「算键提速」一节）：closure_key 与起子进程二进制装缓存、组成清单交惰性的，
    库的写组成清单记住已写过的键。键、拒算形态与组成清单同不装时逐项相同。"""
    judgement.rules.cache_closure_keys(judgement.tree)
    judgement.rules.cache_spawning_binaries()
    memorise_compositions(judgement.library)


def memorise_compositions(library):
    """库的 write_composition 记住这个进程里写过（或已在库里）的键：同一条测试抓到几百条变异，它的组成清单只读一次库、只写一次。
    弄坏开关 composition-write-unmemorised：不记，每次都读一次。"""
    if getattr(library, "written_compositions", None) is not None:
        return
    written = library.written_compositions = set()
    write = library.write_composition

    def write_composition(key, lines):
        if not key or (key in written and not break_is("composition-write-unmemorised")):
            return
        write(key, lines)
        written.add(key)
    library.write_composition = write_composition


def memory_room_bytes():
    """本进程所在 cgroup 到它自己与各层上级的内存上限还剩多少字节（取最小的），并不超过 /proc/meminfo 的 MemAvailable；读不到交 None。"""
    room = None
    relative = None
    try:
        with open("/proc/meminfo", encoding="ascii") as handle:
            for line in handle:
                if line.startswith("MemAvailable:"):
                    room = int(line.split()[1]) * 1024
        with open("/proc/self/cgroup", encoding="utf-8") as handle:
            relative = next((line.strip().split("::", 1)[1] for line in handle if line.startswith("0::")), None)
    except (OSError, ValueError, IndexError):
        return room
    if relative is None:
        return room
    root = "/sys/fs/cgroup"
    directory = os.path.normpath(root + relative)
    while directory.startswith(root):
        try:
            with open(os.path.join(directory, "memory.max"), encoding="ascii") as handle:
                limit = handle.read().strip()
            with open(os.path.join(directory, "memory.current"), encoding="ascii") as handle:
                current = int(handle.read().strip())
        except (OSError, ValueError):
            limit, current = "max", 0
        if limit != "max":
            left = int(limit) - current
            room = left if room is None else min(room, left)
        if directory == root:
            break
        directory = os.path.dirname(directory)
    return room


def key_workers(arguments, count):
    """并行算键的路数与说明：上限是 --threads（缺省取登记的每台默认重活线程数）、每路至少 8 条、内存放得下：本进程所在 cgroup 的余量乘 0.7
    除以每路的估计（本进程驻留内存的一半加 128 MiB，推的：子进程共用 fork 时的页，写到的页才各占一份）；余量读不到就只受线程数与条数限。→ (路数, 说明)。"""
    threads = max(1, getattr(arguments, "threads", 1) or 1)
    if count < 64:
        return 1, f"只 {count} 条，不并行"
    room = memory_room_bytes()
    try:
        with open("/proc/self/statm", encoding="ascii") as handle:
            resident = int(handle.read().split()[1]) * os.sysconf("SC_PAGE_SIZE")
    except (OSError, ValueError, IndexError):
        resident = 0
    estimate = resident // 2 + (128 << 20)
    by_memory = threads if room is None else max(1, int(room * 0.7) // estimate)
    room_text = "读不到" if room is None else f"{room >> 20} MiB"
    return max(1, min(threads, count // 8, by_memory)), f"线程上限 {threads}、内存余量 {room_text}、每路估 {estimate >> 20} MiB（推的）"


def key_cache_digest(judgement):
    """算键缓存认的摘要：专用树导出范围里每个文件的 sha256（换树时算的清单）、全局那几行、驱动与规则库与它们用的两份库的 sha256、弄坏开关；
    任一项变了缓存就不认。换树的清单没有（只读的用法不换树）或读不了库文件交空串：不读也不写缓存。"""
    manifest = judgement.library.manifest
    if not manifest:
        return ""
    lines = [f"file\t{path}\t{manifest[path]}" for path in sorted(manifest)]
    lines += [f"global\t{line}" for line in judgement.globals]
    lines += [f"break\t{name}" for name in sorted(judgement.rules.BREAKS)]
    # 驱动与规则库自己的字节哈希进键缓存的签名：改了算键的代码，缓存里的键就不再取（缓存的是算出来的键，不是键的组成）
    for path in ("scripts/mutation/broad-mutation.py", "scripts/mutation/mutation-prune.py", "scripts/mutation/lib_mutation_units.py", "scripts/gate/lib_function_units.py"):
        try:
            with open(os.path.join(REPOSITORY, path), "rb") as handle:
                lines.append(f"code\t{path}\t{hashlib.sha256(handle.read()).hexdigest()}")
        except OSError:
            return ""
    return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


def write_key_cache(path, digest, items, records):
    """计划那一步把每条的键与带走的几样（mutation_key 的结果去掉容器单元与组成清单）写进计划目录，各片与合并那一步照摘要对得上才读；不进判定库，不进任何键。"""
    rows = [[item["file"], item["start"], item["end"], item["replacement"], item["name"], *record] for item, record in zip(items, records)]
    write_atomically(path, json.dumps({"form": "broad-mutation-key-cache-v1", "digest": digest, "rows": rows}).encode("ascii"))


def read_key_cache(judgement, todo, path, digest):
    """读计划留下的算键缓存：→ (与 todo 同序的记录, "") 或 (None, 没读的原因)。摘要对不上、缺 todo 里的条（身份或变异名对不上）都不读；
    读了的按条次序抽查（至多十二条）：现算一遍与记录逐项相同才用。弄坏开关 key-cache-digest-unchecked：摘要不核；key-cache-sample-unchecked：不抽查。"""
    try:
        with open(path, encoding="ascii") as handle:
            stored = json.load(handle)
    except (OSError, ValueError):
        return None, "计划目录里没有算键缓存"
    if not isinstance(stored, dict) or stored.get("form") != "broad-mutation-key-cache-v1" or (stored.get("digest") != digest and not break_is("key-cache-digest-unchecked")):
        return None, "算键缓存的摘要与此刻这棵树对不上"
    by_identity = {}
    for row in stored.get("rows", []):
        if isinstance(row, list) and len(row) == 11:
            by_identity[(row[0], tuple(row[1]), tuple(row[2]), row[3])] = (row[4], row[5:])
    records = []
    for item in todo:
        found = by_identity.get(key_identity(item))
        if found is None or found[0] != item["name"]:
            return None, "算键缓存里缺这一趟要算的条"
        records.append(found[1])
    if not break_is("key-cache-sample-unchecked"):
        for index in range(0, len(todo), max(1, len(todo) // 12)):
            if judgement.rules.key_record(judgement.tree, todo[index], judgement.globals) != records[index]:
                return None, f"算键缓存抽查第 {todo[index]['sequence']} 条与现算的不同"
    return records, ""


def key_view(judgement, item, record):
    """prefill 的一条记录 → Judgement.key_of 交的那种字典（mutation_key 的返回值）：容器单元照树现找，组成清单惰性地照 mutation_key 现算。"""
    key, refused, relative, original, locator, has_unit = record
    tree, rules = judgement.tree, judgement.rules
    unit = tree.container(item["file"], tuple(item["start"]), tuple(item["end"])) if has_unit else None
    composition = rules.lazy_lines_class()(lambda: rules.mutation_key(tree, item, judgement.globals)["composition"], ())
    return {"key": key, "refused": list(refused), "composition": composition, "unit": unit, "relative": relative, "original": original, "locator": locator}


def prefill_keys(judgement, items, arguments, cache_directory):
    """这些条的键先算齐、填进 Judgement.keys（之后 key_of 一律命中）：计划之后的各片与合并那一步先读计划写在计划目录里的算键缓存（摘要对得上、抽查过才用），
    读不到现算：按线程数并行（key_workers），按条的次序收；计划那一步算完把缓存写进计划目录。cache_directory 空串是不读不写缓存。"""
    todo = [item for item in items if key_identity(item) not in judgement.keys]
    if not todo:
        return
    phase = getattr(arguments, "shard_phase", "")
    path = os.path.join(cache_directory, "key-cache.json") if cache_directory and phase in ("plan", "shard", "merge") else ""
    digest = key_cache_digest(judgement) if path else ""
    records, why, how, worth_saying = None, "", "", False
    if digest and phase in ("shard", "merge"):
        records, why = read_key_cache(judgement, todo, path, digest)
        how, worth_saying = "读计划留下的算键缓存（抽查过）", True
    if records is None:
        workers, reason = key_workers(arguments, len(todo))
        records = judgement.rules.prefill_records(judgement.tree, todo, judgement.globals, workers)
        how = f"现算，{workers} 路（{reason}）"
        worth_saying = worth_saying or workers > 1 or bool(digest and phase == "plan")
        if digest and phase == "plan":
            write_key_cache(path, digest, todo, records)
            how += "，缓存写进计划目录"
    if worth_saying:
        note(f"  · 键：{len(todo)} 条，{how}" + (f"；没读缓存：{why}" if why else ""))
    for item, record in zip(todo, records):
        judgement.keys[key_identity(item)] = key_view(judgement, item, record)


def changed_units_of(judgement, items):
    """Judgement.changed_units 的批量写法：→ {序号: 变了的单元那一句}（没有变了的单元的条不进）。上一回的组成清单按六十四把键一批读（read_many），
    读一批比一批的结果就地比完、不攒着（一份组成清单几千行）。取哪一行、怎么比与 changed_units 相同。
    弄坏开关 changed-units-batch-oldest-row：取同一处变异最早的那一行、不取最后一行。"""
    wanted = []
    for item in items:
        current = judgement.key_of(item)
        previous = [row for row in judgement.by_locator.get(current["locator"], []) if row["key"] != current["key"]]
        if previous:
            old_key = previous[0]["key"] if break_is("changed-units-batch-oldest-row") else previous[-1]["key"]
            if old_key:
                wanted.append((item, current, old_key))
    found = {}
    for start in range(0, len(wanted), 64):
        batch = wanted[start:start + 64]
        keys = sorted({old_key for _item, _current, old_key in batch})
        try:
            values = judgement.library.store.read_many([f"{COMPOSITIONS_TABLE}/{key}" for key in keys])
        except prune_server_client().PruneServerFailure:
            if not break_is("unreachable-as-empty-library"):
                raise
            values = [None] * len(keys)
        stored = dict(zip(keys, values))
        for item, current, old_key in batch:
            data = stored.get(old_key)
            if data is None:
                continue
            text = composition_difference(data.decode("utf-8", "surrogateescape").splitlines(), current["composition"], 5)
            if text:
                found[item["sequence"]] = text
    return found


def item_states(judgement, items):
    return {item["sequence"]: judgement.state(item) for item in items}


def stale_counts(judgement, items):
    """每条在判定库里同一处变异（位置索引相同）而键对不上的判定行数（底稿第二节第 9 条 ⑦：--list-items 报每项键对不上的判定条数）。"""
    counts = {}
    for item in items:
        current = judgement.key_of(item)
        counts[item["sequence"]] = sum(1 for row in judgement.by_locator.get(current["locator"], []) if row["key"] != current["key"])
    return counts


def read_only_inputs(arguments):
    """只读的几种（--list-items、--summary）给了 --toolchain-from / --tools-from 时读进来：→ (工具链文字或 None, 工具版本行或 None)。"""
    toolchain = tools = None
    if arguments.toolchain_from:
        with open(arguments.toolchain_from, encoding="utf-8") as handle:
            toolchain = handle.read().strip() + "\n"
    if arguments.tools_from:
        with open(arguments.tools_from, encoding="utf-8") as handle:
            tools = [line.strip() for line in handle if line.strip()]
    return toolchain, tools


def read_only_judgement(library, arguments, source):
    toolchain, tools = read_only_inputs(arguments)
    judgement = Judgement(library, source, arguments, for_run=False, registered=registered_functions(source), toolchain=toolchain, tools=tools)
    speed_up_judgement(judgement)
    items, unlisted, unrestorable = restore_list(judgement, library.read_list_cache())
    prefill_keys(judgement, items, arguments, "")    # 只读的用法不读也不写计划目录的算键缓存：被判的是 --source 本身，不是换树之后的专用树
    return judgement, items, unlisted, unrestorable


def list_items(library, arguments, source):
    judgement, items, unlisted, _unrestorable = read_only_judgement(library, arguments, source)
    states, stale = item_states(judgement, items), stale_counts(judgement, items)
    states = combined_states(states, gpu_views(library, arguments, judgement, items, states, for_run=False)[1])    # 状态列是两趟合起来的（文件头「GPU 那一趟」）
    for item in items:
        say(f"{item['sequence']}\t{item['package']}\t{item['name']}\t{states[item['sequence']][0].state}\tstale={stale[item['sequence']]}"
            f"\tkey={judgement.key_of(item)['key'] or 'refused'}")
    if unlisted:
        note(f"  · {len(unlisted)} 个单元没记清单（此刻源码上 cargo-mutants 还没列过它们）：例 {unlisted[0].file}:{unlisted[0].first_line} {unlisted[0].name}；"
             f"{SUBMIT_PREFIX} python3 {SELF_PATH} --refresh-list 记上")


def summarise(library, arguments, source):
    """六栏条数与提交门禁那一行（文件头「输出」）；判定库对不上（段数、结局认不出）判红退 1，提交门禁判红退 8。"""
    judgement, items, unlisted, unrestorable = read_only_judgement(library, arguments, source)
    table = judgement.table
    states = item_states(judgement, items)
    judging, views = gpu_views(library, arguments, judgement, items, states, for_run=False)
    states = combined_states(states, views)     # CPU 那一趟没红又要 GPU 那一趟的条，状态取两趟合起来的（文件头「GPU 那一趟」）
    gpu_table = judging.table if judging is not None else read_gpu_verdicts(library)
    gpu_needed = [view for view in views.values() if view.needed]
    gpu_reused = sum(1 for view in gpu_needed if view.gpu.state in DECISIVE_OUTCOMES)
    gpu_not_run = sum(1 for view in gpu_needed if view.gpu.state == NOT_RUN)
    gpu_without = len(gpu_needed) - gpu_reused - gpu_not_run
    gpu_fields = f" gpu_needed={len(gpu_needed)} gpu_reused={gpu_reused} gpu_not_run={gpu_not_run} gpu_without_conclusion={gpu_without}"
    counts = dict.fromkeys(SUMMARY_COLUMNS, 0)
    refused, without, missed_items = [], [], []
    for item in items:
        state, reason, deciding = states[item["sequence"]]
        counts[state.state] = counts.get(state.state, 0) + 1
        forms = judgement.refused_text(item, deciding)
        if forms:
            refused.append((item, forms))
        if state.state not in DECISIVE_OUTCOMES:
            without.append((item, state.state, reason))
        if state.state == "missed":
            missed_items.append(item)
    total = len(items)
    say(f"BROAD_MUTATION_SUMMARY total={total} " + " ".join(f"{SUMMARY_KEYS[column]}={counts[column]}" for column in SUMMARY_COLUMNS)
        + f" refused={len(refused)} unlisted_units={len(unlisted)}{gpu_fields}")
    say("  " + "、".join(f"{OUTCOME_TEXT[column]} {counts[column]}" for column in SUMMARY_COLUMNS) + f"；清单 {total} 条，拒算 {len(refused)} 条，没记清单的单元 {len(unlisted)} 个"
        f"；要 GPU 那一趟的 {len(gpu_needed)} 条（有结论、库里键相同复用 {gpu_reused}，本次未跑 {gpu_not_run}，没结论 {gpu_without}）")
    for item in missed_items:
        say(f"MISSED\t{item['sequence']}\t{item['package']}\t{item['name']}")
    for item in items:
        state, _reason, deciding = states[item["sequence"]]
        if state.state == DRIVER_ERROR:
            last = [row for row in judgement.by_key.get(judgement.key_of(item)["key"], []) if row["outcome"] == DRIVER_ERROR]
            say(f"DRIVER_ERROR\t{item['sequence']}\t{item['package']}\t{item['name']}\t{last[-1]['note'] if last else ''}")
    for item, forms in refused:
        say(f"REFUSED\t{item['sequence']}\t{forms}\t{item['name']}")
    for unit in unlisted:
        say(f"UNLISTED\t{unit.file}:{unit.first_line}\t{unit.kind}\t{unit.name}")
    problems = []
    unknown_outcomes = [row for row in table.rows if row["outcome"] not in ROW_OUTCOMES]
    if unknown_outcomes:
        problems.append(f"{len(unknown_outcomes)} 行是认不出的结局（例：键 {unknown_outcomes[0]['key'][:16]}「{unknown_outcomes[0]['outcome']}」）")
    if table.malformed:
        problems.append(f"判定表里段数不是 {len(VERDICT_COLUMNS)} 的 {table.malformed_text()}")
    unknown_gpu_outcomes = [row for row in gpu_table.rows if row["outcome"] not in GPU_ROW_OUTCOMES]
    if unknown_gpu_outcomes:
        problems.append(f"GPU 判定库里有 {len(unknown_gpu_outcomes)} 行是认不出的结局（例：键 {unknown_gpu_outcomes[0]['key'][:16]}「{unknown_gpu_outcomes[0]['outcome']}」）")
    if gpu_table.malformed:
        problems.append(f"GPU 判定表里段数不是 {len(GPU_VERDICT_COLUMNS)} 的 {gpu_table.malformed_text()}")
    if unrestorable:
        unit, entry = unrestorable[0]
        problems.append(f"清单按单元存的表里有 {len(unrestorable)} 条在此刻的源码上复原不回去（例：{unit.file} {unit.name} 的「{entry.get('description', '')[:60]}」）：lists/ 里那一条写坏了")
    if sum(counts.values()) != total:
        problems.append(f"六栏加起来 {sum(counts.values())}，清单 {total} 条")
    gate_red = bool(without or counts["missed"] or unlisted or total == 0)
    say(f"BROAD_MUTATION_GATE total={total} concluded={total - len(without)} missed={counts['missed']} without_conclusion={len(without)} refused={len(refused)} "
        f"unlisted_units={len(unlisted)}{gpu_fields} verdict={'red' if gate_red else 'green'}")
    changes = changed_units_of(judgement, [item for item, _state, _reason in without])
    for item, state, reason in without:
        changed = changes.get(item["sequence"], "")
        say(f"NO_CONCLUSION\t{item['sequence']}\t{state}\t{item['name']}\t{reason}" + (f"；变了：{changed}" if changed else ""))
    if problems and not break_is("summary-mismatch-silent"):
        print(f"  ✗ 汇总对不上：{'；'.join(problems)}", file=sys.stderr)
        print(f"  → 怎么办：判定库在 {library.location()}（判定在 verdicts/，GPU 趟的在 gpu-verdicts/）。段数不对、结局认不出的条目不是驱动写的（驱动只写成形的行）："
              "交主 agent 查是谁经客户端写进来的；要清掉就用 scripts/prune-server/prune_server_client.py 的 LibraryStore 把点到的条目改写成成形的行（服务没有删键），"
              "改写之后那几条下一趟照沿用不了现判；"
              f"清单复原不回去的：{SUBMIT_PREFIX} python3 {SELF_PATH} --refresh-list 重新记", file=sys.stderr)
        return EXIT_INCONSISTENT
    if gate_red and not break_is("summary-gate-silent"):
        print(f"  ✗ 提交门禁判红：{len(without)} 条沿用不了、也没有现判的结论（其中要 GPU 那一趟而没有结论的 {gpu_not_run + gpu_without} 条），{counts['missed']} 条没红，"
              f"{len(unlisted)} 个单元没记清单", file=sys.stderr)
        print(f"  → 怎么办：主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起驱动剪枝跑（{SUBMIT_PREFIX} python3 {SELF_PATH} "
              "：每条按单元键沿用、沿用不了的现判；CPU 那一趟没红又要 GPU 那一趟的条"
              "（NO_CONCLUSION 里理由是 gpu-…）接在最后跑，要在有卡的机器上跑，没卡的机器上它们是本次未跑、不算没红也不算通过），再跑这一道；"
              "没红的加一条能杀它的测试再判；确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判），理由写在那一行旁边", file=sys.stderr)
        return EXIT_GATE_RED
    return 0


class ArgumentParser(argparse.ArgumentParser):
    def error(self, message):
        print(f"  ✗ 用法错：{message}", file=sys.stderr)
        print(f"  → 怎么办：python3 {SELF_PATH} --help 看每个参数；项的写法照 --list-items 列的", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def default_threads():
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则每台默认重活线程数（.claude/heavy-thread-default 的登记值）；取不了照用法错退，不退回按核数算。"""
    try:
        return default_heavy_threads()
    except HeavyThreadDefaultError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"  → 怎么办：{error.howto}", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def check_real_run_is_allowed():
    """真跑（起 cargo）之前：工具在不在。"""
    if test_mode():
        return
    missing = [tool for tool in ("cargo", "rustc", "cargo-nextest", "eatmydata", "systemd-run", "systemctl") if shutil.which(tool) is None]
    if missing:
        refuse(f"PATH 上缺：{'、'.join(missing)}", "cargo-nextest 用 cargo install cargo-nextest --locked 装，eatmydata 用系统包管理器装（apt install eatmydata）")


def multi_host_configuration_for_tests(source):
    """真跑测试那几步交给测试的多机配置（文件头「多机配置交给测试」）：(绝对路径或空串, 在不在)。调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它，
    没设取 source 的 git common-dir 上一层下的 multi-host.env；取不到 common-dir 交空串（没处找）。"""
    configured = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG", "").strip()
    if configured:
        path = os.path.abspath(configured)
        return path, os.path.isfile(path)
    found = subprocess.run(["git", "-C", source, "rev-parse", "--path-format=absolute", "--git-common-dir"], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if found.returncode != 0 or not found.stdout.strip():
        return "", False
    path = os.path.join(os.path.dirname(found.stdout.strip()), "multi-host.env")
    return path, os.path.isfile(path)


def check_multi_host_configuration(source, arguments):
    """真跑起任何 cargo 之前判多机配置在不在（文件头「多机配置交给测试」），在就记进 arguments.multi_host_configuration；不在拒（退 2），一步都不起。
    弄坏开关 test-steps-configuration-unpassed 下不判。"""
    path, present = multi_host_configuration_for_tests(source)
    arguments.multi_host_configuration = path
    if present or break_is("test-steps-configuration-unpassed"):
        return
    refuse(f"多机配置 {path or '（取不到 git common-dir，没处找）'} 不在（{source} 不是 git 工作树时又没设 SINGLEFS_MULTI_HOST_CONFIG 的也算）：真跑测试的每一步都要把它交给测试"
           "（专用树不是 git 工作树，测试自己找不到，crates 那侧找不到就报错），一步都没起",
           "照仓根 multi-host.env.example 在主工作树根写一份 multi-host.env，或设 SINGLEFS_MULTI_HOST_CONFIG 指到它；不补缺省")


def check_rocksdb_entry(source):
    """真跑起任何 cargo 之前（同步树、清单、基线与每一步都在这之后）判这台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：
    判的是 --source 那棵树（同步进库目录里的就是它）。判红照 entry 原样打红句、拒（退 2），一步都不起；没判红什么都不打。
    弄坏开关 rocksdb-entry-unchecked 下不判（自证 selftest_rocksdb_entry_cells 那一格判错）。"""
    if break_is("rocksdb-entry-unchecked"):
        return
    entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT_PATH, "entry", "--root", source], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if entry.returncode != 0:
        sys.stderr.write(entry.stdout + entry.stderr)
        refuse("起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一步都没起",
               '照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再跑同一条命令')


class SuspectStreaks:
    """连着几条像是环境造成的结局（文件头 --suspect-streak）：连续无效、连续「测试那一步几乎立刻红」的抓到。到了 K 条就在没改过的树上把那一步重跑一遍。"""

    def __init__(self, runner, arguments):
        self.runner, self.arguments = runner, arguments
        self.unviable, self.fast_caught = [], []

    def saw(self, sequence, row):
        outcome = row.fields[VERDICT_COLUMNS.index("outcome")]
        if break_is("no-environment-check"):
            return
        if row.environment_doubt:
            self.runner.verify_on_pristine_tree([row.step_name], row.environment_doubt, [sequence])
        self.unviable = self.unviable + [(sequence, row.step_name)] if outcome == "unviable" else []
        fast = outcome == "caught" and row.step_seconds < self.arguments.suspect_test_seconds
        self.fast_caught = self.fast_caught + [(sequence, row.step_name)] if fast else []
        for streak, why in ((self.unviable, "无效"), (self.fast_caught, f"测试那一步不到 {self.arguments.suspect_test_seconds} 秒就红的抓到")):
            if len(streak) < self.arguments.suspect_streak:
                continue
            sequences = [found_sequence for found_sequence, _step in streak]
            step_names = list(dict.fromkeys(step_name for _sequence, step_name in streak))
            self.runner.verify_on_pristine_tree(step_names, f"连续 {len(streak)} 条{why}", sequences)
            say(f"  · 连续 {len(streak)} 条{why}（第 {sequences[0]}–{sequences[-1]} 条）：{'、'.join(step_names)} 在没改过的树上照过，不是环境，接着跑")
            streak.clear()


def stop_on_signal(_signal_number, _frame):
    raise Stopped()


def coverage_step(specification_path):
    """--coverage-step：驱动在没改过的树上起的一步。插桩编一遍、列出全部测试，第一段逐条单独一个进程跑、验证包逐个二进制跑（登记的崩溃枚举用例与放量分项经 --skip 排掉）
    （spec 的 known 里那几条覆盖记录照用，不跑），每条（每个二进制）的 profraw 合并后 llvm-cov export，记它执行到的单元（文件, 限定名）；
    写 spec 里的 out：{"tests": 全部测试, "jobs": 现跑的那几条, "failed": 跑红的}；有跑红的退 1。"""
    import concurrent.futures
    with open(specification_path, encoding="utf-8") as handle:
        spec = json.load(handle)
    rules = prune_rules()
    tree = rules.UnitTree(spec["tree"])
    llvm = spec["llvm_bin"]
    known = {tuple(job) for job in spec.get("known", [])}
    profiles = os.path.join(spec["scratch"], "profiles")
    os.makedirs(profiles, exist_ok=True)
    environment = dict(os.environ, CARGO_TARGET_DIR=spec["target"], RUSTFLAGS="-C instrument-coverage", CARGO_TERM_COLOR="never",
                       LLVM_PROFILE_FILE=os.path.join(spec["scratch"], "build-%p-%m.profraw"))

    def build(arguments):
        done = subprocess.run(["cargo", "test", "--locked", "--no-run", "--message-format=json", *arguments], cwd=spec["tree"], env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if done.returncode != 0:
            print(done.stderr[-4000:], file=sys.stderr)
            sys.exit(1)
        found = []
        for line in done.stdout.splitlines():
            message = json.loads(line) if line.startswith("{") else {}
            if message.get("reason") == "compiler-artifact" and message.get("executable") and message.get("profile", {}).get("test"):
                package = message["package_id"].split("#")[0].rsplit("/", 1)[-1].split("@")[0]
                kind, name = message["target"]["kind"][0], message["target"]["name"]
                binary = package if kind == "lib" else (f"{package}::bin/{name}" if kind == "bin" else f"{package}::{name}")
                found.append((binary, kind, name, message["executable"], os.path.dirname(message["manifest_path"])))
        return found

    def listed(executable, cwd):
        done = subprocess.run([executable, "--list", "--format", "terse"], cwd=cwd, env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        return [line[:-len(": test")] for line in done.stdout.splitlines() if line.endswith(": test")]

    def cover(job):
        number, binary, test, executable, cwd, argv, single = job
        directory = os.path.join(profiles, str(number))
        os.makedirs(directory, exist_ok=True)
        child = dict(environment, LLVM_PROFILE_FILE=os.path.join(directory, "%p-%m.profraw"))
        if single:
            child.update({name: "1" for name in ("RUST_TEST_THREADS", "SINGLEFS_THREAD_CAP")})
        done = subprocess.run(argv, cwd=cwd, env=child, capture_output=True, text=True, stdin=subprocess.DEVNULL)
        files = sorted(os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".profraw"))
        covered = []
        if files:
            merged = os.path.join(directory, "merged.profdata")
            subprocess.run([os.path.join(llvm, "llvm-profdata"), "merge", "-sparse", *files, "-o", merged], check=True, capture_output=True)
            exported = subprocess.run([os.path.join(llvm, "llvm-cov"), "export", executable, f"-instr-profile={merged}"], capture_output=True, text=True, check=True)
            covered = rules.covered_units(json.loads(exported.stdout), tree)
        shutil.rmtree(directory, ignore_errors=True)
        return {"binary": binary, "test": test, "covered": [list(entry) for entry in covered], "profiles": len(files), "exit": done.returncode}

    tests, jobs, stage_one_arguments = [], [], package_arguments(STAGE_ONE_PACKAGES)
    if spec["kind"] == "one":
        for binary, _kind, _name, executable, cwd in build(stage_one_arguments):
            for test in listed(executable, cwd):
                tests.append([binary, test])
                if (binary, test) not in known:
                    jobs.append((len(jobs), binary, test, executable, cwd, [executable, "--exact", test, "--test-threads=1"], True))
    else:
        skips = [word for name in spec.get("skips", []) for word in ("--skip", name)]   # 登记的崩溃枚举用例与放量分项不在这里跑（驱动交下来的名单）
        for _binary, kind, name, executable, cwd in build(["--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]):
            job = ("verification:" + ("lib" if kind == "lib" else name), "*")
            tests.append(list(job))
            if job not in known:
                jobs.append((len(jobs), job[0], "*", executable, cwd, [executable, *skips], False))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, spec["workers"] if spec["kind"] == "one" else 1)) as pool:
        records = list(pool.map(cover, jobs))
    failed = [record for record in records if record["exit"] != 0]
    write_atomically(spec["out"], (json.dumps({"tests": tests, "jobs": records, "failed": failed}, ensure_ascii=False) + "\n").encode())
    say(f"BROAD_MUTATION_COVERAGE kind={spec['kind']} tests={len(tests)} jobs={len(jobs)} failed={len(failed)}")
    for record in failed:
        print(f"  ✗ 覆盖那一遍在没改过的树上跑红：{record['binary']} {record['test']}（退 {record['exit']}）", file=sys.stderr)  # gate-lint:detail
    if failed:
        print("  → 怎么办：没改过的树上本来就红的测试先修好，再重跑驱动；覆盖那一遍不替红的测试记覆盖", file=sys.stderr)
    return 1 if failed else 0


# ── 主流程 ─────────────────────────────────────────────────────────────────────────────────────────────────

def parse_arguments(argv):
    import math
    parser = ArgumentParser(description="广谱变异的逐条驱动；用法与库目录的布局见文件头", add_help=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--list-items", action="store_true")
    mode.add_argument("--summary", action="store_true")
    mode.add_argument("--refresh-list", action="store_true")
    mode.add_argument("--selftest", action="store_true")
    mode.add_argument("--coverage-step", metavar="spec")
    parser.add_argument("--source")
    parser.add_argument("--toolchain-from")
    parser.add_argument("--tools-from")
    parser.add_argument("--llvm-bin")
    parser.add_argument("--item", action="append", default=[])
    parser.add_argument("--from", dest="start_from", type=int)
    parser.add_argument("--package")
    parser.add_argument("--stage-two-eatmydata", action="store_true")
    parser.add_argument("--threads", type=int, default=default_threads())
    parser.add_argument("--suspect-streak", type=int, default=DEFAULT_SUSPECT_STREAK)
    parser.add_argument("--suspect-test-seconds", type=float, default=DEFAULT_SUSPECT_TEST_SECONDS)
    parser.add_argument("--stop-grace", type=int, default=DEFAULT_STOP_GRACE_SECONDS)
    parser.add_argument("--lock-wait", type=int, default=DEFAULT_LOCK_WAIT_SECONDS)
    parser.add_argument("--home")
    # 弄坏开关 unknown-option-accepted：认不出的选项悄悄略过（改错的样子）
    arguments = parser.parse_known_args(argv)[0] if break_is("unknown-option-accepted") else parser.parse_args(argv)
    if arguments.threads < 1:
        parser.error("--threads 要是正整数")
    if arguments.suspect_streak < 1:
        parser.error("--suspect-streak 要是正整数")
    if not math.isfinite(arguments.suspect_test_seconds) or arguments.suspect_test_seconds < 0:
        parser.error("--suspect-test-seconds 要是有限的非负数")
    if arguments.stop_grace < 0 or arguments.lock_wait < 0:
        parser.error("--stop-grace 与 --lock-wait 要是非负整数")
    if arguments.coverage_step:
        return arguments
    read_only = arguments.list_items or arguments.summary
    if (arguments.toolchain_from or arguments.tools_from) and not read_only:
        parser.error("--toolchain-from、--tools-from 只给只读的 --list-items / --summary 用（提交门禁的样本）；真跑与写行的用法不认")
    return arguments


def plan_line(items, states, selected):
    judged = refused = carried = 0
    selected_sequences = {item["sequence"] for item, _one, _two in selected}
    for item in items:
        state, reason, _row = states.get(item["sequence"], (ItemState(NOT_RUN, False), "", None))
        if item["sequence"] in selected_sequences:
            if reason.startswith("refused") or reason.startswith("catching-test-unresolved"):
                refused += 1
            else:
                judged += 1
        elif state.state in DECISIVE_OUTCOMES:
            carried += 1
    return judged, carried, refused


def run_items(library, arguments, source):
    runner = Runner(library, arguments)
    judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, registered=runner.registered)
    speed_up_judgement(judgement)
    items = generate_list(library)
    written, units = record_list_cache(library, judgement, items)
    say(f"  · 清单：{len(items)} 条，按单元记进判定库 lists/（{units} 个单元，这一趟新写 {written} 条）")
    interrupted = getattr(arguments, "interrupted", None)
    if interrupted is not None and break_is("interrupted-as-done"):
        # 弄坏开关：被打断的那一条照「已完成」落一行抓到，不重跑
        item = next(item for item in items if item["sequence"] == interrupted["sequence"])
        current = judgement.key_of(item)
        library.record_verdict([current["key"] or "", item["sequence"], item["package"], item["name"], "unviable", "one", "", "", "", "0.0", "0.0", today_text(),
                                "fresh", current["locator"], "", "break"])
        judgement = runner.judgement = Judgement(library, library.tree, arguments, for_run=True, registered=runner.registered)
        speed_up_judgement(judgement)
    phase = getattr(arguments, "shard_phase", "")
    if phase == "shard":
        # 只在自己那一堆里判（文件头「分片」shard）：选条参数计划那一步已经照过，这里换成那一堆；键与状态只算这一堆的，不算全部条再筛
        scope = arguments.shard_scope
        unknown = sorted(scope - {item["name"] for item in items})
        if unknown:
            refuse(f"计划那一步分给这一片的 {len(unknown)} 条在此刻清单里找不到（头一条：{unknown[0]}）：树在计划与这一片之间变了，一条都没判",
                   "别在分片跑的时候改被判的树；改完经服务重交同一条命令")
        items = [item for item in items if item["name"] in scope]
        arguments.item, arguments.start_from, arguments.package = [], None, None
    prefill_keys(judgement, items, arguments, os.environ.get("TASK_SERVER_SHARD_PLAN_DIRECTORY", "") if phase in ("plan", "shard", "merge") else "")
    states = item_states(judgement, items)
    withdrawn = [item for item in items if states[item["sequence"]][1] == "missed-rejudged"]
    if withdrawn:      # 撤绿的没红行里，要 GPU 趟、GPU 趟又不是没红的，照沿用：交给 Judgement.state 再算一遍这几条
        judgement.gpu_decided = gpu_decided_keys(library, judgement, withdrawn, arguments)
        states.update({item["sequence"]: judgement.state(item) for item in withdrawn})
    selected = select_items(judgement, items, arguments, states)
    if phase == "plan":
        piles = shard_piles(items, states, selected, arguments, judgement)
        write_shard_plan(shard_plan_directory(), f"要现判 {len(selected)} 条", piles)
        say(f"  ✓ 分片计划：清单 {len(items)} 条，选条参数罩得住、要现判 CPU 那一趟的 {len(selected)} 条，计划写 {len(piles)} 片：" + "；".join(label for label, _names in piles))
        return 0
    sentinel = plan_reuse_sentinel(items, states, arguments)
    forced = next((item for item in items if sentinel.forced(str(item["sequence"]))), None)
    if phase == "merge":
        # 核账（文件头「分片」merge）：还要现判的条（拒算的、复用哨兵抽中的除外）一条都不现判，有就判红
        leftover = [item for item, _one, _two in selected
                    if (forced is None or item["sequence"] != forced["sequence"])
                    and not states.get(item["sequence"], (None, "", None))[1].startswith(("refused", "catching-test-unresolved"))]
        if leftover:
            for item in leftover[:30]:
                print(f"  ✗ 分片没判到：第 {item['sequence']} 条 {item['name']}（{states.get(item['sequence'], (None, '', None))[1] or '没有结论'}）", file=sys.stderr)  # gate-lint:detail
            refuse(f"合并核账：{len(leftover)} 条 CPU 那一趟还没有作数的结论（分片没判到，或分片之后树变了），这一步不现判",
                   "看各片的日志（task-server/task log <那一片的子任务号>）；修好之后经服务重交同一条命令", 1)
        selected = [entry for entry in selected if forced is not None and entry[0]["sequence"] == forced["sequence"]]
    if forced is not None and all(item["sequence"] != forced["sequence"] for item, _one, _two in selected):
        selected = sorted(selected + [(forced, True, not stage_two_skipped())], key=lambda entry: entry[0]["sequence"])
    judged, carried, refused = plan_line(items, states, selected)
    say(f"BROAD_MUTATION_PLAN list={len(items)} judge={judged} carried={carried} refused={refused} selected={len(selected)} threads={arguments.threads}")
    changes = changed_units_of(judgement, [item for item, _one, _two in selected])
    for item, _one, _two in selected:
        _state, reason, _row = states.get(item["sequence"], (None, "selected", None))
        if forced is not None and item["sequence"] == forced["sequence"]:
            reason = "sentinel-forced"
        changed = changes.get(item["sequence"], "")
        say(f"BROAD_MUTATION_JUDGE sequence={item['sequence']} reason={reason or 'selected'}" + (f" changed={changed}" if changed else ""))
    if not selected:
        if finish_reuse_sentinel(sentinel, {}):
            return 1
        gpu_totals = run_gpu_phase(library, runner, items, arguments)     # CPU 那一趟没有要现判的，GPU 那一趟（上一趟没卡、没来得及跑的）照样补
        if gpu_totals.failed:
            return 1
        say(f"  ✓ 选中 0 条要跑：清单 {len(items)} 条，现判 0、沿用 {carried}、拒算 0（--summary 看六栏与提交门禁那一行）；{gpu_totals_text(gpu_totals)}")
        write_shard_counts(arguments, 0, carried, 0, gpu_totals)
        if phase == "merge":
            say(f"  ✓ 分片合并核账：CPU 那一趟没有分片没判到的条；{shard_counts_text()}")
        return 0
    stages = [stage for stage, wanted in (("one", any(one for _item, one, _two in selected)), ("two", any(two for _item, _one, two in selected))) if wanted]
    for stage in stages:
        runner.ensure_baseline(stage)
    runner.ensure_coverage(stages)
    runner.verify_environment_at_start(stages)
    ran = dict.fromkeys(ROW_OUTCOMES, 0)
    outcomes = {}
    streaks = SuspectStreaks(runner, arguments)
    started = time.monotonic()
    try:
        for item, with_stage_one, with_stage_two in selected:
            item_started = time.monotonic()
            row = process_item(library, runner, item, with_stage_one, with_stage_two)
            fields = row.fields
            outcome = fields[VERDICT_COLUMNS.index("outcome")]
            ran[outcome] += 1
            outcomes[str(item["sequence"])] = outcome
            # 入口脚本那一行（文件头「输出」）：一条一判、没红的那一条红，复用的条不在这里（它们没现判）
            counts = "" if break_is("item-counts-dropped") else f"judged=1 red={1 if outcome == 'missed' else 0} reused=0 "
            say(f"BROAD_MUTATION_ITEM sequence={item['sequence']} outcome={outcome} stage={fields[VERDICT_COLUMNS.index('stage')]} "
                f"build_seconds={fields[VERDICT_COLUMNS.index('build_seconds')]} test_seconds={fields[VERDICT_COLUMNS.index('test_seconds')]} "
                f"wall_seconds={time.monotonic() - item_started:.1f} {counts}name={item['name']}"
                + (f" first_failing_test={fields[VERDICT_COLUMNS.index('caught_by')]}" if fields[VERDICT_COLUMNS.index('caught_by')] else ""))
            streaks.saw(item["sequence"], row)
    finally:
        library.flush_deferred_rows()
    say(f"BROAD_MUTATION_TOTAL selected={len(selected)} ran={sum(ran.values())} judged={judged} carried={carried} refused={refused} "
        + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in ran.items()) + f" seconds={time.monotonic() - started:.0f}")
    if finish_reuse_sentinel(sentinel, outcomes):
        return 1
    gpu_totals = run_gpu_phase(library, runner, items, arguments)     # CPU 那一趟全跑完之后：没红又有带 gpu 特性的测试够得着的条，补一趟（文件头「GPU 那一趟」）
    if gpu_totals.failed:
        return 1
    say(f"  ✓ 选中的 {len(selected)} 条都判完（{'、'.join(f'{OUTCOME_TEXT[name]} {count}' for name, count in ran.items())}）：现判 {judged}、沿用 {carried}、拒算 {refused}；"
        f"{gpu_totals_text(gpu_totals)}")
    write_shard_counts(arguments, judged, carried, refused, gpu_totals)
    if phase == "merge":
        say(f"  ✓ 分片合并核账：CPU 那一趟没有分片没判到的条（只现判了复用哨兵抽中的 {len(selected)} 条）；{shard_counts_text()}")
    return 0


def gpu_totals_text(totals):
    """成功句里 GPU 那一趟的一段：要这一趟的条数与现判、复用、本次未跑各几条（本次未跑的没有结论，不算没红，也不算通过）。"""
    return f"GPU 趟（要 {totals.needed} 条）：现判 {totals.judged}、复用 {totals.carried}、本次未跑 {totals.not_run}"


def plan_reuse_sentinel(items, states, arguments):
    """复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论（抓到、没红、无效）的条，库里的判定是那一条的结局；
    自证造「只跑第一段」的现场（BROAD_MUTATION_STAGE_ONE_ONLY）时只取第一段就定了论的（强制现判那一条只跑第一段，比得了）；
    点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）时不抽。"""
    sentinel = reuse_checks_library().ReuseSentinel(SCRIPT_KEY, "广谱变异")
    off = "点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）" if named_items_rejudged() else ""
    candidates = {}
    for item in items if not off else []:
        state, _reason, row = states.get(item["sequence"], (ItemState(NOT_RUN, False), "", None))
        if state.state in DECISIVE_OUTCOMES and in_run_scope(item, arguments) and (not stage_two_skipped() or (row or {}).get("stage") == "one"):
            candidates[str(item["sequence"])] = state.state
    sentinel.plan(candidates, reuse=not off, off_reason=off, printer=say)
    return sentinel


def finish_reuse_sentinel(sentinel, outcomes):
    """哨兵判：outcomes 是这一趟现判的 {序号: 结局}；没在里面的候选就是沿用了。驱动错不是结论，强制现判的那一条撞上它不比。红了交真。"""
    for sequence, stored in sentinel.candidates.items():
        if sequence not in outcomes:
            sentinel.observe(sequence, reuse_checks_library().REUSED, stored)
        elif outcomes[sequence] in DECISIVE_OUTCOMES:
            sentinel.observe(sequence, reuse_checks_library().FORCED if sentinel.forced(sequence) else reuse_checks_library().JUDGED, outcomes[sequence])
    problems = sentinel.finish(printer=say)
    for message, howto in problems:
        print(f"  ✗ {message}", file=sys.stderr)
        print(f"  → 怎么办：{howto}", file=sys.stderr)
    return bool(problems)


# ── 分片（文件头「分片」）──────────────────────────────────────────────────────────────────────────────

def judging_usage(arguments):
    """判条的用法（剪枝跑、--item、--from、--package）：服务按分片三步起它；别的用法（--list-items、--summary、--refresh-list、--coverage-step、--selftest）不分。"""
    return not (arguments.list_items or arguments.summary or arguments.refresh_list or arguments.coverage_step or arguments.selftest)


def shard_plan_directory():
    """计划目录（task-server 给的 TASK_SERVER_SHARD_PLAN_DIRECTORY）；不在退 2。"""
    directory = os.environ.get("TASK_SERVER_SHARD_PLAN_DIRECTORY", "")
    if not directory or not os.path.isdir(directory):
        refuse(f"分片那一步没给在的计划目录（TASK_SERVER_SHARD_PLAN_DIRECTORY={directory or '没设'}）", "这一步只由 task-server 按分片协议起：task-server/task submit -- python3 scripts/mutation/broad-mutation.py …")
    return directory


def write_shard_plan(directory, note_text, piles):
    """plan.json 与每片的 shard-<片号>.items（一行一个变异名）；piles 是 [(label, [变异名…])]，空的堆不进来。片里只写 label（不钉台）。"""
    for index, (_label, names) in enumerate(piles):
        write_atomically(os.path.join(directory, f"shard-{index}.items"), "".join(f"{name}\n" for name in names).encode("utf-8"))
    write_atomically(os.path.join(directory, "plan.json"),
                     json.dumps({"shards": [{"label": label} for label, _names in piles], "note": note_text}, ensure_ascii=False).encode("utf-8"))


def shard_piles(items, states, selected, arguments, judgement):
    """计划那一步分两堆（文件头「分片」plan）：→ [(label, [变异名…])]，空的堆不要。selected 里的条按上一次的编译加测试秒数从长到短放进合计少的那一堆
    （用时取键相同的那一行；键对不上时取同一处变异（位置索引相同）最后一行有定论的编译加测试秒数；都没有的按罩得住的条里记过的中位数，一条都没记过按 1 秒；
    推的：上一次现判的用时）；罩得住而没选中的按序号单双分（GPU 那一趟与复用哨兵在各片自己那一堆里办）。
    弄坏开关 shard-seconds-without-locator：键对不上就当没记过，不看同一处变异的行。"""
    import statistics

    def seconds_of_row(row):
        try:
            return float(row.get("build_seconds") or "") + float(row.get("test_seconds") or "")
        except (TypeError, ValueError):
            return None

    def seconds_of(item):
        known_seconds = seconds_of_row(states.get(item["sequence"], (None, "", None))[2] or {})
        if known_seconds is not None or break_is("shard-seconds-without-locator"):
            return known_seconds
        for row in reversed(judgement.by_locator.get(judgement.key_of(item)["locator"], [])):
            known_seconds = seconds_of_row(row) if row["outcome"] in DECISIVE_OUTCOMES else None
            if known_seconds is not None:
                return known_seconds
        return None

    scoped = [item for item in items if in_run_scope(item, arguments)]
    seconds = {item["sequence"]: seconds_of(item) for item in scoped}
    known = [value for value in seconds.values() if value is not None]
    default = statistics.median(known) if known else 1.0
    heaps = [[0.0, []], [0.0, []]]
    chosen = {item["sequence"] for item, _one, _two in selected}
    chosen_names = {item["name"] for item, _one, _two in selected}
    costs = []
    for item, _one, _two in selected:
        own = seconds[item["sequence"]] if item["sequence"] in seconds else seconds_of(item)
        costs.append((own if own is not None else default, item))
    for cost, item in sorted(costs, key=lambda entry: (-entry[0], entry[1]["sequence"])):
        lighter = heaps[0] if heaps[0][0] <= heaps[1][0] else heaps[1]
        lighter[0] += cost
        lighter[1].append(item["name"])
    for item in scoped:
        if item["sequence"] not in chosen:
            heaps[item["sequence"] % 2][1].append(item["name"])
    names = ("头一堆", "另一堆")
    return [(f"{names[index]}：现判 {sum(1 for name in heap[1] if name in chosen_names)} 条约 {heap[0]:.0f} 秒、共 {len(heap[1])} 条（服务挑台）", heap[1])
            for index, heap in enumerate(heaps) if heap[1]]


def shard_scope_names():
    """这一片的那一堆（文件头「分片」shard）：计划目录里 shard-<片号>.items 的变异名集合；读不到退 2。"""
    shard = os.environ.get("TASK_SERVER_SHARD", "")
    index = shard.split("/")[0]
    path = os.path.join(shard_plan_directory(), f"shard-{index}.items")
    if not index.isdigit() or not os.path.isfile(path):
        refuse(f"这一片读不到自己那一堆（TASK_SERVER_SHARD={shard or '没设'}、{path}）", "计划那一步没写全：看计划那一步的日志")
    with open(path, encoding="utf-8") as handle:
        return {line.rstrip("\n") for line in handle if line.strip()}


def write_shard_counts(arguments, judged, carried, refused, gpu_totals):
    """一片收尾（文件头「分片」shard）：往 TASK_SERVER_SHARD_OUTPUT_DIRECTORY 写 counts.json，合并那一步把各片的数合起来报。不是分片的一片不写。"""
    output = os.environ.get("TASK_SERVER_SHARD_OUTPUT_DIRECTORY", "")
    if getattr(arguments, "shard_phase", "") != "shard" or not output:
        return
    os.makedirs(output, exist_ok=True)
    counts = dict(judged=judged, carried=carried, refused=refused, gpu_needed=gpu_totals.needed, gpu_judged=gpu_totals.judged,
                  gpu_carried=gpu_totals.carried, gpu_not_run=gpu_totals.not_run)
    write_atomically(os.path.join(output, "counts.json"), (json.dumps(counts) + "\n").encode("utf-8"))


def shard_counts_text():
    """合并那一步的成功句接的一段：各片 counts.json 合起来的数（TASK_SERVER_SHARD_OUTPUTS 每行「片号<TAB>台<TAB>产物目录」），缺的片照 TASK_SERVER_SHARD_MISSING 点名。"""
    import collections
    totals, read, unread = collections.Counter(), 0, []
    outputs = os.environ.get("TASK_SERVER_SHARD_OUTPUTS", "")
    if outputs and os.path.isfile(outputs):
        with open(outputs, encoding="utf-8") as handle:
            for line in handle:
                fields = line.rstrip("\n").split("\t")
                if len(fields) < 3:
                    continue
                try:
                    with open(os.path.join(fields[2], "counts.json"), encoding="utf-8") as counts_handle:
                        totals.update(json.load(counts_handle))
                    read += 1
                except (OSError, ValueError):
                    unread.append(fields[0])
    missing = os.environ.get("TASK_SERVER_SHARD_MISSING", "")
    return (f"{read} 片合起来现判 {totals['judged']}、沿用 {totals['carried']}、拒算 {totals['refused']}；"
            f"GPU 趟要 {totals['gpu_needed']} 条：现判 {totals['gpu_judged']}、复用 {totals['gpu_carried']}、本次未跑 {totals['gpu_not_run']}"
            + (f"；第 {'、'.join(unread)} 片的 counts.json 读不到" if unread else "") + (f"；缺第 {missing} 片（人手单独取消的）" if missing else ""))


def shard_phase_main(arguments):
    """服务按分片协议起的这一步（文件头「分片」）：判条的用法记下这一步交给 run_items；别的用法计划那一步写 0 片、交 0，其余交 None 照常往下跑。
    不是服务分片起的判条用法退 2（自证起的假一步（test_mode）照剪枝跑）。"""
    phase = os.environ.get("TASK_SERVER_SHARD_PHASE", "")
    arguments.shard_phase = phase
    if phase not in ("", "plan", "shard", "merge"):
        refuse(f"TASK_SERVER_SHARD_PHASE={phase} 认不出（只认 plan、shard、merge）", "这个变量只由 task-server 设；手跑时去掉它")
    if not judging_usage(arguments):
        if phase == "plan":
            write_shard_plan(shard_plan_directory(), "不判条的用法，不分", [])
            say("  · 不判条的用法：计划写 0 片，合并那一步照常跑")
            return 0
        arguments.shard_phase = ""
        return None
    if not phase and not test_mode():
        refuse("判条的用法只经统一任务服务分片起（环境里要有 TASK_SERVER_SHARD_PHASE）：这一次什么都没判",
               "经服务交：task-server/task submit -- python3 scripts/mutation/broad-mutation.py [选条参数]；服务拆成计划、两片、合并，合并那一步核账；"
               "提交门禁照常经服务交 bash .claude/gate.d/broad-mutation.sh")
    if phase == "plan":
        shard_plan_directory()
    if phase == "shard":
        arguments.shard_scope = shard_scope_names()
    return None
