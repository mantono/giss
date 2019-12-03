pub mod ghrs {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    pub enum State {
        #[serde(rename(deserialize = "open"))]
        #[serde(alias = "OPEN")]
        Open,
        #[serde(rename(deserialize = "closed"))]
        #[serde(alias = "CLOSED")]
        Closed,
    }

    pub trait Closeable {
        fn is_open(&self) -> bool;
    }

    pub trait ModDate: Ord {
        fn mod_time(&self) -> u64;
    }
}
