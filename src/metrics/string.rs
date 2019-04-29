/*!
String metrics

*/
use hashbrown::HashSet;
use std::iter::FromIterator;
use itertools::Itertools;

///  Sørensen–Dice similarity coefficient
///
///  This similarity tokenizes the input string x, y as 2-char n-grams,
///  into two sets of tokens X, Y then computes,
///
///  similarity(x, y) = 2 * |X ∩ Y| / (|X| + |Y|)
///
///  where |X| is the cardinality of set X.
///
///  # Example
///  ```rust
///  use vtext::metrics::string::dice_similarity;
///
///  let res = dice_similarity("yesterday", "today");
///  // returns 0.333
///  ```
pub fn dice_similarity(x: &str, y: &str) -> f64 {
    if (x.len() == 0) | (y.len() == 0) {
        0.0
    } else if (x == y) {
        1.0
    } else {
        let mut x_set: HashSet<(char, char)> = HashSet::new();

        for (char_1, char_2) in x.chars().tuple_windows() {
            x_set.insert((char_1, char_2));
        }

        let mut y_set: HashSet<(char, char)> = HashSet::new();

        for (char_1, char_2) in y.chars().tuple_windows() {
            y_set.insert((char_1, char_2));
        }

        let intersection_len = x_set.intersection(&y_set).count();

        (2 * intersection_len) as f64 / (x_set.len() + y_set.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::string::dice_similarity;

    #[test]
    fn test_dice_similarity() {
        let res = dice_similarity("yesterday", "today");
        assert_eq!((res * 100.).round() / 100., 0.33);

        assert_eq!(dice_similarity("healed", "sealed"), 0.8);

        assert_eq!(dice_similarity("", ""), 0.0);
        // 1 char, doesn't allow to make a single 2-char ngram
        assert_eq!(dice_similarity("1", "test"), 0.0);

        assert_eq!(dice_similarity("test", "test"), 1.0);
    }

}
