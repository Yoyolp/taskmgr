use std::default;

use sysinfo::{Pid, ProcessStatus, System, Networks};                   // 系统信息获取

pub struct SystemInfo
{
    pub sys: System,
    
    pub cpu_usage: f32,         // cpu 使用率

    pub total_mem: f64,        // 单位 GB
    pub used_mem: f64,         // 单位 GB
    pub mem_percent: f64,      // 内存使用率

    pub cpu_usage_points: Vec<(f32, f32)>, // cpu 使用率的映射集合
    pub mem_percent_points: Vec<(f64, f64)>, // 内存 使用率的映射集合

    pub network_info: NetworkInfo,
    
    // 选择 pid 
    // 线程相关
    pub proc_info: ProcessInfo,
    pub selected_pid: u32,
}

impl SystemInfo {
    pub fn new() -> Self{
        let sys = System::new();
        let mut info = Self { 
            sys: sys,
            
            cpu_usage: 0.0,
            
            total_mem: 0.0,
            used_mem: 0.0,
            mem_percent: 0.0,

            cpu_usage_points: vec![(1.0, 0.0); 10],
            mem_percent_points: vec![(1.0, 0.0); 10],
                
            // 网络设置
            network_info: NetworkInfo::new(),

            // 进程选择
            proc_info: ProcessInfo::uninit(),
            selected_pid: 0,

        };

        info.refresh();
        info
    }
    
    // 更新信息
    pub fn refresh(&mut self) {
        // 刷新内存信息
        self.sys.refresh_all();
        
        // 跟新进程信息      =================  待优化
        let proc_count = self.sys.processes().len();
        let procs: Vec<&sysinfo::Process> = self.sys
            .processes()
            .values()
            .collect();

        let mut res: Vec<ProcessInfoItem> = vec![];
        for (_, proc) in procs.iter()
                .take(proc_count.try_into().unwrap())
                .enumerate() {
            
            let name = proc.name().to_string_lossy();
            let name = if name.len() > 35 { &name[..35] } else { &name };
            let pid = proc.pid().as_u32();
            let cpu  = proc.cpu_usage();
            let mem = proc.memory() as f64 / 1024.0 / 1024.0;

            res.push(ProcessInfoItem { 
                name: String::from(name), 
                pid, 
                cpu, 
                mem: mem,
                status: proc.status()
            });
        }
        self.proc_info.procs = res;
        // TODO
        self.proc_info.procs.sort_by(|a, b| {
            b.mem.partial_cmp(&a.mem).unwrap()
        });

        // 获取 CPU 使用率
        self.cpu_usage = self.sys.global_cpu_usage();
        
        // 循环左移1 位, CPU数据集合更新
        let points_len = self.cpu_usage_points.len();
        self.cpu_usage_points.rotate_left(1);
        self.cpu_usage_points[points_len - 1] = (1.2, self.cpu_usage / 100.0);
        self.cpu_usage_points
            .iter_mut()
            .for_each(|item| {
            item.0 -= 0.2
        });
        
        // MEM 数据集合更新
        let points_len = self.mem_percent_points.len();
        self.mem_percent_points.rotate_left(1);
        self.mem_percent_points[points_len - 1] = (1.2, self.mem_percent / 100.0);
        self.mem_percent_points
            .iter_mut()
            .for_each(|item| {
            item.0 -= 0.2
        });

        // 获取内存信息
        self.used_mem = self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        self.total_mem = self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        if self.total_mem > 0.0 {
            self.mem_percent = (self.used_mem / self.total_mem) * 100.0;
        }
        
        // 更新网络信息
        let names = network_interface_name();
        self.network_info.get_network_speed(&names[0]);
        // self.network_info.get_network_speed();
        

    }

    // pub fn processes(&self) -> Vec<&sysinfo::Process> {
    //     self.sys.processes().values().collect()
    // }

    // 总进程数 (包括所有状态的进程)  
    // pub fn total_process_count(&self) -> usize {
    //     self.sys.processes().len()
    // }
   
    // 通过 PID 终止进程
    pub fn stop_proc_by_pid(&mut self, pid: u32) -> Result<bool, String> {
        let pid = Pid::from_u32(pid);
        match self.sys.process(pid) {
            Some(process) => Ok(process.kill()),
            None => Err(format!("进程不存在 {}", pid)) 
        }
    }

