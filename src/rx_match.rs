use crate::parse::{parse_re, Matcher, MatcherKind, Quantifier};

struct BacktrackState {
    is_backtrackable: bool,
    matcher: Matcher,
    consumptions: Vec<usize>,
}

struct Re {
    i: usize,
    matcher_stack: Vec<Matcher>,
    current_state: Option<Matcher>,
    backtrack_stack: Vec<BacktrackState>,
}

fn matches_string_at_index(
    matcher: &Matcher,
    s: &[char],
    i: usize,
) -> Result<(bool, usize), String> {
    if i >= s.len() {
        return Ok((false, 0));
    }

    match &matcher.matcher_kind {
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
        MatcherKind::Group(items) => return Re::new(items).test_internal(&s[i..]),
    }
}

impl Re {
    fn new(states: &Vec<Matcher>) -> Self {
        Self {
            i: 0,
            matcher_stack: (states).into_iter().rev().map(|it| it.clone()).collect(),
            backtrack_stack: Vec::new(),
            current_state: None,
        }
    }

    fn backtrack(&mut self) -> bool {
        self.matcher_stack.push(self.current_state.clone().unwrap());
        let mut could_backtrack = false;

        while self.backtrack_stack.len() > 0 {
            let BacktrackState {
                is_backtrackable,
                matcher,
                mut consumptions,
            } = self.backtrack_stack.pop().unwrap();

            if is_backtrackable {
                if consumptions.len() == 0 {
                    self.matcher_stack.push(matcher);
                    continue;
                } else {
                    let n = consumptions.pop().unwrap();
                    self.i -= n;
                    self.backtrack_stack.push(BacktrackState {
                        is_backtrackable,
                        matcher,
                        consumptions,
                    });
                    could_backtrack = true;
                    break;
                }
            }
            self.matcher_stack.push(matcher);
            for n in consumptions {
                self.i -= n;
            }
        }

        if could_backtrack {
            self.current_state = self.matcher_stack.pop();
        }
        could_backtrack
    }

    fn test_internal(&mut self, s: &[char]) -> Result<(bool, usize), String> {
        self.current_state = self.matcher_stack.pop();

        while self.current_state.is_some() {
            let st = self.current_state.as_ref().unwrap();
            match st.quantifier {
                Quantifier::ExactlyOne => {
                    let (is_match, consumed) = matches_string_at_index(&st, s, self.i)?;
                    if !is_match {
                        let index_before_backtracking = self.i;
                        let could_backtrack = self.backtrack();
                        if !could_backtrack {
                            return Ok((false, index_before_backtracking));
                        }
                        continue;
                    }
                    self.backtrack_stack.push(BacktrackState {
                        is_backtrackable: false,
                        matcher: self.current_state.clone().unwrap(), // another bad `clone`
                        consumptions: vec![consumed],
                    });
                    self.i += consumed;
                    self.current_state = self.matcher_stack.pop();
                }
                Quantifier::ZeroOrOne => {
                    if self.i >= s.len() {
                        self.backtrack_stack.push(BacktrackState {
                            is_backtrackable: false,
                            matcher: self.current_state.clone().unwrap(), // another bad `clone`
                            consumptions: vec![0],
                        });
                        self.current_state = self.matcher_stack.pop();
                        continue;
                    }
                    let (is_match, consumed) = matches_string_at_index(&st, s, self.i)?;
                    self.i += consumed;
                    self.backtrack_stack.push(BacktrackState {
                        is_backtrackable: is_match && consumed > 0,
                        matcher: self.current_state.clone().unwrap(), // another bad `clone`
                        consumptions: vec![consumed],
                    });
                    self.current_state = self.matcher_stack.pop();
                    continue;
                }
                Quantifier::ZeroOrMore => {
                    let mut backtrack_state = BacktrackState {
                        is_backtrackable: true,
                        matcher: self.current_state.clone().unwrap(),
                        consumptions: Vec::new(),
                    };
                    loop {
                        if self.i >= s.len() {
                            if backtrack_state.consumptions.len() == 0 {
                                backtrack_state.is_backtrackable = false;
                                backtrack_state.consumptions.push(0);
                            }
                            self.backtrack_stack.push(backtrack_state);
                            self.current_state = self.matcher_stack.pop();
                            break;
                        }
                        let (is_match, consumed) = matches_string_at_index(&st, s, self.i)?;
                        if !is_match || consumed == 0 {
                            if backtrack_state.consumptions.len() == 0 {
                                backtrack_state.is_backtrackable = false;
                                backtrack_state.consumptions.push(0);
                            }
                            self.backtrack_stack.push(backtrack_state);
                            self.current_state = self.matcher_stack.pop();
                            break;
                        }
                        backtrack_state.consumptions.push(consumed);
                        self.i += consumed;
                    }
                }
            }
        }
        Ok((true, self.i))
    }
}

pub fn test_re(re: &str, s: &str) -> Result<Option<usize>, String> {
    let chars: Vec<char> = s.chars().collect();
    let match_result = Re::new(&parse_re(re)?).test_internal(&chars)?;
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
    fn test_wildcard() {
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
        assert_eq!(Ok(Some(5)), test_re("ab(cd)c", "abcdc"));
        assert_eq!(Ok(Some(2)), test_re("a(bcd)?c", "ac"));
    }

    #[test]
    fn test_backtracking() {
        assert_eq!(Ok(Some(3)), test_re("a.*c", "abc"));
        assert_eq!(Ok(Some(3)), test_re("abc*c", "abc"));
    }
}
