// use std::os::windows::process;
use crossterm::{
    event::{self, Event}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use ratatui::{
    Terminal, backend::CrosstermBackend
};

mod tmgr;
use tmgr::{SystemInfo };

mod render;

mod app;
use app::App;

use crate::render::main_render;

// mod render;
fn main() -> std::io::Result<()> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new();
    let mut sysinfo = SystemInfo::new();

    // let mut render = Render::new(frame)
    
    // 主循环
    loop {
        // terminal.draw(|frame| main_render(frame, &mut app, &mut sysinfo))?;
        if !app.is_running {
            break;
        }

        terminal.draw(|frame| {
            main_render(frame, &mut app, &mut sysinfo);
        })?;
        
        // 处理退出
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // 只在按键释放的时候触发
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                app.solve_keycode(key.code, &mut sysinfo);
            }
        }
    }
    
    // 清理
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
