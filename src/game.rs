use opengl_graphics::{GlGraphics, GlyphCache};
use piston::{Size, Window};
use graphics::{Context, line_from_to, color};
use std::ops::IndexMut;
use vecmath::vec2_sub;
use graphics::math::Vec2d;
use petgraph::{Direction, Graph};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::game::ConnectorDirection::{Output, Input};
use petgraph::matrix_graph::NodeIndex;
use crate::function_box_draw::{FunctionBoxCollideDesc, FunctionBoxDraw, output_input_pair};
use petgraph::prelude::EdgeRef;
use std::borrow::Borrow;

pub type PosF = Vec2d;
type OurGraphics = GlGraphics;
pub type FunctionBoxRef = NodeIndex<u32>;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectorDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct Connector {
    pub name: String,
    pub direction: ConnectorDirection,
    pub idx: usize,
}

impl Connector {
    fn new(name: String, direction: ConnectorDirection, idx: usize) -> Connector {
        Connector {
            name,
            direction,
            idx,
        }
    }
}

impl Display for Connector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.name)
    }
}

impl PartialEq for Connector {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.direction == other.direction
    }
}

#[derive(Debug)]
pub struct FunctionBox {
    pub name: String,
    pub inputs: Vec<Connector>,
    pub outputs: Vec<Connector>,

    pub position: PosF,
}

impl FunctionBox {
    pub fn get_input_connector(&self, name: &str) -> &Connector {
        self.inputs.iter().find(|x| { x.name == name }).unwrap()
    }
    pub fn get_output_connector(&self, name: &str) -> &Connector {
        self.outputs.iter().find(|x| { x.name == name }).unwrap()
    }

    pub(crate) fn new(name: &str, position: PosF, inputs: Vec<String>, outputs: Vec<String>) -> FunctionBox {
        FunctionBox {
            name: name.into(),
            inputs: inputs.into_iter().enumerate().map(|(i, n)| Connector::new(n, ConnectorDirection::Input, i)).collect(),
            outputs: outputs.into_iter().enumerate().map(|(i, n)| Connector::new(n, ConnectorDirection::Output, i)).collect(),
            position,
        }
    }

    fn connectors(&self) -> impl Iterator<Item=&Connector> {
        self.inputs.iter().chain(self.outputs.iter())
    }
}


#[derive(Debug)]
pub struct Container {
    pub graph: Graph<FunctionBox, Vec<(Connector, Connector)>>
}

impl Container {
    pub(crate) fn new() -> Container {
        Container {
            graph: Graph::new()
        }
    }

    pub(crate) fn add(&mut self, function_box: FunctionBox) -> FunctionBoxRef {
        self.graph.add_node(function_box)
    }

