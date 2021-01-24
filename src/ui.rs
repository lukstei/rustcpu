extern crate piston;
extern crate opengl_graphics;
extern crate graphics;
extern crate glutin_window;


use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::mem::replace;
use std::ops::IndexMut;
use std::ptr;
use std::ptr::eq;

use glutin_window::GlutinWindow as AppWindow;
use graphics::{Context, Graphics};
use opengl_graphics::{GlGraphics, OpenGL};
use petgraph::graph::{Node, NodeIndex};
use piston::event_loop::*;
use piston::input::*;
use piston::window::{AdvancedWindow, Window, WindowSettings};
use vecmath::{col_mat3x2_transform_pos2, mat2x3_add, mat2x3_sub, row_mat2x3_mul, vec2_add, vec2_mul, vec2_sub};

use crate::button::Button;
use crate::connector::Connector;
use crate::container::Container;
use crate::function_box::FunctionBox;
use crate::game::{DrawCtx, Entities, PosF};
use crate::game;

use self::graphics::{CharacterCache, CircleArc, color, line_from_to, Rectangle, Text};
use self::graphics::math::{translate, Vec2d};
use self::graphics::rectangle::{Border, square};
use self::graphics::types::{Color, FontSize, Radius};
use self::opengl_graphics::GlyphCache;
use self::piston::{Button as PistonButton, Position, Size};

pub fn rgba(r: i32, g: i32, b: i32, a: f32) -> Color {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255., a]
}

pub fn ui_main() {
    let opengl = OpenGL::V3_2;
    let mut window: AppWindow = WindowSettings::new("piston-example-user_input", [1024, 768])
        .exit_on_esc(true).graphics_api(opengl).build().unwrap();

    let ref mut gl = GlGraphics::new(opengl);

    let mut font_normal = GlyphCache::new("assets/FiraSans-Regular.ttf", (), opengl_graphics::TextureSettings::new()).unwrap();

    let mut entities = Entities {
        add_fb_button: Box::new(Button::new("+".into(), [50., 500.])),
        save_button: Box::new(Button::new("Save".into(), [50. + 1.*(70. +5.), 500.])),
        load_button: Box::new(Button::new("Load".into(), [50. + 2.*(70. +5.), 500.]))
    };

    let mut state = crate::game::State {
        container: Container::new(),
        mouse_button1_pressed: false,
        mouse_position: [0., 0.],
        mouse_delta: [0., 0.],
        window_size: Size { width: 0., height: 0. },
        dragged_function_box: None,
        dragged_connector: None,
        dragged_connector_target: None,
        dragged_entity_kind: None,
    };
    let and_box = state.container.add(FunctionBox::new("nand", [50., 50.], vec!["i1".into(), "i2".into()], vec!["and".into()]));
    let one_box = state.container.add(FunctionBox::new("1", [50., 200.], vec![], vec!["1".into()]));
    let graph = &state.container.graph;
    state.container.connect((one_box, graph[one_box].get_output_connector("1").idx), (and_box, graph[and_box].get_input_connector("i1").idx));

    let mut mouse_position = state.mouse_position;
    let mut mouse_delta = state.mouse_delta;
    let mut mouse_button1_pressed = state.mouse_button1_pressed;
    let mut window_size = state.window_size;

    let mut events = Events::new(EventSettings::new().lazy(true));
    while let Some(e) = events.next(&mut window) {
        if let Some(PistonButton::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
            if let MouseButton::Left = button {
                mouse_button1_pressed = true;
            }
        }
        if let Some(PistonButton::Keyboard(key)) = e.press_args() {
            println!("Pressed keyboard key '{:?}'", key);
        };
        if let Some(args) = e.button_args() {
            println!("Scancode {:?}", args.scancode);
        }
        if let Some(button) = e.release_args() {
            match button {
                PistonButton::Keyboard(key) => println!("Released keyboard key '{:?}'", key),
                PistonButton::Mouse(button) => {
                    println!("Released mouse button '{:?}'", button);
                    if let MouseButton::Left = button {
                        mouse_button1_pressed = false;
                    }
                }
                PistonButton::Controller(button) => println!("Released controller button '{:?}'", button),
                PistonButton::Hat(hat) => println!("Released controller hat `{:?}`", hat),
            }
        };
        e.mouse_cursor(|pos| {
            println!("Mouse moved '{} {}'", pos[0], pos[1]);
            mouse_position = pos;
        });
        e.mouse_relative(|d| {
            println!("Relative mouse moved '{} {}'", d[0], d[1]);
            mouse_delta = d;
        });
        e.resize(|args| {
            println!("Resized '{}, {}'", args.window_size[0], args.window_size[1]);
            window_size = args.draw_size.into();
        });
        if let Some(cursor) = e.cursor_args() {
            if cursor { println!("Mouse entered"); } else { println!("Mouse left"); }
        };
        if let Some(args) = e.render_args() {
            // println!("Render {}", args.ext_dt);

            state.window_size = window_size;
            state.mouse_position = mouse_position;
            state.mouse_button1_pressed = mouse_button1_pressed;
            state.mouse_delta = mouse_delta;

            gl.draw(args.viewport(), |c, g| {
                graphics::clear(rgba(178, 190, 195, 1.0), g);
                let mut ctx = DrawCtx {
                    g,
                    c: &c,
                    window: &window,
                    font_normal: &mut font_normal,
                };
                game::update_entities(&mut entities, &mut state);
                game::update(&mut state);
                game::draw(&state, &mut ctx);
                game::draw_entities(&entities, &state, &mut ctx);
            },
            );
        }
        if let Some(_args) = e.idle_args() {}
        if let Some(_args) = e.update_args() {}
    }
}

pub fn draw_text_centered(text: &str, font_size: FontSize, pos: PosF, color: Color, ctx: &mut DrawCtx) {
    let text_dims = measure_text(text, font_size, ctx.font_normal);
    Text::new_color(color, font_size)
        .draw(text, ctx.font_normal, &Default::default(),
              row_mat2x3_mul(ctx.c.transform,
                             translate(vec2_sub(pos, vec2_mul(text_dims, [0.5, 0.5])))), ctx.g);
}

pub fn measure_text_old(text: &str, font_size: FontSize, font: &mut GlyphCache) -> Vec2d {
    text.chars().into_iter()
        .map(|x| { font.character(font_size, x).unwrap().advance_size })
        .fold([0., 0.], |s, x| { vec2_add(s, x) })
}

pub fn measure_text(text: &str, font_size: FontSize, font: &mut GlyphCache) -> Vec2d {
    [text.chars().into_iter()
        .map(|x| { font.character(font_size, x).unwrap().advance_width() })
        .sum(),
        text.chars().next().map_or(0., |x| -font.character(font_size, x).unwrap().top())]
}

pub fn draw_arc_centered(center: PosF, circle_radius: Radius, color: Color, ctx: &mut DrawCtx) {
    CircleArc::new(color, circle_radius, 0., 2. * PI)
        .draw_tri([center[0] - circle_radius, center[1] - circle_radius, circle_radius * 2., circle_radius * 2.], &Default::default(), ctx.c.transform, ctx.g);
}

