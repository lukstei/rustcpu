use graphics::{Rectangle, line_from_to};
use std::cmp::max;
use crate::game::{PosF, Connector, FunctionBox, Collide, Draw, ConnectorDirection, Update, State, FunctionBoxRef, DrawCtx, ConnectorRef};
use vecmath::{vec2_sub, vec2_add};
use crate::ui::{rgba, draw_arc_centered, draw_text_centered};
use opengl_graphics::OpenGL;
use petgraph::Direction;

pub struct ConnectorDraw<'a> {
    idx: usize,
    connector: &'a Connector,
    highlighted: bool,
    connected: bool,
}

impl<'a> ConnectorDraw<'a> {
    pub fn new(connector: &'a Connector, idx: usize) -> ConnectorDraw {
        ConnectorDraw {
            connector,
            idx,
            highlighted: false,
            connected: false,
        }
    }
}

pub struct FunctionBoxDraw<'a> {
    idx: FunctionBoxRef,
    function_box: &'a FunctionBox,
    rect: [f64; 4],
    padding: f64,
    connector_radius: f64,
    connector_margin: f64,
    highlighted: bool,
    connector_draws: Vec<ConnectorDraw<'a>>,
}

impl<'a> FunctionBoxDraw<'a> {
    pub(crate) fn new(function_box: &'a FunctionBox, idx: FunctionBoxRef) -> Self {
        let padding = 20.;
        let connector_radius = 4.;
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
            connector_draws: function_box.outputs.iter().enumerate().chain(function_box.inputs.iter().enumerate())
                .map(|(i, c)| ConnectorDraw::new(c, i)).collect(),
        }
    }

    pub fn connector_draw_idx(&self, connector: &Connector) -> usize {
        let idx = match connector.direction {
            ConnectorDirection::Output => connector.idx,
            ConnectorDirection::Input => (self.function_box.outputs.len() + connector.idx)
        };
        idx
    }

    pub fn connector_position(&self, connector: &Connector) -> PosF {
        let i = connector.idx as f64;

        [
            self.rect[0] + self.padding + (i * self.connector_margin + i * self.connector_radius * 2. + self.connector_radius),
            self.rect[1] + (self.rect[3] * (if matches!(connector.direction, ConnectorDirection::Input) { 1. } else { 0. }))
        ]
    }

    pub fn draw_connection_line(&self, connector: &Connector, target: PosF, ctx: &mut DrawCtx) {
        let bg = if connector.state { rgba(214, 48, 49, 1.0) } else { rgba(99, 110, 114, 1.0) };

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

        self.connector_draws.iter()
            .for_each(|c| {
                let pos = self.connector_position(c.connector);
                if c.highlighted {
                    draw_arc_centered(pos,
                                      self.connector_radius, rgba(253, 203, 110, 1.0), ctx);
                } else {
                    draw_arc_centered(pos,
                                      self.connector_radius, if c.connector.state { rgba(214, 48, 49, 1.0) } else { rgba(99, 110, 114, 1.0) }, ctx);

                    if !c.connected {
                        draw_arc_centered(pos,
                                          self.connector_radius / 2., rgba(178, 190, 195, 1.0), ctx);
                    }
                }
                draw_text_centered(&c.connector.name, 14,
                                   vec2_add(pos, [0., if matches!(c.connector.direction, ConnectorDirection::Input) { -13. } else { 20. }]), rgba(223, 230, 233, 1.0), ctx);
            });
    }
}

impl<'a> Update for FunctionBoxDraw<'a> {
    fn update(&mut self, state: &State) {
        let i = self.idx;
        if let Some((fb, hpos)) = &state.dragged_function_box {
            if i == *fb {
                self.highlighted = true;
            }
        }
        if let Some((fb, conn, hpos)) = &state.dragged_connector {
            if i == *fb {
                let idx = self.connector_draw_idx(conn);
                self.connector_draws[idx].highlighted = true;
            }
        }
        if let (Some((fb1, c1, _)), Some((fb2, c2, _))) = (&state.dragged_connector, &state.dragged_connector_target) {
            if i == *fb2 {
                if let Some((output, input)) = output_input_pair((*fb1, c1), (*fb2, c2)) {
                    if state.container.can_connect(output, input) {
                        let idx = self.connector_draw_idx(c2);
                        self.connector_draws[idx].highlighted = true;
                    }
                }
            }
        }
        state.container.graph.edges_directed(self.idx, Direction::Outgoing)
            .flat_map(|x| { x.weight().iter().map(|y| &y.0) })
            .chain(state.container.graph.edges_directed(self.idx, Direction::Incoming).flat_map(|x| { x.weight().iter().map(|y| &y.1) }))
            .for_each(|x| {
                let idx = self.connector_draw_idx(x);
                self.connector_draws[idx].connected = true;
            })
    }
}

pub fn output_input_pair(c1: (FunctionBoxRef, ConnectorRef), c2: (FunctionBoxRef, ConnectorRef)) -> Option<((FunctionBoxRef, ConnectorRef), (FunctionBoxRef, ConnectorRef))> {
    if c1.1.direction != c2.1.direction {
        let output = if matches!(c1.1.direction, ConnectorDirection::Output) { c1 } else { c2 };
        let input = if matches!(c1.1.direction, ConnectorDirection::Output) { c2 } else { c1 };
        Some((output, input))
    } else {
        None
    }
}