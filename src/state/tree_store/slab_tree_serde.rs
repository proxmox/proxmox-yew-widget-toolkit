use std::borrow::Cow;

use slab::Slab;

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{KeyedSlabTree, SlabTree, SlabTreeEntry, SlabTreeNodeRef};
use crate::props::ExtractPrimaryKey;

// { record: {}, expanded: true, children: [ record: { }, ... ] }

struct ChildList<'a, T> {
    children: &'a [usize],
    tree: &'a SlabTree<T>,
}

impl<'a, T: 'static + Serialize> Serialize for ChildList<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.children.len()))?;
        for child_id in self.children {
            let child = SlabTreeNodeRef {
                node_id: *child_id,
                tree: self.tree,
            };
            seq.serialize_element(&child)?;
        }
        seq.end()
    }
}

impl<'a, T: 'static + Serialize> Serialize for SlabTreeNodeRef<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TreeNode", 3)?;
        let entry = self.tree.get(self.node_id).unwrap();

        state.serialize_field("record", &entry.record)?;
        if entry.expanded {
            state.serialize_field("expanded", &entry.expanded)?;
        } else {
            state.skip_field("expanded")?;
        }

        if let Some(children) = &entry.children {
            let children = ChildList {
                children,
                tree: self.tree,
            };
            state.serialize_field("children", &children)?;
        } else {
            state.skip_field("children")?;
        }

        state.end()
    }
}

impl<T: 'static + Serialize> Serialize for SlabTree<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.root() {
            None => serializer.serialize_struct("TreeNode", 0)?.end(),
            Some(node_ref) => node_ref.serialize(serializer),
        }
    }
}

impl<T: 'static + Serialize> Serialize for KeyedSlabTree<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.tree.serialize(serializer)
    }
}

struct TreeNodeVisitor<'a, T> {
    level: usize,
    tree: &'a mut SlabTree<T>,
}

struct ChildrenVisitor<'a, T> {
    level: usize,
    tree: &'a mut SlabTree<T>,
}

impl<'a, 'de, T: Deserialize<'de>> Visitor<'de> for ChildrenVisitor<'a, T> {
    type Value = Vec<usize>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a tree node children")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut children = Vec::new();

        loop {
            let visitor = TreeNodeVisitor {
                tree: self.tree,
                level: self.level,
            };
            match seq.next_element_seed(visitor)? {
                Some(child) => children.push(child),
                None => break,
            }
        }
        Ok(children)
    }
}

impl<'a, 'de, T: Deserialize<'de>> DeserializeSeed<'de> for ChildrenVisitor<'a, T> {
    type Value = Vec<usize>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let children: Vec<usize> = deserializer.deserialize_seq(self)?;
        Ok(children)
    }
}

static KNOWN_FIELDS: &[&'static str] = &["record", "expanded", "children"];

impl<'a, 'de, T: Deserialize<'de>> Visitor<'de> for TreeNodeVisitor<'a, T> {
    type Value = usize;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a tree node")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut record: Option<T> = None;
        let mut expanded = false;
        let mut children: Option<Vec<usize>> = None;

        loop {
            let key: Cow<str> = match map.next_key()? {
                None => break,
                Some(key) => key,
            };

            match key.as_ref() {
                "expanded" => {
                    expanded = map.next_value()?;
                }
                "record" => {
                    record = Some(map.next_value()?);
                }
                "children" => {
                    children = Some(map.next_value_seed(ChildrenVisitor {
                        tree: self.tree,
                        level: self.level + 1,
                    })?);
                }
                unknown => {
                    return Err(A::Error::unknown_field(unknown, KNOWN_FIELDS));
                }
            }
        }

        let record = match record {
            Some(record) => record,
            None => {
                return Err(A::Error::missing_field("record"));
            }
        };

        let node_id = self.tree.slab.vacant_key();

        if let Some(children) = children.as_ref() {
            for child_id in children {
                let child = self.tree.slab.get_mut(*child_id).unwrap();
                child.parent_id = Some(node_id);
            }
        }

        let data = SlabTreeEntry {
            record,
            expanded,
            parent_id: None, // set by caller
            level: self.level,
            children: children,
        };

        assert!(self.tree.slab.insert(data) == node_id);

        Ok(node_id)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for SlabTree<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SlabTree<T>, D::Error> {
        let mut tree = SlabTree {
            slab: Slab::new(),
            root_id: None,
            version: 0,
        };

        let visitor = TreeNodeVisitor {
            tree: &mut tree,
            level: 0,
        };

        let root_id = visitor.deserialize(deserializer)?;

        tree.root_id = Some(root_id);

        Ok(tree)
    }
}

impl<'de, T: Deserialize<'de> + ExtractPrimaryKey> Deserialize<'de> for KeyedSlabTree<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<KeyedSlabTree<T>, D::Error> {
        let data = SlabTree::<T>::deserialize(deserializer)?;

        let mut tree = KeyedSlabTree::new();
        tree.set_root_tree(data);

        Ok(tree)
    }
}

impl<'a, 'de, T: Deserialize<'de>> DeserializeSeed<'de> for TreeNodeVisitor<'a, T> {
    type Value = usize;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct("TreeNode", KNOWN_FIELDS, self)
    }
}
