mod parsers;

use parsers::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use indicatif::ProgressBar;

use crate::declarations::*;

/// load_turtle_document is the main entry point for loading a turtle document
/// It will return an error document is not a turtle document
pub fn load_turtle_document(path: &str) -> std::io::Result<()> {
  let file = File::open(path)?;
  let file_copy = file.try_clone()?;
  let reader = BufReader::new(file);
  let reader2 = BufReader::new(file_copy);
  let mut document = TurtleDocument::new();

  let count = reader2.lines().count();
  let count = count.clone();

  let pb = ProgressBar::new(count as u64);

  for (_, line) in reader.lines().enumerate() {
    pb.inc(1);
    let ln = line?;
    let result = parse_turtle(ln.as_str());
    if let Ok(result) = result {
      let (_, kind) = result;

      match kind {
        StatementKind::Comment | StatementKind::Whitespace | StatementKind::None => {} // don't anything. just move to the next statement

        // base prefix has been encountered. This should be reached only once
        StatementKind::BasePrefix => {
          let header = TurtleHeaderItem::new(true, "", "", "");
          document.headers.push_back(header)
        }

        // a prefix statement has been encountered
        StatementKind::NormPrefix => {
          let header = TurtleHeaderItem::new(false, "", "", "");
          document.headers.push_back(header)
        }

        // a valid end or terminator has been encountered
        StatementKind::PartOf
        | StatementKind::StatementWithTerminator
        | StatementKind::Terminator => {}

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
  pb.finish();
  Ok(())
}
