use serde::{Deserialize, Serialize};

use super::rectangle::Rectangle;

/// Represents an SI²R-statistics query.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Query {
    /// The area for which to collect statistics for.
    #[serde(rename = "area")]
    pub area: Rectangle,
}
