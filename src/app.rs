use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
     widgets::{  ScrollbarState, TableState}
};
use tui_input::{Input};
use tui_input::InputRequest;

use std::{cell::RefCell, time::{
    Duration, Instant
}};

use crate::{ tmgr::SystemInfo};

use std::collections::{ VecDeque };
use std::rc::Rc;

#[derive(PartialEq)]
pub enum InputMode {
    False,
    Find,
    Command
}
// TODO： 记得重构 APP

pub struct AppCore {
    is_running: bool,
    messages: Vec<String>,
}

impl AppCore {
    pub fn new() -> Self {
        Self {
            is_running: true,
            messages: Vec::default(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn exit(&mut self) {
        self.is_running = false;
    }

    pub fn push_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn get_last_message(&self) -> String {
        self.messages.last().cloned().unwrap_or_else(|| "nop".into())
    }
}

pub struct ScrollManager {
    // 滑条是实现
    pub table_state: TableState,
    pub scrollbar_state: ScrollbarState,     // 滚动条状态
}

impl ScrollManager {
    pub fn new(total_rows: usize) -> Self {
        Self {
            table_state: TableState::default(),
            scrollbar_state: ScrollbarState::new(total_rows),
        }
    }

    // 滚动处理
    pub fn scroll_up(&mut self, sysinfo: &SystemInfo) {
        let total_rows = sysinfo.proc_info.procs.len();
        if let Some(selected) = self.table_state.selected() {
            if selected > 0 {
                self.table_state.select(Some(selected - 1));
            }
        } else if total_rows > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub fn scroll_down(&mut self, sysinfo: &SystemInfo) {
        let total_rows = sysinfo.proc_info.procs.len();
        if let Some(selected) = self.table_state.selected() {
            if selected + 1 < total_rows {
                self.table_state.select(Some(selected + 1));
            }
        } else if total_rows > 0 {
            self.table_state.select(Some(0));
        }
    }

    pub fn table_state_select(&mut self, index: usize) {
        self.table_state.select(Some(index));
    }
}

pub struct UserInputManager {
    pub input_mode: InputMode,
    pub user_input: Input,

    find_results: Vec<usize>,
    current_find_index: usize,

    // 指向 app 中的 scroll_manager 的指针
    pub scroll_manager_ptr: Rc<RefCell<ScrollManager>>,
}

impl UserInputManager {
    pub fn new(sm_ptr: Rc<RefCell<ScrollManager>>) -> Self {
        Self {
            input_mode: InputMode::False,
            user_input: Input::default(),
            find_results: Vec::default(),
            current_find_index: 0,

            scroll_manager_ptr: sm_ptr,
        }
    }
    
    pub fn handle_key(&mut self, code: KeyCode) {
        // 将 croessterm event -> tui-input
        let request = match code {
            KeyCode::Char(c) => InputRequest::InsertChar(c),
            KeyCode::Backspace => InputRequest::DeletePrevChar,
            _ => return
        };
        self.user_input.handle(request);
    }

    pub fn find_processes(&mut self, sysinfo: &SystemInfo, pattern: &str) {
        if pattern.is_empty() {
            self.find_results.clear();
            return ;
        }
        
        self.find_results = sysinfo.find_processes_by_name(pattern, true);
        // 自动跳转到第一个匹配项
        if !self.find_results.is_empty() {
            self.current_find_index = 0;

            self.scroll_manager_ptr
                .borrow_mut()
                .table_state_select(self.find_results[0]);
        }
    }

    pub fn next_find_result(&mut self) {
        if self.find_results.is_empty() {
            return ;
        }

        self.current_find_index = (self.current_find_index + 1) % self.find_results.len();
        self.scroll_manager_ptr
            .borrow_mut()
            .table_state_select(self.find_results[self.current_find_index]);
    }

    pub fn prev_find_result(&mut self) {
        if self.find_results.is_empty() {
            return ;
        }
        
        self.current_find_index = (self.current_find_index + self.find_results.len() - 1) % self.find_results.len();
        self.scroll_manager_ptr
            .borrow_mut()
            .table_state_select(self.find_results[self.current_find_index]);
    }
}

pub struct App {
    pub core: AppCore,

    pub last_update: Instant,
    pub update_interval: Duration,
    
    // 滑条是实现
    pub scroll_manager: Rc<RefCell<ScrollManager>>,
    pub usr_input_manager: UserInputManager,
    
    pub time_cyclic: u64,
}

impl App {
    pub fn new() -> Self {
        
        let scroll_manager = Rc::new( RefCell::new(ScrollManager::new(0)) );
        let usr_input_manager = UserInputManager::new(Rc::clone(&scroll_manager));

        Self {
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500),   // 500 ms

            scroll_manager: scroll_manager,
            usr_input_manager: usr_input_manager,   
            // user_input: Input::default(),
            // input_mode: InputMode::False,
            time_cyclic: 0,

            // find_results: Vec::default(),
            // current_find_index: 0,

            // core
            core: AppCore::new(),            
        }
    }
   
    pub fn update_if_needed(&mut self, sysinfo: &mut SystemInfo) {
        let now = Instant::now();

        if now.duration_since(self.last_update) >= self.update_interval {
            // 更新信息
            sysinfo.refresh();
            // 跟新滚动条总长度
            let total_rows = sysinfo.proc_info.procs.len();

            {
                let mut srcoll = self.scroll_manager.borrow_mut();
                srcoll.scrollbar_state = srcoll.scrollbar_state.content_length(total_rows);
            }

            let selected = self.scroll_manager.borrow().table_state.selected();
            if let Some(selected) = selected {
                if selected >= total_rows && total_rows > 0 {
                    self.scroll_manager
                        .borrow_mut()
                        .table_state
                        .select(Some(total_rows - 1));
                }
            }
            self.time_cyclic = (self.time_cyclic + 1) & 0xffffffff;
            self.last_update = now;
        }
    }
    
    pub fn solve_keycode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        // step 1 分发模式
        // step 2 处理键盘输入

        match self.usr_input_manager.input_mode {
            InputMode::False => self.solve_keycode_false_mode(key_event, sysinfo),
            InputMode::Find => self.solve_keycode_find_mode(key_event, sysinfo),
            InputMode::Command => self.solve_keycode_command_mode(key_event, sysinfo),
        }
    }

    fn solve_keycode_false_mode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        let key_code = key_event.code;

        match key_code {
            KeyCode::Char('q') => self.core.exit(),

            KeyCode::Char('k') | KeyCode::Up  => self.scroll_manager.borrow_mut().scroll_up(sysinfo),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_manager.borrow_mut().scroll_down(sysinfo),

            KeyCode::Char('d') => {
                let pid = sysinfo.selected_pid;
                let _ = sysinfo.stop_proc_by_pid(pid);
            },
            
            KeyCode::Char('/') => {
                self.usr_input_manager.input_mode = InputMode::Find;
            }

            KeyCode::Char(':') => {
                self.usr_input_manager.input_mode = InputMode::Command;
            }

            KeyCode::Char('n') => {
                self.usr_input_manager.next_find_result();
            }
            KeyCode::Char('N') => {
                self.usr_input_manager.prev_find_result();
            }

            _ => {}
        }
    }

    fn solve_keycode_find_mode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        let key_code = key_event.code;

        if self.usr_input_manager.input_mode != InputMode::Find {
            return ;
        }

        match key_code {
            KeyCode::Esc => {
                self.usr_input_manager.input_mode = InputMode::False;
                self.usr_input_manager.user_input = Input::default();
                self.usr_input_manager.find_results.clear();
                return;
            }

            KeyCode::Enter => {
                // 定位到寻找的进程名字
                let search_pattern = self.usr_input_manager.user_input.value().to_string();

                self.usr_input_manager.find_processes(sysinfo, &search_pattern);
                self.usr_input_manager.user_input = Input::default();
                self.usr_input_manager.input_mode = InputMode::False;     
                return ;
            }

            _ => self.usr_input_manager.handle_key(key_code)
        }
    }

