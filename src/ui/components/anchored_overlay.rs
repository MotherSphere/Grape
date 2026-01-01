use iced::event::{self, Event};
use iced::mouse;
use iced::overlay;
use iced::widget;
use iced::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Theme,
    Vector,
};

#[allow(missing_debug_implementations)]
pub struct AnchoredOverlay<'a, Message> {
    content: Element<'a, Message>,
    overlay: Element<'a, Message>,
    gap: f32,
}

impl<'a, Message> AnchoredOverlay<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        overlay: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            content: content.into(),
            overlay: overlay.into(),
            gap: 6.0,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl<'a, Message> widget::Widget<Message, Theme, iced::Renderer>
    for AnchoredOverlay<'a, Message>
where
    Message: 'a,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.content),
            widget::Tree::new(&self.overlay),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget(), self.overlay.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::layout::Limits,
    ) -> iced::layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let mut children = tree.children.iter_mut();
        let content_overlay = self.content.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
            translation,
        );
        let anchored_overlay = Some(overlay::Element::new(Box::new(Overlay {
            position: layout.position() + translation,
            content_bounds: layout.bounds(),
            overlay: &self.overlay,
            overlay_state: children.next().unwrap(),
            gap: self.gap,
        })));

        match (content_overlay, anchored_overlay) {
            (None, None) => None,
            (Some(content), None) => Some(content),
            (None, Some(overlay)) => Some(overlay),
            (Some(content), Some(overlay)) => Some(
                overlay::Group::with_children(vec![content, overlay]).overlay(),
            ),
        }
    }
}

impl<'a, Message> From<AnchoredOverlay<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(overlay: AnchoredOverlay<'a, Message>) -> Self {
        Element::new(overlay)
    }
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    content_bounds: Rectangle,
    overlay: &'b Element<'a, Message>,
    overlay_state: &'b mut widget::Tree,
    gap: f32,
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, iced::Renderer>
    for Overlay<'a, 'b, Message>
where
    Message: 'a,
{
    fn layout(
        &mut self,
        renderer: &iced::Renderer,
        bounds: Size,
    ) -> iced::layout::Node {
        let viewport = Rectangle::with_size(bounds);
        let overlay_layout = self.overlay.as_widget().layout(
            self.overlay_state,
            renderer,
            &iced::layout::Limits::new(Size::ZERO, Size::INFINITY),
        );
        let overlay_bounds = overlay_layout.bounds();
        let mut target_bounds = Rectangle {
            x: self.position.x,
            y: self.position.y + self.content_bounds.height + self.gap,
            width: overlay_bounds.width,
            height: overlay_bounds.height,
        };

        if target_bounds.x + target_bounds.width > viewport.width {
            target_bounds.x = viewport.width - target_bounds.width;
        }
        if target_bounds.x < viewport.x {
            target_bounds.x = viewport.x;
        }

        iced::layout::Node::with_children(
            target_bounds.size(),
            vec![overlay_layout],
        )
        .translate(Vector::new(target_bounds.x, target_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.overlay.as_widget().draw(
            self.overlay_state,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITY),
        );
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.overlay.as_widget_mut().on_event(
            self.overlay_state,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::with_size(Size::INFINITY),
        )
    }
}
