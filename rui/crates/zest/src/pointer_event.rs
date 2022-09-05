use druid_shell::MouseEvent;

use crate::geometry::Offset;

#[derive(Debug, Clone)]
pub enum PointerEvent {
    MouseUp(MouseEvent),
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
}

impl PointerEvent {
    pub fn position(&self) -> Offset {
        match self {
            PointerEvent::MouseUp(event) => event.pos,
            PointerEvent::MouseDown(event) => event.pos,
            PointerEvent::MouseMove(event) => event.pos,
        }
        .into()
    }
}