    fn solve_keycode_command_mode(&mut self, key_event: KeyEvent, sysinfo: &mut SystemInfo) {
        let key_code = key_event.code;

        if self.usr_input_manager.input_mode != InputMode::Command {
            return ;
        }

        match key_code {
            KeyCode::Esc => {
                self.usr_input_manager.input_mode = InputMode::False;
                self.usr_input_manager.user_input = Input::default();
                return;
            }

            KeyCode::Enter => {

                self.core.push_message(self.usr_input_manager.user_input
                    .value()
                    .to_string());

                let mut paser= 
                    CommandParser::new(&self.core.get_last_message());
                
                paser.explain(self, sysinfo);
                
                // self.command_paser.explain();
                self.usr_input_manager.user_input = Input::default();
                self.usr_input_manager.input_mode = InputMode::False;
                return ;

            }

            _ => self.usr_input_manager.handle_key(key_code)
        }

    }

}

// 存在一个

pub struct CommandItem {
    command_item: String,
}

pub struct CommandParser {
    // command_str: String,         // 命令原串
    // val_table: HashMap<String, ValItem>,
    queue: VecDeque<CommandItem>
        
}


impl CommandParser {
    pub fn new(command: &String) -> Self {
        let mut q = VecDeque::new();
        for chunk in command.split_whitespace() {
            q.push_back(CommandItem { command_item: String::from(chunk) });
        }

        Self {
            // command_str: command.clone(),
            // val_table: HashMap::new(),
            queue: q,
        }
    }
    
    // exit 退出
    // 删除
    // kill -n name
    // kill -p pid
    // kill -n name1 -p pid2
    // shutdown -n name -p pid ...         
    
    pub fn explain(&mut self, app: &mut App, sysinfo: &mut SystemInfo) {
        if self.queue.is_empty() {
            return ;
        }

            let cmd = self.queue.pop_front().unwrap().command_item;
    
            match cmd.as_str() {
                "exit" => app.core.exit(),
                // 这段 命令实现 存在问题
                "kill" => {
                    while !self.queue.is_empty() {
                        let item = self.queue.pop_front().unwrap().command_item;
                        if item.starts_with("-n") {
                            let name = item[2..].trim();
                            app.core.push_message(format!("Killing processes with name: {}", name));
                            let pids = sysinfo.find_processes_by_name(name, false);
                            for pid in pids {
                                sysinfo.stop_proc_by_pid(pid as u32).ok();
                            }                           

                        } else if item.starts_with("-p") {
                            let pid_str = item[2..].trim();
                            if let Ok(pid) = pid_str.parse::<i32>() {

                                app.core.push_message(format!("Killing process with PID: {}", pid));
                                sysinfo.stop_proc_by_pid(pid as u32).ok();

                            } else {
                                app.core.push_message(format!("Invalid PID: {}", pid_str));
                            }
                        }
                    }
                }
                _ => {
                    app.core.push_message(format!("Unknown command: {}", cmd));
                }
            }

    }

}
