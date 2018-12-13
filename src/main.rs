#![feature(uniform_paths)]
#![recursion_limit="128"]
#[macro_use]
extern crate stdweb;
use stdweb::{unstable::{TryInto, TryFrom}, web::{self, CanvasRenderingContext2d, document, Element, INode, html_element::*}};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;

fn create<E>(element: &str, add: bool) -> E
where E: TryFrom<Element> + web::INode,
<E as TryFrom<Element>>::Error: Debug {
    let element: E = document().create_element(element).unwrap().try_into().unwrap();
    if add {
        document().body().unwrap().append_child(&element);
    }
    element
}

fn log(text: &str) {
    let node = document().create_text_node(text);
    document().body().unwrap().append_child(&node);
}

#[derive(Default)]
struct State {
    x: f64, 
    y: f64,
    zoom: f64,
    kind: Kind
}

#[derive(PartialEq)]
enum Kind {
    Released {
        dx: f64,
        dy: f64,
    },
    Dragged,
}

impl std::default::Default for Kind {
    fn default() -> Self {
        Kind::Released {
            dx: 0.,
            dy: 0.,
        }
    }
}

fn main() {
    let canvas = create::<CanvasElement>("canvas", true);
    let canvas_width = 500;
    let canvas_height = 500;
    canvas.set_width(canvas_width);
    canvas.set_height(canvas_height);
    let image = create::<ImageElement>("img", false);
    image.set_src("Agenta2.png");
    let ctx = canvas.get_context::<CanvasRenderingContext2d>().unwrap();
    let state = Rc::new(RefCell::new(State::default()));
    let update = {
        let state = state.clone();
        let image = image.clone();
        move |dt: f64| {
            let state = state.borrow();
            let image_width = image.width() as f64;
            let image_height = image.height() as f64;;
            ctx.draw_image(image.clone(), state.x, state.y).unwrap();
            ctx.draw_image(image.clone(), state.x, state.y - image_height).unwrap();
            ctx.draw_image(image.clone(), state.x - image_width, state.y).unwrap();
            ctx.draw_image(image.clone(), state.x - image_width, state.y - image_height).unwrap();
        }
    };
    let mouse_down = {
        let state = state.clone();
        move || {
            let mut state = state.borrow_mut();
            state.kind = Kind::Dragged;
        }
    };
    let mouse_up = {
        let state = state.clone();
        move || {
            let mut state = state.borrow_mut();
            state.kind = Kind::Released {
                dx: 0.,
                dy: 0.,
            }
        }
    };
    let mouse_move = {
        let state = state.clone();
        move |x: f64, y: f64| {
            let mut state = state.borrow_mut();
            if state.kind == Kind::Dragged {
                let image_width = image.width() as f64;
                let image_height = image.height() as f64;;
                let canvas_width = canvas_width as f64;
                let canvas_height = canvas_height as f64;
                state.x += x;
                if state.x < -(image_width - canvas_width) {
                    state.x += image_width;
                }
                if state.x > image_width {
                    state.x -= image_width;
                }
                state.y += y;
                if state.y < -(image_height - canvas_height) {
                    state.y += image_height;
                }
                if state.x > image_height {
                    state.y -= image_height;
                }
            }
        }
    };
    js!{ @(no_return)
        function update(dt) {
            @{update}(dt);
            window.requestAnimationFrame(update);
        }
        let canvas = @{canvas};
        canvas.addEventListener("mousedown", (e) => {
            @{mouse_down}();
        });
        canvas.addEventListener("mouseup", (e) => {
            @{mouse_up}();
        });
        canvas.addEventListener("mousemove", (e) => {
            @{mouse_move}(e.movementX, e.movementY);
        });
        update(0);
    }
}