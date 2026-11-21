#!/usr/bin/env python3
# admission: always 算的是此刻两台 /proc 与 cgroup 里的内存，上一次算的不替这一次作保
# run-condition: command bash python3 systemctl
"""按任务号现算内存：到两台的 /proc 与 cgroup 里找这条任务此刻的整棵进程树，逐进程加出占用（共享页按比例分摊）。只读，不停进程、不改任何东西。

用法：
  queue-health.py [<任务号>…] [--watch <秒>] [--interval <秒>]
      不给号算全部在跑的任务。--watch 在两台各自按间隔重算（默认每 DEFAULT_INTERVAL_SECONDS 秒一轮，第二台不每轮连一次 ssh），
      这段时间里每条任务留合计最大的那一轮，每台留现算合计最大的那一轮。
  queue-health.py --selftest   造出来的 /proc、cgroup 与假 nvidia-smi 走一遍各格，末尾逐个打开弄坏开关核它红在点名的格
登记成只给主 agent 直接跑的会话工具（.claude/hooks/lib_project_code_runs.py 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY）：要量的是此刻，排队就量不到了。

退出码：0 两台都读到、结论都对得上；1 有要看的结论（此刻找不到进程、现算与 cgroup 对不上、占用贴近上限、现算合计超过在跑上限之和）；
  2 用法错（含弄坏开关设在自证之外）；3 有读不到的（第二台连不上、显卡读不出、cgroup 读不了），读到的照打，同时有要看的结论退 1；
  78 两台都读不到，一条结论都下不了。

数从哪来（只有这几处，都是此刻现读；任务或脚本自己的声明，task status 的峰值、日志里的内存行、脚本标记、登记表都不读）：
  进程占用：逐进程 /proc/<进程号>/smaps_rollup 的 Pss_Anon + Pss_Shmem（共享页按映射它的进程数分摊，fork 出来的写时复制页与共享内存不在每个进程里重复计入，
    整棵树加起来就是这棵树真占的）加 /proc/<进程号>/status 的 VmPTE；smaps_rollup 读不到的进程退回 status 的 RssAnon + RssShmem（共享页各算一份，合计可能偏大），
    行里与结论里标出有几个进程走了退路。进程名取 status 的 Name
  cgroup 占用：memory.stat 的 anon 加 shmem（scripts/hosts/memory_peaks_lib.py 的 process_memory_bytes），不算页缓存，不读 memory.current、memory.peak
  上限：cgroup 的 memory.max，只用来对比；写着 max 的是没设上限
  显存：nvidia-smi --query-compute-apps=pid,used_memory 按进程号归到任务

怎么对上任务号（两台同一段，attempt_patterns 与 label_processes；尝试标签写作 task-<服务标识>-<任务号>-<第几次>）：
  ① 进程的 cgroup 路径里有 singlefs-<尝试标签>.service：服务起的尝试单元，归这个标签
  ② cgroup 路径里有 singlefs-memory-cap-*.scope：内存包装开的 scope。scope 里的进程全归同一个标签，取里面进程环境里最多的那个标签
  ③ 不在这两种 cgroup 里的，看自己的环境：TASK_SERVER_ATTEMPT_LABEL=<尝试标签>，或 CARGO_TARGET_DIR、MULTI_HOST_RUN_PEER_SCRATCH 指到 builds/<尝试标签>
  ④ 还没归上的，顺父进程往上找到归了标签的那一个就归它（自己 setsid 出去、又清了环境的后代）
  派到第二台的任务在本机另有尝试单元里的外壳（attempt_runner、multi-host-run.sh、ssh），两台各打一行。
互相核：每个 cgroup 里的进程 Pss_Anon + Pss_Shmem 之和（走退路的进程按 RssAnon + RssShmem）与这个 cgroup 的 memory.stat anon + shmem 比，
  差过 max(CROSS_CHECK_SLACK_BYTES, 大的那个数的 CROSS_CHECK_SLACK_RATIO) 报对不上。slice 那一层同样比一次。
  按比例分摊之后共享页不再重复，两边剩下的差：进程合计偏小是 tmpfs 文件、已退出进程留下的页，以及与 cgroup 外进程共享的页有一部分记在外面的进程名下；
  进程合计偏大是有进程走了退路，或读几百个进程的这段时间里内存在涨。
每台的对比：现算合计是 slice 下各 scope 里进程的合计，在跑上限之和是这些 scope 的 memory.max 之和，总额是 slice 的 memory.max。
  写现算合计占在跑上限之和的比例，超过上限之和才记问题；这个比例只是提示，不判上限记得对不对。
占用占上限：这条任务的合计除以它 scope 的 memory.max，到 NEAR_CAP_RATIO 记「占用贴近上限」。
找不到：点名的号两台都读到了、都没有它的进程，写「此刻找不到进程」；有一台读不到的写读不到，不说找不到。读不到的一律写「读不到」，不写 0。

第二台怎么连：scripts/hosts/host-dispatch.py 的 Peer（判多机配置，经 peer-host-lib.sh 的 peer_host_run 连，不写死主机名与地址）；送过去的是 memory_peaks_lib.py 的源码连同 PROBE_FUNCTIONS
  这几个函数的源码，gzip 再 base64，同 host-dispatch.py 的 probe_peer。两台对称：哪一台跑它，哪一台就是「本机」。
只给自证用的环境变量：QUEUE_HEALTH_PROCESS_ROOT（/proc 换成哪）、QUEUE_HEALTH_CGROUP_ROOT（/sys/fs/cgroup 换成哪）、QUEUE_HEALTH_SLICE_DIRECTORY（slice 的 cgroup 目录，不问 systemctl）、
  QUEUE_HEALTH_NVIDIA_SMI（nvidia-smi 换成哪）。

弄坏开关（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格（格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  QUEUE_HEALTH_BREAK=scope-members-unlabeled  scope 里的进程不跟着 scope 归标签，只看自己的环境
      「✗ 自证 whole-tree-found：」「✗ 自证 process-total-sums-status-fields：」
  QUEUE_HEALTH_BREAK=descendants-unfollowed  不顺父进程找，自己 setsid 出去又清了环境的后代归不上
      「✗ 自证 whole-tree-found：」「✗ 自证 process-total-sums-status-fields：」
  QUEUE_HEALTH_BREAK=unit-cgroup-unread  不认 cgroup 路径里的尝试单元
      「✗ 自证 whole-tree-found：」「✗ 自证 process-total-sums-status-fields：」
  QUEUE_HEALTH_BREAK=builds-directory-unread  不认环境里指到 builds/<尝试标签> 的路径
      「✗ 自证 builds-directory-names-attempt：」
  QUEUE_HEALTH_BREAK=every-process-labeled  归不上的进程都算进那个标签（把别人的内存算进任务）
      「✗ 自证 whole-tree-found：」「✗ 自证 process-total-sums-status-fields：」「✗ 自证 unrelated-process-excluded：」「✗ 自证 gpu-by-process-read：」
  QUEUE_HEALTH_BREAK=page-table-dropped  合计不加 VmPTE
      「✗ 自证 process-total-sums-status-fields：」
  QUEUE_HEALTH_BREAK=pss-read-as-rss  smaps_rollup 读得到也不用 Pss_Anon + Pss_Shmem，读回 status 的 RssAnon + RssShmem（共享页各算一份）
      「✗ 自证 shared-pages-split-proportionally：」「✗ 自证 shared-pages-not-reported-as-mismatch：」
  QUEUE_HEALTH_BREAK=rss-fallback-dropped  smaps_rollup 读不到的进程算 0，不退回 RssAnon + RssShmem
      「✗ 自证 rss-fallback-reads-status：」
  QUEUE_HEALTH_BREAK=rss-fallback-unflagged  走了退路的进程不标出来
      「✗ 自证 rss-fallback-flagged：」
  QUEUE_HEALTH_BREAK=page-cache-counted  cgroup 占用读 memory.current（连页缓存）
      「✗ 自证 page-cache-not-counted：」
  QUEUE_HEALTH_BREAK=memory-max-read-as-unlimited  memory.max 一律当没设上限
      「✗ 自证 cap-read-from-memory-max：」
  QUEUE_HEALTH_BREAK=gpu-unread-as-zero  显存读成 [N/A] 的当 0
      「✗ 自证 gpu-by-process-read：」
  QUEUE_HEALTH_BREAK=gpu-other-process-counted  显卡上每个计算进程都算进任务，不按进程号归
      「✗ 自证 gpu-by-process-read：」
  QUEUE_HEALTH_BREAK=watch-keeps-latest  --watch 留最后一轮，不留最大的那一轮
      「✗ 自证 watch-keeps-maximum：」
  QUEUE_HEALTH_BREAK=unread-host-as-absent  有一台读不到也说此刻找不到进程
      「✗ 自证 no-process-with-unread-host：」
  QUEUE_HEALTH_BREAK=cross-check-skipped  不比现算与 cgroup
      「✗ 自证 cross-check-mismatch-red：」
  QUEUE_HEALTH_BREAK=unread-cgroup-as-zero  cgroup 读不了的当 0（进程合计对着 0 比，报告里多一行「现算与 cgroup 对不上」，unread-printed-not-zero 把整份报告原文打进失败说明，那一行也以 ✗ 起头）
      「✗ 自证 cross-check-unread-red：」「✗ 自证 unread-printed-not-zero：」「✗ 任务 501 第 2 次 · 本机：现算与 cgroup 对不上」
  QUEUE_HEALTH_BREAK=near-cap-unjudged  占用贴着上限也不报
      「✗ 自证 near-cap-red：」
  QUEUE_HEALTH_BREAK=caps-ratio-omitted  每台那一行不写现算合计占在跑上限之和的比例
      「✗ 自证 caps-ratio-in-text：」
  QUEUE_HEALTH_BREAK=usage-above-caps-unjudged  现算合计超过上限之和也不报
      「✗ 自证 usage-above-caps-red：」
  QUEUE_HEALTH_BREAK=no-running-despite-unread  有一台读不到也说此刻没有在跑任务的进程
      「✗ 自证 no-running-needs-every-host：」
  QUEUE_HEALTH_BREAK=unread-exit-zero  有读不到的也退 0
      「✗ 自证 exit-three-unread：」
  QUEUE_HEALTH_BREAK=nothing-read-exit-zero  两台都读不到也退 0
      「✗ 自证 exit-seventy-eight-nothing-read：」
  QUEUE_HEALTH_BREAK=unread-shown-as-zero  读不到的打成 0 MiB
      「✗ 自证 unread-printed-not-zero：」
  QUEUE_HEALTH_BREAK=top-names-unsorted  按进程名分的前几项不按占用排
      「✗ 自证 top-names-sorted：」
  QUEUE_HEALTH_BREAK=requested-filter-ignored  点了号也打全部任务
      「✗ 自证 requested-number-filters：」
  QUEUE_HEALTH_BREAK=peer-failure-read-as-empty  第二台连不上当它一条任务都没有
      「✗ 自证 peer-failure-unread：」
  QUEUE_HEALTH_BREAK=task-number-unchecked  任务号不是正整数也收
      「✗ 自证 usage-refused：」
  QUEUE_HEALTH_BREAK=source-form-unmatched  核尝试标签与 scope 的写法时拿一串对不上的字去找
      「✗ 自证 attempt-forms-match-sources：」
  QUEUE_HEALTH_BREAK=imported-modules-unread  读任务服务源码只读入口那一份，不读它的拆分族（拆出并真载入的同目录模块）
      「✗ 自证 attempt-forms-match-sources：」「✗ 自证 attempt-forms-found-in-imported-modules：」「✗ 自证 attempt-forms-found-in-import-module-modules：」
  QUEUE_HEALTH_BREAK=unimported-modules-read  读任务服务源码把同目录每份 .py 都读，不认拆分族
      「✗ 自证 attempt-forms-skip-unimported-modules：」「✗ 自证 attempt-forms-skip-name-like-unloaded-modules：」「✗ 自证 attempt-forms-skip-loaded-shared-modules：」「✗ 自证 attempt-forms-skip-comment-named-modules：」
  QUEUE_HEALTH_BREAK=name-like-unloaded-read  读任务服务源码只按名字认拆分族：名字像部分文件的同目录模块都读，不看入口载没载入
      「✗ 自证 attempt-forms-skip-name-like-unloaded-modules：」「✗ 自证 attempt-forms-skip-comment-named-modules：」
  QUEUE_HEALTH_BREAK=loaded-shared-read  读任务服务源码只按载入认拆分族：入口 import 的同目录模块都读，不看名字像不像部分文件
      「✗ 自证 attempt-forms-skip-loaded-shared-modules：」
  QUEUE_HEALTH_BREAK=loaded-by-text-read  读任务服务源码认拆分族的载入按文本行扫：注释、文档串里写的文件名与 import 行都算载入，importlib.import_module 认不出
      「✗ 自证 attempt-forms-skip-comment-named-modules：」「✗ 自证 attempt-forms-found-in-import-module-modules：」
"""
import os as preflight_os, sys as preflight_sys  # noqa: E401,E402
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import base64  # noqa: E402
import concurrent.futures  # noqa: E402
import gzip  # noqa: E402
import importlib.util  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, SCRIPT_DIRECTORY)
import memory_peaks_lib as peaks  # noqa: E402

