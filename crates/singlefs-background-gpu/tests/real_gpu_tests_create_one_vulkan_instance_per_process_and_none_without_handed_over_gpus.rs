//! 真卡测试取卡（`common_discrete_gpus_of_this_host` 的 `handed_over_gpus_through_the_instance_kept_for_this_process`）一个进程只建一个 Vulkan 实例：
//! 同一个进程里连取两次卡只建一次、两次交回与枚举用的都是同一份；服务没交卡、或交的卡里没有这一台的，在建实例之前就交回「本次未跑」、一次都不建。
//! 实例换成数建了几次的替身，`GPU_CARDS_BY_PRIORITY` 的值、这一台的地址、`nvidia-smi` 报的总线与枚举到的独显都在用例里现造：不要卡、不挂特性，
//! `cargo test -p singlefs-background-gpu --test <本目标>` 就能跑。每条用例用自己的那一格（`OnceLock`），不碰真卡那一份进程级的实例。
//! 同一份取卡另守两样：交了这一台的卡而读不成本机的卡（`nvidia-smi`）时在建实例之前判红；取卡的结局打哪一行
//! （`line_reporting_the_gpus_taken`：「本次未跑」的 `NOT_RUN_NO_GPU_ADAPTER` 记号行与交了卡的那一行）与那一行的字。
//! 核这个进程在哪几张卡上读不成、读到一张卡都不在时交回的结局（本次未跑）与要打的两行记号行，经不打行的判定
//! （`gpus_this_process_is_on_and_the_line_to_print`）在这里逐字钉：用例自己不打 `NOT_RUN_NO_GPU_ADAPTER` 行，变异判法才不把它记成本次未跑。
//! 真卡那一层薄壳（`handed_over_gpus_or_report_not_run`）交进去的是不是进程级的那一格、打没打这一行，这里罩不到（要卡），不在这里测。
mod common_discrete_gpus_of_this_host;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use common_discrete_gpus_of_this_host::{
    gpus_this_process_is_on_and_the_line_to_print,
    handed_over_gpus_through_the_instance_kept_for_this_process, line_reporting_the_gpus_taken,
    GpusTakenThroughTheInstanceKeptForThisProcess, GpusThisProcessIsOn,
    GpusThisProcessIsOnAndTheLineToPrint, HandedOverGpu, NvidiaSmiIndex,
};

/// 打行那几条用例里的用例名。
const TEST_NAME_IN_THE_REPORTED_LINE: &str = "real_gpu_case";

/// 这一台在用例里的地址（文档用的地址段）。
const THIS_HOST: &str = "192.0.2.1";
/// 第二台在用例里的地址（文档用的地址段）。
const OTHER_HOST: &str = "192.0.2.2";

/// 实例的替身：带着建它那一刻这个进程一共建过几个（含它自己），两次取卡交回的是不是同一份看得出。
#[derive(Clone, Debug, PartialEq, Eq)]
struct InstanceStandInNumberedByItsCreation {
    creations_up_to_this_one: usize,
}

/// 建替身并在 `creations` 上记一次。
fn stand_in_created_and_counted(creations: &Cell<usize>) -> InstanceStandInNumberedByItsCreation {
    creations.set(creations.get() + 1);
    InstanceStandInNumberedByItsCreation {
        creations_up_to_this_one: creations.get(),
    }
}

/// `nvidia-smi` 报的这一台两张卡：序号 0 在总线 01:00.0，序号 1 在总线 10:00.0（去掉域、小写）。
fn pci_buses_of_two_gpus() -> BTreeMap<NvidiaSmiIndex, String> {
    BTreeMap::from([
        (NvidiaSmiIndex(0), "01:00.0".to_string()),
        (NvidiaSmiIndex(1), "10:00.0".to_string()),
    ])
}

/// 实例上枚举到的两张独显（wgpu 报的总线地址原样）。
fn discrete_adapters_of_two_gpus() -> Vec<String> {
    vec!["0000:01:00.0".to_string(), "0000:10:00.0".to_string()]
}

/// 照 `handover_text` 在 `instance_kept` 上取一次卡：这一台是 [`THIS_HOST`]，`nvidia-smi` 与枚举都报那两张卡；建实例记在 `creations` 上，
/// 枚举时交进来的是哪一份实例记在 `instances_enumerated_through` 上。
fn gpus_taken_on_this_host_with_two_gpus(
    handover_text: &str,
    instance_kept: &OnceLock<InstanceStandInNumberedByItsCreation>,
    creations: &Cell<usize>,
    instances_enumerated_through: &RefCell<Vec<InstanceStandInNumberedByItsCreation>>,
) -> GpusTakenThroughTheInstanceKeptForThisProcess<InstanceStandInNumberedByItsCreation> {
    handed_over_gpus_through_the_instance_kept_for_this_process(
        Some(handover_text),
        || Ok(THIS_HOST.to_string()),
        || Ok(pci_buses_of_two_gpus()),
        instance_kept,
        || stand_in_created_and_counted(creations),
        |instance| {
            instances_enumerated_through
                .borrow_mut()
                .push(instance.clone());
            discrete_adapters_of_two_gpus()
        },
    )
}

