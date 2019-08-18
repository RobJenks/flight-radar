use core::str::Chars;
//1,MULTILINESTRING ((-163.7128956777287 -78.59566741324154, -163.1058009511638 -78.22333871857859, ...)),1,Coastline,1.0

pub struct GeoShpIter<'a> {
    input: &'a String,
    read_point: Chars<'a>,
    read_index: isize,
    block_depth: u32,
}

impl<'a> GeoShpIter<'a> {
    fn new(input: &'a String) -> Self {
        Self { input, read_point: input.chars(), read_index: -1, block_depth: 0 }
    }
}

impl<'a> Iterator for GeoShpIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut el = self.read_point.next();
        self.read_index += 1;

        // Ignore any leading whitespace
        while el.is_some() && el.unwrap().is_whitespace() {
            el = self.read_point.next();
            self.read_index += 1;
        }
        if el.is_none() { return None }

        // Read next entry
        let start = self.read_index as usize;
        let mut non_whitespace = start;
        while let Some(c) = el {
            if c == ',' && self.block_depth  == 0 { break } // End of an entry

            if !c.is_whitespace() { non_whitespace = self.read_index as usize; }

            if c == '(' { self.block_depth += 1 }
            else if c == ')' { self.block_depth -= 1 }

            el = self.read_point.next();
            self.read_index += 1;
        }

        // Ignore any trailing whitespace
        let end = std::cmp::min(non_whitespace + 1,
                                el.map(|_| self.read_index as usize)
                                      .unwrap_or_else(|| self.input.len()));

        Some(&self.input[start..end])
    }
}


pub fn parse_geo_shp(data: &String) -> GeoShpIter {
    GeoShpIter::new(data)
}

#[cfg(test)]
mod tests {
    use super::{GeoShpIter};

    fn run_iter<'a>(input: &'a String) -> Vec<&'a str> {
        GeoShpIter::new(input).collect::<Vec<&'a str>>()
    }

    #[test]
    fn test_standard_iter() {
        assert_eq!(run_iter(&"abc,def,ghi,jkl".to_string()), vec!["abc", "def", "ghi", "jkl"]);
        assert_eq!(run_iter(&"abc".to_string()), vec!["abc"]);
        assert_eq!(run_iter(&"".to_string()), Vec::<&str>::new());
    }

    #[test]
    fn test_iter_whitespace_handling() {
        assert_eq!(run_iter(&"   abc   , def ,ghi    , jkl   ".to_string()), vec!["abc", "def", "ghi", "jkl"]);
        assert_eq!(run_iter(&"     ".to_string()), Vec::<&str>::new());
    }


    #[test]
    fn test_iter_multi_block_handling() {
        assert_eq!(run_iter(&"abc,MULTI ((( def,ghi ))),jkl".to_string()), vec!["abc", "MULTI ((( def,ghi )))", "jkl"]);
        assert_eq!(run_iter(&"abc, (( def, ( ghi, jkl ) )) ,jkl".to_string()), vec!["abc", "(( def, ( ghi, jkl ) ))", "jkl"]);
    }

    #[test]
    fn test_invalid_unclosed_multi_block_handling() {
        assert_eq!(run_iter(&"abc,MULTI ((( def,ghi )),jkl".to_string()), vec!["abc", "MULTI ((( def,ghi )),jkl"]);
    }
}






















