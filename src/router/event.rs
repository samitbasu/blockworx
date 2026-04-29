#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventSense {
    Enter,
    Scan,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Event<T, C> {
    time: T,
    sense: EventSense,
    payload: C,
}

impl<T: Copy, C: Copy> Event<T, C> {
    pub fn sense(&self) -> EventSense {
        self.sense
    }
    pub fn t(&self) -> T {
        self.time
    }
    pub fn cost(&self) -> C {
        self.payload
    }
    pub fn count(&self) -> i32 {
        match self.sense {
            EventSense::Enter => 1,
            EventSense::Exit => -1,
            EventSense::Scan => 0,
        }
    }
    pub fn is_enter(&self) -> bool {
        matches!(self.sense, EventSense::Enter)
    }
    pub fn enter(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Enter,
            payload,
        }
    }
    pub fn exit(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Exit,
            payload,
        }
    }
    pub fn scan(time: T, payload: C) -> Self {
        Event {
            time,
            sense: EventSense::Scan,
            payload,
        }
    }
}
