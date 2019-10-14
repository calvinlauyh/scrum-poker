use super::provider::Provider;
use std::collections::HashMap;

pub struct ProviderService {
    providers: HashMap<String, Box<dyn Provider + Send + Sync>>,
}

impl ProviderService {
    pub fn new(providers: Vec<Box<dyn Provider + Send + Sync>>) -> Self {
        let mut service = ProviderService {
            providers: HashMap::new(),
        };

        for provider in providers {
            service
                .providers
                .insert(provider.provider_id().to_string(), provider);
        }

        service
    }

    pub fn get(&self, provider_id: &str) -> Option<&Box<dyn Provider + Send + Sync>> {
        self.providers.get(provider_id)
    }
}
