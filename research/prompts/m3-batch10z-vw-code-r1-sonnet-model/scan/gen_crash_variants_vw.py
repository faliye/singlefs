import os,re
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet/scan')
S=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs')
src=open(S).read()
D=W+'/crash-variants/'
for f in os.listdir(D): os.remove(D+f)
PIN='    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);\n'
assert src.count(PIN)==1
CHK='    check_blocks_where_until(\n        store,\n        flow,\n        plan,\n        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n        unit_checksums,\n        version,\n        stop_is_requested,\n    )\n}'
assert src.count(CHK)==1,src.count(CHK)
def v(name,old,new,cnt=1):
    assert src.count(old)==cnt,(name,src.count(old))
    open(D+name+'.rs','w').write(src.replace(old,new,1))
# 长的那一层
v('A01_cfg_any_pinned_line_with_static', PIN, '    #[cfg(any())]\n'+PIN)
open(D+'A01_cfg_any_pinned_line_with_static.rs','a').write('\n#[allow(non_upper_case_globals)]\nstatic stop_request: StopRequestNever = StopRequestNever;\n')
v('A02_match_arm_rebind', PIN, PIN+'    match never_stops() { stop_request => { let _ = stop_request; } }\n')
v('A03_closure_param_rebind', PIN, PIN+'    let _f = |stop_request: u8| stop_request;\n')
v('A04_for_rebind', PIN, PIN+'    for stop_request in 0..1 { let _ = stop_request; }\n')
v('A05_early_return_before_pin', PIN, '    if cards.is_empty() { return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { reason: String::new(), tally: CheckingTally::default() }); }\n'+PIN)
v('A06_tuple_let_rebind', PIN, PIN+'    let (stop_request, _unused) = (the_never_stopping_request(), 0);\n')
v('A07_let_extra_spaces', PIN, PIN+'    let   stop_request  =  the_never_stopping_request();\n')
v('A08_let_split_across_lines', PIN, PIN+'    let\n        stop_request\n        = the_never_stopping_request();\n')
v('A09_let_with_comment_between', PIN, PIN+'    let /* c */ stop_request = the_never_stopping_request();\n')
v('A10_if_let_rebind', PIN, PIN+'    if let Some(stop_request) = Some(the_never_stopping_request()) { let _ = stop_request; }\n')
v('A11_trailing_comment_on_pin', PIN, PIN.rstrip('\n')+' // why\n')
v('A12_string_with_let_in_body', PIN, PIN+'    let _note = "let stop_request = x";\n')
v('A13_raw_string_with_brace', PIN, PIN+'    let _raw = r#"}"#;\n')
v('A14_cr_raw_string_with_inner_quote', PIN, PIN+'    let _c = cr#"say "}" here"#;\n')
v('A15_if_false_block_comment_brace', PIN, PIN+'    if false { /* } */ let _x = 1; }\n')
v('A16_char_brace_literals', PIN, PIN+"    let _a = '{'; let _b = '}'; let _c = '\\u{7d}'; let _d = b'}';\n")
v('A17_second_pin_name_use_line', PIN, PIN+'    let _ = &stop_is_requested;\n')
v('A18_type_shadow_use_line', PIN, '    use something::Never as StopRequestRememberedInThisRun;\n'+PIN)
v('A19_stop_request_shadow_by_nested_fn_param', PIN, PIN+'    fn _inner(stop_request: u8) -> u8 { stop_request }\n')
v('A20_lifetime_labels', PIN, PIN+"    'outer: loop { break 'outer; }\n")
# 短的层
v('B01_dead_branch_copy', CHK, '    if false {\n        check_blocks_where_until(\n            store,\n            flow,\n            plan,\n            &|block_sequence| selected_block_sequences.contains(&block_sequence),\n            unit_checksums,\n            version,\n            stop_is_requested,\n        );\n    }\n'+CHK.replace('stop_is_requested,\n    )','&|| false,\n    )'))
v('B02_trailing_comment_in_short_body', CHK, CHK.replace('        version,\n','        version, // v\n'))
# 新层
NL1='\nfn an_extra_layer_never_asking(first_input: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { first_input == 0 }\n'
NL2='\nfn an_extra_layer(first_input: u8, stop: &(dyn Sync + Fn() -> bool)) -> bool { first_input == 0 }\n'
NL3='\nfn an_extra_layer(first_input: u8, stop: &dyn Fn() -> bool) -> bool { first_input == 0 }\n'
NL4='\nfn an_extra_layer(first_input: u8, stop: impl Fn() -> bool) -> bool { first_input == 0 }\n'
NL5='\nfn an_extra_layer<F: Fn() -> bool>(first_input: u8, stop: &F) -> bool { first_input == 0 }\n'
NL6='\ntype StopFn<\'a> = &\'a (dyn Fn() -> bool + Sync);\nfn an_extra_layer(first_input: u8, stop: StopFn<\'_>) -> bool { first_input == 0 }\n'
NL7='\nfn an_extra_layer(first_input: u8, stop: &(dyn Fn() /* c */ -> bool + Sync)) -> bool { first_input == 0 }\n'
NL8='\nfn an_extra_layer(first_input: u8, stop: &(dyn Fn() -> bool + Sync + \'static)) -> bool { first_input == 0 }\n'
NL9='\nfn an_extra_layer(first_input: u8, stop: &mut (dyn Fn() -> bool + Sync)) -> bool { first_input == 0 }\n'
NL10='\nfn an_extra_layer(first_input: u8, stop: &(dyn Fn() -> bool + Send + Sync)) -> bool { first_input == 0 }\n'
NL11='\nfn an_extra_layer(first_input: u8, stop: &\'static (dyn Fn() -> bool + Sync)) -> bool { first_input == 0 }\n'
NL12='\nfn an_extra_layer(first_input: u8, stop: Box<dyn Fn() -> bool + Sync>) -> bool { first_input == 0 }\n'
NL13='\nfn an_extra_layer(f: fn(u8) -> u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { f(0) == 0 }\n'
for i,(n,t) in enumerate([('C01_new_layer_one_line',NL1),('C02_new_layer_sync_first',NL2),('C03_new_layer_dyn_fn_ref',NL3),('C04_new_layer_impl_fn',NL4),('C05_new_layer_generic',NL5),('C06_new_layer_type_alias',NL6),('C07_new_layer_comment_in_type',NL7),('C08_new_layer_static_bound',NL8),('C09_new_layer_mut_ref',NL9),('C10_new_layer_send_sync',NL10),('C11_new_layer_static_ref',NL11),('C12_new_layer_box',NL12),('C13_new_layer_fn_pointer_param',NL13)]):
    open(D+n+'.rs','w').write(src+t)
