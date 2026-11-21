//! 收集：R-结构、R-方法、R-路径、R-隐式与 R-开放这几条规则把单元连进闭包，连同分配器与外部类型上的本仓 trait impl。
use super::*;

impl<'workspace> ClosureRun<'workspace> {
    /// R-结构。交这个单元经 use 别名引到的原名（R-裸名 照原名再解）。
    pub(super) fn follow_structure(
        &mut self,
        unit_index: UnitIndex,
        target: TargetIndex,
    ) -> Vec<String> {
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let mut alias_originals = Vec::new();
        if !self.switches.is_intact(BrokenRule::StructuralEdges)
            || unit.kind == UnitKind::DocumentationTest
        {
            return alias_originals;
        }
        self.rule_being_followed = "R-结构";
        if let Some(container) = unit.container {
            self.push(container, target);
        }
        if let Some(head) = unit.module_head {
            self.push(head, target);
        }
        if let Some(membership) = workspace.membership(unit.file, target) {
            for declaring in &membership.declared_by {
                self.push(*declaring, target);
            }
        }
        let Some(uses) = workspace
            .indexes
            .uses_by_scope
            .get(&(unit.file, unit.local_module_path.clone()))
        else {
            return alias_originals;
        };
        let names = &unit.references.plain_names;
        let methods = &unit.references.method_names;
        for use_index in uses {
            let Some(imports) = workspace.units[use_index.0].imports.as_ref() else {
                continue;
            };
            let touches = imports.is_glob
                || imports.names.iter().any(|name| {
                    names.binary_search(name).is_ok() || methods.binary_search(name).is_ok()
                });
            if !touches {
                continue;
            }
            self.push(*use_index, target);
            for (alias, original) in &imports.aliases {
                if names.binary_search(alias).is_ok() {
                    alias_originals.push(original.clone());
                }
            }
        }
        alias_originals
    }

    pub(super) fn follow_methods(&mut self, unit_index: UnitIndex) {
        if !self.switches.is_intact(BrokenRule::MethodCalls) {
            return;
        }
        self.rule_being_followed = "R-方法";
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let foreign_type_implementations = self.foreign_type_trait_implementations();
        for method_name in &unit.references.method_names {
            if !self.methods_seen.insert(method_name.clone()) {
                continue;
            }
            let Some(members) = workspace.indexes.members_by_name.get(method_name) else {
                continue;
            };
            for member in members {
                let Some(container) = workspace.units[member.0].container else {
                    continue;
                };
                let container_unit = &workspace.units[container.0];
                let is_eligible = if container_unit.kind == UnitKind::TraitHead {
                    true
                } else if container_unit.kind == UnitKind::ImplementationHead {
                    // blanket impl 够得到的方法算边：自身类型里带它的泛型参数，能被点名的具体名字可能一个都没有，不等自身类型被点名
                    container_unit.implementation.as_ref().is_some_and(|shape| {
                        shape.is_blanket_implementation
                            || shape
                                .self_type_names
                                .iter()
                                .any(|type_name| self.named_types.contains(type_name))
                    }) || foreign_type_implementations.contains(&container)
                } else {
                    false
                };
                if is_eligible {
                    self.record_the_first_edge(*member, method_name);
                    self.push_with_a_linked_target(*member);
                }
            }
        }
    }

    pub(super) fn follow_qualified_paths(&mut self, unit_index: UnitIndex) {
        if !self.switches.is_intact(BrokenRule::QualifiedPaths) {
            return;
        }
        self.rule_being_followed = "R-路径";
        let workspace = self.workspace;
        let unit = &workspace.units[unit_index.0];
        let own_container = unit
            .container
            .map(|container| &workspace.units[container.0]);
        let foreign_type_implementations = self.foreign_type_trait_implementations();
        for (qualifier, member_name) in &unit.references.qualified_paths {
            let Some(members) = workspace.indexes.members_by_name.get(member_name) else {
                continue;
            };
            for member in members {
                let Some(container) = workspace.units[member.0].container else {
                    continue;
                };
                let container_unit = &workspace.units[container.0];
                let shape = container_unit.implementation.as_ref();
                let self_types_contain = |type_name: &str| {
                    shape.is_some_and(|implementation_shape| {
                        implementation_shape
                            .self_type_names
                            .iter()
                            .any(|name| name == type_name)
                    })
                };
                let is_match = if container_unit.kind == UnitKind::TraitHead {
                    container_unit.name == *qualifier
                        || (qualifier == "Self"
                            && own_container.is_some_and(|own| {
                                own.kind == UnitKind::TraitHead && own.name == container_unit.name
                            }))
                } else if container_unit.kind == UnitKind::ImplementationHead {
                    self_types_contain(qualifier)
                        || (qualifier == "Self"
                            && own_container.is_some_and(|own| {
                                own.implementation.as_ref().is_some_and(|own_shape| {
                                    own_shape
                                        .self_type_names
                                        .iter()
                                        .any(|name| self_types_contain(name))
                                })
                            }))
                        || shape.is_some_and(|implementation_shape| {
                            implementation_shape.is_trait_implementation
                                && implementation_shape.trait_name.as_deref()
                                    == Some(qualifier.as_str())
                                && (implementation_shape.is_blanket_implementation
                                    || implementation_shape
                                        .self_type_names
                                        .iter()
                                        .any(|type_name| self.named_types.contains(type_name))
                                    || foreign_type_implementations.contains(&container))
                        })
                } else {
                    false
                };
                if is_match {
                    self.record_the_first_edge(*member, &format!("{qualifier}::{member_name}"));
                    self.push_with_a_linked_target(*member);
                }
            }
        }
    }

