pub mod audit {
    pub mod logging;
    pub mod events;
    pub mod verification;
}

pub mod storage {
    pub mod database;
    pub mod blockchain;
    pub mod cloudtrail;
}

pub mod compliance {
    pub mod rules;
    pub mod validation;
    pub mod reporting;
}

pub mod reporting {
    pub mod templates;
    pub mod generation;
    pub mod formats;
}

pub mod metrics {
    pub mod collection;
    pub mod analysis;
    pub mod visualization;
}

pub mod error;
pub mod types;

#[cfg(test)]
mod tests; 