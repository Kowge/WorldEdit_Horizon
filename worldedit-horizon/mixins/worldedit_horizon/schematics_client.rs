// The client's half of the schematics: the files are the player's own, so
// the client keeps them, in the mod's own folder of the instance. The server
// asks it to write, read, list and delete them.
//
// A file is bigger than one message (the game takes 30 000 bytes), so it
// travels in parts: the sender sends them, the receiver asks for the rest
// with "more".

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

/// The parts of a file the server sends, put together.
struct Incoming {
    count: usize,
    parts: Vec<Option<Vec<u8>>>,
    got: usize,
}

/// The files the client sends and the ones it puts together.
static OUT: Mutex<Option<HashMap<String, Outgoing>>> = Mutex::new(None);
static IN: Mutex<Option<HashMap<String, Incoming>>> = Mutex::new(None);

/// The mod's own folder: the schematics live in it, and the button of the
/// mod's page in Options > Mods opens it.
pub fn dir(game: &crate::game::Game<'_>) -> PathBuf {
    game.native_data_dir(crate::game::we_mods_ui::ID)
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
            write_part(game, &name, index, count, payload);
        }
        Some("read") => {
            let name = words.next().unwrap_or("").to_owned();
            read_file(game, &name);
        }
        Some("list") => list_files(game),
        Some("delete") => {
            let name = words.next().unwrap_or("").to_owned();
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
    let whole = {
        let mut guard = IN.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        let entry = map.entry(name.to_owned()).or_insert_with(|| Incoming {
            count,
            parts: Vec::new(),
            got: 0,
        });
        entry.count = count;
        if entry.parts.len() <= index {
            entry.parts.resize(index + 1, None);
        }
        if entry.parts[index].is_none() {
            entry.parts[index] = Some(bytes.to_vec());
            entry.got += 1;
        }
        entry.got >= count
    };
    if !whole {
        // The rest of the file is on its way.
        reply(game, &format!("more {name}"));
        return;
    }
    let file = {
        let mut guard = IN.lock().unwrap_or_else(|e| e.into_inner());
        let map = guard.get_or_insert_with(HashMap::new);
        map.remove(name).map(|entry| {
            let mut out = Vec::new();
            for part in entry.parts.into_iter().flatten() {
                out.extend_from_slice(&part);
            }
            out
        })
    };
    let Some(data) = file else {
        return;
    };
    let folder = dir(game);
    let _ = fs::create_dir_all(&folder);
    let path = folder.join(crate::game::we_core::schematic_file(name));
    if fs::write(&path, &data).is_err() {
        reply(game, &format!("error write {name} io"));
        return;
    }
    let blocks = crate::game::we_core::schematic_info(&data).map_or(0, |(_, count)| count);
    reply(game, &format!("wrote {name} {blocks}"));
}

/// The client sends the file to the server, in parts.
fn read_file(game: &mut crate::game::Game<'_>, name: &str) {
    let Some(safe) = crate::game::we_core::schematic_name(name).ok() else {
        reply(game, &format!("error read {name} not found"));
        return;
    };
    let path = dir(game).join(crate::game::we_core::schematic_file(&safe));
    let Ok(data) = fs::read(&path) else {
        reply(game, &format!("error read {safe} not found"));
        return;
    };
    let parts = crate::game::we_core::parts(&data);
    {
        let mut guard = OUT.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .get_or_insert_with(HashMap::new)
            .insert(safe.clone(), Outgoing { parts, next: 0 });
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
    let Some(safe) = crate::game::we_core::schematic_name(name).ok() else {
        reply(game, &format!("error delete {name} not found"));
        return;
    };
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
