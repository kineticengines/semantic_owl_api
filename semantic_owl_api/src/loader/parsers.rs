use nom::{
  error::{ErrorKind, ParseError},
  AsChar, Err as NomErr, IResult, InputIter, InputTake,
};

use std::convert::From;

pub mod ttl_parser {
  use super::*;

  #[derive(Debug)]
  pub struct TurtleParseError(String);

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
  pub trait TurtleInput: InputTake {
    /// Split the stream at the `count` byte offset returning
    /// the full input on the right part of the tuple
    fn take_right(&self, count: usize) -> (Self, Self);
  }

  impl<'a> TurtleInput for &'a str {
    // return byte index
    #[inline]
    fn take_right(&self, count: usize) -> (Self, Self) {
      (&self[..count], &self[count..])
    }
  }

  pub fn line_ending(i: &str) -> IResult<&str, &str> {
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
        Some(count) => match input.iter_elements().nth(count - 0x1) {
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
        None => {
          let e: ErrorKind = ErrorKind::IsNot;
          Err(NomErr::Error(Error::from_error_kind(input, e)))
        }
      }
    }
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
}
