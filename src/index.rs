use std::cmp::{Ordering, Reverse};
use std::cmp::Ordering::Equal;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};
use std::fmt;
use std::io::{BufReader, BufWriter};
use serde::{Serialize, Deserialize};

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
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Posting {
    pub docid: u32,
    pub value: f32,
}

/// Vanilla LinScan operates on an uncompressed inverted index.
#[derive(Serialize, Deserialize, Debug)]
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

    fn compute_dot_product(&self, coordinate: u32, query_value: f32, scores: &mut [f32]) {
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
    pub fn retrieve(&self, query: &HashMap<u32, f32>,
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

    /// save the index to a file
    pub fn save(&self, file: &mut std::fs::File) {

        // Wrap the file writer in a BufWriter for buffering
        let writer = BufWriter::new(file);

        // Serialize the index into the buffered writer using bincode
        bincode::serialize_into(writer, &self).expect("Failed to serialize");
        // bincode::serialize_into(file, &self).unwrap();
    }

    /// load the index from a file
    pub fn load(file: &std::fs::File) -> Index {
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).unwrap()
    }
}

// To use the `{}` marker, the trait `fmt::Display` must be implemented
// manually for the type.
impl fmt::Display for Index {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let total_elements: usize = self.inverted_index.iter().map(|(_, v)| v.len()).sum();
        write!(f, "Linscan Index [{} documents, {} unique tokens, avg. nnz: {}]", self.num_docs, self.inverted_index.keys().len(), total_elements as f32 / self.num_docs as f32 )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::index::Index;

    #[test]
    fn test_serde() {
        let mut ind = Index::new();

        let v1 = HashMap::from([(1_u32, 0.4_f32), (5, 0.6)]);
        let v2 = HashMap::from([(2_u32, 0.4_f32), (5, 0.9)]);

        ind.insert(&v1);
        ind.insert(&v2);



        // serialize to byte array
        let bytes = bincode::serialize(&ind).unwrap();
        // reconstruct and compare
        let ind_rec: Index = bincode::deserialize(&bytes).unwrap();

        assert_eq!(ind.num_docs, ind_rec.num_docs);
        assert_eq!(ind.inverted_index, ind_rec.inverted_index);
    }
}
