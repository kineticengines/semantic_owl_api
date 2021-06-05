use nom::{
  branch::alt,
  bytes::complete::tag,
  error::{ErrorKind, ParseError},
  AsChar, Err as NomErr, IResult, InputIter, InputTake,
};

use crate::declarations::*;
use std::convert::From;

/// Naive custom error
/// TODO: revisit this
#[derive(Debug)]
struct TurtleParseError(String);

impl<'a> From<(&'a str, ErrorKind)> for TurtleParseError {
  fn from(error: (&'a str, ErrorKind)) -> Self {
    TurtleParseError(format!("error code was: {:?}", error))
  }
}

impl<'a> ParseError<&'a str> for TurtleParseError {
  fn from_error_kind(_: &'a str, kind: ErrorKind) -> Self {
    TurtleParseError(format!("error code was: {:?}", kind))
  }

  fn append(_: &'a str, kind: ErrorKind, other: TurtleParseError) -> Self {
    TurtleParseError(format!("{:?}\nerror code was: {:?}", other, kind))
  }
}

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
  match alt((is_a_comment, line_ending, is_empty_statement))(input) {
    Ok(elements) => {
      let (_, right_elm) = elements;
      match Some(right_elm) {
        Some(x) if x.starts_with('#') && x.len() == 0x1 => Ok(((), StatementKind::Comment)), // parse comments
        Some(x) if x.is_empty() => Ok(((), StatementKind::Whitespace)), // parse whitespaces
        _ => match alt((is_a_norm_prefix, is_a_base_prefix))(input) {
          Ok(elements) => {
            let (_, right_elm) = elements;
            match Some(right_elm) {
              Some(x) if x.starts_with("@prefix") => Ok(((), StatementKind::NormPrefix)), // parse norm prefix
              Some(x) if x.starts_with("@base") => Ok(((), StatementKind::BasePrefix)), // parse base prefix
              _ => Ok(((), StatementKind::None)),
            }
          }
          Err(_) => Ok(((), StatementKind::None)),
        },
      }
    }
    Err(_) => Ok(((), StatementKind::None)),
  }
}

/// line_ending returns the statement if it has a statement ending.
/// turtle statements end if they have a `.` at the end
/// example:
///  cco:agent_in rdf:type owl:ObjectProperty rdfs:label "agent in"@en .
fn line_ending(i: &str) -> IResult<&str, &str> {
  has_reached_end_of_statement()(i)
}

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
        // catch empty input. Input should have a lenght longer or equal to two
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
    line_ending("this is a."),
    Err(NomErr::Error(nom::error::Error::from_error_kind(
      "this is a.",
      ErrorKind::IsNot
    )))
  );
}

#[test]
fn should_know_ttl_statement_ending1() {
  assert_eq!(
    line_ending("@prefix skos: <http://www.w3.org/2004/02/skos/core#> ."),
    Ok(("", "@prefix skos: <http://www.w3.org/2004/02/skos/core#> ."))
  );
}

#[test]
fn should_know_ttl_statement_ending2() {
  assert_eq!(
    line_ending("<http://purl.bioontology.org/ontology/UATC/>"),
    Err(NomErr::Error(nom::error::Error::from_error_kind(
      "<http://purl.bioontology.org/ontology/UATC/>",
      ErrorKind::IsNot
    )))
  );
}

#[test]
fn should_know_ttl_statement_ending3() {
  assert_eq!(
    line_ending("owl:imports <http://www.w3.org/2004/02/skos/core> ;"),
    Err(NomErr::Error(nom::error::Error::from_error_kind(
      "owl:imports <http://www.w3.org/2004/02/skos/core> ;",
      ErrorKind::IsNot
    )))
  );
}

#[test]
fn should_know_ttl_statement_ending4() {
  assert_eq!(
    line_ending(""),
    Err(NomErr::Error(nom::error::Error::from_error_kind(
      "",
      ErrorKind::IsNot
    )))
  );
}

#[test]
fn should_know_ttl_statement_ending5() {
  assert_eq!(
    line_ending("       "),
    Err(NomErr::Error(nom::error::Error::from_error_kind(
      "       ",
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
