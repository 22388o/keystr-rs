use nostr_sdk::prelude::{FromPkStr, FromSkStr, Keys, ToBech32};

#[derive(PartialEq)]
pub enum KeysSetState {
    NotSet,
    PublicOnly,
    SecretAndPublic,
}

// Model for KeyStore part
pub struct Keystore {
    pub set_level: KeysSetState,
    keys: Keys,
    // Input for public key import
    pub public_key_input: String,
    // Input for secret key import
    pub secret_key_input: String,
}

impl Keystore {
    pub fn new() -> Self {
        Keystore {
            set_level: KeysSetState::NotSet,
            keys: Keys::generate(), // placeholder value initially
            public_key_input: String::new(),
            secret_key_input: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.keys = Keys::generate();
        self.set_level = KeysSetState::NotSet;
    }

    /// Generate new random keys
    pub fn generate(&mut self) {
        self.keys = Keys::generate();
        self.set_level = KeysSetState::SecretAndPublic;
    }

    /// Import public key only, in 'npub' bech32 or hex format. Signing will not be possible.
    pub fn import_public_key(&mut self, public_key_str: &str) -> Result<(), String> {
        match Keys::from_pk_str(public_key_str) {
            Err(e) => {
                self.clear();
                Err(e.to_string())
            }
            Ok(k) => {
                self.clear();
                self.keys = k;
                self.set_level = KeysSetState::PublicOnly;
                Ok(())
            }
        }
    }

    /// Warning: Security-sensitive method!
    /// Import secret key, in 'nsec' bech32 or hex format (pubkey is derived from it)
    pub fn import_secret_key(&mut self, secret_key_str: &str) -> Result<(), String> {
        match Keys::from_sk_str(secret_key_str) {
            Err(e) => {
                self.clear();
                Err(e.to_string())
            }
            Ok(k) => {
                self.clear();
                self.keys = k;
                self.set_level = KeysSetState::SecretAndPublic;
                Ok(())
            }
        }
    }

    pub fn is_public_key_set(&self) -> bool {
        self.set_level != KeysSetState::NotSet
    }

    pub fn is_secret_key_set(&self) -> bool {
        self.set_level == KeysSetState::SecretAndPublic
    }

    pub fn get_keys(&self) -> Result<Keys, String> {
        if !self.is_secret_key_set() {
            return Err("(not set)".to_string());
        }
        Ok(self.keys.clone())
    }

    pub fn get_npub(&self) -> String {
        if !self.is_public_key_set() {
            "(not set)".to_string()
        } else {
            match self.keys.public_key().to_bech32() {
                Err(_) => "(conversion error)".to_string(),
                Ok(s) => s,
            }
        }
    }

    /// Warning: Security-sensitive method!
    pub fn get_nsec(&self) -> String {
        if !self.is_secret_key_set() {
            "(not set)".to_string()
        } else {
            match self.keys.secret_key() {
                Err(_) => "(no secret key)".to_string(),
                Ok(key) => match key.to_bech32() {
                    Err(_) => "(conversion error)".to_string(),
                    Ok(s) => s,
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let k = Keystore::new();
        assert_eq!(k.is_public_key_set(), false);
        assert_eq!(k.is_secret_key_set(), false);
        assert_eq!(k.get_npub(), "(not set)");
        assert_eq!(k.get_nsec(), "(not set)");
        assert!(k.get_keys().is_err());
    }

    #[test]
    fn test_generate() {
        let mut k = Keystore::new();
        k.generate();
        assert!(k.is_public_key_set());
        assert!(k.is_secret_key_set());
        assert!(k.get_npub().len() > 60);
        assert!(k.get_nsec().len() > 60);
        assert!(k.get_keys().is_ok());
        assert_eq!(
            k.get_keys().unwrap().public_key().to_bech32().unwrap(),
            k.get_npub()
        );
        assert_eq!(
            k.get_keys()
                .unwrap()
                .secret_key()
                .unwrap()
                .to_bech32()
                .unwrap(),
            k.get_nsec()
        );
    }

    #[test]
    fn test_import_secret_key() {
        let mut k = Keystore::new();
        let _res = k
            .import_secret_key("nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae")
            .unwrap();
        assert!(k.is_public_key_set());
        assert!(k.is_secret_key_set());
        assert_eq!(
            k.get_nsec(),
            "nsec1ktekw0hr5evjs0n9nyyquz4sue568snypy2rwk5mpv6hl2hq3vtsk0kpae"
        );
        assert_eq!(
            k.get_npub(),
            "npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4"
        );

        let res2 = k.import_secret_key("__NOT_A_VALID_KEY__");
        assert!(res2.is_err());
        assert_eq!(k.is_public_key_set(), false);
        assert_eq!(k.is_secret_key_set(), false);
    }

    #[test]
    fn test_import_public_key() {
        let mut k = Keystore::new();
        let _res = k
            .import_public_key("npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4")
            .unwrap();
        assert!(k.is_public_key_set());
        assert_eq!(k.is_secret_key_set(), false);
        assert_eq!(
            k.get_npub(),
            "npub1rfze4zn25ezp6jqt5ejlhrajrfx0az72ed7cwvq0spr22k9rlnjq93lmd4"
        );

        let res2 = k.import_public_key("__NOT_A_VALID_KEY__");
        assert!(res2.is_err());
        assert_eq!(k.is_public_key_set(), false);
        assert_eq!(k.is_secret_key_set(), false);
    }
}
