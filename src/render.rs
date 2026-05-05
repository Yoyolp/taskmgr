use std::rc::Rc;

// use std::os::windows::process;
use ratatui::{
    Frame, layout::Rect, style::Stylize, symbols::Marker, widgets::{Axis, Cell, Chart, Dataset, GraphType, Paragraph, Row, ScrollbarOrientation,  Table, }
};
use ratatui::{
    // backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    // text::{Line, Span},
    widgets::{Block, Borders, Scrollbar},
};

// pub mod tmgr;
use crate::tmgr::{SystemInfo};

use crate::app::App;

// 这个结构主要 处理渲染相关的问题
pub struct Render {
    pub chunks: Rc<[Rect]>,            // 界面分配
}

impl Render {
    pub fn new(frame: &mut Frame) -> Self {
        Self {
            chunks: Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    // Constraint::Length(3),   // 标题栏
                    Constraint::Length(8),   // CPU 仪表
                    Constraint::Min(0),      // 内容区域
                    Constraint::Length(3),   // 状态栏
                ])
                .split(frame.size()),
        }
    }  

    pub fn main_render(mut self, frame: &mut Frame, app: &mut App, mut sysinfo: &mut SystemInfo) {
        
        // 刷新信息
        app.update_if_needed(&mut sysinfo);
        
        // 打印仪表盘
        panel_area_render(&mut self, frame, sysinfo);
                    
        // PROCESS  =====================================
        // 进程列表
            
        // 定义列宽
        let widths = [
            Constraint::Length(8),   // PID
            Constraint::Length(35),  // Name
            Constraint::Length(8),   // CPU
            Constraint::Length(8),   // MEM
            Constraint::Min(10),     // Status
        ];
        // 创建表头
        let header = Row::new(vec![
            Cell::new("PID"),
            Cell::new("Name"),
            Cell::new("CPU%"),
            Cell::new("MEM(MB)"),
            Cell::new("status"),
        ])
        .bottom_margin(1);
        
        let table_area = self.chunks[1];
        //  方法1：滚动条放在表格右边（需要水平布局
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),   // 表格
                Constraint::Length(1), // 滚动条
            ])
            .split(table_area);

        // 渲染表格 
        let table = Table::new(app.processes.clone(), widths)
            .header(header)
            .block(Block::bordered().title("Process"))
            .highlight_style(Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .bold()
            )
            .highlight_symbol(">> ");
            
            // .scroll((app.vertical_scroll, 0));
            
        frame.render_stateful_widget(table, horizontal_chunks[0], &mut app.table_state);
        // 创建滚动条状态
            
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .thumb_symbol("█")
            .track_symbol(Some("│"));

        // 渲染滚动条（使用 inner 可以留出边距）
        frame.render_stateful_widget(
            scrollbar,
            horizontal_chunks[1],
            &mut app.scrollbar_state,
        );
        // frame.render_stateful_widget(table, chunks[1], &mut app.table_state);

        // 状态栏
        // let mut selected_pid: u32 = 0;
        if let Some(selected_index) = app.table_state.selected() {
            // let process_count = SystemInfo::total_process_count(&sysinfo);
            // let procinfo = ProcessInfo::new(&sysinfo, process_count);
            // let target_proc = &procinfo.procs[selected_index];
            let target_proc = &sysinfo.proc_info.procs[selected_index];
            sysinfo.selected_pid = target_proc.pid;
        }
        
        let status = Paragraph::new(
            std::format!(
                "Press 'q' to quit | ↑/↓,j/k to navigate | 'd' delete proc | choose -> {:}",
                &sysinfo.selected_pid
            ))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(status, self.chunks[2]);

    }
}

// 打印仪表盘部分
fn panel_area_render(r: &mut Render, frame: &mut Frame, sysinfo: &mut SystemInfo) {
    // 标题  
    // 数据点：x 和 y 都是 0.0 到 1.0 之间的值
    let cpu_points = sysinfo.cpu_usage_points.clone()
        .into_iter()
        .map(|(x, y)| (x as f64, y as f64))
        .collect::<Vec<(f64, f64)>>();
        
    // 创建数据集
    let cpu_dataset = Dataset::default()
        .name("CPU")
        .marker(Marker::Braille)// 使用盲文
        .graph_type(GraphType::Line)
        .style(Color::Yellow)
        .data(&cpu_points);
        
    // 创建图表自动处理坐标映射和绘制
    let cpu_chart = Chart::new(vec![cpu_dataset])
        .block(Block::default()
            .title(std::format!(
                "CPU-{:.2}%", 
                sysinfo.cpu_usage
            ))
            .borders(Borders::ALL))
        .x_axis(Axis::default().bounds([0.0, 1.0]))
        .y_axis(Axis::default().bounds([0.0, 1.0]));
        // .style(Color::Blue);
        
    // MEM set
        
    let mem_points = sysinfo.mem_percent_points.clone();
    let mem_dataset = Dataset::default()
        .name("MEM")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Color::Yellow)
        .data(&mem_points);
        
    let mem_chart = Chart::new(vec![mem_dataset])
        .block(Block::default()
            .title(std::format!(
                "MEM-{:.1}%,{:.1}GB", 
                sysinfo.mem_percent,
                sysinfo.total_mem
            ))
            .borders(Borders::ALL))
        .x_axis(Axis::default().bounds([0.0, 1.0]))
        .y_axis(Axis::default().bounds([0.0, 1.0]));
        
    let panel_area = r.chunks[0];
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // CPU PANEL AREA 0
            Constraint::Length(20), // MEM PANEL AREA 1
            Constraint::Length(60), // WIFI PANEL AREA 2
            Constraint::Min(0)
        ])
        .split(panel_area);
        
    frame.render_widget(cpu_chart, horizontal_chunks[0]);
    frame.render_widget(mem_chart, horizontal_chunks[1]);
    
    let wifi_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3)
        ])
        .split(horizontal_chunks[2]);

    let network_name = sysinfo.network_info.network_name.clone();
    let tr = sysinfo.network_info.get_network_speed(&network_name).unwrap();
    let wifi_status = Paragraph::new(
            std::format!(
                "{}\nr:{}bs t: {}bs",
                network_name,
                tr.0,
                tr.1
            )
        ).bold();
    
    frame.render_widget(wifi_status, wifi_chunks[0]);
}

