use serde::{Deserialize, Serialize};

pub mod genections;
pub mod genedle;
pub mod spelling_gene;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneNamesResponse<T: Serialize + PartialEq + Eq + Clone> {
    pub(crate) response_header: GeneNamesResponseHeader,
    pub(crate) response: GeneNamesResponseBody<T>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct GeneNamesResponseHeader {
    pub(crate) status: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneNamesResponseBody<T: Serialize + PartialEq + Eq + Clone> {
    pub(crate) num_found: usize,
    pub(crate) docs: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub(crate) struct GeneNamesDoc {
    pub(crate) symbol: String,
}
