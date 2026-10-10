#!/usr/bin/env python3
"""gen_extra_variants.py：这一轮在 gen_crash_variants_vw.py / gen_more_variants.py 之外加的整份源文件变体
（崩溃放量 S14b–S14f、N16–N24；卡池 S14、S14b–S14f、G14b–G14e、G16b–G16g、P01–P04），写到 scan/crash-variants/、scan/gpu-variants/。
源文件取冻结副本；W 换成自己的草稿目录。"""
import os
SRC = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree/crates/singlefs-checker-tier/src/')
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet/scan/')
os.makedirs(W + 'crash-variants', exist_ok=True); os.makedirs(W + 'gpu-variants', exist_ok=True)
# ---- 崩溃放量
lines = open(SRC + 'crash_amplification.rs').read().split('\n')
I0 = [i for i in range(len(lines) - 1) if lines[i] == '#[cfg(test)]' and lines[i + 1] == 'mod tests {'][0]
I1 = max(i for i, l in enumerate(lines) if l == '}')
before, mid, after = lines[:I0], lines[I0:I1], lines[I1 + 1:]
HID = ['    pub fn hidden_layer(stop: &(dyn Fn() -> bool + Sync)) -> bool { stop() }']
mid2 = mid[:2] + HID + mid[2:]
macro = ['macro_rules! keep_hidden { ($($rest:tt)*) => { $($rest)* }; }']
C = 'const BEFORE_THE_TESTS: u8 = 0;'
D = W + 'crash-variants/'
w = lambda n, t: open(D + n + '.rs', 'w').write('\n'.join(t))
w('S14b_tests_wrapped_after_an_item_ending_in_a_semicolon', before + macro + ['keep_hidden!(', C] + mid2 + ['}', ');'] + after)
w('S14c_tests_wrapped_in_braces_after_an_item_ending_in_a_semicolon', before + macro + ['keep_hidden! {', C] + mid2 + ['}', '}'] + after)
w('S14d_tests_wrapped_in_brackets_after_an_item_ending_in_a_semicolon', before + macro + ['keep_hidden![', C] + mid2 + ['}', '];'] + after)
w('S14e_tests_wrapped_in_brackets_no_item_before', before + macro + ['keep_hidden!['] + mid2 + ['}', '];'] + after)
w('S14f_tests_after_a_paren_group_that_ends_the_previous_item_without_semicolon', before + ['static NOT_ENDED_BY_A_SEMICOLON: u8 = (0)'] + mid2 + ['}'] + after)
src = '\n'.join(lines)
NL = '\nfn an_extra_layer(first_input: u8, stop: %s) -> bool { first_input == 0 }\n'
for name, ty in [
    ('N16_return_tuple_bool_u8_not_a_stop', '&(dyn Fn() -> (bool, u8) + Sync)'),
    ('N17_return_tuple_u8_bool_not_a_stop', '&(dyn Fn() -> (u8, bool) + Sync)'),
    ('N18_return_boolean_named_not_a_stop', '&(dyn Fn() -> Boolean + Sync)'),
    ('N19_return_option_bool_not_a_stop', '&(dyn Fn() -> Option<bool> + Sync)'),
    ('N20_return_double_parens', '&(dyn Fn() -> ((bool)) + Sync)'),
    ('N21_return_core_path_with_spaces', '&(dyn Fn() -> core :: primitive :: bool + Sync)'),
    ('N22_return_crate_alias_path', '&(dyn Fn() -> crate::StopBool + Sync)'),
    ('N23_return_bool_ref_not_a_stop', '&(dyn Fn() -> &bool + Sync)'),
    ('N24_return_bool_array_not_a_stop', '&(dyn Fn() -> [bool; 2] + Sync)'),
]:
    open(D + name + '.rs', 'w').write(src + (NL % ty))
