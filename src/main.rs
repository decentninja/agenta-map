#![feature(uniform_paths)]
#![recursion_limit="512"]
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

struct State {
    x: f64, 
    y: f64,
    kind: Kind
}

impl std::default::Default for State {
    fn default() -> Self {
        State {
            x: 0.,
            y: 0.,
            kind: Kind::Released
        }
    }
}

#[derive(PartialEq)]
enum Kind {
    Released,
    Dragged {
        start: (f64, f64)
    },
}

fn main() {
    let canvas = create::<CanvasElement>("canvas", true);
    let image = create::<ImageElement>("img", false);
    image.set_src("agenta3.png");
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
        move |x: f64, y: f64| {
            let mut state = state.borrow_mut();
            state.kind = Kind::Dragged { start: (x - state.x, y - state.y) };
        }
    };
    let mouse_up = {
        let state = state.clone();
        move || {
            let mut state = state.borrow_mut();
            state.kind = Kind::Released;
        }
    };
    let mouse_move = {
        let state = state.clone();
        let canvas = canvas.clone();
        move |x: f64, y: f64| {
            let mut state = state.borrow_mut();
            if let Kind::Dragged { start } = state.kind {
                let image_width = image.width() as f64;
                let image_height = image.height() as f64;;
                let canvas_width = canvas.width() as f64;
                let canvas_height = canvas.height() as f64;
                state.x = x - start.0;
                if state.x < -(image_width - canvas_width) {
                    state.x += image_width;
                }
                if state.x > image_width {
                    state.x -= image_width;
                }
                state.y = y - start.1;
                if state.y < -(image_height - canvas_height) {
                    state.y += image_height;
                }
                if state.y > image_height {
                    state.y -= image_height;
                }
            }
        }
    };
    js!{ @(no_return)
        document.body.style.margin = 0;
        let legend = document.createElement("img");
        legend.src = "legend.png";
        legend.style.position = "absolute";
        legend.style.right = 0;
        legend.style.bottom = 0;
        legend.style.margin = "30px";
        legend.style["pointer-events"] = "none";
        document.body.appendChild(legend);
        document.body.style.overflow = "hidden";
    }
    js!{ @(no_return)
        let canvas = @{canvas};
        canvas.style.cursor = "grab";
        function update(dt) {
            @{update}(dt);
            window.requestAnimationFrame(update);
        }
        function resize() {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
            update(0);
        }
        let mouse_down = @{mouse_down};
        let mouse_up = @{mouse_up};
        let mouse_move = @{mouse_move};
        canvas.addEventListener("mousedown", (e) => {
            canvas.style.cursor = "grabbing";
        });
        canvas.addEventListener("mouseup", (e) => {
            mouse_up();
            canvas.style.cursor = "grab";
        });
        canvas.addEventListener("mousemove", (e) => {
            mouse_move(e.screenX, e.screenY);
        });
        canvas.addEventListener("touchstart", (e) => {
            mouse_down(e.touches[0].screenX, e.touches[0].screenY);
        });
        canvas.addEventListener("touchend", (e) => {
            mouse_up();
        });
        canvas.addEventListener("touchmove", (e) => {
            mouse_move(e.touches[0].screenX, e.touches[0].screenY);
        });
        canvas.addEventListener("mouseout", (e) => {
            mouse_up();
        });
        window.addEventListener("resize", resize);
        resize();
    }
}