BREAK_VARIABLE = "QUEUE_HEALTH_BREAK"
MEBIBYTE = 1024 * 1024
DEFAULT_INTERVAL_SECONDS = 1.0
PEER_TIMEOUT_MARGIN_SECONDS = 60          # 第二台那一趟至多等 --watch 的秒数加这么久
PROBE_ARGUMENT_LIMIT = 96 * 1024          # 程序压完的 base64 是 bash -c 的一个参数，同 host-dispatch.py 的 PROBE_PROGRAM_ARGUMENT_LIMIT
PROBE_MARK = "QUEUE_HEALTH_PROBE "
CROSS_CHECK_SLACK_BYTES = 64 * MEBIBYTE   # 按比例分摊之后两边剩下的零头：小任务读取期间的增长、取整
CROSS_CHECK_SLACK_RATIO = 0.1             # 大任务读几百个进程那段时间里的增长，与 cgroup 外进程共享的页
NEAR_CAP_RATIO = 0.9
TOP_NAME_COUNT = 5
THIS_HOST_NAME, PEER_HOST_NAME = "本机", "第二台"
LEVEL_FINE, LEVEL_NOTICE, LEVEL_PROBLEM, LEVEL_UNREAD = "fine", "notice", "problem", "unread"
EXIT_FINE, EXIT_PROBLEM, EXIT_USAGE, EXIT_UNREAD, EXIT_NOTHING_READ = 0, 1, 2, 3, 78
# 这几种写法照抄自任务服务（task-server/task_server.py 加它的拆分族）与内存包装；attempt-forms-match-sources 格核它们还在
ATTEMPT_UNIT_SOURCE_FORM = "singlefs-task-{self.store.tag}-{task['id']}-{number}.service"
ATTEMPT_LABEL_SOURCE_FORM = "task-{self.store.tag}-{task['id']}-{number}"
SCOPE_SOURCE_FORM = 'unit="singlefs-memory-cap-'


# ── 读这一台（第二台上由 PROBE_FUNCTIONS 这几份源码现跑，只用标准库与 memory_peaks_lib，不引用模块层的名字）──
def broken(item, environment):
    return item in (environment.get("QUEUE_HEALTH_BREAK") or "").split(",")


def split_family_library():
    """scripts/gate/lib_split_family_loaded.py：认「入口加它拆出的部分文件」只在那一份里写，按路径载入。"""
    path = os.path.join(REPOSITORY_ROOT, "scripts", "gate", "lib_split_family_loaded.py")
    specification = importlib.util.spec_from_file_location("lib_split_family_loaded_for_queue_health", path)
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)
    return library


def entry_with_split_family(entry, environment):
    """入口加它的拆分族的路径，入口在前：族是名字落在命名族里、又被入口（或族里别的文件）载入的同目录 .py（import、from … import、字面量参数的 importlib.import_module，
    注释与文档串里写的不算），认法在 split_family_library；不按通配、不写名单。"""
    library = split_family_library()
    candidates = [path for path in library.sibling_files(entry) if path.endswith(".py")]
    if broken("unimported-modules-read", environment):
        return [entry] + [path for path in candidates if path != entry]
    if broken("imported-modules-unread", environment):
        return [entry]
    defects = set()
    if broken("name-like-unloaded-read", environment):
        defects.add(library.NAME_ONLY)
    if broken("loaded-shared-read", environment):
        defects.add(library.LOADED_ONLY)
    if broken("loaded-by-text-read", environment):
        defects.add(library.TEXT_SCAN)
    return library.loaded_family(entry, candidates, defects)


def forms_found_in_files(paths, forms, suffix=""):
    """每种写法（后面接 suffix）在这几份文件的文本里找不找得到：→ {写法: 是否}。"""
    text = ""
    for path in paths:
        with open(path, encoding="utf-8") as handle:
            text += handle.read() + "\n"
    return {form: form + suffix in text for form in forms}


def attempt_patterns():
    """尝试标签的几种认法（文件头「怎么对上任务号」）。"""
    label = r"task-[0-9A-Za-z]+-[0-9]+-[0-9]+"
    return {"unit": re.compile(rf"^singlefs-({label})\.service$"),
            "scope": re.compile(r"^singlefs-memory-cap-[^/]*\.scope$"),
            "environment_label": re.compile(rb"(?:^|\0)TASK_SERVER_ATTEMPT_LABEL=(" + label.encode("ascii") + rb")(?:\0|$)"),
            "builds_directory": re.compile(rb"(?:^|\0)(?:CARGO_TARGET_DIR|MULTI_HOST_RUN_PEER_SCRATCH)=[^\0]*/builds/(" + label.encode("ascii") + rb")(?:/|\0|$)")}


def read_smaps_rollup(directory):
    """/proc/<进程号>/smaps_rollup 的 Pss_Anon 与 Pss_Shmem（字节）；读不了或缺这两行交 None。"""
    try:
        with open(os.path.join(directory, "smaps_rollup"), encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    found = {}
    for line in text.splitlines():
        name, _separator, content = line.partition(":")
        words = content.split()
        if name in ("Pss_Anon", "Pss_Shmem") and words and words[0].isdigit():
            found[name] = int(words[0]) * 1024
    return found if len(found) == 2 else None


def read_process_rows(process_root, patterns, environment):
    """/proc 下每个进程一行：进程号、父进程、名字、anon 与 shmem（字节）、页表、cgroup 路径、环境里的标签、是否走了退路。
    anon 与 shmem 取 smaps_rollup 的 Pss_Anon、Pss_Shmem；读不到的退回 status 的 RssAnon、RssShmem，其中有占用的记走了退路（rss_fallback 为 1）。
    读的时候退出了的跳过；内核线程没有这几行，算 0，不记走了退路。"""
    rows = []
    for entry in os.listdir(process_root):
        if not entry.isdigit():
            continue
        directory = os.path.join(process_root, entry)
        try:
            with open(os.path.join(directory, "status"), encoding="utf-8", errors="replace") as handle:
                status_text = handle.read()
        except OSError:
            continue
        row = {"process_id": int(entry), "parent": 0, "name": "", "anon": 0, "shmem": 0, "page_tables": 0, "cgroup_path": "", "environment_label": "", "rss_fallback": 0}
        field_keys = {"RssAnon": "anon", "RssShmem": "shmem", "VmPTE": "page_tables"}
        for line in status_text.splitlines():
            name, _separator, content = line.partition(":")
            words = content.split()
            if name == "Name":
                row["name"] = content.strip()
            elif name == "PPid" and words and words[0].isdigit():
                row["parent"] = int(words[0])
            elif name in field_keys and words and words[0].isdigit():
                row[field_keys[name]] = int(words[0]) * 1024
        proportional = read_smaps_rollup(directory)
        if proportional is not None:
            if not broken("pss-read-as-rss", environment):
                row["anon"], row["shmem"] = proportional["Pss_Anon"], proportional["Pss_Shmem"]
        else:
            row["rss_fallback"] = 0 if broken("rss-fallback-unflagged", environment) or not (row["anon"] or row["shmem"]) else 1
            if broken("rss-fallback-dropped", environment):
                row["anon"] = row["shmem"] = 0
        try:
            with open(os.path.join(directory, "cgroup"), encoding="utf-8", errors="replace") as handle:
                for line in handle:
                    if line.startswith("0::"):
                        row["cgroup_path"] = line[3:].strip()
        except OSError:
            pass
        try:
            with open(os.path.join(directory, "environ"), "rb") as handle:
                process_environment = handle.read()
        except OSError:
            process_environment = b""
        found = patterns["environment_label"].search(process_environment)
        if found is None and not broken("builds-directory-unread", environment):
            found = patterns["builds_directory"].search(process_environment)
        if found is not None:
            row["environment_label"] = found.group(1).decode("ascii")
        rows.append(row)
    return rows


def cgroup_group(cgroup_path, patterns, environment):
    """cgroup 路径 → (归到的那一层：尝试单元或内存包装的 scope；单元认出的标签)；都不是交两个空串。"""
    prefix = ""
    for component in cgroup_path.strip("/").split("/"):
        prefix += "/" + component
        found = patterns["unit"].match(component)
        if found is not None:
            return prefix, "" if broken("unit-cgroup-unread", environment) else found.group(1)
        if patterns["scope"].match(component):
            return prefix, ""
    return "", ""


def label_processes(rows, patterns, environment):
    """给每一行定 group 与 label（文件头「怎么对上任务号」）：单元、scope、自己的环境、父进程，依次取。"""
    scope_votes = {}
    for row in rows:
        row["group"], row["label"] = cgroup_group(row["cgroup_path"], patterns, environment)
        if row["group"] and not row["label"] and row["environment_label"]:
            votes = scope_votes.setdefault(row["group"], {})
            votes[row["environment_label"]] = votes.get(row["environment_label"], 0) + 1
    scope_labels = {group: max(sorted(votes), key=votes.get) for group, votes in scope_votes.items()}
    for row in rows:
        if not row["label"]:
            from_scope = "" if broken("scope-members-unlabeled", environment) else scope_labels.get(row["group"], "")
            row["label"] = from_scope or row["environment_label"]
    if not broken("descendants-unfollowed", environment):
        by_process = {row["process_id"]: row for row in rows}
        changed = True
        while changed:
            changed = False
            for row in rows:
                parent = by_process.get(row["parent"])
                if not row["label"] and parent is not None and parent["label"]:
                    row["label"] = parent["label"]
                    changed = True
    if broken("every-process-labeled", environment):
        labels = sorted({row["label"] for row in rows if row["label"]})
        for row in rows:
            row["label"] = row["label"] or (labels[0] if labels else "")


def read_compute_processes(environment):
    """显卡上此刻的计算进程：{"rows": [[进程号, 已用 MiB 或 None]…], "unattributed": 进程号读不出的行数, "problem": 读不出的原因}。"""
    command = [environment.get("QUEUE_HEALTH_NVIDIA_SMI") or "nvidia-smi", "--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"]
    try:
        completed = subprocess.run(command, capture_output=True, text=True, timeout=30, stdin=subprocess.DEVNULL)
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"rows": [], "unattributed": 0, "problem": f"nvidia-smi 起不来或超时（{type(error).__name__}）"}
    if completed.returncode != 0:
        return {"rows": [], "unattributed": 0, "problem": f"nvidia-smi 退 {completed.returncode}"}
    rows, unattributed = [], 0
    for line in completed.stdout.splitlines():
        fields = [piece.strip() for piece in line.split(",")]
        if len(fields) != 2 or not fields[0].isdigit():
            unattributed += 1
            continue
        if fields[1].isdigit():
            rows.append([int(fields[0]), int(fields[1])])
        else:
            rows.append([int(fields[0]), 0 if broken("gpu-unread-as-zero", environment) else None])
    return {"rows": rows, "unattributed": unattributed, "problem": ""}


