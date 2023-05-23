use std::collections::HashMap;
use std::time::Duration;
use pyo3::prelude::*;
use rayon::prelude::*;

mod index;

#[pyclass]
struct LinscanIndex {
    index: index::Index
}

#[pymethods]
impl LinscanIndex {
    // creates a new empty index.
    // optional parameter: number of threads to initialize the global pool with.
    // If not supplied, then the number of threads is chosen automatically (recommended).
    #[new]
    pub fn new(num_threads: Option<usize>) -> LinscanIndex {
        println!("Initializing a new LinscanIndex.");
        num_threads.map(|nt| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(nt)
                .build_global()
                .unwrap();
        });
        LinscanIndex {
            index: index::Index::new(),
        }
    }

    pub fn insert(&mut self, newdoc: HashMap<u32, f32>) {
        self.index.insert(&newdoc);
    }

    pub fn retrieve(&mut self, query: HashMap<u32, f32>, top_k: usize, inner_product_budget_ms: Option<f32>) -> Vec<u32> {
        let duration = inner_product_budget_ms.map(|budget_ms| Duration::from_secs_f32(budget_ms / 1000_f32));

        let r = self.index.retrieve(&query, top_k, duration);
        r.into_iter().map(|f| f.docid).collect()
    }

    // search for the top_k, given a collection of queries. Queries are issued in parallel using rayon's par_iter.
    pub fn retrieve_parallel(&mut self, queries: Vec<HashMap<u32, f32>>, top_k: usize, inner_product_budget_ms: Option<f32>) -> Vec<Vec<u32>> {
        let duration = inner_product_budget_ms.map(|budget_ms| Duration::from_secs_f32(budget_ms / 1000_f32));

        queries.par_iter().map(|q| {
            let r = self.index.retrieve(&q, top_k, duration);
            r.into_iter().map(|f| f.docid).collect()
        }).collect()

    }

    // this defines the out of the >str(index) in python
    fn __str__(&self) -> PyResult<String> {
        Ok(self.index.to_string())
    }

    // this defines the out of the >repr(index) in python, as well as simply >index
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.index.to_string())
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