#!/usr/bin/env python3
"""正推腿的样本：在 --work 下造出 unsafe-allowances 格的最小样本根，一个样本一个进程组起判法，打出实际退出码、预测、对不对。
只读仓里的判法（要与开工快照同哈希），样本全写在 --work 下；不改仓。预测是读判法源码推出来的，对不上就是推错了，两样都当数据报。
用法：python3 -I run_samples.py --repo <仓根> --frozen <冻结副本> --work <草稿目录> --out <汇总文件>
"""
import argparse, hashlib, os, signal, subprocess, sys, time

EXPECTED_GATE_SHA256 = "a054a21d5596c1f26d5a2c3c82094018c34bc1e9a6611a2bfeace09304ea4871"   # 开工快照 judged-sha256.txt 里 code-source-discipline.sh 那一行
CRATE = "singlefs-checker-tier"
BIN_DIRECTORY = f"crates/{CRATE}/src/bin"
REASON = "样本的理由：std 没有线程 CPU 钟，不放开就只能读 /proc 里按滴答计的粗读数"
SOURCE = "用户 2026-11-03 弹窗定 records/sample-decision.md"
LIBRARY_ROW = f"{CRATE}\tforbid\t-\t-\t-\t-\t-\t-"
LIBRARY_ROOT_TEXT = "//! 样本：库根 forbid。\n#![forbid(unsafe_code)]\n\npub fn crash_state_count() -> usize {\n    0\n}\n"
EXTERN_BLOCK = 'extern "C" {\n    fn clock_gettime(reading: *mut i64) -> i32;\n}\n\n'
ALLOW = '#[allow(unsafe_code, reason = "样本：std 没有线程 CPU 钟")]\n'
SAFETY = "// SAFETY: reading 是调用方栈帧里活着的整数，指针只在这一次调用里用。\n"
DENY = "#![deny(unsafe_code)]\n\n"
SEPARATOR = "@@U2028@@"


def registered_row(bin_name, function, count, words="栈帧|这一次调用"):
    root = f"{CRATE}/src/bin/{bin_name}.rs"
    return "\t".join([root, "forbid,deny", f"crates/{CRATE}/{root[len(CRATE) + 1:]}", function, str(count), words, REASON, SOURCE])


def indent(text, spaces):
    return "".join((" " * spaces + line if line.strip() else line) for line in text.splitlines(keepends=True))


def registered_bin(body, count=1, function="read_the_thread_clock", head=""):
    """body 是项（函数、impl 块）的文本，不带 #[allow]；样本用 {ALLOW} 占位自己放。"""
    return ([registered_row("e900_x", function, count)], {f"{BIN_DIRECTORY}/e900_x.rs": DENY + head + EXTERN_BLOCK + body + "\nfn main() {}\n"})


def unregistered_bin(text, name="e900_x"):
    return ([], {f"{BIN_DIRECTORY}/{name}.rs": text})


FUNCTION_HEAD = ALLOW + "fn read_the_thread_clock(reading: &mut i64) -> i32 {\n"
CALL = "unsafe { clock_gettime(reading) }"


