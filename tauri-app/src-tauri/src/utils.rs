use crate::shimmy_server::structs::{Id, RequestType};

pub fn create_legit_svelte_id(id: &Id, source: RequestType) -> String {
    let origin = match source {
        RequestType::Client => "client",
        RequestType::Server => "server",
    };

    match id {
        Id::NumberId(number) => format!("{}-{}", origin, number),
        Id::StringId(s) => format!("{}-{}", origin, s),
    }
}
