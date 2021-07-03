use crate::declarations::turtle::*;
use crate::loader::parsers::ttl_parser::{
  get_base_iri_from_raw_statement, get_prefix_iri_from_raw_statement, parse_turtle,
};

use indicatif::ProgressBar;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// load_turtle_document is the main entry point for loading a turtle document
/// It will return an error document is not a turtle document
pub fn load_turtle_document(path: &str) -> std::io::Result<TurtleDocument> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let mut document = TurtleDocument::new();

  let pb = ProgressBar::new(reader.buffer().len().try_into().unwrap());

  for line in reader.lines() {
    pb.inc(1);
    let ln = line?;
    let result = parse_turtle(ln.as_str());
    if let Ok(result) = result {
      let (_, kind) = result;

      match kind {
        // don't anything. just move to the next statement
        StatementKind::Comment | StatementKind::Whitespace | StatementKind::None => continue,

        // base prefix has been encountered. This should be reached only once
        StatementKind::BasePrefix => {
          let header = TurtleHeaderItem::new(
            true,
            false,
            None,
            get_base_iri_from_raw_statement(&ln),
            Some(ln),
          );
          document.headers.push_back(header);
          continue;
        }

        // a prefix statement has been encountered
        StatementKind::NormPrefix => match get_prefix_iri_from_raw_statement(&ln) {
          Some(r) => {
            let (ns, is_empty) = r;
            let header = TurtleHeaderItem::new(false, is_empty, Some(ns), None, Some(ln));
            document.headers.push_back(header);
            continue;
          }
          None => continue,
        },

        StatementKind::PartOfPredicateListWithSubject => {
          println!("part of predicate list with subject found {:?}", ln);
          continue;
        }

        StatementKind::PartOfPredicateList => {
          println!("part of predicate list found {:?}", ln);
          continue;
        }

        StatementKind::PartOfObjectListWithPredicate => {
          println!("part of object list with predicate found {:?}", ln);
          continue;
        }

        StatementKind::PartOfObjectListAsLiteral => {
          println!("part of object list as literal found {:?}", ln);
          continue;
        }

        StatementKind::PartOfObjectList => {
          println!("part of object list found {:?}", ln);
          continue;
        }

        StatementKind::PartOfCollectionList => {
          println!("part of collection list found {:?}", ln);
          continue;
        }

        StatementKind::StatementWithTerminator => {
          println!("statement with terminator found found {:?}", ln);
        }

        StatementKind::Terminator => {
          println!("statement terminator found {:?}", ln);
          continue;
        }

        // not parser has passed, meaning the provider document is not a valid turtle document
        StatementKind::NotATurtle => {
          return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the provided file is not a turtle document",
          ));
        }
      }
    }
  }

  pb.finish_and_clear();
  Ok(document)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::current_dir;
  use tokio_test::assert_ok;

  #[test]
  fn should_load_document0() -> std::io::Result<()> {
    // todo(write a similar test then pass a non-turtle.expect an error)
    let wd = current_dir()?;
    let root = wd.parent().unwrap();
    let root = root.join("testdata/turtle/CurrencyUnitOntology.ttl");
    let path = root.to_str().unwrap();
    match load_turtle_document(path) {
      Ok(d) => {
        assert_ok!(serde_json::to_string(&d));
      }
      Err(_) => panic!("did not expect"),
    }

    Ok(())
  }
}