def build_samples():
    """(名字, 哪一问, 预测 green/red, 红时输出里要有的字, 登记表多出来的行, {路径: 文本})。"""
    samples = []

    def add(name, question, expect, needle, built):
        rows, files = built
        samples.append((name, question, expect, needle, rows, files))

    body_ok = FUNCTION_HEAD + indent(SAFETY, 4) + f"    let status = {CALL};\n    status\n}}\n"
    add("control-registered-green", "校准", "green", None, registered_bin(body_ok))
    body_between = FUNCTION_HEAD + indent(SAFETY, 4) + f"    let checked = 1;\n    let status = {CALL};\n    status + checked - 1\n}}\n"
    add("control-statement-between-red", "校准", "red", "上方没有 // SAFETY: 注释", registered_bin(body_between))

    # ── Q1.4 / Q1.3：判法看不见的 unsafe ──
    add("include-of-a-non-rs-file-green", "Q1.4", "green", None, (
        [], {f"{BIN_DIRECTORY}/e900_x.rs": 'include!("e900_x_part.inc");\n',
             f"{BIN_DIRECTORY}/e900_x_part.inc": "fn main() {\n    let mut reading = 0_i64;\n    let _ = unsafe { std::ptr::read(&mut reading as *mut i64) };\n}\n"}))
    add("raw-c-string-hides-unsafe-green", "Q1.4", "green", None, unregistered_bin(
        'fn main() {\n    let _opening = cr"\\";\n    let _status = unsafe { std::mem::zeroed::<u8>() };\n    let _closing = cr"\\";\n}\n'))
    add("no-mangle-without-the-keyword-green", "Q1.4", "green", None, unregistered_bin(
        '#[no_mangle]\npub extern "C" fn exported_without_the_unsafe_keyword() {}\n\nfn main() {}\n'))
    add("cfg-off-unsafe-is-counted-red", "Q1.4", "red", "unsafe 关键字 1 处，根是 forbid", unregistered_bin(
        "#[cfg(any())]\nfn never_compiled() {\n    let _ = unsafe { std::mem::zeroed::<u8>() };\n}\n\nfn main() {}\n"))
    add("words-in-strings-and-comments-green", "Q1.4", "green", None, unregistered_bin(
        "//! 文档里写 unsafe { } 不算。\n/// unsafe fn\n// unsafe { }\n/* unsafe /* 嵌套 unsafe */ */\nfn main() {\n"
        '    let _a = "unsafe { }";\n    let _b = r#"unsafe " 还在原始字符串里"#;\n    let _c = b"unsafe";\n    let _d = \'u\';\n'
        "    let unsafe_code = 1;\n    let _ = unsafe_code;\n    let _lifetime: &'static str = \"x\";\n}\n"))
    body_macro = (FUNCTION_HEAD + "    macro_rules! clock_status {\n        ($target:expr) => {\n" + indent(SAFETY, 12)
                  + "            unsafe { clock_gettime($target) }\n        };\n    }\n"
                  "    let first = clock_status!(reading);\n    let second = clock_status!(reading);\n    let third = clock_status!(reading);\n    first + second + third\n}\n")
    add("macro-expands-three-unsafe-for-one-keyword-green", "Q1.4", "green", None, registered_bin(body_macro))
    add("bin-helper-file-goes-to-the-library-root-red", "Q1.3", "red", "unsafe 关键字 1 处，根是 forbid", (
        [], {f"{BIN_DIRECTORY}/e900_x.rs": "mod e900_x_helper;\n\nfn main() {\n    e900_x_helper::run();\n}\n",
             f"{BIN_DIRECTORY}/e900_x/e900_x_helper.rs": "pub fn run() {\n    // SAFETY: 样本。\n    let _ = unsafe { std::mem::zeroed::<u8>() };\n}\n"}))

    # ── Q1.2：SAFETY 按语句开头认 ──
    body_separator = (FUNCTION_HEAD + indent(SAFETY, 4) + "    let unrelated = 1;\n" + f"    let status = {CALL};\n    status + unrelated - 1\n}}\n")
    separator_head = f"// 注释里夹一个行分隔符：甲{SEPARATOR}乙\n"
    add("line-separator-shifts-the-safety-lookup-green", "Q1.2", "green", None, registered_bin(body_separator, head=separator_head))
    add("same-text-without-the-separator-red", "Q1.2", "red", "上方没有 // SAFETY: 注释",
        registered_bin(body_separator, head=separator_head.replace(SEPARATOR, " ")))
    body_fake = (FUNCTION_HEAD + '    let _note = "\n' + SAFETY + f'"; let status = {CALL};\n    status\n}}\n')
    add("safety-line-inside-a-string-green", "Q1.2", "green", None, registered_bin(body_fake))
    body_chain = (FUNCTION_HEAD + indent(SAFETY, 4) + f"    let total = {CALL} +\n        {CALL};\n    total\n}}\n")
    add("one-comment-covers-the-second-unsafe-of-a-chain-green", "Q1.2", "green", None, registered_bin(body_chain, count=2))
    body_method_chain = (FUNCTION_HEAD + indent(SAFETY, 4) + "    let status = Some(0)\n        .map(|_| " + CALL + ")\n        .unwrap_or(0);\n    status\n}\n")
    add("unsafe-below-the-first-line-of-a-method-chain-red", "Q1.2", "red", "上方没有 // SAFETY: 注释", registered_bin(body_method_chain))
    arms_ok = (ALLOW + "fn read_the_thread_clock(reading: &mut i64, which: u8) -> i32 {\n    match which {\n" + indent(SAFETY, 8)
               + f"        0 =>\n            {CALL},\n" + indent(SAFETY, 8) + f"        1 => {CALL},\n        _ => {{\n" + indent(SAFETY, 12)
               + f"            {CALL}\n        }}\n    }}\n}}\n")
    add("match-arms-each-with-its-own-comment-green", "Q1.2", "green", None, registered_bin(arms_ok, count=3))
    arms_wrong = (ALLOW + "fn read_the_thread_clock(reading: &mut i64, which: u8) -> i32 {\n    match which {\n" + indent(SAFETY, 8)
                  + f"        0 => 0,\n        _ => {CALL},\n    }}\n}}\n")
    add("match-arm-comment-written-for-the-previous-arm-red", "Q1.2", "red", "上方没有 // SAFETY: 注释", registered_bin(arms_wrong))

    # ── Q1.1：嵌套项与同名方法 ──
    body_nested = (ALLOW + "fn read_the_thread_clock(reading: &mut i64) -> i32 {\n    fn inner_reader(reading: &mut i64) -> i32 {\n" + indent(SAFETY, 8)
                   + f"        {CALL}\n    }}\n    let closure = |slot: &mut i64| {{\n" + indent(SAFETY.replace("reading", "slot"), 8)
                   + "        unsafe { clock_gettime(slot) }\n    };\n    inner_reader(reading) + closure(reading)\n}\n")
    add("inner-fn-and-closure-counted-in-the-outer-row-green", "Q1.1", "green", None, registered_bin(body_nested, count=2))
    body_two_impls = ("struct Reader<Width>(Width);\n\nimpl Reader<u8> {\n    fn read(&self, reading: &mut i64) -> i32 {\n        *reading = 1;\n        0\n    }\n}\n\n"
                      "impl Reader<u16> {\n    " + ALLOW.strip() + "\n    fn read(&self, reading: &mut i64) -> i32 {\n" + indent(SAFETY, 8) + f"        {CALL}\n    }}\n}}\n")
    add("same-method-name-in-two-inherent-impls-red", "Q1.1", "red", "不在登记的", registered_bin(body_two_impls, function="Reader::read"))
    return samples


