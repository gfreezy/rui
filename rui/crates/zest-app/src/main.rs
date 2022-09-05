use sycamore::prelude::*;

#[derive(Prop)]
struct TextButtonProps<'a, F> {
    text: &'a ReadSignal<String>,
    event: F,
}

#[component]
fn TextButton<'a, G: GenericNode, F: FnMut(G::EventType) + 'a>(
    cx: Scope<'a>,
    props: TextButtonProps<'a, F>,
) -> View<G> {
    view! { cx,
        listener(on:click=props.event) {
            text(text=props.text) {}
        }
    }
}

#[component]
fn App(cx: Scope) -> View<ZestNode> {
    let mut signal = create_signal(cx, 16);
    let title = create_memo(cx, || format!("click to {}", signal.get()));

    view! { cx,
        flex {
            listener(on:click=move |_| {
                println!("clicked");
                signal += 1;
            }) {
                text(text="click me") {}
            }


            TextButton(text=title, event=move |_| {
                signal += 1;
            })


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
