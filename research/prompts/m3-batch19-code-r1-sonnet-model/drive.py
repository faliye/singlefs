#!/usr/bin/env python3
"""第 19 批代码轮第一轮 正推腿的复跑驱动：冻结副本的拷贝上跑基线、13 行新变异、三个变体、两个探针。
用法：drive.py <工作目录(空)> <阶段>   阶段：setup | base | rows[:起-止 的 tsv 行号] | variants | probes
每个阶段把一行行结果追加进 <工作目录>/summary.txt，格式「[名] 退出码=… 红的=… 结果行=…」。
只跑 harness 档与 singlefs-core 的 --lib；每条 cargo 经 capped.sh 6 与 run-with-memory-cap.sh。
"""
import os, re, shutil, subprocess, sys

FROZEN = "/tmp/claude-1000/m3-batch19-code-r1/tree"
SNAPSHOT = "/home/fy5090/code/singlefs/research/prompts/m3-batch19-code-r1-snapshot/crates-sha256.txt"
HERE = os.path.dirname(os.path.realpath(__file__))
SCRIPTS = "/home/fy5090/code/singlefs/scripts"
W = os.path.realpath(sys.argv[1])
STAGE = sys.argv[2]
WORK = f"{W}/work"
OUT = f"{W}/out"
SUMMARY = f"{W}/summary.txt"
ENV = dict(os.environ, CARGO_TARGET_DIR=f"{W}/target")

H = "-p singlefs-harness --test "
BINS = {
    "unlink_mount_continuation_and_the_emptied_version": H + "unlink_mount_continuation_and_the_emptied_version",
    "the_mount_after_a_crashed_unmount": H + "the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads",
    "unlink_single_file_publishes": H + "unlink_single_file_publishes",
    "container_trees_grow_a_level": H + "container_trees_grow_a_level_when_the_root_is_full",
    "unlink_writes_per_transaction": H + "unlink_writes_per_transaction_do_not_grow_with_the_file",
    "inode_number_watermark": H + "inode_number_watermark_pressed_back_reissues_a_number",
    "core_lib_pressed_back": "-p singlefs-core --lib -- pressed_back_inode_number_watermark",
}


def run_cargo(args, label):
    """经 capped.sh 6 与内存包装跑 cargo test；上限先 8G，被峰值表拒（251）就改 auto。"""
    result = None
    # 先只编不跑（--no-run 不要求经内存包装）：编译的内存峰值不属于峰值表里那条「跑」的命令，混在一起 auto 的上限会被编译撞破。
    subprocess.run(["nice", "-n", "19", "bash", f"{SCRIPTS}/capped.sh", "6", "cargo", "test", "--offline", "--no-run"] + args.split(" -- ")[0].split(),
                   cwd=WORK, env=ENV, capture_output=True, text=True)
    for cap in ("8G", "auto"):
        cmd = ["nice", "-n", "19", "bash", f"{SCRIPTS}/capped.sh", "6", "bash", f"{SCRIPTS}/run-with-memory-cap.sh", cap,
               "cargo", "test", "--offline"] + args.split()
        result = subprocess.run(cmd, cwd=WORK, env=ENV, capture_output=True, text=True)
        if result.returncode != 251:
            break
    os.makedirs(OUT, exist_ok=True)
    text = result.stdout + result.stderr
    open(f"{OUT}/{label}.log", "w").write(text)
    failed = sorted(set(re.findall(r"^test (\S+)(?: - should panic)? \.\.\. FAILED$", text, re.M)))
    results = re.findall(r"^test result: .*$", text, re.M)
    short = ",".join(name.split("::")[-1][:70] for name in failed)
    with open(SUMMARY, "a") as summary:
        summary.write(f"[{label}] 退出码={result.returncode} 红的={short or '无'} 结果行={' | '.join(r[:62] for r in results)}\n")
    print(open(SUMMARY).read().splitlines()[-1])


def tsv_row(line_number):
    return open(f"{FROZEN}/crates/mutations.tsv", encoding="utf-8").read().split("\n")[line_number - 1].split("\t")


def with_edits(edits, thunk):
    """在 WORK 里按 (文件, 原文, 替换文) 改，跑 thunk，再还原；原文必须恰好命中一次。"""
    saved = {}
    try:
        for path, orig, repl in edits:
            full = f"{WORK}/{path}"
            text = saved.setdefault(full, open(full, encoding="utf-8").read())
            current = open(full, encoding="utf-8").read()
            assert current.count(orig) == 1, f"{path} 原文命中 {current.count(orig)} 次"
            open(full, "w", encoding="utf-8").write(current.replace(orig, repl))
        thunk()
    finally:
        for full, text in saved.items():
            open(full, "w", encoding="utf-8").write(text)


def row_edit(line_number):
    name, path, orig, repl, args, test = tsv_row(line_number)[:6]
    return name, (path, orig.replace("\\n", "\n"), repl.replace("\\n", "\n"))


def setup():
    assert not os.path.exists(WORK), "工作目录必须是空的"
    os.makedirs(W, exist_ok=True)
    shutil.copytree(FROZEN, WORK)
    check = subprocess.run(["sha256sum", "-c", SNAPSHOT], cwd=WORK, capture_output=True, text=True)
    print(check.stdout.strip().splitlines()[-1], "| 退出码", check.returncode)
    assert check.returncode == 0


