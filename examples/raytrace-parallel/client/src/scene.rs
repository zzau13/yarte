use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};

use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use js_sys::{Uint8ClampedArray, WebAssembly};
use rayon::prelude::*;
use serde::{Deserialize, Deserializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

use worker_pool::WorkerPool;

// Inline the definition of `ImageData` here because `web_sys` uses
// `&Clamped<Vec<u8>>`, whereas we want to pass in a JS object here.
#[wasm_bindgen]
extern "C" {
    pub type ImageData;

    #[wasm_bindgen(constructor, catch)]
    fn new(data: &Uint8ClampedArray, width: f64, height: f64) -> Result<ImageData, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = updateImage)]
    pub fn update_image(ctx: &CanvasRenderingContext2d, data: ImageData);
}

pub(crate) struct Scene {
    inner: raytracer::scene::Scene,
}

// TODO: From form
impl<'de> Deserialize<'de> for Scene {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        raytracer::scene::Scene::deserialize(deserializer).map(|inner| Scene { inner })
    }
}

impl Scene {
    /// Renders this scene with the provided concurrency and worker pool.
    ///
    /// This will spawn up to `concurrency` workers which are loaded from or
    /// spawned into `pool`. The `RenderingScene` state contains information to
    /// get notifications when the render has completed.
    pub fn render(self, concurrency: usize, pool: &WorkerPool) -> Result<RenderingImage, JsValue> {
        let scene = self.inner;
        let height = scene.height;
        let width = scene.width;

        // Allocate the pixel data which our threads will be writing into.
        let len = (width * height) as usize;
        let mut rgb_data = vec![0; 4 * len];
        let ptr = rgb_data.as_ptr() as usize;
        let len = rgb_data.len();
        // Configure a rayon thread pool which will pull web workers from
        // `pool`.
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(concurrency)
            .spawn_handler(|thread| {
                pool.run(|| thread.run()).unwrap();
                Ok(())
            })
            .build()
            .unwrap();

        let (tx, rx) = oneshot::channel();
        pool.run(move || {
            thread_pool.install(|| {
                rgb_data
                    .par_chunks_mut(4)
                    .enumerate()
                    .for_each(|(i, chunk)| {
                        let i = i as u32;
                        let x = i % width;
                        let y = i / width;
                        let ray = raytracer::Ray::create_prime(x, y, &scene);
                        let result = raytracer::cast_ray(&scene, &ray, 0).to_rgba();
                        chunk[0] = result.data[0];
                        chunk[1] = result.data[1];
                        chunk[2] = result.data[2];
                        chunk[3] = result.data[3];
                    });
            });
            let _ = tx.send(rgb_data);
        })?;

        let partial = UnsafeImg {
            width,
            height,
            len,
            ptr,
            _p: (),
        };
        Ok(RenderingImage {
            rx,
            partial,
            _p: (),
        })
    }
}

pub(crate) struct Img {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Img {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Img {
        Img {
            data,
            width,
            height,
        }
    }
}

impl Display for Img {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.data, f)
    }
}

impl TryInto<ImageData> for Img {
    type Error = JsValue;

    fn try_into(self) -> Result<ImageData, Self::Error> {
        let ptr = self.data.as_ptr() as usize;
        let len = self.data.len();
        // Safety: The data owner will die
        unsafe { image_data(ptr, len, self.width, self.height) }
    }
}

#[derive(Debug)]
pub(crate) struct UnsafeImg {
    pub ptr: usize,
    pub len: usize,
    pub width: u32,
    pub height: u32,
    // private constructor
    _p: (),
}

impl Display for UnsafeImg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl UnsafeImg {
    pub unsafe fn clone(&self) -> Self {
        UnsafeImg {
            ptr: self.ptr,
            len: self.len,
            width: self.width,
            height: self.height,
            _p: (),
        }
    }

    pub unsafe fn into_image_data(self) -> Result<ImageData, JsValue> {
        image_data(self.ptr, self.len, self.width, self.height)
    }
}

pub(crate) struct RenderingImage {
    pub rx: Receiver<Vec<u8>>,
    pub partial: UnsafeImg,
    // private constructor
    _p: (),
}

// TODO:
unsafe fn image_data(
    base: usize,
    len: usize,
    width: u32,
    height: u32,
) -> Result<ImageData, JsValue> {
    // FIXME: that this may or may not be UB based on Rust's rules. For example
    // threads may be doing unsynchronized writes to pixel data as we read it
    // off here. In the context of wasm this may or may not be UB, we're
    // unclear! In any case for now it seems to work and produces a nifty
    // progressive rendering. A more production-ready application may prefer to
    // instead use some form of signaling here to request an update from the
    // workers instead of synchronously acquiring an update, and that way we
    // could ensure that even on the Rust side of things it's not UB.
    let mem = wasm_bindgen::memory().unchecked_into::<WebAssembly::Memory>();
    let mem = Uint8ClampedArray::new(&mem.buffer()).slice(base as u32, (base + len) as u32);
    ImageData::new(&mem, width as f64, height as f64)
}
