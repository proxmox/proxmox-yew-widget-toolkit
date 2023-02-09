
macro_rules!impl_svg_presentation_attributes{
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
    }
}
