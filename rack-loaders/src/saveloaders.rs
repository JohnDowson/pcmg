use base64::{
    alphabet::URL_SAFE,
    engine::{
        GeneralPurpose,
        GeneralPurposeConfig,
    },
    Engine,
};
use futures::channel::mpsc;
use lz4_flex::{
    compress_prepend_size,
    decompress_size_prepended,
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
    let decompressed = decompress_size_prepended(&decoded).ok()?;
    minicbor_ser::from_slice(&decompressed).ok()
}

pub fn save_to_base64<T: Serialize>(asset: T) -> Option<String> {
    let serialized = minicbor_ser::to_vec(&asset).ok()?;
    let compressed = compress_prepend_size(&serialized);
    let encoder = GeneralPurpose::new(&URL_SAFE, GeneralPurposeConfig::new());
    Some(encoder.encode(compressed))
}

pub fn loader<T: DeserializeOwned + 'static>() -> mpsc::Receiver<Option<T>> {
    let (mut tx, rx) = mpsc::channel(1);

    spawn(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .pick_file()
            .await;
        _ = match file {
            None => tx.try_send(None),
            Some(file) => {
                // TODO: error handling
                let file = file.read().await;
                let asset: T = serde_yaml::from_slice(&file).unwrap();
                tx.try_send(Some(asset))
            }
        };
    });
    rx
}

pub fn loader_many<T: DeserializeOwned + 'static>() -> mpsc::Receiver<Option<Vec<T>>> {
    let (mut tx, rx) = mpsc::channel(1);

    spawn(async move {
        let file = rfd::AsyncFileDialog::new()
            .set_directory(".")
            .pick_files()
            .await;
        _ = match file {
            None => tx.try_send(None),
            Some(files) => {
                // TODO: error handling
                let mut loaded = Vec::with_capacity(files.len());
                for file in files.into_iter() {
                    let file = file.read().await;
                    let asset: T = serde_yaml::from_slice(&file).unwrap();
                    loaded.push(asset);
                }
                tx.try_send(Some(loaded))
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
