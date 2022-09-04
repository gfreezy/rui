use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View<ZestNode> {
    let mut signal = create_signal(cx, 0);
    view! { cx,
        flex {
            listener(on:click=move |_| {
                println!("clicked");
                signal += 1;
            }) {
                text(text="click me") {}
            }
            text(text=format!("hello {signal}"), font-size=signal) { }
            text(text="Hello World!") { }
            text(text="Hello World!") { }
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
