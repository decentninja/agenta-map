#![feature(uniform_paths)]
#![recursion_limit="256"]
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

#[derive(Default)]
struct State {
    x: f64, 
    y: f64,
    kind: Kind
}

#[derive(PartialEq)]
enum Kind {
    Released {
        inertia: (f64, f64),
    },
    Dragged {
        last_move: (f64, f64),
    }
}

impl std::default::Default for Kind {
    fn default() -> Self {
        Kind::Released {
            inertia: (0., 0.),
        }
    }
}

fn main() {
    let canvas = create::<CanvasElement>("canvas", true);
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
            state.kind = Kind::Dragged {
                last_move: (0., 0.)
            };
        }
    };
    let mouse_up = {
        let state = state.clone();
        move || {
            let mut state = state.borrow_mut();
            state.kind = Kind::Released {
                inertia: (0.,0.),
            }
        }
    };
    let mouse_move = {
        let state = state.clone();
        let canvas = canvas.clone();
        move |x: f64, y: f64| {
            let mut state = state.borrow_mut();
            if let Kind::Dragged { .. } = state.kind {
                let image_width = image.width() as f64;
                let image_height = image.height() as f64;;
                let canvas_width = canvas.width() as f64;
                let canvas_height = canvas.height() as f64;
                let orig = (state.x, state.y);
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
                if state.y > image_height {
                    state.y -= image_height;
                }
                state.kind = Kind::Dragged {
                    last_move: (orig.0 - state.x, orig.1 - state.y)
                };
            }
        }
    };
    js!{ @(no_return)
        document.body.style.margin = 0;
    }
    js!{ @(no_return)
        let canvas = @{canvas};
        function update(dt) {
            @{update}(dt);
            window.requestAnimationFrame(update);
        }
        function resize() {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
            update(0);
        }
        canvas.addEventListener("mousedown", (e) => {
            @{mouse_down}();
        });
        canvas.addEventListener("mouseup", (e) => {
            console.log(e);
            @{mouse_up}();
        });
        canvas.addEventListener("mousemove", (e) => {
            @{mouse_move}(e.movementX, e.movementY);
        });
        window.addEventListener("resize", resize);
        resize();
    }
}