    pub fn can_connect(&self, c1: (FunctionBoxRef, &Connector), c2: (FunctionBoxRef, &Connector)) -> bool {
        if let Some(((output_ref, output_connector), (input_ref, input_connector))) = output_input_pair(c1, c2) {
            assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
            assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
            assert!(matches!(output_connector.direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
            assert!(matches!(input_connector.direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
            self.graph.edges_directed(input_ref, Direction::Incoming).into_iter()
                .find(|x| {
                    x.weight().iter().find(|(outp, inp)| { inp == input_connector }).is_some()
                }).is_none()
        } else {
            false
        }
    }

    pub fn connect(&mut self, output: (FunctionBoxRef, &Connector), input: (FunctionBoxRef, &Connector)) {
        let ((output_ref, output_connector), (input_ref, input_connector)) = (output, input);

        assert!(self.graph[output_ref].outputs.contains(&output_connector), "unknown output {}", output_connector);
        assert!(self.graph[input_ref].inputs.contains(&input_connector), "unknown input {}", input_connector);
        assert!(matches!(output_connector.direction, ConnectorDirection::Output), "wrong direction {}", output_connector);
        assert!(matches!(input_connector.direction, ConnectorDirection::Input), "wrong direction {}", input_connector);
        assert!(self.can_connect(output, input), "not connectable {} -> {}", output_connector, input_connector);

        let edge_ref = self.graph.find_edge(output_ref, input_ref).unwrap_or(self.graph.add_edge(output_ref, input_ref, Vec::new()));
        let vec = self.graph.index_mut(edge_ref);

        let new_edge = (output_connector.clone(), input_connector.clone());
        if !vec.contains(&new_edge) {
            vec.push(new_edge)
        }
    }
}


#[derive(Debug)]
pub enum EntityKind {
    FunctionBox,
    Connector,
}

#[derive(Debug)]
pub struct State {
    pub mouse_position: PosF,
    pub mouse_button1_pressed: bool,
    pub mouse_delta: PosF,
    pub window_size: Size,

    pub container: Container,
    pub dragged_entity_kind: Option<EntityKind>,
    pub dragged_function_box: Option<(FunctionBoxRef, PosF)>,
    pub dragged_connector: Option<(FunctionBoxRef, Connector, PosF)>,
    pub dragged_connector_target: Option<(FunctionBoxRef, Connector, PosF)>,
}

pub trait Update {
    fn update(&mut self, state: &State);
}

pub trait Collide {
    type CollideDesc;
    fn collide(&self, point: PosF) -> Option<Self::CollideDesc>;
}

pub trait Draw {
    fn draw(&self, ctx: &mut DrawCtx);
}

pub struct DrawCtx<'a, 'b> {
    pub g: &'a mut GlGraphics,
    pub c: &'a Context,
    pub window: &'a dyn Window,
    pub font_normal: &'a mut GlyphCache<'b>,
}

pub(crate) fn draw(
    state: &mut State,
    ctx: &mut DrawCtx,
) {
    if !state.mouse_button1_pressed {
        if let (Some((fb1, c1, _)), Some((fb2, c2, _))) = (&state.dragged_connector, &state.dragged_connector_target) {
            let (output, input) = output_input_pair((*fb1, c1), (*fb2, c2)).unwrap();
            println!("Connect {:?} to {:?}", output, input);
            state.container.connect(output, input);
        }

        state.dragged_function_box = None;
        state.dragged_connector = None;
        state.dragged_connector_target = None;
        state.dragged_entity_kind = None;
    }

    state.container.graph.node_indices().for_each(|i| {
        if state.mouse_button1_pressed {
            if let Some((fb, hpos)) = state.dragged_function_box {
                if i == fb {
                    let pos = &mut state.container.graph.index_mut(i).position;
                    *pos = vec2_sub(state.mouse_position, hpos);
                    println!("Position {:?}", pos);
                }
            }
        }
    });

    state.dragged_connector_target = None;
    state.container.graph.node_indices().for_each(|i| {
        let draw = FunctionBoxDraw::new(&state.container.graph[i], i);

        if state.mouse_button1_pressed {
            let origin = vec2_sub(state.mouse_position, state.container.graph[i].position);

            match (&state.dragged_entity_kind, draw.collide(state.mouse_position)) {
                (None, Some(FunctionBoxCollideDesc::FunctionBox)) => {
                    if let None = state.dragged_function_box {
                        state.dragged_entity_kind = Some(EntityKind::FunctionBox);
                        state.dragged_function_box = Some((i, origin));
                    }
                }
                (None, Some(FunctionBoxCollideDesc::Connector(connector))) => {
                    state.dragged_entity_kind = Some(EntityKind::Connector);
                    state.dragged_connector = Some((i, connector, origin));
                }
                (Some(EntityKind::Connector), Some(FunctionBoxCollideDesc::Connector(connector))) => {
                    let option = state.dragged_connector.as_ref().unwrap();
                    if state.container.can_connect((option.0, &option.1), (i, &connector)) {
                        state.dragged_connector_target = Some((i, connector, origin));
                    }
                }
                _ => {}
            }
        }

        let mut draw = FunctionBoxDraw::new(&state.container.graph[i], i);

        draw.update(state);
        draw.draw(ctx);

        state.container.graph.edges_directed(i, Direction::Outgoing)
            .for_each(|e| {
                e.weight().iter().for_each(|(c1, c2)| {
                    let d2 = FunctionBoxDraw::new(&state.container.graph[e.target()], e.target());

                    draw.draw_connection_line(c1, d2.connector_position(c2), ctx)
                })
            });

        if let Some((i2, c, o)) = &state.dragged_connector {
            if i == *i2 {
                draw.draw_connection_line(c, state.mouse_position, ctx);
            }
        }
    });
}