def read_memory_limit(directory, environment):
    """cgroup 的 memory.max → (字节或 None, limit|no-limit|unread)。"""
    if broken("memory-max-read-as-unlimited", environment):
        return None, "no-limit"
    try:
        with open(os.path.join(directory, "memory.max"), encoding="utf-8") as handle:
            text = handle.read().strip()
    except OSError:
        return None, "unread"
    if text == "max":
        return None, "no-limit"
    return (int(text), "limit") if text.isdigit() else (None, "unread")


def read_cgroup_anon_shmem(library, directory, environment):
    """cgroup 的 anon 加 shmem（字节）；目录不在或读不了交 None。"""
    if not os.path.isdir(directory):
        return None
    if broken("page-cache-counted", environment):
        try:
            with open(os.path.join(directory, "memory.current"), encoding="utf-8") as handle:
                return int(handle.read().strip())
        except (OSError, ValueError):
            return None
    return library.process_memory_bytes(directory)


def aggregate_host(rows, compute_processes, environment):
    """→ (attempts：{标签: 这条任务在这台的合计}, groups：{cgroup 那一层: 里面进程的合计})。"""
    attempts, groups = {}, {}
    for row in rows:
        total = row["anon"] + row["shmem"] + (0 if broken("page-table-dropped", environment) else row["page_tables"])
        anon_shmem = row["anon"] + row["shmem"]
        if row["group"]:
            group = groups.setdefault(row["group"], {"path": row["group"], "process_count": 0, "process_anon_shmem": 0, "process_total": 0,
                                                                  "rss_fallback_count": 0})
            group["process_count"] += 1
            group["rss_fallback_count"] += row["rss_fallback"]
            group["process_anon_shmem"] += anon_shmem
            group["process_total"] += total
        if not row["label"]:
            continue
        attempt = attempts.setdefault(row["label"], {"label": row["label"], "process_count": 0, "total_bytes": 0, "names": {}, "group_paths": [],
                                                     "gpu_mebibytes": 0, "gpu_processes": 0, "gpu_unread": 0, "rss_fallback_count": 0})
        attempt["process_count"] += 1
        attempt["rss_fallback_count"] += row["rss_fallback"]
        attempt["total_bytes"] += total
        by_name = attempt["names"].setdefault(row["name"], [0, 0])
        by_name[0] += total
        by_name[1] += 1
        if row["group"] and row["group"] not in attempt["group_paths"]:
            attempt["group_paths"].append(row["group"])
    by_process = {row["process_id"]: row for row in rows}
    for process_id, mebibytes in compute_processes["rows"]:
        row = by_process.get(process_id)
        labels = list(attempts) if broken("gpu-other-process-counted", environment) else ([row["label"]] if row is not None and row["label"] else [])
        for label in labels:
            attempts[label]["gpu_processes"] += 1
            if mebibytes is None:
                attempts[label]["gpu_unread"] += 1
            else:
                attempts[label]["gpu_mebibytes"] += mebibytes
    return attempts, groups


def read_host_snapshot(library, environment):
    """这一台此刻的一份：每条任务的合计与它那几层 cgroup 的读数、这台的对比数、显卡。/proc 列不了抛 OSError。"""
    process_root = environment.get("QUEUE_HEALTH_PROCESS_ROOT") or "/proc"
    cgroup_root = (environment.get("QUEUE_HEALTH_CGROUP_ROOT") or "/sys/fs/cgroup").rstrip("/")
    slice_directory = environment.get("QUEUE_HEALTH_SLICE_DIRECTORY") or library.slice_directory_of(library.DEFAULT_SLICE, cgroup_root)
    patterns = attempt_patterns()
    rows = read_process_rows(process_root, patterns, environment)
    label_processes(rows, patterns, environment)
    compute_processes = read_compute_processes(environment)
    attempts, groups = aggregate_host(rows, compute_processes, environment)
    totals = {"usage_bytes": 0, "usage_anon_shmem": 0, "rss_fallback_count": 0, "caps_bytes": 0, "caps_unread": 0, "slice_limit": None, "slice_anon_shmem": None,
              "slice_problem": ""}
    slice_prefix = slice_directory.rstrip("/") + "/" if slice_directory else ""
    for group in groups.values():
        directory = cgroup_root + group["path"]
        group["cgroup_anon_shmem"] = read_cgroup_anon_shmem(library, directory, environment)
        group["limit_bytes"], group["limit_state"] = read_memory_limit(directory, environment)
        group["in_slice"] = bool(slice_prefix) and directory.startswith(slice_prefix)
        if group["in_slice"]:
            totals["usage_bytes"] += group["process_total"]
            totals["usage_anon_shmem"] += group["process_anon_shmem"]
            totals["rss_fallback_count"] += group["rss_fallback_count"]
            totals["caps_bytes"] += group["limit_bytes"] or 0
            totals["caps_unread"] += 1 if group["limit_state"] == "unread" else 0
    if slice_directory:
        totals["slice_limit"], _limit_state = read_memory_limit(slice_directory, environment)
        totals["slice_anon_shmem"] = read_cgroup_anon_shmem(library, slice_directory, environment)
    else:
        totals["slice_problem"] = "slice 还没起" if slice_directory == "" else "查不了用户级 systemd，slice 的 cgroup 目录找不到"
    for attempt in attempts.values():
        attempt["groups"] = [groups[path] for path in attempt.pop("group_paths")]
    return {"attempts": attempts, "totals": totals, "gpu_problem": compute_processes["problem"], "gpu_unattributed": compute_processes["unattributed"], "rounds": 1}


def merge_watch_round(best, snapshot, environment):
    """--watch 的一轮并进这段时间的最大值：每条任务留合计最大的那一轮（显存另取最大），这台留现算合计最大的那一轮。"""
    if best is None:
        return snapshot
    rounds = best["rounds"] + 1
    if broken("watch-keeps-latest", environment):
        snapshot["rounds"] = rounds
        return snapshot
    for label, attempt in snapshot["attempts"].items():
        kept = best["attempts"].get(label)
        if kept is None:
            best["attempts"][label] = attempt
            continue
        gpu_mebibytes = max(kept["gpu_mebibytes"], attempt["gpu_mebibytes"])
        if attempt["total_bytes"] > kept["total_bytes"]:
            best["attempts"][label] = attempt
        best["attempts"][label]["gpu_mebibytes"] = gpu_mebibytes
    if snapshot["totals"]["usage_bytes"] > best["totals"]["usage_bytes"]:
        best["totals"] = snapshot["totals"]
    best["gpu_problem"] = best["gpu_problem"] or snapshot["gpu_problem"]
    best["gpu_unattributed"] = max(best["gpu_unattributed"], snapshot["gpu_unattributed"])
    best["rounds"] = rounds
    return best


def watch_host(library, environment, seconds, interval, clock, pause):
    """按间隔重算到时限，交这段时间的最大值；seconds 是 0 就只算一轮。"""
    deadline = clock() + seconds
    best = None
    while True:
        best = merge_watch_round(best, read_host_snapshot(library, environment), environment)
        if clock() + interval > deadline:
            return best
        pause(interval)


PROBE_FUNCTIONS = (broken, attempt_patterns, read_smaps_rollup, read_process_rows, cgroup_group, label_processes, read_compute_processes, read_memory_limit, read_cgroup_anon_shmem,
                   aggregate_host, read_host_snapshot, merge_watch_round, watch_host)
PROBE_IMPORTS = "import json, os, re, subprocess, sys, time"
PROBE_DRIVER = '''
_report = watch_host(sys.modules["__main__"], dict(os.environ), float(sys.argv[1]), float(sys.argv[2]), time.monotonic, time.sleep)
print("QUEUE_HEALTH_PROBE " + json.dumps(_report))
'''


# ── 两台都读 ──
def peer_probe_program():
    with open(os.path.join(SCRIPT_DIRECTORY, "memory_peaks_lib.py"), encoding="utf-8") as handle:
        library = handle.read()
    return "\n".join([library, PROBE_IMPORTS] + [inspect.getsource(function) for function in PROBE_FUNCTIONS] + [PROBE_DRIVER])


def parse_probe_report(stdout):
    line = next((line for line in reversed(stdout.splitlines()) if line.startswith(PROBE_MARK)), None)
    if line is None:
        return None
    try:
        return json.loads(line[len(PROBE_MARK):])
    except ValueError:
        return None


def empty_snapshot():
    return {"attempts": {}, "totals": {"usage_bytes": 0, "usage_anon_shmem": 0, "rss_fallback_count": 0, "caps_bytes": 0, "caps_unread": 0, "slice_limit": None, "slice_anon_shmem": None, "slice_problem": ""},
            "gpu_problem": "", "gpu_unattributed": 0, "rounds": 1}


def read_peer_host(peer, seconds, interval, environment):
    """第二台：→ (这一份或 None, 读不到的原因)。"""
    if not peer.available:
        return None, f"第二台读不到：{peer.why}"
    program = base64.b64encode(gzip.compress(peer_probe_program().encode("utf-8"), 9)).decode("ascii")
    if len(program) > PROBE_ARGUMENT_LIMIT:
        return None, f"第二台读不到：探测程序压完 {len(program)} 字节，超过单个参数的界 {PROBE_ARGUMENT_LIMIT}"
    text = (f"printf %s {program} | base64 -d | gzip -dc | "
            f"python3 -B -c 'import sys; exec(compile(sys.stdin.read(), \"<queue-health probe>\", \"exec\"))' {seconds} {interval}")
    status, stdout, stderr = peer.run(text, timeout=seconds + PEER_TIMEOUT_MARGIN_SECONDS)
    report = parse_probe_report(stdout)
    if status != 0 or report is None:
        if broken("peer-failure-read-as-empty", environment):
            return empty_snapshot(), ""
        return None, f"第二台读不到（退 {status}：{stderr.strip()[-300:]}）"
    return report, ""


def read_this_host(seconds, interval, environment):
    try:
        return watch_host(peaks, environment, seconds, interval, time.monotonic, time.sleep), ""
    except (OSError, ValueError) as error:
        return None, f"本机读不到（{type(error).__name__}: {error}）"


def load_peer():
    """host-dispatch.py 的 Peer；多机配置判不过时 available 为假、why 写为什么。"""
    specification = importlib.util.spec_from_file_location("host_dispatch_for_queue_health", os.path.join(SCRIPT_DIRECTORY, "host-dispatch.py"))
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module   # dataclass 按模块名找所在模块
    specification.loader.exec_module(module)
    return module.Peer(module.settings_from_environment())


