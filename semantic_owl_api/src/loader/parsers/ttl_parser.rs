use crate::declarations::turtle::StatementKind;
use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::char,
  error::{ErrorKind, ParseError},
  AsChar, Err as NomErr, IResult, InputIter, InputTake,
};

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
pub(crate) fn parse_turtle(input: &str) -> IResult<(), StatementKind> {
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
                  if x.ends_with('.') && !x.starts_with("@prefix") && !x.starts_with("@base") =>
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
            Some(x)
              if (x.starts_with('[') && x.ends_with(';')) || has_tail_collection_ending(x) =>
            {
              Ok(((), StatementKind::PartOfCollectionList))
            } // parse part of collection list

            Some(x) if x.ends_with(';') && has_subject_in_predicate(x) && !is_a_literal(x) => {
              Ok(((), StatementKind::PartOfPredicateListWithSubject))
            } // parse part of predicate list with subject

            Some(x) if x.ends_with(';') && !has_subject_in_predicate(x) && !is_a_literal(x) => {
              Ok(((), StatementKind::PartOfPredicateList))
            } // parse part of predicate list

            Some(x) if x.ends_with(',') && has_predicate_in_object(x) && !is_a_literal(x) => {
              Ok(((), StatementKind::PartOfObjectListWithPredicate))
            } // parse part of object list with predicate

            Some(x) if x.ends_with(',') && !has_predicate_in_object(x) && !is_a_literal(x) => {
              Ok(((), StatementKind::PartOfObjectList))
            } // parse part of object list

            Some(x) if !has_predicate_in_object(x) && is_a_literal(x) => {
              Ok(((), StatementKind::PartOfObjectListAsLiteral))
            }

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
  Input: InputIter + Clone + Copy + InputTake + TurtleInput,
  <Input as InputIter>::Item: AsChar + Copy,
{
  move |i: Input| {
    let input = i;
    match input.iter_elements().size_hint().1 {
      Some(count) => match Some(count) {
        // catch empty input. Input should have a lenght longer or equal to two
        Some(count) if count >= 0x2 => match input.iter_elements().nth(count - 0x1) {
          Some(last_elm) => {
            if last_elm.as_char() == ',' || last_elm.as_char() == ';' {
              match input.iter_elements().nth(count - 0x2) {
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
              }
            } else {
              let e: ErrorKind = ErrorKind::IsNot;
              Err(NomErr::Error(Error::from_error_kind(input, e)))
            }
          }

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

// given a base staement of the form @base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> .
// `get_base_iri_from_raw_statement` returns an Option of `<http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology>`
pub(crate) fn get_base_iri_from_raw_statement(raw: &str) -> Option<String> {
  let x = raw.strip_prefix("@base")?;
  let x = x.strip_suffix('.')?;
  let x = x.trim();
  Some(String::from(x))
}

// given a prefix statement of the form @prefix cco: <http://www.ontologyrepository.com/CommonCoreOntologies/> . or
// @prefix : <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> .,
// `get_prefix_iri_from_raw_statement` returns an Option of `cco`
pub(crate) fn get_prefix_iri_from_raw_statement(raw: &str) -> Option<(String, bool)> {
  let x = raw.strip_prefix("@prefix")?;
  let x = x.strip_suffix('.')?;
  let x = x.trim();
  let x: Vec<&str> = x.split(':').collect();
  let x = x[0x0];
  Some((String::from(x), x.is_empty()))
}

// given a statement of the form -> owl:someValuesFrom cco:Velocity ] ;
// returns the `true`
fn has_tail_collection_ending(raw: &str) -> bool {
  let x = raw.trim();
  let x: Vec<&str> = x.split_whitespace().collect();
  x[x.len() - 0x1] == ";" && x[x.len() - 0x2] == "]"
}

fn has_subject_in_predicate(x: &str) -> bool {
  let n: Vec<&str> = x.split(' ').collect();
  let second_part = n[0x1];
  let n1: Vec<&str> = second_part.split(':').collect();

  if n1.len() != 0x2 || (n1.len() == 0x2 && n.len() == 0x3) {
    false
  } else {
    true
  }
}

fn has_predicate_in_object(x: &str) -> bool {
  let n: Vec<&str> = x.split(' ').collect();
  let first_part = n[0x0];
  let next_part = n[0x1];
  let n1: Vec<&str> = first_part.split(':').collect();
  if n1.len() != 0x2 || next_part == "," {
    false
  } else {
    true
  }
}

fn is_a_literal(x: &str) -> bool {
  let n: Vec<&str> = x.split(' ').collect();
  let first_part = n[0x0];
  let n1: Vec<&str> = first_part.split(':').collect();
  if n1.len() == 0x1 && (x.ends_with(',') || x.ends_with(';')) {
    true
  } else {
    false
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_know_statement_is_a_literal() {
    assert_eq!(is_a_literal("obo:IAO_0000112 \"my body has part my brain (continuant parthood, two material entities)\"@en ,")  , false);
    assert_eq!(
      is_a_literal(
        "\"my body has part my brain (continuant parthood, two material entities)\"@en ,"
      ),
      true
    );
    assert_eq!(
      is_a_literal(
        "\"my body has part my brain (continuant parthood, two material entities)\"@en ;"
      ),
      true
    )
  }

  #[test]
  fn should_know_statement_has_predicate_in_object() {
    assert_eq!(has_predicate_in_object("obo:IAO_0000112 \"my body has part my brain (continuant parthood, two material entities)\"@en ,"),true);

    assert_eq!(
      has_predicate_in_object(
        "\"my body has part my brain (continuant parthood, two material entities)\"@en ,"
      ),
      false
    );

    assert_eq!(has_predicate_in_object("obo:BFO_0000004 ,"), false)
  }

  #[test]
  fn should_know_statement_has_subject_in_predicate() {
    assert_eq!(
      has_subject_in_predicate("cco:doctrinal_source rdf:type owl:AnnotationProperty ;"),
      true
    );
    assert_eq!(has_subject_in_predicate("cco:definition \"A Process Profile that is the rate of change of the Velocity of an object.\"@en ;"), false  );
    assert_eq!(
      has_subject_in_predicate("rdfs:subClassOf obo:BFO_0000015 ;"),
      false
    )
  }

  #[test]
  fn should_know_statement_has_tail_collection_ending() {
    assert_eq!(
      has_tail_collection_ending("owl:someValuesFrom cco:Velocity ] ;"),
      true
    );

    assert_eq!(
      has_tail_collection_ending("owl:someValuesFrom cco:Velocity  ;"),
      false
    )
  }

  #[test]
  fn should_return_prefix_iri_from_statement() {
    assert_eq!(
      get_prefix_iri_from_raw_statement(
        "@prefix cco: <http://www.ontologyrepository.com/CommonCoreOntologies/> ."
      ),
      Some((String::from("cco"), false))
    );

    assert_eq!(
      get_prefix_iri_from_raw_statement(
        "@prefix : <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology#> ."
      ),
      Some((String::from(""), true))
    );

    assert_eq!(
      get_prefix_iri_from_raw_statement(
        "cco: <http://www.ontologyrepository.com/CommonCoreOntologies/> ."
      ),
      None
    )
  }

  #[test]
  fn should_return_base_iri_from_statement() {
    assert_eq!(
      get_base_iri_from_raw_statement(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Some(String::from(
        "<http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology>"
      ))
    );

    assert_ne!(
      get_base_iri_from_raw_statement(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Some(String::from(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ))
    );

    assert_ne!(
      get_base_iri_from_raw_statement(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Some(String::from("@base"))
    );

    assert_ne!(
      get_base_iri_from_raw_statement(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Some(String::from(
        " <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> "
      ))
    );

    assert_ne!(
      get_base_iri_from_raw_statement(
        "@base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/AgentOntology> ."
      ),
      Some(String::from(""))
    );
  }

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
        let res1 = StatementKind::PartOfPredicateList;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements11() {
    let res = parse_turtle("cco:process_precedes rdf:type owl:ObjectProperty ;");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOfPredicateListWithSubject;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements12() {
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

  #[test]
  fn should_know_to_correctly_parse_turtle_statements13() {
    let res = parse_turtle("obo:RO_0040042 obo:BFO_0000002 ,");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOfObjectListWithPredicate;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements14() {
    let res = parse_turtle("obo:BFO_0000004 ,");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOfObjectList;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements15() {
    let res = parse_turtle("\"my stomach has part my stomach cavity (continuant parthood, material entity has part immaterial entity)\"@en ,");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOfObjectListAsLiteral;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }

  #[test]
  fn should_know_to_correctly_parse_turtle_statements16() {
    let res = parse_turtle("\"my stomach has part my stomach cavity (continuant parthood, material entity has part immaterial entity)\"@en ;");
    match res {
      Ok(elm) => {
        let (_, res0) = elm;
        println!("{:?}", res0);
        let res1 = StatementKind::PartOfObjectListAsLiteral;
        assert_eq!(std::mem::discriminant(&res0), std::mem::discriminant(&res1));
      }
      Err(_) => {}
    }
  }
}
