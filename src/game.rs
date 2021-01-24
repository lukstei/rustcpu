use core::fmt;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Formatter};
use std::iter::Take;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use graphics::{color, Context, line_from_to};
use graphics::math::Vec2d;
use opengl_graphics::{GlGraphics, GlyphCache};
use petgraph::{Direction, Graph};
use petgraph::algo::connected_components;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::prelude::EdgeRef;
use piston::{Size, Window};
use vecmath::vec2_sub;

use crate::button::Button;
use crate::connector::{Connector, ConnectorDirection};
use crate::connector::ConnectorDirection::{Input, Output};
use crate::container::{ConnectorRef, Container, FBGraph, FunctionBoxRef};
use crate::function_box::FunctionBox;
use crate::function_box_draw::{FunctionBoxCollideDesc, FunctionBoxDraw, output_input_pair};
use std::fs::File;
use std::io::Write;
use std::error::Error;

pub type PosF = Vec2d;
type OurGraphics = GlGraphics;


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
    pub dragged_connector: Option<(FunctionBoxRef, ConnectorRef, PosF)>,
    pub dragged_connector_target: Option<(FunctionBoxRef, ConnectorRef, PosF)>,
}

pub struct Entities {
    pub add_fb_button: Box<Button>,
    pub save_button: Box<Button>,
    pub load_button: Box<Button>,
}

impl Entities {
    /*pub fn iter(&self) -> impl Iterator<Item=&Box<dyn Draw>> {
        Box::new(vec!(&self.add_fb_button).into_iter())
    }*/
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

pub(crate) fn update_entities(
    entities: &mut Entities,
    state: &mut State,
) {
    entities.add_fb_button.update(state);
    entities.save_button.update(state);
    entities.load_button.update(state);

    if entities.add_fb_button.pressed() {
        println!("Pressed");
        state.container.add(FunctionBox::new("nand", [100., 50.], vec!["i1".into(), "i2".into()], vec!["nand".into()]));
    }
    if entities.save_button.pressed() {
        let json = serde_json::to_string_pretty(&state.container).unwrap();
        println!("JSON: {}", json);
        File::create("save.json").unwrap().write(json.as_bytes()).unwrap();
    }
    if entities.load_button.pressed() {
        match File::open("save.json").map_err(|e| e.to_string())
            .and_then(|file| serde_json::from_reader(file).map_err(|e| e.to_string())) {
            Ok(graph) => state.container = graph,
            Err(e) => println!("Error loading state: {}", e)
        }
    }
}

pub(crate) fn draw_entities(
    entities: &Entities,
    state: &State,
    ctx: &mut DrawCtx,
) {
    entities.add_fb_button.draw(ctx);
    entities.save_button.draw(ctx);
    entities.load_button.draw(ctx);
}

pub(crate) fn update(
    state: &mut State,
) {
    update_general_states(state);
    update_fb_states(state);
}

pub(crate) fn update_general_states(
    state: &mut State,
) {
    if !state.mouse_button1_pressed {
        if let (Some((fb1, c1, _)), Some((fb2, c2, _))) = (state.dragged_connector, state.dragged_connector_target) {
            let (output, input) = output_input_pair(&state.container.graph, (fb1, c1), (fb2, c2)).unwrap();
            println!("Connect {:?} to {:?}", output, input);
            state.container.connect(output, input);
            println!("New graph {:?}", state.container.graph);
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
                    let connector1 = &state.container.graph[i].connectors[connector];
                    if let ConnectorDirection::Input = connector1.direction {
                        state.container.disconnect((i, connector));
                    }

                    state.dragged_entity_kind = Some(EntityKind::Connector);
                    state.dragged_connector = Some((i, connector, origin));
                }
                (Some(EntityKind::Connector), Some(FunctionBoxCollideDesc::Connector(connector))) => {
                    let option = state.dragged_connector.as_ref().unwrap();
                    if state.container.can_connect((option.0, option.1), (i, connector)) {
                        state.dragged_connector_target = Some((i, connector, origin));
                    }
                }
                _ => {}
            }
        }
    })
}


pub(crate) fn update_fb_states(
    state: &mut State,
) {
    let generation = 1 + state.container.graph.raw_nodes().first().map_or(0, |x| x.weight.generation);

    state.container.graph.node_indices().for_each(|x| {
        if state.container.graph[x].generation < generation {
            calculate_and_set_state(&mut state.container.graph, x);
            state.container.graph.index_mut(x).generation = generation;
        }
    });

    state.container.graph.node_indices().for_each(|i| {
        assert_eq!(state.container.graph[i].generation, generation)
    });
}


pub(crate) fn draw(
    state: &State,
    ctx: &mut DrawCtx,
) {
    state.container.graph.node_indices().for_each(|i| {
        let mut draw = FunctionBoxDraw::new(&state.container.graph[i], i);

        draw.update(state);
        draw.draw(ctx);

        state.container.graph.edges_directed(i, Direction::Outgoing)
            .for_each(|e| {
                e.weight().iter().for_each(|&(c1, c2)| {
                    let d2 = FunctionBoxDraw::new(&state.container.graph[e.target()], e.target());

                    draw.draw_connection_line(&state.container.graph[e.source()].connectors[c1], d2.connector_position(&state.container.graph[e.target()].connectors[c2]), ctx)
                })
            });

        if let Some((i2, c, o)) = &state.dragged_connector {
            if i == *i2 {
                draw.draw_connection_line(&state.container.graph[i].connectors[*c], state.mouse_position, ctx);
            }
        }
    });
}

fn calculate_and_set_state(graph: &mut FBGraph, x: NodeIndex) {
    let result_state: bool;
    {
        match graph.index(x).name.as_str() {
            "nand" => {
                result_state = !graph.index(x).inputs_iter().fold(true, |x, y| { x && y.state });
            }
            "1" => {
                result_state = true;
            }
            "0" => {
                result_state = false;
            }
            "output_toggle" => {
                unimplemented!()
                //result_state = x1.state;
            }
            _ => panic!("Unknown function {:?}", graph.index(x).name)
        }
    }

    // TODO: for now only one output supported
    graph.index_mut(x).outputs_iter_mut()
        .for_each(|x| { x.state = result_state });

    let mut neighbors = graph.neighbors_directed(x, Direction::Outgoing)
        .detach();
    while let Some((edge, node)) = neighbors.next(graph) {
        (0..graph[edge].len()).for_each(|i| {
            let connector_idx = graph[edge][i].1;
            graph.node_weight_mut(node).unwrap().connectors[connector_idx].state = result_state;
        });
    }
}

