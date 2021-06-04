mod parsers;

use parsers::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::declarations::*;

pub fn load_document(path: &str) -> std::io::Result<()> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);

  let mut document = TurtleSignature::new();
  for (_, line) in reader.lines().enumerate() {
    let ln = line?;
    let result = parse_turtle(ln.as_str());
    if let Ok(result) = result {
      let (_, kind) = result;
      // TODO: Assemble all into a data structure the represents the entire document
      match kind {
        StatementKind::Comment => {}
        StatementKind::BasePrefix => {
          let header = TurtleHeaderItem::new(false, "", "", "");
          document.headers.push_back(header)
        }
        StatementKind::NormPrefix => {}
        StatementKind::PartOf => {}
        StatementKind::Whitespace => {}
        StatementKind::None => {}
      }
    }
  }
  Ok(())
}