def read_every_host(seconds, interval, environment):
    """两台并行读（--watch 要同一段时间）：→ [{name, snapshot, problem}…]，次序固定。"""
    def read_peer():
        try:
            peer = load_peer()
        except (OSError, ImportError) as error:
            return None, f"第二台读不到（多机配置载不进：{type(error).__name__}: {error}）"
        return read_peer_host(peer, seconds, interval, environment)

    readers = ((THIS_HOST_NAME, lambda: read_this_host(seconds, interval, environment)), (PEER_HOST_NAME, read_peer))
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(readers)) as pool:
        futures = [(name, pool.submit(reader)) for name, reader in readers]
    hosts = []
    for name, future in futures:
        snapshot, problem = future.result()
        hosts.append({"name": name, "snapshot": snapshot, "problem": problem})
    return hosts


# ── 判（纯函数：喂造出来的读数）──
def task_number_of(label):
    return int(label.rsplit("-", 2)[1])


def attempt_number_of(label):
    return int(label.rsplit("-", 1)[1])


def attempt_cap(attempt):
    """这条任务在这台的上限：slice 下 scope 的 memory.max 里最小的 → (字节或 None, limit|unread|none)。只有外壳、不在 scope 里的是 none。"""
    in_slice = [group for group in attempt["groups"] if group.get("in_slice")]
    limits = [group["limit_bytes"] for group in in_slice if group.get("limit_state") == "limit"]
    if limits:
        return min(limits), "limit"
    return None, "unread" if any(group.get("limit_state") == "unread" for group in in_slice) else "none"


def size_text(byte_count, environment):
    if byte_count is None:
        return "0 MiB" if broken("unread-shown-as-zero", environment) else "读不到"
    return peaks.human(byte_count)


def process_basis_text(group):
    """互相核时进程那一边的口径：全是 Pss 的写 Pss_Anon + Pss_Shmem，有走退路的注明几个。"""
    text = "Pss_Anon + Pss_Shmem"
    if group["rss_fallback_count"]:
        text += f"（其中 {group['rss_fallback_count']} 个进程读不到 smaps_rollup，按 RssAnon + RssShmem）"
    return text


def cross_check_differs(process_bytes, cgroup_bytes):
    return abs(process_bytes - cgroup_bytes) > max(CROSS_CHECK_SLACK_BYTES, CROSS_CHECK_SLACK_RATIO * max(process_bytes, cgroup_bytes))


def selected_attempts(view, environment):
    """要打的任务：→ [(台名, 这条任务在这台的合计)…]，按任务号排；没点号的是全部。"""
    requested = view["requested"]
    selected = []
    for host in view["hosts"]:
        for label, attempt in (host["snapshot"] or {"attempts": {}})["attempts"].items():
            if not requested or task_number_of(label) in requested or broken("requested-filter-ignored", environment):
                selected.append((host["name"], attempt))
    return sorted(selected, key=lambda pair: (task_number_of(pair[1]["label"]), attempt_number_of(pair[1]["label"]), pair[0]))


def conclusion(kind, level, text, basis, next_step=""):
    return {"kind": kind, "level": level, "text": text, "basis": basis, "next_step": next_step}


def judge_presence(view, environment):
    """点名的号找不找得到、有没有在跑的。"""
    found = set()
    for host in view["hosts"]:
        found |= {task_number_of(label) for label in (host["snapshot"] or {"attempts": {}})["attempts"]}
    unread_hosts = [host["name"] for host in view["hosts"] if host["snapshot"] is None]
    results = []
    for number in view["requested"]:
        if number in found:
            continue
        if unread_hosts and not broken("unread-host-as-absent", environment):
            read_names = [host["name"] for host in view["hosts"] if host["snapshot"] is not None]
            results.append(conclusion("unread", LEVEL_UNREAD, f"任务 {number}：{'、'.join(read_names) or '两台都'}没找到进程，{'、'.join(unread_hosts)}读不到",
                                      "读不到的那一台不算找不到"))
        else:
            results.append(conclusion("no-process", LEVEL_PROBLEM, f"任务 {number}：此刻找不到进程", "两台的 /proc 与 cgroup 里都没有归到这个号的进程",
                                      f"核号对不对；task-server/task status {number} 看它是还在排还是已有结局"))
    if not view["requested"] and not found and (not unread_hosts or broken("no-running-despite-unread", environment)):
        results.append(conclusion("no-running", LEVEL_FINE, "此刻没有在跑任务的进程", "两台的 /proc 与 cgroup 里都没有归到尝试标签的进程"))
    return results


def judge_attempts(view, environment):
    """选中的这几条：现算与 cgroup 对不对得上、占上限多少。"""
    results, matched, ratios = [], 0, []
    for host_name, attempt in selected_attempts(view, environment):
        title = f"任务 {task_number_of(attempt['label'])} 第 {attempt_number_of(attempt['label'])} 次 · {host_name}"
        for group in attempt["groups"]:
            cgroup_bytes = group.get("cgroup_anon_shmem")
            if cgroup_bytes is None and broken("unread-cgroup-as-zero", environment):
                cgroup_bytes = 0
            if cgroup_bytes is None:
                results.append(conclusion("unread", LEVEL_UNREAD, f"{title}：cgroup 读不到", f"{group['path']} 的 memory.stat 读不了或目录已不在"))
            elif cross_check_differs(group["process_anon_shmem"], cgroup_bytes) and not broken("cross-check-skipped", environment):
                results.append(conclusion("cross-check-mismatch", LEVEL_PROBLEM, f"{title}：现算与 cgroup 对不上",
                                          f"进程 {process_basis_text(group)} 之和 {size_text(group['process_anon_shmem'], environment)}，"
                                          f"cgroup anon + shmem {size_text(cgroup_bytes, environment)}（{group['path']}）",
                                          "进程合计偏小是 tmpfs 文件、已退出进程留下的页，或与 cgroup 外进程共享的页有一部分记在外面；"
                                          "偏大是有进程走了退路（共享页各算一份）或读取期间内存在涨；差得离谱就看那个 cgroup 的 cgroup.procs"))
            else:
                matched += 1
        if attempt["rss_fallback_count"]:
            results.append(conclusion("rss-fallback", LEVEL_NOTICE, f"{title}：{attempt['rss_fallback_count']} 个进程读不到 smaps_rollup，按 RssAnon + RssShmem 算",
                                      "这几个进程里的共享页（fork 出来的写时复制页、共享内存）在每个映射它的进程里各算一份，合计可能偏大",
                                      "核这些进程的 /proc/<进程号>/smaps_rollup 为什么读不了（进程刚退出、没有权限）；合计偏大时以 cgroup anon + shmem 为准"))
        cap, cap_state = attempt_cap(attempt)
        if cap_state == "unread":
            results.append(conclusion("unread", LEVEL_UNREAD, f"{title}：上限读不到", "scope 的 memory.max 读不了"))
        elif cap_state == "limit" and cap:
            ratio = attempt["total_bytes"] / cap
            ratios.append(ratio)
            if ratio >= NEAR_CAP_RATIO and not broken("near-cap-unjudged", environment):
                results.append(conclusion("near-cap", LEVEL_PROBLEM, f"{title}：占用贴近上限",
                                          f"合计 {size_text(attempt['total_bytes'], environment)}，memory.max {size_text(cap, environment)}，占 {ratio:.0%}（线 {NEAR_CAP_RATIO:.0%}）",
                                          "撞上限由服务放大重排，不用手动处置；连着撞的核代码或收小范围"))
        if attempt["gpu_unread"]:
            results.append(conclusion("unread", LEVEL_UNREAD, f"{title}：显存读不到", f"{attempt['gpu_unread']} 个进程的 used_memory 不是数"))
    if matched and not any(item["kind"] == "cross-check-mismatch" for item in results):
        results.append(conclusion("cross-check-matches", LEVEL_FINE, "现算与 cgroup 对得上", f"核了 {matched} 个 cgroup，差都在容差以内"))
    if ratios and not any(item["kind"] == "near-cap" for item in results):
        results.append(conclusion("within-cap", LEVEL_FINE, "占用都在上限以内", f"最高占 {max(ratios):.0%}（线 {NEAR_CAP_RATIO:.0%}）"))
    return results


def judge_hosts(view, environment):
    """每台：读不读得到，现算合计与在跑上限之和比。"""
    results = []
    for host in view["hosts"]:
        name, snapshot = host["name"], host["snapshot"]
        if snapshot is None:
            results.append(conclusion("unread", LEVEL_UNREAD, f"{name}读不到", host["problem"]))
            continue
        if snapshot["gpu_problem"]:
            results.append(conclusion("unread", LEVEL_UNREAD, f"{name}显卡读不到", snapshot["gpu_problem"]))
        totals = snapshot["totals"]
        if totals["slice_problem"]:
            results.append(conclusion("unread", LEVEL_UNREAD, f"{name} slice 读不到", totals["slice_problem"]))
        elif totals["slice_anon_shmem"] is None:
            results.append(conclusion("unread", LEVEL_UNREAD, f"{name} slice 读不到", "slice 的 memory.stat 读不了"))
        elif cross_check_differs(totals["usage_anon_shmem"], totals["slice_anon_shmem"]) and not broken("cross-check-skipped", environment):
            results.append(conclusion("cross-check-mismatch", LEVEL_PROBLEM, f"{name}：slice 现算与 cgroup 对不上",
                                      f"scope 里进程 {process_basis_text(totals)} 之和 {size_text(totals['usage_anon_shmem'], environment)}，"
                                      f"slice anon + shmem {size_text(totals['slice_anon_shmem'], environment)}",
                                      "slice 比进程合计大的那一截不归哪个活进程（tmpfs 文件、刚退出的、与 slice 外进程共享的页）；进程合计大的是有进程走了退路（共享页各算一份）或读取期间内存在涨"))
        usage, caps = totals["usage_bytes"], totals["caps_bytes"]
        figures = f"{name}现算合计 {size_text(usage, environment)}，在跑上限之和 {size_text(caps, environment)}（memory.max 现读）"
        if totals["caps_unread"] or (not usage and not caps):
            continue
        if usage > caps and not broken("usage-above-caps-unjudged", environment):
            results.append(conclusion("usage-above-caps", LEVEL_PROBLEM, f"{name}：现算合计超过在跑上限之和", figures,
                                      "scope 的 memory.max 管得住进程；先看有没有进程走了退路（共享页各算一份），再对着 slice anon + shmem 那一个数看"))
        elif caps:
            text = (f"{name}：现算合计没有超过在跑上限之和" if broken("caps-ratio-omitted", environment)
                    else f"{name}：现算合计占在跑上限之和 {usage / caps:.0%}，没有超过")
            results.append(conclusion("caps-ratio", LEVEL_FINE, text, f"{figures}；这个比例只作提示，不判上限记得对不对"))
    return results


def judge(view, environment):
    return judge_presence(view, environment) + judge_attempts(view, environment) + judge_hosts(view, environment)


def exit_code_of(view, conclusions, environment):
    if all(host["snapshot"] is None for host in view["hosts"]):
        return EXIT_FINE if broken("nothing-read-exit-zero", environment) else EXIT_NOTHING_READ
    levels = {item["level"] for item in conclusions}
    if LEVEL_PROBLEM in levels:
        return EXIT_PROBLEM
    if LEVEL_UNREAD in levels and not broken("unread-exit-zero", environment):
        return EXIT_UNREAD
    return EXIT_FINE


def top_names_text(attempt, environment):
    """按进程名分的前几项：名字 × 个数 占用，按占用从大到小；其余并成一项。"""
    names = list(attempt["names"].items())
    if not broken("top-names-unsorted", environment):
        names.sort(key=lambda pair: (-pair[1][0], pair[0]))
    pieces = [f"{name or '没有名字'} × {count} {size_text(total, environment)}" for name, (total, count) in names[:TOP_NAME_COUNT]]
    rest = names[TOP_NAME_COUNT:]
    if rest:
        pieces.append(f"其余 {len(rest)} 种 {size_text(sum(total for _, (total, _count) in rest), environment)}")
    return "；".join(pieces)


