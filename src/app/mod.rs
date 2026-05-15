mod constants;
mod outline;
mod plot_item;
mod ui;
mod view_options;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_app() -> eframe::Result {
    use crate::app::ui::App;
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Satisfactory World Generator",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, std::env::args().nth(1).as_deref())))),
    )
}

#[cfg(target_arch = "wasm32")]
pub fn run_app() -> eframe::Result {
    use crate::app::ui::App;
    use eframe::wasm_bindgen::JsCast as _;

    // redirect `log` message to `console.log`
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    Ok(Box::new(App::new(
                        cc,
                        Some(&cc.integration_info.web_info.location.url),
                    )))
                }),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });

    eframe::Result::Ok(())
}
