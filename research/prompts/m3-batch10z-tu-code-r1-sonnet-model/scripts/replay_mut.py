#!/usr/bin/env python3
"""replay_mut.py <gpu|crash> : 对扫源码模型套我自己的变异（旧串→新串），编、跑，报哪些用例红（全绿 = 活下来）。"""
import os, re, subprocess, sys
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet')
R = '/home/fy5090/code/singlefs/scripts'
S = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree/crates/singlefs-checker-tier/src/')
kind = sys.argv[1]
only = set(sys.argv[2:])
model = open(f'{W}/replay/{kind}_model.rs').read()
srcfile = S + ('gpu_unit_checks.rs' if kind == 'gpu' else 'crash_amplification.rs')
envname = 'GPU_SRC' if kind == 'gpu' else 'CRASH_SRC'
M = {}
def mut(k, name, old, new): M.setdefault(k, {})[name] = (old, new)
# ---- GPU 分词与 use 树
mut('gpu', 'G1_char_literal_never', "            } else if first_character == '\\'' {\n                match length_of_the_character_literal_at_the_start(rest) {", "            } else if first_character == '\\'' {\n                match None::<usize> {")
mut('gpu', 'G2_raw_string_off', 'if matches!(word, "r" | "br")\n                    && after_the_word.trim_start_matches(\'#\').starts_with(\'"\')', 'if false && matches!(word, "r" | "br")\n                    && after_the_word.trim_start_matches(\'#\').starts_with(\'"\')')
mut('gpu', 'G3_block_comment_off', '            } else if rest.starts_with("/*") {\n                position += length_of_the_block_comment_at_the_start(rest);', '            } else if false && rest.starts_with("/*") {\n                position += length_of_the_block_comment_at_the_start(rest);')
mut('gpu', 'G4_block_comment_not_nested', '            if rest.starts_with("/*") {\n                depth += 1;\n                position += 2;', '            if rest.starts_with("/*") && depth == 0 {\n                depth += 1;\n                position += 2;')
mut('gpu', 'G5_line_comment_off', '            } else if rest.starts_with("//") {\n                position += rest.find(\'\\n\').unwrap_or(rest.len());', '            } else if false && rest.starts_with("//") {\n                position += rest.find(\'\\n\').unwrap_or(rest.len());')
mut('gpu', 'G6_string_escape_off', "            position += if byte == b'\\\\' { 2 } else { 1 };", "            position += 1;")
mut('gpu', 'G7_raw_ident_off', '                } else if word == "r" && after_the_word.starts_with(\'#\') {\n                    // 原始标识符', '                } else if false && word == "r" && after_the_word.starts_with(\'#\') {\n                    // 原始标识符')
mut('gpu', 'G8_byte_string_off', '                } else if word == "b" && after_the_word.starts_with(\'"\') {', '                } else if false && word == "b" && after_the_word.starts_with(\'"\') {')
mut('gpu', 'G9_lifetime_as_char_start', "            .then(|| first_character.len_utf8() + 2)", "            .then(|| first_character.len_utf8() + 1)")
mut('gpu', 'G10_escaped_char_end', "            let closing_quote = 3 + text[3..].find('\\'').expect(\"转义的字符字面量有收尾的引号\");", "            let closing_quote = 2 + text[2..].find('\\'').expect(\"转义的字符字面量有收尾的引号\");")
mut('gpu', 'G11_depth_rule_zero_removed', '            if pair[0].0 == 0\n                && pair[1].1 == "!"', '            if pair[1].1 == "!"')
mut('gpu', 'G12_macro_alnum_removed', '                && pair[0]\n                    .1\n                    .starts_with(|character: char| character.is_alphanumeric() || character == \'_\')', '')
mut('gpu', 'G13_expected_source_ignored', '            if sources_brought_in_under_the_name != [expected_source.as_str()] {', '            if sources_brought_in_under_the_name.len() != 1 {')
mut('gpu', 'G14_self_group_trailing_comma', '                    position =\n                        after_the_inner_tree + usize::from(statement[after_the_inner_tree] == ",");', '                    position = after_the_inner_tree + 1;')
mut('gpu', 'G15_double_colon_token_off', '            } else if rest.starts_with("::") {\n                tokens.push("::");\n                position += 2;', '            } else if rest.starts_with("::") {\n                tokens.push(":");\n                tokens.push(":");\n                position += 2;')
# ---- crash
mut('crash', 'C1_char_literal_off', "                        } else if character == '\\'' {\n                            // 字符字面量", "                        } else if false && character == '\\'' {\n                            // 字符字面量")
mut('crash', 'C2_cr_raw_off', 'matches!(word.as_str(), "r" | "br" | "cr")', 'matches!(word.as_str(), "r" | "br")')
mut('crash', 'C3_block_comment_not_nested', "                        if character == '/' && next_character == Some('*') {\n                            index += 1;\n                            place = PlaceInTheSource::InsideABlockComment {\n                                nesting: nesting + 1,\n                            };", "                        if false {\n                            index += 1;\n                            place = PlaceInTheSource::InsideABlockComment {\n                                nesting: nesting + 1,\n                            };")
mut('crash', 'C4_string_escape_off', "                        if character == '\\\\' {\n                            index += 1;\n                        } else if character == '\"' {\n                            place = PlaceInTheSource::Code;", "                        if false {\n                            index += 1;\n                        } else if character == '\"' {\n                            place = PlaceInTheSource::Code;")
mut('crash', 'C5_line_comment_off', "                        } else if character == '/' && next_character == Some('/') {\n                            break;", "                        } else if false && character == '/' && next_character == Some('/') {\n                            break;")
mut('crash', 'C6_lifetime_branch_off', "                            } else if characters.get(index + 2) == Some(&'\\'') {\n                                index += 2;\n                            }", "                            }")
mut('crash', 'C7_raw_closing_hashes_ignored', "                        if character == '\"' && closing_hashes_follow {", "                        if character == '\"' {")
mut('crash', 'C8_escape_char_literal_end', "                                while characters.get(index + 1).is_some_and(|next| *next != '\\'') {\n                                    index += 1;\n                                }\n                                index += 1;", "                                index += 1;")
mut('crash', 'C9_ident_skip_off', "                            } else {\n                                index = word_end - 1;\n                            }", "                            }")
mut('crash', 'C10_let_pattern_end_only_equals', "                                .find(['=', ';'])", "                                .find(['='])")
mut('crash', 'C11_whole_word_neighbor_ignored', "                !is_part_of_a_name(text[..*position].chars().next_back())\n                    && !is_part_of_a_name(text[position + word.len()..].chars().next())", "                true")
mut('crash', 'C12_ref_type_prefix_any_amp', "                let end_of_the_type = ampersand\n                    + end_of_a_prefix_ignoring_whitespace(", "                let end_of_the_type = ampersand\n                    + end_of_a_prefix_ignoring_whitespace(")
mut('crash', 'C13_layer_name_from_first_fn', "                let layer = whole_word_positions(&code[..ampersand], \"fn\")\n                    .last()", "                let layer = whole_word_positions(&code[..ampersand], \"fn\")\n                    .first()")
mut('crash', 'C14_depth_must_be_1_removed', "                        if depths[*pinned_offset] != 1 {", "                        if depths[*pinned_offset] > 1000 {")
mut('crash', 'C15_type_name_count_removed', "                    if lines_naming_the_type != 1 {", "                    if lines_naming_the_type == 0 {")
mut('crash', 'C16_pinned_lines_compare_removed', "                        != [*pinned_line]", "                        != [*pinned_line, *pinned_line][..0]")
for name, (old, new) in M.get(kind, {}).items():
    if only and name not in only: continue
    if model.count(old) != 1:
        print(f'{name}: 旧串命中 {model.count(old)} 次，跳过'); continue
    mp = f'{W}/replay/{kind}_m_{name}.rs'
    open(mp, 'w').write(model.replace(old, new, 1))
    out_bin = f'{W}/replay/{kind}_m_{name}'
    c = subprocess.run(['rustc', '--edition', '2021', '-O', '-o', out_bin, mp], capture_output=True, text=True)
    if c.returncode != 0:
        print(f'{name}: 编不过 {[l for l in c.stderr.splitlines() if l.startswith("error")][:1]}'); continue
    r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '1G', out_bin], capture_output=True, text=True, env=dict(os.environ, **{envname: srcfile}))
    fails = re.findall(r'^FAIL (\S+)', r.stdout, re.M)
    print(f'{name}: 红 {len(fails)} 条 {[x[:60] for x in fails]}' if fails else f'{name}: 全绿（活）' if 'PASS' in r.stdout else f'{name}: 无结果 rc={r.returncode} {r.stderr[-150:]}')
