use crate::client::{Client, LayoutMode};

#[derive(Debug, Clone)]
pub struct LayoutEngine {
    pub mode: LayoutMode,
    pub master_count: usize,
    pub master_ratio: f64,
    pub gap: u32,
    pub border_width: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub client_index: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            mode: LayoutMode::Tiling,
            master_count: 1,
            master_ratio: 0.55,
            gap: 4,
            border_width: 2,
        }
    }

    pub fn arrange(&self, clients: &[Client], area: &Area, _sel_index: Option<usize>) -> Vec<Placement> {
        let visible: Vec<usize> = clients.iter().enumerate()
            .filter(|(_, c)| !c.fullscreen)
            .map(|(i, _)| i)
            .collect();

        if visible.is_empty() {
            return vec![];
        }

        match self.mode {
            LayoutMode::Floating => self.arrange_floating(clients, &visible),
            LayoutMode::Monocle => self.arrange_monocle(area, &visible),
            LayoutMode::Tiling => self.arrange_tiling(area, &visible),
        }
    }

    fn arrange_floating(&self, clients: &[Client], visible: &[usize]) -> Vec<Placement> {
        visible.iter().map(|&i| {
            let c = &clients[i];
            Placement {
                client_index: i,
                x: c.x,
                y: c.y,
                width: c.width,
                height: c.height,
            }
        }).collect()
    }

    fn arrange_monocle(&self, area: &Area, visible: &[usize]) -> Vec<Placement> {
        let g = self.gap as i32;
        visible.iter().map(|&i| {
            Placement {
                client_index: i,
                x: area.x + g,
                y: area.y + g,
                width: (area.width as u32).saturating_sub((g * 2) as u32),
                height: (area.height as u32).saturating_sub((g * 2) as u32),
            }
        }).collect()
    }

    fn arrange_tiling(&self, area: &Area, visible: &[usize]) -> Vec<Placement> {
        if visible.is_empty() {
            return vec![];
        }

        let g = self.gap as i32;
        let n = visible.len();
        let master_n = self.master_count.min(n);

        let mut placements = Vec::with_capacity(n);

        if n == 1 {
            placements.push(Placement {
                client_index: visible[0],
                x: area.x + g,
                y: area.y + g,
                width: (area.width as u32).saturating_sub((g * 2) as u32),
                height: (area.height as u32).saturating_sub((g * 2) as u32),
            });
            return placements;
        }

        let master_w = ((area.width as f64 - (g * (master_n as i32 + 1)) as f64) * self.master_ratio) as i32;
        let stack_w = (area.width - g * 2 - master_w - g).max(100);

        for (pos, &idx) in visible.iter().enumerate() {
            if pos < master_n {
                let master_h = (area.height - g * (master_n as i32 + 1)) / master_n as i32;
                placements.push(Placement {
                    client_index: idx,
                    x: area.x + g,
                    y: area.y + g + pos as i32 * (master_h + g),
                    width: master_w.max(1) as u32,
                    height: master_h.max(1) as u32,
                });
            } else {
                let stack_i = pos - master_n;
                let stack_n = n - master_n;
                let stack_h = (area.height - g * (stack_n as i32 + 1)) / stack_n as i32;
                placements.push(Placement {
                    client_index: idx,
                    x: area.x + g + master_w + g,
                    y: area.y + g + stack_i as i32 * (stack_h + g),
                    width: stack_w.max(1) as u32,
                    height: stack_h.max(1) as u32,
                });
            }
        }

        placements
    }
}
