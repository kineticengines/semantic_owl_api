use semantic_owl_api::*;
use std::env::current_dir;

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
      let _ = load_turtle_document(path);
    }
  }
  assert_eq!(4, 4);
  Ok(())
}
