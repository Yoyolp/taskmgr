use crossterm::{event::{KeyCode, KeyEvent}};
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

use std::collections::{ HashMap, VecDeque };




#[derive(PartialEq)]
pub enum InputMode {
    False,
    Find,
    Command
}


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
    pub input_mode: InputMode,

    pub time_cyclic: u64,                   // 时间循环

    // Command Paser
    // command_paser: CommandPaser
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
            input_mode: InputMode::False,

            time_cyclic: 0,

            // command_paser: CommandPaser::default() 
        }
    }
    // 将 crossterm 的KeyCode 转化为 InputRqeuest处理
    fn handle_key(&mut self, code: KeyCode) {
        // 将 croessterm event -> tui-input
        let request = match code {
            KeyCode::Char(c) => InputRequest::InsertChar(c),
            KeyCode::Backspace => InputRequest::DeletePrevChar,
            _ => return
        };
        self.user_input.handle(request);
    }

    // 提交当前输入的内容
    // pub fn submit(&mut self) {
    //     let content = self.user_input.value();
    //     if !content.is_empty() {
    //         self.messages.push(std::format!("You {}", content));
    //     }

    // }
    
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
            
            self.time_cyclic = (self.time_cyclic + 1) & 0xffffffff;
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

    pub fn solve_keycode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        // step 1 处理键盘输入模式
        // step 2 分发模式

        match self.input_mode {
            InputMode::False => self.solve_keycode_false_mode(key_event, sysinfo),
            InputMode::Find => self.solve_keycode_find_mode(key_event),
            InputMode::Command => self.solve_keycode_command_mode(key_event),
        }
        
    }

    fn solve_keycode_false_mode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        let key_code = key_event.code;

        match key_code {
            KeyCode::Char('q') => self.is_running = false,

            KeyCode::Char('k') | KeyCode::Up  => self.scroll_up(sysinfo),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(sysinfo),

            KeyCode::Char('d') => {
                let pid = sysinfo.selected_pid;
                let _ = sysinfo.stop_proc_by_pid(pid);
            },
            
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Find;
                return;
            }

            KeyCode::Char(':') => {
                self.input_mode = InputMode::Command;
                return ;

            }
            _ => {}
        }
    }

    fn solve_keycode_find_mode(&mut self, key_event: KeyEvent) {
        let key_code = key_event.code;

        if self.input_mode != InputMode::Find {
            return ;
        }

        match key_code {
            KeyCode::Esc => {
                self.input_mode = InputMode::False;
                self.user_input = Input::default();
                return;
            }

            KeyCode::Enter => {
                // 定位到寻找的进程名字
                

                self.user_input = Input::default();
                self.input_mode = InputMode::False;     
                return ;
            }

            _ => self.handle_key(key_code)
        }
    }

    fn solve_keycode_command_mode(&mut self, key_event: KeyEvent) {
        let key_code = key_event.code;

        if self.input_mode != InputMode::Command {
            return ;
        }

        match key_code {
            KeyCode::Esc => {
                self.input_mode = InputMode::False;
                self.user_input = Input::default();
                return;
            }

            KeyCode::Enter => {

                self.messages.push(self.user_input
                    .value()
                    .to_string());

                let mut paser= 
                    CommandPaser::new(&self.messages
                        .last().unwrap().clone());
                
                paser.explain(self);
                
                // self.command_paser.explain();
                self.user_input = Input::default();
                self.input_mode = InputMode::False;
                return ;

            }

            _ => self.handle_key(key_code)
        }

    }

}

// 存在一个

pub struct CommandItem {
    command_item: String,
}

pub struct ValItem {
    val_type: String,
    val: String
}

pub struct CommandPaser {
    command_str: String,         // 命令原串
    val_table: HashMap<String, ValItem>,
    queue: VecDeque<CommandItem>
        
}

impl CommandPaser {
    pub fn new(command: &String) -> Self {
        let mut q = VecDeque::new();
        for chunk in command.split_whitespace() {
            q.push_back(CommandItem { command_item: String::from(chunk) });
        }

        Self {
            command_str: command.clone(),
            val_table: HashMap::new(),
            queue: q,
        }
    }

    // exit 退出
    // 删除
    // kill -n name
    // kill -p pid
    // kill -n name1 -p pid2
    // shutdown -n name -p pid ...         
    
    pub fn explain(&mut self, app: &mut App) {
        while !self.queue.is_empty() {
            
            if let Some(ptr) = self.queue.pop_front() {
                if ptr.command_item == "exit" {
                    app.is_running = false;
                }
            }

        }
    }

}
