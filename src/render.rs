use std::{rc::Rc, };

use ratatui::{
    Frame, 
    layout::Rect, 
    style::Stylize, 
    symbols::Marker, 
    widgets::{Axis, Cell, Chart, Dataset, GraphType, Paragraph, Row, ScrollbarOrientation,  Table, }
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Scrollbar},
};

// pub mod tmgr;
use crate::tmgr::{SystemInfo};

use crate::app::{App, InputMode};

// 处理渲染相关的问题
pub fn main_render(frame: &mut Frame, app: &mut App, mut sysinfo: &mut SystemInfo) {
    let mut chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Constraint::Length(3),   // 标题栏
            Constraint::Length(8),   // CPU 仪表
            Constraint::Min(0),      // 内容区域
            Constraint::Length(3),   // 状态栏
            Constraint::Length(3),   // 用户输入框
        ])
        .split(frame.area());
    
    app.update_if_needed(&mut sysinfo);
    // panel_area_render(&mut chunks, frame, sysinfo);
    // 打印仪表盘
    panel_area_render(&mut chunks, frame, sysinfo);
    // PROCESS  
    proc_list_render(&mut chunks, frame, app, sysinfo);

    user_input_render(&mut chunks, frame, app, sysinfo);
}

// 打印仪表盘部分
fn panel_area_render(chunks: &mut Rc<[Rect]>, frame: &mut Frame, sysinfo: &mut SystemInfo) {
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
        
    let panel_area = chunks[0];
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

// 进程列表
fn proc_list_render(chunks: &mut Rc<[Rect]>, frame: &mut Frame, app: &mut App, sysinfo: &mut SystemInfo) {
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
    
    let table_area = chunks[1];
    //  方法1：滚动条放在表格右边（需要水平布局
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),   // 表格
            Constraint::Length(1), // 滚动条
        ])
        .split(table_area);
    
    // 创建表格
    let proc_table = sysinfo.proc_info.procs
            .iter()
            .map(|item| {
                Row::new(vec![
                    Cell::new(std::format!("{}", item.pid)),
                    Cell::new(item.name.clone()),
                    Cell::new(std::format!("{:.1}", item.cpu)),
                    Cell::new(std::format!("{:.1}", item.mem)),
                    Cell::new(std::format!("{}", item.status)),
                ])
                
            })
            .collect::<Vec<Row>>();
    
    let total_rows = proc_table.len();

    // 渲染表格 
    // let table = Table::new(app.processes.clone(), widths)
    let table = Table::new(proc_table, widths)
        .header(header)
        .block(Block::bordered()
            .title(std::format!("Process - >> [{:}]", &sysinfo.selected_pid))
        )
        .row_highlight_style(Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .bold()
        )
        .highlight_symbol(">> ");
        
    frame.render_stateful_widget(table, horizontal_chunks[0], &mut app.table_state);
    // 创建滚动条状态
    
    if let Some(selected) = app.table_state.selected() {
        app.scrollbar_state = app.scrollbar_state
            .content_length(total_rows)
            .position(selected);
    } else if total_rows > 0 {
        app.table_state.select(Some(0));
        app.scrollbar_state = app.scrollbar_state
            .content_length(total_rows)
            .position(0);
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("+"))
        .end_symbol(Some("+"))
        .thumb_symbol("█")
        .track_symbol(Some("│"));
    // 渲染滚动条（使用 inner 可以留出边距）
    frame.render_stateful_widget(
        scrollbar,
        horizontal_chunks[1],
        &mut app.scrollbar_state,
    );
    
    // 状态栏
    if let Some(selected_index) = app.table_state.selected() {
        let target_proc = &sysinfo.proc_info.procs[selected_index];
        sysinfo.selected_pid = target_proc.pid;
    }

    // 获取光标位置

    
/*     let status = Paragraph::new(
        std::format!(
            "Press 'q' to quit | ↑/↓,j/k to navigate | 'd' delete proc | choose -> {:} input: {}",
            &sysinfo.selected_pid,
            app.user_input.value()
        ))
        .block(Block::default().borders(Borders::TOP));
    
    frame.render_widget(status, chunks[2]);  */

}

fn user_input_render(chunks: &mut Rc<[Rect]>, frame: &mut Frame, app: &mut App, sysinfo: &mut SystemInfo) {
    let mut status = Paragraph::new(
        std::format!(
            "Press 'q' to quit | ↑/↓,j/k to navigate | 'd' delete proc | choose -> {:}",
            &sysinfo.selected_pid,
        ))
        .block(Block::default().borders(Borders::TOP));
    
    let cursor_char = if (app.time_cyclic >> 2) & 1 == 0  {  '|' } else { ' ' };

    match app.input_mode {
        InputMode::Find => {
            status = Paragraph::new(std::format!(
                "input > {}{}",
                app.user_input.value(),
                cursor_char
            ))
            .block(Block::default().borders(Borders::TOP));
        }

        InputMode::Command => {
            status = Paragraph::new(std::format!(
                "command : {}{}",
                app.user_input.value(),
                cursor_char
            ))
            .block(Block::default().borders(Borders::TOP));
        }
        _ => {}
    }
    
    // if app.input_mode {
        // status = Paragraph::new(std::format!(
            // "INPUT > {}▮",
            // app.user_input.value()
        // ))
        // .block(Block::default().borders(Borders::TOP));
    // }
    
    frame.render_widget(status, chunks[2]); 
}




