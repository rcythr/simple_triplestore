use crate::{prelude::*, PropertiesType};
use serde::{de::DeserializeOwned, Serialize};

mod extend;
mod insert;
mod iter;
mod merge;
mod query;
mod remove;
mod set;

pub enum Error {
    SledError(sled::Error),
    SerializationError(bincode::Error),
    KeySizeError,
    MissingPropertyData,
    SetOpsFailure,
}

/// A triplestore which is backed by a Sled database.
pub struct SledTripleStore<
    NodeProperties: PropertiesType + Serialize + DeserializeOwned,
    EdgeProperties: Serialize + DeserializeOwned,
> {
    _phantom: std::marker::PhantomData<(NodeProperties, EdgeProperties)>,
    node_props: sled::Tree,
    edge_props: sled::Tree,
    spo_data: sled::Tree,
    pos_data: sled::Tree,
    osp_data: sled::Tree,
}

impl<
        NodeProperties: PropertiesType + Serialize + DeserializeOwned,
        EdgeProperties: PropertiesType + Serialize + DeserializeOwned,
    > SledTripleStore<NodeProperties, EdgeProperties>
{
    pub fn new(db: &sled::Db) -> Result<Self, sled::Error> {
        let node_data = db.open_tree(b"node_data")?;
        let edge_data = db.open_tree(b"edge_data")?;
        let spo_data = db.open_tree(b"spo_data")?;
        let pos_data = db.open_tree(b"pos_data")?;
        let osp_data = db.open_tree(b"osp_data")?;

        Ok(Self {
            node_props: node_data,
            edge_props: edge_data,
            spo_data,
            pos_data,
            osp_data,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<
        NodeProperties: PropertiesType + Serialize + DeserializeOwned,
        EdgeProperties: PropertiesType + Serialize + DeserializeOwned,
    > TripleStoreError for SledTripleStore<NodeProperties, EdgeProperties>
{
    type Error = Error;
}
