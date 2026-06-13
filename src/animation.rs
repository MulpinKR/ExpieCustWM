use x11rb::protocol::xproto::Window;

#[derive(Clone, Debug)]
pub struct SlideAnim {
    pub window: Window,
    pub start_x: i32,
    pub end_x: i32,
    pub start_y: i32,
    pub end_y: i32,
    pub start_w: u16,
    pub end_w: u16,
    pub start_h: u16,
    pub end_h: u16,
    pub frame: u32,
    pub total_frames: u32,
}

impl SlideAnim {
    pub fn new(
        window: Window,
        start_x: i32, start_y: i32,
        end_x: i32, end_y: i32,
        w: u16, h: u16,
        frames: u32,
    ) -> Self {
        SlideAnim {
            window,
            start_x, end_x, start_y, end_y,
            start_w: w, end_w: w,
            start_h: h, end_h: h,
            frame: 0,
            total_frames: frames.max(1),
        }
    }

    pub fn lerp(a: i32, b: i32, t: f32) -> i32 {
        (a as f32 + (b - a) as f32 * t) as i32
    }

    pub fn current_x(&self) -> i32 {
        let t = self.frame as f32 / self.total_frames as f32;
        Self::lerp(self.start_x, self.end_x, t)
    }

    pub fn current_y(&self) -> i32 {
        let t = self.frame as f32 / self.total_frames as f32;
        Self::lerp(self.start_y, self.end_y, t)
    }

    pub fn advance(&mut self) -> bool {
        if self.frame < self.total_frames {
            self.frame += 1;
        }
        self.frame < self.total_frames
    }
}
