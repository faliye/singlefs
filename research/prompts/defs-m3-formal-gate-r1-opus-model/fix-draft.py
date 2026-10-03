#!/usr/bin/env python3
"""在草稿拷贝里打两处草稿改法（只在攻方腿的模型上量过、被攻过零轮），不碰仓：
  F1  admission.py stage_fingerprint：登记了 environment= 的阶段，环境的 sha256 进全绿标记的指纹（标记按环境分格）。
  F2  verus-proof-check.py tool_versions：另打 rust_verify、z3、vstd.vir、libvstd.rlib 的 sha256 与 VERUS_Z3_PATH 指的文件。
用法：python3 fix-draft.py <拷贝根> <F1|F1F2>"""
import sys
root, variant = sys.argv[1], sys.argv[2]

def replace_once(path, old, new):
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        sys.exit(f"{path}: 旧串命中 {text.count(old)} 次，要恰好 1 次")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))

replace_once(f"{root}/research/scripts/admission.py",
'''    named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
    _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
    return fingerprint, file_count, input_paths


def stage_marker_path''',
'''    named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
    _environment_lines, environment_digest = current_gate_environment(root, stage_rows)   # F1（草稿）：登记了 environment= 的，环境进标记的指纹
    if environment_digest:
        named_lines.append((f"<环境：{stage}>", environment_digest.encode()))
    _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
    return fingerprint, file_count, input_paths


def stage_marker_path''')
if variant == "F1F2":
    replace_once(f"{root}/research/scripts/verus-proof-check.py",
'''        print(f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest}")''',
'''        extra = []   # F2（草稿）：verus 只是起 rust_verify 的外壳；判定还取决于 rust_verify、z3、vstd，与指走 z3 的 VERUS_Z3_PATH
        for name in ("rust_verify", "z3", "vstd.vir", "libvstd.rlib"):
            path = os.path.join(directory, name)
            extra.append(f"{name}_sha256=" + (hashlib.sha256(open(path, "rb").read()).hexdigest() if os.path.isfile(path) else "没有"))
        z3_override = os.environ.get("VERUS_Z3_PATH", "")
        extra.append("VERUS_Z3_PATH=" + (f"{z3_override}:" + hashlib.sha256(open(z3_override, "rb").read()).hexdigest() if z3_override and os.path.isfile(z3_override) else (z3_override or "未设")))
        print(f"verus version={version.get('version')} commit={version.get('commit')} toolchain={(version.get('toolchain') or '').split(' ')[0]} verus_sha256={digest} " + " ".join(extra))''')
print(f"{variant} 打上了：{root}")
