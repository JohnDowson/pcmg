use base64::{
    alphabet::URL_SAFE,
    engine::{
        GeneralPurpose,
        GeneralPurposeConfig,
    },
    Engine,
};
use futures::channel::oneshot::{
    channel,
    Receiver,
};
use serde::{
    de::DeserializeOwned,
    Serialize,
};

#[cfg(not(target_arch = "wasm32"))]
use futures::executor::block_on as spawn;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local as spawn;

#[cfg(target_arch = "wasm32")]
pub fn load_from_url<T: DeserializeOwned>(what: &str) -> Option<T> {
    use std::collections::BTreeMap;

    use url::Url;
    let window = web_sys::window()?;
    let url = window.location().href().ok()?;
    let url = Url::parse(&url).ok()?;
    let query = url.query_pairs().collect::<BTreeMap<_, _>>();
    let basestr = &**query.get(what)?;
    load_from_base64(basestr)
}

#[cfg(target_arch = "wasm32")]
pub fn save_to_url<T: Serialize>(what: &str, asset: T) -> Option<()> {
    use url::Url;
    let window = web_sys::window()?;
    let url = window.location().href().ok()?;
    let mut url = Url::parse(&url).ok()?;
    let basestr = save_to_base64(asset)?;
    url.query_pairs_mut().clear().append_pair(what, &basestr);
    window.location().set_href(url.as_str()).ok()?;
    Some(())
}

pub fn load_from_base64<T: DeserializeOwned>(basestr: &str) -> Option<T> {
    let decoder = GeneralPurpose::new(&URL_SAFE, GeneralPurposeConfig::new());
    let decoded = decoder.decode(basestr).ok()?;
    minicbor_ser::from_slice(&decoded).ok()
}

pub fn save_to_base64<T: Serialize>(asset: T) -> Option<String> {
    let serialized = minicbor_ser::to_vec(&asset).ok()?;
    let encoder = GeneralPurpose::new(&URL_SAFE, GeneralPurposeConfig::new());
    Some(encoder.encode(serialized))
}

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
