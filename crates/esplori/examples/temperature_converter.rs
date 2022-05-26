use rui::{
    app::{AppLauncher, WindowDesc},
    live_s,
    menu::mac::menu_bar,
    prelude::*,
    ui::Ui,
};

#[memoize_attr::memoize]
fn win(ui: &mut Ui) {
    let temp = ui.state_node(|| 0.0f64);

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
            textbox(
                ui,
                format!("{}", ui[temp]),
                move |val| {
                    if let Ok(cel) = val.parse() {
                        tracing::debug!("c: {}", cel);
                        temp.update(|c| *c = cel);
                    }
                },
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

            text(ui, "Celsius = ".to_string(), live_s!(ui, ""));
            textbox(
                ui,
                format!("{}", ui[temp] * 1.8 + 32.0),
                move |val| {
                    if let Ok(fah) = val.parse::<f64>() {
                        tracing::debug!("f: {}", fah);
                        temp.update(|c| *c = (fah - 32.) / 1.8);
                    }
                },
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
            text(ui, "Fahrenheit".to_string(), live_s!(ui, ""));
        },
    );
}

fn main() {
    let desc = WindowDesc::new("app".to_string(), move |ui| win(ui)).menu(|_| menu_bar());
    let app = AppLauncher::with_windows(vec![desc]).log_to_console();
    app.launch().unwrap();
}
