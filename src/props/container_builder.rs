use yew::virtual_dom::{VList, VNode};

pub trait ContainerBuilder: Into<VNode> {
    fn as_children_mut(&mut self) -> &mut Vec<VNode>;

    /// Builder style method to add a child node
    fn with_child(mut self, child: impl Into<VNode>) -> Self {
        self.add_child(child);
        self
    }

    /// Builder style method to add a (optional) child node
    ///
    /// Note: This adds and empty VList for child is None to make VDom diff more stable.
    /// (Also see bug: <https://github.com/yewstack/yew/issues/2654>)
    fn with_optional_child(mut self, child: Option<impl Into<VNode>>) -> Self {
        self.add_optional_child(child);
        self
    }

    /// Method to add a (optional) child node
    ///
    /// Note: This adds and empty VList for child is None to make VDom diff more stable.
    /// (Also see bug: <https://github.com/yewstack/yew/issues/2654>)
    fn add_optional_child(&mut self, child: Option<impl Into<VNode>>) {
        if let Some(child) = child {
            self.add_child(child);
        } else {
            self.add_child(VList::new());
        }
    }

    /// Method to add a child node
    fn add_child(&mut self, child: impl Into<VNode>) {
        self.as_children_mut().push(child.into());
    }

    /// Builder style method to add multiple children
    fn children(mut self, child: impl IntoIterator<Item = VNode>) -> Self {
        self.add_children(child);
        self
    }

    /// Method to add multiple children.
    fn add_children(&mut self, children: impl IntoIterator<Item = VNode>) {
        self.as_children_mut().extend(children);
    }

    /// Method to set the children property.
    fn set_children(&mut self, children: impl IntoIterator<Item = VNode>) {
        let list = self.as_children_mut();
        list.clear();
        list.extend(children);
    }
}
