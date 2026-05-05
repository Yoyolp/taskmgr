// use std::os::windows::process;
use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use ratatui::{
    Terminal, backend::CrosstermBackend
};

mod tmgr;
use tmgr::{SystemInfo };

mod render;
use render::Render;

mod app;
use app::App;

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
        terminal.draw(|frame| {
            let render = Render::new(frame);
            render.main_render(frame, &mut app, &mut sysinfo);
        })?;
        
        // 处理退出
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // 只在按键释放的时候触发
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                
                match key.code {
                    KeyCode::Char('q') => break,
                    
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Char('k') => app.scroll_up(),

                    KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('j') => app.scroll_down(),

                    KeyCode::Char('d') => {
                        let pid = sysinfo.selected_pid;
                        let _ = sysinfo.stop_proc_by_pid(pid);
                    },
                    
                    _ => {}
                }
            }
        }
    }
    
    // 清理
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
