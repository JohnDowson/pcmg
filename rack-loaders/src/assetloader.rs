use std::collections::BTreeMap;

use futures::channel::mpsc;
use rack::Uuidentified;
use rust_embed::RustEmbed;

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on as spawn;

use serde::{
    de::DeserializeOwned,
    Serialize,
};
use uuid::Uuid;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as spawn;

#[allow(unused_imports)]
use crate::saveloaders::{
    load_from_base64,
    save_to_base64,
};

#[derive(RustEmbed)]
#[folder = "../prefabs/"]
pub struct WidgetPrefab;

#[derive(RustEmbed)]
#[folder = "../prefab_modules/"]
pub struct ModulePrefab;

pub struct AssetLoader<T> {
    #[cfg(target_arch = "wasm32")]
    storage: web_sys::Storage,
    #[cfg(target_arch = "wasm32")]
    store: &'static str,
    assets: BTreeMap<Uuid, T>,
    channel: (mpsc::Sender<T>, mpsc::Receiver<T>),
}

impl<T> AssetLoader<T>
where
    T: Serialize + DeserializeOwned + Uuidentified + 'static,
{
    pub fn new(
        #[cfg(target_arch = "wasm32")] store: &'static str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        let (storage, assets) = {
            let window = web_sys::window().ok_or("Could not get window")?;
            let storage = window
                .local_storage()
                .map_err(|e| format!("{e:?}"))?
                .ok_or("Could not get localStorage")?;
            let assets = storage.get_item(store).map_err(|e| format!("{e:?}"))?;
            let assets = if let Some(assets) = assets {
                log::debug!("{assets:?}");
                load_from_base64(&assets).ok_or("Failed to deserialize")?
            } else {
                BTreeMap::default()
            };
            (storage, assets)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let assets = BTreeMap::default();

        let this = Self {
            #[cfg(target_arch = "wasm32")]
            storage,
            #[cfg(target_arch = "wasm32")]
            store,
            assets,
            channel: mpsc::channel(10),
        };
        this.save()?;
        Ok(this)
    }

    pub fn insert(&mut self, asset: T) -> Result<(), Box<dyn std::error::Error>> {
        self.assets.insert(asset.uuid(), asset);
        self.save()
    }

    pub fn get(&self, uuid: Uuid) -> Option<T>
    where
        T: Clone,
    {
        self.assets.get(&uuid).cloned()
    }

    pub fn assets(&self) -> BTreeMap<Uuid, T>
    where
        T: Clone,
    {
        self.assets.clone()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        {
            let b64 = save_to_base64(&self.assets).ok_or("Failed to serialize")?;
            let res = self.storage.set_item(self.store, &b64);
            if let Err(e) = res {
                log::warn!("Failed to save an asset to LocalStorage because of: {e:?}");
            }
        }
        Ok(())
    }

    pub fn drive(&mut self) {
        match self.channel.1.try_next() {
            Ok(Some(asset)) => {
                self.insert(asset).unwrap();
            }
            Ok(None) => panic!("Closed"),
            Err(_) => {}
        }
    }

    pub fn load_embeds<E: RustEmbed>(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for asset in E::iter() {
            let asset = E::get(&asset).unwrap();
            let asset: T = serde_yaml::from_slice(&asset.data)?;
            self.insert(asset)?;
        }
        Ok(())
    }

    pub fn load_b64(&mut self, b64: &str) {
        let asset = load_from_base64(b64);
        asset.map(|a| self.insert(a));
    }

    pub fn load(&self) {
        let mut tx = self.channel.0.clone();

        spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .set_directory(".")
                .pick_files()
                .await;
            match file {
                None => {}
                Some(files) => {
                    // TODO: error handling
                    for file in files.into_iter() {
                        let file = file.read().await;
                        let asset: T = serde_yaml::from_slice(&file).unwrap();
                        _ = tx.try_send(asset);
                    }
                }
            };
        });
    }
}
