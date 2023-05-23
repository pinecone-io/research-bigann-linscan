use std::cmp::{Ordering, Reverse};
use std::cmp::Ordering::Equal;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};
use std::fmt;
use std::cmp::min;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

/// A structure that reports the outcome of the inner product computation for a single document.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SearchResult {
    pub docid: u32,
    pub score: f32,
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &SearchResult) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// A structure that represents a single `posting` in the inverted list.
#[derive(Clone)]
pub struct Posting {
    pub docid: u32,
    pub value: f32,
}

/// Vanilla LinScan operates on an uncompressed inverted index.
pub struct Index {
    inverted_index: HashMap<u32, Vec<Posting>>,
    num_docs: u32,
    parallel: bool,
}

impl Index {
    pub fn new(parallel: bool) -> Index {
        Index {
            inverted_index: HashMap::new(),
            num_docs: 0,
            parallel,
        }
    }

    /// Inserts a new document into the index.
    ///
    /// This function automatically assigns the document id in the order documents are inserted,
    /// beginning from 1.
    pub fn insert(&mut self, document: &HashMap<u32, f32>) {
        for (&coordinate, &value) in document {
            self.inverted_index.entry(coordinate).or_default().push(Posting{
                docid: self.num_docs,
                value,
            });
        }
        self.num_docs += 1;
    }

    pub fn finalize(&mut self) {
        // sort the posting lists to allow accelerated parallel search
        self.inverted_index.iter_mut().for_each(|(_, postings)| {
            postings.sort_by_key(|posting| posting.docid);
        });

    }

    fn compute_dot_product(&mut self, coordinate: u32, query_value: f32, scores: &mut [f32]) {
        if !self.parallel {
            match self.inverted_index.get(&coordinate){
                None => {}
                Some(postings) => {
                    for posting in postings {
                        scores[posting.docid as usize] += query_value * posting.value;
                    }
                }
            }
        } else { // parallel = true
            let partitions = rayon::current_num_threads();
            let partition_size = (scores.len() as f32 / partitions as f32).ceil() as usize;

            let mut chunks = vec![];
            let mut begin = 0_usize;
            let length = scores.len();

            // partition the output space between threads
            for chunk in scores.chunks_mut(partition_size) {
                let end = min(begin + partition_size, length);
                chunks.push((chunk, begin as u32, end as u32));
                begin = end;
            }

            chunks.par_iter_mut().for_each(|(out, a, b)| {
                match self.inverted_index.get(&coordinate) {
                    None => {}
                    Some(postings) => {

                        let start_index = postings
                            .binary_search_by_key(a, |posting| posting.docid)
                            .unwrap_or_else(|x| x);

                        let end_index = postings
                            .binary_search_by_key(b, |posting| posting.docid)
                            .unwrap_or_else(|x| x);

                        for posting in &postings[start_index..end_index] {
                        // for posting in postings.iter().filter(|posting| posting.docid >= *a && posting.docid < *b) {
                                out[(posting.docid - *a) as usize] += query_value * posting.value;
                        }
                    }
                }
            });


        }
    }

    /// Returns the `top_k` documents according to the inner product score with the given query.
    ///
    /// This function implements a basic coordinate-at-a-time algorithm to compute the inner product
    /// scores, followed by a heap-based algorithm to identify the top-k entries.
    ///
    /// When `inner_product_budget` is provided, this function stops computing document scores when
    /// the budget is exhausted. It then moves on to the sort operation. Note that, the time spent
    /// on the sort operation is separate from the given time budget.
    pub fn retrieve(&mut self, query: &HashMap<u32, f32>,
                top_k: usize,
                inner_product_budget: Option<Duration>) -> Vec<SearchResult> {
        // Create an array with the same size as the number of documents in the index.
        let mut scores = Vec::with_capacity(self.num_docs as usize);
        scores.resize(self.num_docs as usize, 0_f32);

        match inner_product_budget {
            None => {
                // Simply traverse the index one coordinate at a time and accumulate partial scores.
                for (&coordinate, &query_value) in query {
                    self.compute_dot_product(coordinate, query_value, &mut scores);
                }
            }
            Some(budget) => {
                let mut time_left = Duration::from(budget);

                // Sort query coordinates by absolute value in descending order.
                let mut query = query.iter()
                    .map(|(k, v)| (*k, *v)).collect::<Vec<(u32, f32)>>();
                query.sort_by(|(_, v1), (_, v2)| v2.abs().partial_cmp(&v1.abs()).unwrap_or(Equal));

                // Traverse the inverted index one coordinate at a time and accumulate partial scores.
                // Quit as soon as the time budget is exhausted.
                for (coordinate, query_value) in query {
                    let scoring_time = Instant::now();
                    self.compute_dot_product(coordinate, query_value, &mut scores);
                    let scoring_time = scoring_time.elapsed();
                    time_left = if time_left > scoring_time { time_left - scoring_time } else { Duration::ZERO };
                    if time_left.is_zero() {
                        break
                    }
                }
            }
        }

        // Find and return the top-k documents using a heap.
        let mut heap: BinaryHeap<Reverse<SearchResult>> = BinaryHeap::new();

        let mut threshold = f32::MIN;
        for (docid, &score) in scores.iter().enumerate() {
            if score > threshold {
                heap.push(Reverse(SearchResult { docid: docid as u32, score }));
                if heap.len() > top_k {
                    threshold = heap.pop().unwrap().0.score;
                }
            }
        }

        heap.into_sorted_vec().iter().map(|e| e.0).collect()
    }

}

// To use the `{}` marker, the trait `fmt::Display` must be implemented
// manually for the type.
impl fmt::Display for Index {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let total_elements: usize = self.inverted_index.iter().map(|(_, v)| v.len()).sum();
        write!(f, "Linscan Index [{} documents, {} unique tokens, avg. nnz: {}, parallel: {}]", self.num_docs, self.inverted_index.keys().len(), total_elements as f32 / self.num_docs as f32, self.parallel)
    }
}
