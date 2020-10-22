use crate::parse::{parse_re, Matcher, MatcherKind, Quantifier};
use std::collections::VecDeque;
use std::str::Chars;

fn matches_string_at_index(
    matcher: &Matcher,
    s: &Vec<char>,
    i: usize,
) -> Result<(bool, usize), String> {
    if i > s.len() {
        return Ok((false, 0));
    }

    match &matcher.matcherKind {
        MatcherKind::Wildcard => {
            return Ok((true, 1));
        }
        MatcherKind::Element(c) => {
            if *c == s[i] {
                return Ok((true, 1));
            } else {
                return Ok((false, 0));
            }
        }
        MatcherKind::Group(items) => return test_internal(items, s[1..]),,
    }
}

fn test_internal(states: &Vec<Matcher>, s: &Vec<char>) -> Result<(bool, usize), String> {
    let mut queue: VecDeque<&Matcher> = (states).into_iter().collect();

    let mut i = 0;
    let mut current_state = queue.pop_front();

    while current_state.is_some() {
        let st = current_state.unwrap();
        match st.quantifier {
            Quantifier::ExactlyOne => {
                let (is_match, consumed) = matches_string_at_index(&st, s, i)?;
                if !is_match {
                    return Ok((false, i));
                }
                i += consumed;
                current_state = queue.pop_front();
            }
            Quantifier::ZeroOrOne => {
                if i >= s.len() {
                    current_state = queue.pop_front();
                    continue;
                }
                let (_is_match, consumed) = matches_string_at_index(&st, s, i)?;
                i += consumed;
                current_state = queue.pop_front();
                continue;
            }
            Quantifier::ZeroOrMore => loop {
                if i >= s.len() {
                    current_state = queue.pop_front();
                    break;
                }
                let (is_match, consumed) = matches_string_at_index(&st, s, i)?;
                if !is_match || consumed == 0 {
                    current_state = queue.pop_front();
                    break;
                }

                i += consumed;
            },
        }
    }
    Ok((true, i))
}

pub(crate) fn test_re(re: &str, s: &str) -> Result<Option<usize>, String> {
    let match_result = test_internal(&parse_re(re)?, &s.chars().collect())?;
    if let (true, i) = match_result {
        Ok(Some(i))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard(){
        assert_eq!(Ok(Some(1)), test_re(".", "a"));
    }

    #[test]
    fn test_single_char() {
        assert_eq!(Ok(Some(1)), test_re("a", "a"));
    }

    #[test]
    fn test_sequence() {
        assert_eq!(Ok(Some(3)), test_re("abc", "abc"));
    }

    #[test]
    fn test_zero_or_one() {
        assert_eq!(Ok(Some(3)), test_re("ab?c", "abc"));
        assert_eq!(Ok(Some(2)), test_re("ab?c", "ac"));
    }

    #[test]
    fn test_zero_or_more() {
        assert_eq!(Ok(Some(10)), test_re("ab*c*", "abbbbbcccc"));
    }

    #[test]
    fn test_groups() {
        assert_eq!(Ok(Some(5)), test_re("a(bcd)c", "abcdc"));
        assert_eq!(Ok(Some(2)), test_re("a(bcd)?c", "ac"));
    }
}
