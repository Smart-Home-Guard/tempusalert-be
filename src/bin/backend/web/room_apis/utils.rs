use mongodb::bson::doc;
use tempusalert_be::backend_core::features::devices_status_feature::models::Device;

use crate::database_client::{init_database, MONGOC};

pub struct DeviceCheckExistResult {
    pub existing_ids: Vec<u32>,
    pub non_existing_ids: Vec<u32>,
}

pub async fn check_device_exist(
    mut device_ids: Vec<u32>,
    owner_name: &str,
) -> Option<DeviceCheckExistResult> {
    let mut existing_ids = Vec::new();

    let mongoc = MONGOC.get_or_init(init_database).await;

    let device_coll: mongodb::Collection<Device> =
        mongoc.default_database().unwrap().collection("devices");

    if let Ok(mut device_cursor) = device_coll
        .find(doc! { "owner_name": owner_name }, None)
        .await
    {
        while device_cursor.advance().await.unwrap() {
            match device_cursor.deserialize_current() {
                Ok(device) => {
                    let device_id = device.id;
                    if let Some(index) = device_ids.iter().position(|&id| id == device_id) {
                        existing_ids.push(device_id);
                        device_ids.remove(index);
                    }
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                    return None;
                }
            }
        }

        return Some(DeviceCheckExistResult {
            existing_ids,
            non_existing_ids: device_ids,
        });
    }

    return None;
}
