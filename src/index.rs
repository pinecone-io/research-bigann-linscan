use std::cmp::{Ordering, Reverse};
use std::cmp::Ordering::Equal;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

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
pub struct Posting {
    pub docid: u32,
    pub value: f32,
}

/// Vanilla LinScan operates on an uncompressed inverted index.
pub struct Index {
    inverted_index: HashMap<u32, Vec<Posting>>,
    num_docs: u32,
}

impl Index {
    pub fn new() -> Index {
        Index {
            inverted_index: HashMap::new(),
            num_docs: 0,
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

    fn compute_dot_product(&mut self, coordinate: u32, query_value: f32, scores: &mut [f32]) {
        match self.inverted_index.get(&coordinate) {
            None => {}
            Some(postings) => {
                for posting in postings {
                    scores[posting.docid as usize] += query_value * posting.value;
                }
            }
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

    pub fn print_stats(&self) {
        println!("Linscan statistics:");
        println!("# documents: {}", self.num_docs);
        println!("# elements in inverted index: {}", self.inverted_index.keys().len());
        let mut total_elements = 0;
        for t in self.inverted_index.iter() {
            total_elements += t.1.len();
        }
        println!("Avg. nnz per vector: {}", total_elements as f32 / self.num_docs as f32);
    }
}
