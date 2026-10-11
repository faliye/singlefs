import sys,os,subprocess,shlex
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch52-54-code-r1-sonnet')
MUT={
'w1_no_strict_increase':("        judgements.judge(invariant, left_container < right_container, || {","        judgements.judge(invariant, true || left_container < right_container, || {"),
'w2_retirement_knob_ignored':("            retirement_tree.tree,\n            &containers\n                .iter()\n                .map(|container| (container.container, container.records.len()))\n                .collect::<Vec<(u64, usize)>>(),\n            retirement_capacity_knob,","            retirement_tree.tree,\n            &containers\n                .iter()\n                .map(|container| (container.container, container.records.len()))\n                .collect::<Vec<(u64, usize)>>(),\n            None,"),
'w3_lineage_product_is_959':("            LineageOrRetirementTree::Lineage => LINEAGE_CONTAINER_RECORDS,","            LineageOrRetirementTree::Lineage => RETIREMENT_CONTAINER_RECORDS,"),
'w3b_retirement_product_is_1359':("            LineageOrRetirementTree::Retirement => RETIREMENT_CONTAINER_RECORDS,","            LineageOrRetirementTree::Retirement => LINEAGE_CONTAINER_RECORDS,"),
'w4_lineage_gap_not_reported':("        Err(gap) => judgements.part_of_the_range_not_judged(\n            lineage_or_retirement_capacity_invariant_of(lineage_capacity_knob),\n            gap.reason(),\n        ),","        Err(_gap) => {}"),
'w5_retirement_gap_not_reported':("                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter().chain([\n                    lineage_or_retirement_capacity_invariant_of(retirement_capacity_knob),\n                ]) {","                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter() {"),
'w6_version_scan_gap_not_reported':("                .chain([capacity_invariant])\n                .chain(lineage_and_retirement_capacity_invariants)\n            {\n                judgements.part_of_the_range_not_judged(\n                    invariant,","                .chain([capacity_invariant])\n            {\n                judgements.part_of_the_range_not_judged(\n                    invariant,"),
'w7_rightmost_also_full':("                if is_the_rightmost {\n                    continue;\n                }\n                let product_records = tree_kind","                if false && is_the_rightmost {\n                    continue;\n                }\n                let product_records = tree_kind"),
'w8_knob_lineage_swapped_to_none':("    let lineage_capacity_knob = container_capacity_knobs\n        .lineage\n        .map(LineageContainerCapacityKnob::records_per_container);","    let lineage_capacity_knob: Option<u64> = None;"),
'w11_overfull_not_flagged':("judgements.judge(invariant, record_count == product_records, || {","judgements.judge(invariant, record_count >= product_records, || {"),
'w12_retirement_knob_plus_one':("            retirement_tree.tree,\n            &containers\n                .iter()\n                .map(|container| (container.container, container.records.len()))\n                .collect::<Vec<(u64, usize)>>(),\n            retirement_capacity_knob,","            retirement_tree.tree,\n            &containers\n                .iter()\n                .map(|container| (container.container, container.records.len()))\n                .collect::<Vec<(u64, usize)>>(),\n            retirement_capacity_knob.map(|knob| knob + 1),"),
'w9_threshold_one':("        None => 2,\n        Some(_) => 1,","        None => 2,\n        Some(_) => 2,"),
'w10_old_entry_gives_lineage_knob':("            lineage: None,\n            retirement: None,\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口","            lineage: LineageContainerCapacityKnob::new(1),\n            retirement: None,\n        },\n    )\n}\n\n/// 池级 checker 带三棵容器树旋钮值的入口"),
}
name=sys.argv[1]
o,r=MUT[name]
p=W+'/repo/crates/singlefs-checker/src/walk.rs'
base=open(W+'/pristine/walk_with_probe.rs',encoding='utf-8').read()
assert base.count(o)==1,(name,base.count(o))
open(p,'w',encoding='utf-8').write(base.replace(o,r))
try:
    env=dict(os.environ); env['CARGO_TARGET_DIR']=W+'/target'
    tests=sys.argv[2].split(',')
    c=['nice','-n','19','bash','/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh','10G','bash','/home/fy5090/code/singlefs/scripts/hosts/capped.sh','12','cargo','test','--offline','--no-fail-fast','-p','singlefs-harness']
    for t in tests: c+=['--test',t]
    c+=['--','--include-ignored']
    pr=subprocess.run(c,cwd=W+'/repo',env=env,capture_output=True,text=True,timeout=235)
    out=pr.stdout+pr.stderr
    print(name,'rc',pr.returncode)
    for l in out.splitlines():
        if l.startswith('test result') or l.endswith('FAILED') or l.startswith('error[') or l.startswith('error:') and 'could not' in l: print('  ',l[:220])
finally:
    open(p,'w',encoding='utf-8').write(base)