ROWS = {  # tsv 行号 → cargo 参数（第 5 段原样；2670–2680 改跑整个测试二进制看全部红的）
    2670: BINS["unlink_mount_continuation_and_the_emptied_version"],
    2671: BINS["unlink_mount_continuation_and_the_emptied_version"],
    2672: BINS["unlink_mount_continuation_and_the_emptied_version"],
    2673: BINS["unlink_mount_continuation_and_the_emptied_version"],
    2674: BINS["the_mount_after_a_crashed_unmount"],
    2675: BINS["the_mount_after_a_crashed_unmount"],
    2676: BINS["unlink_writes_per_transaction"],
    2677: BINS["inode_number_watermark"],
    2678: BINS["inode_number_watermark"],
    2679: BINS["inode_number_watermark"],
    2680: BINS["inode_number_watermark"],
    2681: BINS["core_lib_pressed_back"],
    2682: BINS["core_lib_pressed_back"],
}


def probe_n7():
    src = open(f"{WORK}/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs", encoding="utf-8").read()
    head = src[: src.index("#[test]")].replace("//!", "//")
    body = open(f"{HERE}/probe/zz_probe_n7_body.rs", encoding="utf-8").read()
    open(f"{WORK}/crates/singlefs-harness/tests/zz_probe_n7.rs", "w", encoding="utf-8").write(head + body)


def probe_n4_admission():
    """N4 那条用例的副本：只把第二个 mount_writable 里的准入开关换成照式子判，看基线上第一次写的结局。"""
    tests = f"{WORK}/crates/singlefs-harness/tests"
    src = open(f"{tests}/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs", encoding="utf-8").read()
    i = src.index("fn after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk")
    assert src[i:].count("SpaceAdmission::SkippedByTheTestOnlySwitch") == 1
    open(f"{tests}/zz_probe_n4.rs", "w", encoding="utf-8").write(
        src[:i] + src[i:].replace("SpaceAdmission::SkippedByTheTestOnlySwitch", "SpaceAdmission::JudgedByTheFormula"))


def main():
    if STAGE == "setup":
        setup()
    elif STAGE == "base":
        for label, args in BINS.items():
            run_cargo(args, f"base-{label}")
    elif STAGE.startswith("rows"):
        first, last = (2670, 2682)
        if ":" in STAGE:  # rows:2670-2676
            first, last = map(int, STAGE.split(":")[1].split("-"))
        for line_number, args in ROWS.items():
            if not first <= line_number <= last:
                continue
            name, edit = row_edit(line_number)
            with_edits([edit], lambda: run_cargo(args, f"row-{line_number}"))
    elif STAGE == "variants":
        # ① 批 17 攻方原版 qfm（会话字段，三处构造 + 本轮多出的第四处构造）
        subprocess.run(["patch", "-p0", "-s", "-i", f"{HERE}/patches/qfm-session-field-batch17.patch"], cwd=WORK, check=True)
        history = f"{WORK}/crates/singlefs-harness/src/history.rs"
        text = open(history, encoding="utf-8").read()
        anchor = "    let mut session_that_loses_power = MountedSession {\n"
        assert text.count(anchor) == 1
        open(history, "w", encoding="utf-8").write(text.replace(anchor, anchor + "        latest_change_time_seconds: 0,\n"))
        try:
            run_cargo(BINS["unlink_mount_continuation_and_the_emptied_version"], "variant-qfm-session-field-batch17")
        finally:
            for rel in ("crates/singlefs-core/src/mounted_session.rs", "crates/singlefs-harness/src/history.rs"):
                shutil.copyfile(f"{FROZEN}/{rel}", f"{WORK}/{rel}")
        # ② 单处写法的会话记忆（thread_local 记最晚时间，只在 publish_user_change 开头改 change）
        anchor2 = ("        let MountedSession {\n            allocator,\n            current,\n            instance,\n            shadow_ledger,\n"
                   "            parameters_and_device_table,\n        } = self;\n        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {")
        insert = open(f"{HERE}/patches/qfm-thread-local-insert.rs.txt", encoding="utf-8").read()
        with_edits([("crates/singlefs-core/src/mounted_session.rs", anchor2, insert + anchor2)],
                   lambda: run_cargo(BINS["unlink_mount_continuation_and_the_emptied_version"], "variant-qfm-thread-local"))
        # ③ 甲（放开挪到暖机之前）再关掉取号之前预演对真发那一判，看 N3 那条自己的断言红不红
        jia = row_edit(2674)[1]
        guard = "    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {"
        with_edits([jia, ("crates/singlefs-core/src/mount.rs", guard,
                          "    if let (Some(planned), false, true) = (&placements_planned, some_copy_was_quarantined, false) {")],
                   lambda: run_cargo(BINS["the_mount_after_a_crashed_unmount"], "variant-jia-without-the-core-assert"))
    elif STAGE == "probes":
        probe_n7()
        try:
            run_cargo(H + "zz_probe_n7 -- --nocapture", "probe-n7-baseline")
            with_edits([row_edit(2679)[1]], lambda: run_cargo(H + "zz_probe_n7 -- --nocapture", "probe-n7-under-row-2679"))
            for label in ("probe-n7-baseline", "probe-n7-under-row-2679"):
                for line in open(f"{OUT}/{label}.log", encoding="utf-8").read().splitlines():
                    if line.startswith("PROBE"):
                        print(f"  [{label}] {line}")
                        open(SUMMARY, "a").write(f"  [{label}] {line}\n")
            probe_n4_admission()
            run_cargo(H + "zz_probe_n4 -- regrows", "probe-n4-admission-judged-baseline")
        finally:
            for probe in ("zz_probe_n7.rs", "zz_probe_n4.rs"):
                if os.path.exists(f"{WORK}/crates/singlefs-harness/tests/{probe}"):
                    os.remove(f"{WORK}/crates/singlefs-harness/tests/{probe}")
    else:
        sys.exit("阶段：setup | base | rows | variants | probes")


main()
