macro_rules! impl_svg_position_attributes {

    () => {
        /// Builder style method to set the object position.
        pub fn position(mut self, x: impl Into<SvgLength>, y: impl Into<SvgLength>) -> Self {
            self.set_position(x, y);
            self
        }

        /// Method to set the object position.
        pub fn set_position(&mut self, x: impl Into<SvgLength>, y: impl Into<SvgLength>) {
            self.set_x(x);
            self.set_y(y);
        }

        /// Builder style method to set the object x position.
        pub fn x(mut self, x: impl Into<SvgLength>) -> Self {
            self.set_x(x);
            self
        }

        /// Method to set the object x.
        pub fn set_x(&mut self, x: impl Into<SvgLength>) {
            self.set_attribute("x", x.into());
        }

        /// Builder style method to set the object y position.
        pub fn y(mut self, y: impl Into<SvgLength>) -> Self {
            self.set_y(y);
            self
        }

        /// Method to set the object y.
        pub fn set_y(&mut self, y: impl Into<SvgLength>) {
            self.set_attribute("y", y.into());
        }
    }
}


macro_rules! impl_svg_presentation_attributes {
    () => {
        /// Builder style method to set the stroke width.
        pub fn stroke_width(mut self, w: impl Into<SvgLength>) -> Self {
            self.set_stroke_width(w);
            self
        }

        /// Method to set the stroke width.
        pub fn set_stroke_width(&mut self, w: impl Into<SvgLength>) {
            self.set_attribute("stroke-width", w.into());
        }

        /// Builder style method to set the stroke color/pattern.
        pub fn stroke(mut self, stroke: impl Into<AttrValue>) -> Self {
            self.set_stroke(stroke);
            self
        }

        /// Method to set the stroke color/pattern.
        pub fn set_stroke(&mut self, stroke: impl Into<AttrValue>) {
            self.set_attribute("stroke", stroke.into());
        }

        /// Builder style method to set the fill color/pattern.
        pub fn fill(mut self, fill: impl Into<AttrValue>) -> Self {
            self.set_fill(fill);
            self
        }

        /// Method to set the fill color/pattern.
        pub fn set_fill(&mut self, fill: impl Into<AttrValue>) {
            self.set_attribute("fill", fill.into());
        }

        /// Builder style method to set the object opacity.
        pub fn opacity(mut self, opacity: f64) -> Self {
            self.set_opacity(opacity);
            self
        }

        /// Method to set the object opacity.
        ///
        /// Opacity is the degree to which content behind an element
        /// is hidden, and is the opposite of transparency. Value are
        /// between 0 (invisible) and 1.0 (intransparent).
        pub fn set_opacity(&mut self, opacity: f64) {
            self.set_attribute("opacity", opacity.to_string());
        }

        /// Builder style method to set the stroke opacity.
        pub fn stroke_opacity(mut self, opacity: f64) -> Self {
            self.set_stroke_opacity(opacity);
            self
        }

        /// Method to set the stroke opacity.
        pub fn set_stroke_opacity(&mut self, opacity: f64) {
            self.set_attribute("stroke-opacity", opacity.to_string());
        }
    }
}

macro_rules! impl_svg_animation_attributes {
    () => {
        /// Builder style Method to add an animation.
        pub fn animate(mut self, animation: impl super::IntoSvgAnimation) -> Self {
            self.add_animate(animation);
            self
        }

        /// Method to add an animation.
        pub fn add_animate(&mut self, animation: impl super::IntoSvgAnimation) {
            match &mut self.children {
                Some(children) => children.push(animation.into_svg_animation()),
                None => self.children = Some(vec![animation.into_svg_animation()]),
            }
        }
    }
}

macro_rules! impl_svg_container_animation_attributes {
    () => {
        /// Builder style Method to add an animation.
        pub fn animate(mut self, animation: impl super::IntoSvgAnimation) -> Self {
            self.add_animate(animation);
            self
        }

        /// Method to add an animation.
        pub fn add_animate(&mut self, animation: impl super::IntoSvgAnimation) {
            self.children.push(animation.into_svg_animation());
        }
    }
}
