use serde::{Serialize, Serializer, Deserialize};
use serde::ser::{SerializeStruct, SerializeSeq};

use super::{SlabTree, SlabTreeNodeRef};

struct SlabTreeChildList<'a, T> {
    children: &'a [usize],
    tree: &'a SlabTree<T>,
}

impl<'a, T: 'static + Serialize> Serialize for SlabTreeChildList<'a, T> {

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
        let entry = self.get();

        state.serialize_field("record", &entry.record)?;
        state.serialize_field("expanded", &entry.expanded)?;

        if let Some(children) = &entry.children {
            let children = SlabTreeChildList {
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



// Tree { root: TreeNode { record: {}, children: [ TreeNode { }, ... ] }
// { record: {}, children: [ record: { }, ... ] }
