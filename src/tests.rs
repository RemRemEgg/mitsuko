use crate::*;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_block_1() {
        let s = "{ this is \\)\\)\\)\\)\\)one {{}} fn} opp!sebf kjshb fkuehbs fkshebf lshebf les".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s, 0);
        let o2 = b.find_size(&s, 0);
        println!("\'{}\'", &s[..o2.unwrap()]);
        assert_eq!(o, Ok(32));
    }

    #[test]
    fn find_block_2() {
        let s = "{ code block array[indicie(function())].method(); let lambda = {}}".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s, 0);
        assert_eq!(o, Ok(s.len()));
    }

    #[test]
    fn find_block_3() {
        let s = "{array[function(])}".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s, 0);
        assert!(o.is_err());
    }

    #[test]
    fn find_block_4() {
        let s = "{ let cp \\|= \")\"}".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s, 0);
        assert_eq!(o, Ok(17));
    }

    #[test]
    fn find_block_5() {
        let s = "(nothing to see here!".to_string();

        let mut b = Blocker::new();
        let o = b.find_size(&s, 0);
        assert!(o.unwrap() >= Blocker::NOT_FOUND);
    }

    #[test]
    fn find_block_vec_1() {
        let s = vec!["[ {this is line one", "and (two)", "and ')\"\\''  {three", "number} four, ", " and } five", "not 6!"].iter().map(|s| String::from(*s)).collect::<Vec<String>>();

        let mut b = Blocker::new();
        let o = b.find_size_vec(&s, 2);
        println!("{:?}", o);
        assert_eq!(o.unwrap(), (4, 6));
    }
}