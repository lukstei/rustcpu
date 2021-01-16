use graphics::{Rectangle, line_from_to};
use std::cmp::max;
use crate::game::{PosF, Connector, FunctionBox, Collide, Draw, ConnectorDirection, Update, State, FunctionBoxRef, DrawCtx};
use vecmath::vec2_sub;
use crate::ui::{rgba, draw_arc_centered};

pub struct FunctionBoxDraw<'a> {
    idx: FunctionBoxRef,
    function_box: &'a FunctionBox,
    rect: [f64; 4],
    padding: f64,
    connector_radius: f64,
    connector_margin: f64,
    highlighted: bool,
    highlighted_connector: Option<Connector>,
}

impl<'a> FunctionBoxDraw<'a> {
    pub(crate) fn new(function_box: &'a FunctionBox, idx: FunctionBoxRef) -> Self {
        let padding = 20.;
        let connector_radius = 6.5;
        let connector_margin = 8.;

        let height = 2. * padding + 40.;
        let width = 2. * padding - connector_margin + ((connector_margin + connector_radius * 2.) * max(function_box.outputs.len(), function_box.inputs.len()) as f64);
        let rect = [function_box.position[0], function_box.position[1], width, height];

        FunctionBoxDraw {
            function_box,
            idx,
            rect,
            padding,
            connector_radius,
            connector_margin,
            highlighted: false,
            highlighted_connector: None,
        }
    }

    pub fn connector_position(&self, connector: &Connector) -> PosF {
        let i = connector.idx as f64;

        [
            self.rect[0] + self.padding + (i * self.connector_margin + i * self.connector_radius * 2. + self.connector_radius),
            self.rect[1] + (self.rect[3] * (if matches!(connector.direction, ConnectorDirection::Input) { 1. } else { 0. }))
        ]
    }

    pub fn draw_connection_line(&self, connector: &Connector, target: PosF, ctx: &mut DrawCtx) {
        let bg = rgba(99, 110, 114, 1.0);

        line_from_to(bg, 1., self.connector_position(connector), target, ctx.c.transform, ctx.g);
    }

}

pub enum FunctionBoxCollideDesc {
    FunctionBox,
    Connector(Connector),
}

impl Collide for FunctionBoxDraw<'_> {
    type CollideDesc = FunctionBoxCollideDesc;

    fn collide(&self, point: [f64; 2]) -> Option<FunctionBoxCollideDesc> {
        let [x1, y1] = point;

        if let Some((_i, conn)) =
        self.function_box.outputs.iter().enumerate()
            .chain(self.function_box.inputs.iter().enumerate())
            .find(|(i, x)| {
                let [x2, y2] = self.connector_position(x);

                f64::sqrt((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) <= self.connector_radius
            }) {
            Some(FunctionBoxCollideDesc::Connector(conn.clone()))
        } else if x1 >= self.rect[0]
            && x1 <= self.rect[0] + self.rect[2]
            && y1 >= self.rect[1]
            && y1 <= self.rect[1] + self.rect[3] {
            Some(FunctionBoxCollideDesc::FunctionBox)
        } else {
            None
        }
    }
}

impl Draw for FunctionBoxDraw<'_> {
    fn draw(&self, ctx: &mut DrawCtx) {
        let bg_color = if self.highlighted { rgba(253, 203, 110, 1.0) } else { rgba(9, 132, 227, 1.0) };
        let mut rectangle = Rectangle::new_round_border(bg_color, 3., 0.);
        rectangle = rectangle.color(bg_color);
        rectangle.draw_tri(self.rect, &Default::default(), ctx.c.transform, ctx.g);

        let fb = self.function_box;
        // draw connectors
        fb.outputs.iter()
            .chain(fb.inputs.iter())
            .for_each(| c| {
                let highlighted = self.highlighted_connector.as_ref().map_or(false, |x| { *x == *c });
                let bg = rgba(99, 110, 114, 1.0);
                draw_arc_centered(self.connector_position(c),
                                  self.connector_radius, bg, highlighted, ctx);
            });
    }
}

impl Update for FunctionBoxDraw<'_> {
    fn update(&mut self, state: &State) {
        let i = self.idx;
        if state.mouse_button1_pressed {
            if let Some((fb, hpos)) = state.dragged_function_box {
                if i == fb {
                    self.highlighted = true;
                }
            } else if let Some((fb, conn, hpos)) = state.dragged_connector.clone() {
                if i == fb {
                    self.highlighted_connector = Some(conn);
                } else if let Some(FunctionBoxCollideDesc::Connector(connector)) =
                self.collide(state.mouse_position) {
                    self.highlighted_connector = Some(connector);
                }
            }
        }
    }
}