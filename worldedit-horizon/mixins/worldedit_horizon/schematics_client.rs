// The client's half of the schematics: the files are the player's own, so
// the client keeps them, in the mod's own folder of the instance. The server
// asks it to write, read, list and delete them.
//
// The files are the player's own, so the client does for a server only what
// the player asked for: the player's own command is what allows a request,
// and a new connection allows nothing at all.

use crate::mod_api::Host;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// A file the client is sending to the server, in parts.
struct Outgoing {
    parts: Vec<Vec<u8>>,
    next: usize,
}

/// The files the client sends and the ones it puts together.
static OUT: Mutex<Option<HashMap<String, Outgoing>>> = Mutex::new(None);
static IN: Mutex<Option<HashMap<String, crate::game::we_core::Parts>>> = Mutex::new(None);
/// What the player asked for, and what the client may therefore do for the
/// server.
static ALLOWED: Mutex<Option<crate::game::we_core::Allowed>> = Mutex::new(None);

/// The mod's own folder: the schematics live in it, and the button of the
/// mod's page in Options > Mods opens it.
pub fn dir(game: &crate::game::Game<'_>) -> PathBuf {
    game.native_data_dir(crate::game::we_mods_ui::ID)
}

/// The player's own command, as the client sees it before the server does:
/// it is what allows the server to ask for the player's files. The command
/// is not claimed: the server runs it.
pub fn player_command(command: &str) {
    let command = command.trim_start_matches('/');
    let mut words = command.split_whitespace();
    if words.next() != Some("we") || words.next() != Some("schematic") {
        return;
    }
    let Some(op) = words.next() else {
        return;
    };
    // The protocol's own words: a save is a write, a load is a read.
    let op = match op {
        "save" => "write",
        "load" => "read",
        "list" => "list",
        "delete" => "delete",
        _ => return,
    };
    let name = if op == "list" {
        String::new()
    } else {
        let Ok(name) = crate::game::we_core::schematic_name(words.next().unwrap_or("")) else {
            return;
        };
        name
    };
    let mut guard = ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(crate::game::we_core::Allowed::default)
        .ask(op, &name);
}

