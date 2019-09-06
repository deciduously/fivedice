use crate::{
    traits::{Drawable, MountedWidget, Widget},
    types::Callback,
    window::WindowPtr,
    {Color, Point, Rc, Region, Result, VALUES},
};
use std::marker::PhantomData;
//
// Reusable Drawables
//

/// A widget that just draws some text
pub struct Text<T> {
    phantom: PhantomData<T>,
    text: String,
}

impl<T> Text<T> {
    pub fn new(s: &str) -> Self {
        Self {
            phantom: PhantomData, // TODO - really?  No better solution?
            text: s.into(),
        }
    }
}

impl<T> Drawable for Text<T> {
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> Result<Point> {
        w.begin_path();
        w.text(&self.text, &VALUES.get_font_string(), top_left)?;
        w.draw_path();
        Ok(Drawable::get_region(self, top_left, w)?.bottom_right())
    }

    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        Ok((
            top_left,
            w.text_width(&self.text)?,
            f64::from(VALUES.font_size),
        )
            .into())
    }
}

impl<T: 'static> Widget for Text<T> {
    type MSG = T; // TODO this is why you need the PhantomData
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        let t: Text<Self::MSG> = Text::new(&self.text);
        ret.set_drawable(Box::new(t));
        ret
    }
    fn handle_click(&mut self, _: Point, _: Point, _: WindowPtr) -> Result<Option<Self::MSG>> {
        Ok(None)
    }
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        Drawable::get_region(self, top_left, w)
    }
}

/// Generic button type.  Optionally takes a "bottom right" point as a width and height
/// Takes a callback to call upon click and a value to pass to the callback
// TODO buttons seem to only accent clicks under the text, not around the whole rectangle
#[derive(Clone)]
pub struct Button<T> {
    bottom_right: Option<Point>,
    callback: Option<Callback<T>>,
    color: Color,
    text: String,
}

impl<T> Button<T>
where
    T: 'static,
{
    pub fn new(
        s: &str,
        bottom_right: Option<Point>,
        color: Color,
        callback: Option<Callback<T>>,
    ) -> Self {
        Self {
            bottom_right,
            callback,
            color,
            text: s.into(),
        }
    }
}

impl<T> Drawable for Button<T> {
    fn draw_at(&self, top_left: Point, w: WindowPtr) -> Result<Point> {
        w.begin_path();
        let outline = Drawable::get_region(self, top_left, Rc::clone(&w))?;
        w.rect(outline, self.color);
        w.text(
            &self.text,
            &VALUES.get_font_string(),
            (
                top_left.x + (VALUES.padding / 2.0),
                top_left.y + (VALUES.padding * 2.0),
            )
                .into(),
        )?;
        w.draw_path();
        Ok(outline.bottom_right())
    }

    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        match self.bottom_right {
            Some(p) => Ok((top_left, p.x, p.y).into()),
            None => Ok((
                top_left,
                w.text_width(&self.text)? + VALUES.padding,
                f64::from(VALUES.font_size) + VALUES.padding * 2.0,
            )
                .into()),
        }
    }
}

impl<T: 'static> Widget for Button<T> {
    type MSG = T;
    fn get_region(&self, top_left: Point, w: WindowPtr) -> Result<Region> {
        Drawable::get_region(self, top_left, w)
    }
    fn handle_click(
        &mut self,
        top_left: Point,
        click: Point,
        w: WindowPtr,
    ) -> Result<Option<Self::MSG>> {
        if Drawable::get_region(self, top_left, w)?.contains(click) {
            match &self.callback {
                Some(f) => Ok(Some(f.call())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
    fn mount_widget(&self) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new();
        // TODO why can't you use the derived Clone??
        let self_clone = Button::new(
            &self.text,
            self.bottom_right,
            self.color,
            self.callback.clone(),
        );
        ret.set_drawable(Box::new(self_clone));
        ret
    }
}
