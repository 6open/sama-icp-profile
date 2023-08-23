use ic_cdk::{
    api::call::ManualReply,
    export::{
        candid::{CandidType, Deserialize},
        Principal,
    },
};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use lazy_static::lazy_static;

// mod declarations;
// use declarations::profile_rs::{profile_rs, Profile};

type ProfileStore = HashMap<Principal, HashMap<String, String>>;
type Users = BTreeSet<Principal>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct MyData {
    pub key : String,
    pub value : String,
}

static mut GLOBAL_CALLER_ID: Option<String> = None;
lazy_static! {
    static ref MY_GLOBAL_BOOL: bool = true;
}

thread_local! {
    static USERS: RefCell<Users> = RefCell::default();
    static PROFILE_STORE: RefCell<ProfileStore> = RefCell::default();
}

#[init]
fn init() {
    USERS.with(|users| users.borrow_mut().insert(ic_cdk::api::caller()));
}

fn is_user() -> Result<(), String> {
    if USERS.with(|users| users.borrow().contains(&ic_cdk::api::caller())) {
        Ok(())
    } else {
        if *MY_GLOBAL_BOOL {
            Ok(())
        } else {
            Err("Store can only be set by the owner of the asset canister.".to_string())
        }
    }
}

#[update(guard = "is_user")]
fn get_self() -> Option<String> {
    // 获取当前调用者的 Principal
    let caller_principal = ic_cdk::api::caller();

    // 将 Principal 转换为 String，并返回
    Some(caller_principal.to_string())
}

#[update(guard = "is_user")]
fn add_user(principal: Principal) {
    USERS.with(|users| users.borrow_mut().insert(principal));
}

// 设置全局身份
fn set_global_caller_id(caller_id: &str) {
    unsafe {
        GLOBAL_CALLER_ID = Some(caller_id.to_string());
    }
}

#[query]
fn get(name: String) -> Option<String> {
    PROFILE_STORE.with(|profile_store| {
        let principal_id = ic_cdk::api::caller();
        if let Some(profiles_map) = profile_store.borrow().get(&principal_id) {
            if let Some(profile_entry) = profiles_map.get(&name) {
                return Some(profile_entry.clone());
            }
        }
        None // 如果找不到相关数据则返回 None
    })
}

#[update(guard = "is_user")]
fn add(key: String, value: String) -> Option<String> {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        let mut profile_store_borrow = profile_store.borrow_mut();
        let profiles_map = profile_store_borrow.entry(principal_id).or_insert_with(HashMap::new);

        // 检查是否已经存在该数据
        if profiles_map.contains_key(&key) {
            return Some("Data already exists.".to_string()); // 数据已存在，返回特定数据
        }

        // 数据不存在，执行插入操作
        profiles_map.insert(key, value);
        return Some("Ok.".to_string()); // 数据已存在，返回特定数据
    })
}

#[update(guard = "is_user")]
fn update(key: String, value: String) -> Option<String> {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        let mut profile_store_borrow = profile_store.borrow_mut();
        if let Some(profiles_map) = profile_store_borrow.get_mut(&principal_id) {
            if let Some(existing_value) = profiles_map.get_mut(&key) {
                *existing_value = value; 
                Some("Ok".to_string()) // 覆盖现有元素
            } else {
                Some("No matching element".to_string()) // 未找到匹配的元素提示
            }
        } else {
            Some("No matching element".to_string()) // 未找到匹配的元素提示
        }
    })
}

#[update(guard = "is_user")]
fn remove(key: String) -> Option<String> {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        let mut profile_store_borrow = profile_store.borrow_mut();
        if let Some(profiles_map) = profile_store_borrow.get_mut(&principal_id) {
            if let Some(existing_value) = profiles_map.remove(&key) {
                Some("Ok".to_string())
            } else {
                Some("No matching element".to_string()) // 未找到匹配的元素提示
            }
        } else {
            Some("No matching element".to_string()) // 未找到匹配的元素提示
        }
    })
}

#[query(manual_reply = true)]
fn get_all() -> ManualReply<Option<Vec<MyData>>> {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        if let Some(profiles_map) = profile_store.borrow().get(&principal_id) {
            let profiles: Vec<MyData> = profiles_map
                .iter()
                .map(|(key, value)| MyData { key: key.clone(), value: value.clone() })
                .collect();
            ManualReply::one(Some(profiles))
        } else {
            ManualReply::one(None::<Vec<MyData>>)
        }
    })
}

// 使用 `PROFILE_STORE` 查询指定 ID 的数据，并返回 ManualReply<Option<Vec<MyData>>>
#[query(manual_reply = true)]
fn get_by_id(id: String) -> ManualReply<Option<Vec<MyData>>> {
    let id_principal = Principal::from_text(&id).expect("Failed to parse the id as Principal.");
    PROFILE_STORE.with(|profile_store| {
        if let Some(profiles_map) = profile_store.borrow().get(&id_principal) {
            let profiles: Vec<MyData> = profiles_map
                .iter()
                .map(|(key, value)| MyData { key: key.clone(), value: value.clone() })
                .collect();
            ManualReply::one(Some(profiles))
        } else {
            ManualReply::one(None::<Vec<MyData>>)
        }
    })
}
//ManualReply::one(None::<Vec<Profile>>)