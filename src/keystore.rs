use crate::commands::input_keystore_password;
use cocoon::MiniCocoon;
use directories::ProjectDirs;
use parity_scale_codec::{Decode, Encode};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
};
use subxt::ext::sp_core::crypto::ExposeSecret;

pub struct Keystore {
    cocoon: MiniCocoon,
    keymap: HashMap<String, String>,
}

impl Keystore {
    pub fn open() -> Self {
        let password = input_keystore_password().unwrap();

        let base_dir = ProjectDirs::from("org", "InvArch", "invarch-cli").unwrap();

        let project_dir = base_dir.data_dir();

        create_dir_all(project_dir).unwrap();

        let mut file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(project_dir.join("keystore.db"))
            .unwrap();

        let cocoon = MiniCocoon::from_password(password.expose_secret().as_bytes(), &[0; 32]);

        let encoded = cocoon.parse(&mut file).unwrap_or_default();
        let keymap = HashMap::<String, String>::from_iter(
            Vec::<(String, String)>::decode(&mut encoded.as_slice()).unwrap_or_default(),
        );

        Self { cocoon, keymap }
    }

    pub fn account_list(&self) -> Vec<String> {
        self.keymap.keys().cloned().collect()
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.keymap.get(&key).cloned()
    }

    pub fn insert_and_save(&mut self, key: String, value: String) -> Result<(), String> {
        self.keymap.insert(key, value);

        let encoded = self
            .keymap
            .clone()
            .into_iter()
            .collect::<Vec<(String, String)>>()
            .encode();

        let mut file = File::options()
            .create(true)
            .write(true)
            .open(
                ProjectDirs::from("org", "InvArch", "invarch-cli")
                    .ok_or(String::from("project dir couldn't be opened"))?
                    .data_dir()
                    .join("keystore.db"),
            )
            .map_err(|e| format!("{:?}", e))?;

        self.cocoon
            .dump(encoded, &mut file)
            .map_err(|e| format!("{:?}", e))
    }
}
