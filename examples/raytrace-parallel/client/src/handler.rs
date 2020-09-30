use std::convert::TryInto;
use std::error;
use std::fmt;

use futures::{FutureExt, StreamExt};
use gloo_timers::future::IntervalStream;
use serde_json::from_str;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yarte_wasm_app::DeLorean;

use utils::{console_error, console_log};

use crate::scene::{update_image, ImageData, Img, RenderingImage, Scene};
use crate::Msg::{EndRender, Error, Paint, UpdateTime};
use crate::RayTracing;

pub(crate) fn start_render(app: &mut RayTracing, addr: DeLorean<RayTracing>) {
    fn error_render<S: fmt::Display>(app: &mut RayTracing, now: f64, e: S) {
        error(app, e);
        bench_time(app, now);
        end_render(app);
    }

    if app.rendering {
        return;
    }
    console_log!("Start Render");
    disable_interface(app);
    let now = window().unwrap().performance().unwrap().now();
    let scene: Scene = match from_str(&app.scene.value()) {
        Ok(s) => s,
        Err(e) => return error_render(app, now, e),
    };
    let concurrency = app.n_concurrency;
    let RenderingImage { rx, partial, .. } = match scene.render(concurrency, app.pool()) {
        Ok(r) => r,
        Err(e) => return error_render(app, now, JsError(e)),
    };

    let width = partial.width;
    let height = partial.height;
    resize_canvas(app, width, height);

    let fut = async move {
        let render = rx.map(|image| {
            addr.send(
                image
                    .map_err(|_| Error(box "Ray Tracing"))
                    .and_then(|data| {
                        Img::new(data, width, height)
                            .try_into()
                            .map_err(|e| Error(box JsError(e)))
                    })
                    .map_or_else(
                        |e| {
                            bench_time_msg(now, addr);
                            e
                        },
                        Paint,
                    ),
            );
        });

        let mut stream = IntervalStream::new(1)
            .map(|_| {
                addr.send(
                    unsafe { partial.clone().into_image_data() }
                        .map_or_else(|e| Error(box JsError(e)), Paint),
                );
                bench_time_msg(now, addr);
            })
            .take_until(render);

        while stream.next().await.is_some() {}

        bench_time_msg(now, addr);
        addr.send(EndRender)
    };

    spawn_local(fut);
}

pub(crate) fn resize_canvas(RayTracing { canvas, .. }: &mut RayTracing, width: u32, height: u32) {
    console_log!("Resize canvas");
    canvas.set_width(width);
    canvas.set_height(height);
}

pub(crate) fn paint(RayTracing { ctx, .. }: &RayTracing, img: ImageData) {
    console_log!("Painting");
    update_image(&ctx, img);
}

// TODO: error enum
#[derive(Debug)]
pub(crate) struct JsError(JsValue);

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl error::Error for JsError {}

// TODO: param error enum
pub(crate) fn error<S: fmt::Display>(_app: &RayTracing, err: S) {
    // TODO: error message with app state
    // TODO: error message in screen
    console_error!("Error: {}", err)
}

pub(crate) fn update_time(RayTracing { time, .. }: &mut RayTracing, t: f64) {
    console_log!("Update Time");
    time.set_text_content(Some(&format!("{:.2} ms", t)));
}

pub(crate) fn bench(start: f64) -> Result<f64, &'static str> {
    let now = window().unwrap().performance().unwrap().now();
    if start < now {
        Ok(now - start)
    } else {
        Err("now is after than start")
    }
}

pub(crate) fn bench_time(app: &mut RayTracing, start: f64) {
    match bench(start) {
        Ok(t) => update_time(app, t),
        Err(e) => error(app, e),
    }
}

pub(crate) fn bench_time_msg(start: f64, addr: DeLorean<RayTracing>) {
    addr.send(bench(start).map_or_else(|e| Error(box e), UpdateTime))
}

pub(crate) fn end_render(app: &mut RayTracing) {
    console_log!("End Render");

    enable_interface(app);
}

pub(crate) fn update_concurrency(app: &mut RayTracing, s: String) {
    console_log!("Update Concurrency");

    app.n_concurrency = match usize::from_str_radix(&s, 10) {
        Ok(i) => i,
        Err(_) => return error(app, "parse concurrency number"),
    };

    app.concurrency_amt.set_text_content(Some(&s));
}

pub(crate) fn disable_interface(
    RayTracing {
        button,
        concurrency,
        rendering,
        ..
    }: &mut RayTracing,
) {
    *rendering = true;
    button.set_disabled(true);
    concurrency.set_disabled(true);
    console_log!("Disable inputs");
}

pub(crate) fn enable_interface(
    RayTracing {
        button,
        concurrency,
        rendering,
        ..
    }: &mut RayTracing,
) {
    button.set_disabled(false);
    concurrency.set_disabled(false);
    *rendering = false;
    console_log!("Enable inputs");
}