def sha256_of(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def materialise(root, rows, files):
    os.makedirs(os.path.join(root, ".claude", "gate.d"), exist_ok=True)
    os.makedirs(os.path.join(root, "records"), exist_ok=True)
    with open(os.path.join(root, ".claude", "gate.d", "unsafe-allowances.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# 样本登记表\n" + "\n".join([LIBRARY_ROW] + rows) + "\n")
    with open(os.path.join(root, "records", "sample-decision.md"), "w", encoding="utf-8") as handle:
        handle.write("# 样本：登记表出处指向的记录\n\n样本登记表的第 8 段指向这份文件。\n")
    with open(os.path.join(root, ".gate-cells"), "w", encoding="utf-8") as handle:
        handle.write("unsafe-allowances\n")
    files = dict(files)
    files[f"crates/{CRATE}/src/lib.rs"] = LIBRARY_ROOT_TEXT
    for relative, text in files.items():
        path = os.path.join(root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="") as handle:
            handle.write(text.replace(SEPARATOR, " "))


def run_in_own_group(command, cwd, env, limit_seconds):
    """起在自己的进程组里，超时与结束都对整组发信号；交回（退出码或 None, 输出, 结束后组里还剩几个进程）。"""
    process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    code = None
    try:
        output, _ = process.communicate(timeout=limit_seconds)
        code = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            output, _ = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            output, _ = process.communicate()
        output += "\n（超时，已对进程组发信号）"
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    time.sleep(0.2)
    try:
        os.killpg(process.pid, 0)
        left = 1
    except ProcessLookupError:
        left = 0
    return code, output, left


def main():
    parser = argparse.ArgumentParser()
    for name in ("repo", "frozen", "work", "out"):
        parser.add_argument(f"--{name}", required=True)
    arguments = parser.parse_args()
    gate = os.path.join(arguments.repo, ".claude", "gate.d", "code-source-discipline.sh")
    actual = sha256_of(gate)
    print(f"name=gate_hash path={gate} sha256={actual} expected={EXPECTED_GATE_SHA256} same={actual == EXPECTED_GATE_SHA256}")
    frozen_gate = os.path.join(arguments.frozen, ".claude", "gate.d", "code-source-discipline.sh")
    print(f"name=frozen_gate_hash path={frozen_gate} sha256={sha256_of(frozen_gate) if os.path.isfile(frozen_gate) else 'absent'}")
    if actual != EXPECTED_GATE_SHA256:
        print("判法与开工快照不是同一份，什么都没跑", file=sys.stderr)
        sys.exit(3)
    environment = {key: value for key, value in os.environ.items() if not key.startswith(("GATE_", "STAGE_CELLS_", "REJECTION_BLOCKS_", "LAG_BASELINE_"))}
    lines, matched, mismatched, leftover = [], 0, 0, 0
    outputs = os.path.join(arguments.work, "samples-output")
    os.makedirs(outputs, exist_ok=True)
    for name, question, expect, needle, rows, files in build_samples():
        root = os.path.join(arguments.work, "samples", name)
        materialise(root, rows, files)
        code, output, left = run_in_own_group(["bash", gate, "--check", "unsafe-allowances", root], arguments.repo, environment, 240)
        leftover += left
        with open(os.path.join(outputs, f"{name}.out"), "w", encoding="utf-8") as handle:
            handle.write(output)
        verdict = {0: "green", 1: "red"}.get(code, f"other({code})")
        needle_found = "n/a" if needle is None else str(needle in output)
        agrees = verdict == expect and (needle is None or needle in output)
        matched, mismatched = matched + agrees, mismatched + (not agrees)
        lines.append(f"name=sample sample={name} question={question} predicted={expect} actual={verdict} rc={code} needle_found={needle_found} prediction_holds={agrees}")
    lines.append(f"name=samples_summary total={matched + mismatched} prediction_holds={matched} prediction_fails={mismatched} leftover_process_groups={leftover}")
    with open(arguments.out, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    print("\n".join(lines))
    sys.exit(0 if leftover == 0 else 1)


if __name__ == "__main__":
    main()
