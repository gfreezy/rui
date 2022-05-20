use rui::{
    app::{AppLauncher, WindowDesc},
    live_s,
    menu::mac::menu_bar,
    prelude::*,
    ui::Ui,
};

fn win(ui: &mut Ui) {
    let count = ui.state_node(|| 0usize);
    println!("win");

    flex(
        ui,
        live_s!(
            ui,
            r#"
    .counter {
        axis: horizontal;
        main-axis-alignment: center;
        cross-axis-alignment: center;
    }
    "#
        ),
        |ui| {
            ui.memoize(comp, ui[count], |_| {});

            text(
                ui,
                &format!("{}", ui[count]),
                live_s!(
                    ui,
                    r#"
                .a {
                    color: rgb(43, 130, 190);
                    font-size: 50.0;
                }
            "#
                ),
            );

            button(
                ui,
                "Increment",
                move || {
                    count.update(|c| *c += 1);
                },
                live_s!(ui, ""),
            );
        },
    );
}

fn comp(ui: &mut Ui, count: usize, _content: impl FnOnce(&mut Ui)) {
    println!("comp");
    column(ui, live_s!(ui, ""), |ui| {
        println!("comp column");
        text(
            ui,
            &format!("{}", count),
            live_s!(
                ui,
                r#"
    .a {
        color: rgb(43, 130, 190);
        font-size: 50.0;
    }
"#
            ),
        );
        let count = ui.state_node(|| 0usize);
        flex(
            ui,
            live_s!(
                ui,
                r#"
    .counter {
        axis: horizontal;
        main-axis-alignment: center;
        cross-axis-alignment: center;
    }
    "#
            ),
            |ui| {
                text(
                    ui,
                    &format!("{}", ui[count]),
                    live_s!(
                        ui,
                        r#"
                .a {
                    color: rgb(43, 130, 190);
                    font-size: 50.0;
                }
            "#
                    ),
                );

                button(
                    ui,
                    "Increment Subcomponent",
                    move || {
                        count.update(|c| *c += 1);
                    },
                    live_s!(ui, ""),
                );
            },
        );
    });
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), move |ui| win(ui)).menu(|_| menu_bar());
    let app = AppLauncher::with_windows(vec![desc]).log_to_console();
    app.launch().unwrap();
}