    // 根据 pid 查找 进程索引
    pub fn find_processes_by_name(&self, pattern: &str, case_sensitive: bool) -> Vec<usize> {
        let pattern = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        self.proc_info.procs
            .iter()
            .enumerate()
            .filter(|(_, proc)| {
                let proc_name = if case_sensitive {
                    proc.name.clone()
                } else {
                    proc.name.to_lowercase()
                }; 
                proc_name.contains(&pattern)
            })
            .map(|(idx, _)| idx)
            .collect()

    }

    // 根据 pid 查找进程索引
    pub fn find_process_index_by_pid(&self, pid: u32) -> Option<usize> {
        self.proc_info.procs
            .iter()
            .position(|proc| proc.pid == pid)
    }

    // 获取进程名字
    pub fn get_process_name(&self, index: usize) -> Option<&str> {
        self.proc_info.procs.get(index).map(|p| p.name.as_str())
    }

    // 检查是否存在匹配的进程
    pub fn has_matching_processes(&self, pattern: &str) -> bool {
        !self.find_processes_by_name(pattern, false).is_empty()
    }


}

pub struct ProcessInfoItem {
    pub name: String,
    pub pid: u32,
    pub cpu: f32,
    pub mem: f64,
    pub status: ProcessStatus
}

pub struct ProcessInfo {
    pub procs: Vec<ProcessInfoItem>    
}

impl ProcessInfo {
    // pub fn new(sys: &SystemInfo, n: usize) -> Self {
    //     let procs = sys.processes();
        
    //     let mut list :Vec<ProcessInfoItem> = Vec::new();
    //     for (_, proc) in procs.iter()
    //             .take(n.try_into().unwrap())
    //             .enumerate() {
            
    //         let name = proc.name().to_string_lossy();
    //         let name = if name.len() > 35 { &name[..35] } else { &name };
    //         let pid = proc.pid().as_u32();
    //         let cpu  = proc.cpu_usage();
    //         let mem = proc.memory() as f64 / 1024.0 / 1024.0;

    //         list.push(ProcessInfoItem { 
    //             name: String::from(name), 
    //             pid, 
    //             cpu, 
    //             mem: mem,
    //             status: proc.status()
    //         });
    //     }  
    //     Self { procs: list }
    // }

    pub fn uninit() -> Self {
        Self {
            procs: vec![]
        }
    }
    
    // pub fn find_by_pid(&self, pid: u32) -> Option<&ProcessInfoItem> {
    //     // for item in &self.procs {
    //     //     if item.pid == pid {
    //     //         return Some(item);
    //     //     }
    //     // }
    //     // None

    //     self.procs.iter().find(|item | {
    //         item.pid == pid
    //     } )
    // }
}

// 流量获取统计
pub struct NetworkInfo {
    // 存储上一次刷新的数据， 用于计算差值
    pub prev_rx: u64,
    pub prev_tx: u64,

    pub network_name: String
}

impl NetworkInfo {
    pub fn new() -> Self {
        let names = network_interface_name();
        let default_name = if names.is_empty() {
            String::from("unkown")
        } else {
            names[0].clone()
        };

           
        Self { 
            prev_rx: 0, 
            prev_tx: 0,
            network_name: default_name    // 暂时
        }
    }
    
    pub fn get_network_speed(&mut self, interface_name: &str) -> Option<(u64, u64)> {
        // 获取最新的网络数据
        let networks = Networks::new_with_refreshed_list();


        let (_, network_data) = match networks
            .iter()
            .find(|(name, _)| *name == interface_name) {
            
            Some(data) => data,
            None => {
                // 尝试使用第一个网卡
                if let Some(first) = networks.iter().next() {
                    first
                } else {
                    return None;
                }
            }
        };

        let curr_rx = network_data.total_received();
        let curr_tx = network_data.total_transmitted();
        
        let speed_down = curr_rx.saturating_sub(self.prev_rx);
        let speed_up = curr_tx.saturating_sub(self.prev_tx);
        
        self.prev_rx = curr_rx;
        self.prev_tx = curr_tx;
        
        Some((speed_down, speed_up))
    }

    // 直接获取某个网卡的累计数据总量
    // pub fn get_total_transfer(interface_name: &str) -> Option<(u64, u64)> {
    //     let networks = Networks::new_with_refreshed_list();
    //     let (_, data) = networks.iter()
    //         .find(|(name, _)| *name == interface_name)?;
    //     Some((data.total_received(), data.total_transmitted()))
    // }

}

// 获取网卡
fn network_interface_name() -> Vec<String> {
    let networks = Networks::new_with_refreshed_list();
    let mut res: Vec<String> = vec![];
    
    for (name, _) in &networks {
        res.push(name.to_string());    
    }
    res
}


