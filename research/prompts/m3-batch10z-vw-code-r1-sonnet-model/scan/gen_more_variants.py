#!/usr/bin/env python3
"""gen_more_variants.py：生成本轮 K4 的整份源文件变体里 gen_crash_variants_vw.py 之外的几组（S14 到 S19、G01 到 G17）。
读冻结副本的 gpu_unit_checks.rs、crash_amplification.rs，写到 scan/crash-variants/、scan/gpu-variants/。"""
import os
SRC = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/crates/singlefs-checker-tier/src/')
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet/scan/')
os.makedirs(W + 'crash-variants', exist_ok=True); os.makedirs(W + 'gpu-variants', exist_ok=True)
# ---- 崩溃放量
lines = open(SRC + 'crash_amplification.rs').read().split('\n')
assert lines[3573] == '#[cfg(test)]' and lines[3574] == 'mod tests {' and lines[7812] == '}'
D = W + 'crash-variants/'
HID = ['    fn hidden_layer(stop: &(dyn Fn() -> bool + Sync)) -> bool { stop() }']
before, mid, after = lines[:3573], lines[3573:7812], lines[7813:]
mid2 = mid[:2] + HID + mid[2:]
macro = ['macro_rules! keep_hidden { (#[cfg(test)] $($rest:tt)*) => { $($rest)* }; }']
w = lambda n, t: open(D + n + '.rs', 'w').write('\n'.join(t))
w('S14_tests_wrapped_in_a_cfg_stripping_macro_hiding_a_layer', before + macro + ['keep_hidden!('] + mid2 + ['}', ');'] + after)
w('S15_layer_inside_the_real_tests_module_control', before + mid2 + ['}'] + after)
w('S16_layer_outside_control', before + HID + mid + ['}'] + after)
fake = ['#[cfg(test)]', 'mod tests {', '}']
w('S17_fake_empty_tests_module_then_hidden_layer_then_real_tests', before + fake + HID + lines[3573:])
mac = ['macro_rules! decoy {', '    () => {', '#[cfg(test)]', 'mod tests {', '}', '    };', '}']
w('S18_opening_copy_inside_a_macro_rules_body_hiding_nothing', before + mac + HID + lines[3573:])
w('S19_macro_wrapped_fake_tests_hiding_a_layer_and_the_real_tests_intact', before + macro + ['keep_hidden!(', '#[cfg(test)]', 'mod tests {'] + HID + ['}', ');'] + lines[3573:])
# ---- 卡池
src = open(SRC + 'gpu_unit_checks.rs').read()
D = W + 'gpu-variants/'
OPEN = '#[cfg(test)]\nmod tests {'
pre, post = src.split(OPEN)
POOL = '    mod card_pool {\n'
wr = lambda n, t: open(D + n + '.rs', 'w').write(t)
EVIL = '        use other::wgpu;\n'
wr('G01_evil_after_block_comment_opening_in_pool', src.replace(POOL, POOL + '        /* ' + OPEN + ' */\n' + EVIL))
wr('G02_evil_after_line_comment_opening', src.replace(POOL, POOL + '        // #[cfg(test)]\n        // mod tests {\n' + EVIL))
wr('G03_evil_after_string_opening', src.replace(POOL, POOL + '        const _O: &str = "#[cfg(test)]\\nmod tests {";\n' + EVIL))
wr('G04_evil_without_opening', src.replace(POOL, POOL + EVIL))
wr('G05_two_real_openings', src + '\n' + OPEN + '\n}\n')
wr('G06_attribute_between', pre + '#[cfg(test)]\n#[allow(unused)]\nmod tests {' + post)
wr('G07_mod_tests_without_space', pre + '#[cfg(test)]\nmod tests{' + post)
wr('G08_one_line_opening', pre + '#[cfg(test)] mod tests {' + post)
wr('G09_crlf', src.replace('\n', '\r\n'))
wr('G10_evil_after_tests', src + '\nfn after() { use core as std; }\n')
wr('G11_spaces_in_attribute', pre + '#[ cfg ( test ) ]\nmod tests {' + post)
wr('G12_extra_spaces_mod', pre + '#[cfg(test)]\nmod  tests  {' + post)
wr('G13_opening_inside_parens_macro', src.replace(POOL, POOL + '        macro_call!(\n        ' + OPEN.replace('{', '') + '\n        );\n' + EVIL))
sig = '        impl Drop for GpuCardLease {\n'
i = src.index(sig); j = src.index('\n        }\n', i) + len('\n        }\n')
block = src[i:j]
evil = sig + '            fn drop(&mut self) {}\n        }\n'
wr('G14_decoy_cfg_any_drop_then_evil_drop', src[:i] + '        #[cfg(any())]\n' + block + evil + src[j:])
wr('G15_evil_drop_only_control', src[:i] + evil + src[j:])
sig = 'pub fn lease_the_first_card_with_room_now('
i = src.index(sig); ls = src.rfind('\n', 0, i) + 1
j = src.index('\n            }\n', i) + len('\n            }\n')
block = src[ls:j]
line = '                self.take_in_the_devices_found_idle();\n'
evil = block.replace(line, '')
wr('G16_decoy_cfg_any_entry_then_evil_entry', src[:ls] + '            #[cfg(any())]\n' + block + evil + src[j:])
wr('G17_evil_entry_only_control', src[:ls] + evil + src[j:])
print('crash', len(os.listdir(W + 'crash-variants')), 'gpu', len(os.listdir(W + 'gpu-variants')))
