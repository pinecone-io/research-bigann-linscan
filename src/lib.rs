use std::collections::HashMap;
use std::time::Duration;
use pyo3::prelude::*;
mod index;

#[pyclass]
struct LinscanIndex {
    index: index::Index
}

#[pymethods]
impl LinscanIndex {
    #[new]
    pub fn new() -> LinscanIndex {
        println!("Initializing a new LinscanIndex.");
        LinscanIndex {
            index: index::Index::new(),
        }
    }

    pub fn insert(&mut self, newdoc: HashMap<u32, f32>) {
        self.index.insert(&newdoc);
    }

    pub fn retrieve(&mut self, query: HashMap<u32, f32>, top_k: usize, inner_product_budget_ms: Option<f32>) -> Vec<u32> {
        let duration = inner_product_budget_ms.map(|budget_ms| Duration::from_secs_f32(budget_ms * 1000_f32));

        let r = self.index.retrieve(&query, top_k, duration);
        // dbg!(r);
        r.into_iter().map(|f| f.docid).collect()
    }

    pub fn print_stats(&self) {
        self.index.print_stats();
    }
}


/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn linscan(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<LinscanIndex>()?;
    Ok(())
}