use crate::{PropertiesType, TripleStoreExtend};

use super::MemTripleStore;

impl<NodeProperties: PropertiesType, EdgeProperties: PropertiesType>
    TripleStoreExtend<NodeProperties, EdgeProperties>
    for MemTripleStore<NodeProperties, EdgeProperties>
{
    fn extend(&mut self, mut other: Self) -> Result<(), ()> {
        for (id, data) in other.node_props {
            match self.node_props.entry(id) {
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    *o.get_mut() = data;
                }
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(data);
                }
            }
        }

        for (id, other_edge_props_id) in other.spo_data {
            match self.spo_data.entry(id) {
                std::collections::btree_map::Entry::Vacant(self_spo_data_v) => {
                    // We don't have this edge already.
                    // Get the content from other.edge_props
                    other
                        .edge_props
                        .remove(&other_edge_props_id)
                        .map(|other_edge_props| {
                            self_spo_data_v.insert(other_edge_props_id);
                            self.edge_props
                                .insert(other_edge_props_id, other_edge_props);
                        });
                }

                std::collections::btree_map::Entry::Occupied(self_spo_data_o) => {
                    let self_edge_props_id = self_spo_data_o.get();

                    let self_edge_data = self.edge_props.entry(*self_edge_props_id);
                    let other_edge_data = other.edge_props.entry(other_edge_props_id);

                    // Merge our edge props using the existing id.

                    match (self_edge_data, other_edge_data) {
                        (
                            std::collections::btree_map::Entry::Vacant(_),
                            std::collections::btree_map::Entry::Vacant(_),
                        ) => {}
                        (
                            std::collections::btree_map::Entry::Vacant(v),
                            std::collections::btree_map::Entry::Occupied(o),
                        ) => {
                            v.insert(o.remove());
                        }
                        (
                            std::collections::btree_map::Entry::Occupied(_),
                            std::collections::btree_map::Entry::Vacant(_),
                        ) => {
                            // Nothing to do.
                        }
                        (
                            std::collections::btree_map::Entry::Occupied(mut self_o),
                            std::collections::btree_map::Entry::Occupied(other_o),
                        ) => *self_o.get_mut() = other_o.remove(),
                    }
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use ulid::Ulid;

    use crate::prelude::*;
    use crate::{MemTripleStore, Triple};

    #[test]
    fn test_extend() {
        let mut left = MemTripleStore::new();
        let mut right = MemTripleStore::new();

        let (node_1, node_props_1) = (Ulid::new(), "a".to_string());
        let (node_2, node_props_2) = (Ulid::new(), "b".to_string());
        let (node_3, node_props_3) = (Ulid::new(), "c".to_string());
        let (node_4, node_props_4) = (node_1, "d".to_string());

        let edge_1 = Ulid::new();
        let edge_props_1 = "1".to_string();
        let edge_props_2 = "2".to_string();

        // Construct the left graph to be (1, "a") -("1")-> (2, "b")
        left.insert_node(node_1.clone(), node_props_1.clone())
            .expect("success");
        left.insert_node(node_2.clone(), node_props_2.clone())
            .expect("success");
        left.insert_edge(
            Triple {
                sub: node_1,
                pred: edge_1,
                obj: node_2,
            },
            edge_props_1.clone(),
        )
        .expect("success");

        // Construct the right graph to be (3, "c") -("2")-> (1, "d")
        right
            .insert_node(node_3.clone(), node_props_3.clone())
            .expect("success");
        right
            .insert_node(node_4.clone(), node_props_4.clone())
            .expect("success");
        right
            .insert_edge(
                Triple {
                    sub: node_3,
                    pred: edge_1,
                    obj: node_4,
                },
                edge_props_2.clone(),
            )
            .expect("success");

        // Perform the extension.
        left.extend(right).expect("success");

        // We expect the result to be (3, "c") -("2")-> (1, "d") -("1")-> (2, "b")
        let node_data = left
            .iter_node()
            .map(|i| i.expect("success"))
            .collect::<Vec<_>>();
        assert_eq!(node_data.len(), 3);
        assert!(node_data.contains(&(node_1, node_props_4)));
        assert!(node_data.contains(&(node_2, node_props_2)));
        assert!(node_data.contains(&(node_3, node_props_3)));

        let edge_data = left
            .iter_edge_spo()
            .map(|i| i.expect("success"))
            .collect::<Vec<_>>();
        assert_eq!(edge_data.len(), 2);
        assert!(edge_data.contains(&(
            Triple {
                sub: node_3,
                pred: edge_1,
                obj: node_1
            },
            edge_props_2
        )));
        assert!(edge_data.contains(&(
            Triple {
                sub: node_1,
                pred: edge_1,
                obj: node_2
            },
            edge_props_1
        )));
    }
}