def attempt_lines(host_name, attempt, environment):
    cap, cap_state = attempt_cap(attempt)
    if cap_state == "limit":
        cap_text = f"上限 {size_text(cap, environment)}（memory.max 现读），占 {attempt['total_bytes'] / cap:.0%}" if cap else "上限是 0"
    elif cap_state == "unread":
        cap_text = "上限读不到"
    else:
        cap_text = "这台不在 scope 里，没有上限"
    cgroup_values = [group.get("cgroup_anon_shmem") for group in attempt["groups"]]
    cgroup_text = ("cgroup anon + shmem " + ("读不到" if None in cgroup_values and not broken("unread-shown-as-zero", environment)
                                              else size_text(sum(value or 0 for value in cgroup_values), environment))) if cgroup_values else "不在受管的 cgroup 里"
    if attempt["gpu_processes"]:
        gpu_text = f"显存 {attempt['gpu_mebibytes']} MiB（{attempt['gpu_processes']} 个进程" + (f"，{attempt['gpu_unread']} 个读不到）" if attempt["gpu_unread"] else "）")
    else:
        gpu_text = "显卡上没有它的进程"
    fallback_text = f"，其中 {attempt['rss_fallback_count']} 个读不到 smaps_rollup、按 RssAnon + RssShmem 算" if attempt["rss_fallback_count"] else ""
    return [f"任务 {task_number_of(attempt['label'])} 第 {attempt_number_of(attempt['label'])} 次 · {host_name}：合计 {size_text(attempt['total_bytes'], environment)}"
            f"（{attempt['process_count']} 个进程{fallback_text}）；{cap_text}；{cgroup_text}；{gpu_text}",
            f"    {top_names_text(attempt, environment)}"]


def host_lines(host, environment):
    snapshot = host["snapshot"]
    if snapshot is None:
        return [f"{host['name']}：读不到（{host['problem']}）"]
    totals = snapshot["totals"]
    pieces = [f"slice 总额 {size_text(totals['slice_limit'], environment)}", f"在跑上限之和 {size_text(totals['caps_bytes'], environment)}",
              f"现算合计 {size_text(totals['usage_bytes'], environment)}", f"slice anon + shmem {size_text(totals['slice_anon_shmem'], environment)}"]
    if snapshot["gpu_unattributed"]:
        pieces.append(f"显卡上有 {snapshot['gpu_unattributed']} 行进程号读不出，归不到任务")
    return [f"{host['name']}：{'、'.join(pieces)}（算了 {snapshot['rounds']} 轮）"]


def render_report(view, conclusions, environment):
    seconds = view.get("seconds") or 0
    lines = [f"── 按任务号现算内存（/proc/<进程号>/smaps_rollup 的 Pss_Anon + Pss_Shmem 加 status 的 VmPTE 逐进程加{f'，这 {seconds:g} 秒里最大的那一轮' if seconds else ''}）"]
    selected = selected_attempts(view, environment)
    for host_name, attempt in selected:
        lines += attempt_lines(host_name, attempt, environment)
    if not selected:
        lines.append("（没有归到任务的进程）")
    lines.append("── 每台")
    for host in view["hosts"]:
        lines += host_lines(host, environment)
    lines.append("── 结论")
    for item in conclusions:
        mark = {LEVEL_FINE: "✓", LEVEL_NOTICE: "·", LEVEL_PROBLEM: "✗", LEVEL_UNREAD: "⚠"}[item["level"]]
        lines.append(f"  {mark} {item['text']}。判据：{item['basis']}")
        if item["next_step"] and item["level"] in (LEVEL_PROBLEM, LEVEL_NOTICE):
            lines.append(f"     → 怎么办：{item['next_step']}")
    lines.append("结论：" + "；".join(item["text"] for item in conclusions))
    return lines


# ── 命令行 ──
class UsageError(ValueError):
    """用法错：str 是原因。"""


def parse_arguments(arguments, environment):
    """→ {requested, seconds, interval}；写错抛 UsageError。"""
    options = {"requested": [], "seconds": 0.0, "interval": DEFAULT_INTERVAL_SECONDS}
    position = 0
    while position < len(arguments):
        word = arguments[position]
        if word in ("--watch", "--interval"):
            if position + 1 >= len(arguments):
                raise UsageError(f"{word} 后面要给秒数")
            try:
                seconds = float(arguments[position + 1])
            except ValueError as error:
                raise UsageError(f"{word} 后面要给秒数，收到「{arguments[position + 1]}」") from error
            if seconds < 0 or (word == "--interval" and seconds == 0):
                raise UsageError(f"{word} 的秒数不对：{seconds:g}")
            options["seconds" if word == "--watch" else "interval"] = seconds
            position += 2
        elif word.isdigit() and int(word) > 0:
            options["requested"].append(int(word))
            position += 1
        elif broken("task-number-unchecked", environment) and not word.startswith("--"):
            options["requested"].append(0)
            position += 1
        else:
            raise UsageError(f"认不出「{word}」：任务号要是正整数")
    return options


def run_once(options, environment, reader=read_every_host):
    """读两台、判、打，交退出码。"""
    view = {"hosts": reader(options["seconds"], options["interval"], environment), "requested": options["requested"], "seconds": options["seconds"]}
    conclusions = judge(view, environment)
    for line in render_report(view, conclusions, environment):
        print(line)
    code = exit_code_of(view, conclusions, environment)
    if code == EXIT_NOTHING_READ:
        print("  ✗ 两台都读不到，一条结论都下不了")
        print("  → 怎么办：照上面每台那一行的原因修（/proc 读不了、多机配置判不过、第二台连不上）再跑")
    return code


# ── 自证：造出来的 /proc 与 cgroup 目录树 ──
class FakeHost:
    """临时目录里的一台：process/ 是 /proc，cgroup/ 是 /sys/fs/cgroup，nvidia-smi 是打固定几行的脚本。"""
    SLICE_PATH = "/user.slice/singlefs.slice/singlefs-heavy.slice"

    def __init__(self, directory, compute_lines=()):
        self.process_root = os.path.join(directory, "process")
        self.cgroup_root = os.path.join(directory, "cgroup")
        self.nvidia_smi = os.path.join(directory, "nvidia-smi")
        os.makedirs(self.process_root)
        with open(self.nvidia_smi, "w", encoding="utf-8") as handle:
            handle.write("#!/bin/sh\n" + "".join(f"echo '{line}'\n" for line in compute_lines))
        os.chmod(self.nvidia_smi, 0o755)
        self.add_group(self.SLICE_PATH, anon=0, shmem=0, page_cache=0, limit="53687091200")

    def add_group(self, path, anon, shmem, page_cache, limit):
        directory = self.cgroup_root + path
        os.makedirs(directory, exist_ok=True)
        with open(os.path.join(directory, "memory.stat"), "w", encoding="utf-8") as handle:
            handle.write(f"anon {anon}\nfile {page_cache}\nshmem {shmem}\n")
        with open(os.path.join(directory, "memory.current"), "w", encoding="utf-8") as handle:
            handle.write(f"{anon + shmem + page_cache}\n")
        with open(os.path.join(directory, "memory.peak"), "w", encoding="utf-8") as handle:
            handle.write(f"{(anon + shmem + page_cache) * 2}\n")
        with open(os.path.join(directory, "memory.max"), "w", encoding="utf-8") as handle:
            handle.write(f"{limit}\n")

    def add_process(self, process_id, parent, name, anon_kibibytes, shmem_kibibytes, page_table_kibibytes, cgroup_path, process_environment=(),
                    proportional_kibibytes=None, smaps_rollup_readable=True):
        """anon_kibibytes、shmem_kibibytes 是 status 的 RssAnon、RssShmem；smaps_rollup 的 Pss_Anon、Pss_Shmem 默认与它们相同（没有共享页），
        proportional_kibibytes=(anon, shmem) 另给；smaps_rollup_readable 为假就不造这个文件。"""
        directory = os.path.join(self.process_root, str(process_id))
        os.makedirs(directory, exist_ok=True)
        if smaps_rollup_readable:
            pss_anon, pss_shmem = proportional_kibibytes if proportional_kibibytes is not None else (anon_kibibytes, shmem_kibibytes)
            with open(os.path.join(directory, "smaps_rollup"), "w", encoding="utf-8") as handle:
                handle.write("00400000-ffffffffff601000 ---p 00000000 00:00 0                          [rollup]\n"   # clock-times:allow 自证造的 smaps_rollup 首行，设备号一栏照 /proc 的写法，不是钟点
                             f"Rss:            {anon_kibibytes + shmem_kibibytes} kB\nPss:            {pss_anon + pss_shmem} kB\n"
                             f"Pss_Anon:       {pss_anon} kB\nPss_File:       0 kB\nPss_Shmem:      {pss_shmem} kB\n")
        with open(os.path.join(directory, "status"), "w", encoding="utf-8") as handle:
            handle.write(f"Name:\t{name}\nPPid:\t{parent}\nRssAnon:\t{anon_kibibytes} kB\nRssShmem:\t{shmem_kibibytes} kB\nVmPTE:\t{page_table_kibibytes} kB\n")
        with open(os.path.join(directory, "cgroup"), "w", encoding="utf-8") as handle:
            handle.write(f"0::{cgroup_path}\n")
        with open(os.path.join(directory, "environ"), "wb") as handle:
            handle.write(b"\0".join(entry.encode("ascii") for entry in process_environment) + b"\0")

    def environment(self):
        return {"QUEUE_HEALTH_PROCESS_ROOT": self.process_root, "QUEUE_HEALTH_CGROUP_ROOT": self.cgroup_root, "QUEUE_HEALTH_SLICE_DIRECTORY": self.cgroup_root + self.SLICE_PATH,
                "QUEUE_HEALTH_NVIDIA_SMI": self.nvidia_smi, BREAK_VARIABLE: os.environ.get(BREAK_VARIABLE, "")}


SAMPLE_LABEL = "task-abc123-501-2"
SAMPLE_SCOPE = FakeHost.SLICE_PATH + "/singlefs-memory-cap-100-200.scope"
SAMPLE_UNIT = f"/user.slice/app.slice/singlefs-{SAMPLE_LABEL}.service"
SAMPLE_CAP_BYTES = 4096 * MEBIBYTE
SAMPLE_TOTAL_BYTES = 1852 * MEBIBYTE    # 1 + 1020 + 4 + 500 + 300（scope 里，4 是 VmPTE）+ 20（尝试单元）+ 7（setsid 出去的）
SAMPLE_SCOPE_ANON_SHMEM = 1821 * MEBIBYTE
SAMPLE_PROCESS_COUNT = 6
SAMPLE_PAGE_CACHE_BYTES = 40960 * MEBIBYTE
SAMPLE_UNRELATED_PROCESS = 2000


def mebibytes_in_kibibytes(mebibytes):
    return mebibytes * 1024


