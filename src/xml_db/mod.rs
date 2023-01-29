pub mod dump;
pub mod parse;

/// In KDBX4, timestamps are stored as seconds, Base64 encoded, since 0001-01-01 00:00:00.
/// This function returns the epoch baseline used by KDBX for date serialization.
pub fn get_epoch_baseline() -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::parse_from_str("0001-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap()
}

#[cfg(test)]
mod tests {
    use secstr::SecStr;

    use crate::{
        config::{Compression, InnerCipherSuite, KdfSettings, OuterCipherSuite},
        meta::{BinaryAttachments, CustomIcons, Icon, MemoryProtection},
        parse::kdbx4,
        BinaryAttachment, CustomData, CustomDataItem, Database, Entry, Group, Meta, Node, Value,
    };

    fn make_key() -> Vec<Vec<u8>> {
        let mut password_bytes: Vec<u8> = vec![];
        let mut password: String = "".to_string();
        password_bytes.resize(40, 0);
        getrandom::getrandom(&mut password_bytes).unwrap();
        for random_char in password_bytes {
            password += &std::char::from_u32(random_char as u32).unwrap().to_string();
        }

        let key_elements = Database::get_key_elements(Some(&password), None).unwrap();
        key_elements
    }

    #[test]
    pub fn test_entry() {
        let mut root_group = Group::new("Root");
        let mut entry = Entry::new();
        let new_entry_uuid = entry.uuid.clone();

        entry.fields.insert(
            "Title".to_string(),
            crate::Value::Unprotected("ASDF".to_string()),
        );
        entry.fields.insert(
            "UserName".to_string(),
            crate::Value::Unprotected("ghj".to_string()),
        );
        entry.fields.insert(
            "Password".to_string(),
            crate::Value::Protected(std::str::from_utf8(b"klmno").unwrap().into()),
        );
        entry.tags.push("test".to_string());
        entry.tags.push("keepass-rs".to_string());
        entry.times.expires = true;

        root_group.children.push(Node::Entry(entry));

        let db = Database::new(
            OuterCipherSuite::AES256,
            Compression::GZip,
            InnerCipherSuite::Salsa20,
            KdfSettings::Argon2 {
                salt: vec![],
                iterations: 1000,
                memory: 65536,
                parallelism: 1,
                version: argon2::Version::Version13,
            },
            root_group,
            vec![],
        )
        .unwrap();

        let key_elements = make_key();

        let encrypted_db = kdbx4::dump(&db, &key_elements).unwrap();
        let decrypted_db = kdbx4::parse(&encrypted_db, &key_elements).unwrap();

        assert_eq!(decrypted_db.root.children.len(), 1);

        let decrypted_entry = match &decrypted_db.root.children[0] {
            Node::Entry(e) => e,
            Node::Group(_) => panic!("Was expecting an entry as the only child."),
        };

        assert_eq!(decrypted_entry.get_uuid(), new_entry_uuid);
        assert_eq!(decrypted_entry.get_title(), Some("ASDF"));
        assert_eq!(decrypted_entry.get_username(), Some("ghj"));
        assert_eq!(decrypted_entry.get("Password"), Some("klmno"));
        assert_eq!(
            decrypted_entry.tags,
            vec!["keepass-rs".to_string(), "test".to_string()]
        );
    }

    #[test]
    pub fn test_group() {
        let mut root_group = Group::new("Root");
        let mut entry = Entry::new();
        let new_entry_uuid = entry.uuid.clone();
        entry.fields.insert(
            "Title".to_string(),
            crate::Value::Unprotected("ASDF".to_string()),
        );

        root_group.children.push(Node::Entry(entry));

        let db = Database::new(
            OuterCipherSuite::AES256,
            Compression::GZip,
            InnerCipherSuite::Salsa20,
            KdfSettings::Argon2 {
                salt: vec![],
                iterations: 1000,
                memory: 65536,
                parallelism: 1,
                version: argon2::Version::Version13,
            },
            root_group,
            vec![],
        )
        .unwrap();

        let key_elements = make_key();

        let encrypted_db = kdbx4::dump(&db, &key_elements).unwrap();
        let decrypted_db = kdbx4::parse(&encrypted_db, &key_elements).unwrap();

        assert_eq!(decrypted_db.root.children.len(), 1);

        let decrypted_entry = match &decrypted_db.root.children[0] {
            Node::Entry(e) => e,
            Node::Group(_) => panic!("Was expecting an entry as the only child."),
        };

        assert_eq!(decrypted_entry.get_title(), Some("ASDF"));
        assert_eq!(decrypted_entry.get_uuid(), new_entry_uuid);

        let decrypted_root_group = &decrypted_db.root;
        assert_eq!(decrypted_root_group.name, "Root");
    }

