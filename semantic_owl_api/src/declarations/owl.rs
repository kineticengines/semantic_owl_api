pub enum OwlSyntax {
  Functional,
  Turtle,
  RdfXml,
  OwlXml,
  Manchester,
}

pub trait RDFDocumentMapperToOwl {
  fn map_to_owl(&self);
}