# 空白换行的类型 (同一层，合法重排)
open(D+'D01_type_reflowed_in_existing_layer.rs','w').write(src.replace('    stop_is_requested: &(dyn Fn() -> bool + Sync),\n) -> Result<CheckingTally, VerdictStoreError> {\n    check_blocks_where_until(','    stop_is_requested: &(\n        dyn Fn() -> bool + Sync\n    ),\n) -> Result<CheckingTally, VerdictStoreError> {\n    check_blocks_where_until(',1))


# ---------- 本轮新增 ----------
NL='\nfn an_extra_layer(first_input: u8, stop: %s) -> bool { first_input == 0 }\n'
for name, ty in [
  ('N01_return_type_path_core_primitive', '&(dyn Fn() -> ::core::primitive::bool + Sync)'),
  ('N02_return_type_in_parens', '&(dyn Fn() -> (bool) + Sync)'),
  ('N03_return_type_via_alias', '&(dyn Fn() -> StopBool + Sync)'),
  ('N04_hrtb', '&(dyn for<\'a> Fn() -> bool + Sync)'),
  ('N05_fn_with_arg_not_a_stop', '&(dyn Fn(u8) -> bool + Sync)'),
  ('N06_atomic_bool_param', '&std::sync::atomic::AtomicBool'),
  ('N07_arc_dyn_fn', 'std::sync::Arc<dyn Fn() -> bool + Send + Sync>'),
  ('N08_fnmut_ref', '&mut dyn FnMut() -> bool'),
  ('N09_fn_ptr', 'fn() -> bool'),
  ('N10_impl_fnonce', 'impl FnOnce() -> bool'),
  ('N11_tuple_of_stops', '(&(dyn Fn() -> bool + Sync), u8)'),
  ('N12_option_stop', 'Option<&(dyn Fn() -> bool + Sync)>'),
  ('N13_closure_trait_via_path', '&(dyn core::ops::Fn() -> bool + Sync)'),
  ('N14_fn_with_unit_default_ret_via_spaces', '&(dyn Fn(  )  ->  bool + Sync)'),
  ('N15_dyn_fn_ret_bool_type_macro', '&(dyn Fn() -> bool_of!() + Sync)'),
]:
    open(D+name+'.rs','w').write(src+(NL % ty)+ ('\ntype StopBool = bool;\n' if 'StopBool' in ty else ''))
# 切法与隐藏（崩溃放量一侧）
OPEN='#[cfg(test)]\nmod tests {'
assert src.count(OPEN)==1
def w(name, text): open(D+name+'.rs','w').write(text)
pre, post = src.split(OPEN)
HID='\nfn hidden_layer(stop: &(dyn Fn() -> bool + Sync)) -> bool { stop() }\n'
w('S01_block_comment_opening_before_a_layer', pre+'/* '+OPEN+' */'+HID+OPEN+post)
w('S02_line_comment_opening', pre+'// #[cfg(test)]\n// mod tests {\n'+HID+OPEN+post)
w('S03_raw_string_opening', pre+'const _O: &str = r#"'+OPEN+'"#;\n'+HID+OPEN+post)
w('S04_two_real_openings', src+'\n'+OPEN+'\n}\n')
w('S05_attribute_between_opening', pre+'#[cfg(test)]\n#[allow(unused)]\nmod tests {'+post)
w('S06_mod_tests_without_space', pre+'#[cfg(test)]\nmod tests{'+post)
w('S07_one_line_opening', pre+'#[cfg(test)] mod tests {'+post)
w('S08_crlf_everywhere', src.replace('\n','\r\n'))
w('S09_hidden_layer_after_tests', src+HID)
w('S10_hidden_layer_before_tests', pre+HID+OPEN+post)
w('S11_opening_inside_parenthesised_macro_then_real', pre+'macro_rules! wrap ( () => ( '+OPEN.replace('{','')+' ) );\n'+HID+OPEN+post)
w('S12_extra_spaces_in_opening', pre+'#[cfg(test)]\nmod  tests  {'+post)
w('S13_trailing_spaces_in_opening', pre+'#[cfg(test)]  \nmod tests {  '+post)
print(len(os.listdir(D)))
