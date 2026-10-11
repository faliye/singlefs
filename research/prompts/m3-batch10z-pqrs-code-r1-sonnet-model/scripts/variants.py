import os, subprocess, sys
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet')
FR = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/crates/singlefs-checker-tier/src/')
R = '/home/fy5090/code/singlefs/scripts'
gpu = open(FR + 'gpu_unit_checks.rs').read()
crash = open(FR + 'crash_amplification.rs').read()
def run(binary, envname, path):
    r = subprocess.run(['bash', R + '/run-with-memory-cap.sh', '1G', W + '/target-scan/release/' + binary], env=dict(os.environ, **{envname: path}), capture_output=True, text=True)
    return (r.stdout.strip() or r.stderr.strip()[-120:])
def gv(name, old, new, count=1):
    assert gpu.count(old) >= 1, (name, 'old not found')
    src = gpu.replace(old, new, 1)
    p = f'{W}/variants/g_{name}.rs'
    open(p, 'w').write(src)
    return f'{name}: {run("scan-gpu", "GPU_SRC", p)}'
def cv(name, old, new):
    assert crash.count(old) >= 1, (name, 'old not found')
    src = crash.replace(old, new, 1)
    p = f'{W}/variants/c_{name}.rs'
    open(p, 'w').write(src)
    return f'{name}: {run("scan-crash", "CRASH_SRC", p)}'
POOL_OPEN = '    mod card_pool {\n        use super::super::{'
OUT = []
which = sys.argv[1]
if which == 'gpu':
    OUT += [gv('control_mod_wgpu', POOL_OPEN, '    mod card_pool {\n        mod wgpu { pub use ::wgpu::*; }\n        use super::super::{')]
    OUT += [gv('K5_use_as', POOL_OPEN, '    mod card_pool {\n        use std::sync::Arc as SharedArc;\n        use super::super::{')]
    OUT += [gv('K5_nested_use_braces', POOL_OPEN, '    mod card_pool {\n        use std::collections::{BTreeMap, btree_map::{Entry}};\n        use super::super::{')]
    OUT += [gv('K5_thread_local_macro', POOL_OPEN, '    mod card_pool {\n        thread_local! { static COUNT: std::cell::Cell<u8> = std::cell::Cell::new(0); }\n        use super::super::{')]
    OUT += [gv('K5_const_ne', POOL_OPEN, '    mod card_pool {\n        const TWO_DIFFER: bool = 1 != 2;\n        use super::super::{')]
    OUT += [gv('K5_const_ne_ident', POOL_OPEN, '    mod card_pool {\n        const A: u8 = 1;\n        const B: u8 = 2;\n        const TWO_DIFFER: bool = A != B;\n        use super::super::{')]
    OUT += [gv('K5_lifetime_char', POOL_OPEN, "    mod card_pool {\n        fn pick<'a>(x: &'a str) -> &'a str { let _c = '{'; let _d = '\\u{7b}'; let _e = 'é'; x }\n        use super::super::{")]
    OUT += [gv('K5_raw_byte_strings', POOL_OPEN, '    mod card_pool {\n        const R: &str = r#"mod x { extern crate y; }"#;\n        const B: &[u8] = b"mod \\" y";\n        use super::super::{')]
    OUT += [gv('K5_block_comment_nested', POOL_OPEN, '    mod card_pool {\n        /* outer /* inner mod std { */ still comment */\n        use super::super::{')]
    OUT += [gv('K5_cfg_test_mod_inside', POOL_OPEN, '    mod card_pool {\n        #[cfg(test)]\n        mod unit { }\n        use super::super::{')]
    OUT += [gv('K5_extern_c_block', POOL_OPEN, '    mod card_pool {\n        extern "C" { fn abs(x: i32) -> i32; }\n        use super::super::{')]
    OUT += [gv('K5_pub_use_self_group', POOL_OPEN, '    mod card_pool {\n        use std::sync::{self, Arc};\n        use super::super::{')]
    OUT += [gv('K5_use_wildcard_enum', POOL_OPEN, '    mod card_pool {\n        use std::cmp::Ordering::*;\n        use super::super::{')]
    OUT += [gv('bypass_use_in_fn_as', POOL_OPEN, '    mod card_pool {\n        fn helper() { use std::sync::Arc as A; let _ = 0; }\n        use super::super::{')]
    OUT += [gv('bypass_type_named_std', POOL_OPEN, '    mod card_pool {\n        #[allow(non_camel_case_types)] struct std;\n        use super::super::{')]
    OUT += [gv('bypass_trait_named_wgpu', POOL_OPEN, '    mod card_pool {\n        #[allow(non_camel_case_types)] trait wgpu {}\n        use super::super::{')]
    OUT += [gv('bypass_const_named_wgpu', POOL_OPEN, '    mod card_pool {\n        #[allow(non_upper_case_globals)] const wgpu: u8 = 0;\n        use super::super::{')]
    OUT += [gv('bypass_enum_named_std', POOL_OPEN, '    mod card_pool {\n        #[allow(non_camel_case_types)] enum std { panic }\n        use super::super::{')]
if which == 'crash':
    PIN = '    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);'
    OUT += [cv('control_never_stop_after', PIN, PIN + '\n    let never_stop = || false;\n    let stop_request = StopRequestRememberedInThisRun::new(&never_stop);')]
    OUT += [cv('bypass_extra_layer_one_line_signature', '\n/// 这份源码里收叫停参数', '\nfn an_extra_layer(a: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool {\n    let _ = stop_is_requested;\n    a == 0\n}\n\n/// 这份源码里收叫停参数') if False else '(skipped)']
    OUT += [cv('bypass_extra_layer_one_line', 'fn check_selected_blocks(', 'fn an_extra_layer(a: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { let _ = stop_is_requested; a == 0 }\n\nfn check_selected_blocks(')]
    OUT += [cv('bypass_extra_layer_other_name', 'fn check_selected_blocks(', 'fn an_extra_layer_other(a: u8, stop: &(dyn Fn() -> bool + Sync),) -> bool { let _ = stop; a == 0 }\n\nfn check_selected_blocks(')]
print('\n'.join(OUT))