/// 交了卡的那一种结局里的实例与卡。
///
/// # Panics
/// 结局是本次未跑：交了这一台的卡的用例里不该是这一种。
fn handed_instance_and_gpus(
    taken: GpusTakenThroughTheInstanceKeptForThisProcess<InstanceStandInNumberedByItsCreation>,
) -> (InstanceStandInNumberedByItsCreation, Vec<HandedOverGpu>) {
    match taken {
        GpusTakenThroughTheInstanceKeptForThisProcess::Handed(handed_over) => {
            (handed_over.vulkan_instance, handed_over.gpus)
        }
        GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { reason } => {
            panic!("交了这一台的卡却报本次未跑：{reason}")
        }
    }
}

#[test]
fn taking_the_gpus_twice_in_one_process_creates_the_instance_once_and_hands_over_the_same_one() {
    let instance_kept = OnceLock::new();
    let creations = Cell::new(0);
    let instances_enumerated_through = RefCell::new(Vec::new());
    let handover_text = format!("{THIS_HOST}:1:16G {THIS_HOST}:0:16G");
    let (instance_from_the_taking_that_creates_it, gpus_from_the_taking_that_creates_it) =
        handed_instance_and_gpus(gpus_taken_on_this_host_with_two_gpus(
            &handover_text,
            &instance_kept,
            &creations,
            &instances_enumerated_through,
        ));
    assert_eq!(creations.get(), 1, "进程里还没有实例时取卡建一个");
    let (instance_from_the_taking_that_reuses_it, gpus_from_the_taking_that_reuses_it) =
        handed_instance_and_gpus(gpus_taken_on_this_host_with_two_gpus(
            &handover_text,
            &instance_kept,
            &creations,
            &instances_enumerated_through,
        ));
    assert_eq!(creations.get(), 1, "进程里已有实例时再取卡不再建");
    let the_only_instance_created = InstanceStandInNumberedByItsCreation {
        creations_up_to_this_one: 1,
    };
    assert_eq!(
        instance_from_the_taking_that_creates_it, the_only_instance_created,
        "建实例的那一次交回它建的那一份"
    );
    assert_eq!(
        instance_from_the_taking_that_reuses_it, the_only_instance_created,
        "再取卡交回的是同一份"
    );
    assert_eq!(
        *instances_enumerated_through.borrow(),
        vec![the_only_instance_created; 2],
        "两次取卡都在那一份实例上枚举"
    );
    let gpus_in_the_handed_order = vec![
        HandedOverGpu {
            nvidia_smi_index: NvidiaSmiIndex(1),
            pci_bus_address: "0000:10:00.0".to_string(),
        },
        HandedOverGpu {
            nvidia_smi_index: NvidiaSmiIndex(0),
            pci_bus_address: "0000:01:00.0".to_string(),
        },
    ];
    assert_eq!(
        gpus_from_the_taking_that_creates_it, gpus_in_the_handed_order,
        "建实例的那一次照交来的次序对上两张独显"
    );
    assert_eq!(
        gpus_from_the_taking_that_reuses_it, gpus_in_the_handed_order,
        "再取卡照样对上同样两张"
    );
}

#[test]
fn no_gpu_handed_over_returns_before_the_instance_is_created() {
    let instance_kept: OnceLock<InstanceStandInNumberedByItsCreation> = OnceLock::new();
    let creations = Cell::new(0);
    let taken = handed_over_gpus_through_the_instance_kept_for_this_process(
        None,
        || panic!("服务没交卡时不该起多机配置判法问这一台是谁"),
        || panic!("服务没交卡时不该读 nvidia-smi"),
        &instance_kept,
        || stand_in_created_and_counted(&creations),
        |_instance| panic!("服务没交卡时不该枚举适配器"),
    );
    assert!(
        matches!(
            taken,
            GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { .. }
        ),
        "服务没交卡报本次未跑"
    );
    assert_eq!(creations.get(), 0, "服务没交卡时一个实例都不建");
    assert!(instance_kept.get().is_none(), "进程里没有留下实例");
}

