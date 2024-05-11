use geng::{prelude::*, Touch};

pub struct TouchController {
    time: f64,
    touch: Option<ActiveTouch>,
}

#[derive(Debug, Clone)]
struct ActiveTouch {
    start: vec2<f64>,
    start_time: f64,
    current: vec2<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum TouchAction {
    ShortTap { position: vec2<f64> },
    Move { position: vec2<f64> },
}

impl TouchController {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            touch: None,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
    }

    pub fn handle_event(&mut self, event: &geng::Event) -> Option<TouchAction> {
        match event {
            geng::Event::TouchStart(touch) => {
                self.touch_start(*touch);
                Some(TouchAction::Move {
                    position: touch.position,
                })
            }
            geng::Event::TouchMove(touch) => {
                if self.touch.is_none() {
                    self.touch_start(*touch);
                }
                self.touch.as_mut().unwrap().current = touch.position;
                Some(TouchAction::Move {
                    position: touch.position,
                })
            }
            geng::Event::TouchEnd(touch) => self.touch_end(*touch),
            _ => None,
        }
    }

    fn touch_start(&mut self, touch: Touch) {
        self.touch = Some(ActiveTouch {
            start: touch.position,
            start_time: self.time,
            current: touch.position,
        });
    }

    fn touch_end(&mut self, touch: Touch) -> Option<TouchAction> {
        if self.touch.is_none() {
            self.touch_start(touch);
        }
        let active = self.touch.take().unwrap();

        if self.time - active.start_time < 0.5 && active.start == touch.position {
            return Some(TouchAction::ShortTap {
                position: touch.position,
            });
        }

        None
    }
}