def sample_host(directory, compute_lines=()):
    """一条任务（第 2 次尝试）的整棵树：尝试单元里的外壳；scope 里只带 builds 路径的 bash、带标签的两个 python3、什么都不带的 worker；setsid 出去又清了环境的后代。
    另有一个不相干的大进程。cgroup 里页缓存远大于进程内存。"""
    host = FakeHost(directory, compute_lines)
    host.add_group(SAMPLE_UNIT, anon=20 * MEBIBYTE, shmem=0, page_cache=0, limit="max")
    host.add_group(SAMPLE_SCOPE, anon=1801 * MEBIBYTE, shmem=20 * MEBIBYTE, page_cache=SAMPLE_PAGE_CACHE_BYTES, limit=str(SAMPLE_CAP_BYTES))
    host.add_group(FakeHost.SLICE_PATH, anon=1801 * MEBIBYTE, shmem=20 * MEBIBYTE, page_cache=SAMPLE_PAGE_CACHE_BYTES, limit="53687091200")
    labelled = [f"TASK_SERVER_ATTEMPT_LABEL={SAMPLE_LABEL}", "LANG=C"]
    host.add_process(900, 1, "python3", mebibytes_in_kibibytes(20), 0, 0, SAMPLE_UNIT)
    host.add_process(1000, 1, "bash", mebibytes_in_kibibytes(1), 0, 0, SAMPLE_SCOPE, [f"CARGO_TARGET_DIR=/build-root/builds/{SAMPLE_LABEL}/target"])
    host.add_process(1001, 1000, "python3", mebibytes_in_kibibytes(1000), mebibytes_in_kibibytes(20), mebibytes_in_kibibytes(4), SAMPLE_SCOPE, labelled)
    host.add_process(1002, 1001, "python3", mebibytes_in_kibibytes(500), 0, 0, SAMPLE_SCOPE, labelled)
    host.add_process(1003, 1, "worker", mebibytes_in_kibibytes(300), 0, 0, SAMPLE_SCOPE)
    host.add_process(1010, 1002, "escaped", mebibytes_in_kibibytes(7), 0, 0, "/user.slice/escape.scope")
    host.add_process(SAMPLE_UNRELATED_PROCESS, 1, "unrelated", mebibytes_in_kibibytes(5120), 0, 0, "/user.slice/other.scope")
    return host


SHARED_LABEL = "task-abc123-801-1"
SHARED_SCOPE = FakeHost.SLICE_PATH + "/singlefs-memory-cap-700-800.scope"
SHARED_PROPORTIONAL_BYTES = 1040 * MEBIBYTE   # Pss：anon 400 + 3 × 200，shmem 4 × 10
SHARED_RSS_BYTES = 3860 * MEBIBYTE            # Rss：anon 1000 + 3 × 900，shmem 4 × 40
SHARED_CGROUP_ANON_SHMEM = 1044 * MEBIBYTE    # 与 Pss 之和差 4 MiB，在容差以内
FALLBACK_LABEL = "task-abc123-901-1"
FALLBACK_SCOPE = FakeHost.SLICE_PATH + "/singlefs-memory-cap-900-1000.scope"
FALLBACK_TOTAL_BYTES = 800 * MEBIBYTE         # smaps_rollup 读得到的 500 + 读不到的退回 RssAnon 300


def shared_pages_host(directory):
    """fork 出来的一棵树：一个父进程和三个子进程，写时复制页与共享内存在每个进程的 Rss 里各算一份，按比例分摊后整棵树加起来 1040 MiB，cgroup 里 1044 MiB。"""
    host = FakeHost(directory)
    host.add_group(SHARED_SCOPE, anon=1004 * MEBIBYTE, shmem=40 * MEBIBYTE, page_cache=0, limit=str(SAMPLE_CAP_BYTES))
    host.add_group(FakeHost.SLICE_PATH, anon=1004 * MEBIBYTE, shmem=40 * MEBIBYTE, page_cache=0, limit="53687091200")
    labelled = [f"TASK_SERVER_ATTEMPT_LABEL={SHARED_LABEL}"]
    host.add_process(5000, 1, "python3", mebibytes_in_kibibytes(1000), mebibytes_in_kibibytes(40), 0, SHARED_SCOPE, labelled,
                     proportional_kibibytes=(mebibytes_in_kibibytes(400), mebibytes_in_kibibytes(10)))
    for process_id in (5001, 5002, 5003):
        host.add_process(process_id, 5000, "python3", mebibytes_in_kibibytes(900), mebibytes_in_kibibytes(40), 0, SHARED_SCOPE, labelled,
                         proportional_kibibytes=(mebibytes_in_kibibytes(200), mebibytes_in_kibibytes(10)))
    return host


def fallback_host(directory):
    """读得到 smaps_rollup 的 python3 500 MiB、读不到的 worker 300 MiB（退回 RssAnon）、读不到也没有占用的内核线程；cgroup 里 800 MiB。"""
    host = FakeHost(directory)
    host.add_group(FALLBACK_SCOPE, anon=800 * MEBIBYTE, shmem=0, page_cache=0, limit=str(SAMPLE_CAP_BYTES))
    host.add_group(FakeHost.SLICE_PATH, anon=800 * MEBIBYTE, shmem=0, page_cache=0, limit="53687091200")
    labelled = [f"TASK_SERVER_ATTEMPT_LABEL={FALLBACK_LABEL}"]
    host.add_process(6000, 1, "python3", mebibytes_in_kibibytes(500), 0, 0, FALLBACK_SCOPE, labelled)
    host.add_process(6001, 6000, "worker", mebibytes_in_kibibytes(300), 0, 0, FALLBACK_SCOPE, labelled, smaps_rollup_readable=False)
    host.add_process(6002, 2, "kworker", 0, 0, 0, FALLBACK_SCOPE, labelled, smaps_rollup_readable=False)
    return host


def read_sample(host):
    return read_host_snapshot(peaks, host.environment())


def sample_attempt(label=SAMPLE_LABEL, total=2 * 1024 * MEBIBYTE, process_anon_shmem=2 * 1024 * MEBIBYTE, cgroup_anon_shmem=2 * 1024 * MEBIBYTE, cap=4096 * MEBIBYTE, limit_state="limit"):
    group = {"path": SAMPLE_SCOPE, "process_count": 3, "process_anon_shmem": process_anon_shmem, "process_total": total, "cgroup_anon_shmem": cgroup_anon_shmem,
             "limit_bytes": cap, "limit_state": limit_state, "in_slice": True, "rss_fallback_count": 0}
    return {"label": label, "process_count": 3, "total_bytes": total, "names": {"python3": [total, 3]}, "groups": [group], "gpu_mebibytes": 0, "gpu_processes": 0, "gpu_unread": 0,
            "rss_fallback_count": 0}


def sample_snapshot(attempts=(), usage=2 * 1024 * MEBIBYTE, caps=3 * 1024 * MEBIBYTE, slice_anon_shmem=None):
    return {"attempts": {attempt["label"]: attempt for attempt in attempts},
            "totals": {"usage_bytes": usage, "usage_anon_shmem": usage, "rss_fallback_count": 0, "caps_bytes": caps, "caps_unread": 0, "slice_limit": 50 * 1024 * MEBIBYTE,
                       "slice_anon_shmem": usage if slice_anon_shmem is None else slice_anon_shmem, "slice_problem": ""},
            "gpu_problem": "", "gpu_unattributed": 0, "rounds": 1}


def sample_view(this_host=None, peer_host=None, requested=(), peer_unread=False):
    """两台的读数；peer_unread 为真时第二台读不到。"""
    idle = sample_snapshot(usage=0, caps=0)
    return {"hosts": [{"name": THIS_HOST_NAME, "snapshot": this_host if this_host is not None else idle, "problem": ""},
                      {"name": PEER_HOST_NAME, "snapshot": None if peer_unread else (peer_host if peer_host is not None else idle), "problem": "第二台读不到（连不上）" if peer_unread else ""}],
            "requested": list(requested), "seconds": 0}


# 每种结论的判法格：(格名, 造读数, 必须有的结论, 不许有的结论)。结论名写在 judge_presence、judge_attempts、judge_hosts 里
JUDGE_CELLS = (
    ("no-process-reported", lambda: sample_view(requested=[777]), {"no-process"}, {"unread"}),
    ("found-task-is-not-no-process", lambda: sample_view(sample_snapshot([sample_attempt()]), requested=[501]), set(), {"no-process"}),
    ("no-process-with-unread-host", lambda: sample_view(requested=[777], peer_unread=True), {"unread"}, {"no-process"}),
    ("no-running-green", lambda: sample_view(), {"no-running"}, {"no-process", "unread"}),
    ("no-running-needs-every-host", lambda: sample_view(peer_unread=True), {"unread"}, {"no-running"}),
    ("cross-check-matches-green", lambda: sample_view(sample_snapshot([sample_attempt()])), {"cross-check-matches"}, {"cross-check-mismatch"}),
    ("cross-check-mismatch-red", lambda: sample_view(sample_snapshot([sample_attempt(cgroup_anon_shmem=512 * MEBIBYTE)])), {"cross-check-mismatch"}, {"cross-check-matches"}),
    ("cross-check-unread-red", lambda: sample_view(sample_snapshot([sample_attempt(cgroup_anon_shmem=None)])), {"unread"}, {"cross-check-mismatch"}),
    ("within-cap-green", lambda: sample_view(sample_snapshot([sample_attempt()])), {"within-cap"}, {"near-cap"}),
    ("near-cap-red", lambda: sample_view(sample_snapshot([sample_attempt(total=3900 * MEBIBYTE, process_anon_shmem=3900 * MEBIBYTE, cgroup_anon_shmem=3900 * MEBIBYTE)],
                                                         usage=3900 * MEBIBYTE, caps=4096 * MEBIBYTE)), {"near-cap"}, {"within-cap"}),
    ("caps-ratio-green", lambda: sample_view(sample_snapshot([sample_attempt()])), {"caps-ratio"}, {"usage-above-caps"}),
    ("caps-ratio-low-green", lambda: sample_view(sample_snapshot([sample_attempt()], caps=40 * 1024 * MEBIBYTE)), {"caps-ratio"}, {"usage-above-caps"}),
    ("usage-above-caps-red", lambda: sample_view(sample_snapshot([sample_attempt()], caps=1024 * MEBIBYTE)), {"usage-above-caps"}, {"caps-ratio"}),
)


class FakePeer:
    """把要在第二台跑的命令文本放在这一台的 bash 里跑（环境是造出来的那一台），不连 ssh。"""
    available, why = True, ""

    def __init__(self, environment):
        self.environment = environment

    def run(self, text, timeout):
        completed = subprocess.run(["bash", "-c", text], capture_output=True, text=True, env=dict(os.environ, **self.environment), timeout=timeout, stdin=subprocess.DEVNULL)
        return completed.returncode, completed.stdout, completed.stderr


class UnreachablePeer:
    available, why = True, ""

    def run(self, _text, timeout):
        return 255, "", f"第二台 {timeout} 秒没回话"


class UnavailablePeer:
    available, why = False, "多机配置里双机开关 ENABLE_ACROSS_MACHINES 没开"


