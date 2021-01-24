use crate::game::{Update, State, Collide, Draw, DrawCtx, PosF, Entity};
use crate::ui::{rgba, draw_text_centered};
use graphics::Rectangle;
use crate::util::rect_center;

#[derive(Debug)]
pub struct Button {
    rect: [f64; 4],
    text: String,
    pressed: bool,
    was_pressed: bool,
    highlighted: bool,
}

impl Button {
    pub fn new(text: String, pos: PosF) -> Self {
        Button {
            rect: [pos[0], pos[1], 70., 35.],
            text,
            was_pressed: false,
            pressed: false,
            highlighted: false,
        }
    }

    pub fn pressed(&mut self) -> bool {
        if self.pressed {
            self.pressed = false;
            self.was_pressed = true;
            true
        } else {
            false
        }
    }
}

impl Collide for Button {
    type CollideDesc = ();

    fn collide(&self, point: [f64; 2]) -> Option<Self::CollideDesc> {
        self.rect.collide(point)
    }
}

impl Entity for Button {}

impl Update for Button {
    fn update(&mut self, state: &State) {
        if state.mouse_button1_pressed {
            if let Some(()) = self.collide(state.mouse_position) {
                if !self.was_pressed {
                    self.pressed = true;
                }
            }
        } else {
            self.was_pressed = false;
        }

        if let Some(()) = self.collide(state.mouse_position) {
            self.highlighted = true
        } else {
            self.highlighted = false
        }
    }
}

impl Draw for Button {
    fn draw(&self, ctx: &mut DrawCtx) {
        let mut rectangle = Rectangle::new_round_border(rgba(45, 52, 54, 1.0), 2., 2.);
        rectangle = rectangle.color(if self.highlighted { rgba(253, 203, 110, 1.0) } else { rgba(178, 190, 195, 1.0) });
        rectangle.draw_tri(self.rect, &Default::default(), ctx.c.transform, ctx.g);
        draw_text_centered(&self.text, 20, rect_center(self.rect), rgba(45, 52, 54, 1.0), ctx);
    }
}