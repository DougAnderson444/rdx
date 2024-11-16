use std::collections::HashMap;

use super::{parse, Component};

type ParsedValue = Component;

pub struct Cache {
    map: HashMap<String, ParsedValue>, // Directly store parsed values
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            map: HashMap::new(),
        }
    }

    pub fn get_or_insert<F>(&mut self, block: String) -> &ParsedValue {
        // Check if the block is already parsed
        self.map.entry(block.clone()).or_insert_with(|| {
            // If not, parse it
            parse(block)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pest::parse;

    #[test]
    fn test_cache() {
        let mut cache = Cache::new();

        let block = "<Label>Hello</Label>".to_string();

        let manual_parsed = parse(&block.clone()).unwrap();

        // First access will parse the block
        let result = cache.get_or_insert(block.clone());

        assert_eq!(result, &manual_parsed);

        // Second access retrieves the cached value
        let result2 = cache.get_or_insert(block.clone());

        assert_eq!(result2, &manual_parsed);
    }
}