#[test]
fn gpus_handed_only_to_another_host_return_before_the_instance_is_created() {
    let instance_kept: OnceLock<InstanceStandInNumberedByItsCreation> = OnceLock::new();
    let creations = Cell::new(0);
    let handover_text = format!("{OTHER_HOST}:0:16G {OTHER_HOST}:1:16G");
    let taken = handed_over_gpus_through_the_instance_kept_for_this_process(
        Some(handover_text.as_str()),
        || Ok(THIS_HOST.to_string()),
        || panic!("交的卡里没有这一台的时不该读 nvidia-smi"),
        &instance_kept,
        || stand_in_created_and_counted(&creations),
        |_instance| panic!("交的卡里没有这一台的时不该枚举适配器"),
    );
    assert!(
        matches!(
            taken,
            GpusTakenThroughTheInstanceKeptForThisProcess::NotRun { .. }
        ),
        "交的卡里没有这一台的报本次未跑"
    );
    assert_eq!(creations.get(), 0, "交的卡里没有这一台的时一个实例都不建");
    assert!(instance_kept.get().is_none(), "进程里没有留下实例");
}

/// 场景：交了这一台的一张卡，读本机的卡（`nvidia-smi`）读不成。
/// 预期：取卡判红（panic），一个实例都不建、进程里没有留下实例：先读 `nvidia-smi`、后取实例。
/// 判别力（`crates/mutations.tsv`）：取实例挪到读 `nvidia-smi` 之前，这条红。
#[test]
fn unreadable_gpus_of_this_host_are_judged_red_before_the_instance_is_created() {
    let instance_kept: OnceLock<InstanceStandInNumberedByItsCreation> = OnceLock::new();
    let creations = Cell::new(0);
    let handover_text = format!("{THIS_HOST}:0:16G");
    let taking = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handed_over_gpus_through_the_instance_kept_for_this_process(
            Some(handover_text.as_str()),
            || Ok(THIS_HOST.to_string()),
            || Err("nvidia-smi 起不来（用例现造）".to_string()),
            &instance_kept,
            || stand_in_created_and_counted(&creations),
            |_instance| discrete_adapters_of_two_gpus(),
        )
    }));
    assert!(taking.is_err(), "交了这一台的卡而读不成本机的卡：取卡判红");
    assert_eq!(creations.get(), 0, "读不成本机的卡时一个实例都不建");
    assert!(instance_kept.get().is_none(), "进程里没有留下实例");
}

/// 场景：服务没交卡（变量没设），取卡的结局打的那一行。
/// 预期：恰好是 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<结局里的原因>`，原因写明没经 `GPU_CARDS_BY_PRIORITY` 交卡；
/// 门禁、变异判法与任务服务按行首这个记号、`test=` 后面不带空白的用例名认「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：记号写错，或没跑成的结局打成交了卡的那一行，这条红。
#[test]
fn no_gpu_handed_over_reports_the_not_run_marker_line_with_the_test_name_and_the_reason() {
    let instance_kept: OnceLock<InstanceStandInNumberedByItsCreation> = OnceLock::new();
    let creations = Cell::new(0);
    let taken = handed_over_gpus_through_the_instance_kept_for_this_process(
        None,
        || panic!("服务没交卡时不该起多机配置判法问这一台是谁"),
        || panic!("服务没交卡时不该读 nvidia-smi"),
        &instance_kept,
        || stand_in_created_and_counted(&creations),
        |_instance| panic!("服务没交卡时不该枚举适配器"),
    );
    assert_eq!(
        line_reporting_the_gpus_taken(TEST_NAME_IN_THE_REPORTED_LINE, &taken),
        "NOT_RUN_NO_GPU_ADAPTER test=real_gpu_case reason=没经 GPU_CARDS_BY_PRIORITY 交卡（没设或是空的）",
        "服务没交卡打本次未跑的记号行"
    );
}

/// 场景：服务交的卡全在第二台上，取卡的结局打的那一行。
/// 预期：恰好是本次未跑的记号行，原因点名交来的值与这一台的地址。
#[test]
fn gpus_handed_only_to_another_host_report_the_not_run_marker_line() {
    let instance_kept: OnceLock<InstanceStandInNumberedByItsCreation> = OnceLock::new();
    let creations = Cell::new(0);
    let handover_text = format!("{OTHER_HOST}:0:16G {OTHER_HOST}:1:16G");
    let taken = handed_over_gpus_through_the_instance_kept_for_this_process(
        Some(handover_text.as_str()),
        || Ok(THIS_HOST.to_string()),
        || panic!("交的卡里没有这一台的时不该读 nvidia-smi"),
        &instance_kept,
        || stand_in_created_and_counted(&creations),
        |_instance| panic!("交的卡里没有这一台的时不该枚举适配器"),
    );
    assert_eq!(
        line_reporting_the_gpus_taken(TEST_NAME_IN_THE_REPORTED_LINE, &taken),
        "NOT_RUN_NO_GPU_ADAPTER test=real_gpu_case reason=交来的卡（GPU_CARDS_BY_PRIORITY=「192.0.2.2:0:16G 192.0.2.2:1:16G」）里没有这一台（HOSTS 里的 192.0.2.1）的",
        "交的卡里没有这一台的打本次未跑的记号行"
    );
}

