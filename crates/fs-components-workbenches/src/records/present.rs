//! How the canonical texts of a field become the words of the reader. Until the field components exist for a basic type, the
//! values are joined; a component replaces this hook for its type, the adapter does not choose it.
//!
//! Pattern: Strategy (`Present`) with the plain join as the default.

use std::sync::Arc;

/// Field name and canonical values to the text the reader sees.
pub type Present = Arc<dyn Fn(&str, &[String]) -> String + Send + Sync>;

/// The default: the values joined by a comma.
pub fn join_values(_field: &str, values: &[String]) -> String {
    values.join(", ")
}
