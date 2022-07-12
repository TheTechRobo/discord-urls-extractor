use tinyjson::JsonValue;

use std::collections::HashMap;

pub fn gateway_parse(json: JsonValue, server_id: String) -> Vec<String> {
    let mut finish = Vec::new();
    let json_hashmap: HashMap<String, JsonValue> = json.clone().try_into().unwrap();
    if !json_hashmap.contains_key("t") || !json_hashmap.contains_key("s") {
        return finish;
    }
    if json["t"].is_null() {
        return finish;
    }
    let t: String = json["t"].clone().try_into().unwrap();
    let s: bool   = json["s"].is_null();
    if (t != "READY") || s { // makes sure this is the correct websocket
        return finish;
    }
    let actual_data: HashMap<String, JsonValue> = json["d"].clone().try_into().unwrap();
    let v: f64 = actual_data["v"].clone().try_into().unwrap();
    if v != 9.0 {
        panic!("Unsupported API version `{v}'");
    }
    let guilds: Vec<JsonValue> = actual_data["guilds"].clone().try_into().unwrap();
    for raw_guild in guilds {
        let guild: HashMap<String, JsonValue> = raw_guild.try_into().unwrap();
        let guild_id: String = guild["id"].clone().try_into().unwrap();
        if guild_id != server_id {
            eprintln!("Found unrelated guild {guild_id}");
            continue;
        }
        if !guild["icon"].is_null() {
            let icon: String = guild["icon"].clone().try_into().unwrap();
            finish.push(format!("https://cdn.discordapp.com/icons/{}/{}.webp?size=4096&quality=lossless", guild_id, icon));
            finish.push(format!("https://cdn.discordapp.com/icons/{}/{}.gif?size=4096&quality=lossless", guild_id, icon));
        }
        let roles: Vec<JsonValue> = guild["roles"].clone().try_into().unwrap();
        for raw_role in roles {
            let role: HashMap<String, JsonValue> = raw_role.try_into().unwrap();
            if role.contains_key("icon") && !role["icon"].is_null() {
                let role_id: String =role["id"].clone().try_into().unwrap();
                let role_icon: String =role["icon"].clone().try_into().unwrap();
                finish.push(format!("https://cdn.discordapp.com/role-icons/{}/{}.webp?size=4096&quality=lossless", role_id, role_icon));
            }
        }
        let emojis: Vec<JsonValue> =guild["emojis"].clone().try_into().unwrap();
        for raw_emoji in emojis {
            let emoji:HashMap<String,JsonValue> = raw_emoji.try_into().unwrap();
            // todo: extract  optional user object from here?
            let id: String = emoji["id"].clone().try_into().unwrap();
            let mut animated: bool = false;
            if emoji.contains_key("animated") {
                animated = emoji["animated"].clone().try_into().unwrap();
            }
            if animated {
                finish.push(format!("https://cdn.discordapp.com/emojis/{}.gif?size=4096&quality=lossless", id));
            }
            else {
                finish.push(format!("https://cdn.discordapp.com/emojis/{}.webp?size=4096&quality=lossless", id));
            }
        }
    }
    finish
}
