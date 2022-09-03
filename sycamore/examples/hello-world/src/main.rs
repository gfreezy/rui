use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View<ZestNode> {
    view! { cx,
        text(text="Hello World!") {

        }
    }
}

fn main() {
    sycamore::run(|root| {
        sycamore::render_to(
            |cx| {
                view! { cx, App {} }
            },
            root,
        );
    });
}
