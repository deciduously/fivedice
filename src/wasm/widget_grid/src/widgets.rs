use crate::{
    traits::{Drawable, MountedWidget, Widget},
    types::Callback,
    window::WindowPtr,
    {Color, Point, Rc, Region, Result, VALUES},
};
use std::{marker::PhantomData, str::FromStr};
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
            phantom: PhantomData,
            text: s.into(),
        }
    }
}

impl<T> Clone for Text<T> {
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            text: self.text.clone(),
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
    type MSG = T;
    fn mount_widget(&self, top_left: Point) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new(top_left);
        ret.set_drawable(Box::new(self.clone()));
        ret
    }
    fn handle_click(&mut self, _: Point, _: Point, _: WindowPtr) -> Result<Option<Self::MSG>> {
        Ok(None)
    }
}

/// Generic button type.  Optionally takes a "bottom right" point as a width and height
/// Takes a callback to call upon click and a value to pass to the callback
// TODO buttons seem to only accept clicks under the text, not around the whole rectangle
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
    pub fn new(s: &str) -> Self {
        let mut ret = Self::default();
        ret.text = s.into();
        ret
    }

    /// Add a border color
    pub fn add_border_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    /// Set onclick action
    pub fn set_onclick(&mut self, f: Callback<T>) -> &mut Self {
        self.callback = Some(f);
        self
    }

    /// Set size
    pub fn set_size(&mut self, width: f64, height: f64) -> &mut Self {
        self.bottom_right = Some((width, height).into());
        self
    }
}

impl<T> Clone for Button<T>
where
    T: 'static,
{
    fn clone(&self) -> Self {
        let mut ret = Button::new(&self.text);
        if let Some(br) = self.bottom_right {
            ret.set_size(br.x, br.y);
        }
        if let Some(c) = &self.callback {
            ret.set_onclick(c.clone());
        }
        ret.add_border_color(self.color);
        ret
    }
}

impl<T> Default for Button<T> {
    fn default() -> Self {
        Self {
            bottom_right: None,
            callback: None,
            color: Color::from_str("black").unwrap(),
            text: "".into(),
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
    fn mount_widget(&self, top_left: Point) -> MountedWidget<Self::MSG> {
        let mut ret = MountedWidget::new(top_left);
        ret.set_drawable(Box::new(self.clone()));
        ret
    }
}
