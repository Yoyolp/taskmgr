use crossterm::event::KeyCode;
// use std::os::windows::process;
use ratatui::{
     widgets::{ Row,  ScrollbarState, TableState}
};

use std::time::{
    Duration, Instant
};

use crate::tmgr::SystemInfo;

pub struct App {
    pub processes: Vec<Row<'static>>,
    pub last_update: Instant,
    pub update_interval: Duration,
    
    pub table_state: TableState,
    
    // 滑条是实现
    pub scrollbar_state: ScrollbarState,     // 滚动条状态
    pub vertical_scroll: usize,              // 当前滚动条状态
    
    // 用户输入
    // pub user_input: Input,
    pub is_running: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500),   // 500 ms
            
            table_state: TableState::default(),
            
            scrollbar_state: ScrollbarState::new(0),
            vertical_scroll: 0,
            
            // user_input: Input::default()
            is_running: true
        }
    }
    
    pub fn update_if_needed(&mut self, sysinfo: &mut SystemInfo) {
        let now = Instant::now();

        if now.duration_since(self.last_update) >= self.update_interval {
            // 更新信息
            sysinfo.refresh();
            // 跟新滚动条总长度
            self.scrollbar_state = self.scrollbar_state.content_length(self.processes.len());
            self.last_update = now;

        }
    }
    
    // 滚动处理
    fn scroll_up(&mut self) {
        if self.vertical_scroll > 0 {
            self.vertical_scroll -= 1;
            self.table_state.select(Some(self.vertical_scroll as usize));
            self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        }
    }

    fn scroll_down(&mut self) {
        if (self.vertical_scroll as usize) < self.processes.len().saturating_sub(1) {
            self.vertical_scroll += 1;
            self.table_state.select(Some(self.vertical_scroll as usize));
            self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        }
    }

    pub fn solve_keycode(&mut self, key_code: KeyCode, sysinfo: &mut SystemInfo) {
        match key_code {
            KeyCode::Char('q') => self.is_running = false,

            KeyCode::Up => self.scroll_up(),
            KeyCode::Char('k') => self.scroll_up(),

            KeyCode::Down => self.scroll_down(),
            KeyCode::Char('j') => self.scroll_down(),

            KeyCode::Char('d') => {
                let pid = sysinfo.selected_pid;
                let _ = sysinfo.stop_proc_by_pid(pid);
            },

            _ => {}
        }
    }
}



