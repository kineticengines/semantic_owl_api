#[derive(Debug, PartialEq)]
pub struct PrefixObject<'a> {
  pub prefix_name: &'a str,
  pub prefix_iri: &'a str,
}

impl<'a> PrefixObject<'a> {
  pub fn new(prefix_name: &'a str, prefix_iri: &'a str) -> PrefixObject<'a> {
    PrefixObject {
      prefix_name,
      prefix_iri,
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum OwlStdPrefix<'a> {
  Rdf(PrefixObject<'a>),
  Rdfs(PrefixObject<'a>),
  Xsd(PrefixObject<'a>),
  Owl(PrefixObject<'a>),
}

pub fn get_rdf_prefix<'a>() -> OwlStdPrefix<'a> {
  OwlStdPrefix::Rdf(PrefixObject {
    prefix_name: "rdf:",
    prefix_iri: "<http://www.w3.org/1999/02/22-rdf-syntax-ns#>",
  })
}

pub fn get_rdfs_prefix<'a>() -> OwlStdPrefix<'a> {
  OwlStdPrefix::Rdfs(PrefixObject {
    prefix_name: "rdfs:",
    prefix_iri: "<http://www.w3.org/2000/01/rdf-schema#>",
  })
}

pub fn get_xsd_prefix<'a>() -> OwlStdPrefix<'a> {
  OwlStdPrefix::Xsd(PrefixObject {
    prefix_name: "xsd:",
    prefix_iri: "<http://www.w3.org/2001/XMLSchema#>",
  })
}

pub fn get_owl_prefix<'a>() -> OwlStdPrefix<'a> {
  OwlStdPrefix::Owl(PrefixObject {
    prefix_name: "owl:",
    prefix_iri: "<http://www.w3.org/2002/07/owl#>",
  })
}


#[cfg(test)]
mod tests{
  use super::*;

  #[test]
  fn should_return_rdf_prefix() {
    let prefix1 = std::mem::discriminant(&get_rdf_prefix());
    let prefix2 = std::mem::discriminant(&OwlStdPrefix::Rdf(PrefixObject::new(
      "rdf:",
      "<http://www.w3.org/1999/02/22-rdf-syntax-ns#>",
    )));
    assert_eq!(prefix1, prefix2);
  }

  #[test]
  fn should_know_rdf_is_not_equal_to_rdfs() {
    let prefix1 = std::mem::discriminant(&get_rdf_prefix());
    let prefix2 = std::mem::discriminant(&get_rdfs_prefix());
    assert_ne!(prefix1, prefix2);
  }

  #[test]
  fn should_know_xsd_is_not_equal_to_owl() {
    let prefix1 = std::mem::discriminant(&get_xsd_prefix());
    let prefix2 = std::mem::discriminant(&get_owl_prefix());

    assert_ne!(prefix1, prefix2);
  }
}