def selftest_cells():
    """全部格：→ 红了几格、一共几格。每格自己造、自己删，互不依赖。"""
    failures, checked = [], 0
    environment = os.environ

    def verify(cell, passed, detail):
        nonlocal checked
        checked += 1
        if not passed:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def scratch_cell():
        return tempfile.TemporaryDirectory(prefix="queue-health-selftest-")

    def exit_code_for(view):
        return exit_code_of(view, judge(view, environment), environment)

    for cell, view_of, required, banned in JUDGE_CELLS:
        kinds = {item["kind"] for item in judge(view_of(), environment)}
        verify(cell, required <= kinds and not banned & kinds, f"要有 {sorted(required)}、不许有 {sorted(banned)}，判出 {sorted(kinds)}")

    with scratch_cell() as directory:
        attempts = read_sample(sample_host(directory))["attempts"]
        count = attempts.get(SAMPLE_LABEL, {}).get("process_count")
        verify("whole-tree-found", count == SAMPLE_PROCESS_COUNT, f"整棵树要找到 {SAMPLE_PROCESS_COUNT} 个进程，找到 {count} 个；标签 {sorted(attempts)}")
    with scratch_cell() as directory:
        attempts = read_sample(sample_host(directory))["attempts"]
        total = attempts.get(SAMPLE_LABEL, {}).get("total_bytes")
        verify("process-total-sums-status-fields", total == SAMPLE_TOTAL_BYTES, f"Pss_Anon + Pss_Shmem + VmPTE 逐进程加要得 {SAMPLE_TOTAL_BYTES}，得 {total}")
    with scratch_cell() as directory:
        host = FakeHost(directory)
        scope = FakeHost.SLICE_PATH + "/singlefs-memory-cap-300-400.scope"
        host.add_group(scope, anon=MEBIBYTE, shmem=0, page_cache=0, limit="max")
        host.add_process(3000, 1, "bash", mebibytes_in_kibibytes(1), 0, 0, scope, ["MULTI_HOST_RUN_PEER_SCRATCH=/build-root/builds/task-abc123-601-1"])
        attempts = read_sample(host)["attempts"]
        verify("builds-directory-names-attempt", "task-abc123-601-1" in attempts, f"环境里指到 builds/<尝试标签> 的要归上，标签 {sorted(attempts)}")
    with scratch_cell() as directory:
        snapshot = read_sample(sample_host(directory))
        names = sorted(name for attempt in snapshot["attempts"].values() for name in attempt["names"])
        verify("unrelated-process-excluded", "unrelated" not in names, f"不相干的进程算进了任务：{names}")
    with scratch_cell() as directory:
        snapshot = read_sample(sample_host(directory))
        groups = {group["path"]: group for attempt in snapshot["attempts"].values() for group in attempt["groups"]}
        stat_bytes = groups.get(SAMPLE_SCOPE, {}).get("cgroup_anon_shmem")
        verify("page-cache-not-counted", stat_bytes == SAMPLE_SCOPE_ANON_SHMEM and snapshot["totals"]["slice_anon_shmem"] == SAMPLE_SCOPE_ANON_SHMEM,
               f"cgroup 占用要是 anon 加 shmem {SAMPLE_SCOPE_ANON_SHMEM}（页缓存 {SAMPLE_PAGE_CACHE_BYTES} 不算），scope 读到 {stat_bytes}、slice 读到 {snapshot['totals']['slice_anon_shmem']}")
    with scratch_cell() as directory:
        snapshot = read_sample(sample_host(directory))
        attempt = snapshot["attempts"].get(SAMPLE_LABEL)
        cap = attempt_cap(attempt) if attempt else None
        verify("cap-read-from-memory-max", cap == (SAMPLE_CAP_BYTES, "limit") and snapshot["totals"]["caps_bytes"] == SAMPLE_CAP_BYTES,
               f"上限要从 scope 的 memory.max 读到 {SAMPLE_CAP_BYTES}，读到 {cap}，在跑上限之和 {snapshot['totals']['caps_bytes']}")
    with scratch_cell() as directory:
        host = sample_host(directory, ["1001, 1234", "1002, [N/A]", f"{SAMPLE_UNRELATED_PROCESS}, 9999", "[N/A], 5"])
        snapshot = read_sample(host)
        attempt = snapshot["attempts"].get(SAMPLE_LABEL, {})
        readings = (attempt.get("gpu_mebibytes"), attempt.get("gpu_processes"), attempt.get("gpu_unread"), snapshot["gpu_unattributed"])
        verify("gpu-by-process-read", readings == (1234, 2, 1, 1), f"按进程号要归到 1234 MiB、2 个进程、1 个读不到、1 行归不上，读到 {readings}")
    with scratch_cell() as directory:
        host = FakeHost(directory)
        scope = FakeHost.SLICE_PATH + "/singlefs-memory-cap-500-600.scope"
        host.add_group(scope, anon=100 * MEBIBYTE, shmem=0, page_cache=0, limit=str(SAMPLE_CAP_BYTES))
        label_entry = ["TASK_SERVER_ATTEMPT_LABEL=task-abc123-701-1"]
        host.add_process(4000, 1, "python3", mebibytes_in_kibibytes(100), 0, 0, scope, label_entry)
        moments = [0.0]
        later_mebibytes = [900, 200]

        def advance(interval):
            moments[0] += interval
            host.add_process(4000, 1, "python3", mebibytes_in_kibibytes(later_mebibytes.pop(0)), 0, 0, scope, label_entry)

        report = watch_host(peaks, host.environment(), 2.0, 1.0, lambda: moments[0], advance)
        total = report["attempts"].get("task-abc123-701-1", {}).get("total_bytes")
        verify("watch-keeps-maximum", total == 900 * MEBIBYTE and report["rounds"] == 3 and report["totals"]["usage_bytes"] == 900 * MEBIBYTE,
               f"三轮 100、900、200 MiB 要留 900 MiB，留了 {total}，轮数 {report['rounds']}，这台合计 {report['totals']['usage_bytes']}")

    with scratch_cell() as directory:
        host = sample_host(directory)
        report, problem = read_peer_host(FakePeer(host.environment()), 0.0, 1.0, environment)
        verify("peer-probe-program-runs", bool(report) and SAMPLE_LABEL in report["attempts"] and report["rounds"] == 1, f"探测程序跑完要带回这条任务：原因「{problem}」，读到 {sorted((report or {}).get('attempts', {}))}")
    report_unreachable, why_unreachable = read_peer_host(UnreachablePeer(), 0.0, 1.0, environment)
    report_unavailable, why_unavailable = read_peer_host(UnavailablePeer(), 0.0, 1.0, environment)
    verify("peer-failure-unread", report_unreachable is None and "读不到" in why_unreachable and report_unavailable is None and "读不到" in why_unavailable,
           f"连不上与配置没开都要报读不到，读到 {report_unreachable} 「{why_unreachable}」、{report_unavailable} 「{why_unavailable}」")

    with scratch_cell() as directory:
        snapshot = read_sample(shared_pages_host(directory))
        attempt = snapshot["attempts"].get(SHARED_LABEL, {})
        total = attempt.get("total_bytes")
        verify("shared-pages-split-proportionally", total == SHARED_PROPORTIONAL_BYTES and attempt.get("process_count") == 4,
               f"共享页多的一棵树要按 Pss 加出 {SHARED_PROPORTIONAL_BYTES}（Rss 之和是 {SHARED_RSS_BYTES}），4 个进程，得 {total}、{attempt.get('process_count')} 个进程")
        view = sample_view(snapshot)
        kinds = {item["kind"] for item in judge(view, environment)}
        verify("shared-pages-not-reported-as-mismatch", "cross-check-matches" in kinds and "cross-check-mismatch" not in kinds and "rss-fallback" not in kinds,
               f"Pss 之和 {SHARED_PROPORTIONAL_BYTES} 对 cgroup {SHARED_CGROUP_ANON_SHMEM}，差在容差以内，要对得上、不报对不上、不报走了退路，判出 {sorted(kinds)}")
    with scratch_cell() as directory:
        snapshot = read_sample(fallback_host(directory))
        attempt = snapshot["attempts"].get(FALLBACK_LABEL, {})
        verify("rss-fallback-reads-status", attempt.get("total_bytes") == FALLBACK_TOTAL_BYTES,
               f"smaps_rollup 读不到的进程要退回 RssAnon + RssShmem，合计要得 {FALLBACK_TOTAL_BYTES}，得 {attempt.get('total_bytes')}")
        view = sample_view(snapshot)
        found = [item for item in judge(view, environment) if item["kind"] == "rss-fallback"]
        shown = "\n".join(render_report(view, judge(view, environment), environment))
        verify("rss-fallback-flagged", attempt.get("rss_fallback_count") == 1 and len(found) == 1 and "1 个读不到 smaps_rollup" in shown,
               f"走退路的只有 worker 一个（没有占用的内核线程不算），要记 1 个、报 1 条 rss-fallback 并写进行里；记 {attempt.get('rss_fallback_count')} 个、报 {len(found)} 条：{shown}")
    ratio_view = sample_view(sample_snapshot([sample_attempt()]))
    ratio_items = [item for item in judge(ratio_view, environment) if item["kind"] == "caps-ratio"]
    low_view = sample_view(sample_snapshot([sample_attempt()], caps=40 * 1024 * MEBIBYTE))
    low_items = [item for item in judge(low_view, environment) if item["kind"] == "caps-ratio"]
    verify("caps-ratio-in-text", len(ratio_items) == 1 and "占在跑上限之和 67%" in ratio_items[0]["text"] and len(low_items) == 1 and "占在跑上限之和 5%" in low_items[0]["text"]
           and low_items[0]["level"] == LEVEL_FINE and "相近" not in ratio_items[0]["text"] + low_items[0]["text"],
           f"2 GiB 对 3 GiB 要写占 67%、对 40 GiB 要写占 5% 且不作问题记、不写「相近」：{[item['text'] for item in ratio_items + low_items]}")

    code = exit_code_for(sample_view(sample_snapshot([sample_attempt()])))
    verify("exit-zero-clean", code == EXIT_FINE, f"都对得上要退 {EXIT_FINE}，退 {code}")
    code = exit_code_for(sample_view(requested=[777]))
    verify("exit-one-problem", code == EXIT_PROBLEM, f"此刻找不到进程要退 {EXIT_PROBLEM}，退 {code}")
    code = exit_code_for(sample_view(sample_snapshot([sample_attempt()]), peer_unread=True))
    verify("exit-three-unread", code == EXIT_UNREAD, f"第二台读不到要退 {EXIT_UNREAD}，退 {code}")
    nothing_read = sample_view(peer_unread=True)
    nothing_read["hosts"][0].update(snapshot=None, problem="本机读不到（OSError）")
    code = exit_code_for(nothing_read)
    verify("exit-seventy-eight-nothing-read", code == EXIT_NOTHING_READ, f"两台都读不到要退 {EXIT_NOTHING_READ}，退 {code}")

    unread_snapshot = sample_snapshot([sample_attempt(cgroup_anon_shmem=None)])
    unread_snapshot["totals"]["slice_limit"] = None
    view = sample_view(unread_snapshot, peer_unread=True)
    text = "\n".join(render_report(view, judge(view, environment), environment))
    verify("unread-printed-not-zero", "读不到" in text and "0 MiB" not in text, f"读不到的要写「读不到」、不写 0：{text}")

    many_names = sample_attempt()
    many_names["names"] = {f"name-{index}": [(index + 1) * MEBIBYTE, 1] for index in range(TOP_NAME_COUNT + 2)}
    names_text = top_names_text(many_names, environment)
    verify("top-names-sorted", names_text.startswith(f"name-{TOP_NAME_COUNT + 1} ") and "其余 2 种" in names_text, f"要从最大的那一种起、其余并成一项：{names_text}")

    other_attempt = sample_attempt(label="task-abc123-502-1")
    view = sample_view(sample_snapshot([sample_attempt(), other_attempt]), requested=[501])
    shown = {attempt["label"] for _name, attempt in selected_attempts(view, environment)}
    verify("requested-number-filters", shown == {SAMPLE_LABEL}, f"点了 501 只打 501，打了 {sorted(shown)}")
    view = sample_view(sample_snapshot([sample_attempt(), other_attempt]))
    shown = {attempt["label"] for _name, attempt in selected_attempts(view, environment)}
    verify("no-number-selects-every-task", shown == {SAMPLE_LABEL, "task-abc123-502-1"}, f"不给号打全部，打了 {sorted(shown)}")

    refused = []
    for arguments in (["abc"], ["--no-such"], ["--watch"], ["--watch", "abc"], ["--interval", "0"], ["-3"]):
        try:
            parse_arguments(arguments, environment)
        except UsageError:
            refused.append(arguments)
    verify("usage-refused", len(refused) == 6, f"六种写法都要判用法错，判了 {refused}")
    options = parse_arguments(["501", "502", "--watch", "30", "--interval", "0.5"], environment)
    verify("usage-accepted", options == {"requested": [501, 502], "seconds": 30.0, "interval": 0.5}, f"读到 {options}")

    outside = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "501"], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60,
                             env=dict(os.environ, **{BREAK_VARIABLE: "watch-keeps-latest"}))
    verify("break-refused-outside-selftest", outside.returncode == EXIT_USAGE and "只在自证里认" in outside.stderr, f"退 {outside.returncode}：{outside.stderr.strip()}")

    task_server_forms = (ATTEMPT_UNIT_SOURCE_FORM, ATTEMPT_LABEL_SOURCE_FORM)
    unmatched_suffix = "不在任何源码里" if broken("source-form-unmatched", environment) else ""
    task_server_files = entry_with_split_family(os.path.join(REPOSITORY_ROOT, "task-server", "task_server.py"), environment)
    forms_read = forms_found_in_files(task_server_files, task_server_forms, unmatched_suffix)
    forms_read.update(forms_found_in_files([os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "run-with-memory-cap.sh")], (SCOPE_SOURCE_FORM,), unmatched_suffix))
    verify("attempt-forms-match-sources", all(forms_read.values()), f"尝试单元、尝试标签与 scope 的写法在源码里要找得到：{forms_read}")

    def forms_found_in_sample_family(files):
        """在一个临时目录里写好 files（[(文件名, 文本)]），读 fake_entry.py 的拆分族，核两种写法找不找得到。"""
        with scratch_cell() as directory:
            for name, text in files:
                with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
                    handle.write(text)
            return forms_found_in_files(entry_with_split_family(os.path.join(directory, "fake_entry.py"), environment), task_server_forms)

    both_forms = f'unit = f"{ATTEMPT_UNIT_SOURCE_FORM}"\nlabel = f"{ATTEMPT_LABEL_SOURCE_FORM}"\n'
    found_in_split_family = forms_found_in_sample_family((
        ("fake_entry.py", "import os\nimport fake_entry_unit\nfrom fake_entry_label import Piece\n"),
        ("fake_entry_unit.py", f'unit = f"{ATTEMPT_UNIT_SOURCE_FORM}"\n'),
        ("fake_entry_label.py", "import fake_entry_label_deeper\n"),
        ("fake_entry_label_deeper.py", f'label = f"{ATTEMPT_LABEL_SOURCE_FORM}"\n')))
    found_in_unrelated = forms_found_in_sample_family((("fake_entry.py", "import os\n"), ("fake_unrelated_module.py", both_forms)))
    found_in_name_like_unloaded = forms_found_in_sample_family((("fake_entry.py", "import os\n"), ("fake_entry_shadow.py", both_forms)))
    found_in_loaded_shared = forms_found_in_sample_family((("fake_entry.py", "import os\nimport fake_shared_checks\n"), ("fake_shared_checks.py", both_forms)))
    found_in_comment_named = forms_found_in_sample_family((
        ("fake_entry.py", '# 别处留下的 "fake_entry_shadow.py" 不归这一份\n"""用法说明：\n    import fake_entry_shadow   # 别这样写\n"""\nimport os\n'),
        ("fake_entry_shadow.py", both_forms)))
    found_in_import_module = forms_found_in_sample_family((
        ("fake_entry.py", 'import importlib\nfake_entry_part = importlib.import_module("fake_entry_part")\n'),
        ("fake_entry_part.py", both_forms)))
    verify("attempt-forms-found-in-imported-modules", all(found_in_split_family.values()),
           f"两种写法只在入口 import 的、名字像部分文件的同目录模块里（label 那份是入口 import 的那份再 import 的），要照旧找得到：{found_in_split_family}")
    verify("attempt-forms-skip-unimported-modules", not any(found_in_unrelated.values()),
           f"入口没 import、名字也不像部分文件的同目录模块里有这两种写法，不许算找到（不按通配）：{found_in_unrelated}")
    verify("attempt-forms-skip-name-like-unloaded-modules", not any(found_in_name_like_unloaded.values()),
           f"名字像部分文件（fake_entry_shadow.py）而入口没 import 的同目录模块里有这两种写法，不许算找到：{found_in_name_like_unloaded}")
    verify("attempt-forms-skip-loaded-shared-modules", not any(found_in_loaded_shared.values()),
           f"入口 import 的共用库（fake_shared_checks.py，名字不像它拆出的部分文件）里有这两种写法，不许算找到：{found_in_loaded_shared}")
    verify("attempt-forms-skip-comment-named-modules", not any(found_in_comment_named.values()),
           f"入口只在注释与文档串里点名 fake_entry_shadow.py、写 import fake_entry_shadow（没有载入它）：那份里有这两种写法，不许算找到：{found_in_comment_named}")
    verify("attempt-forms-found-in-import-module-modules", all(found_in_import_module.values()),
           f"入口按字面量参数的 importlib.import_module 载入的部分文件（fake_entry_part.py）里有这两种写法，要找得到：{found_in_import_module}")
    return failures, checked


