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
