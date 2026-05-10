use crossterm::event::KeyCode;
// use std::os::windows::process;
use ratatui::{
     widgets::{  ScrollbarState, TableState}
};
use tui_input::{Input};
use tui_input::InputRequest;
// use tui_input::backend::crossterm::InputEvent;

use std::time::{
    Duration, Instant
};

use crate::tmgr::SystemInfo;
// use tui_input::backend::crossterm::EventHandler;
pub struct App {
    // pub processes: Vec<Row<'static>>,
    pub last_update: Instant,
    pub update_interval: Duration,
    
    pub table_state: TableState,
    
    // 滑条是实现
    pub scrollbar_state: ScrollbarState,     // 滚动条状态
    // pub vertical_scroll: usize,              // 当前滚动条状态
    
    // 用户输入
    pub user_input: Input,
    messages: Vec<String>,
    
    // 运行状态
    pub is_running: bool,
    pub input_mode: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            // processes: Vec::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500),   // 500 ms
            
            table_state: TableState::default(),
            
            scrollbar_state: ScrollbarState::new(0),
            // vertical_scroll: 0,
            
            user_input: Input::default(),
            messages: Vec::default(),

            // user_input: Input::default()
            is_running: true,
            input_mode: false
        }
    }
    // 将 crossterm 的KeyCode 转化为 InputRqeuest处理
    pub fn handle_key(&mut self, code: KeyCode) {
        // 将 croessterm event -> tui-input
        let request = match code {
            KeyCode::Char(c) => InputRequest::InsertChar(c),
            KeyCode::Backspace => InputRequest::DeletePrevChar,
            // KeyCode::Delete => InputRequest::DeleteChar,
            // KeyCode::Left => InputRequest::MoveCursorLeft,
            // KeyCode::Right => InputRequest::MoveCursorRight,
            // KeyCode::Home => InputRequest::MoveCursorToBeginning,
            // KeyCode::End => InputRequest::MoveCursorToEnd,
            // KeyCode::Enter => InputRequest::,
            _ => return
        };
        self.user_input.handle(request);
    }

    // 提交当前输入的内容
    pub fn submit(&mut self) {
        let content = self.user_input.value();
        if !content.is_empty() {
            self.messages.push(std::format!("You {}", content));
        }

    }

    
    pub fn update_if_needed(&mut self, sysinfo: &mut SystemInfo) {
        let now = Instant::now();

        if now.duration_since(self.last_update) >= self.update_interval {
            // 更新信息
            sysinfo.refresh();
            // 跟新滚动条总长度
            let total_rows = sysinfo.proc_info.procs.len();
            self.scrollbar_state = self.scrollbar_state.content_length(total_rows);
            // self.scrollbar_state = self.scrollbar_state.content_length(self.processes.len());
            
            if let Some(selected) = self.table_state.selected() {
                if selected >= total_rows && total_rows > 0 {
                    self.table_state.select(Some(total_rows - 1));
                }
                
            }
            
            self.last_update = now;
        }
    }
    
    // 滚动处理
    fn scroll_up(&mut self, sysinfo: &SystemInfo) {
        let total_rows = sysinfo.proc_info.procs.len();
        if let Some(selected) = self.table_state.selected() {
            if selected > 0 {
                self.table_state.select(Some(selected - 1));
            }
        } else if total_rows > 0 {
            self.table_state.select(Some(0));
        }
    }

    fn scroll_down(&mut self, sysinfo: &SystemInfo) {
        let total_rows = sysinfo.proc_info.procs.len();
        if let Some(selected) = self.table_state.selected() {
            if selected + 1 < total_rows {
                self.table_state.select(Some(selected + 1));
            }
        } else if total_rows > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub fn solve_keycode(&mut self, key_code: KeyCode, sysinfo: &mut SystemInfo) {
        if self.input_mode {
            if key_code == KeyCode::Esc {
                self.input_mode = if self.input_mode { false } else { true };
                return;
            }

            self.handle_key(key_code);
            return;
        }

        match key_code {
            KeyCode::Char('q') => self.is_running = false,

            KeyCode::Char('k') | KeyCode::Up  => self.scroll_up(sysinfo),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(sysinfo),

            KeyCode::Char('d') => {
                let pid = sysinfo.selected_pid;
                let _ = sysinfo.stop_proc_by_pid(pid);
            },
            
            KeyCode::Char('/') => self.input_mode = true,
            // KeyCode::Esc => self.input_mode = if self.input_mode { false } else { true },

            _ => {}
        }
    }
}



