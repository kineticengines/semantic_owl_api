use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::char,
  error::{ErrorKind, ParseError},
  AsChar, Err as NomErr, IResult, InputIter, InputTake,
};

use crate::declarations::*;

// custom super trait with turtle specific helper methods
trait TurtleInput: InputTake {
  /// Split the stream at the `count` byte offset returning
  /// the full input on the right part of the tuple
  fn take_right(&self, count: usize) -> (Self, Self);
}

impl<'a> TurtleInput for &'a str {
  // return byte index
  fn take_right(&self, count: usize) -> (Self, Self) {
    (&self[..count], &self[count..])
  }
}

///  parse_turtle is the main entry point for parsing turtle documents
pub fn parse_turtle(input: &str) -> IResult<(), StatementKind> {
  // check if the statement is a comment or a valid statement that either
  // has a valid ending
  let input = input.trim_end(); // remove any tail whitespace
  match find_and_trim_tail_comment(input) {
    // parse tail comments
    Ok(elements) => {
      let (_, right_elm) = elements;
      parse_turtle(right_elm)
    }
    Err(_) => match alt((
      is_a_comment,
      statement_ending,
      is_empty_statement,
      is_valid_statement_terminator,
    ))(input)
    {
      Ok(elements) => {
        let (_, right_elm) = elements;
        match Some(right_elm) {
          Some(x) if x.starts_with('#') && x.len() == 0x1 => Ok(((), StatementKind::Comment)), // parse comments
          Some(x) if x.is_empty() => Ok(((), StatementKind::Whitespace)), // parse whitespaces
          Some(x) if (x.starts_with('.') || x.ends_with('.')) && x.len() == 0x1 => {
            Ok(((), StatementKind::Terminator))
          } // parse final end of a statement
          _ => match alt((is_a_norm_prefix, is_a_base_prefix, statement_ending))(input) {
            Ok(elements) => {
              let (_, right_elm) = elements;
              match Some(right_elm) {
                Some(x) if x.starts_with("@prefix") => Ok(((), StatementKind::NormPrefix)), // parse norm prefix
                Some(x) if x.starts_with("@base") => Ok(((), StatementKind::BasePrefix)), // parse base prefix
                Some(x)
                  if x.ends_with('.')
                    && x.starts_with("@prefix") == false
                    && x.starts_with("@base") == false =>
                {
                  Ok(((), StatementKind::StatementWithTerminator))
                } // parse end of a statement
                _ => Ok(((), StatementKind::NotATurtle)),
              }
            }
            Err(_) => Ok(((), StatementKind::NotATurtle)),
          },
        }
      }
      Err(_) => match alt((statement_part_ending, statement_ending))(input) {
        Ok(elements) => {
          let (_, right_elm) = elements;
          match Some(right_elm) {
            Some(x) if x.ends_with(';') => Ok(((), StatementKind::PartOf)), // parse part of statement
            Some(x) if x.ends_with('.') => Ok(((), StatementKind::StatementWithTerminator)), // parse end of a statement
            _ => Ok(((), StatementKind::NotATurtle)),
          }
        }
        Err(_) => Ok(((), StatementKind::NotATurtle)),
      },
    },
  }
}

/// statement_ending returns the statement if it has a statement ending.
/// turtle statements end if they have a `.` at the end
/// example:
///  cco:agent_in rdf:type owl:ObjectProperty rdfs:label "agent in"@en .
fn statement_ending(i: &str) -> IResult<&str, &str> {
  has_reached_end_of_statement()(i)
}

