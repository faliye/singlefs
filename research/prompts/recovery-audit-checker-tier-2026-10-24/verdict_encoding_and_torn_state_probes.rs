#[path="__COMMON_FIXTURE_PATH__"]
mod common;
use singlefs_checker_tier::crash_given_states::{GivenStateConclusion,GivenStatesJudge,GivenStatesOfATable};
use singlefs_checker_tier::crash_amplification::{CrashAmplificationEnvironment,GivenStatesEngine,GivenStatesItem};
use singlefs_checker_tier::crash_library::{DisposableLibraryDirectory,CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
struct HeldCoverage { invariants_held_at_least_once: std::collections::BTreeSet<&'static str> }
impl HeldCoverage { __COVERAGE_METHOD__ }
fn verify_engine_cache_coverage() {
    let disposable=DisposableLibraryDirectory::new("audit-coverage-recheck").unwrap();
    let configuration=disposable.directory().join("cpu.env");
    std::fs::write(&configuration,"ENABLE_GPU=0\n").unwrap();
    let library=disposable.library();
    let _environment=CrashAmplificationEnvironment::cleared_then_set(&[
        ("SINGLEFS_MULTI_HOST_CONFIG",configuration.as_os_str()),
        (CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,library.as_os_str()),
    ]);
    let built=common::build_pool("audit-coverage-recheck");
    let base=built.memory_pool_after_mkfs();
    let operations=built.retained_operations();
    let (writes,segments)=writes_and_segments(&operations[built.mkfs_operation_count..],&common::geometry());
    let table=GivenStatesOfATable{table_name:"coverage-control".to_string(),base:&base,writes:&writes,segments:&segments,seams:&[],persisted_sets:vec![vec![true;writes.len()]]};
    let engine=GivenStatesEngine::start_from_environment(GivenStatesItem{item_name:"audit-cache-coverage".to_string()},&CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test()).unwrap();
    let cold=engine.judge_the_given_states(&table).unwrap();
    let warm=engine.judge_the_given_states(&table).unwrap();
    assert_eq!(cold.count_vector,warm.count_vector);
    assert_eq!(cold.states[0].red_items(),warm.states[0].red_items());
    let mut cold_coverage=HeldCoverage{invariants_held_at_least_once:Default::default()};
    let mut warm_coverage=HeldCoverage{invariants_held_at_least_once:Default::default()};
    cold_coverage.note_the_invariants_held(&cold.states[0]);
    warm_coverage.note_the_invariants_held(&warm.states[0]);
    for inv in ["I-3.1","I-5.4","I-3.11","I-9.15"] {
        assert!(cold_coverage.invariants_held_at_least_once.contains(inv),"cold missing {inv}");
        assert!(!warm_coverage.invariants_held_at_least_once.contains(inv),"warm has {inv}");
    }
    let run=engine.finish();
    assert_eq!(run.tally.tables_judged_here,1);assert_eq!(run.tally.tables_reused,1);
    println!("RECHECK actual engine cold judged=1 warm reused=1; count_vector unchanged; actual test coverage method cold_held={} warm_held={}",cold_coverage.invariants_held_at_least_once.len(),warm_coverage.invariants_held_at_least_once.len());
}
use singlefs_checker_tier::crash_judge_conclusions::*;
use singlefs_checker_tier::crash::*;
use singlefs_checker_tier::crash_facts::*;
use singlefs_core::recovery::{RecoveryReport,RecoveryOutcome,JournalScanReport,PoolReader};
use singlefs_core::address::*;
use singlefs_harness::memory_pool::*;
use singlefs_harness::segments::StepKind;
use singlefs_checker::image::InvariantVerdict;
fn report(txg:u64,content:Vec<u8>)->RecoveryReport{
    let root=(InstanceGeneration(1),CheckpointTxg(txg));
    RecoveryReport{outcome:RecoveryOutcome::FileRead{root,content},effective_root:Some(root),journal:JournalScanReport::default(),mapping_fallbacks:0,devices_without_a_mountable_system_configuration_slot:vec![]}
}
fn words(r:&RecoveryReport,checker:&[(&'static str,InvariantVerdict)])->CrossCheckWords{
    cross_check_words_of_a_given_state(r,r,checker,RecordCheck::default(),ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery)
}
fn write(v:u8)->RetainedWrite{
    RetainedWrite{device:DeviceIdentity(0),offset:DeviceOffsetInBytes(0),kind:StepKind::SystemConfigurationSlot,is_force_unit_access:false,contents:WrittenContents::Bytes(vec![v;4096])}
}
fn main(){
    verify_engine_cache_coverage();
    for low in [0u64,1,7,u32::MAX as u64] {
        for high in [1u64,2,0xFFFF_FFFF] {
            let a=report(low,vec![1]); let b=report((high<<32)|low,vec![1]);
            assert_ne!(a.effective_root,b.effective_root);
            assert_eq!(words(&a,&[]),words(&b,&[]));
            assert!(cross_check_words_agree_in_the_cross_check(&words(&a,&[]),&words(&b,&[])));
            assert!(!recovery_passes_differ(&a,&b));
            assert_eq!(recovery_words_of_the_cross_check_words(&words(&a,&[])),recovery_words_of_the_cross_check_words(&words(&b,&[])));
        }
    }
    assert_ne!(words(&report(7,vec![1]),&[]),words(&report(8,vec![1]),&[]));
    assert!(!cross_check_words_agree_in_the_cross_check(&words(&report(7,vec![1]),&[]),&words(&report(8,vec![1]),&[])));
    println!("RECHECK txg: 12 different-u64 pairs alias; different-low control distinguishes 7 and 8");
    let a=report(7,vec![1]); let b=report((1u64<<32)+7,vec![1]);
    assert_eq!(words(&a,&[]),words(&b,&[])); assert!(!recovery_passes_differ(&a,&b));
    println!("CONFIRMED txg=7 and txg=4294967303 compare equal; journal_differing=false");
    let f1=singlefs_core::recovery::RecoveryFailure::InvariantViolated{invariant:"I-1.2",detail:"birth mismatch"};
    let f2=singlefs_core::recovery::RecoveryFailure::InvariantViolated{invariant:"I-1.3",detail:"tree mismatch"};
    assert_eq!(gpu_failure_point_code(&f1),gpu_failure_point_code(&f2));
    assert_ne!(gpu_failure_point_code(&f1),gpu_failure_point_code(&singlefs_core::recovery::RecoveryFailure::NoValidRoot));
    println!("CONTROL failure kind distinguishes InvariantViolated and NoValidRoot; semantic subcauses intentionally collapsed");
    println!("OBSERVED documented recovery failures I-1.2 and I-1.3 have identical comparison code");
    let c=report(7,vec![99]); assert_eq!(words(&a,&[]),words(&c,&[]));
    println!("OBSERVED scoped same-root content [1] vs [99] compares equal for given states");
    let versions=[PublishedVersion{instance:InstanceGeneration(1),checkpoint_txg:CheckpointTxg(7),content:vec![1]}];
    assert!(classified_oracle_violation_for_versions(&a.outcome,a.effective_root,None,&versions).is_none());
    assert!(classified_oracle_violation_for_versions(&c.outcome,c.effective_root,None,&versions).is_some());
    println!("CONTROL full content oracle accepts [1] and rejects [99]; scoped given-state words do not compare content");
    let inv=singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX;
    let original=words(&a,&[(inv,InvariantVerdict::Holds)]);
    let stored=cross_check_words_of_a_stored_given_state(&red_items_of_a_given_state(&original),recovery_words_of_the_cross_check_words(&original)).unwrap();
    assert_ne!(original,stored); assert_eq!(stored.pool_checker_judged_bits,0);
    println!("OBSERVED documented cache roundtrip judged_bits {} -> {}, scan_outcome {} -> {}",original.pool_checker_judged_bits,stored.pool_checker_judged_bits,original.scan_rebuild_outcome,stored.scan_rebuild_outcome);
    let red=words(&a,&[(inv,InvariantVerdict::Violated("audit".to_string()))]);
    let red_read=cross_check_words_of_a_stored_given_state(&red_items_of_a_given_state(&red),recovery_words_of_the_cross_check_words(&red)).unwrap();
    assert_eq!(red.pool_checker_violated_bits,red_read.pool_checker_violated_bits);
    assert_eq!(red_items_of_a_given_state(&red),red_items_of_a_given_state(&red_read));
    println!("CONTROL cached red items and violated bits preserved; per-state judged bits are documented as omitted");
    let mut base=MemoryPool::with_devices(&[DeviceIdentity(0)],1<<20);
    base.apply_writes(&[write(0x11)]);
    let writes=vec![write(0x22),write(0x33)]; let segments=vec![vec![0,1]];
    let tables=crash_state_enumeration_tables(&base,&writes,&segments,&full_expansion);
    let facts=extract_flow_facts(&base,&writes,&segments,&full_expansion);
    let mut found=false; let mut covers_physical=false;
    let physical:Vec<u8>=[vec![0x33;2048],vec![0x11;2048]].concat();
    for ordinal in 0..facts.state_count{
        let mask=facts.persisted_writes_of_state(ordinal);
        let image=CrashImage{base:&base,writes:&tables.enumerated_writes,persisted:mask.clone()};
        let bytes=PoolReader::read(&image,DeviceIdentity(0),DeviceOffsetInBytes(0),4096).unwrap();
        covers_physical|=bytes==physical;
        let torn=tables.torn_images.iter().find(|t|t.write_index==1).unwrap().torn_image_index;
        let w0_torn=tables.torn_images.iter().find(|t|t.write_index==0).unwrap().torn_image_index;
        if !mask[0] && !mask[w0_torn] && !mask[1] && mask[torn]{
            assert_eq!(&bytes[2048..],&[0x22;2048]);found=true;
            println!("OBSERVED W0 original absent, W1 torn ordinal={ordinal}: tail=0x22 despite both W0 variants absent");
        }
    }
    assert!(found);assert!(!covers_physical);
    let sector_states:Vec<Vec<u8>>=(0u16..256).map(|mask|(0..8).flat_map(|sector|std::iter::repeat_n(if mask&(1<<sector)!=0 {0x33}else{0x11},512)).collect()).collect();
    assert!(sector_states.contains(&physical));
    assert!(sector_states.iter().any(|b|b.iter().all(|v|*v==0x11)));
    assert!(sector_states.iter().any(|b|b.iter().all(|v|*v==0x33)));
    println!("RECHECK synthetic raw writes: {} abstract states omit [0x33*2048,0x11*2048], independent 256 sector states include it; no filesystem outcome claim",facts.state_count);
    let old=vec![0u8;1024]; let mut new=old.clone();new[10]=1;new[20]=2;
    let torn=torn_image_of_in_place_overwrite(&old,&WrittenContents::Bytes(new.clone()));
    assert!(torn[..512]!=old[..512] && torn[..512]!=new[..512]);
    println!("CONFIRMED third-state generator tears within a single 512-byte sector (offsets 10 and 20)");
}