    /// 外部类型上的本仓 trait impl 头（`workspace` 的 `foreign_type_trait_implementations`）：经泛型约束分派、自身类型永远不会被点名，
    /// R-方法落到 `.x(` 时它的成员 `x` 照连，R-路径 `Tr::x` 同样。弄坏开关 foreign-type-trait-implementations：一个都不放行。
    fn foreign_type_trait_implementations(&self) -> &'workspace BTreeSet<UnitIndex> {
        static NO_IMPLEMENTATIONS: BTreeSet<UnitIndex> = BTreeSet::new();
        if self
            .switches
            .is_intact(BrokenRule::ForeignTypeTraitImplementations)
        {
            &self.workspace.indexes.foreign_type_trait_implementations
        } else {
            &NO_IMPLEMENTATIONS
        }
    }

    /// `#[global_allocator]` 的 static 管整个链接出来的程序：链接集里有它所在的目标，它就进闭包（连同它的类型、R-隐式带上 GlobalAlloc 的 impl）。
    /// 弄坏开关 global-allocator-edges：不进。
    pub(super) fn push_global_allocators_of_the_linked_targets(&mut self) {
        if !self.switches.is_intact(BrokenRule::GlobalAllocatorEdges) {
            return;
        }
        self.rule_being_followed = "全局分配器";
        let workspace = self.workspace;
        for allocator in &workspace.indexes.global_allocators {
            for membership in &workspace.memberships[workspace.units[allocator.0].file.0] {
                if self.link.targets.contains(&membership.target) {
                    self.push(*allocator, membership.target);
                }
            }
        }
    }

    pub(super) fn push_container_and_members(&mut self, head: UnitIndex) {
        let workspace = self.workspace;
        self.push_with_a_linked_target(head);
        if let Some(members) = workspace.indexes.members_of_container.get(&head) {
            for member in members {
                self.push_with_a_linked_target(*member);
            }
        }
    }

    /// R-开放（CHA）：链接集里这个 trait 的全部 impl 头与成员。只给入口签名里的 trait 名叫（`compute_closure_of_entries` 里逐个入口那一支）；
    /// 闭包内部的 `dyn Trait` 只在弄坏开关 `dyn-traits-open-every-implementation` 打开时叫。
    pub(super) fn open_trait(&mut self, trait_name: &str) {
        if !self.switches.is_intact(BrokenRule::ClassHierarchy)
            || !self.dyn_traits_seen.insert(trait_name.to_string())
        {
            return;
        }
        let workspace = self.workspace;
        if let Some(implementations) = workspace
            .indexes
            .trait_implementations_by_trait_name
            .get(trait_name)
        {
            for head in implementations {
                self.push_container_and_members(*head);
            }
        }
    }

    /// 类型被点名：R-隐式 给它写的 trait impl 头与全部成员进来；R-方法 补上之前见过名字的方法
    /// （固有 impl 的；R-隐式 被弄坏时 trait impl 的也照这一条补，两条规则各管各的，不互相兜底）。
    pub(super) fn name_type(&mut self, type_name: &str) {
        if !self.named_types.insert(type_name.to_string()) {
            return;
        }
        let workspace = self.workspace;
        let Some(implementations) = workspace
            .indexes
            .implementations_by_self_type_name
            .get(type_name)
        else {
            return;
        };
        for head in implementations {
            let is_trait_implementation = workspace.units[head.0]
                .implementation
                .as_ref()
                .is_some_and(|shape| shape.is_trait_implementation);
            if is_trait_implementation
                && self
                    .switches
                    .is_intact(BrokenRule::TraitImplementationEdges)
            {
                self.push_container_and_members(*head);
            } else if self.switches.is_intact(BrokenRule::MethodCalls) {
                if let Some(members) = workspace.indexes.members_of_container.get(head) {
                    for member in members {
                        if self.methods_seen.contains(&workspace.units[member.0].name) {
                            self.push_with_a_linked_target(*member);
                        }
                    }
                }
            }
        }
    }
}
