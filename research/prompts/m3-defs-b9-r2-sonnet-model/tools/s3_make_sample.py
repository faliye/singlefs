#!/usr/bin/env python3
"""S3：造 `|| X=$变量` 的变量来源判法的边界样本。用法：s3_make_sample.py <样本根>（样本根里要已有 .claude/ 与 scripts/hosts/，从第一轮攻方模型的 feeds/z2/sample 拷）
每份 scripts/experiments/s3_*.sh 只放一个情形；前缀 s3_ok、s3_cap_variable、s3_braced、s3_export、s3_func_local 预期不点名（变量只由登记位读法赋值），
其余各份的预期写在 EXPECT 里（red=点名，hole=按字面本该红而格放了）。"""
import os, sys
root = sys.argv[1]
d = os.path.join(root, "scripts/experiments")
os.makedirs(d, exist_ok=True)
H = 'source "$(dirname "$0")/../hosts/heavy-threads.sh"'
REG = 'default_threads="$(heavy_thread_default)"'
def use(tag, var="default_threads"):
    return f'[[ -n "${{S3_{tag}_THREADS:-}}" ]] || S3_{tag}_THREADS=${var}'
cases = {
  # 名: (预期, [行...])
  "s3_ok_registry":          ("green", [H, REG, use("OK")]),
  "s3_func_local":           ("green", [H, 'pick_default() { local default_threads; default_threads="$(heavy_thread_default)"; }', 'pick_default', use("FUNC")]),
  "s3_func_never_called":    ("green-semantic-hole", [H, 'pick_default() { local default_threads; default_threads="$(heavy_thread_default)"; }', use("NEVER")]),
  "s3_sourced_literal_override": ("hole", [H, REG, 'source "$(dirname "$0")/s3_defaults_literal.sh"', use("OVERRIDE")]),
  "s3_defaults_literal":     ("(被 source 的文件，不点名)", ['# 被 source 的文件：把小写变量赋成字面数', 'default_threads=8']),
  "s3_sourced_only":         ("red", [H, 'source "$(dirname "$0")/s3_defaults_literal.sh"', use("SRCONLY")]),
  "s3_twice_registry_then_literal": ("red", [H, REG, 'default_threads=8', use("TWICEA")]),
  "s3_twice_literal_then_registry": ("red-false-positive", [H, 'default_threads=8', REG, use("TWICEB")]),
  "s3_conditional_literal":  ("red", [H, REG, '[[ -z "${S3_COND_PROBE:-}" ]] && default_threads=8', use("COND")]),
  "s3_let_literal":          ("red", [H, REG, 'let default_threads=8', use("LET")]),
  "s3_arith_spaces":         ("hole", [H, REG, '(( default_threads = 8 ))', use("ARITH")]),
  "s3_eval_literal":         ("hole", [H, REG, 'eval "default_threads=8"', use("EVAL")]),
  "s3_read_literal":         ("hole", [H, REG, 'read -r default_threads <<< 8', use("READ")]),
  "s3_printf_v_literal":     ("hole", [H, REG, 'printf -v default_threads %s 8', use("PRINTFV")]),
  "s3_for_literal":          ("hole", [H, REG, 'for default_threads in 8; do :; done', use("FOR")]),
  "s3_plus_equals":          ("hole", [H, REG, 'default_threads+=0', use("PLUSEQ")]),
  "s3_cap_variable":         ("green", ['cap_jobs="${SINGLEFS_THREAD_CAP}"', use("CAP", "cap_jobs")]),
  "s3_cap_with_literal_fallback": ("red", ['cap_jobs="${SINGLEFS_THREAD_CAP:-8}"', use("CAPFB", "cap_jobs")]),
  "s3_registry_word_in_string": ("red-or-hole", ['default_threads="用 heavy_thread_default 取，这里先写个 8"', use("STR")]),
  "s3_registry_word_in_comment": ("red", ['default_threads=8   # 以后改成 heavy_thread_default', use("CMT")]),
  "s3_registry_or_fallback": ("red", [H, 'default_threads="$(heavy_thread_default || echo 8)"', use("ORFB")]),
  "s3_registry_halved":      ("red", [H, 'default_threads=$(( $(heavy_thread_default) / 2 ))', use("HALF")]),
  "s3_arith_use":            ("red", [H, REG, '[[ -n "${S3_ARITHUSE_THREADS:-}" ]] || S3_ARITHUSE_THREADS=$((default_threads))']),
  "s3_braced_use":           ("green", [H, REG, '[[ -n "${S3_BRACED_THREADS:-}" ]] || S3_BRACED_THREADS=${default_threads}']),
  "s3_two_vars_one_literal": ("red (只点名 bad_threads 那一行)", [H, 'good_threads="$(heavy_thread_default)"', 'bad_threads=8', use("TWOV", "good_threads"), use("TWOV2", "bad_threads")]),
  "s3_export_form":          ("green", [H, 'export default_threads="$(heavy_thread_default)"', use("EXP")]),
  "s3_declare_i_literal":    ("red", [H, REG, 'declare -i default_threads=8', use("DECL")]),
  "s3_assigned_after_use":   ("green-semantic-hole", [H, use("AFTER"), REG]),
  "s3_subshell_assign":      ("green-semantic-hole", [H, '( ' + REG + ' )', use("SUB")]),
}
for name, (_expect, lines) in cases.items():
    with open(os.path.join(d, name + ".sh"), "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\n" + "\n".join(lines) + "\n")
with open(os.path.join(root, "EXPECT.tsv"), "w", encoding="utf-8") as handle:
    for name, (expect, _lines) in cases.items():
        handle.write(f"scripts/experiments/{name}.sh\t{expect}\n")
print(len(cases), "份")
