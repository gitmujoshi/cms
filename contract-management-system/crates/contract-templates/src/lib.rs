pub mod templates {
    pub mod ai_training;
    pub mod data_sharing;
    pub mod model_ownership;
    pub mod compliance;
}

pub mod validation {
    pub mod schema;
    pub mod rules;
    pub mod constraints;
}

pub mod storage {
    pub mod s3;
    pub mod versioning;
}

pub mod rendering {
    pub mod handlebars;
    pub mod tera;
    pub mod variables;
}

pub mod error;
pub mod types;

#[cfg(test)]
mod tests; 