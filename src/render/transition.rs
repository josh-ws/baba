use macroquad::{
    color::{BLACK, Color},
    math::vec2,
    shapes::{draw_rectangle, draw_triangle},
    window::{screen_height, screen_width},
};

const COMPLETE_BLACK: i32 = 30;

#[derive(Debug, PartialEq, Eq)]
pub enum TransitionPhase {
    None,
    FadeOut,
    Black,
    FadeIn,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransitionStyle {
    Fade,
    Diamond,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransitionSmoothing {
    Linear,
    Smoothstep,
}

#[derive(Debug)]
pub struct Transition {
    timer: i32,
    duration: i32,
    phase: TransitionPhase,
    style: TransitionStyle,
    smoothing: TransitionSmoothing,
}

impl Transition {
    pub fn new(style: TransitionStyle, smoothing: TransitionSmoothing) -> Self {
        Self {
            timer: 0,
            duration: 100,
            phase: TransitionPhase::FadeOut,
            style,
            smoothing,
        }
    }

    pub fn is_active(&self) -> bool {
        self.phase != TransitionPhase::None
    }

    pub fn update(&mut self) -> bool {
        self.timer += 1;
        if self.phase == TransitionPhase::Black && self.timer >= COMPLETE_BLACK {
            self.phase = TransitionPhase::FadeIn;
            self.timer = 0;
        } else if self.timer >= self.duration {
            match self.phase {
                TransitionPhase::FadeOut => {
                    self.phase = TransitionPhase::Black;
                    self.timer = 0;
                    return true;
                }
                TransitionPhase::FadeIn => {
                    self.phase = TransitionPhase::None;
                }
                _ => (),
            }
        }
        false
    }

    pub fn draw(&self) {
        if self.phase == TransitionPhase::None {
            return;
        }
        if self.phase == TransitionPhase::Black {
            draw_rectangle(0., 0., screen_width(), screen_height(), BLACK);
            return;
        }
        let t = self.smooth();
        match self.style {
            TransitionStyle::Fade => draw_fade(t),
            TransitionStyle::Diamond => draw_diamond(t),
        }
    }

    fn progress(&self) -> f32 {
        let t = self.timer as f32 / self.duration as f32;
        match self.phase {
            TransitionPhase::FadeIn => 1. - t,
            _ => t,
        }
    }

    fn smooth(&self) -> f32 {
        let t = self.progress();
        match self.smoothing {
            TransitionSmoothing::Linear => t,
            TransitionSmoothing::Smoothstep => t * t * (3. - 2. * t),
        }
    }
}

fn draw_fade(t: f32) {
    let color = Color::new(0., 0., 0., t);
    draw_rectangle(0., 0., screen_width(), screen_height(), color);
}

fn draw_diamond(t: f32) {
    let color = Color::new(0., 0., 0., 1.0);
    let w = screen_width();
    let h = screen_height();
    let zero = vec2(0., 0.);
    draw_triangle(zero, vec2(w * t, 0.), vec2(0., h * t), color);
    draw_triangle(vec2(w, 0.), vec2(w * (1. - t), 0.), vec2(w, h * t), color);
    draw_triangle(vec2(0., h), vec2(0., h * (1. - t)), vec2(w * t, h), color);
    draw_triangle(vec2(w, h), vec2(w, h * (1. - t)), vec2(w * (1. - t), h), color);
}
