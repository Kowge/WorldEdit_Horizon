// The client's half of the selection: what the server sent and the lines it
// is drawn with.
//
// The lines go into the engine's own list of lines drawn over everything, so
// the blocks do not hide the selection: the player sees it through the
// terrain, as the player asked.
//
// The lines are made once, when the server sends a selection: the outline of
// a round shape is a line for every block of it, and the frame of the world
// is drawn again and again.

use crate::game::we_geometry;
use std::sync::Mutex;

/// The selection the server sent.
static SELECTION: Mutex<Option<we_geometry::Selection>> = Mutex::new(None);

/// The lines of the selection, ready to be drawn.
static LINES: Mutex<Vec<we_geometry::Segment>> = Mutex::new(Vec::new());

/// The server sent a selection, or took it away.
pub fn set_selection(data: &[u8]) {
    let selection = we_geometry::parse(data);
    let lines = selection
        .as_ref()
        .map(we_geometry::segments)
        .unwrap_or_default();
    {
        let mut drawn = LINES.lock().unwrap_or_else(|e| e.into_inner());
        *drawn = lines;
    }
    let mut current = SELECTION.lock().unwrap_or_else(|e| e.into_inner());
    *current = selection;
}

/// The lines of the selection, over everything: the blocks do not hide them.
pub fn draw_on_top(dimension: &str, gizmos: &mut crate::gizmos::Gizmos) {
    let selection = SELECTION.lock().unwrap_or_else(|e| e.into_inner());
    let Some(selection) = selection.as_ref() else {
        return;
    };
    // The selection was made in another level: nothing to draw here.
    if selection.dimension != dimension {
        return;
    }
    let drawn = LINES.lock().unwrap_or_else(|e| e.into_inner());
    gizmos.on_top.extend(drawn.iter().map(|s| crate::gizmos::Line {
        start: s.start,
        end: s.end,
        color: s.color,
        width: s.width,
    }));
}
