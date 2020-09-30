use wasm_bindgen::JsCast;
use web_sys::{
    window, CanvasRenderingContext2d, Element, HtmlButtonElement, HtmlCanvasElement,
    HtmlInputElement, HtmlTextAreaElement,
};

use worker_pool::WorkerPool;

pub(crate) struct RayTracing {
    pub button: HtmlButtonElement,
    pub canvas: HtmlCanvasElement,
    pub concurrency: HtmlInputElement,
    pub concurrency_amt: Element,
    pub ctx: CanvasRenderingContext2d,
    pub n_concurrency: usize,
    pub rendering: bool,
    pub scene: HtmlTextAreaElement,
    pub time: Element,
    // Is unsafe expose worker pool
    pool: WorkerPool,
}

impl Default for RayTracing {
    fn default() -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let num_th = window.navigator().hardware_concurrency() as usize;

        let button = document
            .get_element_by_id("render")
            .unwrap()
            .unchecked_into::<HtmlButtonElement>();
        let scene = document
            .get_element_by_id("scene")
            .unwrap()
            .unchecked_into::<HtmlTextAreaElement>();
        let canvas = document
            .get_element_by_id("canvas")
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>();
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .unchecked_into::<CanvasRenderingContext2d>();
        let concurrency = document
            .get_element_by_id("concurrency")
            .unwrap()
            .unchecked_into::<HtmlInputElement>();
        concurrency.set_min("1");
        let max = num_th.to_string();
        concurrency.set_max(&max);
        concurrency.set_value(&max);
        let concurrency_amt = document.get_element_by_id("concurrency-amt").unwrap();
        concurrency_amt.set_text_content(Some(&max));
        let time = document.get_element_by_id("timing-val").unwrap();

        Self {
            button,
            canvas,
            concurrency,
            concurrency_amt,
            ctx,
            n_concurrency: num_th,
            rendering: true,
            scene,
            time,
            pool: WorkerPool::new(num_th).expect("create worker pool"),
        }
    }
}

impl RayTracing {
    /// Expose worker pool
    pub fn pool(&self) -> &WorkerPool {
        &self.pool
    }
}
