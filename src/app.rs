// use std::os::windows::process;
    
use ratatui::{
     widgets::{ Cell, Row,  ScrollbarState, TableState}
};
// use ratatui::{
    // backend::CrosstermBackend,
    // layout::{Constraint, Direction, Layout},
    // style::{Color, Style},
    // text::{Line, Span},
    // widgets::{Block, Borders, Scrollbar},
// };
use std::time::{
    Duration, Instant
};


use crate::tmgr::{SystemInfo, ProcessInfo};

pub struct App {
    pub processes: Vec<Row<'static>>,
    pub last_update: Instant,
    pub update_interval: Duration,
    
    pub table_state: TableState,
    
    // 滑条是实现
    pub scrollbar_state: ScrollbarState,     // 滚动条状态
    pub vertical_scroll: usize,              // 当前滚动条状态
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
        }
    }
    
    pub fn update_if_needed(&mut self, sysinfo: &mut SystemInfo) {
        let now = Instant::now();

        if now.duration_since(self.last_update) >= self.update_interval {
            // 更新信息
            sysinfo.refresh();
            // let process_count = SystemInfo::total_process_count(&sysinfo);
            // let mut procinfo = ProcessInfo::new(&sysinfo, process_count);
            // sysinfo.proc_info

            // 按照内存排序
            // sysinfo.proc_info.procs.sort_by(|a, b| {
                // b.mem.partial_cmp(&a.mem).unwrap()
            // });
            // 创建Table Row
            self.processes = sysinfo.proc_info.procs
                .iter()         
                .map(|item| {
                    Row::new(vec![
                        Cell::new(std::format!("{}", item.pid)),
                        Cell::new(item.name.clone()),
                        Cell::new(std::format!("{:.1}", item.cpu)),
                        Cell::new(std::format!("{:.1}", item.mem)),
                        Cell::new(std::format!("{}", item.status)),
                    ])                    
                    // ListItem::new(std::format!(        
                    //     "{:<6} {:<35} {:5.1} {:<5.1} {}",
                    //     item.pid, item.name, item.cpu, item.mem, item.status
                    // ))
                })
                .collect::<Vec<Row>>();
            
            // 跟新滚动条总长度
            self.scrollbar_state = self.scrollbar_state.content_length(self.processes.len());
            self.last_update = now;
        }
    }


    // 滚动处理
    pub fn scroll_up(&mut self) {
        if self.vertical_scroll > 0 {
            self.vertical_scroll -= 1;
            self.table_state.select(Some(self.vertical_scroll as usize));
            self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        }
    }

    pub fn scroll_down(&mut self) {
        if (self.vertical_scroll as usize) < self.processes.len().saturating_sub(1) {
            self.vertical_scroll += 1;
            self.table_state.select(Some(self.vertical_scroll as usize));
            self.scrollbar_state = self.scrollbar_state.position(self.vertical_scroll);
        }
    }
}