/// has_reached_end_of_statement determines that a statement has reached the end when it
/// detects the character `.`
/// example:
///  rdfs:label "Armored Fighting Vehicle"@en .
///  @prefix xml: <http://www.w3.org/XML/1998/namespace> .
fn has_reached_end_of_statement<Input, Error: ParseError<Input>>(
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputIter + Clone + InputTake + TurtleInput,
  <Input as InputIter>::Item: AsChar,
{
  move |i: Input| {
    let input = i;
    match input.iter_elements().size_hint().1 {
      Some(count) => match Some(count) {
        // catch empty input. Input should have a length longer or equal to two
        Some(count) if count >= 0x2 => match input.iter_elements().nth(count - 0x1) {
          Some(last_elm) => match last_elm.as_char() == '.' {
            true => match input.iter_elements().nth(count - 0x2) {
              Some(b_elm) => match b_elm.as_char() == ' ' {
                true => Ok(input.take_right(0x0)),
                false => {
                  let e: ErrorKind = ErrorKind::IsNot;
                  Err(NomErr::Error(Error::from_error_kind(input, e)))
                }
              },
              None => {
                let e: ErrorKind = ErrorKind::IsNot;
                Err(NomErr::Error(Error::from_error_kind(input, e)))
              }
            },
            false => {
              let e: ErrorKind = ErrorKind::IsNot;
              Err(NomErr::Error(Error::from_error_kind(input, e)))
            }
          },
          None => {
            let e: ErrorKind = ErrorKind::IsNot;
            Err(NomErr::Error(Error::from_error_kind(input, e)))
          }
        },
        _ => {
          let e: ErrorKind = ErrorKind::IsNot;
          Err(NomErr::Error(Error::from_error_kind(input, e)))
        }
      },
      None => {
        let e: ErrorKind = ErrorKind::IsNot;
        Err(NomErr::Error(Error::from_error_kind(input, e)))
      }
    }
  }
}

/// statement_part_ending returns the statement if it has a statement part ending.
/// turtle statements parts ends if they have a `;` at the end
/// example:
///  rdfs:subClassOf cco:Certificate ;
fn statement_part_ending(i: &str) -> IResult<&str, &str> {
  has_reached_end_of_part_of_a_statement()(i)
}

fn has_reached_end_of_part_of_a_statement<Input, Error: ParseError<Input>>(
) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputIter + Clone + InputTake + TurtleInput,
  <Input as InputIter>::Item: AsChar,
{
  move |i: Input| {
    let input = i;
    match input.iter_elements().size_hint().1 {
      Some(count) => match Some(count) {
        // catch empty input. Input should have a lenght longer or equal to two
        Some(count) if count >= 0x2 => match input.iter_elements().nth(count - 0x1) {
          Some(last_elm) => match last_elm.as_char() == ';' {
            true => match input.iter_elements().nth(count - 0x2) {
              Some(b_elm) => match b_elm.as_char() == ' ' {
                true => Ok(input.take_right(0x0)),
                false => {
                  let e: ErrorKind = ErrorKind::IsNot;
                  Err(NomErr::Error(Error::from_error_kind(input, e)))
                }
              },
              None => {
                let e: ErrorKind = ErrorKind::IsNot;
                Err(NomErr::Error(Error::from_error_kind(input, e)))
              }
            },
            false => {
              let e: ErrorKind = ErrorKind::IsNot;
              Err(NomErr::Error(Error::from_error_kind(input, e)))
            }
          },
          None => {
            let e: ErrorKind = ErrorKind::IsNot;
            Err(NomErr::Error(Error::from_error_kind(input, e)))
          }
        },
        _ => {
          let e: ErrorKind = ErrorKind::IsNot;
          Err(NomErr::Error(Error::from_error_kind(input, e)))
        }
      },
      None => {
        let e: ErrorKind = ErrorKind::IsNot;
        Err(NomErr::Error(Error::from_error_kind(input, e)))
      }
    }
  }
}

// is_empty_statement determines if a statement is empty by checking its length
// if true, it return an `Ok`
fn is_empty_statement(i: &str) -> IResult<&str, &str> {
  let n = i.trim().len();
  match n {
    0 => Ok(("", i)),
    _ => {
      let e: ErrorKind = ErrorKind::IsNot;
      Err(NomErr::Error(nom::error::Error::from_error_kind(i, e)))
    }
  }
}

/// is_valid_statement_terminator determines if a terminator has
/// been found. This is true if the terminator is the only character in the statement
fn is_valid_statement_terminator(i: &str) -> IResult<&str, &str> {
  let error_response = || {
    let e: ErrorKind = ErrorKind::IsNot;
    Err(NomErr::Error(nom::error::Error::from_error_kind(i, e)))
  };

  if i.trim().len() == 1 {
    let res = find_terminator(i.trim());
    match res {
      Ok(_) => Ok(("", i)),
      Err(_) => error_response(),
    }
  } else {
    error_response()
  }
}

fn find_terminator(i: &str) -> IResult<&str, char> {
  char('.')(i)
}

/// is_a_comment determines if a statement is a comment
/// that is, it begins with `#`
fn is_a_comment(i: &str) -> IResult<&str, &str> {
  tag("#")(i)
}

/// is_a_norm_prefix checks if the statement begins with `@prefix`
fn is_a_norm_prefix(i: &str) -> IResult<&str, &str> {
  tag("@prefix")(i)
}

/// is_a_base_prefix checks if the statement begins with `@base`
fn is_a_base_prefix(i: &str) -> IResult<&str, &str> {
  tag("@base")(i)
}

fn find_and_trim_tail_comment(input: &str) -> IResult<&str, &str> {
  match is_a_comment(input) {
    Ok(_) => {
      let e: ErrorKind = ErrorKind::IsNot;
      Err(NomErr::Error(nom::error::Error::from_error_kind(input, e)))
    }
    Err(_) => match trim_tail_comment(input) {
      Some(s) => Ok(("", s)),
      None => {
        let e: ErrorKind = ErrorKind::IsNot;
        Err(NomErr::Error(nom::error::Error::from_error_kind(input, e)))
      }
    },
  }
}

fn trim_tail_comment(x: &str) -> Option<&str> {
  if x.is_empty() {
    return None;
  }
  let mut ss = vec![];
  for (idx, c) in x.chars().enumerate() {
    let n = x.chars().nth(idx + 1)?;
    if c == '#' && idx != 0x0 && (n == ' ' || n == '#') {
      let l = x.split_at(idx);
      let res: &str = l.0;
      ss.push(res.trim_end());
      break;
    }
  }
  Some(ss[0])
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_find_terminator_if_present0() {
    assert_eq!(is_valid_statement_terminator("."), Ok(("", ".")))
  }

  #[test]
  fn should_find_terminator_if_present1() {
    assert_eq!(
      is_valid_statement_terminator(".t"),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        ".t",
        ErrorKind::IsNot
      )))
    )
  }

  #[test]
  fn should_find_terminator_if_present2() {
    assert_eq!(
      is_valid_statement_terminator("t."),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "t.",
        ErrorKind::IsNot
      )))
    )
  }

  #[test]
  fn should_find_terminator_if_present3() {
    assert_eq!(is_valid_statement_terminator("    ."), Ok(("", "    .")))
  }

  #[test]
  fn should_find_tail_comment0() {
    assert_eq!(trim_tail_comment("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> . # a comment at the tail of statement"), 
    Some("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."))
  }

  #[test]
  fn should_find_tail_comment1() {
    assert_eq!(trim_tail_comment("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> . ## a comment at the tail of statement"), 
    Some("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .")   )
  }

  #[test]
  fn should_find_tail_comment2() {
    assert_eq!(
      trim_tail_comment(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      None
    )
  }

  #[test]
  fn should_find_tail_comment3() {
    assert_eq!(
      trim_tail_comment(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> ."
      ),
      None
    )
  }

  #[test]
  fn should_find_tail_comment5() {
    assert_eq!(
      trim_tail_comment(
        "# this is a comment @base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> ."
      ),
      None
    )
  }

  #[test]
  fn should_know_statement_has_tail_comment0() {
    assert_eq!(find_and_trim_tail_comment("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> . # a comment at the tail of statement"), 
    Ok(("", "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."))
  )
  }

  #[test]
  fn should_know_statement_has_tail_comment1() {
    assert_eq!(find_and_trim_tail_comment("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> . ## a comment at the tail of statement"), 
    Ok(("", "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."))
  )
  }

  #[test]
  fn should_know_statement_has_tail_comment2() {
    assert_eq!(
      find_and_trim_tail_comment(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .",
        ErrorKind::IsNot
      )))
    )
  }

  #[test]
  fn should_know_statement_has_tail_comment3() {
    assert_eq!(
      find_and_trim_tail_comment("# a comment at the tail of statement"),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "# a comment at the tail of statement",
        ErrorKind::IsNot
      )))
    )
  }

  #[test]
  fn should_know_that_a_statement_is_empty0() {
    assert_eq!(is_empty_statement(""), Ok(("", "")))
  }

  #[test]
  fn should_know_that_a_statement_is_empty1() {
    assert_eq!(
      is_empty_statement("cco:AcademicDegree rdf:type owl:Class ;"),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "cco:AcademicDegree rdf:type owl:Class ;",
        ErrorKind::IsNot
      )))
    )
  }

  #[test]
  fn should_know_a_statemenet_has_norm_prefix0() {
    assert_eq!(
      is_a_norm_prefix(
        "@prefix : <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> ."
      ),
      Ok((
        " : <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> .",
        "@prefix"
      ))
    );
  }

  #[test]
  fn should_know_a_statemenet_has_norm_prefix1() {
    assert_eq!(
      is_a_norm_prefix(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .",
        ErrorKind::Tag
      )))
    );
  }

  #[test]
  fn should_know_a_statemenet_has_base_prefix0() {
    assert_eq!(
      is_a_base_prefix(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Ok((
        " <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .",
        "@base"
      ))
    );
  }

  #[test]
  fn should_know_a_statemenet_has_base_prefix1() {
    assert_eq!(
      is_a_base_prefix("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ."),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .",
        ErrorKind::Tag
      )))
    );
  }

  #[test]
  fn should_know_a_statement_is_a_comment() {
    assert_eq!(is_a_comment("#"), Ok(("", "#")));
    assert_eq!(is_a_comment("##########"), Ok(("#########", "#")));
    assert_eq!(
      is_a_comment("#    Object Properties"),
      Ok(("    Object Properties", "#"))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending0() {
    assert_eq!(
      statement_ending("this is a."),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "this is a.",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending1() {
    assert_eq!(
      statement_ending("@prefix skos: <http://www.w3.org/2004/02/skos/core#> ."),
      Ok(("", "@prefix skos: <http://www.w3.org/2004/02/skos/core#> ."))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending2() {
    assert_eq!(
      statement_ending("<http://purl.bioontology.org/ontology/UATC/>"),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "<http://purl.bioontology.org/ontology/UATC/>",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending3() {
    assert_eq!(
      statement_ending("owl:imports <http://www.w3.org/2004/02/skos/core> ;"),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "owl:imports <http://www.w3.org/2004/02/skos/core> ;",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending4() {
    assert_eq!(
      statement_ending(""),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending5() {
    assert_eq!(
      statement_ending("       "),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "       ",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending6() {
    assert_eq!(
      statement_ending(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Ok((
        "",
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ))
    );
  }

  #[test]
  fn should_know_ttl_statement_ending7() {
    assert_eq!(
      statement_ending("this is a."),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "this is a.",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_part_of_statement_end0() {
    assert_eq!(statement_part_ending("<http://www.ontologyrepository.com/CommonCoreOntologies/Mid/FacilityOntology> rdf:type owl:Ontology ;"),
    Ok(("","<http://www.ontologyrepository.com/CommonCoreOntologies/Mid/FacilityOntology> rdf:type owl:Ontology ;"))   )
  }

  #[test]
  fn should_know_part_of_statement_end1() {
    assert_eq!(
      statement_ending("       "),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "       ",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_part_of_statement_end2() {
    assert_eq!(
      statement_ending(""),
      Err(NomErr::Error(nom::error::Error::from_error_kind(
        "",
        ErrorKind::IsNot
      )))
    );
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements0() {
    let res = parse_turtle(
      "@prefix : <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> .",
    );
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::NormPrefix;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements1() {
    let res = parse_turtle(
      "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .",
    );
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::BasePrefix;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements2() {
    let res = parse_turtle("#################################################################");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Comment;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements3() {
    let res = parse_turtle("#    Object Properties");

    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Comment;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements4() {
    let res = parse_turtle("###  http://www.ontologyrepository.com/CommonCoreOntologies/agent_in");

    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Comment;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements5() {
    let res = parse_turtle("###  http://www.ontologyrepository.com/CommonCoreOntologies/agent_in");

    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Comment;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements6() {
    let res = parse_turtle("");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Whitespace;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements7() {
    let res = parse_turtle("        ");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::Whitespace;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements8() {
    let res = parse_turtle("@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> . # a comment at the tail of statement");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        let res1 = StatementKind::BasePrefix;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements9() {
    let res = parse_turtle(".");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::Terminator;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements10() {
    let res = parse_turtle("rdfs:subClassOf <http://purl.bioontology.org/ontology/AIR/U000097> ;");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOf;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements11() {
    let res = parse_turtle("umls:hasSTY <http://purl.bioontology.org/ontology/STY/T047> .");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::StatementWithTerminator;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }
}
