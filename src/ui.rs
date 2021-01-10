extern crate piston;
extern crate opengl_graphics;
extern crate graphics;

extern crate glutin_window;

use opengl_graphics::{GlGraphics, OpenGL};
use graphics::{Context, Graphics};
use std::collections::HashMap;
use piston::window::{AdvancedWindow, Window, WindowSettings};
use piston::input::*;
use piston::event_loop::*;

use glutin_window::GlutinWindow as AppWindow;
use std::f64::consts::PI;
use crate::{FunctionBox, Container};
use std::cmp::max;
use self::graphics::{Rectangle, color, CircleArc};
use self::graphics::rectangle::{square, Border};
use self::graphics::types::{Color, Radius};
use self::piston::Position;
use self::graphics::math::Vec2d;
use std::cell::RefCell;
use std::ptr;

pub type PosF = Vec2d;
type OurGraphics = GlGraphics;

struct DrawCtx<'a> {
    g: &'a mut GlGraphics,
    c: &'a Context,
    window: &'a dyn Window,
}

struct State<'a> {
    container: Container,
    highlighted_function_box: Option<&'a FunctionBox>
}

pub fn ui_main() {
    let opengl = OpenGL::V3_2;
    let mut window: AppWindow = WindowSettings::new("piston-example-user_input", [1024, 768])
        .exit_on_esc(true).graphics_api(opengl).build().unwrap();

    let ref mut gl = GlGraphics::new(opengl);
    let mut cursor = [0.0, 0.0];
    let mut window_size = window.draw_size();

    let mut state = State {
        container: Container::new(),
        highlighted_function_box: None
    };
    let and_box = state.container.add(FunctionBox::new("and", [50., 50.], vec!["i1".into(), "i2".into()], vec!["and".into()]));
    let one_box = state.container.add(FunctionBox::new("1",  [50., 200.], vec![], vec!["1".into()]));
    state.container.connect(one_box, "1".into(), and_box, "i1".into());

    state.highlighted_function_box = Some(&state.container.graph[and_box]);

    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }
        if let Some(Button::Keyboard(key)) = e.press_args() {
            println!("Pressed keyboard key '{:?}'", key);
        };
        if let Some(args) = e.button_args() {
            println!("Scancode {:?}", args.scancode);
        }
        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(key) => println!("Released keyboard key '{:?}'", key),
                Button::Mouse(button) => println!("Released mouse button '{:?}'", button),
                Button::Controller(button) => println!("Released controller button '{:?}'", button),
                Button::Hat(hat) => println!("Released controller hat `{:?}`", hat),
            }
        };
        e.mouse_cursor(|pos| {
            cursor = pos;
            println!("Mouse moved '{} {}'", pos[0], pos[1]);
        });
        e.mouse_relative(|d| println!("Relative mouse moved '{} {}'", d[0], d[1]));
        e.resize(|args| {
            println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]);
            window_size = args.draw_size.into();
        });
        if let Some(cursor) = e.cursor_args() {
            if cursor { println!("Mouse entered"); } else { println!("Mouse left"); }
        };
        if let Some(args) = e.render_args() {
            // println!("Render {}", args.ext_dt);
            gl.draw(args.viewport(), |c, g| {
                graphics::clear(rgba(178, 190, 195, 1.0), g);
                let mut ctx = DrawCtx {
                    g,
                    c: &c,
                    window: &window,
                };
                draw(&state, &mut ctx);
            },
            );
        }
        if let Some(_args) = e.idle_args() {}
        if let Some(_args) = e.update_args() {}
    }
}

fn rgba(r: i32, g: i32, b: i32, a: f32) -> Color {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255., a]
}

fn draw(
    state: &State,
    ctx: &mut DrawCtx
) {
    state.container.graph.raw_nodes().iter().map(|x| {
        let mut draw = FunctionBoxDraw::new(&x.weight);
        if let Some(highlighted) = state.highlighted_function_box {
            draw.highlighted = ptr::eq(highlighted, &x.weight);
        }
        draw
    }).for_each(|d| {
        d.draw(ctx)
    })
}

struct FunctionBoxDraw<'a> {
    function_box: &'a FunctionBox,
    rect: [f64; 4],
    padding: f64,
    connector_radius: f64,
    connector_margin: f64,
    highlighted: bool,

}

impl<'a> FunctionBoxDraw<'a> {
    fn new(function_box: &'a FunctionBox) -> Self {
        let padding = 20.;
        let connector_radius = 6.5;
        let connector_margin = 8.;

        let height = 2. * padding + 40.;
        let width = 2. * padding - connector_margin + ((connector_margin + connector_radius*2.) * max(function_box.outputs.len(), function_box.inputs.len()) as f64);
        let rect = [function_box.position[0], function_box.position[1], width, height];

        FunctionBoxDraw {
            function_box,
            rect,
            padding,
            connector_radius,
            connector_margin,
            highlighted: false
        }
    }
}

trait Collide {
    type CollideDesc;
    fn collide(&self, point: PosF) -> Option<Self::CollideDesc>;
}

trait Draw {
    fn draw(&self, ctx: &mut DrawCtx);
}

enum FunctionBoxCollideDesc {
    FunctionBox,
    Connector(String)
}

impl Collide for FunctionBoxDraw<'_> {
    type CollideDesc = FunctionBoxCollideDesc;

    fn collide(&self, point: [f64; 2]) -> Option<FunctionBoxCollideDesc> {
        None
    }
}

impl Draw for FunctionBoxDraw<'_> {
    fn draw(&self, ctx: &mut DrawCtx) {
        let bg_color = if self.highlighted { rgba(253, 203, 110,1.0) } else { rgba(9, 132, 227, 1.0) };
        let mut rectangle = Rectangle::new_round_border(bg_color, 3., 0.);
        rectangle = rectangle.color(bg_color);
        rectangle.draw_tri(self.rect, &Default::default(), ctx.c.transform, ctx.g);

        let fb = self.function_box;

        let mut draw_arc = |i, y| {
            draw_arc_centered([
                                  fb.position[0] + self.padding + (i*self.connector_margin + i*self.connector_radius*2. + self.connector_radius),
                                  y],
                              self.connector_radius, rgba(99, 110, 114, 1.0), ctx);
        };

        // draw connectors
        fb.outputs.iter().enumerate().for_each(|(i, name)| {
            draw_arc(i as f64, fb.position[1]);
        });
        fb.inputs.iter().enumerate().for_each(|(i, name)| {
            draw_arc(i as f64, fb.position[1] + self.rect[3]);
        });
    }
}

fn draw_arc_centered(center: PosF, circle_radius: Radius, color: Color, ctx: &mut DrawCtx) {
    CircleArc::new(color, circle_radius/2., 0., 2.*PI)
        .draw_tri([center[0] - circle_radius, center[1] - circle_radius, circle_radius*2., circle_radius*2.], &Default::default(), ctx.c.transform, ctx.g);
}