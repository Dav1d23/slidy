use std::time::SystemTime;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use super::{utils, utils::GenericWindow};

/// Define the status of the timer.
enum TimerStatus {
    /// Stopped.
    Stopped,
    /// Since when it is running.
    Running(SystemTime),
}

pub struct TimerWindow<'a> {
    /// Contains the generic information for a window
    pub generic_win: GenericWindow,
    timer_status: TimerStatus,
    /// Total amount of elapsed seconds the timer run.
    total_elapsed: usize,
    /// If the window is visible
    is_visible: bool,
    /// The default font to be used.
    default_font: &'a sdl2::ttf::Font<'a, 'a>,
}

impl<'a> TimerWindow<'a> {
    pub fn new(
        context: &sdl2::Sdl,
        font: &'a sdl2::ttf::Font,
        resizable: bool,
        h: u32,
        w: u32,
    ) -> Self {
        let timer_status = TimerStatus::Stopped;
        let total_elapsed = 0;
        TimerWindow {
            generic_win: GenericWindow::new(context, resizable, h, w, "Timer"),
            timer_status,
            total_elapsed,
            is_visible: true,
            default_font: font,
        }
    }

    /// Manage the keypresses, or any other even related to this very
    /// window. We don't want other elements to manage our keys!
    pub fn manage_keypress(&mut self, event: &Event) {
        match event {
            // KeyUp: SPACEBAR
            Event::KeyUp {
                keycode: Some(Keycode::Space),
                ..
            } => self.timer_toggle(),
            // KeyUp: R
            Event::KeyUp {
                keycode: Some(Keycode::R),
                ..
            } => self.timer_reset(),
            _ => {}
        }
    }

    /// Toggle visibility
    pub fn visibility_toggle(&mut self) {
        let c = &mut self.generic_win.canvas;
        if self.is_visible {
            c.window_mut().hide();
        } else {
            c.window_mut().show();
        }
        self.is_visible = !self.is_visible;
    }

    /// Toggle between stop and run states.
    pub fn timer_toggle(&mut self) {
        if let TimerStatus::Stopped = self.timer_status {
            self.timer_start()
        } else {
            self.timer_stop()
        }
    }

    /// Reset timer.
    pub fn timer_reset(&mut self) {
        self.timer_stop();
        self.total_elapsed = 0;
    }

    /// Start the timer.
    pub fn timer_start(&mut self) {
        self.timer_status = TimerStatus::Running(SystemTime::now());
    }

    /// Stop the timer, and update the elapsed time.
    pub fn timer_stop(&mut self) {
        let elapsed = match self.timer_status {
            TimerStatus::Running(since) => since.elapsed().unwrap().as_secs(),
            TimerStatus::Stopped => 0,
        };
        self.total_elapsed += elapsed as usize;
        self.timer_status = TimerStatus::Stopped;
    }

    /// Returns a tuple with hours/minutes/seconds elapsed
    fn get_time(&self) -> (u8, u8, u8) {
        let elapsed = match self.timer_status {
            TimerStatus::Running(since) => since.elapsed().unwrap().as_secs(),
            TimerStatus::Stopped => 0,
        };

        let total_secs = self.total_elapsed + elapsed as usize;
        let seconds = total_secs % 60;
        let minutes = ((total_secs - seconds) % (60 * 60)) / 60;
        let hours = (total_secs - (minutes * 60) - seconds) / (60 * 60);
        (hours as u8, minutes as u8, seconds as u8)
    }

    /// Main method to show a slide on the screen.
    pub fn update(&mut self, slides_tot: usize, slides_idx: usize) {
        let (h, m, s) = self.get_time();
        let c = &mut self.generic_win.canvas;
        utils::canvas_change_color(c, Color::CYAN);
        // Draw the timer
        let surface_text = self
            .default_font
            .render(format!("{:02}:{:02}:{:02}", h, m, s).as_str())
            .solid(Color::RED)
            .unwrap();
        let texture_creator = c.texture_creator();
        let texture = surface_text.as_texture(&texture_creator).unwrap();
        let rect = utils::get_scaled_rect(c.window(), 0.04, 0.04, 0.6, 0.6);
        c.copy(&texture, None, rect).unwrap();
        // @safety This is ok, since the texture has been copied and we can
        // safely remove it.
        unsafe {
            texture.destroy();
        }
        // Draw the slide counter
        let surface_text = self
            .default_font
            .render(format!("{}/{}", slides_idx, slides_tot).as_str())
            .solid(Color::BLACK)
            .unwrap();
        let texture_creator = c.texture_creator();
        let texture = surface_text.as_texture(&texture_creator).unwrap();
        let rect = utils::get_scaled_rect(c.window(), 0.65, 0.65, 0.33, 0.33);
        c.copy(&texture, None, rect).unwrap();
        // @safety This is ok, since the texture has been copied and we can
        // safely remove it.
        unsafe {
            texture.destroy();
        }
    }
}
