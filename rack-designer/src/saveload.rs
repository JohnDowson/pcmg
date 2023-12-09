use std::collections::BTreeMap;

use futures::channel::{
    mpsc,
    oneshot::{
        channel,
        Receiver,
    },
};
use rack::Uuidentified;
use rust_embed::RustEmbed;
use serde::{
    de::DeserializeOwned,
    Serialize,
};

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on as spawn;

use uuid::Uuid;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as spawn;

pub fn loader<T: DeserializeOwned + 'static>() -> Receiver<Option<T>> {
    let (tx, rx) = channel();

    spawn(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .pick_file()
            .await;
        _ = match file {
            None => tx.send(None),
            Some(file) => {
                // TODO: error handling
                let file = file.read().await;
                let asset: T = serde_yaml::from_slice(&file).unwrap();
                tx.send(Some(asset))
            }
        };
    });
    rx
}

pub fn loader_many<T: DeserializeOwned + 'static>() -> Receiver<Option<Vec<T>>> {
    let (tx, rx) = channel();

    spawn(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .pick_files()
            .await;
        _ = match file {
            None => tx.send(None),
            Some(files) => {
                // TODO: error handling
                let mut loaded = Vec::with_capacity(files.len());
                for file in files.into_iter() {
                    let file = file.read().await;
                    let asset: T = serde_yaml::from_slice(&file).unwrap();
                    loaded.push(asset);
                }
                tx.send(Some(loaded))
            }
        };
    });
    rx
}

pub fn saver<T: Serialize + 'static>(asset: T) {
    spawn(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .save_file()
            .await;
        match file {
            None => (),
            Some(file) => {
                // TODO: error handling
                let asset = serde_yaml::to_string(&asset).unwrap();
                file.write(asset.as_bytes()).await.unwrap();
            }
        }
    });
}

#[derive(RustEmbed)]
#[folder = "../prefabs/"]
pub struct WidgetPrefab;

pub struct AssetLoader<T> {
    #[cfg(target_arch = "wasm32")]
    storage: web_sys::Storage,
    assets: BTreeMap<Uuid, T>,
    channel: (mpsc::Sender<T>, mpsc::Receiver<T>),
}

impl<T> AssetLoader<T>
where
    T: Serialize + DeserializeOwned + Uuidentified + 'static,
{
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        let (storage, mut assets) = {
            let window = web_sys::window().ok_or("Could not get window")?;
            let storage = window
                .local_storage()
                .map_err(|e| format!("{e:?}"))?
                .ok_or("Could not get localStorage")?;
            let assets = storage
                .get_item("pcmg_widgets")
                .map_err(|e| format!("{e:?}"))?;
            let assets = if let Some(assets) = assets {
                serde_yaml::from_str(&assets).unwrap_or_default()
            } else {
                BTreeMap::default()
            };
            (storage, assets)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let mut assets = BTreeMap::default();

        for asset in WidgetPrefab::iter() {
            let asset = WidgetPrefab::get(&asset).unwrap();
            let asset: T = serde_yaml::from_slice(&asset.data)?;
            assets.insert(asset.uuid(), asset);
        }
        let this = Self {
            #[cfg(target_arch = "wasm32")]
            storage,
            assets,
            channel: mpsc::channel(10),
        };
        this.save()?;
        Ok(this)
    }

    pub fn insert(&mut self, asset: T) -> Result<(), serde_yaml::Error> {
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

    pub fn save(&self) -> Result<(), serde_yaml::Error> {
        #[cfg(target_arch = "wasm32")]
        let res = self
            .storage
            .set_item("pcm_widgets", &serde_yaml::to_string(&self.assets)?);
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = res {
            log::warn!("Failed to save an asset to LocalStorage because of: {e:?}");
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
