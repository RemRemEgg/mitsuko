#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn find_block_1() {
        let s = "{ this is one {{}} fn} opp!sebf kjshb fkuehbs fkshebf lshebf les".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s);
        let o2 = b.find_size(&s);
        println!("\'{}\'", &s[..o2.unwrap()]);
        assert_eq!(o, Ok(22));
    }

    #[test]
    fn find_block_2() {
        let s = "{ code block array[indicie(function())].method(); let lambda = {}}".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s);
        assert_eq!(o, Ok(s.len()));
    }

    #[test]
    #[should_panic]
    fn find_block_3() {
        let s = "{array[function(])}".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s);
        assert_eq!(o, Ok(2));
    }
}