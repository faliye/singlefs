# 把第十八块 mutations-append 里 9 行（改动全文第 2178-2186 行）套到冻结副本源码上，喂对应的扫源码模型；再跑几种手造的绕过与 K5 写法
import os, sys
sys.argv = ['x', 'none']
exec(open(os.path.dirname(os.path.abspath(__file__)) + '/variants.py').read().split("which = sys.argv[1]")[0])
PIN = '    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);'
print('## 九行变异（实现员 mutations-append）')
rows = open('/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-pqrs-code-r1-diff.md').read().split('\n')[2177:2186]
for r in rows:
    f = r.split('\t')
    name, file, old, new = f[0], f[1], f[2].replace('\\n', '\n'), f[3].replace('\\n', '\n')
    base, binary, env = (gpu, 'scan-gpu', 'GPU_SRC') if 'gpu_unit_checks' in file else (crash, 'scan-crash', 'CRASH_SRC')
    if base.count(old) != 1:
        print(name[:50], '旧串命中', base.count(old)); continue
    p = f'{W}/variants/row_{abs(hash(name)) % 10**6}.rs'
    open(p, 'w').write(base.replace(old, new, 1))
    print(name[:70], '->', run(binary, env, p))
print('## 崩溃放量扫源码：手造的绕过')
print(cv('control_second_let', PIN, PIN + '\n    let stop_request = the_never_stopping_request();'))
print(cv('control_if_false_wrap', PIN, '    if false {\n' + PIN + '\n    }'))
print(cv('bypass_tuple_let_rebind', PIN, PIN + '\n    let (stop_request, _unused) = (the_never_stopping_request(), 0);'))
print(cv('bypass_let_extra_spaces', PIN, PIN + '\n    let   stop_request  =  the_never_stopping_request();'))
print(cv('bypass_nested_fn_item', PIN, PIN + '\n    fn the_never_stopping_request() -> u8 { 0 }'))
print(cv('bypass_extra_layer_one_line', 'fn check_selected_blocks(', 'fn an_extra_layer(a: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { let _ = stop_is_requested; a == 0 }\n\nfn check_selected_blocks('))
print(cv('bypass_extra_layer_other_name', 'fn check_selected_blocks(', 'fn an_extra_layer_other(a: u8, stop: &(dyn Fn() -> bool + Sync),) -> bool { let _ = stop; a == 0 }\n\nfn check_selected_blocks('))
