import os, re, subprocess, sys
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet/scan')
R = '/home/fy5090/code/singlefs/scripts'
S = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/crates/singlefs-checker-tier/src/')
M = {
 'gpu': {
  'W1_keywords_list_without_use': ('"unsized", "use", "virtual"', '"unsized", "virtual"'),
  'W2_item_macro_prev_word_without_bracket': ('matches!(token, ";" | "\\u{7d}" | "]")', 'matches!(token, ";" | "\\u{7d}")'),
  'W3_item_macro_prev_word_any': ('token_before.is_none_or(|token| matches!(token, ";" | "\\u{7d}" | "]"))', 'true'),
  'W4_cfg_only_not_cfg_attr': ('matches!(**attribute, "cfg" | "cfg_attr")', 'matches!(**attribute, "cfg")'),
  'W5_use_before_lt_not_skipped': ('&& tokens.get(use_index + 1) != Some(&"<")', ''),
  'W6_scope_always_reaches': ('        let Some(enclosing_depth) = module_tokens[index].0.checked_sub(1) else {\n            return true;\n        };', '        return true;\n        let Some(enclosing_depth) = module_tokens[index].0.checked_sub(1) else {\n            return true;\n        };'),
  'W7_scope_never_reaches_in_blocks': ('        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| {', '        SIGNATURES_OF_THE_PINNED_BODIES.iter().any(|signature| { false &&'),
  'W8_openings_not_unique': ('            openings.len(),\n            1,\n            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的字节位置', '            openings.len().min(1),\n            1,\n            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的字节位置'),
  'W9_depth_zero_not_required_for_opening': ('            if depth == 0\n                && tokens[index..]', '            if tokens[index..]'),
  'W10_source_eq_name_always_ok': ('if matches!(name.as_str(), "std" | "wgpu") && source != name {', 'if matches!(name.as_str(), "std" | "wgpu") && false {'),
  'W11_glob_not_refused': ('BroughtInByAUse::Glob { source } => found.push(format!(', 'BroughtInByAUse::Glob { source } if false => found.push(format!('),
 },
 'crash': {
  'X1_shapes_without_fnonce': ('"FnOnce()->bool",\n', ''),
  'X2_shapes_without_fnptr': ('"fn()->bool",\n', ''),
  'X3_shapes_without_fnmut': ('"FnMut()->bool",\n', ''),
  'X4_shape_next_char_ident_not_skipped': ('                if code[position + shape_length..]\n                    .chars()\n                    .next()\n                    .is_some_and(is_identifier_character)\n                {\n                    continue;\n                }', ''),
  'X5_under_attribute_never': ('                let is_the_parameter_under_an_attribute =\n                    before_the_colon.is_some_and(', '                let is_the_parameter_under_an_attribute = false && \n                    before_the_colon.is_some_and('),
  'X6_openings_not_unique': ('            openings.len(),\n            1,\n            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的行号', '            openings.len().min(1),\n            1,\n            "代码里测试模块的头两行恰好一处（注释、字符串里的不算），读到的行号'),
  'X7_depth_zero_not_required': ('                pair[0].brace_depth_at_the_start == 0\n                    && pair[0].without_comments_and_literals.trim_end() == attribute_line', '                pair[0].without_comments_and_literals.trim_end() == attribute_line'),
  'X8_places_inside_pinned_type_not_excluded': ('                if places.iter().any(|place| {\n                    (place.start_of_the_type..place.end_of_the_type).contains(&position)\n                }) {\n                    continue;\n                }', ''),
 },
}
for kind, ms in M.items():
    model = open(f'{W}/build/{kind}-main.rs').read()
    srcfile = S + ('gpu_unit_checks.rs' if kind == 'gpu' else 'crash_amplification.rs')
    for name, (old, new) in ms.items():
        if model.count(old) != 1:
            print(f'{name}: 旧串命中 {model.count(old)} 次，跳过'); continue
        mp = f'{W}/replay/{kind}_n_{name}.rs'
        open(mp, 'w').write(model.replace(old, new, 1))
        out_bin = f'{W}/replay/{kind}_n_{name}'
        c = subprocess.run(['rustc', '--edition', '2021', '-O', '-o', out_bin, mp], capture_output=True, text=True)
        if c.returncode != 0:
            print(f'{name}: 编不过 {[l for l in c.stderr.splitlines() if l.startswith("error")][:1]}'); continue
        r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '1G', out_bin], capture_output=True, text=True, env=dict(os.environ, SRC=srcfile))
        fails = re.findall(r'^(\S+): RED', r.stdout, re.M)
        print(f'{name}: 红 {len(fails)} 条 {[x[:50] for x in fails[:2]]}' if fails else f'{name}: 全绿（活）')