/// 场景：服务交了这一台的两张卡（序号 1 在前），取卡的结局打的那一行。
/// 预期：恰好是 `GPUS_HANDED_OVER_TO_THE_TEST test=<用例名> gpus=1@0000:10:00.0,0@0000:01:00.0`（照交来的次序，序号接 wgpu 报的总线地址），
/// 不是本次未跑的记号行。
/// 判别力（`crates/mutations.tsv`）：那一行不写序号，这条红。
#[test]
fn gpus_handed_to_this_host_report_the_handed_over_line_in_the_handed_order() {
    let instance_kept = OnceLock::new();
    let creations = Cell::new(0);
    let instances_enumerated_through = RefCell::new(Vec::new());
    let taken = gpus_taken_on_this_host_with_two_gpus(
        &format!("{THIS_HOST}:1:16G {THIS_HOST}:0:16G"),
        &instance_kept,
        &creations,
        &instances_enumerated_through,
    );
    assert_eq!(
        line_reporting_the_gpus_taken(TEST_NAME_IN_THE_REPORTED_LINE, &taken),
        "GPUS_HANDED_OVER_TO_THE_TEST test=real_gpu_case gpus=1@0000:10:00.0,0@0000:01:00.0",
        "交了这一台的卡打交了卡的那一行"
    );
}

/// 交来的一张卡（序号 0，总线地址现造），核这个进程在哪几张卡上的两条用例共用。
fn one_handed_over_gpu() -> Vec<HandedOverGpu> {
    vec![HandedOverGpu {
        nvidia_smi_index: NvidiaSmiIndex(0),
        pci_bus_address: "0000:01:00.0".to_string(),
    }]
}

/// 场景：交了一张卡，核这个进程在哪几张卡上时 `nvidia-smi` 读不成（原因现造）。
/// 预期：结局是本次未跑（不当成只在交来的卡上），要打的那一行恰好是
/// `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=读不成这个进程在哪几张卡上：<读不成的那一句>`；
/// 门禁、变异判法与任务服务按行首这个记号认「本次未跑」，记号后面多一个字它们就认不出。用例只核交回的值、不打那一行。
/// 判别力（`crates/mutations.tsv`）：结局换成只在交来的卡上、原因里丢了读不成的那一句，这条都红；记号写错由 `not_run_line_of_the_test` 那一行变异证。
#[test]
fn gpus_of_the_process_not_read_are_judged_not_run_with_the_marker_line_carrying_the_reason_not_read(
) {
    assert_eq!(
        gpus_this_process_is_on_and_the_line_to_print(
            TEST_NAME_IN_THE_REPORTED_LINE,
            Err("nvidia-smi 起不来（用例现造）".to_string()),
            &one_handed_over_gpu(),
            4242,
        ),
        GpusThisProcessIsOnAndTheLineToPrint {
            outcome: GpusThisProcessIsOn::NotReadSoThisRunIsNotRun,
            line_to_print: "NOT_RUN_NO_GPU_ADAPTER test=real_gpu_case reason=读不成这个进程在哪几张卡上：nvidia-smi 起不来（用例现造）".to_string(),
        },
        "读不成这个进程在哪几张卡上时交回本次未跑、要打本次未跑的记号行"
    );
}

/// 场景：交了一张卡，核这个进程在哪几张卡上时读到它一张卡都不在（读法看不见它开着的设备；进程号现造）。
/// 预期：结局是本次未跑（不当成只在交来的卡上），要打的那一行恰好是
/// `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=…（进程号 <号>）…：读法看不见它`。用例只核交回的值、不打那一行。
/// 判别力（`crates/mutations.tsv`）：结局换成只在交来的卡上、原因里不写进程号，这条都红。
#[test]
fn process_seen_on_no_gpu_is_judged_not_run_with_the_marker_line_carrying_its_process_identifier() {
    assert_eq!(
        gpus_this_process_is_on_and_the_line_to_print(
            TEST_NAME_IN_THE_REPORTED_LINE,
            Ok(BTreeSet::new()),
            &one_handed_over_gpu(),
            4242,
        ),
        GpusThisProcessIsOnAndTheLineToPrint {
            outcome: GpusThisProcessIsOn::NotReadSoThisRunIsNotRun,
            line_to_print: "NOT_RUN_NO_GPU_ADAPTER test=real_gpu_case reason=nvidia-smi 列的计算进程里没有这个进程（进程号 4242），而交来的卡上开着它的设备：读法看不见它".to_string(),
        },
        "读到这个进程一张卡都不在时交回本次未跑、要打本次未跑的记号行"
    );
}
