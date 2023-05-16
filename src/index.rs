use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};

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

pub struct Posting {
    pub docid: u32,
    pub value: f32,
}

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

    fn insert(&mut self, document: &HashMap<u32, f32>) {
        self.num_docs += 1;

        for (&coordinate, &value) in document {
            self.inverted_index.entry(coordinate).or_default().push(Posting{
                docid: self.num_docs,
                value,
            });
        }
    }

    fn retrieve(&mut self, query: &HashMap<u32, f32>, top_k: usize) -> Vec<SearchResult> {
        let mut scores = Vec::with_capacity((self.num_docs + 1) as usize);
        scores.resize((self.num_docs + 1) as usize, 0_f32);

        for (coordinate, query_value) in query {
            match self.inverted_index.get(&coordinate) {
                None => {}
                Some(postings) => {
                    for posting in postings {
                        scores[posting.docid as usize] += query_value * posting.value;
                    }
                }
            }
        }

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
