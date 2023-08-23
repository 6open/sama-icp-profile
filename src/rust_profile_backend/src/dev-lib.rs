use ic_cdk::{
    api::call::ManualReply,
    export::{
        candid::{CandidType, Deserialize},
        Principal,
    },
};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

type IdStore = BTreeMap<String, Principal>;
type ProfileStore = HashMap<Principal, Vec<Profile>>;
type ProfileStore = BTreeMap<Principal, HashMap<string, Vec<Profile>>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Profile {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

thread_local! {
    static PROFILE_STORE: RefCell<ProfileStore> = RefCell::default();
    static ID_STORE: RefCell<IdStore> = RefCell::default();
}

//TODO 直接返回principal id
#[query(name = "getSelf")]
fn get_self() -> Vec<Profile> {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        profile_store
            .borrow()
            .get(&principal_id)
            .cloned()
            .unwrap_or_default()
    })
}


#[query]
fn get(name: String) -> Vec<Profile> {
    ID_STORE.with(|id_store| {
        PROFILE_STORE.with(|profile_store| {
            let principal_id = id_store.borrow().get(&name).cloned();
            principal_id
                .and_then(|id| profile_store.borrow().get(&id).cloned())
                .unwrap_or_default()
        })
    })
}

#[update]
fn add(profile: Profile) {
    let principal_id = ic_cdk::api::caller();
    ID_STORE.with(|id_store| {
        id_store
            .borrow_mut()
            .insert(profile.name.clone(), principal_id);
    });
    PROFILE_STORE.with(|profile_store| {
        profile_store
            .borrow_mut()
            .entry(principal_id)
            .or_insert_with(Vec::new)
            .push(profile);
    });
}

#[update]
fn update(profile: Profile) {
    let principal_id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| {
        let mut profile_store_borrow = profile_store.borrow_mut();
        if let Some(profiles) = profile_store_borrow.get_mut(&principal_id) {
            profiles.push(profile);
        } else {
            let mut new_profiles = Vec::new();
            new_profiles.push(profile);
            profile_store_borrow.insert(principal_id, new_profiles);
        }
    });
}


#[update]
fn remove(name: String) {
    ID_STORE.with(|id_store| {
        let principal_id = ic_cdk::api::caller();
        PROFILE_STORE.with(|profile_store| {
            if let Some(profiles) = profile_store.borrow_mut().get_mut(&principal_id) {
                profiles.retain(|p| p.name != name);
            }
        });
    });
}

#[query(manual_reply = true)]
fn search(text: String) -> ManualReply<Option<Vec<Profile>>> {
    let text = text.to_lowercase();
    PROFILE_STORE.with(|profile_store| {
        for (_, profiles) in profile_store.borrow().iter() {
            for profile in profiles {
                if profile.name.to_lowercase().contains(&text)
                    || profile.description.to_lowercase().contains(&text)
                {
                    return ManualReply::one(Some(profiles.clone()));
                }

                for keyword in profile.keywords.iter() {
                    if keyword.to_lowercase() == text {
                        return ManualReply::one(Some(profiles.clone()));
                    }
                }
            }
        }
        ManualReply::one(None::<Vec<Profile>>)
    })
}

#[query(manual_reply = true)]
fn iterate() -> ManualReply<Vec<Vec<Profile>>> {
    PROFILE_STORE.with(|profile_store| {
        let profiles: Vec<Vec<Profile>> = profile_store
            .borrow()
            .values()
            .cloned()
            .collect();
        ManualReply::one(profiles)
    })
}