    #[test]
    pub fn test_meta() {
        let mut db = Database::new(
            OuterCipherSuite::AES256,
            Compression::GZip,
            InnerCipherSuite::Salsa20,
            KdfSettings::Argon2 {
                salt: vec![],
                iterations: 1000,
                memory: 65536,
                parallelism: 1,
                version: argon2::Version::Version13,
            },
            Group::new("Root"),
            vec![],
        )
        .unwrap();

        let meta = Meta {
            generator: Some("test-generator".to_string()),
            database_name: Some("test-database-name".to_string()),
            database_name_changed: Some("2000-12-31T12:34:56".parse().unwrap()),
            database_description: Some("test-database-description".to_string()),
            database_description_changed: Some("2000-12-31T12:34:57".parse().unwrap()),
            default_username: Some("test-default-username".to_string()),
            default_username_changed: Some("2000-12-31T12:34:58".parse().unwrap()),
            maintenance_history_days: Some(123),
            color: Some("#C0FFEE".to_string()),
            master_key_changed: Some("2000-12-31T12:34:59".parse().unwrap()),
            master_key_change_rec: Some(-1),
            master_key_change_force: Some(42),
            memory_protection: Some(MemoryProtection {
                protect_title: true,
                protect_username: false,
                protect_password: true,
                protect_url: false,
                protect_notes: true,
            }),
            custom_icons: CustomIcons {
                icons: vec![Icon {
                    uuid: "a-fake-uuid".to_string(),
                    data: b"fake-data".to_vec(),
                }],
            },
            recyclebin_enabled: Some(true),
            recyclebin_uuid: Some("another-fake-uuid".to_string()),
            recyclebin_changed: Some("2000-12-31T12:35:00".parse().unwrap()),
            entry_templates_group: Some("even-more-fake-uuid".to_string()),
            entry_templates_group_changed: Some("2000-12-31T12:35:01".parse().unwrap()),
            last_selected_group: Some("so-many-fake-uuids".to_string()),
            last_top_visible_group: Some("hey-another-fake-uuid".to_string()),
            history_max_items: Some(456),
            history_max_size: Some(789),
            settings_changed: Some("2000-12-31T12:35:02".parse().unwrap()),
            binaries: BinaryAttachments {
                binaries: vec![
                    BinaryAttachment {
                        identifier: Some("1".to_string()),
                        flags: 0,
                        compressed: false,
                        content: b"i am binary data".to_vec(),
                    },
                    BinaryAttachment {
                        identifier: Some("2".to_string()),
                        flags: 0,
                        compressed: true,
                        content: b"i am compressed binary data".to_vec(),
                    },
                ],
            },
            custom_data: CustomData {
                items: vec![
                    CustomDataItem {
                        key: "custom-data-key".to_string(),
                        value: Some(Value::Unprotected("custom-data-value".to_string())),
                        last_modification_time: Some("2000-12-31T12:35:03".parse().unwrap()),
                    },
                    CustomDataItem {
                        key: "custom-data-key-without-value".to_string(),
                        value: None,
                        last_modification_time: None,
                    },
                    CustomDataItem {
                        key: "custom-data-protected-key".to_string(),
                        value: Some(Value::Protected(SecStr::new(b"custom-data-value".to_vec()))),
                        last_modification_time: Some("2000-12-31T12:35:03".parse().unwrap()),
                    },
                ],
            },
        };

        db.meta = meta.clone();

        let key_elements = make_key();

        let encrypted_db = kdbx4::dump(&db, &key_elements).unwrap();
        let decrypted_db = kdbx4::parse(&encrypted_db, &key_elements).unwrap();

        assert_eq!(decrypted_db.meta, meta);
    }
}