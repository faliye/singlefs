use super::*;

/// 根之下节点的指针按位置直接取，不在指针表上逐项找（运行时决策路径之外装单元时每个照抄的节点都问一次，逐项找合起来是节点数的平方）：
/// 层级 1 有 20000 个节点的三层形状（根之下 20200 个节点），指针表按 bump 次序、每项的出生序号就是它的下标；表头那一项故意记成要问的
/// 那个位置、带另一个指针——逐项找的写法先碰到它、交回那个错的，按下标取的写法只看「这一层之前各层的节点数之和 + 同层序号」那一格。
#[test]
fn node_pointer_below_the_root_is_taken_at_its_position_in_bump_order() {
    const LEVEL_ONE_NODES: usize = 20_000;
    const LEVEL_TWO_NODES: usize = 200;
    let shape = ContainerTreeShape::from_children_of_each_node_at_each_level(vec![
        vec![3; LEVEL_ONE_NODES],
        vec![100; LEVEL_TWO_NODES],
        vec![LEVEL_TWO_NODES],
    ])
    .expect("每层孩子数之和等于下一层的节点数、根一个节点");
    let pointer_numbered = |number: u32| NodePointer {
        head: NodePointer::empty_root().head,
        locations: NodePointer::empty_root().locations,
        instance: NodePointer::empty_root().instance,
        birth_sequence: BirthSequence(number),
    };
    let asked = ContainerTreeNodePosition {
        level: 2,
        index_in_level: 7,
    };
    let mut pointers_below_the_root: Vec<(ContainerTreeNodePosition, NodePointer)> = shape
        .positions_in_bump_order()
        .into_iter()
        .filter(|position| !shape.is_the_root(*position))
        .zip(0..)
        .map(|(position, number)| (position, pointer_numbered(number)))
        .collect();
    pointers_below_the_root[0] = (asked, pointer_numbered(u32::MAX));
    let version = ContainerTreeIndexNodesVersion {
        shape,
        pointers_below_the_root,
    };
    assert_eq!(
        version.pointer_below_the_root(asked),
        Some(pointer_numbered(
            u32::try_from(LEVEL_ONE_NODES + 7).expect("两万多装得进 u32")
        )),
        "层级 2 第 7 个在 bump 次序里是层级 1 的 20000 个之后的第 7 项"
    );
    assert_eq!(
        version.pointer_below_the_root(ContainerTreeNodePosition {
            level: 3,
            index_in_level: 0
        }),
        None,
        "根不在根之下的指针表里"
    );
    assert_eq!(
        version.pointer_below_the_root(ContainerTreeNodePosition {
            level: 2,
            index_in_level: u32::try_from(LEVEL_TWO_NODES).expect("两百装得进 u32")
        }),
        None,
        "形状里没有的位置"
    );
}
