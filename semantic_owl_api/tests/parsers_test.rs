use std::env::current_dir;

use semantic_owl_api::loader::load::load_turtle_document;

#[test]
fn should_parse_turle_document() -> std::io::Result<()> {
  let wd = current_dir()?;
  let root = wd.parent().unwrap();
  let root = root.join("testdata/turtle");
  for file in std::fs::read_dir(root)? {
    let file = file?;
    let path = file.path();
    let path = path.to_str().unwrap();
    if path.ends_with(".ttl") {
      match load_turtle_document(path) {
        Ok(d) => assert_ne!(d.headers.len(), 0),
        Err(err) => assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput),
      }
    }
  }
  assert_eq!(4, 4);
  Ok(())
}
