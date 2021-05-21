mod declarations;
pub use crate::declarations::*;

#[cfg(test)]
mod tests {
    use super::declarations as pre;
    use std::mem;

    #[test]
    fn should_return_rdf_prefix() {
        let prefix1 = mem::discriminant(&pre::get_rdf_prefix());
        let prefix2 = mem::discriminant(&pre::OwlStdPrefix::Rdf(pre::PrefixObject::new(
            "rdf:",
            "<http://www.w3.org/1999/02/22-rdf-syntax-ns#>",
        )));
        assert_eq!(prefix1, prefix2);
    }

    #[test]
    fn should_know_rdf_is_not_equal_to_rdfs() {
        let prefix1 = mem::discriminant(&pre::get_rdf_prefix());
        let prefix2 = mem::discriminant(&pre::get_rdfs_prefix());
        assert_ne!(prefix1, prefix2);
    }

    #[test]
    fn should_know_xsd_is_not_equal_to_owl() {
        let prefix1 = mem::discriminant(&pre::get_xsd_prefix());
        let prefix2 = mem::discriminant(&pre::get_owl_prefix());

        assert_ne!(prefix1, prefix2);
    }
}
