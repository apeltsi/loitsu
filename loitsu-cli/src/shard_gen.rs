use loitsu::scene_management::Scene;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};

#[derive(Debug, Clone)]
pub struct Shard {
    pub assets: Vec<String>,
    pub is_root: bool,
    pub dependents: Vec<String>,
    pub name: String
}

impl PartialEq for Shard {
    fn eq(&self, other: &Self) -> bool {
        for asset in &self.assets {
            if !other.assets.contains(asset) {
                return false;
            }
        }
        for dependent in &self.dependents {
            if !other.dependents.contains(dependent) {
                return false;
            }
        }
        self.is_root == other.is_root
    }
}

impl Shard {
    pub fn new(assets: Vec<String>, is_root: bool, dependents: Vec<String>) -> Shard {
        Shard {
            assets,
            is_root,
            dependents,
            name: "".to_string()
        }
    }

    pub fn generate_name(&mut self) {
        // we'll use a hash of the assets to generate a name
        let mut hasher = DefaultHasher::new();
        self.assets.hash(&mut hasher);
        let hash = hasher.finish();

        self.name = format!("{:X}", hash);
    }

    pub fn analyse_against(&self, others: &Vec<Shard>) -> Option<Vec<Shard>> {
        // lets go through all of the other shards and see if we have any duplicates
        // if we do, we need to remove them from our assets and their assets and create
        // a new shard with the duplicates
        let mut is_different = false;
        let mut others = others.clone();
    
        let index_of_self = others.iter().position(|x| x == self).unwrap();

        for i in 0..others.len() {
            let other = others[i].clone();
            if other == others[index_of_self].clone() {
                continue;
            }
            let duplicates = others[index_of_self].find_intersection(&other);
            if duplicates.len() > 0 {
                // lets remove the duplicates from both shards
                others[index_of_self].remove_items(duplicates.clone());
                others[i].remove_items(duplicates.clone());
                // lets create a new shard with the duplicates
                let new_shard = Shard::new(duplicates, false, others[index_of_self].dependents.clone().into_iter().chain(other.dependents.clone()).collect());
                // lets add the new shard to the list of shards
                others.push(new_shard);
                is_different = true;
            }
        }
        if !is_different {
            return None;
        }
        Some(others)
    }

    pub fn find_intersection(&self, other: &Shard) -> Vec<String> {
        let mut duplicates = Vec::new();
        for asset in &self.assets {
            if other.assets.contains(asset) {
                duplicates.push(asset.clone());
            }
        }
        duplicates
    }

    pub fn remove_items(&mut self, items: Vec<String>) {
        // lets remove any items from our assets, if we have them
        for item in items {
            if let Some(index) = self.assets.iter().position(|x| *x == item) {
                self.assets.remove(index);
            }
        }
    }
}

pub fn generate_shards(scenes: Vec<Scene>) -> Vec<Shard> {
    let mut initial_shards = Vec::new();
    for scene in scenes {
        initial_shards.push(Shard::new(scene.required_assets, true, vec![scene.name]));
    }
    let mut did_change = true;

    // actually process the shards
    let mut shards: Vec<Shard> = initial_shards;
    while did_change {
        did_change = false;
        for shard in shards.clone() {
            let result = shard.analyse_against(&shards);
            match result {
                Some(new_shards) => {
                    shards = new_shards;
                    did_change = true;
                },
                None => {}
            }
        }
    }

    // remove any empty shards
    shards = shards.into_iter().filter(|x| x.assets.len() > 0).collect();
    
    // and finally lets generate our names
    for i in 0..shards.len() {
        shards[i].generate_name();
    }

    shards
}
