pub mod runtime {
    pub mod initialization;
    pub mod lifecycle;
    pub mod resources;
}

pub mod attestation {
    pub mod verification;
    pub mod proof;
    pub mod pcrs;
}

pub mod security {
    pub mod crypto;
    pub mod keys;
    pub mod memory;
}

pub mod compute {
    pub mod model;
    pub mod data;
    pub mod metrics;
}

pub mod communication {
    pub mod protocol;
    pub mod messages;
    pub mod channels;
}

pub mod error;
pub mod types;

#[cfg(test)]
mod tests; 