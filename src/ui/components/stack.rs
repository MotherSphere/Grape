#![allow(dead_code)]

use iced::event::{self, Event};
use iced::layout;
use iced::mouse;
use iced::renderer;
use iced::widget::tree::Tree;
use iced::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget};

#[allow(missing_debug_implementations)]
pub struct Stack<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::core::Renderer,
{
    width: Length,
    height: Length,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Stack<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
{
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    pub fn with_children(children: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            children,
        }
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, Message, Renderer> Default for Stack<'a, Message, Renderer>
where
    Renderer: iced::core::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Stack<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let mut max_size = Size::ZERO;
        let mut nodes = Vec::with_capacity(self.children.len());

        for (child, state) in self.children.iter().zip(&mut tree.children) {
            let node = child.as_widget().layout(state, renderer, &limits);
            let size = node.size();
            max_size.width = max_size.width.max(size.width);
            max_size.height = max_size.height.max(size.height);
            nodes.push(node);
        }

        let size = limits.resolve(self.width, self.height, max_size);
        let children = nodes
            .into_iter()
            .map(|node| node.move_to(Point::ORIGIN))
            .collect();

        layout::Node::with_children(size, children)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let mut status = event::Status::Ignored;
        let child_layouts: Vec<_> = layout.children().collect();

        for index in (0..self.children.len()).rev() {
            let child = &mut self.children[index];
            let state = &mut tree.children[index];
            let layout = child_layouts[index];

            status = event::Status::merge(
                status,
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                ),
            );

            if status == event::Status::Captured {
                break;
            }
        }

        status
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<iced::overlay::Element<'b, Message, Theme, Renderer>> {
        iced::overlay::from_children(&mut self.children, tree, layout, renderer, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Stack<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::core::Renderer + 'a,
{
    fn from(stack: Stack<'a, Message, Theme, Renderer>) -> Self {
        Element::new(stack)
    }
}