# ---- 卡池
s = open(SRC + 'gpu_unit_checks.rs').read()
src_lines = s.split('\n')
J0 = [i for i in range(len(src_lines) - 1) if src_lines[i] == '#[cfg(test)]' and src_lines[i + 1] == 'mod tests {'][0]
gb, gm = src_lines[:J0], src_lines[J0:]
gm2 = gm[:2] + ['    pub fn hidden_layer() {}'] + gm[2:]
D = W + 'gpu-variants/'
w = lambda n, t: open(D + n + '.rs', 'w').write('\n'.join(t))
w('S14_gpu_tests_wrapped_in_a_cfg_stripping_macro', gb + macro + ['keep_hidden!('] + gm2 + [');'])
w('S14b_gpu_tests_wrapped_after_an_item_ending_in_a_semicolon', gb + macro + ['keep_hidden!(', C] + gm2 + [');'])
w('S14c_gpu_tests_wrapped_in_braces_after_an_item_ending_in_a_semicolon', gb + macro + ['keep_hidden! {', C] + gm2 + ['}'])
w('S14d_gpu_tests_wrapped_in_brackets_after_an_item_ending_in_a_semicolon', gb + macro + ['keep_hidden![', C] + gm2 + ['];'])
w('S14f_gpu_tests_after_an_item_not_ended_by_a_semicolon', gb + ['static NOT_ENDED_BY_A_SEMICOLON: u8 = (0)'] + gm2)
J = len('\n'.join(src_lines[:J0]))
def once(sig, rep):
    i = s.index(sig); assert i < J
    return s[:i] + rep + s[i + len(sig):]
wr = lambda n, t: open(D + n + '.rs', 'w').write(t)
sig = '        impl Drop for GpuCardLease {\n'
i = s.index(sig); j = s.index('\n        }\n', i) + len('\n        }\n')
block = s[i:j]; evil = sig + '            fn drop(&mut self) {}\n        }\n'
wr('G14b_decoy_cfg_any_drop_then_evil_drop_through_a_path', s[:i] + '        #[cfg(any())]\n' + block + evil.replace('impl Drop for GpuCardLease', 'impl Drop for self::GpuCardLease') + s[j:])
wr('G14c_decoy_in_a_macro_call_expanding_to_nothing_then_evil_drop', s[:i] + '        fn a_place_for_the_clean_copy() {\n            expands_to_nothing! {\n' + block + '            }\n        }\n' + evil + s[j:])
wr('G14d_single_drop_under_cfg_any', s[:i] + '        #[cfg(any())]\n' + block + s[j:])
wr('G14e_decoy_cfg_attr_drop_then_evil_drop', s[:i] + '        #[cfg_attr(test, allow(unused))]\n' + block + evil + s[j:])
wr('G14h_sole_drop_written_through_a_path', once(sig, '        impl Drop for self::GpuCardLease {\n'))
wr('G14i_sole_drop_with_lifetime_generics_legit', once(sig, "        impl<'a> Drop for GpuCardLease<'a> {\n"))
sg = 'pub fn lease_the_first_card_with_room_now('
i = s.index(sg); ls = s.rfind('\n', 0, i) + 1; j = s.index('\n            }\n', i) + len('\n            }\n')
blk = s[ls:j]; evil2 = blk.replace('                self.take_in_the_devices_found_idle();\n', '')
wr('G16b_decoy_cfg_any_entry_then_evil_pub_crate_entry', s[:ls] + '            #[cfg(any())]\n' + blk + evil2.replace(sg, 'pub(crate) fn lease_the_first_card_with_room_now(') + s[j:])
wr('G16c_decoy_entry_in_unsafe_wrapper_then_evil_entry', s[:ls] + '            #[cfg(any())]\n' + blk.replace('pub fn lease_the_first', 'pub unsafe fn lease_the_first') + evil2 + s[j:])
for n, sig2 in [('G16d_sole_pub_entry_under_cfg_any', '            pub fn lease_the_first_card_with_room_now('), ('G16e_sole_plain_entry_under_cfg_any', '            fn lease_card('), ('G16f_sole_pub_entry_under_cfg_any_with_doc_between', '            pub fn book_the_scratch_of_the_cards_with_room_now(')]:
    wr(n, once(sig2, ('            #[cfg(any())]\n' + sig2) if 'doc_between' not in n else ('            #[cfg(any())]\n            /// doc\n' + sig2)))
sig2 = '            pub fn card_rooms_now_and_when_empty_of('
wr('G16g_sole_pub_entry_under_cfg_attr', once(sig2, '            #[cfg_attr(not(test), cfg(any()))]\n' + sig2))
sig = '    mod card_pool {\n'
wr('P01_pool_module_with_pub_legit', once(sig, '    pub mod card_pool {\n'))
wr('P02_pool_module_cfg_any_before_pub', once(sig, '    #[cfg(any())]\n    pub mod card_pool {\n'))
wr('P03_pool_module_cfg_any_no_pub', once(sig, '    #[cfg(any())]\n    mod card_pool {\n'))
wr('P04_pool_module_pub_crate_cfg_feature', once(sig, '    #[cfg(feature = "gpu")]\n    pub(crate) mod card_pool {\n'))
print('ok')
