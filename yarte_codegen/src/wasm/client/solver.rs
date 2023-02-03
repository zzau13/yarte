use std::collections::{BTreeSet, HashMap};

use yarte_dom::dom::{ExprId, TreeMap, Var, VarId, VarInner, VarMap};

use crate::client::utils::get_self_id;

#[derive(Default, Debug)]
pub struct Solver {
    /// Expresion -> Inner Variables
    tree_map: TreeMap,
    /// Variables grouped by base field
    grouped_map: HashMap<VarId, BTreeSet<VarId>>,
    /// VarId -> Variable details
    var_map: HashMap<VarId, VarInner>,
}

impl Solver {
    #[inline]
    pub fn init(&mut self, tree_map: TreeMap, var_map: VarMap) {
        let mut grouped = HashMap::new();
        let var_map: HashMap<VarId, VarInner> = var_map
            .into_iter()
            .filter_map(|(i, x)| match x {
                Var::This(x) => {
                    grouped
                        .entry(x.base)
                        .and_modify(|x: &mut BTreeSet<VarId>| {
                            x.insert(i);
                        })
                        .or_insert_with(|| {
                            // Need Order
                            let mut b = BTreeSet::new();
                            b.insert(i);
                            b
                        });
                    Some((i, x))
                }
                Var::Local(..) => None,
            })
            .collect();

        if grouped.get(&get_self_id()).is_none() {
            todo!("need any field in struct of application")
        }
        self.grouped_map = grouped;
        self.tree_map = tree_map;
        self.var_map = var_map;
    }

    pub fn expr_inner_var(&self, id: &ExprId) -> &BTreeSet<VarId> {
        self.tree_map
            .get(id)
            .unwrap_or_else(|| panic!("unregistered expression: {id}"))
    }

    pub fn group(&self, id: &VarId) -> &BTreeSet<VarId> {
        self.grouped_map
            .get(id)
            .unwrap_or_else(|| panic!("unregistered group: {id}"))
    }

    pub fn var_inner(&self, id: &VarId) -> &VarInner {
        self.var_map
            .get(id)
            .unwrap_or_else(|| panic!("unregistered variable: {id}"))
    }

    pub fn var_base(&self, id: &VarId) -> VarId {
        self.var_inner(id).base
    }

    pub fn var_ident(&self, id: &VarId) -> &str {
        &self.var_inner(id).ident
    }
}
