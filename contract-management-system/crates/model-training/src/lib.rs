pub mod training {
    pub mod pipeline;
    pub mod optimizer;
    pub mod validation;
    pub mod metrics;
}

pub mod data {
    pub mod preprocessing;
    pub mod batching;
    pub mod augmentation;
}

pub mod models {
    pub mod architecture;
    pub mod layers;
    pub mod loss;
}

pub mod enclave {
    pub mod integration;
    pub mod security;
    pub mod resources;
}

pub mod monitoring {
    pub mod metrics;
    pub mod logging;
    pub mod alerts;
}

pub mod error;
pub mod types;

#[cfg(test)]
mod tests; 