# 弄坏开关点名的格（文件头那张表；开关轮照它逐条比，多红少红都算错）
SWITCH_EXPECTED_CELLS = {
    "scope-members-unlabeled": {"whole-tree-found", "process-total-sums-status-fields"},
    "descendants-unfollowed": {"whole-tree-found", "process-total-sums-status-fields"},
    "unit-cgroup-unread": {"whole-tree-found", "process-total-sums-status-fields"},
    "builds-directory-unread": {"builds-directory-names-attempt"},
    "every-process-labeled": {"whole-tree-found", "process-total-sums-status-fields", "unrelated-process-excluded", "gpu-by-process-read"},
    "page-table-dropped": {"process-total-sums-status-fields"},
    "pss-read-as-rss": {"shared-pages-split-proportionally", "shared-pages-not-reported-as-mismatch"},
    "rss-fallback-dropped": {"rss-fallback-reads-status"},
    "rss-fallback-unflagged": {"rss-fallback-flagged"},
    "page-cache-counted": {"page-cache-not-counted"},
    "memory-max-read-as-unlimited": {"cap-read-from-memory-max"},
    "gpu-unread-as-zero": {"gpu-by-process-read"},
    "gpu-other-process-counted": {"gpu-by-process-read"},
    "watch-keeps-latest": {"watch-keeps-maximum"},
    "unread-host-as-absent": {"no-process-with-unread-host"},
    "cross-check-skipped": {"cross-check-mismatch-red"},
    "unread-cgroup-as-zero": {"cross-check-unread-red", "unread-printed-not-zero"},
    "near-cap-unjudged": {"near-cap-red"},
    "caps-ratio-omitted": {"caps-ratio-in-text"},
    "usage-above-caps-unjudged": {"usage-above-caps-red"},
    "no-running-despite-unread": {"no-running-needs-every-host"},
    "unread-exit-zero": {"exit-three-unread"},
    "nothing-read-exit-zero": {"exit-seventy-eight-nothing-read"},
    "unread-shown-as-zero": {"unread-printed-not-zero"},
    "top-names-unsorted": {"top-names-sorted"},
    "requested-filter-ignored": {"requested-number-filters"},
    "peer-failure-read-as-empty": {"peer-failure-unread"},
    "task-number-unchecked": {"usage-refused"},
    "source-form-unmatched": {"attempt-forms-match-sources"},
    "imported-modules-unread": {"attempt-forms-match-sources", "attempt-forms-found-in-imported-modules", "attempt-forms-found-in-import-module-modules"},
    "unimported-modules-read": {"attempt-forms-skip-unimported-modules", "attempt-forms-skip-name-like-unloaded-modules", "attempt-forms-skip-loaded-shared-modules",
                                "attempt-forms-skip-comment-named-modules"},
    "name-like-unloaded-read": {"attempt-forms-skip-name-like-unloaded-modules", "attempt-forms-skip-comment-named-modules"},
    "loaded-shared-read": {"attempt-forms-skip-loaded-shared-modules"},
    "loaded-by-text-read": {"attempt-forms-skip-comment-named-modules", "attempt-forms-found-in-import-module-modules"},
}


def red_cells_under(switch):
    """设上这一条开关再起一遍自证：→ (开关, 退出码, 红的格)。"""
    completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest"], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=300,
                               env=dict(os.environ, **{BREAK_VARIABLE: switch}))
    marker = "  ✗ 自证 "
    return switch, completed.returncode, {line[len(marker):].split("：", 1)[0] for line in completed.stdout.splitlines() if line.startswith(marker)}


def selftest():
    failures, checked = selftest_cells()
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ queue-health.py 自证 {checked} 格里 {len(failures)} 格不对" if failures else f"  ✓ queue-health.py 自证 {checked} 格全过（开了弄坏开关 {os.environ[BREAK_VARIABLE]}，一格都没红）")   # gate-lint:detail
        return 1 if failures else 0
    if failures:
        print(f"  ✗ queue-health.py 自证 {checked} 格里 {len(failures)} 格不对")
        print("  → 怎么办：照上面点名的格看判法；改了判法就连同文件头开关表与 SWITCH_EXPECTED_CELLS 一起改")
        return 1
    import heavy_threads
    jobs = heavy_threads.default_heavy_threads()
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        collected = list(pool.map(red_cells_under, SWITCH_EXPECTED_CELLS))
    switch_failures = []
    if len(collected) != len(SWITCH_EXPECTED_CELLS):
        switch_failures.append(f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(collected)} 个")
    for switch, code, red in collected:
        print(f"  · 弄坏开关 {switch}：退 {code}，红在 {'、'.join(sorted(red)) or '一格都没有'}")
        if red != SWITCH_EXPECTED_CELLS[switch] or code == 0:
            switch_failures.append(f"{switch} 要红在 {'、'.join(sorted(SWITCH_EXPECTED_CELLS[switch]))}，退 {code}、红在 {'、'.join(sorted(red)) or '一格都没有'}")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头开关表")
        return 1
    print(f"  ✓ queue-health.py 自证通过：{checked} 格全过；弄坏开关 {len(collected)} 个逐个打开都红在点名的格")
    return 0


def main(arguments):
    if os.environ.get(BREAK_VARIABLE) and arguments != ["--selftest"]:
        print(f"  ✗ {BREAK_VARIABLE} 只在自证里认，平常跑不认", file=sys.stderr)
        print(f"  → 怎么办：去掉这个环境变量再跑；要证红写 {BREAK_VARIABLE}=<项> python3 scripts/hosts/queue-health.py --selftest", file=sys.stderr)
        return EXIT_USAGE
    unknown = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item and item not in SWITCH_EXPECTED_CELLS]
    if unknown:
        print(f"  ✗ {BREAK_VARIABLE} 里有认不出的项：{'、'.join(unknown)}", file=sys.stderr)
        print("  → 怎么办：项名照文件头开关表写", file=sys.stderr)
        return EXIT_USAGE
    if arguments == ["--selftest"]:
        return selftest()
    try:
        options = parse_arguments(arguments, os.environ)
    except UsageError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print("  → 怎么办：写成 queue-health.py [<任务号>…] [--watch <秒>] [--interval <秒>]，或 --selftest", file=sys.stderr)
        return EXIT_USAGE
    return run_once(options, dict(os.environ))


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
