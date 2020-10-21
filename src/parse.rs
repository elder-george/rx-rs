#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMore,
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MatcherKind {
    Wildcard,
    Element(char),
    Group(Vec<Matcher>),
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Matcher {
    pub(crate) quantifier: Quantifier,
    pub(crate) matcherKind: MatcherKind,
}

impl Matcher {
    pub(crate) fn Wildcard(q: Quantifier) -> Self {
        Self {
            quantifier: q,
            matcherKind: MatcherKind::Wildcard,
        }
    }
    pub(crate) fn Element(c: char, q: Quantifier) -> Self {
        Self {
            quantifier: q,
            matcherKind: MatcherKind::Element(c),
        }
    }
    pub(crate) fn Group(items: Vec<Matcher>, q: Quantifier) -> Self {
        Self {
            quantifier: q,
            matcherKind: MatcherKind::Group(items),
        }
    }
}

pub(crate) fn parse_re(re: &str) -> Result<Vec<Matcher>, String> {
    let mut stack = vec![Vec::new()];

    let mut i = 0;
    for next in re.chars() {
        match next {
            '.' => {
                stack
                    .last_mut()
                    .unwrap()
                    .push(Matcher::Wildcard(Quantifier::ExactlyOne));
                i += 1;
            }
            '\\' => {
                if i + 1 >= re.len() {
                    return Err(format!("Bad escape character at index {}", i));
                }
                stack.last_mut().unwrap().push(Matcher::Element(
                    re.chars().nth(i + 1).unwrap(),
                    Quantifier::ExactlyOne,
                ));
                i += 2;
            }
            '(' => {
                stack.push(Vec::new());
                i += 1;
            }
            ')' => {
                if stack.len() <= 1 {
                    return Err(format!("No group to close at index {}", i));
                }
                let states = stack.pop().unwrap();
                stack
                    .last_mut()
                    .unwrap()
                    .push(Matcher::Group(states, Quantifier::ExactlyOne));
                i += 1;
            }
            '?' => {
                let mut last_elem = stack.last_mut().unwrap().last_mut().unwrap();
                if last_elem.quantifier != Quantifier::ExactlyOne {
                    return Err(
                        "Quantifier must follow an unqualified element or group".to_string()
                    );
                }
                last_elem.quantifier = Quantifier::ZeroOrOne;
                i += 1;
            }
            '*' => {
                let mut last_elem = stack.last_mut().unwrap().last_mut().unwrap();
                if last_elem.quantifier != Quantifier::ExactlyOne {
                    return Err(
                        "Quantifier must follow an unqualified element or group".to_string()
                    );
                }
                last_elem.quantifier = Quantifier::ZeroOrMore;
                i += 1;
            }
            '+' => {
                let last_elem = stack.last_mut().unwrap().last_mut().unwrap();
                if last_elem.quantifier != Quantifier::ExactlyOne {
                    return Err(
                        "Quantifier must follow an unqualified element or group".to_string()
                    );
                }
                // split into two operations:
                let mut zero_or_more_copy = last_elem.clone();
                zero_or_more_copy.quantifier = Quantifier::ZeroOrMore;
                stack.last_mut().unwrap().push(zero_or_more_copy);
                i += 1;
            }

            _ => {
                stack
                    .last_mut()
                    .unwrap()
                    .push(Matcher::Element(next, Quantifier::ExactlyOne));
                i += 1;
            }
        }
    }

    if stack.len() != 1 {
        return Err("Unmatched groups in regular expression".to_string());
    }
    Ok(stack.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty() {
        assert_eq!(Ok(vec![]), parse_re(""));
    }
    #[test]
    fn single_char() {
        assert_eq!(
            Ok(vec![Matcher::Element('a', Quantifier::ExactlyOne)]),
            parse_re("a")
        );
    }
    #[test]
    fn sequence() {
        assert_eq!(
            Ok(vec![
                Matcher::Element('a', Quantifier::ExactlyOne),
                Matcher::Element('b', Quantifier::ExactlyOne),
                Matcher::Element('c', Quantifier::ExactlyOne)
            ]),
            parse_re("abc")
        );
    }
    #[test]
    fn zero_or_one() {
        assert_eq!(
            Ok(vec![
                Matcher::Element('a', Quantifier::ExactlyOne),
                Matcher::Element('b', Quantifier::ZeroOrOne),
                Matcher::Element('c', Quantifier::ExactlyOne)
            ]),
            parse_re("ab?c")
        );
    }
    #[test]
    fn zero_or_more() {
        assert_eq!(
            Ok(vec![
                Matcher::Element('a', Quantifier::ExactlyOne),
                Matcher::Element('b', Quantifier::ZeroOrMore),
                Matcher::Element('c', Quantifier::ExactlyOne)
            ]),
            parse_re("ab*c")
        );
    }
    #[test]
    fn one_or_more() {
        assert_eq!(
            Ok(vec![
                Matcher::Element('a', Quantifier::ExactlyOne),
                Matcher::Element('b', Quantifier::ExactlyOne),
                Matcher::Element('b', Quantifier::ZeroOrMore),
                Matcher::Element('c', Quantifier::ExactlyOne)
            ]),
            parse_re("ab+c")
        );
    }
}