/// The client made or dropped its level: another server, another dimension
/// or a disconnect. What the last server asked for is not allowed any more,
/// and what it sent does not wait for an answer.
pub fn level_changed() {
    {
        let mut guard = ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(crate::game::we_core::Allowed::default());
    }
    {
        let mut guard = IN.lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
    {
        let mut guard = OUT.lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
}

/// Whether the server may ask the client for this.
fn allowed(op: &str, name: &str) -> bool {
    let mut guard = ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(crate::game::we_core::Allowed::default)
        .allows(op, name)
}

/// Whether the server may ask the client for this, and the allowance is
/// spent.
fn take(op: &str, name: &str) -> bool {
    let mut guard = ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .get_or_insert_with(crate::game::we_core::Allowed::default)
        .take(op, name)
}

/// The server asked for something the player did not ask for.
fn refuse(game: &mut crate::game::Game<'_>, op: &str, name: &str) {
    // The name in a reply is the player's own: a name the server made up
    // is not put in a header.
    let name = crate::game::we_core::schematic_name(name).unwrap_or_default();
    reply(game, &format!("error {op} {name} refused"));
}

/// What the server asks of the client.
pub fn from_server(game: &mut crate::game::Game<'_>, data: &[u8]) {
    let Some((header, payload)) = crate::game::we_core::split_message(data) else {
        return;
    };
    let mut words = header.split_whitespace();
    match words.next() {
        Some("write") => {
            let name = words.next().unwrap_or("").to_owned();
            let index = words.next().and_then(|n| n.parse().ok()).unwrap_or(0);
            let count = words.next().and_then(|n| n.parse().ok()).unwrap_or(0);
            // Only what the player asked for, and only a part of a size a
            // file may have: a header of a hostile size is refused here,
            // before anything is allocated for it.
            if !allowed("write", &name)
                || !crate::game::we_core::part_ok(index, count, payload.len())
            {
                refuse(game, "write", &name);
                return;
            }
            write_part(game, &name, index, count, payload);
        }
        Some("read") => {
            let name = words.next().unwrap_or("").to_owned();
            if !take("read", &name) {
                refuse(game, "read", &name);
                return;
            }
            read_file(game, &name);
        }
        Some("list") => {
            if !take("list", "") {
                refuse(game, "list", "");
                return;
            }
            list_files(game);
        }
        Some("delete") => {
            let name = words.next().unwrap_or("").to_owned();
            if !take("delete", &name) {
                refuse(game, "delete", &name);
                return;
            }
            delete_file(game, &name);
        }
        Some("more") => {
            let name = words.next().unwrap_or("").to_owned();
            send_next_parts(game, &name);
        }
        _ => {}
    }
}

/// The client writes the file: its parts arrive one by one.
fn write_part(
    game: &mut crate::game::Game<'_>,
    name: &str,
    index: usize,
    count: usize,
    bytes: &[u8],
) {
    // The file goes into the player's own folder, under its own name.
    let Some(safe) = crate::game::we_core::schematic_name(name).ok() else {
        refuse(game, "write", name);
        return;
    };
    let (whole, refused) = {
        let mut guard = IN.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        // Only one file is written at a time: the file of the last command
        // is the only one the server may write, so the file before it goes.
        if !map.is_empty() && !map.contains_key(&safe) {
            map.clear();
        }
        let entry = map
            .entry(safe.clone())
            .or_insert_with(|| crate::game::we_core::Parts::new(count));
        match entry.put(index, count, bytes) {
            Ok(whole) => (whole, false),
            Err(_) => {
                // The parts do not fit what the player asked for: the file
                // goes, and the server is told that it is refused.
                map.remove(&safe);
                (false, true)
            }
        }
    };
    if refused {
        refuse(game, "write", &safe);
        return;
    }
    if !whole {
        // The rest of the file is on its way.
        reply(game, &format!("more {safe}"));
        return;
    }
    let file = {
        let mut guard = IN.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        map.remove(&safe).and_then(|mut entry| entry.take())
    };
    let Some(data) = file else {
        return;
    };
    let folder = dir(game);
    let _ = fs::create_dir_all(&folder);
    let path = folder.join(crate::game::we_core::schematic_file(&safe));
    if fs::write(&path, &data).is_err() {
        reply(game, &format!("error write {safe} io"));
        return;
    }
    // The file is there: the server may not write it again.
    {
        let mut guard = ALLOWED.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(allowed) = guard.as_mut() {
            allowed.clear("write", &safe);
        }
    }
    let blocks = crate::game::we_core::schematic_info(&data).map_or(0, |(_, count)| count);
    reply(game, &format!("wrote {safe} {blocks}"));
}

/// The client sends the file to the server, in parts.
fn read_file(game: &mut crate::game::Game<'_>, name: &str) {
    // The name in a reply is the player's own: a name the server made up
    // is not put in a header.
    let safe = crate::game::we_core::schematic_name(name).unwrap_or_default();
    if safe.is_empty() {
        reply(game, "error read  not found");
        return;
    }
    let path = dir(game).join(crate::game::we_core::schematic_file(&safe));
    let Ok(data) = fs::read(&path) else {
        reply(game, &format!("error read {safe} not found"));
        return;
    };
    // A file of the player's own that is bigger than a schematic may be is
    // not one.
    if data.len() > crate::game::we_core::MAX_FILE_BYTES {
        reply(game, &format!("error read {safe} too big"));
        return;
    }
    let parts = crate::game::we_core::parts(&data);
    {
        let mut guard = OUT.lock().unwrap_or_else(|e| e.into_inner());
        // Only one file is sent at a time: the file before it goes.
        let map = guard.get_or_insert_with(HashMap::new);
        map.clear();
        map.insert(safe.clone(), Outgoing { parts, next: 0 });
    }
    send_next_parts(game, &safe);
}

/// The next parts of the file the client sends.
fn send_next_parts(game: &mut crate::game::Game<'_>, name: &str) {
    let (count, batch) = {
        let mut guard = OUT.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        let Some(out) = map.get_mut(name) else {
            return;
        };
        let count = out.parts.len();
        let mut batch = Vec::new();
        while batch.len() < crate::game::we_core::BATCH && out.next < out.parts.len() {
            batch.push((out.next, out.parts[out.next].clone()));
            out.next += 1;
        }
        (count, batch)
    };
    for (index, bytes) in batch {
        let header = format!("read {name} {index} {count}");
        reply_bytes(game, &header, &bytes);
    }
}

/// The client tells the server which files it has.
fn list_files(game: &mut crate::game::Game<'_>) {
    let folder = dir(game);
    let mut lines: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str())
                != Some(crate::game::we_core::SCHEMATIC_EXTENSION)
            {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let blocks = fs::read(&path)
                .ok()
                .and_then(|data| crate::game::we_core::schematic_info(&data))
                .map_or(0, |(_, count)| count);
            lines.push(format!("{stem} {blocks}"));
        }
    }
    lines.sort();
    let count = lines.len();
    let payload = lines.join("\n");
    reply_bytes(game, &format!("listed {count}"), payload.as_bytes());
}

/// The client removes the file.
fn delete_file(game: &mut crate::game::Game<'_>, name: &str) {
    // The name in a reply is the player's own: a name the server made up
    // is not put in a header.
    let safe = crate::game::we_core::schematic_name(name).unwrap_or_default();
    if safe.is_empty() {
        reply(game, "error delete  not found");
        return;
    }
    let path = dir(game).join(crate::game::we_core::schematic_file(&safe));
    match fs::remove_file(&path) {
        Ok(()) => reply(game, &format!("deleted {safe}")),
        Err(_) => reply(game, &format!("error delete {safe} not found")),
    }
}

/// A message to the server.
fn reply(game: &mut crate::game::Game<'_>, header: &str) {
    reply_bytes(game, header, &[]);
}

/// A message to the server, with bytes.
fn reply_bytes(game: &mut crate::game::Game<'_>, header: &str, payload: &[u8]) {
    let data = crate::game::we_core::message(header, payload);
    let host: &mut dyn Host = game;
    let _ = host.net_send_to_server(crate::game::we_core::SCHEMATIC_CHANNEL, &data);
}
