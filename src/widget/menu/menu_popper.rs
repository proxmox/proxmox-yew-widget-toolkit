use yew::prelude::*;

use crate::widget::align::{AlignOptions, GrowDirection, Point};

pub struct MenuPopper {
    content_ref: NodeRef,
    submenu_ref: NodeRef,
    align_options: Option<AlignOptions>,
    prefer_bottom: bool,
}

impl MenuPopper {
    pub fn new(content_ref: NodeRef, submenu_ref: NodeRef, prefer_bottom: bool) -> Self {
        Self {
            content_ref,
            submenu_ref,
            prefer_bottom,
            align_options: None,
        }
    }

    pub fn update(&mut self) {
        if self.align_options.is_none() {
            self.align_options = Some(if self.prefer_bottom {
                AlignOptions::new(Point::BottomStart, Point::TopStart, GrowDirection::StartEnd)
                    .align_width(true)
                    .viewport_padding(5.0)
                    .with_fallback_placement(
                        Point::TopEnd,
                        Point::TopStart,
                        GrowDirection::TopBottom,
                    )
                    .with_fallback_placement(
                        Point::TopStart,
                        Point::TopEnd,
                        GrowDirection::TopBottom,
                    )
            } else {
                AlignOptions::new(Point::TopEnd, Point::TopStart, GrowDirection::TopBottom)
                    .viewport_padding(5.0)
                    .align_width(true)
                    .with_fallback_placement(
                        Point::TopStart,
                        Point::TopEnd,
                        GrowDirection::TopBottom,
                    )
            });
        }

        if let Err(err) = crate::widget::align::align_to(
            &self.content_ref,
            &self.submenu_ref,
            self.align_options.clone(),
        ) {
            log::error!("could not position menu: {}", err.to_string());
        }
    }
}
