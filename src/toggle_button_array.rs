use crate::game::{Update, State, Collide, Draw, DrawCtx, PosF, Entity};
use crate::ui::{rgba, draw_text_centered};
use graphics::Rectangle;
use crate::util::rect_center;
use crate::function_box_draw::FunctionBoxDraw;
use crate::container::ConnectorRef;
use crate::button::Button;
use vecmath::vec2_add;

pub struct ToggleButtonsDraw<'a> {
    function_box_draw: &'a FunctionBoxDraw<'a>,

    buttons: Vec<Button>,
}

impl<'a> ToggleButtonsDraw<'a> {
    pub fn new(function_box_draw: &'a FunctionBoxDraw<'a>) -> Self {
        ToggleButtonsDraw {
            function_box_draw,
            buttons: function_box_draw.function_box.connectors.iter().map(|c| {
                Button::new("x".into(), vec2_add(function_box_draw.connector_position(c), [0., 20.]))
            }).collect(),
        }
    }
}

impl Entity for ToggleButtonsDraw<'_> {}

impl Collide for ToggleButtonsDraw<'_>  {
    type CollideDesc = ConnectorRef;

    fn collide(&self, point: [f64; 2]) -> Option<Self::CollideDesc> {
        self.buttons.iter().enumerate().find_map(|(i,x)| {
            x.collide(point).map(|_|i)
        }).map(|i| self.function_box_draw.function_box.connectors[i].idx)
    }
}

impl Update for ToggleButtonsDraw<'_>  {
    fn update(&mut self, state: &State) {
        self.buttons.iter_mut().for_each(|b| {
            b.update(state);
        });
    }
}

impl Draw for ToggleButtonsDraw<'_>  {
    fn draw(&self, ctx: &mut DrawCtx) {
        self.buttons.iter().for_each(|x| x.draw(ctx));
    }
}