use crate::shimmy_server::structs::Id;

pub fn create_legit_svelte_id(id: &Id) -> Id {
    match id {
        Id::NumberId(number) => Id::NumberId(number + 1),
        rest => rest.clone(),
    }
}
