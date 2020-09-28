use std::fmt;

use futures::{FutureExt, StreamExt};
use gloo_timers::future::IntervalStream;
use serde_json::from_str;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yarte_wasm_app::A;

use utils::{console_error, console_log};

use crate::scene::{put_image, put_image_ptr, RenderingImage, Scene, UnsafeImg};
use crate::Msg::{EndRender, Error, Paint, UnsafePaint, UpdateTime};
use crate::{Img, RayTracing};

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

pub(crate) fn start_render(app: &mut RayTracing, addr: A<RayTracing>) {
    if app.rendering {
        return;
    }
    console_log!("Start Render");
    disable_interface(app);
    let now = window().unwrap().performance().unwrap().now();
    let scene: Scene = match from_str(&app.scene.value()) {
        Ok(s) => s,
        Err(e) => return error(app, e),
    };
    let concurrency = app.n_concurrency;
    let RenderingImage { rx, partial, .. } = match scene.render(concurrency, app.pool()) {
        Ok(r) => r,
        Err(e) => {
            error(app, format!("Worker pool\n{:?}", e));
            return end_render(app, now);
        }
    };

    let width = partial.width;
    let height = partial.height;

    let fut = async move {
        let render = rx.map(|image| {
            let msg = match image.ok() {
                Some(data) => Paint(Img {
                    data,
                    width,
                    height,
                }),
                None => Error(Box::new("Ray tracing")),
            };

            addr.send(msg);
        });

        let mut stream = IntervalStream::new(1000 / 24)
            .map(|_| {
                addr.send(UpdateTime(now));
                addr.send(UnsafePaint(unsafe { partial.clone() }));
            })
            .take_until(render);

        while stream.next().await.is_some() {}

        addr.send(EndRender(now))
    };

    spawn_local(fut);
}

/// Paint image in ptr
///
/// # Safety
/// Assume [`ptr`, `ptr` + `len`] is a valid image data of `width x height`
pub(crate) unsafe fn unsafe_paint(app: &RayTracing, img: UnsafeImg) {
    console_log!("Unsafe painting");
    if let Err(e) = put_image_ptr(img) {
        error(app, format!("{:?}", e));
    }
}

pub(crate) fn paint(app: &RayTracing, img: Img) {
    console_log!("Painting");
    if let Err(e) = put_image(img) {
        error(app, format!("{:?}", e));
    }
}

pub(crate) fn error<S: fmt::Display>(_app: &RayTracing, err: S) {
    // TODO: Some error message with app state
    console_error!("Error: {}", err)
}

pub(crate) fn update_time(app: &mut RayTracing, start: f64) {
    console_log!("Update Time");

    let now = window().unwrap().performance().unwrap().now();
    if start < now {
        app.time
            .set_text_content(Some(&format!("{:.2} ms", now - start)));
    } else {
        error(app, Box::new("now is after start"));
    }
}

pub(crate) fn end_render(app: &mut RayTracing, start: f64) {
    console_log!("End Render");

    update_time(app, start);
    enable_interface(app);
}

pub(crate) fn update_concurrency(app: &mut RayTracing, s: String) {
    console_log!("Change Concurrency");

    app.n_concurrency = match usize::from_str_radix(&s, 10) {
        Ok(i) => i,
        Err(_) => return error(app, "parse concurrency number"),
    };

    app.concurrency_amt.set_text_content(Some(&s));
}
